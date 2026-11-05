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
  第四段写 cargo：经 Verus 安装里的 cargo-verus 起 cargo verus focus -p <包> --offline -- <参数>（第二轮正推 Y4 第 2 条），副本是 crate 连同它经 path 连到的 crate、
    仓根的 Cargo.lock 与一份只列这几个成员的工作区 Cargo.toml（整个工作区的 cargo metadata 在离线机器上过不了）；几处同时验各占一个编译目录，vstd 与依赖在里面只编一次。
  两种都是一处判定一份新副本：cargo 的编译指纹不看 --verify-* 参数，同一个目录里换参数再验拿到的是上一趟的结果、stdout 没有判定行（第二轮正推 Y4 第 3 条，
    2026-10-26 在草稿的小工作区里复现过）；没有判定行的那一处判红，不当绿。
不带 --no-cheating（出口 5 第三轮判决第三节「门禁」）：assume_specification、external_body、uninterp 与 external_*_specification 照信任清单
  .claude/gate.d/verus-trust-list.tsv 逐条登记（没登记、规格原文的 sha256 对不上、差分测试不在都判红），admit(…)、assume(…)、verifier::external、axiom 在核里一律判红，
  都在起 Verus 之前判（region_problems、cheating_markers）。
--rlimit 取登记表 rlimit 行；没有这一行时取 DEFAULT_RLIMIT，就是 Verus 0.2026.09.20 自己的默认 10。proof、vacuity、must-red-worlds 三格同一个数，各 crate 同一个数。
判定取 stdout 的 JSON（verification-results 的 verified / errors / encountered-vir-error，verus 的 commit），报错位置取 stderr（直接起的相对 crate 目录，cargo 那一种相对副本根）。
只验一个函数（证明逐项、空真每处）时按那个函数所在的核点模块（kernel_module）：--verify-function 只在 --verify-root 或 --verify-only-module 点的那一个模块里找。
  核是 crate 的入口文件（Cargo.toml 的 lib.path，默认 src/lib.rs）的点 --verify-root；入口所在目录下的 <名>.rs 与 <名>/mod.rs 点 --verify-only-module <名>，
  更深的用 :: 接起来（<甲>/<乙>.rs 与 <甲>/<乙>/mod.rs 是 <甲>::<乙>）。Verus 0.2026.09.20 的认法（2026-10-18 在草稿的三层小 crate 上量的）：
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
      在 crate 的 src 里 verus! 外面有 impl <Trait> for <类型> 的判红（cfg(test) 下的不判）。
      登记式放过（.claude/gate.d/verus-trait-impls-outside-lag.tsv，格式写在表头，只缩不涨）：表里一行（trait 全路径、实现类型、所在文件、哪一簇会把它挪进 verus!、
      为什么此刻放过）对上的 verus! 外面的 impl 不判红，成功句逐个点名「<trait> for <类型>（<文件>:<行>，契约没证、靠信任，归「<簇>」挪进 verus!）」、写这一格不算绿到底；
      表里这个 crate 的行，那个 impl 已挪进 verus! 或在那份文件里找不到的判红（删行）；impl 还在外面而它的 trait 与类型此刻都不在被验区域里的
      （那一簇的补丁还没打上）不红、没放过什么，成功句点名；trait 全路径与被验的那个不同判红；表与 gate 基准那一版比
      （.claude/gate.d/lib-lag-baseline.py 的 gate_base_of、baseline_rows，键是 trait 全路径、实现类型、所在文件），基准里没有的行判红，
      被判的根不在 git 仓里、基准里还没有这张表时不比、成功句写明没比，git 取不到判红。
      external_body 的规格原文取签名到函数体的 { 之前（body_open：单元是半开区间，最后一个词是 } 才有函数体），一个 impl 里最后一个 fn 也取得到签名，改签名 sha256 就变。
      uninterp、external_*_specification、带 external_body 的类型规格与 cfg 原子（规格原文是它所在的条目）取整个条目，也照半开区间 [first_token, last_token) 取，
      不连下一个条目的头一个词（whole_entry_end，几样同一种取法）。
      放过的每一处另往 $GATE_NOT_RUN_FILE 追加一行本次未跑（门禁汇总的「本次未跑」收它；这个变量不在时不写，单跑与喂样本时它不在），成功句报追加了几行。
      Verus 逐项验：核的被验区域里有证明义务的单元各一项（proof_items）：proof fn、exec fn、带 decreases 的 spec fn（终止义务）、
      derive(Clone) 的类型（Verus 验它生成的 <类型>::clone）；impl 与 trait 里的方法照样成项；不带 decreases 的 spec fn、trait 里没有函数体的声明、external_body 的 fn
      没有证明义务、不成项，随用到它的项进键（没有函数体按切出的单元认：单元以 ; 收尾，ensures 里的结构体字面量不算函数体）。另两类不是 fn、Verus 整 crate 验时照样各算条数，各成一项（里程碑四出口 5 规格底座，core 登记 16 条里有这 3 条）：
      verus! 里带值的 const（impl 里的关联 const 与 spec const 也算：真 Verus 整 crate 验时 spec const 各 1 条、点得中名；没写值的不成项）与 assume_specification。
      项名是全路径 proof:<crate>::<模块>::<名>（第二轮正推 Y4 第 7 条；核是 crate 入口的没有模块那一段），
      <名> 是交给 --verify-function 的那一个：自由 fn 写它的名字，impl 里的方法写 <类型>::<方法>（trait impl 也写类型），trait 里的写 <trait>::<方法>，clone 写 <类型>::clone；
      带值的 const 项名写 <类型或 trait>::<名>（自由的写 <名>）；交给 --verify-function 的：自由 const 写 <名>，trait 里带默认值的写 <trait>::<名>，
      impl 里的关联 const（固有 impl 与 trait impl 都算）写 *::<名>（Verus 把它列成 impl&%<号>::<名>，号数这个模块里的全部 impl，verus! 外面的与 derive 生成的也数，
      从源码算不准；*::<名> 是「以 ::<名> 收尾」，不会点到以 <名> 起头的自由 const）；assume_specification 项名写 assume_specification[<点名的条目>]，交给 --verify-function 的是那个条目（只有标识符与 :: 的照写，
      例 singlefs_format::SLOT_BYTES；别的取最后一个标识符，例 <[T]>::binary_search 交 binary_search）。
      起 Verus 之前（列项也算）每一项的点名照 Verus 的点法在它那个模块的可点名函数表里点一遍（point_name_problems、verus_listing、verus_function_matches）：
      不带 * 的先找恰好同名的、没有再找包含它的，*<串> 是以它收尾、<串>* 是以它起头、*<串>* 是包含它，带 * 的点中几个 Verus 就一起验几个；
      命中不是恰好一项的判红并点名命中的那几项，出路是改名。Verus 只在 --verify-root 或 --verify-only-module 点的那一个模块里找，表按核（一个核一个模块）各列各的，
      不同模块里同名的不撞。点名规则是 2026-10-29 用 Verus 0.2026.09.20 在探针 crate 上现量的：<类型>::<关联 const>、<模块>::<类型>::<名>、
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
      本机 2026-10-26 在小 crate 上量到 1.8 GB 对 1.0 GB）里都有、整段逐字相同，对不上判红（第三轮判决 Z2：宏写法产品构建返回 11、verus --compile 编出返回 12）。
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
      里面闭包的那一处就验得过了（2026-10-01 在草稿里实测：七处一起插只报六处）。
  verus-proof-check.py coverage <根>
      静态检查每个带契约的函数与 invariant 块都有唯一锚点负例；不运行 Verus。
  verus-proof-check.py must-red-worlds <根> [通用选项]
      必红世界：登记表 must-red-worlds 行点名的表（六段制表符分隔：变异名、文件、原文、替换文、proof_errors=M、landing=<落点>；\\n 表换行），
      每行原文在文件里恰好命中一次，改坏的副本上 Verus 要出 JSON、errors ≥ 1，证明错的落点等于第六段登记的落点。一行都没有判红。
      判的是落点，不是条数（C654：条数会随求解环境、无关的 spec 改动漂，攻方在核上加一个没人用的 spec fn，同一行就从 2 条跳到 3 条）：
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
              由 task-server 按记录给、撞了放大重排（task-server/README.md「提交写法」）；弄坏开关 outer-memory-narrows-jobs 下照改前的样子设了它就收到 1 处
            每次 Verus 直接起：不套内存包装、capped.sh 与 nice，不设每个进程的内存上限，也不限时（用户定案「工具里不限时」）：卡住由 task-server 判
              （日志满 10 分钟不涨、CPU 也几乎不涨，或跑过记录用时 5 倍而日志不涨，停掉记 stalled，task-server/README.md），Verus 没出结局就没有判定，不当通过；
              放进它自己的进程组（start_new_session），退了之后组里还留着的（rust_verify、z3 这类）一并 KILL，stdout 与 stderr 边跑边收
              （finish_process_group；弄坏开关 group-leftovers-kept 下不停留下的，它拿着输出管道时判法一直等到它自己退）
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
      记绿、记红、撤绿（用户 2026-10-29 定案）：判绿的照键记绿；判红的（Verus 报错）照同一把键调共用库的 record_red 记红，
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
        「✗ trait-impl-lag-registered-green 这一格判错」、「✗ trait-impl-lag-baseline-green 这一格判错」、「✗ proof-trait-declaration-struct-literal 这一格判错」
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
                                                     （改前的样子：core 表每行必红世界多一条、落点多一个（无位置）；假 cargo-verus 照真 cargo 打这一句）
      cargo-mode-must-red（「✗ cargo-mode-must-red 这一格判错」：落点多出（无位置））
  VERUS_PROOF_CHECK_BREAK=const-fn-heading-unrecognized  空真插入点的说明（FUNCTION_LINE）不认 const fn（改前的样子：core 的 to_device_offset 那一处写成「fn first_file_byte」）
      vacuity-const-fn-label（「✗ vacuity-const-fn-label 这一格判错」：说明写成 fn keep_small）
逐项点名与按条复用的几项：
  VERUS_PROOF_CHECK_BREAK=default-install-by-home  不设 --verus-root 与 SINGLEFS_VERUS_ROOT 时，Verus 安装目录照改前取 ${XDG_DATA_HOME:-~/.local/share}/<登记表 release 行第五列>（落进各台自己的家目录）
      default-install-not-by-home（「✗ default-install-not-by-home 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=list-items-pipe-unhandled  list-items 不照 SIGPIPE 惯例退（接 | head 时打 BrokenPipeError 的回溯、退 1）
      list-items-closed-pipe（「✗ list-items-closed-pipe 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=item-filter-ignored  --item 不起作用、照判全部（写错的项名也不退 2）
      item-select-one、item-unknown、item-proof-one、item-proof-unknown（『… 这一格判错』）
      红的格：「✗ item-select-one 这一格判错」、「✗ item-unknown 这一格判错」、「✗ item-proof-one 这一格判错」、「✗ item-proof-unknown 这一格判错」、
        「✗ multi-crate-item-routed 这一格判错」
  VERUS_PROOF_CHECK_BREAK=reuse-key-whole-crate  被验函数那一样换回核 crate 整个目录的原文摘要（旧判法：整 crate 一把）
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
  VERUS_PROOF_CHECK_BREAK=vacuity-key-by-line-number  空真每处的键另带核的路径与 requires 的行号（旧判法）
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
  VERUS_PROOF_CHECK_BREAK=red-keeps-green  判红的那一项不记红（不调共用库的 record_red），同一把键原有的绿留着照复用（改前的样子）
      reuse-red-withdraws-green（先判绿入库，同一把键只写不读地判红一趟，下一趟那一项要现判：「✗ reuse-red-withdraws-green 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=proof-item-narrows  证明那一格同时收到 proof 与 proof:<名> 时只验点名的那一个、不比之和（改前的样子：--item proof --item proof:X 丢了整格）
      proof-whole-kept（「✗ proof-whole-kept 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=reuse-key-without-crate-build  键里不带 crate 起法（cargo 还是直接起、Cargo.toml 的 edition 与包名）（改前的样子）
      reuse-key-crate-build（edition 2021 / 2024、cargo 起法三种要三把键：「✗ reuse-key-crate-build 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=reuse-key-without-closure-dependencies  键里不带闭包真点名的外部 crate 在 Cargo.lock 里的版本（改前的样子）
      reuse-key-closure-dependencies（Cargo.lock 里 vstd 换版本，点名 vstd:: 的那一项要换键：「✗ reuse-key-closure-dependencies 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=reuse-key-without-entry-unit  证明与空真的键里不带「被验入口单元」「所属函数单元」（改前的样子：一个 trait impl 里几个方法闭包相同、共用一把键）
      reuse-proof-entry-unit-own-key（第二趟 fill 复用到 width 那一份，之和 4 与登记的 5 不同：「✗ reuse-proof-entry-unit-own-key 这一格判错」）、
      reuse-vacuity-owner-unit-own-key（后写的绿盖掉空真那一处的红、第二趟复用成绿：「✗ reuse-vacuity-owner-unit-own-key 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=assume-specification-refused  assume_specification 那一项一律拒算、不入库（改前的样子）
      reuse-assume-specification-three-questions（第二趟两条 assume_specification 照样现判，① 红：「✗ reuse-assume-specification-three-questions 这一格判错」）、
      reuse-assume-specification-outside-refused（拒算行不再点名 assume-specification-target：「✗ reuse-assume-specification-outside-refused 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=assume-specification-target-unkeyed  assume_specification 点名的条目不进键
      reuse-assume-specification-three-questions（改 SLOT_WIDTH 的值，指它的那一项照样复用，③ 红：「✗ reuse-assume-specification-three-questions 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=toolchain-key-with-paths  键里工具链那一样照 tool-versions 打的整行，带 RUSTUP_HOME、rustup、VERUS_ROOT、VERUS_Z3_PATH 的路径（改前的样子）
      reuse-key-toolchain-without-paths（同内容的工具链与 rustup 换个位置键要不变：「✗ reuse-key-toolchain-without-paths 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=threads-default-cpu-count  --threads 的缺省取核数（改前的样子），不取登记的重活默认并行度
      threads-default-registered（「✗ threads-default-registered 这一格判错」）
  证明成哪几项、每项怎么判（proof_items、verification_clean、check_proof；假 Verus 照真 Verus 的形态出 JSON，自证 fake-verus-real-shape 盯着它）：
  VERUS_PROOF_CHECK_BREAK=spec-fn-as-item  不带 decreases 的 spec fn 也成项（改前的样子：真仓 90 项里 49 个 spec fn 验出 0 条）（样本 verus-must-red-worlds-list-items 判错）
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
  VERUS_PROOF_CHECK_BREAK=valued-const-not-item  verus! 里带值的 const 不成项（改前的样子：core 的 DataUnitIndexInFile::FIRST 那一条没人验，之和少 1）
      proof-valued-const-and-assume-specification-items（「✗ proof-valued-const-and-assume-specification-items 这一格判错」：之和 3 与登记的 5 不同）
      连带红的格：「✗ list-items-associated-const-prefix 这一格判错」、「✗ proof-associated-const-prefix-green 这一格判错」、「✗ proof-associated-const-point-names 这一格判错」、
        「✗ proof-point-name-trait-default-collision-red 这一格判错」、「✗ proof-point-name-two-impls-collision-red 这一格判错」、「✗ proof-spec-const-items 这一格判错」
        （2026-10-29 逐个开关扫一遍现量的；spec const 也是带值的 const）
  VERUS_PROOF_CHECK_BREAK=assume-specification-not-item  assume_specification 不成项（改前的样子：core 两条格式常量规格各 1 条没人验，之和少 2）
      proof-valued-const-and-assume-specification-items（「✗ proof-valued-const-and-assume-specification-items 这一格判错」：之和 4 与登记的 5 不同）
      连带红的格：「✗ trust-registered-green 这一格判错」、「✗ no-cheating-dropped 这一格判错」
  VERUS_PROOF_CHECK_BREAK=require-success-key-per-function  --verify-function 下 JSON 没有 success 键当没验过（改前的样子：真仓 90 项全红）
      proof-fn-verified-one-green（「✗ proof-fn-verified-one-green 这一格判错」）；proof-green、vacuity-vacuous 等跟着判错
      连带红的格：「✗ proof-green 这一格判错」、「✗ proof-count 这一格判错」、「✗ vacuity-vacuous 这一格判错」、「✗ vacuity-closure-vacuous 这一格判错」、「✗ proof-rlimit-registered 这一格判错」、
        「✗ proof-rlimit-default 这一格判错」、「✗ item-proof-one 这一格判错」、「✗ reuse-proof 这一格判错」、「✗ reuse-comment-in-function-body 这一格判错」、
        「✗ reuse-proof-function-only 这一格判错」、「✗ reuse-spec-function-all-users 这一格判错」、
        「✗ proof-sum-from-reused-records 这一格判错」、「✗ module-per-kernel 这一格判错」、「✗ proof-spec-fn-not-item 这一格判错」、
        「✗ proof-entire-crate-false-not-red 这一格判错」、「✗ proof-spec-decreases-item 这一格判错」、「✗ proof-derived-clone-item 这一格判错」、
        「✗ proof-sum-equals-registered-green 这一格判错」、「✗ proof-sum-short-red-named 这一格判错」、「✗ proof-zero-verified-red 这一格判错」、
        「✗ list-items-key-mismatch-count 这一格判错」、「✗ reuse-must-red-table-row-only 这一格判错」、「✗ reuse-must-red-table-comment-only 这一格判错」、
        「✗ reuse-crate-other-file-no-cells 这一格判错」、「✗ reuse-three-questions 这一格判错」
        「✗ trait-impl-lag-registered-green 这一格判错」、「✗ trait-impl-lag-baseline-green 这一格判错」、「✗ proof-trait-declaration-struct-literal 这一格判错」
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
        「✗ trait-impl-lag-registered-green 这一格判错」、「✗ trait-impl-lag-baseline-green 这一格判错」、「✗ proof-trait-declaration-struct-literal 这一格判错」
        「✗ proof-assert-by-requires-skip-reported 这一格判错」、「✗ proof-spec-const-items 这一格判错」
  VERUS_PROOF_CHECK_BREAK=zero-verified-accepted  点的函数一条义务都没验（verified 0）也判绿
      proof-zero-verified-red（「✗ proof-zero-verified-red 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=nonzero-errors-accepted  errors 不是 0 也当验过（不是 vir 错的）
      proof-errors-one-red（「✗ proof-errors-one-red 这一格判错」）、proof-broken
      连带红的格：「✗ proof-broken 这一格判错」、「✗ proof-rlimit-too-low 这一格判错」
  VERUS_PROOF_CHECK_BREAK=submodule-verify-root  只验一个函数时不按核所在的模块点，一律 --verify-root（改前的样子：真仓子模块 compaction_plan_proofs 的 20 项证明、9 处空真全红）
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
        「✗ trait-impl-lag-registered-green 这一格判错」、「✗ trait-impl-lag-baseline-green 这一格判错」、「✗ proof-trait-declaration-struct-literal 这一格判错」
        「✗ list-items-assert-by-requires-skipped 这一格判错」、「✗ vacuity-assert-by-requires-skipped 这一格判错」、「✗ proof-assert-by-requires-skip-reported 这一格判错」、
        「✗ proof-spec-const-items 这一格判错」
  VERUS_PROOF_CHECK_BREAK=key-mismatch-by-item-only  list-items 数键时这一项判绿过就算键相同，不比键
      list-items-key-mismatch-count（「✗ list-items-key-mismatch-count 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=item-key-lines-skipped  list-items --key-mismatches 不逐项打 ITEM_KEY 行
      list-items-item-key-lines（「✗ list-items-item-key-lines 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=key-items-ignored  list-items --key-mismatches 带 --item 也比整表
      list-items-item-key-with-item（「✗ list-items-item-key-with-item 这一格判错」）
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
  VERUS_PROOF_CHECK_BREAK=body-open-past-unit-end  external_body 的规格原文照改前的取法：把单元的 last_token 当最后一个词（一个 impl 里最后一个 fn 取成空串、
                                                   改签名 sha256 不变；别的 fn 取成整个 fn 再多一个词）（样本 verus-proof-trust-digest-last-fn-in-impl-red 判绿）
      trust-digest-signature-only（「✗ trust-digest-signature-only 这一格判错」：impl 里最后一个 fn 是 e3b0c442… 那一个）
  VERUS_PROOF_CHECK_BREAK=trait-declaration-by-brace  trait 里的声明照改前的认法：单元里一个 { 都没有才不成项（ensures 里的结构体字面量当成函数体）
                                                      （样本 verus-proof-list-items-trait-declaration-struct-literal 判错：Store::put 列成了一项）
      proof-trait-declaration-struct-literal（「✗ proof-trait-declaration-struct-literal 这一格判错」：Store::put 成了项、之和 4 与登记的 3 不同）
  VERUS_PROOF_CHECK_BREAK=trait-impl-lag-matched-by-trait-only  放过表的行只按所在文件与 trait 名认、不看实现类型（同一个 trait 在外面的别的 impl 也被放过）
                                                                （样本 verus-proof-trait-impl-outside-unregistered-red 判绿）
      trait-impl-lag-unregistered-red（「✗ trait-impl-lag-unregistered-red 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=trait-impl-lag-stale-unchecked  放过表里已挪进 verus! 或找不到的行不判（样本 verus-proof-trait-impl-outside-moved-red 判绿）
      trait-impl-lag-stale-rows-red（「✗ trait-impl-lag-stale-rows-red 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=trait-impl-lag-growth-unchecked  放过表不与 gate 基准那一版比（涨了也不红，成功句写「没与 gate 基准比」）
      trait-impl-lag-baseline-green（「✗ trait-impl-lag-baseline-green 这一格判错」）、trait-impl-lag-grown-red（「✗ trait-impl-lag-grown-red 这一格判错」）
      连带红的格：「✗ trait-impl-lag-registered-green 这一格判错」
  VERUS_PROOF_CHECK_BREAK=trait-impl-lag-passed-unnamed  放过的 impl 在成功句里只报个数、不逐个点名「契约没证、靠信任」
                                                       （样本 verus-proof-trait-impl-outside-registered-green 判错）
      trait-impl-lag-registered-green（「✗ trait-impl-lag-registered-green 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=trait-impl-lag-not-run-unreported  放过的 impl 不往 $GATE_NOT_RUN_FILE 报本次未跑（改前的样子：只写在成功句里）
      trait-impl-lag-not-run-reported（「✗ trait-impl-lag-not-run-reported 这一格判错」：文件里 0 行）
  VERUS_PROOF_CHECK_BREAK=mir-panic-path-unnormalized  MIR 比对不照规整表 MIR_PATH_ALIASES 改写（改前的样子：core 103 个条目红在 std::rt::panic_fmt 对 core::panicking::panic_fmt）
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
  VERUS_PROOF_CHECK_BREAK=whole-entry-past-unit-end  uninterp、external_*_specification、带 external_body 的类型规格与 cfg 原子照改前的取法多取下一个条目的头一个词
      trust-digest-whole-entry-half-open（「✗ trust-digest-whole-entry-half-open 这一格判错」）
      连带红的格：「✗ trust-digest-cfg-entry-half-open 这一格判错」
  VERUS_PROOF_CHECK_BREAK=cfg-entry-past-unit-end  只 cfg 原子的「所在条目整个」照改前的取法多取下一个条目的头一个词
      trust-digest-cfg-entry-half-open（「✗ trust-digest-cfg-entry-half-open 这一格判错」：后接 pub 与后接 fn 的 sha256 不同）
  VERUS_PROOF_CHECK_BREAK=associated-const-bare-name  impl 里的关联 const 交给 --verify-function 的是 <名>（改前的样子），不是 *::<名>
                                                      （样本 verus-proof-list-items-associated-const-prefix 判错：列项退 1）
      list-items-associated-const-prefix（「✗ list-items-associated-const-prefix 这一格判错」：FIRST 在表里命中 2 项）
      连带红的格：「✗ proof-associated-const-prefix-green 这一格判错」、「✗ proof-associated-const-point-names 这一格判错」、
        「✗ proof-point-name-trait-default-collision-red 这一格判错」、「✗ proof-point-name-two-impls-collision-red 这一格判错」（红句点名的是 LIMIT、FIRST，不是 *::<名>）
  VERUS_PROOF_CHECK_BREAK=point-names-by-containment  点名不照 Verus 的点法在可点名函数表里点，照改前的判法只比成项的关联 const 与 assume_specification 谁是谁的一段
                                                      （样本 verus-proof-list-items-trait-default-collision-red 判错：列项退 0）
      proof-point-name-trait-default-collision-red（「✗ proof-point-name-trait-default-collision-red 这一格判错」：找不到「命中 2 项」）
      连带红的格：「✗ proof-point-name-two-impls-collision-red 这一格判错」（红句是改前那一句，不点名撞的几项）
  VERUS_PROOF_CHECK_BREAK=assert-by-requires-as-insertion-point  属于 assert(…) by (…) 的 requires 照改前当空真插入点（往下找不到函数体的 { 判红）
                                                                 （样本 verus-vacuity-list-items-assert-by-requires-green 判错：列项退 1）
      list-items-assert-by-requires-skipped（「✗ list-items-assert-by-requires-skipped 这一格判错」）、vacuity-assert-by-requires-skipped（「✗ vacuity-assert-by-requires-skipped 这一格判错」）、
      proof-assert-by-requires-skip-reported（「✗ proof-assert-by-requires-skip-reported 这一格判错」：成功句报跳过 0 处）
  VERUS_PROOF_CHECK_BREAK=assert-by-requires-by-semicolon-line  照「往下先碰到一行只有 ; 」认 assert-by 的 requires，不看前面接的是不是 assert(…) by (…)
                                                                （样本 verus-vacuity-list-items-requires-on-plain-block-red 判错：列项退 0）
      list-items-requires-on-plain-block-red（「✗ list-items-requires-on-plain-block-red 这一格判错」）、
      list-items-requires-on-assert-without-by-red（「✗ list-items-requires-on-assert-without-by-red 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=spec-const-not-item  spec const 不成项（改前的样子：真 Verus 整 crate 验时每个 spec const 多 1 条，之和少）
                                               （样本 verus-proof-list-items-spec-const 判错：成项 3 个，找不到两个 spec const 那两行）
      proof-spec-const-items（「✗ proof-spec-const-items 这一格判错」：之和 2 与登记的 4 不同）
  VERUS_PROOF_CHECK_BREAK=outer-memory-narrows-jobs  环境里设了外层内存上限（RUN_WITH_MEMORY_CAP_OUTER_CAP）就把几处同时验收到 1 处（改前按外层上限 ÷ 每个 Verus 进程的上限收的样子）
      jobs-not-narrowed-by-outer-memory（「✗ jobs-not-narrowed-by-outer-memory 这一格判错」：外层 1G、--threads 8、两处，每处起的 Verus 带 --num-threads 8）
  VERUS_PROOF_CHECK_BREAK=group-leftovers-kept  Verus 退了之后不停它的进程组里留下的（改前直接起那一支的样子）
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
  差分测试只核在 crates/ 下标了 #[test]，不在这里跑（它归 验证包，随改动跑）；被验区域之外对被验类型的构造与对带 requires 的被验函数的调用（第二轮正推 Y4 第 8 条）
  这一份不判；必红世界还是逐行手写，没有按 ensures 自动造（Y4 第 6 条）；逐函数结局还是逐项起 Verus，没改成读整 crate 一趟的 func-details（Y4 第 4 条）。
被验区域与多 crate 的弄坏开关（门禁 formal-verification 那一道的变量 FORMAL_VERIFICATION_BREAK=<项>[,<项>…]，判法这一半在这份脚本里，那一道文件头另列一份
它们让哪份门禁样本判错）；逐项写开关、弄坏哪一处，下一行起写打开后自证哪几格判错（「」里是那一格判错行的开头）：
  FORMAL_VERIFICATION_BREAK=single-crate-registry  登记表只认第一个 crate（改前的样子）
      multi-crate-proof（「✗ multi-crate-proof 这一格判错」）、multi-crate-list-items（「✗ multi-crate-list-items 这一格判错」）、
      multi-crate-item-routed（「✗ multi-crate-item-routed 这一格判错」）
  FORMAL_VERIFICATION_BREAK=cargo-mode-ignored  crate 行第四段 cargo 不认、照直接起 verus 判（有别的依赖就红）
      cargo-mode-proof（「✗ cargo-mode-proof 这一格判错」）、cargo-mode-vacuity（「✗ cargo-mode-vacuity 这一格判错」）、
      cargo-mode-must-red（「✗ cargo-mode-must-red 这一格判错」）、cargo-mode-command（「✗ cargo-mode-command 这一格判错」）、
      cargo-mode-shared-build-directory（「✗ cargo-mode-shared-build-directory 这一格判错」）
  FORMAL_VERIFICATION_BREAK=methods-refused  impl 与 trait 里的方法判红、要挪成自由 fn（改前的样子）
      proof-method-items（「✗ proof-method-items 这一格判错」）、proof-same-point-name（「✗ proof-same-point-name 这一格判错」）
      连带红的格：「✗ trait-impl-lag-registered-green 这一格判错」、「✗ trait-impl-lag-baseline-green 这一格判错」、「✗ proof-trait-declaration-struct-literal 这一格判错」
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
        「✗ trait-impl-lag-baseline-green 这一格判错」、「✗ trait-impl-lag-grown-red 这一格判错」
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
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
REGISTRY = ".claude/gate.d/verus-pins.tsv"
# 复用三问的规范输出与复用哨兵认的脚本名（scripts/gate/lib_reuse_checks.py 文件头「一」「二」）
SCRIPT_KEY = "scripts/gate/verus-proof-check.py"
EXIT_GREEN, EXIT_RED, EXIT_USAGE, EXIT_NOT_RUN = 0, 1, 2, 77
# 登记表没有 rlimit 行时 Verus 命令行上写的数：Verus 0.2026.09.20 自己的默认值（登记表 rlimit 行的依据写在那一行上面的注释里）
DEFAULT_RLIMIT = 10
RLIMIT_EXHAUSTED = "Resource limit (rlimit) exceeded"
# 必红世界表第五段的唯一形态：proof_errors=<证明错条数>（proof_error_messages 数的那些，不含 rlimit 用尽）
EXPECTED_PROOF_ERRORS = re.compile(r"proof_errors=([1-9][0-9]*)")
EXPECTATION_HOWTO = ("必红世界表第五段一律写 proof_errors=M：M 是改坏的副本在登记的 rlimit 上 Verus 报的证明错条数"
                     "（error 起头的报错，除去含 Resource limit (rlimit) exceeded 的、以 aborting due to 起头的与 cargo 那一种自己打的 could not compile），至少 1")
# task-server 起任务时套的内存包装给命令设的外层上限（run-with-memory-cap.sh 文件头「套在包装里」）：几处同时验不按它收（parallel_locations），
# 自证里那一格造它、别的格开跑前从环境里清掉
OUTER_MEMORY_CAP_VARIABLE = "RUN_WITH_MEMORY_CAP_OUTER_CAP"
# Verus 被信号停了（直接起时负的信号号，经 bash 起时 128 + 信号号）那一处的出路：是外面停的，不写修证明（killed_signal）
KILLED_HOWTO = ("这一次是外面停了 Verus：查是谁在那一刻停了它（同一时刻的 ps、journalctl --user、看门狗的记录），那件事不再发生之后重跑这一格；"
                "整条命令撞了内存上限由 task-server 放大重排、不在这里判；不改证明、核或必红世界表")
# tool-versions 除 verus 外另哈希的安装文件：verus 只是起 rust_verify 的外壳，判定还取决于 rust_verify、z3 与 vstd
TOOL_VERSION_FILES = ("rust_verify", "z3", "vstd.vir", "libvstd.rlib")
# Verus 的 launcher 写死 rustup run <version.json 的 toolchain> 起 rust_verify；rust_verify 从那条工具链的 lib/ 读 librustc_driver 与 libLLVM，
# 从 lib/rustlib/<三元组>/lib/ 读 std、core 这些 crate 的 rlib 与 rmeta（rustup_toolchain_fields）
RUSTUP_DEFAULT_HOME = os.path.join(os.path.expanduser("~"), ".rustup")
EXCLUDED_PACKAGE_FILES = {"Cargo.toml", "Cargo.toml.orig", ".cargo_vcs_info.json", ".cargo-ok"}
# 核里一律不许的标记（不带 --no-cheating 之后这几样 Verus 不拒，由这里判）：admit(…)、assume(…)、verifier::external（让整个条目对 Verus 不可见）。
# assume_specification、external_body、uninterp 这几样不在这里：照信任清单逐条登记（TRUST_KINDS）；axiom 另判（region_problems，按词法单元认）
CHEATING_MARKER = re.compile(r"\badmit\s*\(|\bassume\s*\(|verifier\s*::\s*external\b|verifier\s*\(\s*external\s*\)")
# 信任清单（出口 5 第三轮判决第三节「门禁」）：被验区域里每一处可信声明一行，vstd 按版本整体一行；格式写在表自己的文件头
TRUST_LIST = ".claude/gate.d/verus-trust-list.tsv"
TRUST_KINDS = ("assume_specification", "external_body", "uninterp", "external_fn_specification", "external_trait_specification",
               "external_type_specification", "external_trait_extension")
CFG_KIND = "cfg"
TRUST_ROW_HOWTO = (f"照 {TRUST_LIST} 文件头写一行（制表符分隔）：种类、文件、点名的条目、规格原文的 sha256、一条差分测试的测试名、理由；"
                   "红句里给了这一处现算的 sha256 与整行的样子，测试名写那条把这份规格与真实现逐值比的 #[test]（没有就先写它）")
# verus! 外面的 trait impl 的登记式放过表（只缩不涨；格式写在表自己的文件头）：与 gate 基准那一版比、切内容行，都用滞后表的共用读法 lib-lag-baseline.py
TRAIT_IMPLS_OUTSIDE_LAG = ".claude/gate.d/verus-trait-impls-outside-lag.tsv"
LAG_BASELINE_LIBRARY = os.path.join(REPOSITORY_ROOT, ".claude", "gate.d", "lib-lag-baseline.py")
TRAIT_IMPL_OUTSIDE_HOWTO = ("把这个 impl 挪进 verus! 里（方法照 trait 的 requires / ensures 证出来）；确是不该被验的类型，改名或挪出被验区域。"
                            f"{TRAIT_IMPLS_OUTSIDE_LAG} 只缩不涨：gate 基准那一版里还没有这张表时照它的文件头初次登记，之后只删不加，新写的 impl 不往里登记")
# 函数（或模块）上的 rlimit 属性：#[verifier::rlimit(N)] 与旧写法 #[verifier(rlimit(N))] 都盖过命令行的 --rlimit（Verus 0.2026.09.20 实测两种写法都认、--no-cheating 不拒）
RLIMIT_ATTRIBUTE = re.compile(r"verifier\s*::\s*rlimit\b|verifier\s*\(\s*rlimit\b")
# 一行 fn 的开头（空真取插入点的说明用：requires 往上最近的一行）；const fn 也认（core 的 SlotNumber::to_device_offset 就是 const fn）
FUNCTION_LINE = re.compile(r"^\s*(pub(\([^)]*\))?\s+)?(open\s+|closed\s+)?(proof\s+|spec\s+|exec\s+)?(?:const\s+)?fn\s+(\w+)")
# 必红世界表第六段（落点）的唯一形态：landing=<落点>[,<落点>…]；落点是证明错报在哪个函数里（一个 fn 名），或两个占位：
# （函数外）证明错的位置不在任何 fn 的行里，（无位置）输出里这一条证明错没带 --> 位置。条数会随求解环境漂，落点不会（C654）
LANDING_OUTSIDE, LANDING_NO_LOCATION = "（函数外）", "（无位置）"
LANDING_TOKEN = rf"(?:[A-Za-z_][A-Za-z0-9_]*|{LANDING_OUTSIDE}|{LANDING_NO_LOCATION})"
EXPECTED_LANDING = re.compile(rf"landing=({LANDING_TOKEN}(?:,{LANDING_TOKEN})*)")
LANDING_HOWTO = ("必红世界表第六段一律写 landing=<函数名>[,<函数名>…]：证明错报在哪个 fn 里（Verus 报错 --> 位置所在的函数），多个用逗号、顺序随意；"
                 "不是为了数条数：同一个 fn 里多报一条循环不变式、条数跳了，落点不变就不红")
# 一行 fn 的开头（verus! 里 spec、proof、exec、open、closed、broadcast 这些修饰都认）：证明错的位置往上找最近一行这样的，就是它落在哪个函数里
FUNCTION_HEADING = re.compile(r"^\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?(?:(?:open|closed|broadcast|proof|spec(?:\([a-z]+\))?|exec|axiom|const|unsafe)\s+)*fn\s+(\w+)")
DIAGNOSTIC_LOCATION = re.compile(r"^\s*-->\s+(\S+?):(\d+):(\d+)")
BREAK = {item for item in os.environ.get("VERUS_PROOF_CHECK_BREAK", "").split(",") if item}
if "const-fn-heading-unrecognized" in BREAK:   # 弄坏开关：FUNCTION_LINE 不认 const fn（改前的样子）
    FUNCTION_LINE = re.compile(r"^\s*(pub(\([^)]*\))?\s+)?(open\s+|closed\s+)?(proof\s+|spec\s+|exec\s+)?fn\s+(\w+)")
# cargo 那一种（cargo verus focus）编不过时 cargo 自己打的那一句「error: could not compile `<包>` (lib) due to N previous error(s)」：不是证明错，
# 与 rustc 的「aborting due to」一样不数、不进落点（proof_error_messages、diagnostic_blocks）
CARGO_GAVE_UP = "could not compile"
# 门禁 formal-verification 那一道的弄坏开关（文件头「被验区域与多 crate 的弄坏开关」一段）：判法这一半在这份脚本里，两份文件头各列一份
FORMAL_BREAK = {item for item in os.environ.get("FORMAL_VERIFICATION_BREAK", "").split(",") if item}
# 这份脚本读的那几项（自证的开关轮逐项打开；门禁那一道自己读的另几项不在这里）
FORMAL_BREAK_ITEMS = ("single-crate-registry", "cargo-mode-ignored", "methods-refused", "attribute-mode-unrecognized", "no-cheating-kept",
                      "trust-list-unchecked", "trust-digest-unchecked", "trust-test-unchecked", "trust-vstd-unchecked", "axiom-accepted",
                      "cfg-atom-accepted", "cfg-macro-unchecked", "trait-impl-outside-accepted", "mir-compare-skipped", "rustc-pin-unchecked",
                      "reuse-key-without-trust-list", "cargo-fingerprint-kept")
# 门禁给的一份文件：每格判完往里追加一行「格<TAB>现判数<TAB>复用数<TAB>原因分栏」（lib_verdict_library.Tally.append_to），门禁整道相加
ITEM_TALLY_FILE_VARIABLE = "FORMAL_ITEM_TALLY_FILE"
# 门禁给的另一份文件：每项判完（现判或复用）往里追加一行「项<TAB>这一项自己的挂钟秒数<TAB>现判条数<TAB>复用条数<TAB>退出码」，门禁收尾转成 ITEM_DONE 行
ITEM_DONE_FILE_VARIABLE = "FORMAL_ITEM_DONE_FILE"
# list-items --key-mismatches 往 stderr 逐项打的一行前缀：「ITEM_KEY item=<项名> state=<键相同|键对不上|无记录|拒算>」（门禁 --list-items --with-keys 转成 ITEM_PLAN 行）
ITEM_KEY_PREFIX = "ITEM_KEY"


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


def one_line(text):
    """自证一格判错的细节行（以 ✗ 起头、带格名）里转述了被测那一格的输出：换行换成「 | 」，整条留在这一行里，
    转述里以 ✗ 起头的那几行不另起一行、不被当成别的格（门禁 code-tooling 的 break-switch-replay 格按以 ✗ 起头的行认格）。"""
    rows = text.split("\n")
    return " | ".join([rows[0].rstrip(), *(row.strip() for row in rows[1:] if row.strip())])


def report_cell(problems):
    """一格的问题逐条打（(卡在哪, 下一步)）。"""
    for message, howto in problems:
        report_red(message, howto)


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
        self.rustc = None            # rustc 行：(Verus 用的工具链, 产品 rustc 的版本, 产品 rustc 的 commit-hash)；只有 verus-toolchain-pinned 一格要它
        self.cargo_mode = False      # crate 行第四段写 cargo：经 cargo verus focus 验（可以有别的依赖），不写是直接起 verus（只依赖 vstd）
        self.crates = []             # 每个登记的 crate 一份（CRATE_FIELDS 换成那个 crate 的），只登记一个 crate 时就是 [自己]
        self.all_crate_directories = []


# 每个 crate 自己的几样（per_crate_view 换成那个 crate 的）；别的（release、package、rlimit、rustc）各 crate 共用
CRATE_FIELDS = ("crate_directory", "verified_count", "kernels", "must_red_table", "cargo_mode")
# 登记表各种行的段数；crate 行三段或四段（第四段只许写 cargo）
REGISTRY_WIDTHS = {"release": (5,), "package": (6,), "crate": (3, 4), "kernel": (2,), "must-red-worlds": (2,), "rlimit": (2,), "rustc": (4,)}


def per_crate_view(registry, crate):
    """登记的一个 crate 的那一份登记：共用的几样照抄，CRATE_FIELDS 换成这个 crate 的。"""
    view = Registry()
    view.__dict__.update(registry.__dict__)
    for name in CRATE_FIELDS:
        setattr(view, name, crate[name])
    view.crates = []
    return view


def crate_identifier(registry, manifest=None):
    """crate 的 Rust 名（Cargo.toml 的 package.name，- 换成 _）：项名里全路径的头一段，与 Verus 的 func-details 同一种写法。"""
    if manifest is None:
        try:
            manifest = tomllib.load(open(os.path.join(registry.root, registry.crate_directory, "Cargo.toml"), "rb"))
        except (OSError, tomllib.TOMLDecodeError, TypeError, AttributeError):
            manifest = {}
    return (manifest.get("package", {}).get("name") or os.path.basename(registry.crate_directory or "crate")).replace("-", "_")


def read_registry(root):
    path = os.path.join(root, REGISTRY)
    if not os.path.isfile(path):
        raise Red(f"登记表 {REGISTRY} 不在 {root} 里", "它随仓走；被删了从 git 找回，格式写在它自己的文件头")
    registry = Registry()
    registry.root = root
    problems = []
    crates, kernels, tables = [], [], []
    for number, raw in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
        if not raw.strip() or raw.startswith("#"):
            continue
        fields = raw.split("\t")
        kind, widths = fields[0], REGISTRY_WIDTHS.get(fields[0])
        if widths is None or len(fields) not in widths:
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
            directory = fields[1].rstrip("/")
            if any(crate["crate_directory"] == directory for crate in crates):
                problems.append(f"第 {number} 行：crate {directory} 登记了两次")
            if len(fields) == 4 and fields[3] != "cargo":
                problems.append(f"第 {number} 行：crate 行第四段只许写 cargo（经 cargo verus focus 验），读到 {fields[3]!r}")
            if not fields[2].isdigit() or int(fields[2]) == 0:
                problems.append(f"第 {number} 行：verified 的登记数要是正整数，读到 {fields[2]!r}")
            crates.append({"crate_directory": directory, "verified_count": int(fields[2]) if fields[2].isdigit() else None,
                           "kernels": [], "must_red_table": None,
                           # 弄坏开关 cargo-mode-ignored：第四段 cargo 不认，照直接起 verus 判（改前的样子：有别的依赖就红）
                           "cargo_mode": len(fields) == 4 and fields[3] == "cargo" and "cargo-mode-ignored" not in FORMAL_BREAK})
        elif kind == "kernel":
            kernels.append((number, fields[1]))
        elif kind == "rlimit":
            if registry.rlimit is not None:
                problems.append(f"第 {number} 行：rlimit 行不止一行")
            if not fields[1].isdigit() or int(fields[1]) == 0:
                problems.append(f"第 {number} 行：rlimit 要是正整数，读到 {fields[1]!r}")
            else:
                registry.rlimit = int(fields[1])
        elif kind == "rustc":
            if registry.rustc is not None:
                problems.append(f"第 {number} 行：rustc 行不止一行")
            if not all(fields[1:]) or not re.fullmatch(r"[0-9a-f]{40}", fields[3]):
                problems.append(f"第 {number} 行：rustc 行三段是 Verus 用的工具链、产品 rustc 的版本、产品 rustc 的 commit-hash（40 位小写十六进制）")
            registry.rustc = tuple(fields[1:])
        else:
            tables.append((number, fields[1]))
    if len(crates) > 1 and "single-crate-registry" in FORMAL_BREAK:
        crates = crates[:1]   # 弄坏开关：只认登记表里第一个 crate（改前的样子）
    for number, kernel in kernels:
        owner = [crate for crate in crates if kernel.startswith(crate["crate_directory"] + "/")]
        if not owner:
            if "single-crate-registry" not in FORMAL_BREAK:
                problems.append(f"核 {kernel} 不在登记的 crate {'、'.join(crate['crate_directory'] for crate in crates) or '（一个都没有）'} 里")
            continue
        owner[0]["kernels"].append(kernel)
    for number, table in tables:
        owner = [crate for crate in crates if table.startswith(crate["crate_directory"] + "/")] if len(crates) > 1 else crates
        if not owner:
            if "single-crate-registry" not in FORMAL_BREAK:
                problems.append(f"第 {number} 行：必红世界表 {table} 不在登记的哪个 crate 的目录里（登记了几个 crate 时，每个 crate 的表放在它自己的目录里）")
            continue
        if owner[0]["must_red_table"] is not None:
            problems.append(f"第 {number} 行：crate {owner[0]['crate_directory']} 的 must-red-worlds 行不止一行")
        owner[0]["must_red_table"] = table
    if registry.tag is None or not crates or not registry.packages or any(not crate["kernels"] or crate["must_red_table"] is None for crate in crates):
        problems.append("release 恰好一行，crate 至少一行、每个 crate 至少一行 kernel、恰好一行 must-red-worlds，package 至少一行")
    if problems:
        raise Red(f"登记表 {REGISTRY} 写坏了：" + "；".join(problems), "照它文件头的格式改；一行一条、制表符分隔")
    if registry.rlimit is None:
        registry.rlimit = DEFAULT_RLIMIT
    registry.all_crate_directories = [crate["crate_directory"] for crate in crates]
    if len(crates) == 1:
        for name in CRATE_FIELDS:
            setattr(registry, name, crates[0][name])
        registry.crates = [registry]
    else:
        registry.crates = [per_crate_view(registry, crate) for crate in crates]
    return registry


def kernel_files(root, registry):
    """登记的核都在就交回它们；核没进仓 NotRun；核不在而 crate 已经接了 Verus 判红。"""
    missing = [kernel for kernel in registry.kernels if not os.path.isfile(os.path.join(root, kernel))]
    if not missing:
        # 不依赖登记数：新增模块的契约即使未验证，也能被已登记函数使用。
        registered = {os.path.realpath(os.path.join(root, path)) for path in registry.kernels}
        for directory, _subdirectories, filenames in os.walk(os.path.join(root, registry.crate_directory, "src")):
            for filename in filenames:
                if not filename.endswith(".rs"):
                    continue
                path = os.path.join(directory, filename)
                _kinds, words, _lines, _docs = rust_lex(open(path, encoding="utf-8").read())
                # verus! 宏包的，与属性写法（#[verus_verify]、#[verus_spec(…)]）标的，都是被验区域
                attribute_mode = "attribute-mode-unrecognized" not in FORMAL_BREAK and any(word in ("verus_verify", "verus_spec") for word in words)
                if (any(words[index:index + 2] == ["verus", "!"] for index in range(len(words) - 1)) or attribute_mode) and os.path.realpath(path) not in registered:
                    raise Red(f"未登记的 Verus 核：{os.path.relpath(path, root)}" + ("（属性写法 #[verus_verify] / #[verus_spec]）" if attribute_mode else ""),
                              f"把该文件加入 {REGISTRY} 的 kernel 行；所有被使用的证明契约都必须验证")
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

# 登记表 release 行第五列相对的是数据根的上一层：起头的 singlefs/ 是数据根本身，再下一层 formal-tools 是数据根下的共享名（lib_work_root.py 的 SHARED_DATA_ENTRIES）
INSTALL_SHARED_ENTRY = "formal-tools"
INSTALL_PREFIX = "singlefs/" + INSTALL_SHARED_ENTRY + "/"


def work_root_library():
    """.claude/hooks/lib_work_root.py（数据根只在那一份里定义；按文件路径导入，与 scripts/kb/pdf-text.py 同法）。读不到判红。"""
    path = os.path.join(REPOSITORY_ROOT, ".claude", "hooks", "lib_work_root.py")
    try:
        specification = importlib.util.spec_from_file_location("lib_work_root_for_verus_proof_check", path)
        library = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(library)
    except (OSError, ImportError, SyntaxError, AttributeError) as error:
        raise Red(f"读不到 {path}（{error}），取不到数据根", "在带着 .claude/hooks/lib_work_root.py 的仓里跑，或用 --verus-root、SINGLEFS_VERUS_ROOT 指到 Verus 的安装目录") from error
    return library


def verus_directory(registry, override):
    """Verus 安装目录：--verus-root，再 SINGLEFS_VERUS_ROOT，再共享数据根下的登记位置（formal-tools 在共享名单里，两台同一个绝对路径读同一份）。
    不经 ~ 与 XDG_DATA_HOME：第二台的家目录是另一个。登记的第五列是绝对路径就是它自己（门禁样本把它改成替身安装）。
    弄坏开关 default-install-by-home 下照改前取 ${XDG_DATA_HOME:-~/.local/share}/<第五列>。"""
    if override:
        return os.path.abspath(override)
    if os.environ.get("SINGLEFS_VERUS_ROOT"):
        return os.path.abspath(os.environ["SINGLEFS_VERUS_ROOT"])
    if "default-install-by-home" in BREAK:
        data_home = os.environ.get("XDG_DATA_HOME") or os.path.join(os.path.expanduser("~"), ".local", "share")
        return os.path.join(data_home, registry.install_relative)
    if os.path.isabs(registry.install_relative):
        return registry.install_relative
    if not registry.install_relative.startswith(INSTALL_PREFIX):
        raise Red(f"{REGISTRY} 的 release 行第五列 {registry.install_relative} 不以 {INSTALL_PREFIX} 起头，认不出它在共享数据根下的位置",
                  f"第五列写成 {INSTALL_PREFIX}<发布目录>，或写绝对路径，或用 --verus-root、SINGLEFS_VERUS_ROOT 指到安装目录")
    library = work_root_library()
    try:
        shared_directory = library.shared_data_path(INSTALL_SHARED_ENTRY)
    except ValueError as error:
        raise Red(f"取不到数据根下的 {INSTALL_SHARED_ENTRY}：{error}",
                  f"{getattr(library, 'DATA_ROOT_HOWTO', '配好数据根')}；或用 --verus-root、SINGLEFS_VERUS_ROOT 指到 Verus 的安装目录") from error
    return os.path.join(shared_directory, registry.install_relative[len(INSTALL_PREFIX):])


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
    if registry.cargo_mode:
        return   # 经 cargo verus focus 验：依赖照 Cargo.toml 由 cargo 编（copy_crate 连路径依赖一起拷）
    others = sorted(name for name in manifest.get("dependencies", {}) if name != "vstd")
    if others:
        raise Red(f"{registry.crate_directory}/Cargo.toml 除 vstd 之外还依赖 {' '.join(others)}：直接起的 Verus 只带它自带的 vstd，编不了别的 crate",
                  f"确要别的依赖（singlefs-core 这类），把 {REGISTRY} 里这个 crate 那一行写成四段、第四段 cargo：经 cargo verus focus 验，路径依赖连同 crate 一起拷进副本；"
                  "只依赖 vstd 的核照旧三段、直接起 verus")


def path_dependencies(root, crate_directory, seen=None):
    """crate（相对仓根的目录）与它经 Cargo.toml 的 path 依赖（dependencies、build-dependencies，一层一层往下）连到的 crate 目录，相对仓根、排好序。
    dev-dependencies 不收（cargo verus focus 不编测试）。路径出了仓根、或那份 Cargo.toml 不在的判红。"""
    seen = set() if seen is None else seen
    if crate_directory in seen:
        return seen
    seen.add(crate_directory)
    manifest_path = os.path.join(root, crate_directory, "Cargo.toml")
    try:
        manifest = tomllib.load(open(manifest_path, "rb"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise Red(f"{crate_directory}/Cargo.toml 读不出：{error}", "先让 cargo build 认得这份清单")
    for section in ("dependencies", "build-dependencies"):
        for name, specification in sorted(manifest.get(section, {}).items()):
            if not isinstance(specification, dict) or "path" not in specification:
                continue
            target = os.path.normpath(os.path.join(crate_directory, specification["path"]))
            if target.startswith(os.pardir) or os.path.isabs(target) or not os.path.isfile(os.path.join(root, target, "Cargo.toml")):
                raise Red(f"{crate_directory}/Cargo.toml 的路径依赖 {name} = {specification['path']!r} 不在仓里或没有 Cargo.toml",
                          "被验的 crate 的路径依赖要都在仓里（cargo verus focus 在副本里连它们一起编）")
            path_dependencies(root, target.replace(os.sep, "/"), seen)
    return seen


def copy_crate(root, registry, work):
    """一处判定一份副本：直接起 verus 的拷 crate 自己，交 crate 副本的目录（verus 在那里起）；cargo 那一种另拷它的路径依赖、仓根的 Cargo.lock，
    在副本根写一份只列这几个成员的 Cargo.toml（整个工作区的 cargo metadata 在离线机器上过不了），交副本根（cargo verus focus 在那里起）。
    每处判定都在一份新目录里：cargo 的编译指纹不看 --verify-* 参数，同一个目录里换参数再验拿到的是上一趟的结果、没有判定行（第二轮正推 Y4 第 3 条）。"""
    if not registry.cargo_mode:
        source = os.path.join(root, registry.crate_directory)
        destination = os.path.join(work, registry.crate_directory)
        shutil.copytree(source, destination, ignore=shutil.ignore_patterns("target"), symlinks=True)
        return destination
    members = sorted(path_dependencies(root, registry.crate_directory))
    for member in members:
        shutil.copytree(os.path.join(root, member), os.path.join(work, member), ignore=shutil.ignore_patterns("target"), symlinks=True)
    with open(os.path.join(work, "Cargo.toml"), "w", encoding="utf-8") as handle:
        handle.write("# verus-proof-check.py 拷的副本：只列被验的 crate 与它的路径依赖\n[workspace]\nresolver = \"2\"\nmembers = [\n"
                     + "".join(f"    {json.dumps(member)},\n" for member in members) + "]\n")
    if os.path.isfile(os.path.join(root, "Cargo.lock")):
        shutil.copy2(os.path.join(root, "Cargo.lock"), os.path.join(work, "Cargo.lock"))
    return work


RUST_IDENTIFIER = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")


def kernel_module(registry, manifest, relative):
    """核 relative（相对仓根）所在的模块：是 crate 的入口文件（Cargo.toml 的 lib.path，默认 src/lib.rs）交 None（点 --verify-root）；
    入口所在目录下的 <名>.rs 与 <名>/mod.rs 交 <名>，更深的用 :: 接起来（Verus 0.2026.09.20 的 --verify-only-module 认 outer::inner，文件头「只验一个函数」一段）。
    不在入口所在目录下、或有一段不是 Rust 标识符的判红。弄坏开关 submodule-verify-root 下一律交 None（改前的样子）。"""
    if "submodule-verify-root" in BREAK:
        return None
    entry = os.path.normpath(manifest.get("lib", {}).get("path", "src/lib.rs"))
    crate_relative = os.path.normpath(os.path.relpath(relative, registry.crate_directory))
    if crate_relative == entry:
        return None
    rest = os.path.relpath(crate_relative, os.path.dirname(entry) or ".")
    parts = rest[:-len(".rs")].split(os.sep) if rest.endswith(".rs") and not rest.startswith(os.pardir + os.sep) else []
    if parts and parts[-1] == "mod":
        parts = parts[:-1]
    if not parts or not all(RUST_IDENTIFIER.fullmatch(part) for part in parts):
        raise Red(f"核 {relative} 认不出是 {registry.crate_directory} 的哪个模块：要是入口 {entry}，"
                  f"或入口所在目录 {os.path.dirname(entry) or '.'} 下的 <名>.rs、<名>/mod.rs（更深的照样一层一层往下）",
                  f"把核挪到入口所在目录下、照文件路径用 mod 声明接上，并照新路径改 {REGISTRY} 的 kernel 行；确要别的写法（#[path]、内联 mod），"
                  "先给 scripts/gate/verus-proof-check.py 的 kernel_module 补上认法")
    return "::".join(parts)


def verus_arguments(registry, manifest, threads, function=None, module=None):
    """Verus 的命令行；function 给了就只验这一个函数（证明逐函数一项、空真只验它所属的那一个函数）：module 是 None（核是 crate 的入口）点 --verify-root，
    不是 None 点 --verify-only-module <module>（kernel_module），再 --verify-function <名>。--verify-function 只在点的那一个模块里找；
    Verus 0.2026.09.20 拒 --verify-module 与 --verify-function 一起给、拒 --verify-root 与 --verify-only-module 一起给（文件头「只验一个函数」一段）。
    Verus 0.2026.09.20 按子串点名、有恰好同名的就取它（主 agent 2026-10-15 在副本上量的：名字是别人子串的四个都退 0、没报 more than one match），
    所以 verus! 里的 fn 同名判红（verus_functions），名字是别人子串的照常用；点名在表里是不是恰好命中一项，起 Verus 之前由 point_name_problems 判。"""
    package = manifest.get("package", {})
    entry = manifest.get("lib", {}).get("path", "src/lib.rs")
    # 不带 --no-cheating（出口 5 第三轮判决第三节「门禁」）：assume_specification、external_body、uninterp 这几样照信任清单逐条登记（trust_list_problems），
    # admit、assume(、axiom、verifier::external 在被验区域里一律判红（region_problems），都在起 Verus 之前判
    arguments = [] if registry.cargo_mode else ["--crate-type=lib", f"--edition={package.get('edition', '2021')}", "--crate-name", package.get("name", "crate").replace("-", "_")]
    arguments += ["--num-threads", str(threads), "--output-json"]
    if "rlimit-unpinned" not in BREAK:
        arguments += ["--rlimit", str(registry.rlimit)]
    if "no-cheating-kept" in FORMAL_BREAK:
        arguments.insert(0, "--no-cheating")   # 弄坏开关：照旧带 --no-cheating（登记了的 assume_specification 也被 Verus 拒）
    if function:
        arguments += ["--verify-root"] if module is None else ["--verify-only-module", module]
        arguments += ["--verify-function", function]
    return arguments if registry.cargo_mode else arguments + [entry]


def forget_crate_fingerprints(target, package):
    """cargo 那一种几处同时验共用一个编译目录：起之前删掉被验的 crate 在这个编译目录里的指纹目录（<编译目录>/**/.fingerprint/<包>-*），让它每处都现编、出判定行；
    cargo 的编译指纹不看 --verify-* 参数，换一份副本、改了文件时间也照样当它没变（2026-10-26 在样本上撞到，第二轮正推 Y4 第 3 条）。依赖的指纹不动、照旧复用。
    弄坏开关 FORMAL_VERIFICATION_BREAK=cargo-fingerprint-kept 下不删（改前的样子：第二处起拿不到判定行）。"""
    if "cargo-fingerprint-kept" in FORMAL_BREAK:
        return
    for path in glob.glob(os.path.join(target, "**", ".fingerprint", f"{package}-*"), recursive=True):
        shutil.rmtree(path, ignore_errors=True)


def verus_program(registry, executable):
    """起 Verus 的程序：直接起的是 verus（在 crate 副本里起）；cargo 那一种是 Verus 安装里 verus 旁边的 cargo-verus（在副本根起）。"""
    if not registry.cargo_mode:
        return executable
    return os.path.join(os.path.dirname(executable), "cargo-verus")


def verus_program_arguments(registry, arguments, build=False):
    """交给 verus_program 的参数：直接起的照原样；cargo 那一种是 verus focus -p <包> --offline -- <参数…>（build 时是 verus build，MIR 比对用）。"""
    if not registry.cargo_mode:
        return list(arguments)
    return ["verus", "build" if build else "focus", "-p", crate_package_name(registry), "--offline", "--", *arguments]


def crate_package_name(registry):
    try:
        manifest = tomllib.load(open(os.path.join(registry.root, registry.crate_directory, "Cargo.toml"), "rb"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise Red(f"{registry.crate_directory}/Cargo.toml 读不出：{error}", "先让 cargo build 认得这份清单")
    return manifest.get("package", {}).get("name") or os.path.basename(registry.crate_directory)


def diagnostic_path(registry, relative):
    """Verus 报错 --> 后面的路径怎么写这份文件（relative 相对仓根）：直接起的相对 crate 目录（在 crate 副本里起），cargo 那一种相对副本根（就是相对仓根）。"""
    if registry.cargo_mode:
        return relative
    return os.path.relpath(relative, registry.crate_directory)


def located_file(registry, path):
    """Verus 报错 --> 后面的路径交回相对仓根的那一份（diagnostic_path 的反过来）。"""
    return os.path.normpath(path if registry.cargo_mode else f"{registry.crate_directory}/{path}")


def kernel_modules(root, registry, manifest, kernels):
    """每份核（相对仓根的路径）点哪个模块：{相对路径: kernel_module}。认不出的判红（开跑之前）。"""
    modules = {}
    for kernel in kernels:
        relative = os.path.relpath(kernel, root).replace(os.sep, "/")
        modules[relative] = kernel_module(registry, manifest, relative)
    return modules


def stop_process_group(group, signal_number):
    """对自己起的那一组发信号（start_new_session 起的，进程组号就是直接起的那一个的进程号）；组里已经没有进程了不算错。"""
    try:
        os.killpg(group, signal_number)
    except (ProcessLookupError, PermissionError):
        pass


def finish_process_group(process):
    """start_new_session 起的、stdout 与 stderr 接了管道的那一个：两条线程边跑边收，它退了先停组里留下的（拿着管道的子进程不让读的一头等到 EOF），
    再收齐输出，交 (stdout, stderr)。弄坏开关 group-leftovers-kept 下不停留下的（读的一头等到它们自己退）。"""
    collected = {}
    readers = [threading.Thread(target=lambda name=name, stream=stream: collected.__setitem__(name, stream.read()), daemon=True)
               for name, stream in (("stdout", process.stdout), ("stderr", process.stderr))]
    for reader in readers:
        reader.start()
    process.wait()
    if "group-leftovers-kept" not in BREAK:
        stop_process_group(process.pid, signal.SIGKILL)
    for reader in readers:
        reader.join()
    return collected.get("stdout", ""), collected.get("stderr", "")


def run_verus(program, arguments, crate_copy, extra_environment=None):
    """起一次 Verus（program 是 verus_program 交的、arguments 是 verus_program_arguments 交的；MIR 比对的产品构建交 cargo 与它的参数）；交回 {exit, json, diagnostics, stderr}。
    直接起：不套内存包装、capped.sh 与 nice，线程数在 Verus 自己的 --num-threads 上（verus_arguments），整条命令的内存额度由 task-server 给；不限时，卡住由 task-server 判。
    放进它自己的进程组，输出与收尾照 finish_process_group。extra_environment 另设进环境（cargo 那一种的 CARGO_TARGET_DIR）。"""
    environment = dict(os.environ, **(extra_environment or {}))
    process = subprocess.Popen([program, *arguments], cwd=crate_copy, env=environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               text=True, errors="replace", start_new_session=True)
    stdout, stderr = finish_process_group(process)
    outcome = {"exit": process.returncode, "json": None, "diagnostics": stderr + "\n" + stdout, "stderr": stderr}
    try:
        outcome["json"] = json.loads(stdout) if stdout.strip().startswith("{") else None
    except ValueError:
        outcome["json"] = None
    return outcome


def diagnostic_blocks(text):
    """stderr 里的「error: <说什么>」与它下面第一处「--> 文件:行:列」配成一对。"""
    blocks, pending = [], None
    for line in text.split("\n"):
        if line.startswith("error"):
            pending = line.split(":", 1)[1].strip() if ":" in line else line
            if pending.startswith("aborting due to") or (pending.startswith(CARGO_GAVE_UP) and "could-not-compile-counted" not in BREAK):
                pending = None
            continue
        match = DIAGNOSTIC_LOCATION.match(line)
        if match and pending is not None:
            blocks.append((pending, match.group(1), int(match.group(2))))
            pending = None
    return blocks


def proof_error_messages(text, count_rlimit=False):
    """输出里 error 起头的报错，除去含 rlimit 用尽的、以「aborting due to」起头的与 cargo 自己那一句「could not compile」之后剩下的（证明错）；
    必红世界表第五段的 M 数的就是它们，只剩 rlimit 用尽的那一行不作数。count_rlimit 只给弄坏开关 rlimit-counted-in-proof-errors 用：rlimit 用尽也算进去。
    弄坏开关 could-not-compile-counted：cargo 那一句照算（改前的样子：cargo 那一种每行必红世界多一条、落点多一个（无位置））。"""
    messages = []
    for line in text.split("\n"):
        if not line.startswith("error"):
            continue
        message = line.split(":", 1)[1].strip() if ":" in line else line
        if (RLIMIT_EXHAUSTED in message and not count_rlimit) or message.startswith("aborting due to"):
            continue
        if message.startswith(CARGO_GAVE_UP) and "could-not-compile-counted" not in BREAK:
            continue
        messages.append(message)
    return messages


def integer_counts(results):
    """JSON 的 verified 与 errors 都是整数才交 True：缺一个是 vir 错没走到验证，或 Verus 换了 JSON 的写法。"""
    return all(isinstance(results.get(key), int) and not isinstance(results.get(key), bool) for key in ("verified", "errors"))


def outcome_problem(outcome, options):
    """这一次 Verus 没得出可判的结局时交回一句（被信号停了、没出 JSON），得出了交 None。"""
    signal_number = killed_signal(outcome)
    if signal_number is not None:
        tail = " | ".join(line for line in outcome["diagnostics"].strip().split("\n")[-4:] if line.strip())
        return f"Verus 被信号停了（退 {outcome['exit']}，信号 {signal_number}），这一次的输出不算判定{'（' + tail[:300] + '）' if tail else ''}"
    if outcome["json"] is None or "verification-results" not in outcome["json"]:
        tail = " | ".join(line for line in outcome["diagnostics"].strip().split("\n")[-6:] if line.strip())
        return f"Verus 没出判定（退 {outcome['exit']}，编不过或起不来）：{tail}"
    return None


def outcome_judged(outcome):
    """这一次 Verus 出没出能记进判定库的结局（红的记红、撤绿用）：被信号停了交 False（这一次的输出不算判定）；
    别的（Verus 报错、没出 JSON）交 True。"""
    return killed_signal(outcome) is None


def problem_howto(outcome, howto):
    """outcome_problem 报了问题时给的出路：被信号停了给 KILLED_HOWTO（是外面停的，不写修证明），别的给这一格自己的 howto。"""
    if killed_signal(outcome) is not None:
        return KILLED_HOWTO
    return howto


def killed_signal(outcome):
    """Verus 被信号停了就交信号号：直接起时 subprocess 交负的信号号，经 bash 起时退 128 + 信号号（129–192）；
    别的退出码交 None。弄坏开关 killed-as-proof-red 下一律交 None。"""
    code = outcome["exit"]
    if "killed-as-proof-red" in BREAK:
        return None
    if code < 0:
        return -code
    return code - 128 if 128 < code <= 128 + 64 else None


# ── 四格 ──

def check_toolchain(root, options):
    registry = read_registry(root)
    for view in registry.crates:
        kernel_files(root, view)
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
    rustc_note = rustc_pin_problems(root, registry, version, options, problems)
    if problems:
        for message, howto in problems:
            report_red(message, howto)
        return EXIT_RED
    print(f"  ✓ Verus {registry.tag}（commit {registry.commit[:12]}…）与登记相同，{zip_note}；Cargo.lock 与本机缓存里 {len(registry.packages)} 个包的版本、出处与源码指纹都与登记相同；"
          f"{rustc_note}；按条：toolchain 一项，不复用（每趟现核）")
    return EXIT_GREEN


def product_rustc_version(root, options):
    """产品构建的 rustc（在被判的仓根起 rustc -vV，rustup 照那里的设定选工具链）交 (release, commit-hash)；起不了交 (None, 原因)。
    样本给 options["rustc_version_file"]（.formal-sample/rustc-version）时读那份文件当 rustc -vV 的输出。"""
    if options.get("rustc_version_file"):
        text = open(options["rustc_version_file"], encoding="utf-8").read()
    else:
        try:
            completed = subprocess.run(["rustc", "-vV"], cwd=root, capture_output=True, text=True)
        except OSError as error:
            return None, f"起不了 rustc -vV：{error}"
        if completed.returncode != 0:
            return None, f"rustc -vV 退 {completed.returncode}：{completed.stderr.strip()[-200:]}"
        text = completed.stdout
    fields = dict(line.split(": ", 1) for line in text.split("\n") if ": " in line)
    return (fields.get("release"), fields.get("commit-hash")), None


def rustc_pin_problems(root, registry, version, options, problems):
    """Verus 与产品的 rustc 版本差进工具链钉表（出口 5 第三轮判决第三节「门禁」）：登记表 rustc 行三段——Verus 用的工具链（version.json 的 toolchain 第一段）、
    产品 rustc 的 release、commit-hash，都与此刻的相同。两边不同不判红（今天 Verus 1.98.1、产品 1.98.0），判的是「差」被钉住：哪一边换了都要改这一行，
    改之前先在新的一对上把 MIR 比对跑一遍。问题追加进 problems，交一句给成功句。弄坏开关 FORMAL_VERIFICATION_BREAK=rustc-pin-unchecked 下不核。"""
    if "rustc-pin-unchecked" in FORMAL_BREAK:
        return "Verus 与产品的 rustc 没核（弄坏开关 rustc-pin-unchecked）"
    if registry.rustc is None:
        problems.append((f"登记表 {REGISTRY} 没有 rustc 行：Verus 与产品的 rustc 版本差没钉住",
                         "加一行 rustc<TAB><Verus 的 version.json 里 toolchain 的第一段><TAB><产品 rustc -vV 的 release><TAB><它的 commit-hash>"))
        return ""
    verus_toolchain = ((version or {}).get("toolchain") or "").split(" ")[0]
    if verus_toolchain != registry.rustc[0]:
        problems.append((f"Verus 用的工具链是 {verus_toolchain or '（version.json 里没有）'}，登记表 rustc 行写的是 {registry.rustc[0]}",
                         "换了 Verus 就照它 version.json 的 toolchain 改 rustc 行第二段，改之前在新工具链上把 MIR 比对（verus-proof 整格）跑一遍"))
    parsed, failure = product_rustc_version(root, options)
    release, commit = parsed if parsed else (None, None)
    if failure:
        problems.append((f"产品的 rustc 版本取不到：{failure}", "让 rustc -vV 在仓根跑得通（rustup 装着产品的工具链）"))
    elif (release, commit) != (registry.rustc[1], registry.rustc[2]):
        problems.append((f"产品的 rustc 是 {release}（commit-hash {commit}），登记表 rustc 行写的是 {registry.rustc[1]}（{registry.rustc[2]}）",
                         "产品的工具链换了：照 rustc -vV 改 rustc 行第三、四段，改之前在新的一对上把 MIR 比对（verus-proof 整格）跑一遍"))
    return f"Verus 的工具链 {registry.rustc[0]} 与产品的 rustc {registry.rustc[1]} 都与 rustc 行相同"


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


# ── 被验区域：信任清单、axiom、cfg 原子、verus! 外面的 trait impl（出口 5 第三轮判决第三节「门禁」；起 Verus 之前判）──

def read_trust_list(root):
    """信任清单 → (vstd 那几行 [(版本, 内容指纹, 行号)], 条目行 [dict], 写坏的行 [说明])；表不在交 None。
    条目行六段：种类（TRUST_KINDS 之一或 cfg）、文件（相对仓根）、点名的条目、规格原文的 sha256、差分测试的测试名、理由。vstd 行四段：vstd、版本、内容指纹、理由。"""
    path = os.path.join(root, TRUST_LIST)
    if not os.path.isfile(path):
        return None
    vstd_rows, rows, problems = [], [], []
    for number, raw in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
        if not raw.strip() or raw.startswith("#"):
            continue
        fields = raw.split("\t")
        if fields[0] == "vstd":
            if len(fields) != 4 or not all(fields[1:]) or not re.fullmatch(r"[0-9a-f]{64}", fields[2]):
                problems.append(f"{TRUST_LIST} 第 {number} 行：vstd 行四段（vstd、版本、内容指纹 64 位小写十六进制、理由）")
                continue
            vstd_rows.append((fields[1], fields[2], number))
        elif fields[0] in TRUST_KINDS + (CFG_KIND,):
            if len(fields) != 6 or not all(fields[1:]) or not re.fullmatch(r"[0-9a-f]{64}", fields[3]) or len(fields[5]) < 4:
                problems.append(f"{TRUST_LIST} 第 {number} 行：条目行六段（种类、文件、点名的条目、规格原文的 sha256、差分测试名、理由至少 4 个字）")
                continue
            rows.append({"number": number, "kind": fields[0], "file": fields[1], "point": fields[2], "digest": fields[3], "test": fields[4]})
        else:
            problems.append(f"{TRUST_LIST} 第 {number} 行：第一段「{fields[0]}」不是 vstd、cfg 或 {'、'.join(TRUST_KINDS)} 之一")
    return vstd_rows, rows, problems


_LAG_BASELINE = []


def lag_baseline_library():
    """滞后表「只缩不涨」的共用读法 .claude/gate.d/lib-lag-baseline.py（content_rows、gate_base_of、baseline_rows），取这份脚本所在的仓里那一份，载一次。"""
    if not _LAG_BASELINE:
        import importlib.util
        specification = importlib.util.spec_from_file_location("lib_lag_baseline_for_verus_proof_check", LAG_BASELINE_LIBRARY)
        library = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(library)
        _LAG_BASELINE.append(library)
    return _LAG_BASELINE[0]


def read_trait_impls_outside_lag(root):
    """登记式放过表 → (行 [dict], 写坏的行 [说明])；表不在交 ([], [])：一行都没登记。
    五段（制表符分隔）：trait 的全路径（<crate>::<模块>::<trait>）、实现类型（impl 头里 for 后面那个类型的名字，不带路径与泛型参数）、
    所在文件（相对仓根）、哪一簇会把它挪进 verus!、为什么此刻放过（至少 8 个字）。"""
    path = os.path.join(root, TRAIT_IMPLS_OUTSIDE_LAG)
    if not os.path.isfile(path):
        return [], []
    rows, problems = [], []
    for number, fields in lag_baseline_library().content_rows(open(path, encoding="utf-8").read()):
        fields = [field.strip() for field in fields]
        if len(fields) != 5 or not all(fields) or "::" not in fields[0] or not RUST_IDENTIFIER.fullmatch(fields[1]) or len(fields[4]) < 8:
            problems.append(f"{TRAIT_IMPLS_OUTSIDE_LAG} 第 {number} 行：要五段（trait 的全路径 <crate>::<模块>::<trait>、实现类型的名字、所在文件、"
                            "哪一簇会把它挪进 verus!、为什么此刻放过至少 8 个字）")
            continue
        rows.append({"number": number, "trait": fields[0], "type": fields[1], "file": fields[2], "cluster": fields[3], "reason": fields[4]})
    return rows, problems


def trait_impl_lag_growth(root, rows):
    """放过表只缩不涨：rows（这个 crate 的行）与 gate 基准那一版的表比（lib-lag-baseline.py 的 gate_base_of、baseline_rows），
    交 (问题 [(卡在哪, 下一步)], 给成功句的一句)。键是（trait 全路径、实现类型、所在文件），基准里没有的键判红。
    被判的根不在 git 仓里、基准里还没有这张表（初次登记）不比、成功句照说；git 取不到基准那一版判红。弄坏开关 trait-impl-lag-growth-unchecked 下不比。"""
    if "trait-impl-lag-growth-unchecked" in BREAK:
        return [], "没与 gate 基准比"
    library = lag_baseline_library()
    base, problem = library.gate_base_of(root)
    if problem is not None:
        return [(f"取不到 gate 基准，{TRAIT_IMPLS_OUTSIDE_LAG} 只缩不涨比不了：{problem}",
                 "照这一句修好 git（scripts/gate/changed-paths.sh 的 gate_diff_base gate 在被判的根上跑通）再跑这一格；不当成没有基准放过")], ""
    if base is None:
        return [], "被判的根不在 git 仓里，没与 gate 基准比"
    base_rows, problem = library.baseline_rows(base, TRAIT_IMPLS_OUTSIDE_LAG, cwd=root)
    if problem is not None:
        return [(f"{TRAIT_IMPLS_OUTSIDE_LAG} 只缩不涨比不了：{problem.sentence()}", "修好 git 对象库（git fsck）或照 GATE_BASE 给一个取得到的基准，再跑这一格")], ""
    if base_rows is None:
        return [], f"gate 基准 {base[:12]} 里还没有这张表（初次登记），没比"
    base_keys = {tuple(field.strip() for field in fields[:3]) for _number, fields in base_rows}
    grown = [row for row in rows if (row["trait"], row["type"], row["file"]) not in base_keys]
    return ([(f"{TRAIT_IMPLS_OUTSIDE_LAG} 第 {row['number']} 行 {row['trait']} for {row['type']}（{row['file']}）在 gate 基准 {base[:12]} 那一版的表里没有：这张表只缩不涨",
              "删掉这一行，把那个 impl 挪进 verus! 里（方法照 trait 的 requires / ensures 证出来）；登记式放过只给 gate 基准里已有的行") for row in grown],
            f"与 gate 基准 {base[:12]} 那一版比过、没涨")


# 阶段只跑了一部分时跳过的逐项报进门禁汇总的「本次未跑」（.claude/singlefs-ai-sop/rules/show-me-test.md「门禁不许假装通过」）：门禁给的那份文件，不在时不写
NOT_RUN_FILE_VARIABLE = "GATE_NOT_RUN_FILE"


def report_passed_not_run(options, registry):
    """放过表放过的每一处（region_problems 记进 options["trait_impls_passed"]）往 $GATE_NOT_RUN_FILE 追加一行本次未跑：那个 impl 的契约这一次没人验，只写在成功句里不算报了。
    变量不在时不写（单跑与喂样本时它不在），交一句给成功句说明报了几行或为什么没报。弄坏开关 trait-impl-lag-not-run-unreported 下不写（改前的样子）。"""
    passed = options.get("trait_impls_passed") or []
    if not passed:
        return ""
    path = os.environ.get(NOT_RUN_FILE_VARIABLE)
    if "trait-impl-lag-not-run-unreported" in BREAK or not path:
        return f"；放过的 {len(passed)} 处没往 ${NOT_RUN_FILE_VARIABLE} 报本次未跑（{'弄坏开关 trait-impl-lag-not-run-unreported' if path else '这个变量不在：单跑或喂样本'}）"
    with open(path, "a", encoding="utf-8") as handle:
        for row, where in passed:
            handle.write(f"formal-verification 的 verus-proof 格（{registry.crate_directory}）：verus! 外面的 {row['trait']} for {row['type']}"
                         f"（{where.split(' ', 1)[0]}）登记在 {TRAIT_IMPLS_OUTSIDE_LAG} 里放过，契约本次未验（靠信任，归「{row['cluster']}」挪进 verus!）\n")
    return f"；放过的 {len(passed)} 处逐个往 ${NOT_RUN_FILE_VARIABLE} 报了本次未跑"


def trust_list_text(root, registry):
    """进每一项键的「信任清单」那一样：vstd 行与这个 crate 里的条目行（去掉理由），排好序。表不在交空串。"""
    listed = read_trust_list(root)
    if listed is None:
        return ""
    vstd_rows, rows, _problems = listed
    lines = [f"vstd\t{version}\t{fingerprint}" for version, fingerprint, _number in vstd_rows]
    lines += [f"{row['kind']}\t{row['file']}\t{row['point']}\t{row['digest']}\t{row['test']}" for row in rows
              if row["file"].startswith(registry.crate_directory + "/")]
    return "\n".join(sorted(lines))


TEST_FUNCTION = re.compile(r"#\s*\[\s*test\s*\]\s*(?:#\s*\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)")


def test_functions(root):
    """仓里 crates/ 下每份 .rs 里标了 #[test] 的函数名（差分测试的测试名照它核：写全路径的取最后一段）。"""
    names = set()
    for path in glob.glob(os.path.join(root, "crates", "**", "*.rs"), recursive=True):
        try:
            names.update(TEST_FUNCTION.findall(open(path, encoding="utf-8", errors="replace").read()))
        except OSError:
            continue
    return names


def innermost_unit(rust_file, index):
    """词法单元 index 落在的最里层那个单元（数据单元不算）；不在任何单元里交 None。"""
    best = None
    for unit in rust_file.units:
        if unit.data or unit.first_token is None or unit.last_token is None or not unit.first_token <= index <= unit.last_token:
            continue
        if best is None or unit.last_token - unit.first_token < best.last_token - best.first_token:
            best = unit
    return best


def unit_path(crate, module, unit):
    """条目的全路径 <crate>::<模块>::<名>：fn 照 verus_point_name（impl 里的写 <类型>::<方法>），别的条目是它的名字（套在别的条目里的写嵌套路径）。"""
    if unit is None:
        return "::".join(part for part in (crate, module, "（不在任何条目里）") if part)
    name = (verus_point_name(unit) if unit.kind == "fn" else None) or (unit.short if unit.container is None else unit.name)
    return "::".join(part for part in (crate, module, name) if part)


def joined_tokens(rust_file, low, high):
    """词法单元 [low, high) 用一个空格连起来（去了注释与空白：只改注释、换行的不变）。"""
    return " ".join(rust_file.texts[low:high])


def statement_end(rust_file, index, limit):
    """从 index 往后找同一层的 ;（跳过配对的括号），交它的下标；找不到交 limit。"""
    position = index
    while position < limit:
        if rust_file.texts[position] in ("(", "[", "{") and position in rust_file.pairs:
            position = rust_file.pairs[position] + 1
            continue
        if rust_file.texts[position] == ";":
            return position
        position += 1
    return limit


def body_open(rust_file, unit):
    """fn 单元函数体的 {：lib_function_units 切出的单元是半开区间 [first_token, last_token)，单元里最后一个词法单元（last_token - 1）是 } 时，
    交与它配对、落在单元里的那个 {；最后一个词是 ;（trait 里只有签名与契约的声明）或配不上的，交 None。只按切出的单元与括号配对认，
    签名与 ensures 里的花括号（结构体字面量）不算函数体。external_body 的规格原文（trusted_declarations）与 trait 里的声明不成项（proof_items）都用它。"""
    if unit is None or unit.last_token is None:
        return None
    last = unit.last_token - 1
    if last < unit.first_token or rust_file.texts[last] != "}":
        return None
    return next((opening for opening, closing in rust_file.pairs.items() if closing == last and opening >= unit.first_token), None)


def body_open_past_unit_end(rust_file, unit):
    """弄坏开关 body-open-past-unit-end 下 external_body 的规格原文用的取法（改前的样子）：把 last_token 本身当单元最后一个词，
    一个 impl 里最后一个 fn 落在 impl 的 } 上、配到 impl 的 {，签名那一截取成空串；后面还接着别的条目的 fn 配不上，取成整个 fn 再多一个词。"""
    if unit is None or unit.last_token is None or unit.last_token >= rust_file.count or rust_file.texts[unit.last_token] != "}":
        return None
    return next((opening for opening, closing in rust_file.pairs.items() if closing == unit.last_token), None)


def whole_entry_end(unit, cfg_atom=False):
    """取整个条目当规格原文时的上界（uninterp、external_*_specification、带 external_body 的类型规格，与 cfg 原子的「所在条目整个」）：
    单元是半开区间 [first_token, last_token)，取到 last_token 为止、不连下一个条目的头一个词。
    弄坏开关 whole-entry-past-unit-end 下几处都照改前的取法多取 last_token 那一个词；cfg-entry-past-unit-end 下只 cfg 原子那一处多取（cfg_atom 为真的调用）。"""
    if "whole-entry-past-unit-end" in BREAK or cfg_atom and "cfg-entry-past-unit-end" in BREAK:
        return unit.last_token + 1
    return unit.last_token


def region_spans(rust_file):
    """被验区域在这份文件里的词法单元区间：每个 verus! { … }，与属性写法标的条目（attribute_verified）。"""
    spans = list(verus_ranges(rust_file))
    for unit in rust_file.units:
        if not unit.data and unit.first_token is not None and unit.last_token is not None and attribute_verified(rust_file, unit):
            spans.append((unit.first_token, unit.last_token))
    return spans


def assume_specification_parts(rust_file, index, limit):
    """index 是一处 assume_specification 那个词：交 (点名的条目, 方括号 ] 的下标或 None, 同一层的 ; 的下标)。点名的条目是方括号里那一项的词法单元连写
    （例 <[T]>::binary_search、singlefs_format::SLOT_BYTES），认不出方括号时是「（认不出方括号里的那一项）」。信任清单（trusted_declarations）与证明成项（proof_items）共用。"""
    texts = rust_file.texts
    opening = next((position for position in range(index + 1, min(index + 40, rust_file.count)) if texts[position] == "["), None)
    closing = rust_file.pairs.get(opening) if opening is not None else None
    point = "".join(texts[opening + 1:closing]) if closing is not None else "（认不出方括号里的那一项）"
    return point, closing, statement_end(rust_file, index, limit)


def trusted_declarations(rust_file, relative, crate, module, spans, cfg_macros):
    """被验区域里每一处要登记的可信声明与 cfg 原子：[{kind, file, point, digest, line}]，次序照文件里的位置。
    assume_specification：点名的是方括号里那一项（词法单元连写，例 <[T]>::binary_search），规格原文是从它到同一层的 ; 为止；
    external_body：点名的是带它的 fn 的全路径，规格原文是函数体的 { 之前那一截（签名、requires、ensures；函数体的 { 照 body_open 认，
    一个 impl 里最后一个 fn 也取得到签名，改签名 sha256 就变、只改函数体不变）；带它的不是 fn（类型规格）时规格原文是整个条目；
    uninterp 与 external_*_specification：点名的是那个条目的全路径，规格原文是整个条目（整个条目都照半开区间取到 whole_entry_end，不连下一个条目的头一个词）；
    cfg 原子（#[cfg(…)]、#[cfg_attr(…)]、cfg!(…)，与定义里有 cfg 的 macro_rules! 宏在被验区域里的调用）：点名的是 <所在条目的全路径>#<原子>（例 crate::m::pick#cfg(feature="wide")、
    crate::m::pick#plus_span!），规格原文是所在条目整个（与 uninterp 同一种半开区间，取到 whole_entry_end）。"""
    found = []
    seen = set()
    texts = rust_file.texts
    for low, high in spans:
        for index in range(low, min(high + 1, rust_file.count)):
            word = texts[index]
            if rust_file.kinds[index] != "ident" or (index, word) in seen or not (word in TRUST_KINDS or word in ("cfg", "cfg_attr") or word in cfg_macros):
                continue
            seen.add((index, word))
            unit = innermost_unit(rust_file, index)
            if word in TRUST_KINDS:
                if word == "assume_specification":
                    point, _closing, end = assume_specification_parts(rust_file, index, high)
                    spec = joined_tokens(rust_file, index, end + 1)
                elif word == "external_body":
                    point = unit_path(crate, module, unit)
                    # fn 取签名到函数体的 { 之前；类型规格（external_type_specification 那几个 struct）取整个条目，与别的种类同一种取法
                    if "body-open-past-unit-end" in BREAK:
                        opening = body_open_past_unit_end(rust_file, unit)
                    else:
                        opening = body_open(rust_file, unit) if unit is not None and unit.kind == "fn" else None
                    spec = joined_tokens(rust_file, unit.first_token, opening if opening is not None else whole_entry_end(unit)) if unit is not None else word
                else:
                    point = unit_path(crate, module, unit)
                    spec = joined_tokens(rust_file, unit.first_token, whole_entry_end(unit)) if unit is not None else word
                found.append({"kind": word, "file": relative, "point": point, "digest": hashlib.sha256(spec.encode()).hexdigest(), "line": rust_file.lines[index]})
                continue
            atom = None
            if word in ("cfg", "cfg_attr") and index + 1 < rust_file.count and texts[index + 1] in ("(", "!"):
                opening = index + 1 if texts[index + 1] == "(" else index + 2
                closing = rust_file.pairs.get(opening)
                if closing is not None:
                    atom = word + ("!" if texts[index + 1] == "!" else "") + "".join(texts[opening:closing + 1])
            elif word in cfg_macros and index + 1 < rust_file.count and texts[index + 1] == "!" and "cfg-macro-unchecked" not in FORMAL_BREAK:
                atom = f"{word}!"
            if atom is not None:
                spec = joined_tokens(rust_file, unit.first_token, whole_entry_end(unit, cfg_atom=True)) if unit is not None else atom
                found.append({"kind": CFG_KIND, "file": relative, "point": f"{unit_path(crate, module, unit)}#{atom}",
                              "digest": hashlib.sha256(spec.encode()).hexdigest(), "line": rust_file.lines[index]})
    return found


def region_problems(root, registry, kernels, options):
    """被验区域的几样，交 [(卡在哪, 下一步)]；options["region_note"] 记一句给成功句。
    ① axiom（axiom fn、broadcast axiom）在核里一律判红；② 被验区域里每一处可信声明（TRUST_KINDS）与 cfg 原子都要在信任清单里恰好登记一行：
       种类、文件、点名的条目相同，规格原文的 sha256 与现算的相同，差分测试名在 crates/ 下标了 #[test]；清单里这个 crate 的行在核里找不到那一处的判红（标记集合与清单相等）；
       vstd 恰好一行，版本与内容指纹等于登记表 package vstd 那一行；③ 被验区域里声明的类型与 trait，在 crate 的 src 里 verus! 外面（属性写法也没标）有 impl <Trait> for <类型>
       的判红（cfg(test) 下的不判：不进产品构建），登记在放过表 TRAIT_IMPLS_OUTSIDE_LAG 里的（所在文件、trait 名、实现类型三样对上）放过、成功句逐个点名
       「契约没证、靠信任」；表里这个 crate 的行在源码里已挪进 verus! 或找不到的判红（impl 还在外面、trait 与类型此刻都不在被验区域里的不红，成功句点名），
       trait 写的全路径与被验的那个不同判红，比 gate 基准那一版涨了判红（trait_impl_lag_growth）。
    弄坏开关（FORMAL_VERIFICATION_BREAK）：axiom-accepted 不判 ①；trust-list-unchecked 不判 ② 的条目行；trust-digest-unchecked 不比 sha256；trust-test-unchecked 不核测试名；
    trust-vstd-unchecked 不核 vstd 行；cfg-atom-accepted 不认 cfg 原子；cfg-macro-unchecked 不认宏里的 cfg；trait-impl-outside-accepted 不判 ③（放过表也不读）。
    放过表那一半的弄坏开关是 VERUS_PROOF_CHECK_BREAK 的 trait-impl-lag-matched-by-trait-only、trait-impl-lag-stale-unchecked、trait-impl-lag-growth-unchecked、
    trait-impl-lag-passed-unnamed（文件头逐项写）。"""
    problems = []
    manifest = crate_manifest(root, registry)
    crate = crate_identifier(registry, manifest)
    workspace = unit_index(root, options).workspace()
    crate_files = sorted(os.path.relpath(path, root).replace(os.sep, "/")
                         for path in glob.glob(os.path.join(root, registry.crate_directory, "src", "**", "*.rs"), recursive=True))
    try:
        parsed = {relative: workspace.file(relative) for relative in crate_files}
    except (OSError, ValueError) as error:
        raise Red(f"切不出 {registry.crate_directory} 的源码（scripts/gate/lib_function_units.py）：{error}",
                  "先让 python3 scripts/gate/lib_function_units.py dump <根> --language rust 在这棵树上跑通")
    cfg_macros = set()
    for rust_file in parsed.values():
        for unit in rust_file.units:
            if unit.kind == "macro_rules" and unit.first_token is not None and "cfg" in rust_file.texts[unit.first_token:unit.last_token + 1][2:]:
                cfg_macros.add(unit.short)
    kernel_relatives = [os.path.relpath(kernel, root).replace(os.sep, "/") for kernel in kernels]
    declarations, axioms = [], []
    verified_types, verified_traits, verified_trait_paths = set(), set(), {}
    for relative in kernel_relatives:
        rust_file = parsed.get(relative) or workspace.file(relative)
        module = kernel_module(registry, manifest, relative)
        spans = region_spans(rust_file)
        for index, word in enumerate(rust_file.texts):
            if word == "axiom" and rust_file.kinds[index] == "ident":
                axioms.append(f"{relative}:{rust_file.lines[index]}（{unit_path(crate, module, innermost_unit(rust_file, index))}）")
        declarations += trusted_declarations(rust_file, relative, crate, module, spans, cfg_macros)
        ranges = verus_ranges(rust_file)
        for unit in rust_file.units:
            if unit.container is None and unit.kind in ("struct", "enum", "union", "type", "trait") and in_verified_region(rust_file, unit, ranges):
                (verified_traits if unit.kind == "trait" else verified_types).add(unit.short)
                if unit.kind == "trait":
                    verified_trait_paths.setdefault(unit.short, set()).add(unit_path(crate, module, unit))
    if "axiom-accepted" not in FORMAL_BREAK:
        for where in axioms:
            problems.append((f"核里有 axiom：{where}（axiom fn 与 broadcast axiom 不验、直接当真）",
                             "axiom 一律不许：把它写成 proof fn 证出来；确是可信的外部事实，写成 uninterp spec fn 或 external_body 的 exec fn，逐条登记进信任清单 " + TRUST_LIST))
    cfg_declarations = [entry for entry in declarations if entry["kind"] == CFG_KIND]
    if "cfg-atom-accepted" in FORMAL_BREAK:
        declarations = [entry for entry in declarations if entry["kind"] != CFG_KIND]
        cfg_declarations = []
    listed = read_trust_list(root)
    checked_rows = 0
    if "trust-list-unchecked" not in FORMAL_BREAK or cfg_declarations:
        if listed is None:
            problems.append((f"信任清单 {TRUST_LIST} 不在（被验区域里要登记的 {len(declarations)} 处：{listed_names(entry['kind'] + ' ' + entry['point'] for entry in declarations)}）",
                             f"建这张表：头一行 vstd、版本、内容指纹、理由（版本与指纹照 {REGISTRY} 的 package vstd 行），再逐处登记；{TRUST_ROW_HOWTO}"))
        else:
            vstd_rows, rows, written_problems = listed
            problems += [(problem, f"照 {TRUST_LIST} 文件头的格式改") for problem in written_problems]
            vstd_package = next((package for package in registry.packages if package["name"] == "vstd"), None)
            if "trust-vstd-unchecked" not in FORMAL_BREAK:
                if len(vstd_rows) != 1:
                    problems.append((f"信任清单里 vstd 行有 {len(vstd_rows)} 行，要恰好一行（vstd 按版本整体登记）", f"照 {REGISTRY} 的 package vstd 行写一行 vstd、版本、内容指纹、理由"))
                elif vstd_package is None or (vstd_rows[0][0], vstd_rows[0][1]) != (vstd_package["version"], vstd_package["fingerprint"]):
                    problems.append((f"信任清单第 {vstd_rows[0][2]} 行的 vstd {vstd_rows[0][0]}（指纹 {vstd_rows[0][1][:12]}…）与登记表 package vstd 行"
                                     f"（{vstd_package['version'] + '，指纹 ' + vstd_package['fingerprint'][:12] + '…' if vstd_package else '没有这一行'}）不同",
                                     "换了 vstd 就照新的 package vstd 行一起改这一行（信任的是那一版 vstd 的全部规格），并复核同源"))
            mine = [row for row in rows if row["file"].startswith(registry.crate_directory + "/")]
            if registry.all_crate_directories and registry.crate_directory == registry.all_crate_directories[0]:
                for row in rows:
                    if not any(row["file"].startswith(directory + "/") for directory in registry.all_crate_directories):
                        problems.append((f"信任清单第 {row['number']} 行的文件 {row['file']} 不在登记的哪个 crate 里", "删掉这一行，或照新路径改"))
            tests = test_functions(root) if "trust-test-unchecked" not in FORMAL_BREAK else set()
            used = set()
            for entry in declarations:
                if entry["kind"] != CFG_KIND and "trust-list-unchecked" in FORMAL_BREAK:
                    continue
                matched = [row for row in mine if (row["kind"], row["file"], row["point"]) == (entry["kind"], entry["file"], entry["point"])]
                wanted_row = f"{entry['kind']}\\t{entry['file']}\\t{entry['point']}\\t{entry['digest']}\\t<差分测试名>\\t<理由>"
                if not matched:
                    problems.append((f"{entry['file']}:{entry['line']} 的 {entry['kind']} {entry['point']} 没登记进信任清单 {TRUST_LIST}（这一处的规格原文 sha256 {entry['digest']}）",
                                     f"{TRUST_ROW_HOWTO}；这一处那一行：{wanted_row}；cfg 原子能不用就不用（被验区域里一律判红，确要的才登记）"))
                    continue
                if len(matched) > 1:
                    problems.append((f"信任清单第 {'、'.join(str(row['number']) for row in matched)} 行登记的是同一处 {entry['kind']} {entry['point']}", "留一行"))
                row = matched[0]
                used.add(row["number"])
                checked_rows += 1
                if row["digest"] != entry["digest"] and "trust-digest-unchecked" not in FORMAL_BREAK:
                    problems.append((f"信任清单第 {row['number']} 行 {entry['kind']} {entry['point']} 登记的规格原文 sha256 是 {row['digest'][:12]}…，现算的是 {entry['digest']}"
                                     f"（{entry['file']}:{entry['line']} 的规格改过）",
                                     "规格改了就先让那条差分测试对着新规格跑过，再照现算的 sha256 改这一行；不是有意改的就照 git 找回原来的规格"))
                if "trust-test-unchecked" not in FORMAL_BREAK and row["test"].split("::")[-1] not in tests:
                    problems.append((f"信任清单第 {row['number']} 行 {entry['kind']} {entry['point']} 点名的差分测试 {row['test']} 在 crates/ 下找不到标了 #[test] 的函数",
                                     "先写那条差分测试（把这份规格与真实现在一批输入上逐值比，规格写错时它要红），再照它的函数名改这一行"))
            if "trust-list-unchecked" not in FORMAL_BREAK:
                for row in mine:
                    if row["number"] not in used and (row["kind"] != CFG_KIND or "cfg-atom-accepted" not in FORMAL_BREAK):
                        problems.append((f"信任清单第 {row['number']} 行登记了 {row['kind']} {row['point']}（{row['file']}），核的被验区域里没有这一处",
                                         "那一处删了或改了名：删掉这一行，或照新的点名改（信任清单与被验区域里的标记一一对应）"))
    trait_impls, passed, outside_note = [], [], "verus! 外面的 trait impl 0 处"
    if "trait-impl-outside-accepted" not in FORMAL_BREAK:
        lag_rows, lag_problems = read_trait_impls_outside_lag(root)
        problems += [(problem, f"照 {TRAIT_IMPLS_OUTSIDE_LAG} 文件头的格式改") for problem in lag_problems]
        mine_lag = [row for row in lag_rows if row["file"].startswith(registry.crate_directory + "/")]
        if registry.all_crate_directories and registry.crate_directory == registry.all_crate_directories[0]:
            for row in lag_rows:
                if not any(row["file"].startswith(directory + "/") for directory in registry.all_crate_directories):
                    problems.append((f"{TRAIT_IMPLS_OUTSIDE_LAG} 第 {row['number']} 行的文件 {row['file']} 不在登记的哪个 crate 里", "删掉这一行，或照新路径改"))
        inside_impls, unverified_impls, used_rows = set(), set(), set()
        for relative, rust_file in parsed.items():
            ranges = verus_ranges(rust_file)
            for unit in rust_file.units:
                if unit.kind != "impl" or unit.first_token is None:
                    continue
                trait, target = impl_parts(unit.name[unit.name.rfind("<impl"):] if "<impl" in unit.name else unit.name)
                if trait is None or any(re.search(r"\btest\b", str(condition[1] if isinstance(condition, tuple) else condition)) for condition in unit.conditions):
                    continue
                if in_verified_region(rust_file, unit, ranges):
                    inside_impls.add((trait, target, relative))
                    continue
                if not (trait in verified_traits or target in verified_types):
                    unverified_impls.add((trait, target, relative))
                    continue
                where = f"{relative}:{unit.first_line} {unit.name}"
                # 登记行按（所在文件、trait 名、实现类型）认；弄坏开关 trait-impl-lag-matched-by-trait-only 下不看实现类型（同一个 trait 的别的 impl 也放过）
                row = next((row for row in mine_lag if row["file"] == relative and row["trait"].split("::")[-1] == trait
                            and (row["type"] == target or "trait-impl-lag-matched-by-trait-only" in BREAK)), None)
                if row is None:
                    trait_impls.append(where)
                    continue
                used_rows.add(row["number"])
                if trait in verified_trait_paths and row["trait"] not in verified_trait_paths[trait]:
                    problems.append((f"{TRAIT_IMPLS_OUTSIDE_LAG} 第 {row['number']} 行写的 trait 是 {row['trait']}，被验的那个是 {'、'.join(sorted(verified_trait_paths[trait]))}（{where}）",
                                     "照被验区域里那个 trait 的全路径（<crate>::<模块>::<trait>）改这一行"))
                    continue
                passed.append((row, where))
        for where in trait_impls:
            problems.append((f"被验的类型或 trait 在 verus! 外面有 trait impl：{where}（verus! 外面的 impl Verus 不看，trait 的 ensures 在这里没人核）", TRAIT_IMPL_OUTSIDE_HOWTO))
        unneeded = []
        for row in mine_lag:
            if row["number"] in used_rows:
                continue
            key = (row["trait"].split("::")[-1], row["type"], row["file"])
            if key in unverified_impls:
                unneeded.append(row)   # impl 还在外面，它的 trait 与类型此刻都不在被验区域里（那一簇的补丁还没打上）：没放过什么，成功句点名
                continue
            if "trait-impl-lag-stale-unchecked" in BREAK:
                continue
            if key in inside_impls:
                problems.append((f"{TRAIT_IMPLS_OUTSIDE_LAG} 第 {row['number']} 行登记放过的 {row['trait']} for {row['type']}（{row['file']}）已经挪进 verus! 里了",
                                 "删掉这一行（表只缩不涨，挪进去的那一行就该删）"))
            else:
                problems.append((f"{TRAIT_IMPLS_OUTSIDE_LAG} 第 {row['number']} 行登记放过的 {row['trait']} for {row['type']}（{row['file']}）在那份文件里找不到"
                                 "（verus! 外面与里面都没有这个 impl）",
                                 "impl 删了就删掉这一行；改了名、挪了文件的，照现在的写法改这一行（不许借这一行放过别的 impl）"))
        growth_note = ""
        if mine_lag:
            growth_problems, growth_note = trait_impl_lag_growth(root, mine_lag)
            problems += growth_problems
        options["trait_impls_passed"] = passed   # check_proof 判完往 $GATE_NOT_RUN_FILE 逐个报本次未跑（report_passed_not_run）
        if passed:
            named = ("、".join(f"{row['trait']} for {row['type']}（{where.split(' ', 1)[0]}，契约没证、靠信任，归「{row['cluster']}」挪进 verus!）" for row, where in passed)
                     if "trait-impl-lag-passed-unnamed" not in BREAK else f"{len(passed)} 处")
            outside_note = f"verus! 外面的 trait impl {len(passed)} 处，都登记在 {TRAIT_IMPLS_OUTSIDE_LAG} 里放过、这一格不算绿到底：{named}"
        if unneeded:
            outside_note += (f"；放过表里 {len(unneeded)} 行的 trait 与类型此刻都不在被验区域里、没放过什么："
                             + "、".join(f"第 {row['number']} 行 {row['trait']} for {row['type']}" for row in unneeded))
        if mine_lag:
            outside_note += f"（放过表：{growth_note}）"
    cfg_count = sum(1 for entry in declarations if entry["kind"] == CFG_KIND)
    options["region_note"] = (f"被验区域：可信声明与 cfg 原子 {len(declarations)} 处都登记在信任清单里（其中 cfg 原子 {cfg_count} 处）、核对了 {checked_rows} 行，"
                              f"axiom 0 处，被验类型与 trait {len(verified_types) + len(verified_traits)} 个、{outside_note}，vstd 按版本整体登记一行")
    return problems


# ── 产品构建与 verus --compile 构建的 MIR 逐函数比（出口 5 第三轮判决 Z2：cfg 分叉、宏里的 cfg 让两边编出的函数体不同，两次产品编译看不见）──

MIR_ITEM_HEAD = re.compile(r"^(fn|const|static|promoted\[\d+\] in) ")
# MIR 规整表：同一个函数两边印法不同的名字，(另一种印法, 规整成的印法)；比之前两边都把前一个整词写成后一个（mir_path_pattern 卡词边界）。
# std::rt::panic_fmt 是 core::panicking::panic_fmt 的再导出：Verus 构建里 verus! 的展开经 ::core 用了一次 core，rustc 印路径时改取 core 自己的路径，
# 产品构建照旧印 std 的再导出。里程碑四出口 5 调查员 2026-10-29 在 core 上现量：103 个条目 124 行不同全是这一处，两份 rlib 引用的都是 core::panicking::panic_fmt 的符号（nm），
# 规整之后 5956 个条目逐字相同，在 verus! 外面造的一处真分叉规整之后照报。表外的名字照红：核过两边是同一个函数（rlib 引用的符号相同）再加一行，不按前缀整片当成同一个。
MIR_PATH_ALIASES = (("std::rt::panic_fmt", "core::panicking::panic_fmt"),)


def mir_path_pattern(alias):
    """规整表一行的认法：整词，前面不接标识符字符或 :，后面接 ( 或 ::<（调用与带类型参数的调用）。
    弄坏开关：mir-alias-without-boundary 下不卡词边界（前后接什么都改）；mir-alias-by-prefix 下按路径前缀认（std::rt::<名> 一律当 core::panicking::<名>）。"""
    if "mir-alias-by-prefix" in BREAK:
        return re.compile(r"(?<![A-Za-z0-9_:])" + re.escape(alias.rsplit("::", 1)[0] + "::") + r"(?=[A-Za-z_])")
    if "mir-alias-without-boundary" in BREAK:
        return re.compile(re.escape(alias))
    return re.compile(r"(?<![A-Za-z0-9_:])" + re.escape(alias) + r"(?=\(|::<)")


def normalized_mir(text):
    """照 MIR_PATH_ALIASES 规整一份 MIR 文本：交 (规整后的文本, 改写了几处)。弄坏开关 mir-panic-path-unnormalized 下不改（改前的样子）。"""
    if "mir-panic-path-unnormalized" in BREAK:
        return text, 0
    total = 0
    for alias, canonical in MIR_PATH_ALIASES:
        replacement = canonical.rsplit("::", 1)[0] + "::" if "mir-alias-by-prefix" in BREAK else canonical
        text, count = mir_path_pattern(alias).subn(replacement.replace("\\", "\\\\"), text)
        total += count
    return text, total


def mir_alias_text(product_rewrites, verus_rewrites):
    """成功句与红句里报规整了几处：表里每一行写成「甲 → 乙」，两边各几处。"""
    table = "、".join(f"{alias} → {canonical}" for alias, canonical in MIR_PATH_ALIASES)
    if "mir-panic-path-unnormalized" in BREAK:
        return f"没照规整表改写（弄坏开关 mir-panic-path-unnormalized；表里 {table}）"
    return f"比之前照规整表改写了产品构建 {product_rewrites} 处、Verus 构建 {verus_rewrites} 处（{table}，同一个函数的两种印法）"


def mir_items(text):
    """--emit=mir 的文本按条目切：{条目名: [整段原文…]}（fn 的名字取到 ( 之前；同名的几段按出现的次序收）。"""
    items, current, buffer = {}, None, []
    for line in text.split("\n"):
        if current is None:
            if MIR_ITEM_HEAD.match(line) and line.endswith("{"):
                current = re.sub(r"\(.*", "", line) if line.startswith("fn ") else line
                buffer = [line]
            continue
        buffer.append(line)
        if line == "}":
            items.setdefault(current, []).append("\n".join(buffer))
            current = None
    return items


def newest_mir(target, identifier):
    """编译目录 debug/deps 下这个 crate 的 .mir（几份取最新的一份）；没有交 None。"""
    found = sorted(glob.glob(os.path.join(target, "debug", "deps", f"{identifier}-*.mir")), key=os.path.getmtime)
    return found[-1] if found else None


def compare_mir(root, registry, manifest, executable, options):
    """产品构建（cargo rustc --emit=mir，产品的 rustc；路径不裁短：RUSTC_BOOTSTRAP=1 -Z trim-diagnostic-paths=false，与 Verus 印的同一种写法）与
    Verus 构建（直接起的 verus --no-verify --compile -- --emit=mir；cargo 那一种 cargo verus build -- --no-verify -- --emit=mir）的 MIR 逐条目比：产品构建的每个条目在 Verus 构建里都有、
    整段逐字相同；Verus 构建多出来的（spec fn、proof fn 擦出来的空壳）不比。比之前两边照 MIR_PATH_ALIASES 规整（normalized_mir），成功句与红句报各改写几处。
    交 ([(卡在哪, 下一步)], 一句给成功句)。弄坏开关 mir-aliased-items-skipped 下改写过的条目整个不比（真分叉也放过）。
    两边各在这一份副本里编，编译目录各一份，比完删。弄坏开关 FORMAL_VERIFICATION_BREAK=mir-compare-skipped 下不比（改前的样子）。"""
    if "mir-compare-skipped" in FORMAL_BREAK:
        return [], "MIR 比对没跑（弄坏开关 mir-compare-skipped）"
    identifier = crate_identifier(registry, manifest)
    package = crate_package_name(registry)
    howto = ("两边编出的函数体不同：多半是 cfg 原子或宏里的 cfg 让 Verus 看的与产品编的不是同一份（verus_keep_ghost、feature、debug_assertions），"
             "把分叉去掉；确要的分叉连同它登记的那一行一起重想（信任清单只管规格，不替两边的执行代码作保）")
    with tempfile.TemporaryDirectory(prefix="verus-proof-check-mir-") as work:
        copy = copy_crate(root, registry, work)
        if not registry.cargo_mode:
            # 直接起的那一种：产品构建在 crate 副本里单独成一个工作区编（MIR 里 impl 的位置写 src/…，与 verus 在 crate 副本里起印的相同）
            with open(os.path.join(copy, "Cargo.toml"), "a", encoding="utf-8") as handle:
                handle.write("\n# verus-proof-check.py 的 MIR 比对：副本自成一个工作区\n[workspace]\n")
            if os.path.isfile(os.path.join(root, "Cargo.lock")):
                shutil.copy2(os.path.join(root, "Cargo.lock"), os.path.join(copy, "Cargo.lock"))
        product_target, verus_target = os.path.join(work, "product-target"), os.path.join(work, "verus-target")
        # 产品的 cargo 照 PATH 找（自证把假 cargo 放在 PATH 最前面）
        product = run_verus("cargo", ["rustc", "-p", package, "--lib", "--offline", "--", "--emit=mir", "-Z", "trim-diagnostic-paths=false"], copy,
                            extra_environment={"CARGO_TARGET_DIR": product_target, "RUSTC_BOOTSTRAP": "1"})
        if registry.cargo_mode:
            built = run_verus(verus_program(registry, executable),
                              verus_program_arguments(registry, verus_arguments(registry, manifest, options["threads"]) + ["--no-verify", "--", "--emit=mir"], build=True),
                              copy, extra_environment={"CARGO_TARGET_DIR": verus_target})
            verus_mir = newest_mir(verus_target, identifier)
        else:
            verus_mir = os.path.join(work, "verus.mir")
            built = run_verus(verus_program(registry, executable),
                              verus_program_arguments(registry, verus_arguments(registry, manifest, options["threads"]) + ["--no-verify", "--compile", "--", "--emit=mir", "-o", verus_mir]),
                              copy)
        problems = []
        verus_label = "Verus 构建（cargo verus build -- --no-verify -- --emit=mir）" if registry.cargo_mode else "Verus 构建（verus --no-verify --compile -- --emit=mir）"
        for label, outcome in (("产品构建（cargo rustc --emit=mir）", product), (verus_label, built)):
            if killed_signal(outcome) is not None or outcome["exit"] != 0:
                tail = " | ".join(line for line in outcome.get("diagnostics", "").strip().split("\n")[-6:] if line.strip())
                problems.append((f"MIR 比对：{label} 没编成（退 {outcome['exit']}）：{outcome_problem(outcome, options) or tail[-400:]}",
                                 problem_howto(outcome, "照报错修；产品构建编不过先让 cargo build 过，Verus 构建编不过先看完整 crate 证明那一步")))
        if problems:
            return problems, ""
        product_mir = newest_mir(product_target, identifier)
        if product_mir is None or verus_mir is None or not os.path.isfile(verus_mir):
            return [(f"MIR 比对：找不到 MIR 文件（产品构建 {product_mir or '没有'}，Verus 构建 {verus_mir or '没有'}）",
                     "cargo 或 Verus 换了 --emit=mir 的落点：照新落点改 scripts/gate/verus-proof-check.py 的 newest_mir")], ""
        product_text = open(product_mir, encoding="utf-8", errors="replace").read()
        verus_text = open(verus_mir, encoding="utf-8", errors="replace").read()
    # 同一个函数两边印法不同的名字先照规整表写成一种（MIR_PATH_ALIASES），函数体别处一个字都不碰
    product_normalized, product_rewrites = normalized_mir(product_text)
    verus_normalized, verus_rewrites = normalized_mir(verus_text)
    product_items, verus_items = mir_items(product_normalized), mir_items(verus_normalized)
    aliases = mir_alias_text(product_rewrites, verus_rewrites)
    missing = [name for name in product_items if name not in verus_items]
    different = [name for name in product_items if name in verus_items and product_items[name] != verus_items[name]]
    if "mir-aliased-items-skipped" in BREAK:
        # 弄坏开关：改写过的条目整个当相同、不比（同一个条目里另有真分叉也放过）
        raw_items = mir_items(product_text)
        different = [name for name in different if raw_items.get(name) == product_items[name]]
    if not product_items:
        return [("MIR 比对：产品构建的 MIR 里一个条目都切不出来", "看 mir_items 认的条目头（fn / const / static / promoted）还对不对得上 rustc 的印法")], ""
    if missing or different:
        return [(f"MIR 比对：产品构建 {len(product_items)} 个条目里，{len(different)} 个与 Verus 构建的函数体不同（{listed_names(different, 10)}），"
                 f"{len(missing)} 个在 Verus 构建里没有（{listed_names(missing, 10)}）；{aliases}",
                 howto + "；不同行只差一个名字的路径（std:: 对 core:: 这类），先核两边是不是同一个函数（两份 rlib 引用的符号相同），"
                 "是就往 scripts/gate/verus-proof-check.py 的 MIR_PATH_ALIASES 加一行，不按前缀整片放过")], ""
    return [], (f"产品构建与 {'cargo verus build' if registry.cargo_mode else 'verus --compile'} 构建的 MIR 逐条目比：产品的 {len(product_items)} 个条目都在、整段逐字相同"
                f"（Verus 构建另有 {len(verus_items) - len(product_items)} 个擦出来的条目，不比；{aliases}）")


def verus_executable(registry, options):
    directory = verus_directory(registry, options.get("verus_root"))
    executable = os.path.join(directory, "verus")
    version = read_version_json(directory)
    if not os.access(executable, os.X_OK) or version is None:
        raise Red(f"找不到 Verus（{executable} 不可执行或旁边没有 version.json）",
                  f"装登记的发布 {registry.tag}（{REGISTRY} 的 release 行），或用 SINGLEFS_VERUS_ROOT 指到它的安装目录")
    return executable, version


def crate_registry(root, options):
    """这一趟判的那一个 crate 的登记：run_per_crate 交的 options["crate_view"]；没有时读登记表（只登记了一个 crate）。"""
    if options.get("crate_view") is not None:
        return options["crate_view"]
    registry = read_registry(root)
    if len(registry.crates) != 1:
        raise Red(f"登记表 {REGISTRY} 登记了 {len(registry.crates)} 个 crate，这一步只能判一个", "经 run_check 起（它逐个 crate 判）")
    return registry


def prepared(root, options):
    registry = crate_registry(root, options)
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


class VerusFunction(collections.namedtuple("VerusFunction", "name relative unit path")):
    """被验区域里的一个 fn：name 是交给 --verify-function 的名字（自由 fn 是它的名字，impl 与 trait 里的是 <类型或 trait>::<方法>，与 Verus 列的可点名函数同一种写法），
    relative 是核相对仓根的路径，unit 是 lib_function_units 切出的单元（算键的入口是 <核>::<unit.name>），path 是全路径
    <crate>::<模块>::<name>（项名用它，与 Verus 的 func-details 同一种写法；核是 crate 入口的没有模块那一段）。"""


def impl_parts(label):
    """impl 单元的嵌套路径（lib_function_units 的写法：<impl …>，去掉了 impl 后面的泛型参数表与 where 子句）交 (trait 名或 None, 类型名)：都取路径的最后一段、去掉类型参数。
    认不出（&T、dyn、元组这类）交 (trait, None)。"""
    inner = label[len("<impl "):-1] if label.startswith("<impl ") and label.endswith(">") else label
    trait, _for, target = inner.rpartition(" for ") if " for " in inner else ("", "", inner)

    def last_segment(text):
        text = text.strip().split("<", 1)[0].strip()
        segment = text.split("::")[-1].strip()
        return segment if RUST_IDENTIFIER.fullmatch(segment or "") else None
    return (last_segment(trait) if trait else None), last_segment(target)


def unit_attribute_words(rust_file, unit):
    """单元开头那几个外属性 #[…] 里的词（文档注释不算）。"""
    words, index = [], unit.first_token
    while index is not None and index + 1 < rust_file.count and rust_file.texts[index] == "#" and rust_file.texts[index + 1] == "[" and index + 1 in rust_file.pairs:
        close = rust_file.pairs[index + 1]
        words += rust_file.texts[index + 2:close]
        index = close + 1
    return words


def attribute_verified(rust_file, unit):
    """属性写法标的被验单元：它自己或包它的 impl / trait 带 #[verus_verify] 或 #[verus_spec(…)]（弄坏开关 attribute-mode-unrecognized 下不认）。"""
    if "attribute-mode-unrecognized" in FORMAL_BREAK:
        return False
    for owner in (unit, unit.container):
        if owner is not None and owner.first_token is not None and any(word in ("verus_verify", "verus_spec") for word in unit_attribute_words(rust_file, owner)):
            return True
    return False


def in_verified_region(rust_file, unit, ranges):
    """单元在被验区域里：落在某个 verus! { … } 里，或是属性写法标的（attribute_verified）。"""
    if unit.first_token is None:
        return False
    return any(low < unit.first_token < high for low, high in ranges) or attribute_verified(rust_file, unit)


def verus_point_name(unit):
    """交给 --verify-function 的名字：自由 fn 是它的名字；impl 里的方法是 <类型>::<方法>（trait impl 也写类型，Verus 0.2026.09.20 列可点名函数就是这么写的，
    2026-10-26 在草稿的小 crate 上量的：impl Device for GoodDevice 的 read_slot 列成 GoodDevice::read_slot，同一个类型的固有方法与 trait 方法同名时按名点两个一起验）；
    trait 里的方法是 <trait>::<方法>。认不出类型的交 None。
    弄坏开关 methods-refused 下 impl 与 trait 里的一律交 None（改前的样子：判红要挪成自由 fn）。"""
    container = unit.container
    if container is None:
        return unit.short
    if "methods-refused" in FORMAL_BREAK:
        return None
    if container.kind == "trait":
        return f"{container.short}::{unit.short}"
    if container.kind == "impl":
        _trait, target = impl_parts(container.name)
        return f"{target}::{unit.short}" if target else None
    return None


def verus_functions(root, registry, kernels, options):
    """核的被验区域（verus! { … } 里，与属性写法标的）里的每个 fn 单元（spec、proof、exec 都算；impl 与 trait 里的方法也算），按核与行号的次序：[VerusFunction]。
    空真认所属函数、必红世界认受影响的函数用它；证明成哪几项另由 proof_items 从里面挑（spec fn 没有证明义务的不成项，trait 里没有函数体的声明不成项）。
    同一个模块里两个 fn 交给 --verify-function 的名字相同（同一个类型的固有方法与 trait 方法同名，Verus 按名点会把两个一起验）判红，出路是改名。
    认不出类型的 impl（&T、dyn 这类）里的方法判红。项名是全路径（VerusFunction.path），不同模块里同名的各是各的项。"""
    index = unit_index(root, options)
    manifest = crate_manifest(root, registry)
    crate = crate_identifier(registry, manifest)
    functions = []
    try:
        workspace = index.workspace()
        for kernel in kernels:
            relative = os.path.relpath(kernel, root).replace(os.sep, "/")
            module = kernel_module(registry, manifest, relative)
            rust_file = workspace.file(relative)
            ranges = verus_ranges(rust_file)
            names = {}
            for unit in rust_file.units:
                if unit.kind != "fn" or not in_verified_region(rust_file, unit, ranges):
                    continue
                if unit.container is not None and unit.container.kind not in ("impl", "trait"):
                    continue   # fn 里套的 fn 不另成函数（随外层验）
                name = verus_point_name(unit)
                if name is None:
                    raise Red(f"{relative}:{unit.first_line} 的 {unit.name} 在 verus! 里的 impl 或 trait 里，认不出交给 --verify-function 的名字"
                              + ("（弄坏开关 methods-refused）" if "methods-refused" in FORMAL_BREAK else "（impl 的类型不是一个具名的类型）"),
                              "impl 写成 impl <Trait> for <类型名> 或 impl <类型名>；确要别的写法，给 verus-proof-check.py 的 verus_point_name 补上认法")
                if name in names:
                    raise Red(f"{relative} 的被验区域里有两个 fn 交给 --verify-function 的名字都是 {name}（第 {names[name].first_line} 行与第 {unit.first_line} 行）："
                              "Verus 按名点会把两个一起验，逐项点不准",
                              "给其中一个改名（同一个类型的固有方法与 trait 方法同名最常见），同步改必红世界表里引它的行")
                names[name] = unit
                functions.append(VerusFunction(name, relative, unit, "::".join(part for part in (crate, module, name) if part)))
    except (OSError, KeyError, ValueError) as error:
        raise Red(f"切不出核里的函数单元（scripts/gate/lib_function_units.py）：{error}", "先让 python3 scripts/gate/lib_function_units.py dump <根> --language rust 在这棵树上跑通")
    return functions


# verus! 里的 derive 只有 Clone 生成 Verus 要验的函数（<类型>::clone）。依据是主 agent 2026-10-15 的真 Verus 探针（0.2026.09.20，逐函数 verified 数在
# research/prompts/pruning-survey-2026-10-15/verus-probe.log）：--verify-function 点不中时 Verus 列的可点名函数是 90 个自由 fn、8 个 <类型>::clone
# （核里 derive(Clone, …) 的正好 8 个类型）与 2 个 CoreFailure::arrow_*（spec，没有义务）；Copy、Debug、PartialEq、Eq 的 derive 不在表里。
# 整 crate 验出 56 条 = 逐函数验出的 48 条 + 这 8 个 clone 各 1 条（推的：clone 那 8 条没有逐个量过）。
# 不同时 derive(Copy) 的也一样成项：Verus 对它们报「autoderive Clone impl when the clone is not a copy … without adding a specification」，不给规格，
# 函数体照验、各 1 条义务（里程碑四出口 5 收尾工具员 2026-10-29 用 0.2026.09.20 在探针 crate 上逐个 --verify-function 现量：Copy 的、只有 u64 字段不带 Copy 的、
# 带 Vec 字段的、带 Vec 字段又标了 verifier::allow(autoderive_clone_without_spec) 的、带泛型参数的、不带 Copy 的枚举，<类型>::clone 都是 verified 1；整 crate 9 条 = 1 个 exec fn + 8 个 clone）。
DERIVED_VERIFIED_TRAITS = ("Clone",)
PROOF_FN, EXEC_FN, SPEC_FN_WITH_DECREASES, DERIVED_CLONE = "proof fn", "exec fn", "带 decreases 的 spec fn", "derive(Clone) 生成的 clone"
# Verus 整 crate 验时另算条数的两类（里程碑四出口 5 规格底座，工具实现员 2026-10-29 在 core 的草稿副本与探针 crate 上用 Verus 0.2026.09.20 现量）：
# verus! 里带值的 const（impl 里的关联 const 也算）各 1 条，可点名函数表里自由 const 列成它的名字、trait 里的列成 <trait>::<名>、
#   impl 里的（固有 impl 与 trait impl）列成 impl&%<号>::<名>（号数这个模块里的全部 impl：2026-10-29 的探针里 verus! 外面一个 impl、一个 derive(Clone, Copy) 的类型
#   排在前面，verus! 里第一个 impl 就是 impl&%4），按 *::<名> 点（valued_const_point_name；探针 crate 的根模块里关联 const FIRST 与自由 const FIRST_ROOT_COUNTER 同在，
#   点 *::FIRST 验出 1 条，点 FIRST 报 more than one match；同一个模块里两个 impl 各有一个 FIRST 时 *::FIRST 两个一起验、验出 2 条）；
# assume_specification 各 0 或 1 条：指向常量、写 returns 的 1 条（core 的 singlefs_format::SLOT_BYTES 点它验出 1 条，探针 core::u8::MAX 也是 1 条），
#   指向函数的（<[T]>::binary_search、u64::reverse_bits 写 returns）与指向常量写 ensures 的 0 条；可点名函数表里外部路径照写（singlefs_format::SLOT_BYTES），
#   std 的方法列成 core::slice::impl&%0::binary_search 这种，点最后一段 binary_search 点得中。
VALUED_CONST, ASSUME_SPECIFICATION = "带值的 const", "assume_specification"
PLAIN_PATH = re.compile(r"[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*")


class DeclarationSite(collections.namedtuple("DeclarationSite", "first_token first_line")):
    """assume_specification 那一项在核里的位置（它不是 lib_function_units 切出的单元）：排序用词法单元下标，列项用行号。"""


class ProofItem(collections.namedtuple("ProofItem", "item verus_name relative unit kind entry estimate")):
    """证明的一项：项名 proof:<verus_name>、交给 --verify-function 的名字、核的相对路径、单元、哪一类（PROOF_FN 这几样）、
    算键的入口（<核>::<单元名>：fn 是它自己，clone 是那个类型，带值的 const 是它自己，assume_specification 是 lib_function_units 切出的那一条单元，找不到交 None、那一项拒算）、
    静态估的义务条数（只给计数之和对不上时点名用，不判每一项）。assume_specification 那一项的 unit 是 DeclarationSite。"""


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
    不成项，随用到它的项进那一项的单元闭包键；trait 里没有函数体的声明与 external_body 的 fn 也不成项。项名 proof:<全路径>（<crate>::<模块>::<名>），
    交给 --verify-function 的 <名> 是 Verus 可点名函数表里恰好那一个（自由 fn 的名字、<类型>::<方法>、<trait>::<方法>、<类型>::clone），
    Verus 有恰好同名的就取它，不会点到别人。另两类不是 fn、Verus 整 crate 验时照样各算条数：verus! 里带值的 const（valued_const_entries）与
    assume_specification（assume_specification_entries），各一项。弄坏开关：spec-fn-as-item 下 spec fn 一律成项（改前的样子），spec-decreases-not-item 下带 decreases 的 spec fn
    不成项，derived-clone-not-item 下 derive(Clone) 不成项，valued-const-not-item、assume-specification-not-item 下那两类不成项。"""
    workspace = unit_index(root, options).workspace()
    manifest = crate_manifest(root, registry)
    crate = crate_identifier(registry, manifest)
    by_kernel = {}
    for function in functions:
        by_kernel.setdefault(function.relative, []).append(function)
    items, left_out = [], []
    for kernel in kernels:
        relative = os.path.relpath(kernel, root).replace(os.sep, "/")
        rust_file = workspace.file(relative)
        module = kernel_module(registry, manifest, relative)
        ranges = verus_ranges(rust_file)
        entries = []
        for function in by_kernel.get(relative, []):
            name, unit = function.name, function.unit
            mode = function_mode(rust_file, unit)
            body = rust_file.texts[unit.first_token:unit.last_token]
            # trait 里只有签名与契约、没有函数体的声明：Verus 不验它（义务落在各 impl 上）。按切出的单元认（body_open：单元以 ; 收尾），
            # ensures 里的结构体字面量不算函数体；弄坏开关 trait-declaration-by-brace 下退回「单元里一个 { 都没有」（改前的样子）
            declared_only = ("{" not in body) if "trait-declaration-by-brace" in BREAK else body_open(rust_file, unit) is None
            if unit.container is not None and unit.container.kind == "trait" and declared_only:
                left_out.append((function.path, relative))
                continue
            if "external_body" in unit_attribute_words(rust_file, unit):
                left_out.append((function.path, relative))   # external_body：函数体不验，规格照信任清单登记（region_problems），不成项
                continue
            if mode == "proof":
                kind, estimate = PROOF_FN, 1
            elif mode == "exec":
                kind, estimate = EXEC_FN, 1 + sum(1 for text in body if text in ("while", "loop", "for"))
            elif "decreases" in body and "spec-decreases-not-item" not in BREAK:
                kind, estimate = SPEC_FN_WITH_DECREASES, 1
            elif "spec-fn-as-item" in BREAK:
                kind, estimate = "spec fn", 0
            else:
                left_out.append((function.path, relative))
                continue
            entries.append((unit.first_token, ProofItem(proof_item(function.path), name, relative, unit, kind, f"{relative}::{unit.name}", estimate)))
        if "derived-clone-not-item" not in BREAK:
            for unit in rust_file.units:
                if unit.kind not in ("struct", "enum", "union") or unit.first_token is None or unit.container is not None \
                        or not any(low < unit.first_token < high for low, high in ranges):
                    continue
                for trait in DERIVED_VERIFIED_TRAITS:
                    derived = derived_traits(rust_file, unit)
                    # 弄坏开关 derived-clone-copy-only：只认同时 derive(Copy) 的（把 Verus 那条「clone is not a copy」的警告当成没有义务）
                    if trait in derived and ("derived-clone-copy-only" not in BREAK or "Copy" in derived):
                        verus_name = f"{unit.short}::{trait.lower()}"
                        full = "::".join(part for part in (crate, module, verus_name) if part)
                        entries.append((unit.first_token, ProofItem(proof_item(full), verus_name, relative, unit, DERIVED_CLONE, f"{relative}::{unit.short}", 1)))
        entries += valued_const_entries(rust_file, relative, crate, module, ranges)
        entries += assume_specification_entries(rust_file, relative, crate, module)
        point_name_problems(relative, [item for _position, item in entries], verus_listing(rust_file, relative, ranges))
        items += [item for _position, item in sorted(entries, key=lambda pair: pair[0])]
    return items, left_out


def valued_const_entries(rust_file, relative, crate, module, ranges):
    """被验区域里带值的 const 各一项（VALUED_CONST，静态估 1 条）：自由的与 impl、trait 里的关联 const 都算，spec const 也算；没写值的（trait 里的声明）不成项
    （也不进「没成项的 verus! fn」那一份清单：那一份只列 fn）。
    spec const 照真 Verus 0.2026.09.20（里程碑四收口工具员 2026-10-29 在探针 crate 上现量，草稿 m4-close-formal-judge-fix/spec-const-probe）：自由的整 crate 验多 1 条
    （去掉它 19 条变 18 条），可点名函数表列成 <名>，点它验出 1 条；impl 里的整 crate 也多 1 条，列成 impl&%<号>::<名>，点 *::<名> 验出 1 条，与 impl 里的关联 const 一样；
    trait 里写 spec const Verus 报语法错（expected `!`），写不出来。所以 spec const 与别的带值的 const 同一种成项、同一种点名，没有点不了名、要另算进之和的。
    弄坏开关 spec-const-not-item 下 spec const 不成项（改前的样子：之和少 1）。
    交给 --verify-function 的照 valued_const_point_name（自由的写名字、trait 里的写 <trait>::<名>、impl 里的写 *::<名>）；项名的全路径是
    <crate>::<模块>::<类型或 trait>::<名>（自由 const 没有类型那一段）。算键的入口是 <核>::<单元名>，与 fn 同一种。弄坏开关 valued-const-not-item 下一项都不成（改前的样子）。"""
    if "valued-const-not-item" in BREAK:
        return []
    entries = []
    for unit in rust_file.units:
        if unit.kind != "const" or unit.first_token is None or not in_verified_region(rust_file, unit, ranges):
            continue
        container = unit.container
        if container is not None and container.kind not in ("impl", "trait"):
            continue   # fn 里的 const 随外层验
        owner = None if container is None else container.short if container.kind == "trait" else impl_parts(container.name)[1]
        if container is not None and owner is None:
            raise Red(f"{relative}:{unit.first_line} 的 const {unit.short} 在 verus! 里的 impl 里，认不出 impl 的类型",
                      "impl 写成 impl <Trait> for <类型名> 或 impl <类型名>；确要别的写法，给 verus-proof-check.py 的 valued_const_entries 补上认法")
        shown = f"{owner}::{unit.short}" if owner else unit.short
        full = "::".join(part for part in (crate, module, shown) if part)
        words = rust_file.texts[unit.first_token:unit.last_token]
        if "=" not in words or ("spec-const-not-item" in BREAK and "spec" in leading_words(rust_file, unit, "const")):
            continue
        entries.append((unit.first_token, ProofItem(proof_item(full), valued_const_point_name(unit), relative, unit, VALUED_CONST, f"{relative}::{unit.name}", 1)))
    return entries


def valued_const_point_name(unit):
    """带值的 const 交给 --verify-function 的名字（Verus 0.2026.09.20 的可点名函数表，2026-10-29 在探针 crate 上现量）：自由 const 列成 <名>，照写；
    trait 里带默认值的列成 <trait>::<名>，照写；impl 里的（固有 impl 与 trait impl）列成 impl&%<号>::<名>，号数这个模块里的全部 impl（verus! 外面的、derive 生成的也数），
    从源码算不准，写 *::<名>（以 ::<名> 收尾的都点中、一起验；同一个模块里只有它一个以 ::<名> 收尾才点得准，point_name_problems 判）。
    <类型>::<名> 与带模块、带 crate 的全路径 Verus 都报 could not find。弄坏开关 associated-const-bare-name 下一律交 <名>（改前的样子：
    与以 <名> 起头的自由 const 同在一个模块时 Verus 报 more than one match）。"""
    container = unit.container
    if container is None or "associated-const-bare-name" in BREAK:
        return unit.short
    if container.kind == "trait":
        return f"{container.short}::{unit.short}"
    return f"*::{unit.short}"


def verus_listing(rust_file, relative, ranges):
    """一个核（一个模块）在 Verus 可点名函数表里的样子：[(列出的名字, 说明)]，说明写它在核里的位置。照 Verus 0.2026.09.20 列的（2026-10-29 探针现量）：
    被验区域里的每个 fn（spec、proof、exec，trait 里的声明与 external_body 的也列）照 verus_point_name；verus! 里 derive(Clone) 的类型列 <类型>::clone；
    const（带不带值、spec 不 spec 都列）自由的列 <名>、trait 里的列 <trait>::<名>、impl 里的列 impl&%?::<名>（号算不准，写 ?：点名里不出现号，只按收尾比）；
    enum 变体的字段 Verus 另生成的 spec fn 照 enum_arrow_names 列；
    assume_specification 点名的条目只有标识符与 :: 的列它自己，别的列 core::impl&%?::<最后一个标识符>（真 Verus 列成 core::slice::impl&%0::binary_search 这种）。
    fn 里套的 fn 与 const 不列（随外层验）。"""
    listing = []
    for unit in rust_file.units:
        if unit.first_token is None or unit.container is not None and unit.container.kind not in ("impl", "trait"):
            continue
        if unit.kind == "fn" and in_verified_region(rust_file, unit, ranges):
            name = verus_point_name(unit)
            if name is not None:
                listing.append((name, f"fn {name}（{relative}:{unit.first_line}）"))
        elif unit.kind == "const" and in_verified_region(rust_file, unit, ranges):
            container = unit.container
            name = unit.short if container is None else f"{container.short}::{unit.short}" if container.kind == "trait" else f"impl&%?::{unit.short}"
            shown = unit.short if container is None else f"{container.short if container.kind == 'trait' else impl_parts(container.name)[1]}::{unit.short}"
            listing.append((name, f"const {shown}（{relative}:{unit.first_line}）"))
        elif unit.kind in ("struct", "enum", "union") and unit.container is None and any(low < unit.first_token < high for low, high in ranges):
            for trait in DERIVED_VERIFIED_TRAITS:
                if trait in derived_traits(rust_file, unit):
                    name = f"{unit.short}::{trait.lower()}"
                    listing.append((name, f"derive({trait}) {name}（{relative}:{unit.first_line}）"))
            if unit.kind == "enum":
                listing += [(name, f"enum 字段的 spec fn {name}（{relative}:{unit.first_line}）") for name in enum_arrow_names(rust_file, unit)]
    texts = rust_file.texts
    for low, high in region_spans(rust_file):
        for index in range(low, min(high + 1, rust_file.count)):
            if texts[index] == "assume_specification" and rust_file.kinds[index] == "ident":
                point, _closing, _end = assume_specification_parts(rust_file, index, high)
                identifiers = re.findall(r"[A-Za-z_][A-Za-z0-9_]*", point)
                name = point if PLAIN_PATH.fullmatch(point) else f"core::impl&%?::{identifiers[-1] if identifiers else point}"
                listing.append((name, f"assume_specification[{point}]（{relative}:{rust_file.lines[index]}）"))
    return listing


def top_level_segments(rust_file, low, high):
    """词法单元 [low, high) 按同一层的 , 切成几段（配对的括号整段跳过，<…> 里的 , 不算；-> 的 > 不算收尾），交 [(段首, 段尾)]，空段不交。"""
    texts, segments, start, angle, index = rust_file.texts, [], low, 0, low
    while index < high:
        word = texts[index]
        if word in ("(", "[", "{") and index in rust_file.pairs:
            index = rust_file.pairs[index] + 1
            continue
        if word == "<":
            angle += 1
        elif word == ">" and angle and not (index > low and texts[index - 1] == "-"):
            angle -= 1
        elif word == "," and angle == 0:
            if index > start:
                segments.append((start, index))
            start = index + 1
        index += 1
    if high > start:
        segments.append((start, high))
    return segments


def enum_arrow_names(rust_file, unit):
    """verus! 里的 enum，Verus 给变体的每个字段生成一个 spec fn、列进可点名函数表（2026-10-29 在 core 上用 Verus 0.2026.09.20 现量，block_device、logged_operations、
    pointer、records、verified_model 几个模块的表里都有）：<类型>::arrow_<变体>_<字段>（元组变体的字段写下标 0、1…），只在一个变体里出现的字段名另列 <类型>::arrow_<字段>。
    外属性 #[…] 与 pub、pub(…) 跳过；判别值（= <数>）不算字段。"""
    texts, pairs = rust_file.texts, rust_file.pairs
    keyword = next((index for index in range(unit.first_token, unit.last_token) if texts[index] == "enum"), None)
    opening = next((index for index in range(keyword, unit.last_token) if texts[index] == "{" and index in pairs), None) if keyword is not None else None
    if opening is None:
        return []
    variants = []
    for start, end in top_level_segments(rust_file, opening + 1, pairs[opening]):
        index = start
        while index < end and texts[index] == "#" and index + 1 < end and texts[index + 1] == "[" and index + 1 in pairs:
            index = pairs[index + 1] + 1
        if index >= end or rust_file.kinds[index] != "ident":
            continue
        variant, fields, body = texts[index], [], index + 1
        if body < end and texts[body] in ("(", "{") and body in pairs:
            for position, (field_start, field_end) in enumerate(top_level_segments(rust_file, body + 1, pairs[body])):
                if texts[body] == "(":
                    fields.append(str(position))
                    continue
                cursor = field_start
                while cursor < field_end and (texts[cursor] == "#" and cursor + 1 < field_end and texts[cursor + 1] == "[" and cursor + 1 in pairs):
                    cursor = pairs[cursor + 1] + 1
                if cursor < field_end and texts[cursor] == "pub":
                    cursor = pairs[cursor + 1] + 1 if cursor + 1 < field_end and texts[cursor + 1] == "(" and cursor + 1 in pairs else cursor + 1
                if cursor < field_end and rust_file.kinds[cursor] == "ident":
                    fields.append(texts[cursor])
        variants.append((variant, fields))
    seen = collections.Counter(field for _variant, fields in variants for field in set(fields))
    names = [f"{unit.short}::arrow_{variant}_{field}" for variant, fields in variants for field in fields]
    return names + [f"{unit.short}::arrow_{field}" for field in sorted(seen) if seen[field] == 1]


def verus_function_matches(argument, names):
    """照 Verus 0.2026.09.20 的 --verify-function 在可点名函数表 names 里点 argument，交点中的下标（2026-10-29 探针现量，帮助原文
    「matches on unique substring (foo) or wildcards at ends of the argument (*foo, foo*, *foo*)」）：
    *<串>* 是包含它、*<串> 是以它收尾、<串>* 是以它起头，带 * 的点中几个 Verus 就一起验几个；不带 * 的先找恰好同名的，没有再找包含它的
    （包含它的不止一个 Verus 报 more than one match，一个都没有报 could not find）。"""
    if len(argument) > 1 and argument.startswith("*") and argument.endswith("*"):
        return [index for index, name in enumerate(names) if argument[1:-1] in name]
    if argument.startswith("*"):
        return [index for index, name in enumerate(names) if name.endswith(argument[1:])]
    if argument.endswith("*"):
        return [index for index, name in enumerate(names) if name.startswith(argument[:-1])]
    exact = [index for index, name in enumerate(names) if name == argument]
    return exact or [index for index, name in enumerate(names) if argument in name]


def assume_specification_entries(rust_file, relative, crate, module):
    """被验区域里每一处 assume_specification 一项（ASSUME_SPECIFICATION）：项名 proof:<crate>::<模块>::assume_specification[<点名的条目>]，
    交给 --verify-function 的是点名的条目（是一段只有标识符与 :: 的路径时照写，例 singlefs_format::SLOT_BYTES；别的写法取最后一个标识符，例 binary_search）。
    静态估：指向常量（] 后面不是参数表）而写了 returns 的 1 条，别的 0 条；这一类验出 0 条也是绿（指向函数的规格没有义务）。
    算键的入口是 lib_function_units 切出的那个 assume_specification 单元（<核>::assume_specification@<关键字那一行>，单元从属性起、含这个词）；
    找不到的交 None，那一项拒算（check_proof 里的 assume_specification_components）。弄坏开关 assume-specification-not-item 下一项都不成（改前的样子）。"""
    if "assume-specification-not-item" in BREAK:
        return []
    entries = []
    texts = rust_file.texts
    declared = [unit for unit in rust_file.units if unit.kind == "assume_specification" and unit.first_token is not None]
    for low, high in region_spans(rust_file):
        for index in range(low, min(high + 1, rust_file.count)):
            if texts[index] != "assume_specification" or rust_file.kinds[index] != "ident":
                continue
            point, closing, end = assume_specification_parts(rust_file, index, high)
            identifiers = re.findall(r"[A-Za-z_][A-Za-z0-9_]*", point)
            verus_name = point if PLAIN_PATH.fullmatch(point) else (identifiers[-1] if identifiers else point)
            to_constant = closing is not None and closing + 1 < rust_file.count and texts[closing + 1] != "("
            estimate = 1 if to_constant and "returns" in texts[index:end] else 0
            full = "::".join(part for part in (crate, module, f"assume_specification[{point}]") if part)
            owners = [unit for unit in declared if unit.first_token <= index < unit.last_token]
            entries.append((index, ProofItem(proof_item(full), verus_name, relative, DeclarationSite(index, rust_file.lines[index]), ASSUME_SPECIFICATION,
                                             f"{relative}::{owners[0].name}" if len(owners) == 1 else None, estimate)))
    return entries


def point_name_problems(relative, items, listing):
    """一份核（一个模块）里每一项交给 --verify-function 的名字，照 Verus 的点法（verus_function_matches）在这个模块的可点名函数表（verus_listing）里
    要恰好命中一项：命中几项 Verus 就一起验几项（带 *）或报 more than one match（不带 *），逐项点不准，判红并点名命中的那几项；一项都没命中是表与点名对不上
    （这份判法的毛病），也判红。fn 之间的同名在 verus_functions 里先判了。弄坏开关 point-names-by-containment 下照改前的判法（point_names_by_containment）。"""
    if "point-names-by-containment" in BREAK:
        point_names_by_containment(relative, items)
        return
    names = [name for name, _where in listing]
    for item in items:
        hits = verus_function_matches(item.verus_name, names)
        if len(hits) == 1:
            continue
        if not hits:
            raise Red(f"{relative} 的 {item.item} 交给 --verify-function 的 {item.verus_name} 在这个模块的 Verus 可点名函数表里一项都点不中"
                      f"（表里 {len(names)} 项：{listed_names(names)}）",
                      "这是 verus-proof-check.py 的 verus_listing 与点名（verus_point_name、valued_const_point_name、assume_specification_entries）对不上："
                      "用真 Verus 在这个模块上点一个不在的名字，照它列的可点名函数表改这两处")
        raise Red(f"{relative} 的 {item.item} 交给 --verify-function 的 {item.verus_name} 在这个模块的 Verus 可点名函数表里命中 {len(hits)} 项："
                  f"{'、'.join(listing[index][1] for index in hits)}：Verus 会把它们一起验（或报 more than one match），逐项点不准",
                  "给撞上的 const 或函数改名，让这个模块里只有它一个以 ::<名> 收尾（关联 const）或恰好叫这个名字，同步改必红世界表里引它的行")


def point_names_by_containment(relative, items):
    """弄坏开关 point-names-by-containment 下的改前判法：只看关联 const 与 std 方法的 assume_specification 两类，只在成项的几项之间比名字是不是别人的一段，
    不照 Verus 的点法、不看表里不成项的函数与 const。"""
    names = collections.Counter(item.verus_name for item in items)
    for item in items:
        if item.kind not in (VALUED_CONST, ASSUME_SPECIFICATION):
            continue
        if names[item.verus_name] > 1:
            raise Red(f"{relative} 的被验区域里有 {names[item.verus_name]} 项交给 --verify-function 的名字都是 {item.verus_name}（{item.item} 是其中一项）",
                      "给其中一个改名，同步改必红世界表里引它的行")
        exact = item.kind == ASSUME_SPECIFICATION and PLAIN_PATH.fullmatch(item.verus_name) and "::" in item.verus_name \
            or item.kind == VALUED_CONST and item.unit.container is None
        clashing = [other.verus_name for other in items if other is not item and item.verus_name in other.verus_name]
        if not exact and clashing:
            raise Red(f"{relative} 的 {item.item} 交给 --verify-function 的 {item.verus_name} 是 {listed_names(clashing)} 的一段",
                      "给 const 或那几个函数改名，让它的名字不是别的函数名的一段")


def owner_function(functions, relative, line):
    """核 relative 第 line 行落在哪个被验区域里的 fn 单元里，交那个 VerusFunction；不在任何一个里交 None。"""
    for function in functions:
        if function.relative == relative and function.unit.first_line <= line <= function.unit.last_line:
            return function
    return None


def crate_build_text(root, registry):
    """这个 crate 怎么交给 Verus 编（键里「crate 起法」那一样）：起法（crate 行第四段 cargo 的经 cargo verus focus，不写的直接起 verus）、
    Cargo.toml 的 edition 与包名（直接起的那一种照它们拼 --edition、--crate-name；cargo 那一种由 cargo 照它们编）。edition 写 workspace = true 的取仓根 [workspace.package]。"""
    package = crate_manifest(root, registry).get("package", {})
    edition = package.get("edition", "未写")
    if isinstance(edition, dict) and edition.get("workspace"):
        try:
            with open(os.path.join(root, "Cargo.toml"), "rb") as handle:
                edition = ((tomllib.load(handle).get("workspace") or {}).get("package") or {}).get("edition", "未写")
        except (OSError, tomllib.TOMLDecodeError):
            edition = "仓根 Cargo.toml 读不出"
    return f"mode={'cargo' if registry.cargo_mode else 'verus'}\tedition={edition}\tname={package.get('name', '未写')}"


_MUTATION_UNITS = []


def mutation_units_module():
    """scripts/mutation/lib_mutation_units.py（闭包真点名的外部 crate 落到 Cargo.lock 的哪几个包，与 crates 变异的行键同一份 dependency_lines），第一次用到时才读进来。"""
    if not _MUTATION_UNITS:
        import importlib.util
        specification = importlib.util.spec_from_file_location("lib_mutation_units_for_verus_proof_check", os.path.join(SCRIPT_DIRECTORY, "..", "mutation", "lib_mutation_units.py"))
        module = importlib.util.module_from_spec(specification)
        sys.modules[specification.name] = module
        specification.loader.exec_module(module)
        _MUTATION_UNITS.append(module)
    return _MUTATION_UNITS[0]


def closure_dependency_lines(root, registry, options, unit_key):
    """键里「依赖版本」那一样：单元闭包（unit_key 的组成清单）里各单元真点名的外部 crate 在 Cargo.lock 里落到的包与传递依赖，一包一行
    （lib_mutation_units.py 的 dependency_lines，口径与 crates 变异的行键相同）。点名了而 Cargo.lock 里落不到的（拒算），退回这个 crate 在 Cargo.lock 里
    传递引到的全部外部包（lib_verdict_library.py 的 locked_dependency_lines，只多作废、不少）。每个单元的那几行按单元存进 options，几项共用。"""
    mutation_units = mutation_units_module()
    workspace = unit_index(root, options).workspace()
    cache = options.setdefault("unit_dependency_lines", {})
    try:
        if options.get("lock_packages") is None:
            options["lock_packages"] = mutation_units.read_lock_packages(root)
    except mutation_units.MutationUnitsError:
        return ["按整个 crate 收（Cargo.lock 读不懂）"] + lib_verdict_library.locked_dependency_lines(root, crate_package_name(registry))
    lines, refused = set(), False
    for text in unit_key.units:
        kind, name, location = text.split("\t")[:3]
        path, _colon, span = location.rpartition(":")
        if not path.endswith(".rs"):
            continue
        if (path, kind, name, span) not in cache:
            try:
                found = [unit for unit in workspace.file(path).units if unit.kind == kind and unit.name == name and f"{unit.first_line}-{unit.last_line}" == span]
            except (OSError, ValueError):
                found = []
            cache[(path, kind, name, span)] = mutation_units.dependency_lines(workspace, found, options["lock_packages"])
        unit_lines, unit_refusals = cache[(path, kind, name, span)]
        lines |= set(unit_lines)
        refused = refused or bool(unit_refusals)
    if refused:
        return ["按整个 crate 收（闭包里有点名的外部 crate 在 Cargo.lock 里落不到）"] + lib_verdict_library.locked_dependency_lines(root, crate_package_name(registry))
    return sorted(lines)


def dependency_component(root, registry, options, unit_key):
    """「依赖版本」那一样（closure_dependency_lines）；单元闭包拒算的不算（那一项本来就每趟现判）。弄坏开关 reuse-key-without-closure-dependencies：不进（改前的样子）。"""
    if "reuse-key-without-closure-dependencies" in BREAK or not isinstance(unit_key, UnitKey) or unit_key.key is None:
        return []
    return [("依赖版本", digest_text("\n".join(closure_dependency_lines(root, registry, options, unit_key))))]


def shared_components(root, registry, options):
    """Verus 每一项都带的几样：工具链（tool-versions 那一行去掉路径的那一份，tool_versions_text 的 for_key：Verus、它自带的 vstd、z3、rustup 工具链的版本与内容摘要）、
    rlimit（登记的数，改变结论）、crate 起法（crate_build_text：cargo 还是直接起、edition、包名）。
    verus-pins.tsv 整份不进（登记表的注释、verified 的登记数与别的行不改任何一项的结论）。
    弄坏开关 toolchain-key-with-paths：工具链那一样照 tool-versions 打的整行（带 RUSTUP_HOME、rustup、VERUS_ROOT、VERUS_Z3_PATH 的路径，改前的样子）；
    reuse-key-without-crate-build：crate 起法不进（改前的样子）。算一次存进 options；没给判定库时不算（交空表，lookup 也不看它）。"""
    if not options.get("verdict_library"):
        return []
    if options.get("shared_components") is None:
        components = []
        if "reuse-key-without-toolchain" not in BREAK:
            components.append(("工具链", digest_text(tool_versions_text(root, options.get("verus_root"), for_key="toolchain-key-with-paths" not in BREAK))))
        components.append(("rlimit", digest_text(str(registry.rlimit))))
        if "reuse-key-without-crate-build" not in BREAK:
            components.append(("crate 起法", digest_text(crate_build_text(root, registry))))
        # 信任清单（vstd 行与这个 crate 的条目行）：登记的规格就是证明的前提，改一行这个 crate 的每一项都现判（第二轮正推 Y4 第 9 条）
        if "reuse-key-without-trust-list" not in FORMAL_BREAK:
            components.append(("信任清单", digest_text(trust_list_text(root, registry))))
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
    return [(label, key)] + dependency_component(root, registry, options, key)


def entry_unit_component(rust_unit, label):
    """这一项自己是闭包里的哪一个单元（键里「被验入口单元」「所属函数单元」那一样）：入口单元连同包它的 impl / trait 头一层层往外，各取（种类，去注释指纹），
    不带名字、路径与行号。闭包是单元的多重集，同一个 impl 里几个方法经 R-结构、R-隐式互相收进对方的闭包，闭包一模一样，没有这一样就几项共用一把键、
    一份判定（core 的 impl BlockDevice for FileBackedBlockDevice 八个方法共用一把，复用时 write_zeroes_at 拿到 size_in_bytes 那一份的 1 条）。
    弄坏开关 reuse-key-without-entry-unit：不进（改前的样子）。"""
    if "reuse-key-without-entry-unit" in BREAK or rust_unit is None:
        return []
    lines, owner = [], rust_unit
    while owner is not None:
        lines.append(f"{owner.kind}\t{owner.fingerprint}")
        owner = owner.container
    return [(label, digest_text("\n".join(lines)))]


def proof_entry_unit(root, options, entry):
    """证明一项算键的入口单元：fn、clone 的类型与带值的 const 是 ProofItem 的 unit 本身；assume_specification 那一项的 unit 是 DeclarationSite，
    按 entry（<核>::<单元名>）到核里找那个单元，找不到交 None（那一项本来就拒算）。"""
    if entry.kind != ASSUME_SPECIFICATION:
        return entry.unit
    if entry.entry is None:
        return None
    name = entry.entry[len(entry.relative) + 2:]
    found = [unit for unit in unit_index(root, options).workspace().file(entry.relative).units if unit.name == name]
    return found[0] if len(found) == 1 else None


ASSUME_TARGET_KINDS = ("const", "static", "fn")


def assume_specification_components(root, registry, options, entry):
    """assume_specification 那一项的被验函数那一样：入口是这一条 assume_specification 单元（lib_function_units 切出的，assume_specification_entries 交的 entry），
    闭包经 R-路径跨包收到它点名的那个条目（工作区包 singlefs_format 里的常量连同它引到的单元），与 fn 同一种算法、同样带依赖版本；它验出几条取决于那个条目。
    找不到这一条的单元、或闭包里没有点名的条目（种类是 const、static、fn，限定名最后一段是点名的最后一个标识符；std 的方法这类不在工作区里的）拒算：
    这一项照判、不复用，拒算行点名是哪一处。弄坏开关：assume-specification-refused 下一律拒算（改前的样子）；assume-specification-target-unkeyed 下
    点名的条目不进键（改它的值照旧复用）。没给判定库交空表。"""
    place = f"{entry.relative}:{entry.unit.first_line}\t{entry.verus_name}"
    if "assume-specification-refused" in BREAK or entry.entry is None:
        return [("被验函数单元闭包", UnitKey(None, (), (f"refused\tassume-specification\t{place}",)))]
    components = function_components(root, registry, options, [entry.entry], "被验函数单元闭包")
    keyed = dict(components).get("被验函数单元闭包")
    if not isinstance(keyed, UnitKey) or keyed.key is None:
        return components
    target = (RUST_IDENTIFIER.findall(entry.verus_name) or [entry.verus_name])[-1]
    pointed = [line.split("\t") for line in keyed.units]
    pointed = [fields for fields in pointed if fields[0] in ASSUME_TARGET_KINDS and fields[1].rsplit("::", 1)[-1] == target]
    if not pointed:
        return [("被验函数单元闭包", UnitKey(None, keyed.units, (f"refused\tassume-specification-target\t{place}",)))]
    if "assume-specification-target-unkeyed" in BREAK:
        excluded = [f"{fields[2].rsplit(':', 1)[0]}::{fields[1]}" for fields in pointed]
        reduced = unit_index(root, options).key([entry.entry], excluded_units=excluded)
        return [("被验函数单元闭包", reduced)] + dependency_component(root, registry, options, reduced)
    return components


FORMAL_LIBRARY_DIRECTORY_NAME = "singlefs-formal-unit-verdicts"


def formal_library_directory(root):
    """门禁 formal-verification 的判定库目录（文件头「逐项点名与按条复用」的库位置），只由被判那棵仓定：被判那棵树的 git common-dir 底下的
    singlefs-formal-unit-verdicts/（共用库 lib_verdict_library.py 的 library_directory）；不是 git 工作树退 2（共用库的 LibraryUnavailable）。"""
    return lib_verdict_library.library_directory(root, FORMAL_LIBRARY_DIRECTORY_NAME)


FAKE_TOOLCHAIN = "1.98.1-x86_64-unknown-linux-gnu"   # 假安装 version.json 里 toolchain 那一段（门禁样本里没有 rustc 登记行时不比它）


def write_fake_tools(directory, root):
    """write-fake-tools <目录> <根>：往 <目录> 写一份假的 Verus 安装（verus、cargo、cargo-verus 三个可执行文件，加 version.json）；
    version.json 的 version 与 commit 取 <根> 登记表 release 行的第 2、3 段（样本照真仓登记，装的 Verus 与登记表同版本，工具链那一格才对得上）。
    三个假工具就是自证（--selftest）用的 FAKE_VERUS、FAKE_CARGO、FAKE_CARGO_VERUS，按源码里的标记出判定、不起真 Verus 与真 cargo；
    门禁 formal-verification 的样本档在样本根里写它、用 --verus-root 指到它、PATH 前面放它（文件头「门禁样本」）。
    弄坏开关 fake-tools-commit-unpinned 下 version.json 的 commit 写成全 0（工具链那一格与 Verus 报的 commit 对不上）。"""
    try:
        with open(os.path.join(root, REGISTRY), encoding="utf-8") as handle:
            rows = [line.split("\t") for line in handle.read().split("\n") if line.startswith("release\t")]
    except OSError as error:
        report_red(f"读不了登记表 {REGISTRY}：{error}", "第二个参数给带着登记表的仓根（样本根）")
        return EXIT_USAGE
    if not rows or len(rows[0]) < 3 or not rows[0][2]:
        report_red(f"{REGISTRY} 里没有 release 行（或它不到三段）：假安装的 version.json 没有版本与 commit 可抄", "登记表照真仓写 release 行")
        return EXIT_USAGE
    os.makedirs(directory, exist_ok=True)
    for tool, text in (("verus", FAKE_VERUS), ("cargo", FAKE_CARGO), ("cargo-verus", FAKE_CARGO_VERUS)):
        path = os.path.join(directory, tool)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
        os.chmod(path, 0o755)
    commit = "0" * 40 if "fake-tools-commit-unpinned" in BREAK else rows[0][2]
    with open(os.path.join(directory, "version.json"), "w", encoding="utf-8") as handle:
        json.dump({"verus": {"version": rows[0][1], "commit": commit, "toolchain": FAKE_TOOLCHAIN}}, handle)
    print(f"  · 假 Verus 安装写在 {directory}（verus、cargo、cargo-verus、version.json；版本 {rows[0][1]}，commit {commit}）")  # gate-lint:detail
    return EXIT_GREEN


def library_directory_command(root):
    """library-directory <根>：stdout 打判定库目录（门禁定库位置读它）；不是 git 工作树退 2（stderr 说为什么）。"""
    print(formal_library_directory(os.path.abspath(root)))
    return EXIT_GREEN


def verdict_library(options, kind, cell):
    version = judging_units(cell) if "reuse-key-without-judging-units" in BREAK else (options.get("judging_units") or judging_units(cell))
    store = VerdictLibrary(options.get("verdict_library"), kind, version)
    store.plan = collections.Counter() if options.get("plan") is not None else None
    store.plan_items = [] if options.get("plan") is not None else None   # [(项, 状态)]：只数键那一趟逐项的键状态（ITEM_KEY 行的原料）
    return store


PLAN_STATES = ("键对不上", "无记录", "键相同", "拒算")


def peek(store, item, components):
    """只数键（list-items 给了判定库）：这一项照现在的键在库里是哪一种，不写库：键相同（会复用）、键对不上（这一项判绿过，键变了）、无记录、拒算。"""
    if refused_component(components, store.judging_units) is not None:
        return "拒算"
    # 键文件与 items/ 那一份经判定库读（prune-server 里那一份：scripts/crash/lib_verdict_library.py 文件头「存放」；服务不可用客户端退 75），不写库
    if "key-mismatch-by-item-only" in BREAK:   # 弄坏开关：这一项判绿过就算键相同，不比键
        return "键相同" if store.item_record(item) is not None else "无记录"
    if store.key_exists(key_of(components, store.judging_units)):
        return "键相同"
    return "键对不上" if store.item_record(item) is not None else "无记录"


def plan_done(options, cell, store):
    """只数键的那一趟：这一格各项的键都比过了，记下各几条，不起 Verus。"""
    options["plan"].append((cell, store.plan))
    if options.get("plan_items") is not None:
        options["plan_items"].extend(store.plan_items)
    return EXIT_GREEN


def lookup(store, item, components):
    if getattr(store, "plan", None) is not None:
        state = peek(store, item, components)
        store.plan[state] += 1
        store.plan_items.append((item, state))
        return False, "只数键"
    if "reuse-ignores-key" in BREAK:
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
    candidates = {item: "绿" for item, components in entries
                  if components and reuse_checks.verdict_library_candidate(options.get("verdict_library"), kind, components, store.judging_units)}
    sentinel.plan(candidates)
    return sentinel


def sentinel_lookup(store, sentinel, item, components):
    """抽中强制现判的那一项不读库；别的照 lookup。弄坏开关 reuse-lookup-skipped 下一律不读库（复用哨兵该判红「复用失灵」）。"""
    if sentinel.forced(item):
        return False, "复用哨兵抽中强制现判"
    if "reuse-lookup-skipped" in BREAK:
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


def record_item_done(item, seconds, judged, reused, exit_code):
    """环境里有 FORMAL_ITEM_DONE_FILE 的（门禁给的）往那份文件追加一行「项<TAB>秒<TAB>现判条数<TAB>复用条数<TAB>退出码」，一次 write 写完一整行（几处同时追加不拧）。
    弄坏开关 item-done-unrecorded 下不写。"""
    path = os.environ.get(ITEM_DONE_FILE_VARIABLE)
    if not path or "item-done-unrecorded" in BREAK:
        return
    with open(path, "a", encoding="utf-8") as handle:
        handle.write(f"{item}\t{seconds:.1f}\t{judged}\t{reused}\t{exit_code}\n")


def report_item(item, verdict, how, seconds=None):
    """每项一行：项名、判定（绿 / 红）、现判还是复用、现判的用了几秒。同时记一行 ITEM_DONE 的原料（record_item_done）：复用的 0 秒、复用 1 条；
    现判的取这一项自己的秒数（verify_in_parallel 里这一处从起到出结局的挂钟，不是整格的）、现判 1 条；退出码绿 0、红 1。"""
    print(f"  · {item}：{verdict}，{how}" + (f"，{seconds:.1f} 秒" if seconds is not None else ""))  # gate-lint:detail
    reused = how.startswith("复用")
    record_item_done(item, 0.0 if reused else (seconds or 0.0), 0 if reused else 1, 1 if reused else 0, 0 if verdict == "绿" else 1)


def cell_tally(options):
    """这一格（登记了几个 crate 时是这个 crate 那一趟）的现判、复用数；几个 crate 一格时记进 options["crate_tallies"]，由 run_per_crate 合起来报一次。"""
    tally = Tally()
    if options.get("crate_tallies") is not None:
        options["crate_tallies"].append(tally)
    return tally


def finish_tally(tally, cell, options=None):
    """一格判完报「按条：现判 N、复用 M」，并往门禁给的 FORMAL_ITEM_TALLY_FILE 追加一行；几个 crate 一格时只报这个 crate 的、不追加（run_per_crate 合起来追加一行）。"""
    if options is not None and options.get("crate_tallies") is not None:
        print(f"  · 按条（{options['crate_view'].crate_directory}）：{tally.text()}")  # gate-lint:detail
        return
    print(f"  · 按条：{tally.text()}")  # gate-lint:detail
    tally.append_to(os.environ.get(ITEM_TALLY_FILE_VARIABLE), cell)


def merged_tally(tallies):
    total = Tally()
    for tally in tallies:
        total.reused += tally.reused
        for reason, count in tally.reasons.items():
            total.reasons[reason] = total.reasons.get(reason, 0) + count
    return total


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
    """成项的几类各几个，次序固定：proof fn、exec fn、带 decreases 的 spec fn、derive(Clone) 生成的 clone、带值的 const、assume_specification
    （弄坏开关 spec-fn-as-item 下另有 spec fn）。"""
    kinds = collections.Counter(entry.kind for entry in items)
    return "、".join(f"{kind} {kinds[kind]}" for kind in (PROOF_FN, EXEC_FN, SPEC_FN_WITH_DECREASES, DERIVED_CLONE, VALUED_CONST, ASSUME_SPECIFICATION, "spec fn")
                    if kinds[kind])


def listed_names(names, limit=20):
    """点名用：至多 limit 个名字，多的报个数。"""
    names = list(names)
    return "、".join(names[:limit]) + (f" 等 {len(names)} 个" if len(names) > limit else "") if names else "没有"


def check_proof(root, options):
    """证明逐项：有证明义务的单元各一项（proof_items：proof fn、exec fn、带 decreases 的 spec fn、derive(Clone) 的 clone；spec fn 不带 decreases 的不成项），
    各验一次（--verify-root 或 --verify-only-module <核所在的模块>，再 --verify-function <名>；kernel_module）。每项判据：出结局、没有 vir 错、errors 0（verification_clean：--verify-function 下不看 success 与
    is-verifying-entire-crate）、verified 至少 1（0 是 Verus 在点的函数上一条义务都没验：点错了函数，或它不该成项）、跑的 Verus 的 commit 等于登记。
    整格跑时各项 verified 之和（复用的取记下的数；按 --verify-function 点的名字计，同名的只算一次）要等于登记的数（verus-pins.tsv 的 crate 行，整 crate 验的数），
    不等判红并点名：verified 与静态估的义务条数不同的项、没成项的 verus! fn。--item proof 或不给 --item 是整格；--item proof:<名> 只验点名的，不比之和。"""
    registry, kernels, manifest, executable = prepared(root, options)
    modules = kernel_modules(root, registry, manifest, kernels)
    # 被验区域的另几样（出口 5 第三轮判决第三节「门禁」）：信任清单逐条登记、axiom、cfg 原子、被验类型的 trait impl 在 verus! 外面。
    # 先于切函数判：按 cfg 分叉的两份同名 fn 在切函数那一步就红（逐项点不准），红句要先说 cfg 原子
    region = region_problems(root, registry, kernels, options)
    try:
        functions = verus_functions(root, registry, kernels, options)
        items, left_out = proof_items(root, registry, kernels, functions, options)
    except Red as red:
        if not region:
            raise
        report_cell(region + [(red.message, red.howto)])
        return EXIT_RED
    if not items:
        raise Red(f"{len(kernels)} 份核的 verus! 里一个有证明义务的单元都没有（verus! 里 {len(functions)} 个 fn，{len(left_out)} 个是不带 decreases 的 spec fn）：证明这一格没有项",
                  "核里的证明写在 verus! { … } 里的 proof fn 与 exec fn 上；核挪了地方就改 .claude/gate.d/verus-pins.tsv 的 kernel 行")
    available = [entry.item for entry in items]
    named = options.get("items") or []
    wanted = [item for item in named if item != "proof"]
    # 点名里有整格 proof：点名的 proof:<名> 加在整格之上，整格照跑（各项之和、完整 crate 证明、MIR 比对），点名的只核项名在不在。
    # 弄坏开关 proof-item-narrows：proof 与 proof:<名> 一起点名时只验点名的那几个（改前的样子：--item proof --item proof:X 丢了整格）
    whole_cell_named = "proof" in named and "proof-item-narrows" not in BREAK
    whole = not wanted or whole_cell_named or "item-filter-ignored" in BREAK
    chosen = selected_items(dict(options, items=wanted), ["proof", *available], "verus-proof") if wanted else available
    if whole_cell_named:
        chosen = available
    selected = [entry for entry in items if entry.item in chosen]
    store, tally = verdict_library(options, "proof-function", "proof"), cell_tally(options)
    shared = shared_components(root, registry, options)
    problems = []
    if "cheating-unchecked" not in BREAK:
        for marker in cheating_markers(root, kernels):
            problems.append((f"核里有不许的标记 {marker}", "被验区域里不许 admit、assume(…)、axiom、verifier::external：把它换成真证明，证不出就照实报，不绕过；"
                             f"std 规格与设备、校验和这类可信声明写成 assume_specification / external_body / uninterp spec fn，逐条登记进信任清单 {TRUST_LIST}"))
        if "rlimit-attribute-unchecked" not in BREAK:
            for marker in rlimit_attributes(root, kernels):
                problems.append((f"核里有不许的标记 {marker}（函数上的 rlimit 属性盖过命令行的 --rlimit {registry.rlimit}，这个函数不在登记的额度上验）",
                                 f"删掉这个属性；证明确要更多额度，照 {REGISTRY} rlimit 行上面的注释在新数上把核与必红世界表整张重验一遍，再调高登记的数"))
    problems += region
    counts, pending = {}, []
    keyed = [(entry, shared + (assume_specification_components(root, registry, options, entry) if entry.kind == ASSUME_SPECIFICATION
                               else function_components(root, registry, options, [entry.entry], "被验函数单元闭包"))
              + entry_unit_component(proof_entry_unit(root, options, entry), "被验入口单元") if shared else []) for entry in selected]
    sentinel = plan_sentinel(options, store, "proof-function", "verus-proof", [(entry.item, components) for entry, components in keyed])
    for entry, components in keyed:
        reused, reason = sentinel_lookup(store, sentinel, entry.item, components)
        verified = ((store.record_of(entry.item) or {}).get("details") or {}).get("verified") if reused else None
        if reused and not str(verified).isdigit():
            reused, reason = False, "记录里没有 verified 数"
        tally.count(reused, reason)
        if reused:
            counts[entry.item] = int(verified)
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
            return [(f"{item}：{problem}", problem_howto(outcome, "看上面 Verus 的报错"))], "", None
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
        if results["verified"] < 1 and "zero-verified-accepted" not in BREAK and entry.kind != ASSUME_SPECIFICATION:   # 指向函数的 assume_specification 验出 0 条
            return [(f"{item}：Verus 在 --verify-function {entry.verus_name} 上一条义务都没验（{results_text(results)}；它是 {entry.kind}，静态估 {entry.estimate} 条）",
                     "点到的不是要验的那个函数，或它本没有证明义务：看 scripts/gate/verus-proof-check.py 的 proof_items 怎么认它这一类"
                     "（不带 decreases 的 spec fn 不成项），名字与别的函数撞了就改名")], "", None
        counts[entry.item] = results["verified"]
        return [], f"{results['verified']} verified、0 errors", {"verified": str(results["verified"])}

    judging = ItemJudging(store, judge_one, "verus-proof",
                          f"{'cargo verus focus' if registry.cargo_mode else 'verus'} --verify-root|--verify-only-module <核所在的模块> --verify-function <名> "
                          f"--rlimit {registry.rlimit}（{registry.tag}）",
                          sentinel=sentinel)
    started = time.monotonic()
    verify_in_parallel(root, registry, manifest, executable, options,
                       [(entry.item, {}, entry.verus_name, modules[entry.relative]) for entry, _components, _reason in pending],
                       judged=lambda index, outcome, seconds: judging(index, outcome, seconds, pending[index][0].item, pending[index][1],
                                                                      judged_how(store, pending[index][0].item, pending[index][2], sentinel)))
    problems += judging.finish(len(pending))
    problems += sentinel.finish()
    total = sum(counts.values())
    breakdown = item_kinds_text(selected)
    if whole and not problems and total != registry.verified_count and "count-unchecked" not in BREAK:
        off = [f"{entry.item} 验出 {counts[entry.item]}、静态估 {entry.estimate}" for entry in selected
               if entry.item in counts and counts[entry.item] != entry.estimate]
        problems.append((f"各项 verified 之和 {total} 与登记的 {registry.verified_count} 不同（{'少' if total < registry.verified_count else '多'} "
                         f"{abs(registry.verified_count - total)} 条；成项 {len(selected)} 个：{breakdown}，静态估之和 {sum(entry.estimate for entry in selected)}）："
                         f"verified 与静态估不同的项：{listed_names(off)}；没成项的 verus! fn（不带 decreases 的 spec fn，静态估 0 条）{len(left_out)} 个："
                         f"{listed_names(name for name, _relative in left_out)}",
                         "少了是整 crate 验出的义务有一部分不落在成项的单元上：在副本上整 crate 验一次（不带 --verify-function），再逐个 --verify-function 没成项的那几个，"
                         "验出非 0 的那一类补进 scripts/gate/verus-proof-check.py 的 proof_items；多了先看 verified 与静态估不同的那几项；"
                         f"确是加减了证明就照整 crate 验出的数改 {REGISTRY} 的 crate 行，并在改动说明里写加减了哪几个"))
    # 逐函数的计数相同并不表示义务集合完整；完整验证也检查未选中的引理。
    whole_started = time.monotonic()   # 整格这一项（proof）自己的活：各项之和、完整 crate 验证、MIR 比对，都不按条入库，每趟现判
    if whole and not problems:
        outcome = verify_in_parallel(root, registry, manifest, executable, options,
                                     [("完整 crate 证明", {})])[0]
        problem = outcome_problem(outcome, options)
        if problem or not verification_clean((outcome.get("json") or {}).get("verification-results", {}), False) or (outcome.get("json") or {}).get("verification-results", {}).get("is-verifying-entire-crate") is not True:
            problems.append((f"完整 crate 验证失败：{problem or outcome['diagnostics'][-3000:]}",
                             "逐项通过不能替代完整验证：检查漏登记、宏生成或未验证的引理"))
    # 产品构建与 verus --compile 构建的 MIR 逐函数比（出口 5 第三轮判决 Z2）：整格跑、前面都没红时比一次，不按条入库（同完整 crate 证明）
    mir_note = "MIR 比对没跑（--item 点名、或前面已经红了）"
    if whole and not problems:
        mir_problems, mir_note = compare_mir(root, registry, manifest, executable, options)
        problems += mir_problems
    if whole:
        record_item_done("proof", time.monotonic() - whole_started, 1, 0, 1 if problems else 0)
    finish_tally(tally, "verus-proof", options)
    not_run_note = report_passed_not_run(options, registry)
    if problems:
        report_cell(problems)
        return EXIT_RED
    scope = (f"--item 点名的 {len(selected)} 项（不比之和）" if not whole
             else f"成项 {len(selected)} 个（{breakdown}），各项 verified 之和 {total}、0 errors（登记 {registry.verified_count}）；"
                  f"不成项的 spec fn {len(left_out)} 个（没有证明义务，随用到它们的项进键）")
    print(f"  ✓ Verus 逐项验 {registry.crate_directory}{'（cargo verus focus）' if registry.cargo_mode else ''}：{scope}，--rlimit {registry.rlimit}，"
          f"{len(kernels)} 份核里 0 处不许的标记，{options.get('region_note', '被验区域没扫')}{not_run_note}；{mir_note}；"
          f"空真那一格{assert_by_skipped_text(assert_by_requires_sites(root, kernels))}；"
          f"按条{tally.text()}（{time.monotonic() - started:.1f} 秒）")
    return EXIT_GREEN


def assert_by_requires_lines(text):
    """属于 assert(…) by (…) 的 requires 所在的行（从 1 数）：这种 requires 是那一处要先证出来的前提，不是外层函数的前提，不是空真插入点
    （外层函数入口那一处照旧插 assert(false)）。按词法单元认：requires 前面紧接 assert(…) by (…)（两组配对的圆括号，中间是 by，头一组前面是 assert），
    往后在同一层先碰到 ; 收尾（配对的圆括号、方括号整段跳过；先碰到 { 或 } 的是函数体或别的块，不算）。没有函数体的 fn 声明、trait 里无体的方法、
    挂在普通块或不带 by 的 assert 上的 requires 都不是这个样子，照旧走 requires_insertion_points 的找函数体那一支。
    弄坏开关 assert-by-requires-as-insertion-point 下一处都不认（改前的样子：当插入点、找不到函数体判红）；
    assert-by-requires-by-semicolon-line 下照「往下先碰到一行只有 ; 」认（不看前面接的是不是 assert(…) by (…)：普通块与不带 by 的 assert 上的 requires 也放过）。"""
    if "assert-by-requires-as-insertion-point" in BREAK:
        return []
    if "assert-by-requires-by-semicolon-line" in BREAK:
        lines, found = text.split("\n"), []
        for index, line in enumerate(lines):
            stripped = strip_line_comment(line).strip()
            if stripped != "requires" and not stripped.startswith("requires "):
                continue
            indent = len(line) - len(line.lstrip())
            for later in range(index + 1, len(lines)):
                code = strip_line_comment(lines[later])
                if FUNCTION_LINE.match(code) or (code.strip().startswith("{") and len(code) - len(code.lstrip()) <= indent):
                    break
                if code.strip() == ";":
                    found.append(index + 1)
                    break
        return found
    _kinds, words, token_lines, _docs = rust_lex(text)
    opening_of, closing_of, stack = {}, {}, []
    for position, word in enumerate(words):
        if word in ("(", "[", "{"):
            stack.append(position)
        elif word in (")", "]", "}") and stack and words[stack[-1]] == {")": "(", "]": "[", "}": "{"}[word]:
            opening = stack.pop()
            opening_of[position], closing_of[opening] = opening, position
    found = []
    for position, word in enumerate(words):
        if word != "requires" or position < 1 or words[position - 1] != ")":
            continue
        by_opening = opening_of.get(position - 1)
        if by_opening is None or by_opening < 2 or words[by_opening - 1] != "by" or words[by_opening - 2] != ")":
            continue
        assert_opening = opening_of.get(by_opening - 2)
        if assert_opening is None or assert_opening < 1 or words[assert_opening - 1] != "assert":
            continue
        index = position + 1
        while index < len(words) and words[index] not in (";", "{", "}"):
            index = closing_of[index] + 1 if words[index] in ("(", "[") and index in closing_of else index + 1
        if index < len(words) and words[index] == ";":
            found.append(token_lines[position])
    return found


def assert_by_requires_sites(root, kernels):
    """各份核里属于 assert(…) by (…) 的 requires（assert_by_requires_lines），交 [<相对仓根的核>:<行>]：列项与证明、空真两格的成功句报跳过了哪几处。"""
    sites = []
    for kernel in kernels:
        relative = os.path.relpath(kernel, root).replace(os.sep, "/")
        sites += [f"{relative}:{line}" for line in assert_by_requires_lines(open(kernel, encoding="utf-8").read())]
    return sites


def assert_by_skipped_text(sites):
    """「跳过 assert-by requires N 处：<文件>:<行>…」（逐处列，0 处写「没有」）。"""
    return f"跳过 assert-by requires {len(sites)} 处（属于 assert(…) by (…)，在那一处先证出来，不是外层函数的前提）：{listed_names(sites, limit=max(1, len(sites)))}"


def requires_insertion_points(text, relative=None):
    """每一处 requires 的函数体头：[(requires 行号, { 所在行的下标, 说明)]；认不出函数体的抛 Red（relative 给了就写进红句）。
    属于 assert(…) by (…) 的 requires（assert_by_requires_lines）不是插入点，跳过、不判独立契约行。"""
    lines = text.split("\n")
    where = f"{relative} " if relative else ""
    skipped = set(assert_by_requires_lines(text))
    kinds, words, token_lines, _docs = rust_lex(text)
    for position, word in enumerate(words):
        if word != "requires" or (position > 0 and words[position - 1] in (".", "::")):
            continue
        line = token_lines[position]
        if line in skipped:
            continue
        prefix = strip_line_comment(lines[line - 1]).strip()
        if not (prefix == "requires" or prefix.startswith("requires ")):
            raise Red(f"{where}第 {line} 行的 requires 不是独立契约行：拒绝漏检",
                      "将 requires 单独起行，函数体的 { 单独起行后再做空真筛查")
    points = []
    for index, line in enumerate(lines):
        stripped = strip_line_comment(line).strip()
        if stripped != "requires" and not stripped.startswith("requires "):
            continue
        if index + 1 in skipped:
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
            raise Red(f"{where}第 {index + 1} 行的 requires 找不到函数体的 {{（没有函数体，或 {{ 没单独起行；也不是 assert(…) by (…) 接 requires … ; 的样子）",
                      "把函数体的 { 单独起一行（verusfmt 的写法）；是 assert 的前提就写成 assert(…) by (…) 接 requires … ;（判法 assert_by_requires_lines）；"
                      "确是没有函数体的声明，把判法补进 scripts/gate/verus-proof-check.py 的 requires_insertion_points")
        owner = next((FUNCTION_LINE.match(strip_line_comment(lines[back])).group(5) for back in range(index, -1, -1)
                      if FUNCTION_LINE.match(strip_line_comment(lines[back]))), "?")
        inline = lines[brace].split("{", 1)[1].strip()
        label = f"{owner} 里第 {index + 1} 行起的闭包" if inline else f"fn {owner}"
        points.append((index + 1, brace, label))
    return points


def parallel_locations(location_count, options):
    """几处同时验、每处几条线程：交 (处数, 每处的线程数)。处数是 --jobs，0 取 min(处数, max(1, N // 4))；每处 N // 处数条、至少 1；N 是 --threads。
    不按内存收：外层内存上限（OUTER_MEMORY_CAP_VARIABLE）不看，整条命令的内存额度归 task-server（文件头「通用选项」--jobs 一段）。
    弄坏开关 outer-memory-narrows-jobs 下照改前的样子：设了外层上限就收到 1 处。"""
    jobs = options["jobs"] or min(location_count, max(1, options["threads"] // 4))
    if "outer-memory-narrows-jobs" in BREAK and os.environ.get(OUTER_MEMORY_CAP_VARIABLE):
        jobs = 1
    return jobs, max(1, options["threads"] // jobs)


def verify_in_parallel(root, registry, manifest, executable, options, edits, judged=None):
    """edits：[(说明, {相对仓根的文件: 新内容}[, 只验的函数名, 它所在的模块])]；每一处一份副本、一次 Verus（给了函数名就只验它：模块是 None 点 --verify-root，
    不是 None 点 --verify-only-module <模块>，再 --verify-function，verus_arguments）。交回与 edits 同序的 outcome。
    judged(下标, outcome, 秒数) 给了就在那一处出结局时当场调（按条判定在这里判完就落盘，打断之后判过的不丢）。
    几处同时、每处几条线程照 parallel_locations。"""
    if not edits:
        return []
    jobs, threads = parallel_locations(len(edits), options)
    # cargo 那一种：几处同时验各占一个编译目录（同一个编译目录 cargo 一次只让一处用），vstd 与依赖在一个编译目录里只编一次；
    # 副本各在新目录里（copy_crate），被验的 crate 每处都现编，不会拿到上一处的结果
    slots = queue.Queue()
    slot_root = tempfile.mkdtemp(prefix="verus-proof-check-targets-") if registry.cargo_mode else None
    for slot in range(jobs):
        slots.put(os.path.join(slot_root, f"target-{slot}") if slot_root else None)

    def one(index, edit):
        started = time.monotonic()
        slot = slots.get()
        try:
            with tempfile.TemporaryDirectory(prefix="verus-proof-check-") as work:
                crate_copy = copy_crate(root, registry, work)
                for relative, content in edit[1].items():
                    with open(os.path.join(work, relative), "w", encoding="utf-8") as handle:
                        handle.write(content)
                arguments = verus_program_arguments(registry, verus_arguments(registry, manifest, threads, edit[2] if len(edit) > 2 else None,
                                                                              edit[3] if len(edit) > 3 else None))
                if slot:
                    forget_crate_fingerprints(slot, crate_package_name(registry))
                outcome = run_verus(verus_program(registry, executable), arguments, crate_copy,
                                    extra_environment={"CARGO_TARGET_DIR": slot} if slot else None)
        finally:
            slots.put(slot)
        if judged is not None:
            judged(index, outcome, time.monotonic() - started)
        return outcome

    try:
        with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
            futures = [pool.submit(one, index, edit) for index, edit in enumerate(edits)]
            return [future.result() for future in futures]
    finally:
        if slot_root:
            shutil.rmtree(slot_root, ignore_errors=True)


class ItemJudging:
    """一格里逐项判：每项出结局时（verify_in_parallel 的 judged）判出这一项的问题，判绿就当场记进判定库；问题按项的次序交回。
    弄坏开关 reuse-records-whole-cell-only 下不当场记，等整格判完、一处问题都没有才一起记（被打断或有一项红，判过的全不留）；
    reuse-records-red 下红的也记成绿。"""

    def __init__(self, store, judge_one, cell, run="", sentinel=None):
        self.store, self.judge_one, self.cell, self.run, self.sentinel = store, judge_one, cell, run, sentinel
        self.lock = threading.Lock()
        self.created = time.monotonic()
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
        elif outcome_judged(outcome) and "red-keeps-green" not in BREAK:
            # 记红、撤绿（文件头「逐项点名与按条复用」，用户 2026-10-29 定案）：判红的照同一把键记一条红（共用库的 record_red），盖掉这把键原有的绿，
            # 下一趟这一项现判；没判成的（被信号停了）不碰库。弄坏开关 red-keeps-green：判红不记（改前的样子：原有的绿照复用）
            failed_record = self.store.record_red(item, components, one_line(problems[0][0])[:400], run=self.run, details=details)
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
                if "item-done-whole-cell-seconds" in BREAK:   # 弄坏开关：每项都记整格从起到收尾的挂钟（改前没有逐项秒数时的样子），不记这一项自己的
                    seconds = time.monotonic() - self.created
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
    workspace = unit_index(root, {}).workspace()
    for kernel in kernels:
        text = open(kernel, encoding="utf-8").read()
        relative = os.path.relpath(kernel, root).replace(os.sep, "/")
        rust_file = workspace.file(relative)
        lines = text.split("\n")
        every_point = list(lines)
        ordinals = {}
        for requires_line, brace, label in requires_insertion_points(text, relative):
            owner = owner_function(functions, relative, requires_line)
            if owner is None:
                raise Red(f"{relative} 第 {requires_line} 行的 requires 不在 verus! 里的任何 fn 里：认不出它属于哪个函数",
                          "requires 只写在 verus! 里的 fn 与它里面的闭包上；确是新写法，把判法补进 scripts/gate/verus-proof-check.py 的 vacuity_points")
            if "external_body" in unit_attribute_words(rust_file, owner.unit):
                continue
            ordinals[owner.path] = ordinals.get(owner.path, 0) + 1
            changed = list(lines)
            later = brace + 1
            last_header = None
            while later < len(lines):
                code = strip_line_comment(lines[later]).strip()
                if not code:
                    later += 1
                    continue
                if re.match(r"^(?:hide|reveal|reveal_with_fuel)\b", code):
                    last_header = later
                    later += 1
                    continue
                break
            if last_header is not None:
                insert_line = last_header
                indent = " " * (len(lines[insert_line]) - len(lines[insert_line].lstrip()))
                changed[insert_line] = changed[insert_line] + f"\n{indent}assert(false);"
                every_point[insert_line] = every_point[insert_line] + f"\n{indent}assert(false);"
                points.append((f"vacuity:{owner.path}#{ordinals[owner.path]}", f"{relative} {label}", {relative: "\n".join(changed)},
                               (diagnostic_path(registry, relative), insert_line + 2, requires_line), owner, ordinals[owner.path]))
            else:
                changed[brace] = with_assert_false(changed[brace])
                every_point[brace] = with_assert_false(every_point[brace])
                points.append((f"vacuity:{owner.path}#{ordinals[owner.path]}", f"{relative} {label}", {relative: "\n".join(changed)},
                               (diagnostic_path(registry, relative), brace + 1, requires_line), owner, ordinals[owner.path]))
        all_at_once.append((relative, {relative: "\n".join(every_point)}))
    return points, all_at_once


def vacuity_item_problems(label, expected, outcome, options):
    """空真一处的问题（(卡在哪, 下一步)），交回 (问题, 一句判定)。"""
    path, line, requires_line = expected
    problem = outcome_problem(outcome, options)
    if problem:
        return [(f"{label}（第 {requires_line} 行的 requires）：{problem}",
                 problem_howto(outcome, "插断言的副本判不出来：看报错，修好再跑"))], ""
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
    """空真每处 requires 一项：插了 assert(false) 的副本上只验它所属的那一个函数（--verify-root 或 --verify-only-module <核所在的模块>，再 --verify-function；
    kernel_module），键是所属函数的单元闭包、所属函数单元（entry_unit_component）与函数内第几处。"""
    registry, kernels, manifest, executable = prepared(root, options)
    modules = kernel_modules(root, registry, manifest, kernels)
    functions = verus_functions(root, registry, kernels, options)
    points, all_at_once = vacuity_points(root, registry, kernels, functions)
    if not points:
        raise NotRun(f"{len(kernels)} 份核里一处 requires 都没有：没有前置条件，空真这一格无对象可判")
    chosen = selected_items(options, [point[0] for point in points], "verus-vacuity")
    points = [point for point in points if point[0] in chosen]
    store, tally = verdict_library(options, "vacuity-requires", "vacuity"), cell_tally(options)
    shared = shared_components(root, registry, options)
    pending, keyed = [], []
    for item, label, change, expected, owner, ordinal in points:
        components = []
        if shared:
            relative = label.split(" ", 1)[0]
            components = shared + function_components(root, registry, options, [f"{relative}::{owner.unit.name}"], "所属函数单元闭包")
            components += entry_unit_component(owner.unit, "所属函数单元")
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
                            verify_in_parallel(root, registry, manifest, executable, options, all_at_once)))
        problems = []
        for item, label, _change, expected, _components, reason, _owner in pending:
            item_problems, _summary = vacuity_item_problems(label, expected, together[label.split(" ", 1)[0]], options)
            report_item(item, "红" if item_problems else "绿", f"现判（{reason}）")
            problems += item_problems
    else:
        judging = ItemJudging(store, lambda index, outcome: vacuity_item_problems(pending[index][1], pending[index][3], outcome, options), "verus-vacuity",
                              f"{'cargo verus focus' if registry.cargo_mode else 'verus'} --verify-root|--verify-only-module <核所在的模块> --verify-function <所属函数> --rlimit {registry.rlimit}"
                              f"（{registry.tag}），函数体头插 assert(false)",
                              sentinel=sentinel)
        verify_in_parallel(root, registry, manifest, executable, options,
                           [(label, change, owner.name, modules[label.split(" ", 1)[0]]) for _item, label, change, _expected, _components, _reason, owner in pending],
                           judged=lambda index, outcome, seconds: judging(index, outcome, seconds, pending[index][0], pending[index][4],
                                                                          judged_how(store, pending[index][0], pending[index][5], sentinel)))
        problems = judging.finish(len(pending))
        problems += sentinel.finish()
    finish_tally(tally, "verus-vacuity", options)
    if problems:
        report_cell(problems)
        return EXIT_RED
    named = f"（--item 点名的 {len(points)} 处）" if options.get("items") else ""
    print(f"  ✓ 空真：{len(kernels)} 份核里 {len(points)} 处 requires{named}，每一处在函数体头插 assert(false)、只验它所属的函数，都在插的那一行报 assertion failed（未检出入口空真；这不是前置条件可满足性的证明）；"
          f"{assert_by_skipped_text(assert_by_requires_sites(root, kernels))}；按条{tally.text()}")
    return EXIT_GREEN


def must_red_rows(root, registry):
    """必红世界表的行：六段制表符分隔——变异名、文件、原文、替换文、proof_errors=M（条数，只报）、landing=<落点>（判的是它）。
    不是六段的行（或变异名、文件、原文、第五段、第六段有空的）一律判红，报行号、要六段。弄坏开关：row-segments-unchecked 不核段数。"""
    path = os.path.join(root, registry.must_red_table)
    if not os.path.isfile(path):
        raise Red(f"必红世界表 {registry.must_red_table} 不在", "核进了仓就要带它（六段制表符分隔：变异名、文件、原文、替换文、proof_errors=<证明错条数>、landing=<落点>）；路径改了就改登记表的 must-red-worlds 行")
    rows, problems = [], []
    for number, raw in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
        if not raw.strip() or raw.startswith("#"):
            continue
        fields = raw.split("\t")
        required = (0, 1, 2, 4)
        if "row-segments-unchecked" in BREAK:   # 弄坏开关：段数不核，缺的段当空串往下走
            fields += [""] * (6 - len(fields))
        else:
            required += (5,)
        if len(fields) != 6 or not all(fields[index] for index in required):
            problems.append(f"第 {number} 行不是六段（或变异名、文件、原文、第五段 proof_errors=、第六段 landing= 有空的）")
            continue
        rows.append({"number": number, "name": fields[0], "file": fields[1], "original": fields[2].replace("\\n", "\n"),
                     "replacement": fields[3].replace("\\n", "\n"), "expected": fields[4], "landing_text": fields[5]})
    if problems:
        raise Red(f"{registry.must_red_table} 写坏了：" + "；".join(problems), "要六段制表符分隔：变异名、文件、原文、替换文、proof_errors=<证明错条数>、landing=<落点>；照表头的格式改")
    if not rows:
        raise Red(f"{registry.must_red_table} 一行都没有：核没有一个必须报红的世界", "每一层证明至少一条改回去 Verus 就红的变异（show-me-test.md「设计论证同样照办」）")
    return rows


def must_red_item(row):
    return f"must-red:{row['name']}"


def function_keys(root, options, functions):
    """被验区域里每个 fn 的单元闭包键（{全路径: UnitKey}），一格里算一次存进 options。"""
    if options.get("function_keys") is None:
        index = unit_index(root, options)
        keys = {}
        try:
            for function in functions:
                keys[function.path] = index.key([f"{function.relative}::{function.unit.name}"])
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
    pieces = [row["original"], row["replacement"], ",".join(sorted(parse_landing(row["landing_text"]) or ()))]
    if "must-red-key-with-name-and-path" in BREAK:
        pieces = [row["name"], row["file"]] + pieces
    row_component = [] if "reuse-key-without-row" in BREAK else [("这一行", digest_text("\t".join(pieces)))]
    if "reuse-key-whole-crate" in BREAK:
        return shared + [("核 crate", digest_tree(os.path.join(root, registry.crate_directory)))] + row_component
    index = unit_index(root, options)
    position = text.find(row["original"])
    if position == -1:
        first_line = 0
        last_line = 0
    else:
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
    for function in functions:
        members = {(line.split("\t")[0], line.split("\t")[1], line.split("\t")[2].rsplit(":", 1)[0]) for line in keys[function.path].units}
        if identities & members:
            affected.append(f"{function.relative}::{function.unit.name}")
    entries = affected + [f"{row['file']}::{unit.name}" for unit in mutated if f"{row['file']}::{unit.name}" not in affected]
    if not entries:
        return shared + [("受影响的函数单元", digest_text("原文不在任何单元里"))] + row_component
    try:
        units = index.key(entries)
    except ValueError as error:
        raise Red(f"第 {row['number']} 行「{row['name']}」算不出受影响单元的闭包：{error}", "原文所在的单元名字不唯一时让原文只落在一个 fn 里")
    return shared + [("受影响的函数单元", units)] + dependency_component(root, registry, options, units) + row_component


def enclosing_function(rows, line):
    """核文本的行列表 rows 里第 line 行落在哪个 fn 里：往上找最近一行 fn 的开头（FUNCTION_HEADING），交它的名字；往上一行都没有交 None。"""
    for index in range(min(line, len(rows)) - 1, -1, -1):
        found = FUNCTION_HEADING.match(rows[index])
        if found:
            return found.group(1)
    return None


def parse_landing(text):
    """第六段 landing=<落点>[,<落点>…] 读成集合；不是这个形态交 None。"""
    form = EXPECTED_LANDING.fullmatch(text or "")
    return set(form.group(1).split(",")) if form else None


def proof_error_landing(root, registry, files, diagnostics):
    """改坏的副本上 Verus 的证明错（除去 rlimit 用尽）各落在哪个 fn 里，交排好序、去重的落点列表：每条证明错取它 --> 位置的那一行，
    在改坏之后的文件文本里（files 是 {相对仓根的文件: 改坏之后的内容}，别的文件读 root 里的）往上找最近一行 fn 的开头；位置在 crate 之外、文件读不出、
    往上没有 fn 的记（函数外），输出里有证明错没带位置的记（无位置）。落点不数条数：同一个 fn 里多报一条循环不变式，落点还是那一个（C654）。
    弄坏开关 landing-read-from-unmutated-text：读改坏之前的文件（改坏的行数不一样，位置对不上）。"""
    texts = {}

    def rows_of(relative):
        if relative not in texts:
            if relative in files and "landing-read-from-unmutated-text" not in BREAK:
                texts[relative] = files[relative].split("\n")
            else:
                path = os.path.join(root, relative)
                texts[relative] = open(path, encoding="utf-8").read().split("\n") if os.path.isfile(path) else None
        return texts[relative]

    located = [(path, line) for message, path, line in diagnostic_blocks(diagnostics) if RLIMIT_EXHAUSTED not in message]
    found = set()
    for path, line in located:
        rows = None if os.path.isabs(path) else rows_of(located_file(registry, path))
        name = enclosing_function(rows, line) if rows is not None else None
        found.add(name or LANDING_OUTSIDE)
    if len(proof_error_messages(diagnostics)) > len(located):
        found.add(LANDING_NO_LOCATION)
    return sorted(found)


def must_red_row_problems(row, outcome, options, registry, root=None, files=None):
    """必红世界一行改坏之后的结局判出的问题，交回 (问题, 一句判定)；一句判定里带「rlimit 用尽夹证明错」时成功句把它数进去。
    判的是落点（proof_error_landing 与第六段 landing= 相等），条数只报：与第五段 proof_errors= 不同的在一句判定里写「条数只报」，成功句数进去。
    弄坏开关：landing-unchecked 不比落点；landing-by-count 改回按条数判（条数不同判红、不比落点）。"""
    problem = outcome_problem(outcome, options)
    if problem:
        return [(f"第 {row['number']} 行「{row['name']}」：{problem}",
                 problem_howto(outcome, "必红世界要红在证明上：改坏之后编不过的变异不算，换一处只改判定的改法"))], ""
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
        return [(f"第 {row['number']} 行「{row['name']}」：Verus 输出里有 {RLIMIT_EXHAUSTED}，除它、aborting due to 与 cargo 的 could not compile 之外没有别的报错（--rlimit {registry.rlimit}；"
                 f"verified {results.get('verified')}、errors {results.get('errors')}：{listed}），这一行的红不作数",
                 f"这一行红在算力额度上，不是证明失败：改替换文或调高登记的 rlimit（{REGISTRY} 的 rlimit 行）")], ""
    landing = proof_error_landing(root, registry, files or {}, outcome["diagnostics"]) if root is not None else []
    registered_count = row.get("proof_errors")
    summary = f"落点 {'、'.join(landing) or '无'}；证明错 {len(counted)} 条" + ("，rlimit 用尽夹证明错" if rlimit_exhausted else "")
    if registered_count is not None and len(counted) != registered_count:
        summary += f"（条数与登记的 proof_errors={registered_count} 不同，条数只报）"
    if results.get("errors", 0) < 1 and "mutation-accepts-green" not in BREAK:
        return [(f"第 {row['number']} 行「{row['name']}」改坏之后 Verus 照样验过（verified {results.get('verified')}、errors 0）：证明没盯住这一处",
                 "补证明或收紧 spec，让这一处改坏就验不过；确是等价变异就从表里删掉并写明为什么")], summary
    registered = parse_landing(row["landing_text"])
    listed = "；".join(f"{path}:{line} {message}" for message, path, line in diagnostic_blocks(outcome["diagnostics"])[:6])
    if "landing-by-count" in BREAK:   # 弄坏开关：改回按条数判，不比落点
        if registered_count is not None and len(counted) != registered_count and "mutation-accepts-green" not in BREAK:
            return [(f"第 {row['number']} 行「{row['name']}」红了，但证明错 {len(counted)} 条，表里写的是 proof_errors={registered_count}", "按条数判（弄坏开关）")], summary
        return [], summary
    if registered is not None and set(landing) != registered and "landing-unchecked" not in BREAK and "mutation-accepts-green" not in BREAK:
        return [(f"第 {row['number']} 行「{row['name']}」红了，但证明错落在 {'、'.join(landing) or '无'}，表里写的落点是 {'、'.join(sorted(registered))}"
                 f"（--rlimit {registry.rlimit}；verified {results.get('verified')}、errors {results.get('errors')}；证明错 {len(counted)} 条，报的是：{listed}）",
                 "先看红的是不是这条变异要盯的那一处：落到别的函数里，多半是证明或 spec 改过、红已经不是这条变异的本意（改证明或换改法）；"
                 "确是本意（函数改了名、证明挪了位置），照这一次的落点改第六段 landing=。条数跳了不用管，只报不判")], summary
    return [], summary


def prepare_must_red_row(root, registry, row):
    """必红世界一行起 Verus 之前的几样：文件在登记的 crate 里、第五段是 proof_errors=<正整数>、第六段是 landing=<落点>、替换文里没有 rlimit 属性、
    原文在文件里恰好命中一次。交 (问题或 None, 改坏之前的文本, {文件: 改坏之后的内容})；问题是 (卡在哪, 下一步)，有问题的这一行不起 Verus。
    row["proof_errors"] 在这里读出（第五段不是规范形态时 None）。
    弄坏开关：expectation-form-unchecked、landing-form-unchecked 不核第五段、第六段的形态；mutation-rlimit-attribute-accepted、anchor-unchecked 见文件头。"""
    target = os.path.join(root, row["file"])
    if not row["file"].startswith(registry.crate_directory + "/") or not os.path.isfile(target):
        return (f"第 {row['number']} 行「{row['name']}」的文件 {row['file']} 不在登记的 crate 里或不存在", "变异只许改被验的那个 crate 里的文件"), None, None
    form = EXPECTED_PROOF_ERRORS.fullmatch(row["expected"])
    row["proof_errors"] = int(form.group(1)) if form else None
    if form is None and "expectation-form-unchecked" not in BREAK:
        return (f"第 {row['number']} 行「{row['name']}」的第五段「{row['expected']}」不是 proof_errors=<证明错条数>（没起 Verus）", EXPECTATION_HOWTO), None, None
    if parse_landing(row["landing_text"]) is None and "landing-form-unchecked" not in BREAK:
        return (f"第 {row['number']} 行「{row['name']}」的第六段「{row['landing_text']}」不是 landing=<落点>[,<落点>…]（没起 Verus）", LANDING_HOWTO), None, None
    attributes = [match.group(0) for match in (RLIMIT_ATTRIBUTE.search(strip_line_comment(line)) for line in row["replacement"].split("\n")) if match]
    if attributes and "mutation-rlimit-attribute-accepted" not in BREAK:
        return (f"第 {row['number']} 行「{row['name']}」的替换文里有 {attributes[0]}：函数上的 rlimit 属性盖过命令行的 --rlimit {registry.rlimit}，改坏的副本不在登记的额度上验",
                "删掉替换文里的 rlimit 属性；这一行要更多额度才红得出来，就换一处改法（必红世界要在登记的 rlimit 上红）"), None, None
    text = open(target, encoding="utf-8").read()
    hits = text.count(row["original"])
    if hits != 1 and "anchor-unchecked" not in BREAK:
        return (f"第 {row['number']} 行「{row['name']}」的原文在 {row['file']} 里命中 {hits} 次，要恰好命中一次（锚点腐化）",
                "照今天的代码改这一行的原文，让它只命中要改坏的那一处"), None, None
    return None, text, {row["file"]: text.replace(row["original"], row["replacement"], 1)}


def negative_coverage_gaps(root, registry, kernels, functions, rows):
    """静态覆盖：每个带契约的函数、每个循环不变式块都要有唯一锚点落入。
    只判负例的登记完整性；变异是否真的失败仍由 must-red-worlds 验证。
    """
    gaps = []
    for kernel in kernels:
        relative = os.path.relpath(kernel, root).replace(os.sep, "/")
        source = open(kernel, encoding="utf-8").read()
        spans = []
        for row in rows:
            if row["file"] != relative or source.count(row["original"]) != 1:
                continue
            start = source.index(row["original"])
            first = source[:start].count("\n") + 1
            spans.append((first, first + row["original"].count("\n")))
        def covered(first, last):
            return any(low <= last and first <= high for low, high in spans)
        lines = source.splitlines()
        for name, path, unit, _full in functions:
            if path != relative:
                continue
            text = "\n".join(lines[unit.first_line - 1:unit.last_line])
            _kinds, words, _lines, _docs = rust_lex(text)
            if any(word == "ensures" and (index == 0 or words[index - 1] != ".") for index, word in enumerate(words)) and not covered(unit.first_line, unit.last_line):
                gaps.append(f"{relative}:{unit.first_line} 函数 {name}")
        _kinds, words, token_lines, _docs = rust_lex(source)
        for index, word in enumerate(words):
            if word != "invariant":
                continue
            first = token_lines[index]
            last = next((token_lines[later] for later in range(index + 1, len(words))
                         if words[later] == "decreases"), first)
            if not covered(first, last):
                gaps.append(f"{relative}:{first} invariant 块")
    return gaps


def check_negative_coverage(root, options):
    registry = crate_registry(root, options)
    kernels = kernel_files(root, registry)
    functions = verus_functions(root, registry, kernels, options)
    rows = must_red_rows(root, registry)
    gaps = negative_coverage_gaps(root, registry, kernels, functions, rows)
    if gaps:
        raise Red("必红覆盖缺口：" + "；".join(gaps),
                  "给点名的函数或 invariant 块补唯一锚点的负例，再用真实 Verus 验证落点")
    print("  ✓ 必红覆盖：每个带契约的函数与 invariant 块都有唯一锚点负例")
    return EXIT_GREEN


def check_must_red_worlds(root, options):
    registry, kernels, manifest, executable = prepared(root, options)
    functions = verus_functions(root, registry, kernels, options)
    rows = must_red_rows(root, registry)
    chosen = selected_items(options, [must_red_item(row) for row in rows], "verus-must-red-worlds")
    rows = [row for row in rows if must_red_item(row) in chosen]
    store, tally = verdict_library(options, "must-red-world-row", "must-red-worlds"), cell_tally(options)
    problems, edits, judged, texts = [], [], [], {}
    rlimit_with_proof_error = 0
    for row in rows:
        problem, text, files = prepare_must_red_row(root, registry, row)
        if problem:
            problems.append(problem)
            continue
        edits.append((row["name"], files))
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
    judging = ItemJudging(store, lambda index, outcome: must_red_row_problems(pending[index][0], outcome, options, registry, root, pending[index][1][1]), "verus-must-red-worlds",
                          f"{'cargo verus focus' if registry.cargo_mode else 'verus'} --rlimit {registry.rlimit}（{registry.tag}），整个 crate、原文换成替换文的副本", sentinel=sentinel)
    verify_in_parallel(root, registry, manifest, executable, options, [edit for _row, edit, _item, _components, _reason in pending],
                       judged=lambda index, outcome, seconds: judging(index, outcome, seconds, pending[index][2], pending[index][3],
                                                                      judged_how(store, pending[index][2], pending[index][4], sentinel)))
    problems += judging.finish(len(pending))
    problems += sentinel.finish()
    rlimit_with_proof_error = sum(1 for summary in judging.summaries.values() if "rlimit 用尽夹证明错" in summary)
    count_drifted = [pending[index][0]["name"] for index, summary in sorted(judging.summaries.items()) if "条数只报" in summary]
    finish_tally(tally, "verus-must-red-worlds", options)
    if problems:
        report_cell(problems)
        return EXIT_RED
    named = f"（--item 点名的 {len(rows)} 行）" if options.get("items") else ""
    drift = (f"；其中 {len(count_drifted)} 行证明错条数与第五段 proof_errors= 不同，只报不判（{'、'.join(count_drifted[:5])}{'等' if len(count_drifted) > 5 else ''}）"
             if count_drifted else "")
    print(f"  ✓ 必红世界：{registry.must_red_table} 的 {len(rows)} 行{named}每行改坏之后 Verus 都报错，证明错落点与第六段 landing= 相同；原文都恰好命中一次；"
          f"没有一行只红在 rlimit 用尽上（--rlimit {registry.rlimit}；{rlimit_with_proof_error} 行输出里夹着 rlimit 用尽、另有证明错，红计入，rlimit 用尽不计入证明错条数）{drift}；"
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


def rustup_toolchain_fields(toolchain_text, for_key=False):
    """tool-versions 里 rustup 那几段（前面带空格）：
    RUSTUP_HOME=<未设|空串|原值>:<解析到的真实路径>（没设或空串时照 rustup 取 ~/.rustup）；
    rustup_toolchain_lib_sha256=<摘要>:<文件数>，工具链目录 <RUSTUP_HOME>/toolchains/<version.json 的 toolchain 第一个词> 下
      lib/ 顶层全部普通文件与 lib/rustlib/*/lib/ 下全部普通文件（逐层往下），逐个「相对工具链目录的路径 NUL 它的 sha256 换行」按路径字节序接起来再取 sha256；
      version.json 里没有工具链名写「不知道工具链名」，目录不在写「没有」；
    rustup=<PATH 里找到的那一份>:<它的真实路径>:<sha256>（launcher 照 PATH 找 rustup），找不到写「找不到」。
    for_key（按条判定的「工具链」那一样用）：不写 RUSTUP_HOME 的原值与真实路径、不写 rustup 在 PATH 里的位置，rustup 只写 rustup_sha256=<程序的 sha256>：
    同内容的工具链换个目录、PATH 里同一份 rustup 换个位置，判定不变，键也不变；工具链目录下的文件与 rustup 程序换了一个字节照样变。"""
    value = os.environ.get("RUSTUP_HOME")
    shown = "未设" if value is None else "空串" if value == "" else value
    home = os.path.realpath(os.path.abspath(value) if value else RUSTUP_DEFAULT_HOME)
    fields = "" if for_key else f" RUSTUP_HOME={shown}:{home}"
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
    if for_key:
        fields += f" rustup_sha256={streamed_sha256(located)}" if located else " rustup=找不到"
    else:
        fields += f" rustup={located}:{os.path.realpath(located)}:{streamed_sha256(located)}" if located else " rustup=找不到"
    return fields


def tool_versions(root, override):
    try:
        print(tool_versions_text(root, override))
    except Red as red:   # 安装目录取不到（数据根没配）：判红说出路，不抛回溯
        report_red(red.message, red.howto)
        return EXIT_RED
    return EXIT_GREEN


def tool_versions_text(root, override, for_key=False):
    """tool-versions 打的那一行（不带换行）。for_key 交按条判定的「工具链」那一样（键是它的摘要）：同一行去掉不改判定的环境差异——
    RUSTUP_HOME 与 rustup 的位置（rustup_toolchain_fields 的 for_key）、VERUS_ROOT 与 VERUS_Z3_PATH 的原值与解析到的路径、没装时的安装目录，
    只留版本串与内容摘要（Verus 安装、工具链、z3、VERUS_ROOT 目录的内容换了照样变）。"""
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
                shown = "未设" if value is None else "空串" if value == "" else "设了" if for_key and variable == "VERUS_ROOT" else value
                if variable == "VERUS_ROOT" and value and os.path.isdir(value):
                    shown += f":top_level_sha256={top_level_sha256(value)}"
                extra += f" {variable}={shown}"
        if "tool-versions-rustup-ignored" not in BREAK:
            # launcher 经 rustup 起 rust_verify：那条工具链里 rust_verify 读的库、RUSTUP_HOME、rustup 程序本身，换了任何一样这一行都变
            extra += rustup_toolchain_fields((version or {}).get("toolchain") or "", for_key)
        z3_override = os.environ.get("VERUS_Z3_PATH")
        if z3_override is None or (z3_override == "" and "tool-versions-empty-z3-path-as-unset" in BREAK):
            extra += " VERUS_Z3_PATH=未设"
        elif z3_override == "":
            extra += " VERUS_Z3_PATH=空串"   # Verus 不把空串当没设：照它去起 z3、起不来
        else:
            bare = "/" not in z3_override and "tool-versions-bare-z3-from-current-directory" not in BREAK
            located = (shutil.which(z3_override) or "") if bare else z3_override
            if for_key:
                extra += f" VERUS_Z3_PATH=设了:{file_sha256(located) if located else '没有'}"
            else:
                extra += f" VERUS_Z3_PATH={z3_override}:{located or '找不到'}:{file_sha256(located) if located else '没有'}"
    if version is None:
        return f"verus 没装（{'安装目录' if for_key else directory} 里没有 version.json） verus_sha256={digest}{extra}"
    return f"verus version={version.get('version')} commit={version.get('commit')} toolchain={(version.get('toolchain') or '').split(' ')[0]} verus_sha256={digest}{extra}"


# ── 命令行 ──

def default_threads(environment=os.environ):
    """--threads 的缺省：SINGLEFS_THREAD_CAP 设了取它，否则登记的重活默认并行度（.claude/heavy-thread-default）。"""
    if "threads-default-cpu-count" in BREAK:   # 弄坏开关：改前的样子
        return int(environment.get("SINGLEFS_THREAD_CAP") or os.cpu_count() or 1)
    return default_heavy_threads(environment)


def parse_options(arguments):
    options = {"verus_root": None, "cargo_home": None,
               "threads": default_threads(),
               "jobs": int(os.environ.get("GATE_VERUS_JOBS") or 0),
               "items": [], "verdict_library": None, "cargo": None, "rustc_version_file": None}
    positional = []
    flags = {"--verus-root": "verus_root", "--cargo-home": "cargo_home", "--threads": "threads",
             "--jobs": "jobs", "--rustc-version-file": "rustc_version_file"}
    position = 0
    while position < len(arguments):
        argument = arguments[position]
        if argument == "--item":
            if position + 1 >= len(arguments) or not arguments[position + 1]:
                raise ValueError("--item 后面缺项名（可点名的项：verus-proof-check.py list-items <根>）")
            options["items"].append(arguments[position + 1])
            position += 2
        elif argument == "--key-mismatches":
            options["key_mismatches"] = True
            position += 1
        elif argument in flags:
            if position + 1 >= len(arguments):
                raise ValueError(f"{argument} 后面缺值")
            value = arguments[position + 1]
            options[flags[argument]] = int(value) if flags[argument] in ("threads", "jobs") else value
            position += 2
        elif argument.startswith("--"):
            raise ValueError(f"认不出的选项 {argument}")
        else:
            positional.append(argument)
            position += 1
    if options["threads"] < 1 or options["jobs"] < 0:
        raise ValueError("--threads 要正整数，--jobs 不许负")
    return positional, options


CHECKS = {"coverage": check_negative_coverage, "toolchain": check_toolchain, "proof": check_proof, "vacuity": check_vacuity, "must-red-worlds": check_must_red_worlds}


# 逐个 crate 判的几格（登记了几个 crate 时 run_per_crate 一个一个判，--item 照各 crate 列出的项分过去）；toolchain 各 crate 共用一份
PER_CRATE_CHECKS = {"coverage": None, "proof": "verus-proof", "vacuity": "verus-vacuity", "must-red-worlds": "verus-must-red-worlds"}


def run_check(name, root, options):
    options = dict(options, shared_components=None, unit_index=None, function_keys=None)
    try:
        if name == "toolchain" and options.get("items"):
            raise UsageError("verus-toolchain-pinned 格只有一项（toolchain），不收 --item", ["toolchain"])
        if name in PER_CRATE_CHECKS and options.get("crate_view") is None:
            registry = read_registry(os.path.abspath(root))
            if len(registry.crates) > 1:
                return run_per_crate(name, os.path.abspath(root), options, registry)
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


def crate_item_lines(root, registry, options):
    """一个 crate 的可点名的项（只读登记表、核与必红世界表，不起 Verus）：交 (项的那几行 [(项, 格, 说明)], 成项的证明 [ProofItem], 不成项的 [(全路径, 核)], 核在不在)。"""
    try:
        kernels = kernel_files(root, registry)
    except NotRun:
        kernels = []
    functions = verus_functions(root, registry, kernels, options) if kernels else []
    items, left_out = proof_items(root, registry, kernels, functions, options) if kernels else ([], [])
    lines = [(entry.item, "verus-proof", f"{entry.kind} {entry.relative}:{entry.unit.first_line}") for entry in items]
    points, _all_at_once = vacuity_points(root, registry, kernels, functions)
    lines += [(point[0], "verus-vacuity", point[1]) for point in points]
    if kernels:
        lines += [(must_red_item(row), "verus-must-red-worlds", f"第 {row['number']} 行") for row in must_red_rows(root, registry)]
    return lines, items, left_out, bool(kernels)


def run_per_crate(name, root, options, registry):
    """登记了几个 crate 时，一格逐个 crate 判（第二轮正推 Y4 第 1 条：crate 行可以多条、各有各的登记数）：--item 照各 crate 列出的项（crate_item_lines）分过去，
    点名的项一个 crate 都没有的退 2；一个 crate 一项都没点到的这一趟不判它。各 crate 的按条数合成一行报、往门禁的 FORMAL_ITEM_TALLY_FILE 追加一行。
    退出码：有用法错退 2，有红退 1，全是本次未跑退 77，否则 0。"""
    wanted = list(options.get("items") or [])
    owners = {}
    if wanted:
        for view in registry.crates:
            lines, _items, _left_out, _wired = crate_item_lines(root, view, {"unit_index": unit_index(root, options)})
            for item, cell, _note in lines:
                if PER_CRATE_CHECKS.get(name) not in (None, cell):
                    continue
                if item in owners and owners[item] != view.crate_directory:
                    raise Red(f"两个 crate（{owners[item]}、{view.crate_directory}）都有项 {item}", "必红世界表里的变异名在几张表之间也不许重名：给其中一行改名")
                owners[item] = view.crate_directory
        unknown = [item for item in wanted if item not in owners and not (name == "proof" and item == "proof")]
        if unknown and "item-filter-ignored" not in BREAK:
            raise UsageError(f"{PER_CRATE_CHECKS.get(name) or name} 格里没有点名的项：{'、'.join(unknown)}", (["proof"] if name == "proof" else []) + sorted(owners))
    exits, tallies = [], []
    for view in registry.crates:
        mine = [item for item in wanted if owners.get(item) == view.crate_directory or item == "proof"]
        if wanted and not mine and "item-filter-ignored" not in BREAK:
            print(f"  · {view.crate_directory}：--item 没点到这个 crate 的项，这一格这一趟不判它")
            continue
        print(f"  ── {view.crate_directory}{'（cargo verus focus）' if view.cargo_mode else ''}")
        crate_options = dict(options, crate_view=view, items=mine, crate_tallies=tallies, shared_components=None, function_keys=None)
        try:
            exits.append(CHECKS[name](root, crate_options))
        except UsageError:
            raise
        except Red as red:
            report_red(red.message, red.howto)
            exits.append(EXIT_RED)
        except NotRun as not_run:
            print(f"  ! {not_run}")
            exits.append(EXIT_NOT_RUN)
    if tallies and PER_CRATE_CHECKS.get(name):
        total = merged_tally(tallies)
        print(f"  · 按条：{total.text()}")  # gate-lint:detail
        total.append_to(os.environ.get(ITEM_TALLY_FILE_VARIABLE), PER_CRATE_CHECKS[name])
    if EXIT_USAGE in exits:
        return EXIT_USAGE
    if EXIT_RED in exits:
        return EXIT_RED
    if exits and all(code == EXIT_NOT_RUN for code in exits):
        return EXIT_NOT_RUN
    print(f"  ✓ {PER_CRATE_CHECKS.get(name) or name}：登记的 {len(registry.crates)} 个 crate 判了 {len(exits)} 个（{'、'.join(view.crate_directory for view in registry.crates)}），都没红")
    return EXIT_GREEN


def list_items(root, verdict_library=None, verus_root=None, items=None):
    """逐行打「项<TAB>格[<TAB>说明]」：toolchain、proof（整格；说明里报成项几个、各哪一类，不成项的 spec fn 逐个点名）各一项，证明每个成项的单元一项
    proof:<全路径>（说明是哪一类与它在核里的位置），空真每处 requires 一项 vacuity:<所属函数的全路径>#<第几处>，必红世界每行一项（表不是六段的红在这里，同必红世界那一格）。登记了几个 crate 的，逐个 crate 列（全路径的头一段是 crate 名，各 crate 的项不会重名）。
    只读登记表、核与表（核经 lib_function_units.py 切单元），不起 Verus、不编。给了 verdict_library（命令行的 --key-mismatches 取库放在哪那一份）另往 stderr 报每格键对不上的条数
    （report_key_mismatches）。"""
    root = os.path.abspath(root)
    lines = ["toolchain\tverus-toolchain-pinned", "proof\tverus-proof\t整格（有证明义务的单元各一项）"]
    if not os.path.isfile(os.path.join(root, REGISTRY)):
        # 登记表不在（只摆了红例两格的样本根）：只列两项固定的；真要判时那几格自己报登记表不在、判红
        print("\n".join(lines))
        return EXIT_GREEN
    try:
        registry = read_registry(root)
        options = {"unit_index": None}
        all_items, all_left_out, wired, seen, assert_by_sites = [], [], False, {}, []
        for view in registry.crates:
            # 这一个 crate 的成项单元叫 crate_units，不与参数 items（--item 点名的项名，后面交给 report_key_mismatches）同名
            crate_lines, crate_units, left_out, crate_wired = crate_item_lines(root, view, options)
            if crate_wired:
                assert_by_sites += assert_by_requires_sites(root, kernel_files(root, view))
            for item, _cell, _note in crate_lines:
                if item in seen:
                    raise Red(f"两个 crate（{seen[item]}、{view.crate_directory}）都有项 {item}", "必红世界表里的变异名在几张表之间也不许重名：给其中一行改名")
                seen[item] = view.crate_directory
            lines += ["\t".join(line) for line in crate_lines]
            all_items += crate_units
            all_left_out += left_out
            wired = wired or crate_wired
        if wired:
            lines[1] = (f"proof\tverus-proof\t整格（{len(registry.crates)} 个 crate 成项 {len(all_items)} 个：{item_kinds_text(all_items)}" if len(registry.crates) > 1
                        else f"proof\tverus-proof\t整格（成项 {len(all_items)} 个：{item_kinds_text(all_items)}") + \
                (f"；不成项的 spec fn {len(all_left_out)} 个：{listed_names((name for name, _relative in all_left_out), limit=len(all_left_out))}）") + \
                f"；空真{assert_by_skipped_text(assert_by_sites)}"
    except Red as red:
        report_red(f"列不出可点名的项：{red.message}", red.howto)
        return EXIT_RED
    for line in lines:
        print(line)
    if verdict_library and wired:
        report_key_mismatches(root, verdict_library, verus_root, items)
    return EXIT_GREEN


def report_key_mismatches(root, verdict_library, verus_root, items=None):
    """list-items 给了 --key-mismatches：证明、空真、必红世界三格逐项照现在的键比判定库，每格往 stderr 打一行各几条（键对不上、无记录、键相同、拒算），
    末了一行合计（底稿第二节第 9 条 ⑦：各道 --list-items 报每项键对不上的判定条数）。只读库、不写库，不起 Verus（要装着：键里的工具链那一样读它的安装）；
    项名逐行的 stdout 不变。必红世界里原文、第五段、rlimit 属性没过的行不起 Verus、不进库，不数。"""
    plan, plan_items, unreadable = [], [], []
    _positional, defaults = parse_options([])
    registry = read_registry(root)
    views = registry.crates if len(registry.crates) > 1 else [None]
    # 给了 items（--item，可几个）只比点名的那几项：每格只收属于它的项（proof 与 proof:*、vacuity:*、must-red:*），这一格没有点名项的整格不比；
    #   proof 这一项（整格，不按条入库）自己没有键，单独点名它时 proof 格不比。成本随点名项数走。弄坏开关 key-items-ignored 下照旧比整表
    prefixes = {"proof": ("proof:",), "vacuity": ("vacuity:",), "must-red-worlds": ("must-red:",)}
    for cell in ("proof", "vacuity", "must-red-worlds"):
        named = [item for item in (items or []) if item.startswith(prefixes[cell])] if "key-items-ignored" not in BREAK else []
        if items and "key-items-ignored" not in BREAK and not named:
            continue
        for view in views:
            buffer = io.StringIO()
            try:
                with contextlib.redirect_stdout(buffer):
                    CHECKS[cell](root, dict(defaults, items=named, verdict_library=verdict_library, verus_root=verus_root, plan=plan, plan_items=plan_items, shared_components=None,
                                             unit_index=None, function_keys=None, crate_view=view))
            except (Red, NotRun, UsageError) as error:
                unreadable.append(f"{cell}{'（' + view.crate_directory + '）' if view is not None else ''}：{getattr(error, 'message', str(error))}")
    totals = collections.Counter()
    merged = collections.OrderedDict()
    for cell, counts in plan:
        merged.setdefault(cell, collections.Counter()).update(counts)
    for cell, counts in merged.items():
        totals.update(counts)
        print(f"  · {cell}：" + "、".join(f"{state} {counts[state]} 条" for state in PLAN_STATES), file=sys.stderr)  # gate-lint:detail
    for line in unreadable:
        print(f"  ! 这一格的键数不出：{line}", file=sys.stderr)
    print(f"  · 键对不上共 {totals['键对不上']} 条（{len(merged)} 格比了 {sum(totals.values())} 项，判定库 {verdict_library}）", file=sys.stderr)
    # 逐项的键状态，一项一行（门禁 --list-items --with-keys 转成 ITEM_PLAN；没比出状态的项——必红世界里原文、第五段、rlimit 属性没过的行——不打）。
    # 弄坏开关 item-key-lines-skipped 下不打
    if "item-key-lines-skipped" not in BREAK:
        for item, state in plan_items:
            print(f"{ITEM_KEY_PREFIX} item={item} state={state}", file=sys.stderr)


def main(arguments):
    if arguments[:1] == ["--selftest"]:
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


# ── 自证 ──

FAKE_VERUS = r'''#!/usr/bin/env python3
# 假的 Verus：按源码里的标记出判定。只给 verus-proof-check.py --selftest 用。
# 输出照真 Verus 0.2026.09.20 的形态（主 agent 2026-10-15 的探针）：整 crate 验时 verification-results 有 success、is-verifying-entire-crate true；
# --verify-function 下没有 success 键、is-verifying-entire-crate false；vir 错只有 encountered-error 与 encountered-vir-error 两样。
# 义务条数照真 Verus 量到的：proof fn 1 条，exec fn 1 条另加每个 while 1 条，spec fn 带 decreases 的 1 条、不带的 0 条，
# derive(Clone) 的类型另有一个 <类型>::clone（1 条）；函数里有 SELFTEST_ZERO_VERIFIED 的那个函数验出 0 条。
# impl 与 trait 里的方法照真 Verus 列可点名函数的写法记成 <类型>::<方法>（trait impl 也写类型）、<trait>::<方法>。
# --verify-function 照真 Verus 点：有恰好同名的取它，否则唯一包含它的那一个，几个都包含它报 more than one match，一个都没有报 could not find；
# 两头带 * 的照真 Verus 认通配（*<串> 以它收尾、<串>* 以它起头、*<串>* 包含它），点中几个一起验几个（2026-10-29 探针现量）。
# 模块也照真 Verus 0.2026.09.20（2026-10-18 的三层小 crate 探针）：--verify-function 只在 --verify-root（src/lib.rs）或 --verify-only-module <模块>
# （src/<甲>.rs、src/<甲>/mod.rs 是 <甲>，src/<甲>/<乙>.rs 是 <甲>::<乙>，不含子模块）点的那一个模块里找；两样都没给、两样都给、给的是 --verify-module，
# 退 255、stdout 空；点的模块没有报 could not find module。不同模块里同名的函数各是各的。环境里有 SELFTEST_VERUS_ARGUMENTS 的，每起一次往那份文件追加一行命令行参数（JSON；
# --compile 那一趟是 MIR 比对，不记）。
# MIR：带 --compile 与 -o <文件> 的那一趟验完整个 crate 之后往 -o 写一份假 MIR（verus 那一种）；--selftest-mir <product|verus> <crate 目录> <文件> 只写 MIR、不验
# （假 cargo 与假 cargo-verus 用）。假 MIR 每个 fn 一段，正文是那个 fn 的源码行；产品那一种去掉带 SELFTEST_MIR_VERUS_ONLY 的行，verus 那一种去掉带
# SELFTEST_MIR_PRODUCT_ONLY 的行、另多一段擦出来的空壳 selftest_ghost_shell；带 SELFTEST_MIR_RT_PATH 的行产品那一种把 core::panicking:: 印成 std::rt::。
import json, os, re, sys, time
here = os.path.dirname(os.path.realpath(__file__))
commit = json.load(open(os.path.join(here, "version.json")))["verus"]["commit"]
function_line = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:open\s+|closed\s+)?(?:broadcast\s+)?(?:uninterp\s+)?(?:axiom\s+)?(?P<mode>proof\s+|spec\s+|exec\s+)?(?:const\s+)?fn\s+(?P<name>\w+)")
# 带值的 const 与 assume_specification（照真 Verus 0.2026.09.20，2026-10-29 在 core 与探针 crate 上量的）：带值的 const 1 条（spec const 也是 1 条），
# 自由 const 列成它的名字、trait 里的列成 <trait>::<名>、impl 里的列成 impl&%<号>::<名>；assume_specification 指向常量（] 后面不是参数表）且写了 returns 的 1 条、别的 0 条，
# 点名的条目只有标识符与 :: 的照写，别的列成 core::impl&%0::<最后一个标识符>
const_line = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?P<mode>spec\s+)?const\s+(?P<name>\w+)\s*:[^=]*=")
assume_line = re.compile(r"^\s*(?:pub\s+)?assume_specification(?:<[^\[]*>)?\s*\[")
type_line = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum)\s+(?P<name>\w+)")
impl_line = re.compile(r"^(?P<indent>\s*)impl\b(?:\s*<[^>]*>)?\s+(?:(?P<trait>[\w:]+(?:\s*<[^>]*>)?)\s+for\s+)?(?P<target>[\w:]+)")
trait_line = re.compile(r"^(?P<indent>\s*)(?:pub(?:\([^)]*\))?\s+)?trait\s+(?P<name>\w+)")


def module_of(path, source="src"):
    parts = os.path.relpath(path, source)[:-len(".rs")].split(os.sep)
    parts = parts[:-1] if parts[-1] == "mod" else parts
    return "" if parts == ["lib"] else "::".join(parts)


def source_files(source="src"):
    files = []
    for directory, _s, names in os.walk(source):
        for name in sorted(names):
            if name.endswith(".rs"):
                files.append(os.path.join(directory, name))
    return sorted(files)


def blocks_of(path):
    """[(点名的名字, 模式, [源码行])]：一个 fn 从它那一行到下一个 fn 那一行之前；impl / trait 里的照 <类型或 trait>::<方法> 记。"""
    blocks, container, external = [], None, False
    for line in open(path).read().split("\n"):
        if container is not None and line.strip() == "}" and len(line) - len(line.lstrip()) == container[1]:
            container = None
        opened = impl_line.match(line) or trait_line.match(line)
        if opened and line.rstrip().endswith("{"):
            name = (opened.groupdict().get("target") or opened.groupdict().get("name")).split("::")[-1]
            container = (name, len(opened.group("indent")))
        heading = function_line.match(line)
        if heading:
            name = heading.group("name") if container is None else f"{container[0]}::{heading.group('name')}"
            # 带 external_body 的 fn 函数体不验（真 Verus 照它的规格当真、不出义务）：模式记成 trusted，义务 0 条
            blocks.append([name, "trusted" if external else (heading.group("mode") or "exec ").strip(), [line]])
            external = False
        elif blocks:
            blocks[-1][2].append(line)
        if "external_body" in line.split("//")[0]:
            external = True
    return blocks


def mir_text(flavor, source="src"):
    skip = "SELFTEST_MIR_VERUS_ONLY" if flavor == "product" else "SELFTEST_MIR_PRODUCT_ONLY"
    pieces = []
    for path in source_files(source):
        module = module_of(path, source)
        for name, _mode, lines in blocks_of(path):
            body = [line.strip() for line in lines if line.strip() and skip not in line]
            if flavor == "product":
                # 带 SELFTEST_MIR_RT_PATH 的行照真 rustc 的产品构建把 core::panicking:: 印成 std 的再导出 std::rt::（Verus 构建照 core 的路径印）
                body = [line.replace("core::panicking::", "std::rt::") if "SELFTEST_MIR_RT_PATH" in line else line for line in body]
            pieces.append(f"fn {module + '::' if module else ''}{name}() -> () {{\n" + "".join(f"    {line}\n" for line in body) + "}\n")
    if flavor == "verus":
        pieces.append("fn selftest_ghost_shell() -> () {\n}\n")
    return "\n".join(pieces)


if sys.argv[1:2] == ["--selftest-mir"]:
    flavor, crate_directory, out = sys.argv[2:5]
    open(out, "w").write(mir_text(flavor, os.path.join(crate_directory, "src")))
    sys.exit(0)
compile_output = sys.argv[sys.argv.index("-o") + 1] if "--compile" in sys.argv and "-o" in sys.argv[:-1] else None
if os.environ.get("SELFTEST_VERUS_ARGUMENTS") and compile_output is None:
    open(os.environ["SELFTEST_VERUS_ARGUMENTS"], "a").write(json.dumps(sys.argv[1:]) + "\n")
no_cheating = "--no-cheating" in sys.argv
rlimit = float(sys.argv[sys.argv.index("--rlimit") + 1]) if "--rlimit" in sys.argv[:-1] else None
only = sys.argv[sys.argv.index("--verify-function") + 1] if "--verify-function" in sys.argv[:-1] else None
only_module = sys.argv[sys.argv.index("--verify-only-module") + 1] if "--verify-only-module" in sys.argv[:-1] else None
if only is not None:
    refusal = ("When using --verify-function, use --verify-only-module instead of --verify-module" if "--verify-module" in sys.argv
               else "--verify-function option requires --verify-only-module or --verify-root" if "--verify-root" not in sys.argv and only_module is None
               else "Must pass at most one --verify-only-module or --verify-root when using --verify-function" if "--verify-root" in sys.argv and only_module is not None
               else None)
    if refusal:
        sys.stderr.write(f"Error: {refusal}\n")
        sys.exit(255)
target_module = None if only is None else only_module or ""
if os.environ.get("SELFTEST_VERUS_TRACE"):
    open(os.environ["SELFTEST_VERUS_TRACE"], "a").write("起了一次\n")
files = source_files()
# 键是「模块\0点名的名字」：不同模块里同名的各是各的
obligations, order, home = {}, [], {}
for path in files:
    module, derive_clone = module_of(path), False
    for name, mode, lines in blocks_of(path):
        key = f"{module}\0{name}"
        obligations[key] = 0 if mode in ("spec", "trusted") else 1
        order.append(key)
        home[key] = module
        for line in lines[1:]:
            if mode == "spec" and re.search(r"\bdecreases\b", line) and obligations[key] == 0:
                obligations[key] = 1
            if mode == "exec" and re.match(r"^\s*while\b", line):
                obligations[key] += 1
        for line in lines:
            if "SELFTEST_ZERO_VERIFIED" in line:
                obligations[key] = -10 ** 6
    for line in open(path).read().split("\n"):
        declared = type_line.match(line)
        if declared and derive_clone:
            key = f"{module}\0{declared.group('name')}::clone"
            obligations[key] = 1
            order.append(key)
            home[key] = module
        derive_clone = line.lstrip().startswith("#[derive(") and "Clone" in line
    container, impls = None, 0
    for line in open(path).read().split("\n"):
        if container is not None and line.strip() == "}" and len(line) - len(line.lstrip()) == container[1]:
            container = None
        opened = impl_line.match(line) or trait_line.match(line)
        if opened and line.rstrip().endswith("{"):
            impls += 1
            container = (impls, len(opened.group("indent")), opened.groupdict().get("name"))
        constant = const_line.match(line)
        named = None
        if constant:
            owner = None if container is None else container[2] or f"impl&%{container[0]}"
            named = (constant.group("name") if owner is None else f"{owner}::{constant.group('name')}", 1)
        specification = assume_line.match(line)
        if specification:
            start = specification.end() - 1
            depth, end = 0, None
            for position in range(start, len(line)):
                depth += {"[": 1, "]": -1}.get(line[position], 0)
                if depth == 0:
                    end = position
                    break
            point = line[start + 1:end].strip() if end is not None else ""
            rest = line[end + 1:].lstrip() if end is not None else ""
            plain = re.fullmatch(r"[A-Za-z_]\w*(?:::[A-Za-z_]\w*)*", point)
            last = (re.findall(r"[A-Za-z_]\w*", point) or ["?"])[-1]
            named = (point if plain else f"core::impl&%0::{last}", 1 if rest.startswith("->") and "returns" in rest else 0)
        if named:
            key = f"{module}\0{named[0]}"
            obligations[key] = named[1]
            order.append(key)
            home[key] = module
selected = None
if target_module and target_module not in {module_of(path) for path in files}:
    sys.stderr.write(f"error: could not find module {target_module} specified by --verify-module or --verify-only-module\n"
                     "error: aborting due to 1 previous error\n\n")
    print(json.dumps({"verification-results": {"encountered-error": True, "encountered-vir-error": True}, "verus": {"commit": commit}}))
    sys.exit(1)
if only is not None:
    order = [key for key in order if home[key] == target_module]
    names = {key: key.split("\0", 1)[1] for key in order}
    exact = [key for key in order if names[key] == only]
    if len(only) > 1 and only.startswith("*") and only.endswith("*"):
        selected = [key for key in order if only[1:-1] in names[key]]
    elif only.startswith("*"):
        selected = [key for key in order if names[key].endswith(only[1:])]
    elif only.endswith("*"):
        selected = [key for key in order if names[key].startswith(only[:-1])]
    elif exact:
        selected = exact[:1]
    else:
        matched = [key for key in order if only in names[key]]
        if len(matched) > 1:
            sys.stderr.write(f"error: more than one match found for --verify-function {only}, consider using wildcard *{only}* to verify all matched results,\n"
                             "       or specify a unique substring for the desired function, matched results are:\n"
                             + "".join(f"         - {names[key]}\n" for key in matched) + "\nerror: aborting due to 1 previous error\n\n")
            print(json.dumps({"verification-results": {"encountered-error": True, "encountered-vir-error": True}, "verus": {"commit": commit}}))
            sys.exit(1)
        selected = matched[:1]
selected_names = {key.split("\0", 1)[1] for key in selected or ()}
functions, errors, vir, found = 0, [], [], bool(selected)
for key in order:
    if only is None or key in selected:
        functions += max(obligations[key], 0)
for path in files:
    lines = open(path).read().split("\n")
    owner_vacuous = owner_failed = closure_vacuous = False
    current = None
    in_scope = only is None or module_of(path) == target_module
    container = None
    for number, line in enumerate(lines, 1):
        if "SELFTEST_KILLED" in line or (os.environ.get("SELFTEST_KILL_MARK") and os.environ["SELFTEST_KILL_MARK"] in line):
            os.kill(os.getpid(), 9)
        if no_cheating and re.search(r"\bassume\(", line.split("//")[0]):
            vir.append(("assume/admit not allowed with --no-cheating", path, number, 1))
        if no_cheating and re.search(r"\b(assume_specification|external_body)\b", line.split("//")[0]):
            vir.append(("external_body/assume_specification not allowed with --no-cheating", path, number, 1))
        if container is not None and line.strip() == "}" and len(line) - len(line.lstrip()) == container[1]:
            container = None
        opened = impl_line.match(line) or trait_line.match(line)
        if opened and line.rstrip().endswith("{"):
            container = ((opened.groupdict().get("target") or opened.groupdict().get("name")).split("::")[-1], len(opened.group("indent")))
        heading = function_line.match(line)
        if heading:
            current = heading.group("name") if container is None else f"{container[0]}::{heading.group('name')}"
        if only is not None and (current not in selected_names or not in_scope):
            continue
        sleeping = re.search(r"SELFTEST_SLEEP_(\d+)", line)
        if sleeping:
            time.sleep(int(sleeping.group(1)) / 10)   # SELFTEST_SLEEP_<n>：验到这一行睡 n/10 秒（逐项秒数各算各的那一格用）
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
        if "SELFTEST_BROKEN" in line or (os.environ.get("SELFTEST_FAIL_MARK") and os.environ["SELFTEST_FAIL_MARK"] in line):
            errors.append(("postcondition not satisfied", path, number, 1))   # SELFTEST_FAIL_MARK：环境里点名的字（不进键）也当证明错，记红撤绿那一格用
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
if compile_output is not None and not errors:
    open(compile_output, "w").write(mir_text("verus"))
sys.exit(1 if errors else 0)
'''

# 假的 cargo（产品构建那一趟，MIR 比对用）与假的 cargo-verus（cargo 那一种 crate 起的 cargo verus focus / build）：只给 --selftest 用。
# 两份都照 -p 点的包在副本里找成员（当前目录的 Cargo.toml 是那个包，或工作区 members 里名字对得上的那一个）；
# 假 cargo 只认 rustc：往 $CARGO_TARGET_DIR/debug/deps/<包名>-0.mir 写产品那一种假 MIR。
# 假 cargo-verus 认 verus focus 与 verus build：focus 在成员目录里起假 Verus（-- 之后的参数原样交给它），报错位置照真 cargo verus 改写成相对副本根的路径；
# build 照样验整个 crate，再往 $CARGO_TARGET_DIR/debug/deps/<包名>-1.mir 写 verus 那一种假 MIR。环境里有 SELFTEST_CARGO_VERUS_ARGUMENTS 的，每起一次追加一行参数（JSON）。
# focus 照真 cargo 的编译指纹：同一个编译目录里 <编译目录>/verus-partial/debug/.fingerprint/<包名>-fake 在就只打一行 Finished、不出判定行，不在就建它再验。
FAKE_CARGO_COMMON = r'''#!/usr/bin/env python3
import json, os, subprocess, sys, tomllib
here = os.path.dirname(os.path.realpath(__file__))
verus = os.path.join(os.environ.get("SELFTEST_FAKE_VERUS_DIRECTORY") or here, "verus")


def member_of(package):
    own = os.path.join(os.getcwd(), "Cargo.toml")
    manifest = tomllib.load(open(own, "rb"))
    if manifest.get("package", {}).get("name") == package:
        return os.getcwd()
    for member in manifest.get("workspace", {}).get("members", []):
        path = os.path.join(os.getcwd(), member)
        if tomllib.load(open(os.path.join(path, "Cargo.toml"), "rb")).get("package", {}).get("name") == package:
            return path
    sys.stderr.write(f"error: package `{package}` not found\n")
    sys.exit(101)


arguments = sys.argv[1:]
package = arguments[arguments.index("-p") + 1]
member = member_of(package)
deps = os.path.join(os.environ.get("CARGO_TARGET_DIR") or os.path.join(os.getcwd(), "target"), "debug", "deps")
'''

FAKE_CARGO = FAKE_CARGO_COMMON + r'''
if arguments[:1] != ["rustc"]:
    sys.stderr.write(f"假 cargo 只认 rustc：{arguments}\n")
    sys.exit(101)
os.makedirs(deps, exist_ok=True)
sys.exit(subprocess.run([verus, "--selftest-mir", "product", member, os.path.join(deps, package.replace("-", "_") + "-0.mir")]).returncode)
'''

FAKE_CARGO_VERUS = FAKE_CARGO_COMMON + r'''
if os.environ.get("SELFTEST_CARGO_VERUS_ARGUMENTS"):
    open(os.environ["SELFTEST_CARGO_VERUS_ARGUMENTS"], "a").write(json.dumps(arguments) + "\n")
subcommand = arguments[1]
rest = arguments[arguments.index("--") + 1:]
# cargo 的编译指纹：同一个编译目录里被验的包验过一回就当没变，focus 只打一行 Finished、不起 Verus、stdout 没有判定行（真 cargo verus 的样子）
fingerprint = os.path.join(os.environ.get("CARGO_TARGET_DIR") or os.path.join(os.getcwd(), "target"), "verus-partial", "debug", ".fingerprint", package + "-fake")
if subcommand == "focus":
    if os.path.isdir(fingerprint):
        sys.stderr.write("    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s\n")
        sys.exit(0)
    os.makedirs(fingerprint)
mir = "--emit=mir" in rest
if mir:
    rest = rest[:rest.index("--")] if "--" in rest else [word for word in rest if word != "--emit=mir"]
relative = os.path.relpath(member, os.getcwd())
ran = subprocess.run([verus, *rest, "src/lib.rs"], cwd=member, capture_output=True, text=True)
sys.stdout.write(ran.stdout)
sys.stderr.write(ran.stderr.replace("--> src/", f"--> {relative}/src/"))
if ran.returncode:
    # 真 cargo 编不过时自己另打一句（没有 --> 位置），不是证明错（判法的 proof_error_messages 去掉它）
    previous = sum(1 for line in ran.stderr.split("\n") if line.startswith("error") and "aborting due to" not in line)
    sys.stderr.write(f"error: could not compile `{package}` (lib) due to {previous} previous error{'' if previous == 1 else 's'}\n")
if subcommand == "build" and mir and ran.returncode == 0:
    os.makedirs(deps, exist_ok=True)
    subprocess.run([verus, "--selftest-mir", "verus", member, os.path.join(deps, package.replace("-", "_") + "-1.mir")], check=True)
sys.exit(101 if ran.returncode else 0)
'''

# 被验区域那几格的自证用：插在 SELFTEST_KERNEL 的「} // verus!」之前（methods：impl 与 trait 里的方法成项；trust：要登记的三种可信声明；
# trait：被验的 trait，另在 verus! 外面接一个 trait impl 与一个 cfg(test) 下的 impl）
SELFTEST_METHODS_EXTRA = """
pub trait Device {
    fn read_slot(&self, slot: u64) -> (r: u64)
        ensures
            r <= 10;
}

pub struct Region {
    pub start: u64,
}

impl Region {
    pub fn contains(&self, slot: u64) -> (r: bool)
        ensures
            r == (self.start <= slot),
    {
        self.start <= slot
    }
}

impl Device for Region {
    fn read_slot(&self, slot: u64) -> (r: u64)
    {
        3
    }
}
"""

SELFTEST_TRUST_EXTRA = """
pub assume_specification<T: Ord>[ <[T]>::binary_search ](s: &[T], x: &T) -> (r: Result<usize, usize>)
    ensures
        r matches Ok(i) ==> i < s.len();

#[verifier::external_body]
pub fn read_device(x: u64) -> (r: u64)
    ensures
        r == x,
{
    x
}

pub uninterp spec fn crc(x: u64) -> u64;
"""

# 带值的 const 与 assume_specification 成项那一格用（proof-valued-const-and-assume-specification-items）：impl 里的关联 const、自由 const、
# 指向常量写 returns 的 assume_specification（假 Verus 验出 1 条）、指向 std 方法的 assume_specification（0 条）
SELFTEST_CONST_EXTRA = """
pub struct Width(pub u64);

impl Width {
    pub const NARROWEST: Self = Self(1);
}

pub const WIDEST_SAMPLE: u64 = 64;

pub assume_specification[ selftest_format::SLOT_WIDTH ] -> (width: u64) returns 8u64;

pub assume_specification<T: Ord>[ <[T]>::binary_search ](s: &[T], x: &T) -> (r: Result<usize, usize>)
    ensures
        r matches Ok(i) ==> i < s.len();
"""

# 一个 trait impl 里的几个方法闭包相同那两格用（reuse-proof-entry-unit-own-key、reuse-vacuity-owner-unit-own-key；core 的 impl BlockDevice for FileBackedBlockDevice 的样子）：
# 方法连它的 impl 头、impl 头点名类型、类型照 R-隐式收它的 trait impl 全部成员，两个方法收到的单元一模一样；fill 带一个 while（假 Verus 验出 2 条）、width 1 条
SELFTEST_IMPL_SIBLINGS_EXTRA = """
pub trait SampleStore {
    fn fill(&self, count: u64) -> (filled: u64)
        ensures
            filled == count;

    fn width(&self) -> (width: u64)
        ensures
            width <= 64;
}

pub struct SampleDevice {
    pub width: u64,
}

impl SampleStore for SampleDevice {
    fn fill(&self, count: u64) -> (filled: u64) {
        let mut filled: u64 = 0;
        while filled < count
            invariant
                filled <= count,
            decreases count - filled,
        {
            filled = filled + 1;
        }
        filled
    }

    fn width(&self) -> (width: u64) {
        if self.width > 64 { 64 } else { self.width }
    }
}
"""

# 同一个 trait impl 里两个方法各一处 requires，头一个的前置条件空真（假 Verus 认 SELFTEST_VACUOUS：插的 assert(false) 照样验过）
SELFTEST_IMPL_SIBLINGS_VACUITY_EXTRA = """
pub trait SampleGate {
    fn open_gate(&self, level: u64) -> (opened: u64);

    fn close_gate(&self, level: u64) -> (closed: u64);
}

pub struct SampleLatch {
    pub level: u64,
}

impl SampleGate for SampleLatch {
    fn open_gate( /* SELFTEST_VACUOUS */ &self, level: u64) -> (opened: u64)
        requires
            level > 0,
    {
        level
    }

    fn close_gate(&self, level: u64) -> (closed: u64)
        requires
            level > 0,
    {
        level
    }
}
"""

# assume_specification 指到工作区里另一个包的常量那一格用（reuse-assume-specification-three-questions；core 指 singlefs_format 常量的样子）：
# 两条各指一个常量，第三个常量没人指
SELFTEST_ASSUME_TARGETS_EXTRA = """
pub assume_specification[ singlefs_format::SLOT_WIDTH ] -> (width: u64) returns 8u64;

pub assume_specification[ singlefs_format::RING_START ] -> (start: u64) returns 16u64;
"""
SELFTEST_ASSUME_TARGETS_FORMAT = "pub const SLOT_WIDTH: u64 = 8;\npub const RING_START: u64 = 16;\npub const UNUSED_SAMPLE: u64 = 3;\n"

# 关联 const 点名那几格用（list-items-associated-const-prefix、proof-associated-const-prefix-green、proof-associated-const-point-names）：
# 关联 const FIRST 是自由 const FIRST_COUNTER 的前一段，自由 const LAST 是关联 const LAST_SLOT 的前一段
SELFTEST_CONST_PREFIX_EXTRA = """
pub struct Ordinal(pub u32);

impl Ordinal {
    pub const FIRST: Self = Self(1);

    pub const LAST_SLOT: u64 = 9;
}

pub const FIRST_COUNTER: u64 = 1;

pub const LAST: u64 = 2;
"""

# 同一个模块里另一个 impl 也有 FIRST（proof-point-name-two-impls-collision-red）
SELFTEST_CONST_SECOND_IMPL_EXTRA = """
pub struct OtherOrdinal(pub u32);

impl OtherOrdinal {
    pub const FIRST: Self = Self(2);
}
"""

# trait 里带默认值的 const 与 impl 里覆盖它的同名 const（proof-point-name-trait-default-collision-red）
SELFTEST_CONST_TRAIT_DEFAULT_EXTRA = """
pub struct Ordinal(pub u32);

pub trait Limited {
    const LIMIT: u64 = 3;
}

impl Limited for Ordinal {
    const LIMIT: u64 = 4;
}
"""

# 空真插入点认 const fn 那一格用（vacuity-const-fn-label）：带 requires 的 const fn 接在 keep_small 后面
SELFTEST_CONST_FN_EXTRA = """
pub const fn to_slot_offset(slot: u64) -> (offset: u64)
    requires
        slot < 1000,
    ensures
        offset == slot * 8,
{
    slot * 8
}
"""

# 空真插入点不认 assert(…) by (…) 的 requires 那几格用（list-items-assert-by-requires-skipped、vacuity-assert-by-requires-skipped、
# proof-assert-by-requires-skip-reported）：core 的 bytes.rs、unit.rs、journal.rs 里 by (nonlinear_arith) 接 requires … ; 的写法
SELFTEST_ASSERT_BY_EXTRA = """
pub fn halve_within(value: u64, bound: u64) -> (result: u64)
    requires
        value < 2 * bound,
    ensures
        result < bound,
{
    let result = value / 2;
    proof {
        assert(result < bound) by (nonlinear_arith)
            requires
                value < 2 * bound,
                result == value / 2,
        ;
    }
    result
}
"""

# requires 挂在普通 proof 块后面、往下一行只有 ;（list-items-requires-on-plain-block-red）
SELFTEST_PLAIN_BLOCK_REQUIRES_EXTRA = """
pub fn halve_plain(value: u64) -> (result: u64)
    ensures
        result <= value,
{
    let result = value / 2;
    proof {
        assert(result <= value);
    }
        requires
            value > 1,
    ;
    result
}
"""

# requires 挂在不带 by 的 assert 上、往下一行只有 ;（list-items-requires-on-assert-without-by-red）
SELFTEST_ASSERT_WITHOUT_BY_EXTRA = """
pub fn halve_unproved(value: u64) -> (result: u64)
    ensures
        result <= value,
{
    let result = value / 2;
    proof {
        assert(result <= value)
            requires
                value > 1,
        ;
    }
    result
}
"""

# spec const 成项那一格用（proof-spec-const-items）：impl 里的与自由的 spec const 各一个（真 Verus 各验出 1 条，点 *::<名> 与 <名>）
SELFTEST_SPEC_CONST_EXTRA = """
pub struct Window(pub u64);

impl Window {
    pub spec const SPEC_WIDTH: u64 = 4;
}

pub spec const SPEC_HEIGHT: u64 = 3;
"""

SELFTEST_TRAIT_EXTRA = """
pub trait Device {
    fn read_slot(&self, slot: u64) -> (r: u64)
        ensures
            r <= 10;
}
"""

SELFTEST_TRAIT_OUTSIDE = """
pub struct BadDevice;

impl Device for BadDevice {
    fn read_slot(&self, _slot: u64) -> u64 { 99 }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Mock;
    impl Device for Mock {
        fn read_slot(&self, _slot: u64) -> u64 { 1 }
    }
}
"""

# 登记式放过表那几格用（trait-impl-lag-*）：放过表登记 BadDevice 那一行；同一个 trait 在 verus! 外面另有一个没登记的 impl；BadDevice 已挪进 verus! 里的样子
SELFTEST_TRAIT_LAG_ROW = "core_sample::kernel::Device\tBadDevice\tcrates/core/src/kernel.rs\t会话丢弃与实例表清扫\t自证用：这一簇还没把它挪进 verus!"

SELFTEST_TRAIT_OUTSIDE_SECOND = """
pub struct WorseDevice;

impl Device for WorseDevice {
    fn read_slot(&self, _slot: u64) -> u64 { 98 }
}
"""

SELFTEST_TRAIT_INSIDE = """
pub struct BadDevice;

impl Device for BadDevice {
    fn read_slot(&self, _slot: u64) -> (r: u64)
    {
        3
    }
}
"""

# verus! 外面一个 trait 与类型都不在被验区域里的 impl（放过表里登记它不红、没放过什么，成功句点名）
SELFTEST_UNVERIFIED_OUTSIDE = """
pub trait Plain {
    fn plain(&self) -> u64;
}

pub struct Other;

impl Plain for Other {
    fn plain(&self) -> u64 { 1 }
}
"""

# external_body 的规格原文那一格用（trust-digest-signature-only）：impl 里唯一（也是最后）的 fn 带 external_body
SELFTEST_IMPL_EXTERNAL_BODY = """
pub struct Shown;

impl Shown {
    #[verifier::external_body]
    pub fn show(&self, x: u64) -> (shown: u64)
        ensures
            shown == x,
    {
        x
    }
}
"""

# trait 里没有函数体的声明、ensures 里有结构体字面量那一格用（proof-trait-declaration-struct-literal）：Store::put 不成项，Shelf::put 成项
SELFTEST_TRAIT_LITERAL_EXTRA = """
pub struct Request {
    pub slot: u64,
}

pub open spec fn issued(request: Request) -> bool {
    request.slot < 10
}

pub trait Store {
    fn put(&mut self, slot: u64) -> (r: bool)
        ensures
            r == issued(Request { slot: slot }),
    ;
}

pub struct Shelf;

impl Store for Shelf {
    fn put(&mut self, slot: u64) -> (r: bool)
    {
        slot < 10
    }
}
"""


def captured_stdout(function, *arguments):
    """自证用：起 function，交 (它的返回值, 收下的 stdout)（StringIO，读 getvalue()）。"""
    buffer = io.StringIO()
    with contextlib.redirect_stdout(buffer):
        returned = function(*arguments)
    return returned, buffer


def selftest_kernel_with(extra, after=""):
    """SELFTEST_KERNEL 的 verus! 里另加 extra，verus! 之后接 after。"""
    return SELFTEST_KERNEL.replace("} // verus!\n", extra + "} // verus!\n") + after


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


# 必红世界按落点判的自证用：tiny 的函数体带一个原来就有的标记（假 Verus 在这一行报一条证明错），在 clamp_to_limit 与 keep_small 之间；
# 改坏 clamp_to_limit 的替换文比原文多几行时，tiny 里那一行在改坏之后的文件里挪到 keep_small 在改坏之前的文件里的行数上
SELFTEST_LANDING_KERNEL = """use vstd::prelude::*;
verus! {
pub fn clamp_to_limit(value: u64, limit: u64) -> (result: u64)
    requires
        limit > 0,
    ensures
        result <= limit,
{
    if value > limit { limit } else { value }
}

pub fn tiny(value: u64) -> (result: u64)
    ensures
        result <= 3,
{
    if value > 3 { 3 } else { value } // SELFTEST_BROKEN
}

pub fn keep_small(value: u64) -> (result: u64)
    ensures
        result <= 10,
{
    if value > 10 { 10 } else { value }
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


def selftest_break_caught(returncode, output):
    """故障注入必须被具体自证格检出；崩溃、信号或用法错不算证明会红。"""
    return returncode == EXIT_RED and re.search(r"(?m)^\s*✗ .+ 这一格判错[：:]", output) is not None


def selftest_break_miss_detail(returncode, output):
    """开关轮里没检出的那一趟为什么没检出（selftest_break_caught 交 False 之后调）：被信号停了、崩了（输出里有 Python 回溯）、
    退 0（每格都判对）、退 1 却没有一格报「这一格判错」、别的退出码；后面接输出的末一个非空行（崩了的那就是回溯末行的异常）。"""
    lines = [line.strip() for line in output.splitlines() if line.strip()]
    last = lines[-1][-200:] if lines else "（没有输出）"
    if returncode < 0:
        why = f"被信号 {-returncode} 停了"
    elif "Traceback (most recent call last):" in output:
        why = f"退 {returncode}、崩了（输出里有 Python 回溯）"
    elif returncode == EXIT_GREEN:
        why = "退 0、每格都判对（开关没让任何一格红）"
    elif returncode == EXIT_RED:
        why = "退 1 却没有一格报「这一格判错」"
    else:
        why = f"退 {returncode}"
    return f"{why}；输出末行：{last}"


def selftest():
    # 判定库那几格写进 prune-server：只在临时服务上跑（scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service），
    # 不是就一格都不跑，不写真服务
    client = lib_verdict_library.prune_server_client()
    service_problem = client.selftest_service_problem() if client is not None else "这棵树里没有 scripts/prune-server/prune_server_client.py"
    if service_problem:
        print(f"  ✗ verus-proof-check 自证没跑：{service_problem}")
        print(f"     → 怎么办：{client.SELFTEST_SERVICE_REMEDY if client is not None else '在带着 scripts/prune-server/ 的仓里跑'}")
        return 2
    os.environ.pop(NOT_RUN_FILE_VARIABLE, None)   # 同上：放过项报本次未跑那一格自己建一份，别的格不往门禁的汇总里写
    # task-server 套的内存包装给的外层上限不进自证：jobs-not-narrowed-by-outer-memory 那一格自己设（开关 outer-memory-narrows-jobs 下别的格不跟着收）
    os.environ.pop(OUTER_MEMORY_CAP_VARIABLE, None)
    # 复用哨兵：别的格比的是现判与复用的条数，键相同的全当哨兵、不抽强制现判（哨兵数比项数多）；reuse-three-questions 与 reuse-sentinel 两格自己设
    os.environ[reuse_checks.SENTINEL_COUNT_VARIABLE] = "1000"
    cases, wrong = 0, []
    with tempfile.TemporaryDirectory(prefix="verus-proof-check-selftest-") as work:
        commit, other_commit = "a" * 40, "b" * 40
        install = os.path.join(work, "tools", "verus-x86-linux")
        os.makedirs(install)
        for tool, text in (("verus", FAKE_VERUS), ("cargo", FAKE_CARGO), ("cargo-verus", FAKE_CARGO_VERUS)):
            with open(os.path.join(install, tool), "w") as handle:
                handle.write(text)
            os.chmod(os.path.join(install, tool), 0o755)
        rustc_version_file = os.path.join(work, "rustc-version.txt")
        open(rustc_version_file, "w").write(f"rustc 1.0.0 (dddddddd 2026-01-01)\nbinary: rustc\ncommit-hash: {'d' * 40}\nrelease: 1.0.0\n")
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
                      rlimit=7, trust_rows=(), trust_list=True, rustc_row=f"rustc\tstable\t1.0.0\t{'d' * 40}\n", crate_mode=""):
            root = os.path.join(work, name)
            os.makedirs(os.path.join(root, ".claude", "gate.d"))
            os.makedirs(os.path.join(root, "crates", "core", "src"))
            os.makedirs(os.path.join(root, "crates", "core", "tests"))
            json.dump({"verus": {"version": "0.9.9.tag", "commit": version_commit, "toolchain": "stable"}}, open(os.path.join(install, "version.json"), "w"))
            open(os.path.join(root, REGISTRY), "w").write(
                "# 自证用\n"
                f"release\t0.9.9.tag\t{commit}\t{zip_digest}\tunused\n"
                f"package\tvstd\t0.0.1\t{'c' * 40}\tsource/vstd\t{fingerprint}\n"
                "crate\tcrates/core\t%d%s\n" % (count, crate_mode) +
                "kernel\tcrates/core/src/kernel.rs\n"
                "must-red-worlds\tcrates/core/verus-must-red-worlds.tsv\n" + (f"rlimit\t{rlimit}\n" if rlimit is not None else "") + rustc_row)
            if trust_list:
                open(os.path.join(root, TRUST_LIST), "w").write(f"# 自证用\nvstd\t0.0.1\t{fingerprint}\t自证用：vstd 整体登记\n" + "".join(row + "\n" for row in trust_rows))
            open(os.path.join(root, "crates", "core", "tests", "differential.rs"), "w").write(
                "#[test]\nfn selftest_differential() {}\n\n#[test]\n#[ignore]\nfn selftest_differential_other() {}\n")
            open(os.path.join(root, "Cargo.lock"), "w").write(f'version = 4\n\n[[package]]\nname = "vstd"\nversion = "{lock_version}"\n')
            open(os.path.join(root, "crates", "core", "Cargo.toml"), "w").write(
                '[package]\nname = "core-sample"\nedition = "2021"\n\n[dependencies]\n' + ('vstd = "=0.0.1"\n' if wired else "") + extra_dependency)
            open(os.path.join(root, "crates", "core", "src", "lib.rs"), "w").write("pub mod kernel;\n" if with_kernel else "")
            if with_kernel:
                open(os.path.join(root, "crates", "core", "src", "kernel.rs"), "w").write(kernel)
            rows = must_red if must_red is not None else ["收紧改松\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\tif value > limit { limit } else { value } // SELFTEST_BROKEN\tproof_errors=1\tlanding=clamp_to_limit"]
            open(os.path.join(root, "crates", "core", "verus-must-red-worlds.tsv"), "w").write("# 自证用\n" + "".join(row + "\n" for row in rows))
            return root

        options = {"verus_root": install, "cargo_home": cargo_home, "threads": 2, "jobs": 2,
                   "rustc_version_file": rustc_version_file}
        fresh_libraries = []

        def fresh_library():
            """一份新的空库（库目录在这一趟的临时目录底下，只当名字：临时服务底下另占一个命名空间）：只看判法不看复用的那几格各用一份，不串。"""
            fresh_libraries.append(os.path.join(work, f"fresh-library-{len(fresh_libraries)}"))
            return fresh_libraries[-1]
        # MIR 比对的产品构建照 PATH 找 cargo：假 cargo 放在最前面（只在这一趟自证的进程里）
        os.environ["PATH"] = install + os.pathsep + os.environ.get("PATH", "")

        def expect(label, check, root, want_exit, want_text):
            nonlocal cases
            cases += 1
            import io
            import contextlib
            buffer = io.StringIO()
            with contextlib.redirect_stdout(buffer):
                got = run_check(check, root, dict(options, verdict_library=fresh_library()))
            output = buffer.getvalue()
            want_texts = want_text if isinstance(want_text, tuple) else (want_text,)
            if got != want_exit or any(text not in output for text in want_texts):
                wrong.append(label)
                print(one_line(f"  ✗ {label} 这一格判错：期望退 {want_exit}、输出含「{'」「'.join(want_texts)}」，实测退 {got}：{output.strip()[-300:]}"))  # gate-lint:detail
            return output

        def expect_absent(label, output, unwanted):
            """一格已经跑出来的输出里不许有 unwanted（输出是 expect 交回的那一份，不再起一遍 Verus）。"""
            nonlocal cases
            cases += 1
            if unwanted in output:
                wrong.append(label)
                print(one_line(f"  ✗ {label} 这一格判错：期望输出里没有「{unwanted}」，实测有：{output.strip()[-300:]}"))  # gate-lint:detail

        omitted_root = make_root("unregistered-proof-module")
        open(os.path.join(omitted_root, "crates/core/src/omitted.rs"), "w").write(
            "use vstd::prelude::*;\nverus! { pub proof fn bad() ensures false, {} }\n")
        open(os.path.join(omitted_root, "crates/core/src/lib.rs"), "a").write("pub mod omitted;\n")
        expect("unregistered-proof-module", "proof", omitted_root, 1, "未登记的 Verus 核")
        inline_requires = SELFTEST_KERNEL.replace("    requires\n        limit > 0,", "    requires limit > 0,")
        inline_requires = inline_requires.replace("-> (result: u64)\n    requires limit > 0,", "-> (result: u64) requires limit > 0,")
        expect("inline-requires-refused", "vacuity", make_root("inline-requires", kernel=inline_requires), 1, "拒绝漏检")
        coverage_root = make_root("negative-coverage")
        expect("negative-coverage-missing", "coverage", coverage_root, 1, "函数 keep_small")
        with open(os.path.join(coverage_root, "crates/core/verus-must-red-worlds.tsv"), "a") as handle:
            handle.write("keep-small-contract\tcrates/core/src/kernel.rs\tresult <= 10,\tresult < 10,\tproof_errors=1\tlanding=keep_small\n")
        expect("negative-coverage-complete", "coverage", coverage_root, 0, "每个带契约的函数")
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
        # Verus 被信号停了：输出不算判定、出路是查谁停了它，不写修证明（开关 killed-as-proof-red 下这一格与 proof-killed-no-proof-howto 判错）
        killed = expect("proof-killed-direct", "proof", make_root("killeddirect", kernel=SELFTEST_KERNEL + "// SELFTEST_KILLED\n"), 1,
                        ("Verus 被信号停了（退 -9，信号 9）", "查是谁在那一刻停了它"))
        cases += 1
        if "照 Verus 报错的位置修" in killed or "看上面 Verus 的报错" in killed:
            wrong.append("proof-killed-no-proof-howto")
            print(one_line(f"  ✗ proof-killed-no-proof-howto 这一格判错：被信号停了的那一条不该给修证明的出路：{killed.strip()[-300:]}"))  # gate-lint:detail
        vacuous = SELFTEST_KERNEL.replace("pub fn clamp_to_limit(", "pub fn clamp_to_limit( /* SELFTEST_VACUOUS */")
        expect("vacuity-vacuous", "vacuity", make_root("vacuous", kernel=vacuous), 1, "空真")
        expect("vacuity-closure-found", "vacuity", green, 0, "2 处 requires")
        closure_vacuous = SELFTEST_KERNEL.replace("pub fn keep_small(value: u64) -> (result: u64)\n",
                                                  "pub fn keep_small(value: u64) -> (result: u64)\n    requires\n        value < 100,\n").replace(
            "            candidate <= 10,", "            candidate <= 10, // SELFTEST_CLOSURE_VACUOUS")
        expect("vacuity-closure-vacuous", "vacuity", make_root("closurevacuous", kernel=closure_vacuous), 1, "keep_small 里第 20 行起的闭包：函数体头插 assert(false) 照样验得过")
        expect("crate-extra-dependency", "proof", make_root("extradependency", extra_dependency='singlefs-format = { path = "../format" }\n'), 1, "除 vstd 之外还依赖 singlefs-format")

        # ── 被验区域与多 crate（出口 5 第三轮判决第三节「门禁」；文件头「被验区域与多 crate 的弄坏开关」）──
        def expect_true(label, passed, detail):
            nonlocal cases
            cases += 1
            if not passed:
                wrong.append(label)
                print(one_line(f"  ✗ {label} 这一格判错：{detail}"))  # gate-lint:detail

        # 按全路径逐项：不同模块里同名的各是各的项；impl 与 trait 里的方法成项（<类型>::<方法>），trait 里没有函数体的声明不成项（开关 methods-refused 下判错）
        two_modules = make_root("same-name-in-two-modules", count=4)
        with open(os.path.join(two_modules, REGISTRY), "a") as handle:
            handle.write("kernel\tcrates/core/src/other.rs\n")
        open(os.path.join(two_modules, "crates/core/src/other.rs"), "w").write(SELFTEST_KERNEL)
        open(os.path.join(two_modules, "crates/core/src/lib.rs"), "a").write("pub mod other;\n")
        expect("same-name-in-two-modules", "proof", two_modules, 0,
               ("proof:core_sample::kernel::clamp_to_limit：绿", "proof:core_sample::other::clamp_to_limit：绿", "成项 4 个（exec fn 4）"))
        expect("proof-method-items", "proof", make_root("methoditems", kernel=selftest_kernel_with(SELFTEST_METHODS_EXTRA), count=4), 0,
               ("proof:core_sample::kernel::Region::contains：绿", "proof:core_sample::kernel::Region::read_slot：绿", "成项 4 个（exec fn 4）"))
        same_point = selftest_kernel_with(SELFTEST_METHODS_EXTRA + "\nimpl Region {\n    pub fn read_slot(&self, slot: u64) -> (r: u64)\n    {\n        4\n    }\n}\n")
        expect("proof-same-point-name", "proof", make_root("samepoint", kernel=same_point, count=5), 1, "交给 --verify-function 的名字都是 Region::read_slot")
        # 属性写法（#[verus_verify]）标的文件也是核：没登记判红（开关 attribute-mode-unrecognized 下判错）
        attribute_root = make_root("unregistered-attribute-kernel")
        open(os.path.join(attribute_root, "crates/core/src/attributed.rs"), "w").write("#[verus_verify]\npub fn attributed(x: u64) -> u64 { x }\n")
        open(os.path.join(attribute_root, "crates/core/src/lib.rs"), "a").write("pub mod attributed;\n")
        expect("unregistered-attribute-kernel", "proof", attribute_root, 1, "未登记的 Verus 核：crates/core/src/attributed.rs（属性写法")
        # 信任清单：没登记判红，红句给现算的 sha256；登记齐了判绿；sha256 不对、差分测试不在、登记了却没有那一处、vstd 行对不上、表不在，各判红
        trust_kernel = selftest_kernel_with(SELFTEST_TRUST_EXTRA)
        unregistered = expect("trust-unregistered", "proof", make_root("trustunregistered", kernel=trust_kernel), 1,
                              ("assume_specification <[T]>::binary_search 没登记进信任清单", "external_body core_sample::kernel::read_device 没登记进信任清单",
                               "uninterp core_sample::kernel::crc 没登记进信任清单"))
        digests = dict(re.findall(r"的 (?:assume_specification|external_body|uninterp) (\S+) 没登记进信任清单 \S+（这一处的规格原文 sha256 ([0-9a-f]{64})）", unregistered))
        expect_true("trust-digests-reported", len(digests) == 3, f"没登记的三处期望各报一个现算的 sha256，读到 {digests}")

        def trust_rows(test="selftest_differential", wrong_digest=None, extra=()):
            rows = []
            for kind, point in (("assume_specification", "<[T]>::binary_search"), ("external_body", "core_sample::kernel::read_device"), ("uninterp", "core_sample::kernel::crc")):
                digest = digests.get(point, "0" * 64)
                rows.append(f"{kind}\tcrates/core/src/kernel.rs\t{point}\t{wrong_digest if wrong_digest and kind == 'external_body' else digest}\t{test}\t自证用：登记的可信声明")
            return rows + list(extra)

        trust_green = expect("trust-registered-green", "proof", make_root("trustregistered", kernel=trust_kernel, trust_rows=trust_rows()), 0,
                             ("成项 3 个（exec fn 2、assume_specification 1）", "可信声明与 cfg 原子 3 处都登记在信任清单里"))
        expect("trust-digest", "proof", make_root("trustdigest", kernel=trust_kernel, trust_rows=trust_rows(wrong_digest="e" * 64)), 1,
               "external_body core_sample::kernel::read_device 登记的规格原文 sha256 是 eeeeeeeeeeee…")
        expect("trust-test-missing", "proof", make_root("trusttest", kernel=trust_kernel, trust_rows=trust_rows(test="no_such_test")), 1,
               "点名的差分测试 no_such_test 在 crates/ 下找不到标了 #[test] 的函数")
        stale = "external_body\tcrates/core/src/kernel.rs\tcore_sample::kernel::gone\t" + "f" * 64 + "\tselftest_differential\t自证用：删了的那一处"
        expect("trust-stale-row", "proof", make_root("truststale", kernel=trust_kernel, trust_rows=trust_rows(extra=[stale])), 1,
               "登记了 external_body core_sample::kernel::gone（crates/core/src/kernel.rs），核的被验区域里没有这一处")
        vstd_root = make_root("trustvstd")
        edit_text = open(os.path.join(vstd_root, TRUST_LIST)).read().replace("vstd\t0.0.1\t", "vstd\t0.0.2\t")
        open(os.path.join(vstd_root, TRUST_LIST), "w").write(edit_text)
        expect("trust-vstd", "proof", vstd_root, 1, "的 vstd 0.0.2")
        expect("trust-list-missing", "proof", make_root("trustmissing", trust_list=False), 1, "信任清单 .claude/gate.d/verus-trust-list.tsv 不在")
        # 不带 --no-cheating：登记了的 assume_specification 与 external_body 照验（开关 no-cheating-kept 下假 Verus 照真 Verus 拒，trust-registered-green 判错）
        expect_true("no-cheating-dropped", "--no-cheating" not in trust_green and "成项 3 个" in trust_green, f"登记齐了期望判绿：{one_line(trust_green.strip()[-300:])}")
        # axiom 一律判红（开关 axiom-accepted 下判错）
        axiom_kernel = selftest_kernel_with("\npub axiom fn lemma_false()\n    ensures\n        false;\n")
        expect("axiom-red", "proof", make_root("axiom", kernel=axiom_kernel, count=3), 1, "核里有 axiom：crates/core/src/kernel.rs:")
        # cfg 原子：cfg!(…)、#[cfg(…)]、宏定义里有 cfg 的宏在被验区域里的调用，没登记判红；登记了判绿（开关 cfg-atom-accepted、cfg-macro-unchecked 下判错）
        body = "if value > limit { limit } else { value }"
        cfg_kernel = SELFTEST_KERNEL.replace(body, "if cfg!(debug_assertions) { " + body + " } else { " + body + " }")
        cfg_output = expect("cfg-atom-red", "proof", make_root("cfgatom", kernel=cfg_kernel), 1, "cfg core_sample::kernel::clamp_to_limit#cfg!(debug_assertions) 没登记进信任清单")
        cfg_digest = re.search(r"#cfg!\(debug_assertions\) 没登记进信任清单 \S+（这一处的规格原文 sha256 ([0-9a-f]{64})）", cfg_output)
        cfg_row = f"cfg\tcrates/core/src/kernel.rs\tcore_sample::kernel::clamp_to_limit#cfg!(debug_assertions)\t{cfg_digest.group(1) if cfg_digest else '0' * 64}\tselftest_differential\t自证用：登记的 cfg 原子"
        expect("cfg-atom-registered", "proof", make_root("cfgregistered", kernel=cfg_kernel, trust_rows=[cfg_row]), 0, "其中 cfg 原子 1 处")
        macro_kernel = SELFTEST_KERNEL.replace("verus! {\n", "macro_rules! plus_span {\n    ($e:expr) => { if cfg!(verus_keep_ghost) { $e } else { $e } };\n}\nverus! {\n", 1).replace(body, f"plus_span!({body})")
        expect("cfg-macro-red", "proof", make_root("cfgmacro", kernel=macro_kernel), 1, "cfg core_sample::kernel::clamp_to_limit#plus_span! 没登记进信任清单")
        fork_kernel = SELFTEST_KERNEL.replace("pub fn keep_small(", "#[cfg(feature = \"wide\")]\npub fn keep_small(", 1) + \
            "verus! {\n#[cfg(not(feature = \"wide\"))]\npub fn keep_small(value: u64) -> (result: u64)\n    ensures\n        result <= 10,\n{\n    value % 10\n}\n} // verus!\n"
        expect("cfg-feature-fork", "proof", make_root("cfgfork", kernel=fork_kernel), 1,
               ("keep_small#cfg(feature=\"wide\") 没登记进信任清单", "交给 --verify-function 的名字都是 keep_small"))
        # 被验的 trait 在 verus! 外面有 impl 判红，cfg(test) 下的不判（开关 trait-impl-outside-accepted 下判错）
        trait_output = expect("trait-impl-outside", "proof", make_root("traitoutside", kernel=selftest_kernel_with(SELFTEST_TRAIT_EXTRA, SELFTEST_TRAIT_OUTSIDE)), 1,
                              "verus! 外面有 trait impl：crates/core/src/kernel.rs:")
        expect_absent("trait-impl-test-module-not-judged", trait_output, "Mock")

        # 登记式放过（放过表 .claude/gate.d/verus-trait-impls-outside-lag.tsv）：登记了的 verus! 外面的 impl 放过，成功句逐个点名「契约没证、靠信任」、这一格不算绿到底
        # （开关 trait-impl-lag-passed-unnamed 下判错）
        def lag_root(name, kernel, rows, count=2):
            root = make_root(name, kernel=kernel, count=count)
            open(os.path.join(root, TRAIT_IMPLS_OUTSIDE_LAG), "w").write("# 自证用\n" + "".join(row + "\n" for row in rows))
            return root

        # 另登记一行 trait 与类型都不在被验区域里的 impl（那一簇的补丁还没打上的样子）：不红、没放过什么，成功句点名
        lag_kernel = selftest_kernel_with(SELFTEST_TRAIT_EXTRA, SELFTEST_TRAIT_OUTSIDE)
        plain_row = "core_sample::kernel::Plain\tOther\tcrates/core/src/kernel.rs\t会话丢弃与实例表清扫\t自证用：它的 trait 还不在被验区域里"
        expect("trait-impl-lag-registered-green", "proof", lag_root("traitlag", lag_kernel + SELFTEST_UNVERIFIED_OUTSIDE, [SELFTEST_TRAIT_LAG_ROW, plain_row]), 0,
               (f"都登记在 {TRAIT_IMPLS_OUTSIDE_LAG} 里放过、这一格不算绿到底", "core_sample::kernel::Device for BadDevice（crates/core/src/kernel.rs:",
                "契约没证、靠信任，归「会话丢弃与实例表清扫」挪进 verus!",
                "放过表里 1 行的 trait 与类型此刻都不在被验区域里、没放过什么：第 3 行 core_sample::kernel::Plain for Other",
                "放过表：被判的根不在 git 仓里，没与 gate 基准比"))
        # 放过的每一处另往 $GATE_NOT_RUN_FILE 报一行本次未跑（只写在成功句里不算报了；开关 trait-impl-lag-not-run-unreported 下判错）；变量不在时不写
        not_run_file = os.path.join(work, "trait-impl-lag-not-run.txt")
        open(not_run_file, "w").close()
        lag_not_run_root = lag_root("traitlagnotrun", lag_kernel + SELFTEST_UNVERIFIED_OUTSIDE, [SELFTEST_TRAIT_LAG_ROW, plain_row])
        os.environ[NOT_RUN_FILE_VARIABLE] = not_run_file
        try:
            not_run_exit, not_run_output = captured_stdout(run_check, "proof", lag_not_run_root, dict(options, verdict_library=fresh_library()))
        finally:
            os.environ.pop(NOT_RUN_FILE_VARIABLE, None)
        not_run_lines = [line for line in open(not_run_file, encoding="utf-8").read().split("\n") if line]
        expect_true("trait-impl-lag-not-run-reported",
                    not_run_exit == 0 and len(not_run_lines) == 1 and "verus-proof 格（crates/core）：verus! 外面的 core_sample::kernel::Device for BadDevice（crates/core/src/kernel.rs:" in not_run_lines[0]
                    and "契约本次未验" in not_run_lines[0] and "放过的 1 处逐个往 $GATE_NOT_RUN_FILE 报了本次未跑" in not_run_output.getvalue(),
                    f"期望退 0、$GATE_NOT_RUN_FILE 里恰好一行点名 BadDevice、成功句写报了，实测退 {not_run_exit}、文件里 {not_run_lines}：{one_line(not_run_output.getvalue().strip()[-300:])}")
        # 同一个 trait 在外面另有一个没登记的 impl：照旧判红（开关 trait-impl-lag-matched-by-trait-only 下它也被那一行放过、判错）
        expect("trait-impl-lag-unregistered-red", "proof", lag_root("traitlagextra", lag_kernel + SELFTEST_TRAIT_OUTSIDE_SECOND, [SELFTEST_TRAIT_LAG_ROW]), 1,
               ("verus! 外面有 trait impl：crates/core/src/kernel.rs:", "<impl Device for WorseDevice>"))
        # 登记了的已挪进 verus!、登记了的在那份文件里找不到：都判红（开关 trait-impl-lag-stale-unchecked 下判错）
        gone_row = "core_sample::kernel::Device\tGoneDevice\tcrates/core/src/kernel.rs\t会话丢弃与实例表清扫\t自证用：这个 impl 已经删了"
        expect("trait-impl-lag-stale-rows-red", "proof",
               lag_root("traitlagstale", selftest_kernel_with(SELFTEST_TRAIT_EXTRA + SELFTEST_TRAIT_INSIDE), [SELFTEST_TRAIT_LAG_ROW, gone_row], count=3), 1,
               ("第 2 行登记放过的 core_sample::kernel::Device for BadDevice（crates/core/src/kernel.rs）已经挪进 verus! 里了",
                "第 3 行登记放过的 core_sample::kernel::Device for GoneDevice（crates/core/src/kernel.rs）在那份文件里找不到"))
        # 只缩不涨：被判的根是 git 仓、gate 基准（没有上游取 HEAD）里的表只有 BadDevice 那一行，照旧判绿、成功句写比过；
        # 再登记一个 WorseDevice 判红（开关 trait-impl-lag-growth-unchecked 下两格都判错）
        grown_root = lag_root("traitlaggrown", lag_kernel, [SELFTEST_TRAIT_LAG_ROW])
        git_environment = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")}
        for arguments in (["init", "-q"], ["add", "-A"],
                          ["-c", "user.name=selftest", "-c", "user.email=selftest@invalid", "-c", "commit.gpgsign=false", "commit", "-q", "-m", "自证用的基准"]):
            subprocess.run(["git", *arguments], cwd=grown_root, env=git_environment, check=True, capture_output=True)
        saved_git = {name: os.environ.pop(name) for name in list(os.environ) if name.startswith("GIT_") or name == "GATE_BASE"}
        try:
            expect("trait-impl-lag-baseline-green", "proof", grown_root, 0, ("放过表：与 gate 基准 ", " 那一版比过、没涨"))
            with open(os.path.join(grown_root, "crates", "core", "src", "kernel.rs"), "a") as handle:
                handle.write(SELFTEST_TRAIT_OUTSIDE_SECOND)
            with open(os.path.join(grown_root, TRAIT_IMPLS_OUTSIDE_LAG), "a") as handle:
                handle.write("core_sample::kernel::Device\tWorseDevice\tcrates/core/src/kernel.rs\t会话丢弃与实例表清扫\t自证用：基准里没有这一行\n")
            expect("trait-impl-lag-grown-red", "proof", grown_root, 1,
                   ("第 3 行 core_sample::kernel::Device for WorseDevice（crates/core/src/kernel.rs）在 gate 基准 ", "这张表只缩不涨"))
        finally:
            os.environ.update(saved_git)

        # external_body 的规格原文是签名到函数体的 { 之前：impl 里唯一（也是最后）的 fn 取得到签名、不是空串，改签名 sha256 就变、只改函数体不变；
        # 后面还接着别的条目的 fn 只改函数体也不变（开关 body-open-past-unit-end 下判错：impl 里最后一个 fn 取成空串，别的取成整个 fn）
        def external_body_digests(name, kernel):
            root = make_root(name, kernel=kernel)
            relative = "crates/core/src/kernel.rs"
            rust_file = FunctionUnitIndex(root).workspace().file(relative)
            # 按条目名的最后一段认（Shown::show 记 show）：点名的全路径怎么拼不归这一格
            return {entry["point"].split("::")[-1]: entry["digest"] for entry in trusted_declarations(rust_file, relative, "core_sample", "kernel", region_spans(rust_file), set())
                    if entry["kind"] == "external_body"}

        plain_digests = external_body_digests("externalbodyplain", selftest_kernel_with(SELFTEST_TRUST_EXTRA + SELFTEST_IMPL_EXTERNAL_BODY))
        signature_digests = external_body_digests("externalbodysignature", selftest_kernel_with(
            SELFTEST_TRUST_EXTRA.replace("        r == x,", "        r >= x,") + SELFTEST_IMPL_EXTERNAL_BODY.replace("shown == x,", "shown >= x,")))
        body_digests = external_body_digests("externalbodybody", selftest_kernel_with(
            SELFTEST_TRUST_EXTRA.replace("{\n    x\n}", "{\n    x + 0\n}") + SELFTEST_IMPL_EXTERNAL_BODY.replace("        x\n", "        x + 0\n")))
        last_point, other_point, empty_digest = "show", "read_device", hashlib.sha256(b"").hexdigest()
        expect_true("trust-digest-signature-only",
                    all(point in digests for point in (last_point, other_point) for digests in (plain_digests, signature_digests, body_digests))
                    and plain_digests[last_point] != empty_digest
                    and all(plain_digests[point] != signature_digests[point] and plain_digests[point] == body_digests[point] for point in (last_point, other_point)),
                    f"期望 impl 里最后一个 fn 的 sha256 不是空串的、改签名就变、只改函数体不变，实测 原样 {plain_digests}、改签名 {signature_digests}、改函数体 {body_digests}"
                    f"（空串的 sha256 是 {empty_digest[:12]}…）")

        # uninterp、external_type_specification 与带 external_body 的类型规格取整个条目，照半开区间取：后面接着的条目换了头一个词（pub fn 换成 fn），
        # 三处的 sha256 都不变；改条目自己就变（开关 whole-entry-past-unit-end 下判错：连下一个条目的头一个词一起取）
        def whole_entry_digests(name, extra):
            root = make_root(name, kernel=selftest_kernel_with(extra))
            relative = "crates/core/src/kernel.rs"
            rust_file = FunctionUnitIndex(root).workspace().file(relative)
            return {(entry["kind"], entry["point"].split("::")[-1]): entry["digest"]
                    for entry in trusted_declarations(rust_file, relative, "core_sample", "kernel", region_spans(rust_file), set())
                    if entry["kind"] in ("uninterp", "external_type_specification") or entry["point"].endswith("OfDuration")}

        type_specification = ("\n#[verifier::external_type_specification]\n#[verifier::external_body]\n"
                              "pub struct ExternalTypeSpecificationOfDuration(std::time::Duration);\n")
        following = "\npub fn after_the_entry(x: u64) -> (r: u64)\n    ensures\n        r == x,\n{\n    x\n}\n"
        between = "pub fn between(x: u64) -> (r: u64)\n    ensures\n        r == x,\n{\n    x\n}\n"
        entries_then_public = whole_entry_digests("wholeentrypublic", type_specification + between + SELFTEST_TRUST_EXTRA + following)
        entries_then_private = whole_entry_digests("wholeentryprivate", type_specification + between.replace("pub fn", "fn", 1) + SELFTEST_TRUST_EXTRA
                                                   + following.replace("\npub fn", "\nfn", 1))
        entries_changed = whole_entry_digests("wholeentrychanged", type_specification.replace("Duration);", "Duration, );") + between
                                              + SELFTEST_TRUST_EXTRA.replace("crc(x: u64) -> u64;", "crc(x: u64) -> u32;") + following)
        whole_keys = [("uninterp", "crc"), ("external_type_specification", "ExternalTypeSpecificationOfDuration"), ("external_body", "ExternalTypeSpecificationOfDuration")]
        expect_true("trust-digest-whole-entry-half-open",
                    all(key in digests for key in whole_keys for digests in (entries_then_public, entries_then_private, entries_changed))
                    and all(entries_then_public[key] == entries_then_private[key] and entries_then_public[key] != entries_changed[key] for key in whole_keys),
                    f"期望三处整个条目的 sha256 不随下一个条目的头一个词变、改条目自己就变，实测 后接 pub {entries_then_public}、后接 fn {entries_then_private}、"
                    f"改条目 {entries_changed}")
        # cfg 原子的规格原文是它所在的条目整个，与 uninterp 同一种半开区间：clamp_to_limit 里一处 cfg!(debug_assertions)，后面接着的 keep_small 换了头一个词
        # （pub fn 换成 fn）sha256 不变，改 clamp_to_limit 自己就变（开关 cfg-entry-past-unit-end 下判错：连 keep_small 的头一个词一起取；whole-entry-past-unit-end 下同样判错）
        def cfg_entry_digests(name, kernel):
            root = make_root(name, kernel=kernel)
            relative = "crates/core/src/kernel.rs"
            rust_file = FunctionUnitIndex(root).workspace().file(relative)
            return [entry["digest"] for entry in trusted_declarations(rust_file, relative, "core_sample", "kernel", region_spans(rust_file), set())
                    if entry["kind"] == CFG_KIND]

        cfg_then_public = cfg_entry_digests("cfgentrypublic", cfg_kernel)
        cfg_then_private = cfg_entry_digests("cfgentryprivate", cfg_kernel.replace("pub fn keep_small(", "fn keep_small(", 1))
        cfg_changed = cfg_entry_digests("cfgentrychanged", cfg_kernel.replace("limit > 0,", "limit > 1,", 1))
        expect_true("trust-digest-cfg-entry-half-open",
                    len(cfg_then_public) == len(cfg_then_private) == len(cfg_changed) == 1
                    and cfg_then_public == cfg_then_private and cfg_then_public != cfg_changed,
                    f"期望 cfg 原子所在条目的 sha256 不随下一个条目的头一个词变、改条目自己就变，实测 后接 pub {cfg_then_public}、后接 fn {cfg_then_private}、"
                    f"改条目 {cfg_changed}")

        # trait 里没有函数体的声明、ensures 里有结构体字面量：按切出的单元认（以 ; 收尾），不成项（开关 trait-declaration-by-brace 下 Store::put 成项、之和 4 与登记的 3 不同）
        expect("proof-trait-declaration-struct-literal", "proof", make_root("traitliteral", kernel=selftest_kernel_with(SELFTEST_TRAIT_LITERAL_EXTRA), count=3), 0,
               ("proof:core_sample::kernel::Shelf::put：绿", "成项 3 个（exec fn 3）"))
        # MIR：产品构建与 Verus 构建的函数体不同判红（开关 mir-compare-skipped 下判错）；判绿时成功句报比了几个条目
        expect("mir-divergence", "proof", make_root("mirdivergence", kernel=SELFTEST_KERNEL.replace(body, body + " // SELFTEST_MIR_VERUS_ONLY")), 1,
               "MIR 比对：产品构建 2 个条目里，1 个与 Verus 构建的函数体不同（fn kernel::clamp_to_limit）")
        expect("mir-green", "proof", green, 0, "产品构建与 verus --compile 构建的 MIR 逐条目比：产品的 2 个条目都在、整段逐字相同")
        # MIR 规整表（MIR_PATH_ALIASES）：产品构建印 std::rt::panic_fmt、Verus 构建印 core::panicking::panic_fmt 的同一个函数，规整之后判绿、报两边各改写几处
        # （开关 mir-panic-path-unnormalized 下判错）；同一个条目里另有一处真分叉照报（开关 mir-aliased-items-skipped 下判错）；
        # 表外的同类名字照红（开关 mir-alias-by-prefix 下判错）；前面接 ::、后面接标识符字符的不改（开关 mir-alias-without-boundary 下判错）
        rt_call = " // SELFTEST_MIR_RT_PATH _6 = core::panicking::panic_fmt(move _7) -> bb18;"
        aliased_kernel = SELFTEST_KERNEL.replace(body, body + rt_call).replace(
            "if value > 10 { pick(10) } else { pick(value) }", "if value > 10 { pick(10) } else { pick(value) } // SELFTEST_MIR_RT_PATH _3 = core::panicking::panic_fmt::<u64>(move _4);")
        expect("mir-panic-path-normalized-green", "proof", make_root("mirpanicpath", kernel=aliased_kernel), 0,
               ("MIR 逐条目比：产品的 2 个条目都在、整段逐字相同",
                "比之前照规整表改写了产品构建 2 处、Verus 构建 0 处（std::rt::panic_fmt → core::panicking::panic_fmt，同一个函数的两种印法）"))
        diverged_kernel = SELFTEST_KERNEL.replace(body, body + rt_call).replace("        limit > 0,\n", "        limit > 0, // SELFTEST_MIR_VERUS_ONLY\n", 1)
        expect("mir-panic-path-divergence-red", "proof", make_root("mirpanicdivergence", kernel=diverged_kernel), 1,
               ("MIR 比对：产品构建 2 个条目里，1 个与 Verus 构建的函数体不同（fn kernel::clamp_to_limit）", "比之前照规整表改写了产品构建 1 处、Verus 构建 0 处"))
        unlisted_kernel = SELFTEST_KERNEL.replace(body, body + " // SELFTEST_MIR_RT_PATH _3 = core::panicking::panic_display::<u64>(move _4) -> bb2;")
        expect("mir-unlisted-panic-path-red", "proof", make_root("mirunlisted", kernel=unlisted_kernel), 1,
               ("1 个与 Verus 构建的函数体不同（fn kernel::clamp_to_limit）", "比之前照规整表改写了产品构建 0 处、Verus 构建 0 处"))
        boundary_kernel = SELFTEST_KERNEL.replace(body, body + " // SELFTEST_MIR_RT_PATH _6 = wrapper::core::panicking::panic_fmt(move _7);").replace(
            "        result <= limit,\n", "        result <= limit, // SELFTEST_MIR_RT_PATH _8 = core::panicking::panic_fmt_with(move _9);\n", 1)
        expect("mir-panic-path-boundary-red", "proof", make_root("mirboundary", kernel=boundary_kernel), 1,
               ("1 个与 Verus 构建的函数体不同（fn kernel::clamp_to_limit）", "比之前照规整表改写了产品构建 0 处、Verus 构建 0 处"))
        # Verus 与产品的 rustc 版本差进工具链钉表（开关 rustc-pin-unchecked 下判错）
        expect("rustc-pin-product", "toolchain", make_root("rustcproduct", rustc_row=f"rustc\tstable\t1.0.1\t{'d' * 40}\n"), 1, "产品的 rustc 是 1.0.0")
        expect("rustc-pin-verus", "toolchain", make_root("rustcverus", rustc_row=f"rustc\tnightly\t1.0.0\t{'d' * 40}\n"), 1, "Verus 用的工具链是 stable")
        expect("rustc-pin-missing", "toolchain", make_root("rustcmissing", rustc_row=""), 1, "没有 rustc 行")
        # cargo 那一种（crate 行第四段 cargo）：经 cargo verus focus 验、路径依赖连同 crate 一起拷，报错位置相对副本根（开关 cargo-mode-ignored 下判错）
        cargo_root = make_root("cargomode", crate_mode="\tcargo", extra_dependency='singlefs-format = { path = "../format" }\n')
        os.makedirs(os.path.join(cargo_root, "crates", "format", "src"))
        open(os.path.join(cargo_root, "crates", "format", "Cargo.toml"), "w").write('[package]\nname = "singlefs-format"\nedition = "2021"\n')
        open(os.path.join(cargo_root, "crates", "format", "src", "lib.rs"), "w").write("pub const LIMIT: u64 = 10;\n")
        cargo_arguments = os.path.join(work, "cargo-verus-arguments.txt")
        open(cargo_arguments, "w").close()
        os.environ["SELFTEST_CARGO_VERUS_ARGUMENTS"] = cargo_arguments
        try:
            expect("cargo-mode-proof", "proof", cargo_root, 0, ("Verus 逐项验 crates/core（cargo verus focus）", "cargo verus build 构建的 MIR 逐条目比"))
            expect("cargo-mode-vacuity", "vacuity", cargo_root, 0, "2 处 requires")
            expect("cargo-mode-must-red", "must-red-worlds", cargo_root, 0, "证明错落点与第六段 landing= 相同")
        finally:
            os.environ.pop("SELFTEST_CARGO_VERUS_ARGUMENTS", None)
        # 几处同时验共用一个编译目录（一处一个 job、两项轮流用它）：每处起之前删被验的 crate 的指纹，第二项照样出判定行（开关 cargo-fingerprint-kept 下第二项红）
        shared_target_exit, shared_target_output = captured_stdout(run_check, "proof", cargo_root, dict(options, jobs=1, verdict_library=fresh_library()))
        expect_true("cargo-mode-shared-build-directory", shared_target_exit == 0 and "proof:core_sample::kernel::keep_small：绿" in shared_target_output.getvalue(),
                    f"一个编译目录轮流验两项期望都判绿，实测退 {shared_target_exit}：{one_line(shared_target_output.getvalue().strip()[-300:])}")
        cargo_runs = [json.loads(line) for line in open(cargo_arguments, encoding="utf-8").read().split("\n") if line]
        expect_true("cargo-mode-command", bool(cargo_runs) and all(run[:6] in (["verus", "focus", "-p", "core-sample", "--offline", "--"],
                                                                                  ["verus", "build", "-p", "core-sample", "--offline", "--"]) for run in cargo_runs)
                    and any(run[1] == "build" for run in cargo_runs) and not any("--no-cheating" in run for run in cargo_runs),
                    f"cargo-verus 期望照 verus focus|build -p core-sample --offline -- … 起、不带 --no-cheating，实测 {cargo_runs[:3]}")
        # 几个 crate（crate 行可以多条）：逐个 crate 判、各比各的登记数，--item 分到它所属的 crate（开关 single-crate-registry 下判错）
        multi_root = make_root("multicrate")
        os.makedirs(os.path.join(multi_root, "crates", "second", "src"))
        open(os.path.join(multi_root, "crates", "second", "Cargo.toml"), "w").write('[package]\nname = "second-sample"\nedition = "2021"\n\n[dependencies]\nvstd = "=0.0.1"\n')
        open(os.path.join(multi_root, "crates", "second", "src", "lib.rs"), "w").write(
            "use vstd::prelude::*;\nverus! {\npub fn double_it(value: u64) -> (result: u64)\n    requires\n        value < 100,\n    ensures\n        result == value + value,\n"
            "{\n    value + value\n}\n} // verus!\n")
        open(os.path.join(multi_root, "crates", "second", "verus-must-red-worlds.tsv"), "w").write(
            "# 自证用\n第二个 crate\tcrates/second/src/lib.rs\t    value + value\t    value + value // SELFTEST_BROKEN\tproof_errors=1\tlanding=double_it\n")
        with open(os.path.join(multi_root, REGISTRY), "a") as handle:
            handle.write("crate\tcrates/second\t1\nkernel\tcrates/second/src/lib.rs\nmust-red-worlds\tcrates/second/verus-must-red-worlds.tsv\n")
        expect("multi-crate-proof", "proof", multi_root, 0, ("proof:second_sample::double_it：绿", "proof:core_sample::kernel::clamp_to_limit：绿", "登记的 2 个 crate 判了 2 个"))
        multi_listed, multi_listing = captured_stdout(list_items, multi_root)
        expect_true("multi-crate-list-items", multi_listed == 0 and "proof:second_sample::double_it\tverus-proof" in multi_listing.getvalue()
                    and "vacuity:second_sample::double_it#1\tverus-vacuity" in multi_listing.getvalue() and "2 个 crate 成项 3 个" in multi_listing.getvalue(),
                    f"期望列出两个 crate 的项：{one_line(multi_listing.getvalue().strip()[-400:])}")
        multi_options = dict(options, items=["proof:second_sample::double_it"], verdict_library=fresh_library())
        multi_item_exit, multi_item_buffer = captured_stdout(run_check, "proof", multi_root, multi_options)
        expect_true("multi-crate-item-routed", multi_item_exit == 0 and "crates/core：--item 没点到这个 crate 的项" in multi_item_buffer.getvalue()
                    and "proof:second_sample::double_it：绿" in multi_item_buffer.getvalue(),
                    f"--item 点第二个 crate 的项期望只判它，实测退 {multi_item_exit}：{one_line(multi_item_buffer.getvalue().strip()[-300:])}")
        # 带值的 const 与 assume_specification 各成一项（core 的 DataUnitIndexInFile::FIRST 与两条格式常量规格的样子）：假 Verus 整 crate 验出
        # 2（exec fn）+ 2（两个带值的 const）+ 1（指向常量、写 returns 的那一条）+ 0（指向 std 方法的那一条）= 5，各项之和要等于它
        # （开关 valued-const-not-item 下少 2 条、assume-specification-not-item 下少 1 条，判红）；指向 std 方法的那一项验出 0 条照判绿
        const_kernel = selftest_kernel_with(SELFTEST_CONST_EXTRA)
        _harvest_exit, harvested = captured_stdout(run_check, "proof", make_root("constharvest", kernel=const_kernel, count=5), dict(options, verdict_library=fresh_library()))
        const_digests = dict(re.findall(r"的 assume_specification (\S+) 没登记进信任清单 \S+（这一处的规格原文 sha256 ([0-9a-f]{64})）", harvested.getvalue()))
        const_rows = [f"assume_specification\tcrates/core/src/kernel.rs\t{point}\t{const_digests.get(point, '0' * 64)}\tselftest_differential\t自证用：登记的可信声明"
                      for point in ("selftest_format::SLOT_WIDTH", "<[T]>::binary_search")]
        expect("proof-valued-const-and-assume-specification-items", "proof", make_root("constitems", kernel=const_kernel, count=5, trust_rows=const_rows), 0,
               ("成项 6 个（exec fn 2、带值的 const 2、assume_specification 2），各项 verified 之和 5、0 errors（登记 5）",
                "proof:core_sample::kernel::Width::NARROWEST：绿", "proof:core_sample::kernel::WIDEST_SAMPLE：绿",
                "proof:core_sample::kernel::assume_specification[selftest_format::SLOT_WIDTH]：绿", "proof:core_sample::kernel::assume_specification[<[T]>::binary_search]：绿"))
        # 关联 const 与自由 const 名字一前一后互为前缀（core 的 JournalRecordOrdinalWithinPublish::FIRST 与 FIRST_JOURNAL_COUNTER 同在 journal 的样子）：
        # 关联 const 点 *::FIRST、自由 const 点 FIRST_COUNTER，各恰好命中一项，列项退 0、逐项验各 1 条、之和 6 等于登记（开关 associated-const-bare-name 下
        # 关联 const 点 FIRST，表里 FIRST_COUNTER 与 impl&%?::FIRST 都包含它，列项与证明都判红）
        prefix_root = make_root("constprefix", kernel=selftest_kernel_with(SELFTEST_CONST_PREFIX_EXTRA), count=6)
        prefix_listed, prefix_listing = captured_stdout(list_items, prefix_root)
        expect_true("list-items-associated-const-prefix", prefix_listed == 0
                    and all(f"proof:core_sample::kernel::{name}\tverus-proof\t带值的 const " in prefix_listing.getvalue()
                            for name in ("Ordinal::FIRST", "FIRST_COUNTER", "LAST", "Ordinal::LAST_SLOT")),
                    f"期望列项退 0、四个 const 各成一项，实测退 {prefix_listed}：{one_line(prefix_listing.getvalue().strip()[-400:])}")
        prefix_arguments = os.path.join(work, "constprefix-arguments.jsonl")
        os.environ["SELFTEST_VERUS_ARGUMENTS"] = prefix_arguments
        try:
            expect("proof-associated-const-prefix-green", "proof", prefix_root, 0,
                   ("成项 6 个（exec fn 2、带值的 const 4），各项 verified 之和 6、0 errors（登记 6）",
                    "proof:core_sample::kernel::Ordinal::FIRST：绿", "proof:core_sample::kernel::FIRST_COUNTER：绿"))
        finally:
            os.environ.pop("SELFTEST_VERUS_ARGUMENTS", None)
        prefix_points = [arguments[arguments.index("--verify-function") + 1] for arguments in
                         (json.loads(line) for line in (open(prefix_arguments).read().split("\n") if os.path.isfile(prefix_arguments) else []) if line)
                         if "--verify-function" in arguments]
        expect_true("proof-associated-const-point-names", sorted(point for point in prefix_points if "FIRST" in point or "LAST" in point)
                    == ["*::FIRST", "*::LAST_SLOT", "FIRST_COUNTER", "LAST"],
                    f"期望关联 const 点 *::<名>、自由 const 点 <名>，实测交给 --verify-function 的 {prefix_points}")
        # 撞名当场判红并点名撞的几项（起 Verus 之前）：trait 里带默认值的 Limited::LIMIT 与 impl 里覆盖它的 LIMIT，*::LIMIT 两个都以 ::LIMIT 收尾
        # （开关 point-names-by-containment 下改前的判法只比成项的名字谁是谁的一段，放过它）；同一个模块两个 impl 各有一个 FIRST
        expect("proof-point-name-trait-default-collision-red", "proof",
               make_root("consttraitdefault", kernel=selftest_kernel_with(SELFTEST_CONST_TRAIT_DEFAULT_EXTRA), count=4), 1,
               ("proof:core_sample::kernel::Ordinal::LIMIT 交给 --verify-function 的 *::LIMIT 在这个模块的 Verus 可点名函数表里命中 2 项",
                "const Limited::LIMIT（crates/core/src/kernel.rs:", "const Ordinal::LIMIT（crates/core/src/kernel.rs:"))
        expect("proof-point-name-two-impls-collision-red", "proof",
               make_root("consttwoimpls", kernel=selftest_kernel_with(SELFTEST_CONST_PREFIX_EXTRA + SELFTEST_CONST_SECOND_IMPL_EXTRA), count=7), 1,
               ("交给 --verify-function 的 *::FIRST 在这个模块的 Verus 可点名函数表里命中 2 项",
                "const Ordinal::FIRST（crates/core/src/kernel.rs:", "const OtherOrdinal::FIRST（crates/core/src/kernel.rs:"))
        # 空真插入点的说明认 const fn（core 的 SlotNumber::to_device_offset）：列项里 const fn 那一处写「fn <它自己>」（开关 const-fn-heading-unrecognized 下写成上一个 fn）
        const_fn_listed, const_fn_listing = captured_stdout(list_items, make_root("constfn", kernel=selftest_kernel_with(SELFTEST_CONST_FN_EXTRA), count=3))
        expect_true("vacuity-const-fn-label", const_fn_listed == 0
                    and "vacuity:core_sample::kernel::to_slot_offset#1\tverus-vacuity\tcrates/core/src/kernel.rs fn to_slot_offset\n" in const_fn_listing.getvalue(),
                    f"期望 const fn 那一处的说明是 fn to_slot_offset，实测退 {const_fn_listed}：{one_line(const_fn_listing.getvalue().strip()[-400:])}")
        # 属于 assert(…) by (…) 的 requires 不是空真插入点（core 的 by (nonlinear_arith) 接 requires … ;）：列项退 0、halve_within 只有入口那一处，
        # 列项的 proof 那一行与空真、证明两格的成功句报「跳过 assert-by requires 1 处」并点名那一行（开关 assert-by-requires-as-insertion-point 下三格判错）；
        # requires 挂在普通块与不带 by 的 assert 上、下面一行只有 ; 的照旧判红、红句带文件名（开关 assert-by-requires-by-semicolon-line 下这两格判错）
        assert_by_kernel = selftest_kernel_with(SELFTEST_ASSERT_BY_EXTRA)
        assert_by_rows = assert_by_kernel.split("\n")
        assert_by_line = next((number for number, text in enumerate(assert_by_rows, 1) if text.strip() == "requires" and "by (nonlinear_arith)" in assert_by_rows[number - 2]), None)
        assert_by_skipped = f"跳过 assert-by requires 1 处（属于 assert(…) by (…)，在那一处先证出来，不是外层函数的前提）：crates/core/src/kernel.rs:{assert_by_line}"
        assert_by_root = make_root("assertby", kernel=assert_by_kernel, count=3)
        assert_by_listed, assert_by_listing = captured_stdout(list_items, assert_by_root)
        expect_true("list-items-assert-by-requires-skipped", assert_by_listed == 0 and f"；空真{assert_by_skipped}\n" in assert_by_listing.getvalue()
                    and [row.split("\t")[0] for row in assert_by_listing.getvalue().split("\n") if row.startswith("vacuity:core_sample::kernel::halve_within#")]
                    == ["vacuity:core_sample::kernel::halve_within#1"],
                    f"期望列项退 0、proof 那一行报「{assert_by_skipped}」、halve_within 只有 #1，实测退 {assert_by_listed}：{one_line(assert_by_listing.getvalue().strip()[-500:])}")
        expect("vacuity-assert-by-requires-skipped", "vacuity", assert_by_root, 0, ("1 份核里 3 处 requires，", assert_by_skipped))
        expect("proof-assert-by-requires-skip-reported", "proof", assert_by_root, 0, ("成项 3 个（exec fn 3）", f"空真那一格{assert_by_skipped}"))
        for stray_label, stray_extra in (("list-items-requires-on-plain-block-red", SELFTEST_PLAIN_BLOCK_REQUIRES_EXTRA),
                                         ("list-items-requires-on-assert-without-by-red", SELFTEST_ASSERT_WITHOUT_BY_EXTRA)):
            stray_kernel = selftest_kernel_with(stray_extra)
            stray_line = max(number for number, text in enumerate(stray_kernel.split("\n"), 1) if text.strip() == "requires")
            stray_exit, stray_listing = captured_stdout(list_items, make_root(stray_label, kernel=stray_kernel, count=3))
            expect_true(stray_label, stray_exit == 1 and f"crates/core/src/kernel.rs 第 {stray_line} 行的 requires 找不到函数体的 {{" in stray_listing.getvalue(),
                        f"期望列项退 1、红句点名 crates/core/src/kernel.rs 第 {stray_line} 行，实测退 {stray_exit}：{one_line(stray_listing.getvalue().strip()[-400:])}")
        # spec const 与别的带值的 const 一样成项（真 Verus 整 crate 验时各算 1 条）：impl 里的点 *::SPEC_WIDTH、自由的点 SPEC_HEIGHT，之和 4 等于登记
        # （开关 spec-const-not-item 下两个 spec const 不成项、之和 2 与登记的 4 不同）
        expect("proof-spec-const-items", "proof", make_root("specconst", kernel=selftest_kernel_with(SELFTEST_SPEC_CONST_EXTRA), count=4), 0,
               ("成项 4 个（exec fn 2、带值的 const 2），各项 verified 之和 4、0 errors（登记 4）",
                "proof:core_sample::kernel::Window::SPEC_WIDTH：绿", "proof:core_sample::kernel::SPEC_HEIGHT：绿"))
        # 几处同时验不按内存收（内存额度归 task-server）：外层内存上限 1G、--threads 8、两处同时验，每处起的 Verus 都带 --num-threads 4
        # （开关 outer-memory-narrows-jobs 下收到 1 处、每处 8 条，判错）
        jobs_arguments = os.path.join(work, "jobs-verus-arguments.jsonl")
        saved_outer = os.environ.get(OUTER_MEMORY_CAP_VARIABLE)
        os.environ.update({OUTER_MEMORY_CAP_VARIABLE: "1G", "SELFTEST_VERUS_ARGUMENTS": jobs_arguments})
        try:
            jobs_registry, _jobs_kernels, jobs_manifest, jobs_executable = prepared(green, options)
            jobs_outcomes = verify_in_parallel(green, jobs_registry, jobs_manifest, jobs_executable, dict(options, jobs=0, threads=8),
                                               [("整 crate 甲", {}), ("整 crate 乙", {})])
        finally:
            os.environ.pop("SELFTEST_VERUS_ARGUMENTS", None)
            if saved_outer is None:
                os.environ.pop(OUTER_MEMORY_CAP_VARIABLE, None)
            else:
                os.environ[OUTER_MEMORY_CAP_VARIABLE] = saved_outer
        jobs_threads = [arguments[arguments.index("--num-threads") + 1] if "--num-threads" in arguments else None
                        for arguments in (json.loads(line) for line in open(jobs_arguments, encoding="utf-8").read().split("\n") if line.strip())] \
            if os.path.isfile(jobs_arguments) else []
        expect_true("jobs-not-narrowed-by-outer-memory", len(jobs_outcomes) == 2 and jobs_threads == ["4", "4"],
                    f"外层上限 1G、--threads 8、两处：期望起两次 Verus、各带 --num-threads 4，实测收回 {len(jobs_outcomes)} 处、--num-threads {jobs_threads}")
        # Verus 退了先停它的进程组里留下的：直接起的 bash 在后台起一个拿着 stdout 与 stderr 的 sleep 20 就退，判法几秒内交回、sleep 也不在了
        # （开关 group-leftovers-kept 下不停，判法等到 sleep 自己退才交回，判错；留下的 sleep 按记下的进程号停掉）
        child_file = os.path.join(work, "leftover-child-pid")
        group_started = time.monotonic()
        group_process = subprocess.Popen(["bash", "-c", 'sleep 20 & echo "$!" > "$1"; echo left-a-child', "leftover-group", child_file], cwd=work,
                                         stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, errors="replace", start_new_session=True)
        group_stdout, _group_stderr = finish_process_group(group_process)
        group_seconds = time.monotonic() - group_started
        child_pid = int(open(child_file, encoding="utf-8").read().strip() or 0) if os.path.isfile(child_file) else 0
        child_alive = False
        for _attempt in range(20):
            try:
                with open(f"/proc/{child_pid}/stat", encoding="utf-8") as handle:
                    child_alive = child_pid > 0 and handle.read().rsplit(")", 1)[1].split()[0] != "Z"
            except (OSError, IndexError):
                child_alive = False
            if not child_alive:
                break
            time.sleep(0.1)
        if child_alive:
            os.kill(child_pid, signal.SIGKILL)   # 这一格自己起的那一个 sleep，按记下的进程号停
        expect_true("group-leftovers-stopped", group_process.returncode == 0 and "left-a-child" in group_stdout
                    and child_pid > 0 and not child_alive and group_seconds < 10,
                    f"期望 bash 退 0、10 秒内交回、后台的 sleep（进程号 {child_pid}）也停了，"
                    f"实测退 {group_process.returncode}、用了 {group_seconds:.1f} 秒、sleep 还在 {child_alive}")
        # 每个 Verus 进程的内存上限没有了：--memory-cap 当认不出的选项，退 2（用法错）
        try:
            parse_options([green, "--memory-cap", "x"])
            memory_option = "收下了"
        except ValueError as error:
            memory_option = str(error)
        expect_true("memory-cap-option-gone", memory_option == "认不出的选项 --memory-cap", f"期望「认不出的选项 --memory-cap」，实测：{memory_option}")
        expect("must-red-survives", "must-red-worlds", make_root("survives", must_red=["改注释\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\tif value > limit { limit } else { value } // 只是注释\tproof_errors=1\tlanding=clamp_to_limit"]), 1, "照样验过")
        expect("must-red-anchor", "must-red-worlds", make_root("anchor", must_red=["不在的原文\tcrates/core/src/kernel.rs\tnot present anywhere\tx\tproof_errors=1\tlanding=clamp_to_limit"]), 1, "恰好命中一次")
        # 必红世界按落点判、条数只报（C654）：证明错条数与第五段 proof_errors= 不同而落点没变，判绿并在成功句里数进「只报不判」（开关 landing-by-count 下判错）；
        # 同一个 fn 里多报两条（条数 3，登记 1）也是；落点换了函数而条数没变判红（开关 landing-unchecked、landing-by-count 下判错）
        broken_line = "if value > limit { limit } else { value } // SELFTEST_BROKEN"
        clamp_row = "\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\t"
        expect("must-red-proof-error-count-drift", "must-red-worlds",
               make_root("proofcount", must_red=["条数不对" + clamp_row + broken_line + "\tproof_errors=2\tlanding=clamp_to_limit"]),
               0, ("证明错落点与第六段 landing= 相同", "1 行证明错条数与第五段 proof_errors= 不同，只报不判（条数不对）"))
        expect("must-red-landing-same-function-more-errors", "must-red-worlds",
               make_root("moreerrors", must_red=["多报两条" + clamp_row + broken_line + "\\n    // SELFTEST_BROKEN\\n    // SELFTEST_BROKEN\tproof_errors=1\tlanding=clamp_to_limit"]),
               0, ("证明错落点与第六段 landing= 相同", "1 行证明错条数与第五段 proof_errors= 不同，只报不判（多报两条）"))
        expect("must-red-landing-moved", "must-red-worlds",
               make_root("landingmoved", must_red=["落到别的函数" + "\tcrates/core/src/kernel.rs\tif value > 10 { pick(10) } else { pick(value) }\t"
                                                   "if value > 10 { pick(10) } else { pick(value) } // SELFTEST_BROKEN\tproof_errors=1\tlanding=clamp_to_limit"]),
               1, "证明错落在 keep_small，表里写的落点是 clamp_to_limit")
        # 落点按改坏之后的文本找（替换文比原文多几行，后面的行都挪了位置）：改坏的是 clamp_to_limit，原来就带标记的 tiny 在它后面，落点是 tiny；
        # 读改坏之前的文本找就落到后面的 keep_small（开关 landing-read-from-unmutated-text 下判错）
        shifted = "if value > limit { limit } else { value }" + "\\n" * 9
        expect("must-red-landing-line-shift", "must-red-worlds",
               make_root("landingshift", kernel=SELFTEST_LANDING_KERNEL, must_red=["往后挪九行" + clamp_row + shifted + "\tproof_errors=1\tlanding=tiny"]),
               0, "证明错落点与第六段 landing= 相同")
        # 两个函数各报一条：落点是两个名字的集合，登记的顺序随意
        expect("must-red-landing-two-functions", "must-red-worlds",
               make_root("twofunctions", kernel=SELFTEST_LANDING_KERNEL, must_red=["两处" + clamp_row + broken_line + "\tproof_errors=2\tlanding=tiny,clamp_to_limit"]),
               0, "证明错落点与第六段 landing= 相同")
        expect("must-red-landing-two-functions-one-registered", "must-red-worlds",
               make_root("twoonereg", kernel=SELFTEST_LANDING_KERNEL, must_red=["两处只登记一处" + clamp_row + broken_line + "\tproof_errors=2\tlanding=clamp_to_limit"]),
               1, "证明错落在 clamp_to_limit、tiny，表里写的落点是 clamp_to_limit")
        # 证明错的位置不在任何 fn 里（核的第一行）：落点记（函数外）
        expect("must-red-landing-outside-function", "must-red-worlds",
               make_root("outsidefunction", must_red=["函数外\tcrates/core/src/kernel.rs\tuse vstd::prelude::*;\tuse vstd::prelude::*; // SELFTEST_BROKEN\tproof_errors=1\tlanding=（函数外）"]),
               0, "证明错落点与第六段 landing= 相同")
        # 不是六段的行：判红，报行号、要六段（开关 row-segments-unchecked 下判错）；第六段不是 landing=<落点>：判红（开关 landing-form-unchecked 下判错）
        expect("must-red-row-segments", "must-red-worlds", make_root("rowsegments", must_red=["少一段" + clamp_row + broken_line + "\tproof_errors=1"]),
               1, ("第 2 行不是六段", "要六段制表符分隔"))
        expect("must-red-landing-form", "must-red-worlds", make_root("landingform", must_red=["落点写法不对" + clamp_row + broken_line + "\tproof_errors=1\tclamp_to_limit"]),
               1, ("的第六段「clamp_to_limit」不是 landing=<落点>", "必红世界表第六段一律写 landing="))
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
                                                                                        "if value > limit { limit } else { value } // SELFTEST_NEEDS_RLIMIT_99\tproof_errors=1\tlanding=clamp_to_limit"]),
               1, "这一行红在算力额度上，不是证明失败")
        # 输出里夹着 rlimit 用尽、另有一条真的证明错：红计入（只剩 rlimit 用尽的才不作数）
        # rlimit 用尽不计入证明错条数：证明错 1 条对得上登记的 proof_errors=1，成功句里没有「条数不同、只报不判」那一段
        # （开关 rlimit-counted-in-proof-errors 下条数成 2，成功句多出那一段：下面的 no-count-drift 一格判错）
        count_drift_text = "证明错条数与第五段 proof_errors= 不同"
        with_proof_error_output = expect("must-red-rlimit-with-proof-error", "must-red-worlds", make_root("mustredrlimitproof", must_red=[
            "撞 rlimit 又有证明错\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\t"
            "if value > limit { limit } else { value } // SELFTEST_NEEDS_RLIMIT_99 SELFTEST_BROKEN\tproof_errors=1\tlanding=clamp_to_limit"]),
               0, "1 行输出里夹着 rlimit 用尽、另有证明错，红计入")
        expect_absent("must-red-rlimit-with-proof-error-no-count-drift", with_proof_error_output, count_drift_text)
        # 代码轮第三轮判决 J3 那四行的情形：同一行在 rlimit 10 上输出一条 rlimit 用尽加一条证明错（Verus 把 rlimit 用尽也计进 errors，errors 2），
        # 在 rlimit 20 上只输出同一条证明错（errors 1）；第五段 proof_errors=1 只数证明错，两份都判绿，成功句里都没有「条数不同」那一段
        # （开关 rlimit-counted-in-proof-errors 下 rlimit 10 那份条数成 2、成功句多出那一段：no-count-drift 一格判错）
        j3_row = ("rlimit 用尽另记\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\t"
                  "if value > limit { limit } else { value } // SELFTEST_NEEDS_RLIMIT_15\\n    // SELFTEST_BROKEN\t")
        rlimit_10_output = expect("must-red-rlimit-10-counts-proof-errors-only", "must-red-worlds",
                                  make_root("j3rlimit10", must_red=[j3_row + "proof_errors=1\tlanding=clamp_to_limit"], rlimit=10),
                                  0, ("1 行输出里夹着 rlimit 用尽、另有证明错，红计入", "证明错落点与第六段 landing= 相同"))
        expect_absent("must-red-rlimit-10-no-count-drift", rlimit_10_output, count_drift_text)
        expect("must-red-rlimit-20-counts-proof-errors-only", "must-red-worlds", make_root("j3rlimit20", must_red=[j3_row + "proof_errors=1\tlanding=clamp_to_limit"], rlimit=20),
               0, ("0 行输出里夹着 rlimit 用尽", "证明错落点与第六段 landing= 相同"))
        # 第五段不是 proof_errors=<正整数>（写成 0）的行判红、出路写约定（开关 expectation-form-unchecked 下判错）
        expect("must-red-expectation-form", "must-red-worlds", make_root("expectationform", must_red=[j3_row + "proof_errors=0\tlanding=clamp_to_limit"], rlimit=10),
               1, ("的第五段「proof_errors=0」不是 proof_errors=<证明错条数>", "必红世界表第五段一律写 proof_errors=M"))
        # 函数上的 rlimit 属性盖过命令行的 --rlimit：核里有它，proof 判红；必红世界的替换文里有它，那一行判红
        expect("proof-rlimit-attribute", "proof", make_root("rlimitattribute", kernel=SELFTEST_KERNEL.replace(
            "pub fn clamp_to_limit(", "#[verifier::rlimit(20)]\npub fn clamp_to_limit(")), 1, "verifier::rlimit（函数上的 rlimit 属性盖过命令行的 --rlimit 7")
        expect("must-red-rlimit-attribute", "must-red-worlds", make_root("mustredrlimitattribute", must_red=[
            "加 rlimit 属性\tcrates/core/src/kernel.rs\tpub fn clamp_to_limit(\t#[verifier::rlimit(20)] // SELFTEST_BROKEN\\npub fn clamp_to_limit(\tproof_errors=1\tlanding=clamp_to_limit"]),
               1, "的替换文里有 verifier::rlimit")
        # JSON 里缺整数的 verified 与 errors（vir 错没走到验证、或 Verus 换了 JSON 的写法）：那一行判红
        expect("must-red-counts-missing", "must-red-worlds", make_root("mustrednocounts", must_red=[
            "JSON 缺数\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\t"
            "if value > limit { limit } else { value } // SELFTEST_BROKEN SELFTEST_NO_COUNTS\tproof_errors=1\tlanding=clamp_to_limit"]),
               1, "Verus 的 JSON 里没有整数的 verified 与 errors")
        # ── 逐项点名与按条复用（文件头「逐项点名与按条复用」）──
        import io
        import contextlib
        item_line = re.compile(r"^  · (\S.*?)：(绿|红)，(强制现判|现判|复用)", re.M)

        def run_items(check, root, environment=None, **changes):
            """跑一格交回 (退出码, 输出, {项: (绿/红, 现判/复用)})；changes 盖进 options（items、verdict_library、judging_units、reuse），environment 临时设进 os.environ。
            changes 里没给 verdict_library 的给一份新的空库（fresh_library）：判定库没有不读不写的形态。"""
            changes = dict({"verdict_library": fresh_library()}, **changes)
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
                print(one_line(f"  ✗ {label} 这一格判错：{detail}"))  # gate-lint:detail

        # ── 复用的几处修补（文件头「逐项点名与按条复用」：整格不丢、键的组成、记红撤绿、库位置）──
        # 整格不丢：proof 与 proof:<名> 一起点名照整格判（开关 proof-item-narrows 下只验点名的那一个、不比之和，这一格判错）
        core_root = make_root("proof-whole-kept")
        got, output, verdicts = run_items("proof", core_root, items=["proof", "proof:core_sample::kernel::clamp_to_limit"])
        judge_cell("proof-whole-kept", got == 0 and len(verdicts) == 2 and "各项 verified 之和 2" in output and "不比之和" not in output,
                   f"proof 与 proof:core_sample::kernel::clamp_to_limit 一起点名期望照整格判两项、比之和（退 0），实测退 {got}、判了 {list(verdicts)}：{output.strip()[-300:]}")

        # 键的组成：crate 起法（edition、包名、cargo 还是直接起）、闭包真点名的外部 crate 在 Cargo.lock 里的版本进键；RUSTUP_HOME、PATH 里 rustup 的位置不进键
        def item_key(root, entry):
            registry = read_registry(root)
            cell_options = dict(options, verdict_library=os.path.join(work, "key-only-library"), shared_components=None, unit_index=None)
            components = shared_components(root, registry, cell_options) + function_components(root, registry, cell_options, [entry], "被验函数单元闭包")
            return key_of(components, "自证的判法单元"), components

        clamp_entry, vstd_entry = "crates/core/src/kernel.rs::clamp_to_limit", "crates/core/src/kernel.rs::through_vstd"
        build_roots = {}
        for name, mode in (("key-edition-2021", ""), ("key-edition-2024", ""), ("key-cargo-mode", "\tcargo")):
            build_roots[name] = make_root(name, crate_mode=mode)
        manifest_path = os.path.join(build_roots["key-edition-2024"], "crates", "core", "Cargo.toml")
        with open(manifest_path, encoding="utf-8") as handle:
            manifest_text = handle.read()
        with open(manifest_path, "w", encoding="utf-8") as handle:
            handle.write(manifest_text.replace('edition = "2021"', 'edition = "2024"'))
        build_keys = {name: item_key(root, clamp_entry)[0] for name, root in build_roots.items()}
        judge_cell("reuse-key-crate-build", len(set(build_keys.values())) == 3,
                   f"edition 2021、edition 2024、cargo 起法三种期望三把不同的键，实测 {build_keys}")
        vstd_kernel = selftest_kernel_with("\npub fn through_vstd(value: u64) -> (result: u64)\n    ensures\n        result == value,\n{\n    vstd::prelude::identity_for_selftest(value)\n}\n")
        lock_text = 'version = 4\n\n[[package]]\nname = "core-sample"\nversion = "0.1.0"\ndependencies = [\n "vstd",\n]\n\n[[package]]\nname = "vstd"\nversion = "{0}"\nsource = "registry+https://example.invalid/selftest"\n'
        dependency_keys = {}
        for version in ("0.0.1", "0.0.2"):
            dependency_root = make_root(f"key-lock-vstd-{version}", kernel=vstd_kernel)
            with open(os.path.join(dependency_root, "Cargo.lock"), "w", encoding="utf-8") as handle:
                handle.write(lock_text.format(version))
            dependency_keys[version] = (item_key(dependency_root, vstd_entry)[0], item_key(dependency_root, clamp_entry)[0])
        judge_cell("reuse-key-closure-dependencies", dependency_keys["0.0.1"][0] != dependency_keys["0.0.2"][0],
                   f"Cargo.lock 里 vstd 0.0.1 → 0.0.2：点名 vstd:: 的 through_vstd 期望键变（另报闭包不点名它的 clamp_to_limit 变没变），实测 {dependency_keys}")
        saved_rustup_home, saved_search_path = os.environ.get("RUSTUP_HOME"), os.environ.get("PATH", "")
        toolchain_digests, printed_lines = [], []
        try:
            for variant in ("a", "b"):
                home = os.path.join(work, f"key-rustup-home-{variant}")
                library = os.path.join(home, "toolchains", "stable", "lib")
                os.makedirs(library, exist_ok=True)
                with open(os.path.join(library, "librustc_driver-selftest.so"), "wb") as handle:
                    handle.write(b"selftest driver, same content in both homes")
                program_directory = os.path.join(work, f"key-path-rustup-{variant}")
                os.makedirs(program_directory, exist_ok=True)
                with open(os.path.join(program_directory, "rustup"), "w", encoding="utf-8") as handle:
                    handle.write("#!/bin/sh\n# 自证用：两份内容相同、位置不同的 rustup\n")
                os.chmod(os.path.join(program_directory, "rustup"), 0o755)
                os.environ["RUSTUP_HOME"], os.environ["PATH"] = home, program_directory + os.pathsep + saved_search_path
                registry = read_registry(green)
                cell_options = dict(options, verdict_library=os.path.join(work, "key-only-library"), shared_components=None)
                toolchain_digests.append(dict(shared_components(green, registry, cell_options)).get("工具链"))
                printed_lines.append(tool_versions_text(green, install))
        finally:
            os.environ["PATH"] = saved_search_path
            if saved_rustup_home is None:
                os.environ.pop("RUSTUP_HOME", None)
            else:
                os.environ["RUSTUP_HOME"] = saved_rustup_home
        judge_cell("reuse-key-toolchain-without-paths", toolchain_digests[0] == toolchain_digests[1] and printed_lines[0] != printed_lines[1],
                   f"同内容的工具链与 rustup 换个位置：键里工具链那一样期望不变、tool-versions 打的行期望照旧不同，实测键 {toolchain_digests}，打的行相同 {printed_lines[0] == printed_lines[1]}")

        # 记红撤绿：先判绿入库；同一把键（只在环境里点名一处证明错，不改源码）走复用哨兵强制现判那一路判一趟红（哨兵 1 条、种子 5：keep_small 当哨兵，clamp_to_limit 被抽成强制现判）；
        # 下一趟那一项现判，别的复用（开关 red-keeps-green 下照复用，这一格判错）
        withdraw_root = make_root("red-withdraws-green")
        withdraw_store = os.path.join(work, "verdict-library-red-withdraws-green")
        first_got, _output, _verdicts = run_items("proof", withdraw_root, verdict_library=withdraw_store)
        red_got, _output, red_verdicts = run_items("proof", withdraw_root, {"SELFTEST_FAIL_MARK": "if value > limit { limit }", reuse_checks.SENTINEL_COUNT_VARIABLE: "1",
                                                                           reuse_checks.SENTINEL_SEED_VARIABLE: "5"},
                                                   verdict_library=withdraw_store)
        got, output, verdicts = run_items("proof", withdraw_root, verdict_library=withdraw_store)
        judge_cell("reuse-red-withdraws-green", first_got == 0 and red_got == 1 and red_verdicts.get("proof:core_sample::kernel::clamp_to_limit", ("", ""))[0] == "红"
                   and verdicts.get("proof:core_sample::kernel::clamp_to_limit", ("", ""))[1] == "现判" and verdicts.get("proof:core_sample::kernel::keep_small", ("", ""))[1] == "复用" and got == 0,
                   f"头一趟退 {first_got}、判红那一趟退 {red_got}（{red_verdicts}），下一趟期望 clamp_to_limit 现判、keep_small 复用，实测退 {got}、{verdicts}")

        # 库位置只由被判那棵仓定（文件头「库放在哪」）：git 工作树的 library-directory 打 <common-dir>/singlefs-formal-unit-verdicts；不是 git 工作树的退 2、
        # proof 一项都不判（共用库 library_directory 退 2，它的弄坏开关 library-directory-outside-git-tolerated 由共用库自证证红）
        library_main = os.path.join(work, "library-main")
        os.makedirs(library_main)
        subprocess.run(["git", "init", "-q", library_main], check=True)
        library_copy = make_root("library-copy")
        located = subprocess.run([sys.executable, os.path.realpath(__file__), "library-directory", library_main], capture_output=True, text=True, stdin=subprocess.DEVNULL)
        outside = subprocess.run([sys.executable, os.path.realpath(__file__), "proof", library_copy], capture_output=True, text=True, stdin=subprocess.DEVNULL)
        expected_library = os.path.join(library_main, ".git", FORMAL_LIBRARY_DIRECTORY_NAME)
        judge_cell("library-directory-from-judged-tree", located.returncode == EXIT_GREEN and os.path.realpath(located.stdout.strip()) == os.path.realpath(expected_library)
                   and outside.returncode == EXIT_USAGE and "不是 git 工作树" in outside.stderr,
                   f"git 工作树期望打 {expected_library}，实测退 {located.returncode}「{located.stdout.strip()}」；不是 git 工作树的 proof 期望退 {EXIT_USAGE}、"
                   f"stderr 点名，实测退 {outside.returncode}：{(outside.stdout + outside.stderr)[-300:]}")

        green_row = "改坏{0}\tcrates/core/src/kernel.rs\tif value > limit {{ limit }} else {{ value }}\tif value > limit {{ limit }} else {{ value }} // SELFTEST_BROKEN {0}\tproof_errors=1\tlanding=clamp_to_limit"
        survives_row = "改注释{0}\tcrates/core/src/kernel.rs\tif value > limit {{ limit }} else {{ value }}\tif value > limit {{ limit }} else {{ value }} // 只是注释 {0}\tproof_errors=1\tlanding=clamp_to_limit"
        # 三行必红世界改的是同一处原文：原文要恰好命中一次，三行各自改一份副本
        mixed_root = make_root("itemsmixed", must_red=[green_row.format("A"), survives_row.format("B"), green_row.format("C")])
        vacuity_lines = [requires_line for requires_line, _brace, _label in requires_insertion_points(SELFTEST_KERNEL)]
        listed_items = ["toolchain", "proof", "proof:core_sample::kernel::clamp_to_limit", "proof:core_sample::kernel::keep_small", "vacuity:core_sample::kernel::clamp_to_limit#1", "vacuity:core_sample::kernel::keep_small#1",
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
        got, output, verdicts = run_items("proof", mixed_root, items=["proof:core_sample::kernel::clamp_to_limit"])
        judge_cell("item-proof-one", got == 0 and list(verdicts) == ["proof:core_sample::kernel::clamp_to_limit"] and "不比之和" in output,
                   f"--item proof:core_sample::kernel::clamp_to_limit 期望只验这一个函数、不比之和（退 0），实测退 {got}、判了 {list(verdicts)}：{output.strip()[-200:]}")
        got, output, _verdicts = run_items("proof", mixed_root, items=["proof:没有这个函数"])
        judge_cell("item-proof-unknown", got == EXIT_USAGE and "proof:core_sample::kernel::keep_small" in output, f"点名没有的函数期望退 2、列出可点名的项，实测退 {got}")
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
        # 改一个函数只重验那一项：clamp_to_limit 的 ensures 改一个字，证明与空真只现判它那一项、keep_small 的复用；必红世界三行改的都在它里面，全现判
        # （开关 reuse-key-without-units 下全复用、reuse-key-whole-crate 下全现判，这一格都判错）
        edit_kernel(kernel_path, "        result <= limit,\n", "        result <= limit + 0,\n")
        after = {check: run_items(check, reuse_root, verdict_library=store_directory) for check in ("proof", "vacuity", "must-red-worlds")}
        judge_cell("reuse-proof-function-only", all(got == 0 for got, _output, _verdicts in after.values())
                   and after["proof"][2].get("proof:core_sample::kernel::clamp_to_limit", ("", ""))[1] == "现判" and after["proof"][2].get("proof:core_sample::kernel::keep_small", ("", ""))[1] == "复用"
                   and after["vacuity"][2].get("vacuity:core_sample::kernel::clamp_to_limit#1", ("", ""))[1] == "现判" and after["vacuity"][2].get("vacuity:core_sample::kernel::keep_small#1", ("", ""))[1] == "复用"
                   and "现判 3、复用 0" in after["must-red-worlds"][1] and "改了 fn clamp_to_limit" in after["proof"][1],
                   "期望只现判 clamp_to_limit 那几项、点名它变了，实测 " + "；".join(f"{check} 退 {got}：{output.strip()[-300:]}" for check, (got, output, _verdicts) in after.items()))
        with open(os.path.join(install, "libextra.so"), "w") as handle:
            handle.write("安装目录里多一份库\n")
        try:
            got, output, _verdicts = run_items("must-red-worlds", reuse_root, verdict_library=store_directory)
        finally:
            os.remove(os.path.join(install, "libextra.so"))
        judge_cell("reuse-toolchain-changed", got == 0 and "现判 3、复用 0（现判的原因：工具链 变了 3）" in output, f"退 {got}：{output.strip()[-300:]}")
        # 信任清单进键：这个 crate 的一行换一条差分测试，证明两项都现判、原因写「信任清单 变了」（开关 reuse-key-without-trust-list 下照复用，判错）
        trust_store = os.path.join(work, "verdict-library-trust")
        # 用登记了 cfg 原子的核：样本根里指不到工作区条目的 assume_specification 拒算（那一项照判、不复用），比不出复用
        trust_reuse_root = make_root("reusetrust", kernel=cfg_kernel, trust_rows=[cfg_row])
        first_got, first_output, _verdicts = run_items("proof", trust_reuse_root, verdict_library=trust_store)
        trust_path = os.path.join(trust_reuse_root, TRUST_LIST)
        trust_text = open(trust_path).read()
        open(trust_path, "w").write(trust_text.replace("\tselftest_differential\t", "\tselftest_differential_other\t", 1))
        got, output, _verdicts = run_items("proof", trust_reuse_root, verdict_library=trust_store)
        judge_cell("reuse-trust-list-changed", first_got == 0 and got == 0 and "现判 2、复用 0（现判的原因：信任清单 变了 2）" in output,
                   f"头一趟退 {first_got}，改了信任清单一行之后期望两项都现判，实测退 {got}：{output.strip()[-300:]}")
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
                   == {"proof:core_sample::kernel::clamp_to_limit": "复用", "proof:core_sample::kernel::keep_small": "现判", "proof:core_sample::kernel::keep_tiny": "现判"}
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
        for label, extra in (("按函数", ["--verify-only-module", "kernel", "--verify-function", "count_up"]), ("整 crate", [])):
            ran = subprocess.run([os.path.join(install, "verus"), "--no-cheating", "--rlimit", "7", "--output-json", *extra, "src/lib.rs"],
                                 cwd=crate_of_items, capture_output=True, text=True)
            shapes[label] = json.loads(ran.stdout)["verification-results"] if ran.stdout.strip().startswith("{") else {"读不出": ran.stdout[-200:]}
        judge_cell("fake-verus-real-shape",
                   shapes["按函数"] == {"encountered-error": False, "encountered-vir-error": False, "verified": 2, "errors": 0, "is-verifying-entire-crate": False}
                   and shapes["整 crate"].get("success") is True and shapes["整 crate"].get("is-verifying-entire-crate") is True and shapes["整 crate"].get("verified") == 5,
                   f"假 Verus 的输出不像真 Verus：{shapes}")
        # 假 Verus 点模块的形态照真 Verus（文件头「只验一个函数」一段）：点根找不到子模块 kernel 里的函数，--verify-module 配 --verify-function 退 255、stdout 空
        module_shapes = {}
        for label, extra in (("点根", ["--verify-root"]), ("点 --verify-module", ["--verify-module", "kernel"]), ("点没有的模块", ["--verify-only-module", "nosuch"])):
            ran = subprocess.run([os.path.join(install, "verus"), "--no-cheating", "--rlimit", "7", "--output-json", *extra, "--verify-function", "count_up", "src/lib.rs"],
                                 cwd=crate_of_items, capture_output=True, text=True)
            module_shapes[label] = (ran.returncode, ran.stdout.strip()[:80], ran.stderr.strip().split("\n")[0] if ran.stderr.strip() else "")
        judge_cell("fake-verus-module-shape",
                   module_shapes["点根"][0] == 1 and module_shapes["点根"][2] == "error: could not find function count_up specified by --verify-function"
                   and module_shapes["点 --verify-module"] == (255, "", "Error: When using --verify-function, use --verify-only-module instead of --verify-module")
                   and module_shapes["点没有的模块"][2] == "error: could not find module nosuch specified by --verify-module or --verify-only-module",
                   f"假 Verus 点模块不像真 Verus：{module_shapes}")
        # 核按所在模块点（kernel_module）：根 src/lib.rs 点 --verify-root，src/kernel.rs、src/nested/mod.rs、src/nested/deep.rs 点 --verify-only-module
        # kernel、nested、nested::deep。四份核都进登记表，证明与空真两格跑完，逐次核假 Verus 收到的命令行（开关 submodule-verify-root 下子模块的项一律
        # --verify-root、点不中，这一格判错）
        def lemma_kernel(name, head=""):
            return (head + "use vstd::prelude::*;\nverus! {\n"
                    f"pub proof fn {name}(value: u64)\n    requires\n        value > 0,\n    ensures\n        value >= 1,\n{{\n}}\n}} // verus!\n")

        modules_root = make_root("modules", count=5)
        modules_source = os.path.join(modules_root, "crates", "core", "src")
        os.makedirs(os.path.join(modules_source, "nested"))
        for relative, text in (("lib.rs", lemma_kernel("root_lemma", "pub mod kernel;\npub mod nested;\n")), ("nested/mod.rs", lemma_kernel("nested_lemma", "pub mod deep;\n")),
                               ("nested/deep.rs", lemma_kernel("deep_lemma"))):
            with open(os.path.join(modules_source, relative), "w", encoding="utf-8") as handle:
                handle.write(text)
        edit_kernel(os.path.join(modules_root, REGISTRY), "kernel\tcrates/core/src/kernel.rs\n",
                    "".join(f"kernel\tcrates/core/src/{relative}\n" for relative in ("lib.rs", "kernel.rs", "nested/mod.rs", "nested/deep.rs")))
        arguments_path = os.path.join(work, "verus-arguments.txt")
        open(arguments_path, "w").close()
        os.environ["SELFTEST_VERUS_ARGUMENTS"] = arguments_path
        try:
            proof_exit, proof_output, _verdicts = run_items("proof", modules_root)
            vacuity_exit, vacuity_output, _verdicts = run_items("vacuity", modules_root)
        finally:
            os.environ.pop("SELFTEST_VERUS_ARGUMENTS", None)
        want_scopes = {"root_lemma": ["--verify-root"], "clamp_to_limit": ["--verify-only-module", "kernel"], "keep_small": ["--verify-only-module", "kernel"],
                       "nested_lemma": ["--verify-only-module", "nested"], "deep_lemma": ["--verify-only-module", "nested::deep"]}
        seen_scopes = collections.defaultdict(list)
        for line in open(arguments_path, encoding="utf-8").read().split("\n"):
            if not line:
                continue
            given = json.loads(line)
            function = given[given.index("--verify-function") + 1] if "--verify-function" in given else "（整个 crate）"
            seen_scopes[function].append([word for index, word in enumerate(given)
                                          if word in ("--verify-root", "--verify-only-module", "--verify-module")
                                          or (index and given[index - 1] in ("--verify-only-module", "--verify-module"))])
        # 每个函数两次：证明一项、空真一处（keep_small 那一处在闭包上，所属的还是 keep_small）
        judge_cell("module-per-kernel",
                   proof_exit == 0 and "各项 verified 之和 5、0 errors（登记 5）" in proof_output and vacuity_exit == 0 and "5 处 requires" in vacuity_output
                   and dict(seen_scopes) == {**{name: [scope, scope] for name, scope in want_scopes.items()}, "（整个 crate）": [[]]},
                   f"期望证明与空真都退 0、每个函数两次各点 {want_scopes}，实测证明退 {proof_exit}、空真退 {vacuity_exit}、点的是 {dict(seen_scopes)}："
                   f"{proof_output.strip()[-200:]} | {vacuity_output.strip()[-200:]}")
        # 核不在入口所在目录下：认不出模块，开跑之前判红、出路点 kernel_module
        outside_root = make_root("moduleoutside")
        os.makedirs(os.path.join(outside_root, "crates", "core", "proofs"))
        os.replace(os.path.join(outside_root, "crates", "core", "src", "kernel.rs"), os.path.join(outside_root, "crates", "core", "proofs", "kernel.rs"))
        with open(os.path.join(outside_root, "crates", "core", "src", "lib.rs"), "w", encoding="utf-8") as handle:
            handle.write('#[path = "../proofs/kernel.rs"]\npub mod kernel;\n')
        edit_kernel(os.path.join(outside_root, REGISTRY), "kernel\tcrates/core/src/kernel.rs\n", "kernel\tcrates/core/proofs/kernel.rs\n")
        got, output, _verdicts = run_items("proof", outside_root)
        judge_cell("module-unrecognized-red", got == 1 and "核 crates/core/proofs/kernel.rs 认不出是 crates/core 的哪个模块" in output and "kernel_module" in output,
                   f"期望认不出模块判红、出路点 kernel_module，实测退 {got}：{output.strip()[-300:]}")
        got, output, verdicts = run_items("proof", items_root)
        want_items = ["proof:core_sample::kernel::Pair::clone", "proof:core_sample::kernel::sum_to", "proof:core_sample::kernel::lemma_within_itself", "proof:core_sample::kernel::count_up"]
        # spec fn 不带 decreases 的不成项（开关 spec-fn-as-item 下 within 成项、验出 0 条判红，这一格判错）
        judge_cell("proof-spec-fn-not-item", got == 0 and list(verdicts) == want_items and "不成项的 spec fn 1 个" in output,
                   f"期望退 0、逐项 {want_items}、报不成项的 spec fn 1 个，实测退 {got}、{list(verdicts)}：{output.strip()[-300:]}")
        # proof fn 按函数验出 verified 1（JSON 里没有 success 键）判绿（开关 require-success-key-per-function 下判错）
        judge_cell("proof-fn-verified-one-green", verdicts.get("proof:core_sample::kernel::lemma_within_itself") == ("绿", "现判") and "proof:core_sample::kernel::lemma_within_itself：绿" in output,
                   f"期望 proof:core_sample::kernel::lemma_within_itself 现判判绿，实测 {verdicts}：{output.strip()[-300:]}")
        # 按函数验时 is-verifying-entire-crate 报 false 不判红（开关 require-entire-crate-per-function 下判错）
        judge_cell("proof-entire-crate-false-not-red", got == 0 and all(verdict == "绿" for verdict, _how in verdicts.values()) and "is-verifying-entire-crate" not in output,
                   f"期望逐项都绿，实测退 {got}、{verdicts}：{output.strip()[-300:]}")
        # 带 decreases 的 spec fn 有终止义务，成项（开关 spec-decreases-not-item 下判错）
        judge_cell("proof-spec-decreases-item", verdicts.get("proof:core_sample::kernel::sum_to", ("",))[0] == "绿" and "带 decreases 的 spec fn 1" in output,
                   f"期望 proof:core_sample::kernel::sum_to 成项判绿，实测 {verdicts}：{output.strip()[-300:]}")
        # verus! 里 derive(Clone) 的类型：Verus 验它生成的 <类型>::clone，成项（开关 derived-clone-not-item 下判错）
        judge_cell("proof-derived-clone-item", verdicts.get("proof:core_sample::kernel::Pair::clone", ("",))[0] == "绿" and "derive(Clone) 生成的 clone 1" in output,
                   f"期望 proof:core_sample::kernel::Pair::clone 成项判绿，实测 {verdicts}：{output.strip()[-300:]}")
        # 不同时 derive(Copy)、带 Vec 字段的 derive(Clone) 也成项（真 Verus 报 clone is not a copy、不给规格，函数体照验 1 条；开关 derived-clone-copy-only 下判错：之和 5 与登记的 6 不同）
        holder_kernel = SELFTEST_ITEMS_KERNEL.replace("pub open spec fn within(", "#[derive(Clone, Debug, PartialEq, Eq)]\npub struct Holder {\n    pub bytes: Vec<u8>,\n}\n\npub open spec fn within(", 1)
        holder_exit, holder_output, holder_verdicts = run_items("proof", make_root("proofitemsholder", kernel=holder_kernel, count=6))
        judge_cell("proof-derived-clone-not-copy-item", holder_exit == 0 and holder_verdicts.get("proof:core_sample::kernel::Holder::clone", ("",))[0] == "绿"
                   and "derive(Clone) 生成的 clone 2" in holder_output and "各项 verified 之和 6、0 errors（登记 6）" in holder_output,
                   f"期望 proof:core_sample::kernel::Holder::clone 成项判绿、之和 6，实测退 {holder_exit}、{holder_verdicts}：{holder_output.strip()[-300:]}")
        # 各项之和等于登记的数判绿（真仓的 56 那一格的小样：1 + 1 + 1 + 2 = 5）
        judge_cell("proof-sum-equals-registered-green", got == 0 and "各项 verified 之和 5、0 errors（登记 5）" in output,
                   f"期望之和 5 等于登记的 5、判绿，实测退 {got}：{output.strip()[-300:]}")
        # 之和比登记的少一条判红，点名没成项的 fn 与验出数和静态估不同的项（开关 count-unchecked 下判错）
        short_root = make_root("proofitemsshort", kernel=SELFTEST_ITEMS_KERNEL, count=6)
        got, output, _verdicts = run_items("proof", short_root)
        judge_cell("proof-sum-short-red-named", got == 1 and "各项 verified 之和 5 与登记的 6 不同（少 1 条" in output
                   and "没成项的 verus! fn（不带 decreases 的 spec fn，静态估 0 条）1 个：core_sample::kernel::within" in output and "verified 与静态估不同的项：没有" in output,
                   f"期望之和 5 与登记的 6 不同判红、点名 within，实测退 {got}：{output.strip()[-400:]}")
        # 选中的函数一条义务都没验（verified 0）判红（开关 zero-verified-accepted 下判错）；--item 只点它，不比之和
        zero_root = make_root("proofzero", kernel=SELFTEST_ITEMS_KERNEL.replace("        within(value, value),\n{\n", "        within(value, value),\n{\n    // SELFTEST_ZERO_VERIFIED\n"), count=5)
        got, output, verdicts = run_items("proof", zero_root, items=["proof:core_sample::kernel::lemma_within_itself"])
        judge_cell("proof-zero-verified-red", got == 1 and "Verus 在 --verify-function lemma_within_itself 上一条义务都没验（verified 0" in output,
                   f"期望 verified 0 判红，实测退 {got}、{verdicts}：{output.strip()[-300:]}")
        # errors 1 判红，红句照原样报 JSON 的几样（开关 nonzero-errors-accepted 下判错）；--item 只点它，不比之和
        broken_root = make_root("prooferrorsone", kernel=SELFTEST_KERNEL.replace("result <= limit,", "result <= limit, // SELFTEST_BROKEN"))
        got, output, verdicts = run_items("proof", broken_root, items=["proof:core_sample::kernel::clamp_to_limit"])
        judge_cell("proof-errors-one-red", got == 1 and "Verus 没验过（exec fn clamp_to_limit）：verified 0、errors 1、encountered-error true、vir 错 false" in output
                   and "postcondition not satisfied" in output,
                   f"期望 errors 1 判红、红句带 JSON 的几样与报错位置，实测退 {got}、{verdicts}：{output.strip()[-300:]}")
        # list-items 给了判定库：每格报键对不上几条（底稿第二节第 9 条 ⑦）；只读库、不写库、不起 Verus。三格记过之后改 clamp_to_limit 的 ensures：
        # 证明 1 项、空真 1 处、必红世界 3 行（都改在它里面）键对不上，共 5 条（开关 key-mismatch-by-item-only 下全报键相同，判错）
        listing_store = os.path.join(work, "verdict-library-listing")
        listing_root = make_root("listingkeys", must_red=[green_row.format("A"), green_row.format("B"), green_row.format("C")])
        for check in ("proof", "vacuity", "must-red-worlds"):
            run_items(check, listing_root, verdict_library=listing_store)

        def listing_entries():
            # 三格那三份库在临时服务里的全部条目（库目录只当名字，本地没有文件）：前后一样就是没写库
            return [(kind, entry) for kind in ("proof-function", "vacuity-requires", "must-red-world-row")
                    for entry in VerdictLibrary(listing_store, kind, "").store.scan("")]

        def listing_counts():
            snapshot = listing_entries()
            open(trace_path, "w").close()
            os.environ["SELFTEST_VERUS_TRACE"] = trace_path
            out, err = io.StringIO(), io.StringIO()
            try:
                with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
                    listed_exit = list_items(listing_root, listing_store, install)
            finally:
                os.environ.pop("SELFTEST_VERUS_TRACE", None)
            after = listing_entries()
            return listed_exit, err.getvalue(), bool(snapshot) and snapshot == after and open(trace_path).read() == ""

        before_exit, before_text, before_untouched = listing_counts()
        edit_kernel(os.path.join(listing_root, "crates", "core", "src", "kernel.rs"), "        result <= limit,\n", "        result <= limit + 0,\n")
        after_exit, after_text, after_untouched = listing_counts()
        judge_cell("list-items-key-mismatch-count",
                   before_exit == 0 and after_exit == 0 and before_untouched and after_untouched
                   and "verus-proof：键对不上 0 条、无记录 0 条、键相同 2 条、拒算 0 条" in before_text and "键对不上共 0 条" in before_text
                   and "verus-proof：键对不上 1 条、无记录 0 条、键相同 1 条" in after_text and "verus-vacuity：键对不上 1 条、无记录 0 条、键相同 1 条" in after_text
                   and "verus-must-red-worlds：键对不上 3 条、无记录 0 条、键相同 0 条" in after_text and "键对不上共 5 条（3 格比了 7 项" in after_text,
                   f"改之前退 {before_exit}（库与 Verus 没动 {before_untouched}）：{before_text.strip()!r}；改之后退 {after_exit}（没动 {after_untouched}）：{after_text.strip()!r}")
        # list-items --key-mismatches 逐项往 stderr 打 ITEM_KEY 行（门禁 --list-items --with-keys 转成 ITEM_PLAN 的原料）：改之前 7 项都键相同；
        # 改 clamp_to_limit 的 ensures 之后它的证明、空真与三行必红世界 5 项键对不上，keep_small 的证明与空真 2 项键相同（开关 item-key-lines-skipped 下一行都不打，这一格判错）
        def item_key_states(text):
            return dict(match.groups() for match in re.finditer(rf"^{ITEM_KEY_PREFIX} item=(.+) state=(\S+)$", text, re.M))
        before_states, after_states = item_key_states(before_text), item_key_states(after_text)
        judge_cell("list-items-item-key-lines",
                   len(before_states) == 7 and set(before_states.values()) == {"键相同"} and len(after_states) == 7
                   and sorted(after_states.values()).count("键对不上") == 5 and sorted(after_states.values()).count("键相同") == 2
                   and after_states.get("proof:core_sample::kernel::clamp_to_limit") == "键对不上" and after_states.get("proof:core_sample::kernel::keep_small") == "键相同"
                   and all(after_states.get(f"must-red:改坏{name}") == "键对不上" for name in "ABC"),
                   f"改之前 {before_states}，改之后 {after_states}")
        # list-items --key-mismatches 带 --item：只比点名的项，成本随点名项数走：点名 keep_small 的证明一项，只打这一项的 ITEM_KEY、只比了 1 项（"1 格比了 1 项"），
        # 空真与必红世界两格不比；点名两项（一项证明、一项必红世界）比 2 项（开关 key-items-ignored 下照旧比整表 7 项，这一格判错）
        def item_key_listing(named):
            keyed_error = io.StringIO()
            with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(keyed_error):
                keyed_exit = list_items(listing_root, listing_store, install, named)
            return keyed_exit, keyed_error.getvalue()
        one_exit, one_text = item_key_listing(["proof:core_sample::kernel::keep_small"])
        two_exit, two_text = item_key_listing(["proof:core_sample::kernel::keep_small", "must-red:改坏A"])
        judge_cell("list-items-item-key-with-item",
                   one_exit == 0 and two_exit == 0 and item_key_states(one_text) == {"proof:core_sample::kernel::keep_small": "键相同"}
                   and "（1 格比了 1 项" in one_text
                   and item_key_states(two_text) == {"proof:core_sample::kernel::keep_small": "键相同", "must-red:改坏A": "键对不上"} and "（2 格比了 2 项" in two_text,
                   f"点名一项 {item_key_states(one_text)}：{one_text.strip()[-200:]!r}；点名两项 {item_key_states(two_text)}：{two_text.strip()[-200:]!r}")
        # 逐项挂钟与条数（ITEM_DONE 的原料，环境里的 FORMAL_ITEM_DONE_FILE）：clamp_to_limit 的函数体里一行 SELFTEST_SLEEP_10 让假 Verus 在它身上睡 1 秒，keep_small 没有；
        # 两项同时验（jobs 2），每项记的是自己的挂钟（clamp_to_limit ≥ 0.9、keep_small < 0.6），不是整格的（开关 item-done-whole-cell-seconds 下 keep_small 也 ≥ 0.9，这一格判错）；
        # 整格这一项 proof 另记一行（现判 1、复用 0）；第二趟两项全复用（0 秒、现判 0、复用 1）；有一项判红的那一趟它与 proof 的退出码是 1
        # （开关 item-done-unrecorded 下一行都不记，这一格判错）
        done_root = make_root("item-done")
        edit_kernel(os.path.join(done_root, "crates", "core", "src", "kernel.rs"), "    if value > limit { limit } else { value }\n",
                    "    if value > limit { limit } else { value } // SELFTEST_SLEEP_10\n")
        done_file = os.path.join(work, "item-done.tsv")
        done_store = os.path.join(work, "verdict-library-item-done")

        def item_done_rows(verdict_library, environment=None):
            open(done_file, "w").close()
            run_items("proof", done_root, dict({ITEM_DONE_FILE_VARIABLE: done_file}, **(environment or {})), verdict_library=verdict_library, jobs=2)
            rows = [line.split("\t") for line in open(done_file, encoding="utf-8").read().split("\n") if line]
            return {row[0]: row[1:] for row in rows}, len(rows)
        clamp_item, small_item = "proof:core_sample::kernel::clamp_to_limit", "proof:core_sample::kernel::keep_small"
        first_rows, first_count = item_done_rows(done_store)
        second_rows, second_count = item_done_rows(done_store)
        red_rows, red_count = item_done_rows(fresh_library(), {"SELFTEST_FAIL_MARK": "if value > limit { limit }"})
        judge_cell("item-done-lines",
                   first_count == 3 and set(first_rows) == {clamp_item, small_item, "proof"}
                   and float(first_rows[clamp_item][0]) >= 0.9 and float(first_rows[small_item][0]) < 0.6
                   and all(first_rows[name][1:] == ["1", "0", "0"] for name in first_rows)
                   and second_count == 3 and second_rows[clamp_item] == ["0.0", "0", "1", "0"] and second_rows[small_item] == ["0.0", "0", "1", "0"]
                   and second_rows["proof"][1:] == ["1", "0", "0"]
                   and red_count == 3 and red_rows[clamp_item][3] == "1" and red_rows[small_item][3] == "0" and red_rows["proof"][3] == "1",
                   f"第一趟 {first_rows}，第二趟（全复用）{second_rows}，一项判红那一趟 {red_rows}")
        # 门禁样本档的假工具（write-fake-tools）：写出可执行的 verus、cargo、cargo-verus，version.json 的 commit 抄登记表 release 行（开关 fake-tools-commit-unpinned 下写成全 0，这一格判错）
        fake_tools_directory = os.path.join(work, "fake-tools-written")
        with contextlib.redirect_stdout(io.StringIO()):
            fake_tools_exit = write_fake_tools(fake_tools_directory, make_root("fake-tools-root"))
        fake_version = json.load(open(os.path.join(fake_tools_directory, "version.json")))["verus"] if fake_tools_exit == 0 else {}
        judge_cell("fake-tools-written",
                   fake_tools_exit == 0 and fake_version.get("commit") == commit
                   and all(os.access(os.path.join(fake_tools_directory, tool), os.X_OK) for tool in ("verus", "cargo", "cargo-verus")),
                   f"退 {fake_tools_exit}，version.json {fake_version}（期望 commit {commit}）")
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
        three_affected = ["proof:core_sample::kernel::clamp_to_limit", "vacuity:core_sample::kernel::clamp_to_limit#1", "must-red:改坏A", "must-red:改坏B", "must-red:改坏C"]
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
        # 一个 trait impl 里两个方法的单元闭包一模一样（样本先核这一点：两个入口的闭包键相同），键里另带「被验入口单元」才各是各的：
        # 一趟一处（jobs=1，width 后判、后写）记下，第二趟两项都复用、各取自己那一份，各项之和 5 等于登记（开关 reuse-key-without-entry-unit 下两项共用一把键，
        # fill 复用到 width 那一份的 1 条，之和 4 判红，这一格判错）
        sibling_store = os.path.join(work, "verdict-library-entry-unit")
        sibling_root = make_root("reuseentryunit", kernel=selftest_kernel_with(SELFTEST_IMPL_SIBLINGS_EXTRA), count=5)
        sibling_index = FunctionUnitIndex(sibling_root)
        sibling_closures = {name: sibling_index.key([f"crates/core/src/kernel.rs::<impl SampleStore for SampleDevice>::{name}"]).key for name in ("fill", "width")}
        sibling_first = run_items("proof", sibling_root, verdict_library=sibling_store, jobs=1)
        sibling_second = run_items("proof", sibling_root, verdict_library=sibling_store, jobs=1)
        judge_cell("reuse-proof-entry-unit-own-key", None not in sibling_closures.values() and len(set(sibling_closures.values())) == 1
                   and sibling_first[0] == 0 and "现判 4、复用 0" in sibling_first[1]
                   and sibling_second[0] == 0 and "现判 0、复用 4" in sibling_second[1] and "各项 verified 之和 5" in sibling_second[1],
                   f"两个方法的闭包键 {sibling_closures}（期望相同）；头一趟退 {sibling_first[0]}：{sibling_first[1].strip()[-200:]}；"
                   f"第二趟期望全复用、之和 5，实测退 {sibling_second[0]}：{sibling_second[1].strip()[-300:]}")
        # 空真同理：两个方法各一处 requires，头一个空真（判红、记红）、后一个判绿；第二趟空真那一处照样现判、判红，另一处复用
        # （开关 reuse-key-without-entry-unit 下两处共用一把键，后写的绿盖掉红，空真那一处复用成绿，这一格判错）
        latch_store = os.path.join(work, "verdict-library-owner-unit")
        latch_root = make_root("reuseownerunit", kernel=selftest_kernel_with(SELFTEST_IMPL_SIBLINGS_VACUITY_EXTRA), count=4)
        latch_first = run_items("vacuity", latch_root, verdict_library=latch_store, jobs=1)
        latch_second = run_items("vacuity", latch_root, verdict_library=latch_store, jobs=1)
        judge_cell("reuse-vacuity-owner-unit-own-key", latch_first[0] == 1 and latch_second[0] == 1
                   and latch_second[2].get("vacuity:core_sample::kernel::SampleLatch::open_gate#1") == ("红", "现判")
                   and latch_second[2].get("vacuity:core_sample::kernel::SampleLatch::close_gate#1") == ("绿", "复用"),
                   f"头一趟退 {latch_first[0]}（期望 1）；第二趟期望空真那一处现判判红、另一处复用，实测退 {latch_second[0]}、{latch_second[2]}：{latch_second[1].strip()[-300:]}")
        # assume_specification 进键、入库、复用（复用三问的形态，判法同 reuse-three-questions，只是不打规范输出那一行）：cargo 那一种 crate 连同路径依赖
        # singlefs-format，两条 assume_specification 各指它的一个常量。头一趟（空库）、同一份输入第二趟（现判 0）、改一个没人指的常量（现判 0）、
        # 改 SLOT_WIDTH 的值（现判的恰好是指它的那一项）各跑一趟
        # （开关 assume-specification-refused 下一律拒算、第二趟照样现判，assume-specification-target-unkeyed 下改 SLOT_WIDTH 照样复用，这一格都判错）
        assume_store = os.path.join(work, "verdict-library-assume-specification")
        assume_kernel = selftest_kernel_with(SELFTEST_ASSUME_TARGETS_EXTRA)

        def assume_root(name, trust_rows=()):
            root = make_root(name, kernel=assume_kernel, count=4, crate_mode="\tcargo", extra_dependency='singlefs-format = { path = "../format" }\n', trust_rows=trust_rows)
            os.makedirs(os.path.join(root, "crates", "format", "src"))
            open(os.path.join(root, "crates", "format", "Cargo.toml"), "w").write('[package]\nname = "singlefs-format"\nedition = "2021"\n')
            open(os.path.join(root, "crates", "format", "src", "lib.rs"), "w").write(SELFTEST_ASSUME_TARGETS_FORMAT)
            return root

        _assume_harvest_exit, assume_harvested = captured_stdout(run_check, "proof", assume_root("assumeharvest"), dict(options, verdict_library=fresh_library()))
        assume_digests = dict(re.findall(r"的 assume_specification (\S+) 没登记进信任清单 \S+（这一处的规格原文 sha256 ([0-9a-f]{64})）", assume_harvested.getvalue()))
        assume_three_root = assume_root("assumethree", trust_rows=[
            f"assume_specification\tcrates/core/src/kernel.rs\t{point}\t{assume_digests.get(point, '0' * 64)}\tselftest_differential\t自证用：登记的可信声明"
            for point in ("singlefs_format::SLOT_WIDTH", "singlefs_format::RING_START")])
        format_path = os.path.join(assume_three_root, "crates", "format", "src", "lib.rs")

        def assume_round():
            got, output, _verdicts = run_items("proof", assume_three_root, verdict_library=assume_store)
            return reuse_checks.states_from_output(output), got, output

        assume_rounds = [assume_round(), assume_round()]
        edit_kernel(format_path, "UNUSED_SAMPLE: u64 = 3;", "UNUSED_SAMPLE: u64 = 4;")
        assume_rounds.append(assume_round())
        edit_kernel(format_path, "SLOT_WIDTH: u64 = 8;", "SLOT_WIDTH: u64 = 9;")
        assume_rounds.append(assume_round())
        assume_affected = ["proof:core_sample::kernel::assume_specification[singlefs_format::SLOT_WIDTH]"]
        assume_passed, _assume_line, assume_problems = reuse_checks.three_questions(SCRIPT_KEY, *[states for states, _got, _output in assume_rounds], assume_affected)
        judge_cell("reuse-assume-specification-three-questions", assume_passed and len(assume_rounds[0][0]) == 4 and all(got == 0 for _states, got, _output in assume_rounds),
                   f"四趟各退 {[got for _states, got, _output in assume_rounds]}、头一趟的项 {assume_rounds[0][0]}；{'；'.join(assume_problems)}；"
                   f"末一趟：{assume_rounds[-1][2].strip()[-300:]}")
        # 指到工作区外（std 的方法）的那一条照旧拒算，拒算行点名是哪一处（开关 assume-specification-refused 下另一条也拒算，上一格判错）
        assume_outside = run_items("proof", make_root("assumeoutside", kernel=const_kernel, count=5, trust_rows=const_rows), verdict_library=os.path.join(work, "verdict-library-assume-outside"))
        judge_cell("reuse-assume-specification-outside-refused", assume_outside[0] == 0
                   and "assume_specification[<[T]>::binary_search]：绿，现判（拒算（被验函数单元闭包）：被验函数单元闭包：refused\tassume-specification-target\tcrates/core/src/kernel.rs:" in assume_outside[1],
                   f"指 std 方法的那一条期望拒算、拒算行点名 assume-specification-target，实测退 {assume_outside[0]}：{assume_outside[1].strip()[-400:]}")
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
            print(one_line(f"  ✗ tool-versions-z3 这一格判错：换 z3、设 VERUS_Z3_PATH 之后打的行应当各不相同，实测：{without_z3.strip()} | {with_z3.strip()} | {with_override.strip()}"))  # gate-lint:detail
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
                print(one_line(f"  ✗ tool-versions-{name} 这一格判错：前后两种环境打的行应当不同，实测都是：{first.strip()}"))  # gate-lint:detail
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
            print(one_line(f"  ✗ tool-versions-verus-environment 这一格判错：{len(environment_lines)} 种环境打的行应当两两不同，实测只有 {len(set(environment_lines.values()))} 种："  # gate-lint:detail
                  + " | ".join(f"{label}：{line.strip()[-120:]}" for label, line in environment_lines.items())))
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
            print(one_line(f"  ✗ tool-versions-rustup-toolchain 这一格判错：{len(rustup_lines)} 种 rustup 工具链与 rustup 打的行应当两两不同，实测只有 {len(set(rustup_lines.values()))} 种："  # gate-lint:detail
                  + " | ".join(f"{label}：{line.strip()[-160:]}" for label, line in rustup_lines.items())))

    # 默认安装目录不经家目录：不设 --verus-root 与 SINGLEFS_VERUS_ROOT 时落在共享数据根下的 formal-tools（HOME 与 XDG_DATA_HOME 指到别处、SINGLEFS_DATA_ROOT 指到一份假数据根）；
    # --verus-root 盖过它；第五列是绝对路径就是它自己；数据根取不到、第五列写法不认都判红并给出路，不退回家目录（开关 default-install-by-home 下这一格判错）
    cases += 1
    fake_data_root = os.path.join(work, "default-install-data-root")
    saved_install_environment = {name: os.environ.get(name) for name in ("HOME", "XDG_DATA_HOME", "SINGLEFS_VERUS_ROOT", "SINGLEFS_DATA_ROOT", "SINGLEFS_MULTI_HOST_CONFIG")}
    install_registry, odd_install_registry, absolute_install_registry = Registry(), Registry(), Registry()
    install_registry.install_relative = "singlefs/formal-tools/verus-0.9.9/verus-x86-linux"
    odd_install_registry.install_relative = "elsewhere/formal-tools/verus"
    absolute_install_registry.install_relative = os.path.join(work, "stand-in-install")
    try:
        os.environ["HOME"] = os.path.join(work, "default-install-other-home")
        os.environ["XDG_DATA_HOME"] = os.path.join(work, "default-install-xdg")
        os.environ.pop("SINGLEFS_VERUS_ROOT", None)
        os.environ["SINGLEFS_DATA_ROOT"] = fake_data_root
        landed_install = verus_directory(install_registry, None)
        explicit_install = verus_directory(install_registry, os.path.join(work, "explicit-install"))
        absolute_install = verus_directory(absolute_install_registry, None)
        os.environ.pop("SINGLEFS_DATA_ROOT")
        os.environ["SINGLEFS_MULTI_HOST_CONFIG"] = os.path.join(work, "no-such-multi-host.env")
        refusals = {}
        for label, registry_under_test in (("数据根取不到", install_registry), ("第五列不以 singlefs/formal-tools/ 起头", odd_install_registry)):
            try:
                refusals[label] = f"没拒，交 {verus_directory(registry_under_test, None)}"
            except Red as red:
                refusals[label] = "拒" if red.howto else "拒但没给出路"
    finally:
        for name, value in saved_install_environment.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value
    if (landed_install != os.path.join(fake_data_root, "formal-tools", "verus-0.9.9", "verus-x86-linux")
            or explicit_install != os.path.join(work, "explicit-install") or absolute_install != absolute_install_registry.install_relative
            or set(refusals.values()) != {"拒"}):
        wrong.append("default-install-not-by-home")
        print(one_line(f"  ✗ default-install-not-by-home 这一格判错：HOME 与 XDG_DATA_HOME 指到别处、SINGLEFS_DATA_ROOT 指到 {fake_data_root} 时，默认安装目录应当是它下面的 "  # gate-lint:detail
                       f"formal-tools/verus-0.9.9/verus-x86-linux，实测 {landed_install}；--verus-root 应当盖过它，实测 {explicit_install}；第五列是绝对路径应当就是它自己，实测 {absolute_install}；"
                       f"取不到数据根与第五列写法不认应当判红，实测 {refusals}"))

    # --threads 的缺省：没设 SINGLEFS_THREAD_CAP 取登记文件第一条非注释行的数，设了取它
    with open(os.path.join(SCRIPT_DIRECTORY, "..", "..", ".claude", "heavy-thread-default"), encoding="utf-8") as handle:
        registered_default = next((int(line.split()[0]) for line in handle if line.strip() and not line.lstrip().startswith("#")), None)
    cases += 1
    unset_threads, capped_threads = default_threads({}), default_threads({"SINGLEFS_THREAD_CAP": "7"})
    if registered_default is None or unset_threads != registered_default or capped_threads != 7:
        wrong.append("threads-default-registered")
        print(one_line(f"  ✗ threads-default-registered 这一格判错：登记 {registered_default}，没设上限得 {unset_threads}，上限 7 得 {capped_threads}"))  # gate-lint:detail

    # 开关轮没检出的那一趟要说出为什么：信号、崩了（回溯末行）、退 0、退 1 没有判错格、别的退出码，各带输出末行
    cases += 1
    miss_samples = {
        "crashed": (1, "  ✓ 前面的格\nTraceback (most recent call last):\n  File \"x.py\", line 1, in <module>\nValueError: substring not found\n"),
        "signalled": (-9, ""),
        "all-green": (0, "  ✓ verus-proof-check 自证 3 格都判对\n"),
        "red-without-cell": (1, "  ✗ 别的一句\n     → 怎么办：x\n"),
        "usage": (2, "  ✗ 用法\n"),
    }
    miss_wants = {"crashed": ("退 1、崩了", "输出末行：ValueError: substring not found"), "signalled": ("被信号 9 停了", "输出末行：（没有输出）"),
                  "all-green": ("退 0、每格都判对",), "red-without-cell": ("退 1 却没有一格报",), "usage": ("退 2；",)}
    miss_details = {name: selftest_break_miss_detail(*sample) for name, sample in miss_samples.items()}
    miss_wrong = {name: miss_details[name] for name, wants in miss_wants.items() if any(want not in miss_details[name] for want in wants)}
    if miss_wrong:
        wrong.append("break-miss-detail")
        print(one_line(f"  ✗ break-miss-detail 这一格判错：开关轮没检出的那一趟要说出为什么，这几样说得不对：{miss_wrong}"))  # gate-lint:detail

    breaks_missed, breaks_proven = [], 0
    if not wrong and not BREAK and not reuse_checks.BREAK and not FORMAL_BREAK:
        items = ("commit-unchecked", "fingerprint-unchecked", "absent-kernel-not-run", "cheating-unchecked", "count-unchecked", 
                     "vacuity-accepts-verified", "vacuity-all-at-once", "mutation-accepts-green", "anchor-unchecked", "rlimit-counts-as-red", "rlimit-unpinned",
                     "rlimit-any-line-voids", "rlimit-attribute-unchecked", "mutation-rlimit-attribute-accepted",
                     "expectation-form-unchecked", "row-segments-unchecked", "landing-unchecked", "landing-by-count", "landing-read-from-unmutated-text",
                     "landing-form-unchecked", "rlimit-counted-in-proof-errors",
                     "tool-versions-launcher-only", "tool-versions-five-files-only", "tool-versions-empty-z3-path-as-unset",
                     "tool-versions-bare-z3-from-current-directory", "tool-versions-verus-environment-ignored", "killed-as-proof-red",
                     "tool-versions-rustup-ignored", "default-install-by-home",
                     "item-filter-ignored", "reuse-key-without-units", "reuse-key-whole-crate", "reuse-key-function-only",
                     "vacuity-key-by-line-number", "must-red-key-with-name-and-path", "reuse-key-without-judging-units",
                     "reuse-key-without-toolchain", "reuse-key-without-row",
                     "reuse-ignores-key",
                     "reuse-records-red", "reuse-records-whole-cell-only", "threads-default-cpu-count", "list-items-pipe-unhandled",
                     "spec-fn-as-item", "spec-decreases-not-item", "derived-clone-not-item", "valued-const-not-item", "assume-specification-not-item",
                     "could-not-compile-counted", "const-fn-heading-unrecognized", "require-success-key-per-function",
                     "require-entire-crate-per-function", "zero-verified-accepted", "nonzero-errors-accepted", "submodule-verify-root", "key-mismatch-by-item-only",
                     "reuse-lookup-skipped", "body-open-past-unit-end", "trait-declaration-by-brace", "trait-impl-lag-matched-by-trait-only",
                     "trait-impl-lag-stale-unchecked", "trait-impl-lag-growth-unchecked", "trait-impl-lag-passed-unnamed", "REUSE_CHECKS_BREAK=rejudged-sentinel-passes",
                     "mir-panic-path-unnormalized", "mir-aliased-items-skipped", "mir-alias-by-prefix", "mir-alias-without-boundary", "derived-clone-copy-only",
                     "whole-entry-past-unit-end", "trait-impl-lag-not-run-unreported", "cfg-entry-past-unit-end", "associated-const-bare-name",
                     "point-names-by-containment", "assert-by-requires-as-insertion-point", "assert-by-requires-by-semicolon-line", "spec-const-not-item",
                     "outer-memory-narrows-jobs", "group-leftovers-kept",
                     "proof-item-narrows", "red-keeps-green", "reuse-key-without-crate-build", "reuse-key-without-closure-dependencies",
                     "toolchain-key-with-paths",
                     "item-key-lines-skipped", "key-items-ignored", "item-done-unrecorded", "item-done-whole-cell-seconds", "fake-tools-commit-unpinned",
                     "reuse-key-without-entry-unit", "assume-specification-refused", "assume-specification-target-unkeyed",
                     *(f"FORMAL_VERIFICATION_BREAK={item}" for item in FORMAL_BREAK_ITEMS))
        # 各开关的那一趟互不依赖、各用各的临时目录：几趟同时起（至多 SELFTEST_BREAK_WORKERS 趟），按开关的次序收退出码、数份数；
        # 写成「变量=项」的是共用库 lib_reuse_checks.py 的开关（复用哨兵被现判不红：reuse-sentinel 那一格判错）
        def run_break(item):
            variable, _equals, value = item.rpartition("=") if "=" in item else ("VERUS_PROOF_CHECK_BREAK", "=", item)
            completed = subprocess.run([sys.executable, os.path.realpath(__file__), "--selftest"], capture_output=True, text=True,
                                       env=dict(os.environ, **{variable: value}))
            output = completed.stdout + completed.stderr
            caught = selftest_break_caught(completed.returncode, output)
            return item, caught, "" if caught else selftest_break_miss_detail(completed.returncode, output)

        with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, min(SELFTEST_BREAK_WORKERS, int(default_heavy_threads(os.environ))))) as pool:
            outcomes = list(pool.map(run_break, items))
        if len(outcomes) != len(items):
            breaks_missed.append(f"开关轮收回 {len(outcomes)} 趟、派出 {len(items)} 趟")
        for item, caught, detail in outcomes:
            if not caught:
                breaks_missed.append(item)
                print(f"    · 开关 {item} 没检出：{detail}")
            else:
                breaks_proven += 1
    if wrong:
        report_red(f"verus-proof-check 自证 {cases} 格里 {len(wrong)} 格判错（逐格见上面以 ✗ 起头、带「这一格判错」的几行）",  # gate-lint:summary
                   "按判错那几格的标签找对应的 check_* 函数修；判据写在文件头")
        return EXIT_RED
    if breaks_missed:
        report_red(f"弄坏开关没有被具体自证格检出：{' '.join(breaks_missed)}", "看上面每个开关「没检出」那一行：崩了的照回溯末行修那一趟撞上的代码，被信号停了的查是谁停的；退 0 或退 1 却没有判错格的，补自证或修开关")
        return EXIT_RED
    suffix = f"；弄坏开关 {breaks_proven} 个，每个都让至少一格判错" if not (BREAK or FORMAL_BREAK or reuse_checks.BREAK) else "（开着弄坏开关，没重跑开关）"
    print(f"  ✓ verus-proof-check 自证 {cases} 格都判对{suffix}")
    return EXIT_GREEN


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
