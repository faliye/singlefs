#!/usr/bin/env python3
# admission: always 判的是这一次给的根下此刻的源码，上一次算出来的键不替这一次作保
# run-condition: command bash
# gate-similar: lib_source_digest.py 那一份按文件算不随注释变的摘要，不切单元、不收调用图；这里的 bash 规范化与它同法（搬过来没 import：它下一批改成调这里）
# gate-similar: mutation-prune.py 它的 FileIndex 切 fn 项与 impl 方法、没有调用图，函数外部分与宏按整文件取；这里照设计第三节另写切分与八条闭包规则
# gate-similar: admission.py 它的 crash_case_judging_digest_text 那份 python 闭包只看一个模块、只认 ast.Name、单元原文带注释；这里扩到跨模块、方法收窄与拒算
"""函数单元：把源码切成函数一级的单元，从入口顺调用图收闭包，算一把只随闭包里单元内容变的键（Rust、WGSL、Python、shell）。
设计照第二轮复查 design 路报告第三节（3.1 键、3.3 Rust 八条闭包规则 R-目标 / R-裸名 / R-路径 / R-方法 / R-隐式 / R-开放 / R-结构 / R-宏、
3.4 拒算、3.5 WGSL、3.6 Python、3.7 shell）。Rust 这一份与 crates/singlefs-unit-graph 是两条不共用代码的路子：这里不 import、不读那个 crate，
两边照同一份规则各写各的；门禁 code-tooling 的 function-units-two-implementations-agree 格逐单元对拍 dump（种类、限定名、起止、指纹），
再逐个比登记入口的闭包成员与键（python 这一边的键照 unit-graph 的写法算：unit_graph_key）。dump 与闭包输出的写法以 crates/singlefs-unit-graph 为准。

输出（一行一项、制表符分隔、次序确定；Rust 与 WGSL 的单元行与 crates/singlefs-unit-graph 的 unit-graph 逐字相同）：
  单元行   <种类>\t<限定名>\t<文件>:<起>-<止>\t<指纹>（Unit.dump_text；Unit.text 是进 UnitKey 组成清单与各判法按（种类, 限定名, 文件）认单元的那一行，
           限定名是文件里的嵌套路径、不带前缀）
  数据行   data\t<数据文件>\t<读它的文件>:<行>-<行>\t<字节 sha256>（dump 里每一处读一行，与 unit-graph 的 dump 同一种写法）
  键行     key\t<sha256>
  拒算行   refused\t<形态>\t<文件>:<行>\t<原样片段>
  Rust、WGSL 与数据行按（文件，起行，止行，整行）排（与 unit-graph 的 dump 同一种次序），python 与 shell 的单元行按（文件，起行，种类，限定名）排，
  拒算行按（文件，行，形态，片段）排；文件是相对根的路径，起止是 1 起算的行号；
  原样片段是那一行原文，空白折成一个空格，至多 160 个字符。closure --explain 另打 via\t<文件>:<限定名>\t<从哪个单元来>\t<规则>，只当诊断看。

种类与限定名：
  Rust   种类名照 crates/singlefs-unit-graph（src/units.rs 的 output_name）：fn（自由 fn 与 impl / trait 里的方法）、impl（impl 头：属性起、到 { 之前）、
         trait（trait 头）、const、static、struct、enum、union、type、macro_rules（macro_rules! 整个定义）、use、item_macro（条目级宏调用）、mod（mod x;）、
         mod_head（模块头：行内模块从外属性起、到 { 与紧跟的内属性 #![…]；文件开头的内属性 #![…] 另一个，限定名 <module attributes>；模块里的单元都连它）、
         extern_block、extern_crate、broadcast_group（verus! 里的 broadcast group）、doctest（文档注释里的代码块）；
         闭包里的数据单元 data-file（include_str! / include_bytes! 与运行期拼得出字面路径读的文件：限定名是那份文件相对根的路径，起止 1-<行数>，指纹是字节 sha256），
         dump 照 unit-graph 按读它的那一处列（上面「数据行」）。
         文件里的嵌套路径（Unit.name，入口 <文件>::<限定名> 照它找）用 :: 连：内联 mod 写名字；impl 写 <impl 词法单元…>（impl 关键字之后、去掉紧跟它的泛型参数表与 where 子句，
         逐个词法单元用一个空格连，相邻两个 : 连成 :: 且两边不留空格，例 <impl std::ops::Add for Money>、<impl Wrapper < T >>）；trait 头写名字，它的成员挂在 <trait 名> 下；use 写 use@<起行>
         （verus! 里的 broadcast use 写 broadcast-use@<broadcast 那一行>）；条目级宏调用写 <宏名>!@<宏名那一行>；extern 块写 extern@<起行>；
         文档测试写 <所挂条目的嵌套路径>::<doctest line 代码块起行>，挂模块的写 <模块路径>::<doctest line 行>（文件开头的就是 <doctest line 行>）。
         输出行里的限定名（Unit.full）照 unit-graph 带前缀：<crate 名>::<目标标记>::<模块路径>::<嵌套路径>——目标标记取这份文件所属的头一个目标
         （crate 里 lib、bin、test、example、bench、build 的次序，同种按名字与根文件排；lib 不写，bin 写 <bin 名>，构建脚本写 <build>），模块路径是它在那个目标的
         模块树里经 mod 声明连进来的那一截；哪个目标都不属于的写 <unlinked 文件>。例 singlefs_checker_tier::<bin e158_root_choice_repair>::main。
         verus! { … } 与 thread_local! { … } 展开成里面的条目再切；verus! 里一个条目从它起头到下一个同层条目起头之前（括号配平，
         同层条目起头 = ; 或 } 之后接属性、条目关键字或宏调用）。
         指纹 = 单元的词法单元原文用 0x1f 连起来的 sha256。词法：去掉 // 与可嵌套的 /* */，文档注释 /// //! /** */ /*! */ 也去掉；
         串、原始串、字节串、C 串、字符、生命周期、数字（带指数与后缀）各一个词法单元，标识符一个，其余标点一个字符一个。
         单元从它的第一个外属性起（文档注释不算）、到条目末尾（含收尾的 ;）。
         文档测试照 unit-graph：条目的外文档注释（落在条目首词法单元到属性之后那个词法单元之间的）与模块开头（文件开头、行内模块体开头到内属性收尾）的
         内文档注释各自拼起来找代码块（三个以上 ` 或 ~ 的围栏，信息串是 rustdoc 当 Rust 编的那几种），一个代码块一个单元；指纹 = 代码块（去掉隐藏行记号 # ）
         重新切词、去注释，词法单元以 0x1f 连的 sha256（只改代码块里的注释不变）。字段、枚举变体、函数体里的条目与行内模块头上的外文档注释不挂。
  WGSL   wgsl_fn、wgsl_compute（带 @compute 的 fn；@vertex / @fragment 照 wgsl_fn）、wgsl_const、wgsl_override、wgsl_struct、wgsl_alias、wgsl_var、
         wgsl_directive（enable / requires / diagnostic，名字 <词>@<行>）、wgsl_const_assert（名字 const_assert@<行>）；输出行的限定名是 <crate 名>::<文件名>::<名字>；
         词法与指纹同 Rust。Rust 入口的闭包里各单元 include_str! / include_bytes! 读到的 .wgsl 合成一个模块，从模块里的 @compute 入口（加 enable 一类指令
         与 const_assert）按名字收（模块里一个 @compute 都没有就收全部）。
         外名（模块里没定义、不是内建名的名字；_ 算内建名）的着落照 unit-graph（WgslNameSources）：整名是闭包里某个 Rust 串字面量里的一个词；或整名是闭包里
         常量、静态量、条目级宏调用按名字定义的名字，或这样一个名字以 _ 加整名收尾（GPU_FAILURE_POINT_NONE 供 FAILURE_POINT_NONE）；或首段连 _
         （按第一个 _ 切、连那个 _，T_HEADER 取 T_）不短于 2 个字符、落在闭包里某个 Rust 单元的串字面量或 macro_rules! 体里，且其余那一段（不分大小写）
         是闭包里某个串字面量的一个词或某个单元定义、点名的名字；没有着落的拒算 wgsl-undefined-name。
         着落做成边：Rust 入口的闭包里还没有着落的外名，把链接集里、闭包外供它的单元（按名字定义了它或以 _ 加它收尾的常量、静态量、条目级宏调用，
         不是这一种的、串字面量里有它这个词的单元）连同它们的闭包并进来，再收再核，到不再有新单元为止（每个只试一次）；
         外名在链接集里别的单元 include_str! / include_bytes! 读进来、还不在模块里的一份 .wgsl 的顶层条目里有定义时，同样补一条着落边：
         读它的那个 Rust 单元连同它的闭包并进来，那份 .wgsl 进模块、名字在合起来的名字空间里解；拉不进的才拒算 wgsl-undefined-name。
         WGSL 入口（--rust-strings-entry 给 Rust 入口）的闭包带上给名字着落的 Rust 单元连同它的闭包。
  Python def（模块级 def / async def，类里的方法写 <类>.<方法>）、class（类头：装饰器、基类、类体里不是 def 的语句）、assign（模块级赋值，
         按目标名各一单元）、import（模块级 import，按绑定的名字各一单元）、module-body（其余模块级语句，限定名 <module-body>，每份文件一个，
         没有语句也有）、main-guard（if __name__ == "__main__": 那一段，限定名 <main-guard>）；指纹 = ast.dump(节点, include_attributes=False)
         的 sha256，节点里各层 def / class 的文档串先去掉。数据单元 docstring（被读 __doc__ 的文档串，限定名 <所属>.__doc__，模块的写 <module>.__doc__）、
         source-text（读自己原文的那份文件整份字节，限定名 <source-text>）。
  shell  function（规范文本里 function <名> () 起、同缩进的 } 止；嵌套的另算一个，限定名 <外层>::<名>，外层文本里留着它；here-doc 正文跳过不切）、
         script-body（规范文本去掉顶层函数，限定名 <script-body>）、shell-variable（顶层赋值语句，按变量名各一单元）、
         kept-comments（数据单元：程序读的注释「# <词>:」与 # shellcheck，限定名 <kept-comments>）。规范文本 = bash -n 过了且没有告警、
         整份包进一个函数定义 eval（不执行）、取 declare -f；指纹 = 单元规范文本的 sha256。起止行是回到原文按函数头找的，只当索引。

Rust 闭包（3.3，与 crates/singlefs-unit-graph 的 src/closure.rs 逐条同义；这里只写 R-结构、R-路径、R-隐式与 R-开放的口径，别的几条照设计原文）：
  R-结构  成员连它的 impl / trait 头；单元（文档测试除外）连所在模块的模块头（mod_head），连这个目标里把它所在文件连进模块树的每一条 mod 声明
          （mod 声明再连它所在模块的头与声明，一路连到目标根；#[path] mod 的被引文件一样），连所在模块里引进了它用到的名字（裸名或方法名）的 use，
          use a::* 一律连。use 引进的名字只用来判连不连那条 use，不往下解（被引模块里没用到的单元不进闭包）；use a::b as c 里用到 c 的，b 按 R-裸名再解。
          use、mod 声明、模块头、extern crate 只是结构：它们读的文件照收，里面的名字不往下解。use 里花括号中的 self 引进 { 前面那一段
          （use std::io::{self, Seek} 引进 io）。文档测试是另编的一个 crate：不连所挂的条目、模块头、mod 声明与那个模块的 use。
  R-路径  Q::x：trait Q 的成员 x、自身类型里有 Q 的 impl 的成员 x、以 Q 为 trait 写的 impl（自身类型已被点名或是 blanket impl）的成员 x；
          impl 成员里的 Self::x 落到自身类型相同的 impl 的成员 x，trait 成员里的 Self::x 落到这个 trait 自己的成员 x。
  R-方法  .x( 落到 trait 的成员 x、blanket impl 的成员 x、自身类型已被点名的 impl 的成员 x；trait 头进闭包时它的 blanket impl 连同成员一起进。
          blanket impl：自身类型里任何深度出现这个 impl 的任一类型或常量泛型参数的 trait impl（impl<T> Tr for T、&T、Box<T>、(T, u32)、[T]、[T; N]、
          Option<Box<T>>、Vec<T> 都是）；只有生命周期参数、自身类型全是具体类型的不是，按被点名的类型。
  R-隐式  类型被点名时，自身类型（impl … for 之后那一截）里有这个名字的 trait impl 头与全部成员进闭包；只出现在 trait 参数里的（From<X> 的 X）不算。
  R-开放  入口签名（到函数体的 { 之前）里紧跟 impl、dyn、:、+ 的名字当 trait 名（dyn Trait 与泛型 trait 约束），连这个 trait 的全部 impl 头与成员；
          按 trait 自己的名字（for 之前那一截最外层的最后一个名字）找它的 impl，不按 trait 参数里的名字。
          闭包里单元的 dyn Trait 不连 impl：具体类型照 R-隐式连，Trait 照 R-裸名连到 trait 头，trait 头带它的 blanket impl。

键（3.1）：下面这些行各以 \\n 收尾、连起来取 sha256：
  form\tsinglefs-function-units-v1
  cfg\t<原子>\t<true|false>           Rust 闭包里各单元真引到的 cfg 原子（定案 2）：单元自己连外层的条件链（#[test] 不算）与它词法单元里每一处
                                        #[cfg(…)]、#![cfg(…)]、#[cfg_attr(条件, …)]（条件与里面套的 cfg）、cfg!(…) 的谓词的每个叶子，写 <名> 或 <名>="<值>"，
                                        真值按收这个单元进来的那个目标的生效 cfg 判（特性按这个单元所在的包）；没引到的原子不进键；判不了真假的拒算
                                        cfg-unrecognized。没记原子的闭包（别处拼的、并起来没并 cfg_atoms 的）退回整套生效 cfg（全局写 <名> 或 <名>="<值>"，
                                        特性写 feature="<特性>"@<包名>，测试编译的目标写 test@<目标>）；排序
  toolchain\t<调用方给的一行>          排序
  dependency\t<调用方给的一行>         排序
  external\t<程序名>\t<sha256>\t<--version 那一行>   调用方给；排序。闭包里点名了登记的外部程序（qemu-system-*、herd7、busybox、cargo）
                                        而调用方没给的，拒算。点名 = 一个词（python 里一个串）去掉目录后等于 cargo / herd7 /
                                        busybox，或以 qemu-system- 起头、后面还有字；带空白的串、消息命令（echo、printf、die、bad、
                                        howto）后面的词不算
  unit\t<种类>\t<指纹>                 闭包里每个单元一行（多重集：同种类同指纹的几个单元各一行），排序
  data\t<种类>\t<指纹>                 数据单元，排序
  不带路径、名字与行号；组成清单（单元行）随键打出当索引。
目标写法：<包目录名>:<lib|bin|test|bench|example|build|doctest>:<名>。

入口：<文件>::<限定名>（按后缀认语言：.rs .wgsl .py .sh / .bash）；只写 <文件> 的，Python 是程序入口（main-guard，没有就 module-body），
  shell 是 script-body；Rust 另认 fn 路径 <crate 名>::<模块…>::<名>（不带文件，按 lib 目标的模块树找）。

用法：
  lib_function_units.py dump <根> [--language rust|python|shell|wgsl] [--under <目录>]…
  lib_function_units.py closure <根> <入口> [选项] [--explain]
  lib_function_units.py key <根> <入口> [选项]
  lib_function_units.py controls <根>     V5 / V11 两处变异（crates/mutations.tsv 里原文是那两行的）在临时拷贝上施加，
                                           三个入口的键变不变与 design 报告 2.2 节那张表比；只读根，不写根
  lib_function_units.py --selftest
  选项：--target <目标>  --cfg <名或 名="值">（加进全局 cfg）  --no-default-cfg  --feature <包名>/<特性>  --no-default-features
        --toolchain <行>  --dependency <行>  --external <程序名>=<sha256>=<--version 那一行>
        --wgsl-module <文件>（WGSL 入口拼进同一个模块的别的 .wgsl）  --rust-strings-entry <Rust 入口>（WGSL 里没定义的名字到这个入口闭包的串字面量里找）
        --under <目录>（python / shell 扫哪几处，默认 scripts、.claude/gate.d、.claude/hooks、.claude/scripts 与入口所在目录）
退出码：0 判完；2 用法错；3 算不出（根不在、bash 起不来、清单读不出）；4 拒算（closure / key 的闭包里有拒算：打拒算行，key 不打键行）。

库接口（门禁里 python 写的判法直接 import 它算键，不为算键先编 Rust）：
  rust_units(根) → [单元]；rust_closure(根, 入口, 目标=None, cfg=None) → 闭包；wgsl_units(根)、python_units(根, under)、shell_units(根, under)；
  closure_of(根, 入口, 选项) → 闭包（.members 成员与数据单元、.refusals、.cfg_lines、.programs）；
  closure_key(闭包, toolchain, dependencies, externals) → (键或 None, 拒算)；unit.text() 是进 UnitKey 组成清单的单元行（嵌套路径），
  unit.dump_text() 是 dump 与命令行输出的那一行（unit-graph 的写法）；unit_graph_key(Rust 闭包, 工具链行, 依赖行) → 照 unit-graph 的 reuse_key 算的键（对拍用）。

拒算（3.4，认不出不静默跳过：这一条判定不复用、每趟现判）。形态名：
  Rust   unbalanced-brackets unknown-item cfg-unrecognized include-argument include-missing include-code runtime-path current-exe
         macro-generated-names proc-macro-crate extern-block module-file-missing bin-target-missing
  WGSL   unbalanced-brackets unknown-item wgsl-undefined-name
  Python python-syntax getattr-variable globals-lookup import-unresolved import-module-variable spec-path-unfolded spec-path-missing
         sys-path-unfolded subprocess-path-unfolded subprocess-path-missing subprocess-command-unfolded（命令认不出、命令位置上的程序折不出）
  shell  bash-syntax shell-path-unfolded shell-path-missing shell-path-outside-root shell-command-unfolded（命令位置上的程序折不出）
  公共   external-program-unkeyed inline-script-not-literal（-c 的串、喂给标准输入的脚本折不出，或标准输入从哪来认不出）
  起代码的写法逐种认（shell 里每一个解释器词，python 里每一次起子进程），每一种要么连边、要么拒算：
    解释器 + 脚本：连到那份脚本；脚本之后接着往后认（bash capped.sh N python3 x.py、run-with-memory-cap.sh 上限 bash y.sh 里的第二个解释器）。
    -c 的串、- / /dev/fd/N / bash -s 喂的 here-doc、here-string（含 { … } 3<<'词' 写在组后面的）：正文字面的、或折得出的
      （$变量、$(cat <<'词' …)、$(只打字面行的函数)）当内联脚本：python 的切成单元（不进闭包，穿过它连下去；sys.argv、命令前设的环境变量
      折成那条命令里的词），bash 的切开再认；折不出拒算 inline-script-not-literal。内联 python exec sys.argv[k:] 时第 k 个词起照命令认。
    -m <模块>：仓里的连到程序入口；-m unittest / pytest 点名的测试模块整份连上；标准库与外部的不装仓里的代码。
    --version、--help、python3 -V、bash -n：不跑代码。参数位置上光一个解释器名（command -v python3）：不是要跑的命令。
    命令位置（经 env、nice、timeout、sudo、xargs、exec 这几个包装之后）：仓里的脚本连上；折不出的程序词拒算，
      除了 "$@" "$1" 一类（命令就是调用处那几个词）、"前缀_$1"（按名字前缀连看得见的函数）、"${数组[@]}"（按数组的字面赋值摊开）、
      「$目录/<不带脚本后缀、根里没有的字面尾巴>」与 shutil.which(…)（根外的程序）。
    python 的命令：字面列表、a + b、x if c else y、*展开、本函数里的赋值与 .append / .extend / +=、函数参数（本模块每一处调用给的实参）、
      调仓内函数的返回值都折；shell=True 与 os.system 的串当内联 bash；认不出拒算 subprocess-command-unfolded。
    命令替换 $( … ) 与 `…`、进程替换 <( … ) 里的命令照命令认；[[ … ]]、(( … ))、case 分支的模式、数组字面量 x=( … ) 里的词不当命令。
    文件头登记的 check 判据（# admission: check / # run-condition: check，上游 preflight 开跑前在仓根 bash -c 跑它们）：从声明它的脚本的
      程序入口当内联 bash 连；上游 preflight.py 里跑它们的那一处（bash -c declaration.command）因此不拒算。
  文件级的（括号配不上、认不出的条目、语法错）：Rust 在那份文件进了入口的链接集时算进来，Python / shell 在闭包碰到那份文件时算进来；
  单元级的在那个单元进了闭包时算进来。dump 把扫到的全部列出来。

弄坏开关（FUNCTION_UNITS_BREAK=<项>[,<项>…]，只给自证证红用；自证逐个打开、核点名的那几格转红，见 BREAK_CELLS）：
  rust-open-entry-traits-ignored   去掉 R-开放（入口签名里的 dyn 与 trait 约束不连全部 impl）
  rust-dyn-traits-open-every-implementation 闭包里单元的 dyn Trait 照旧连这个 trait 的全部 impl（CHA）
  rust-constants-dropped           const 与 static 单元不进闭包
  rust-macros-dropped              macro 与 item-macro 单元不进闭包
  rust-trait-impl-edges-dropped    去掉 R-隐式（类型被点名时给它写的 trait impl）
  rust-verus-segments-ignored      verus! 里照普通 Rust 切（签名后第一个 { 当函数体）
  rust-cfg-ignored                 cfg 一律当真
  rust-doc-comments-kept           文档注释进词法单元
  wgsl-raw-text                    include_str! 读的 .wgsl 取整份字节、不切单元
  python-comments-kept             def 的指纹取原文（带注释）
  python-docstrings-kept           文档串不从指纹里去掉
  python-docstring-readers-ignored 读 __doc__ 不收文档串数据单元
  python-getattr-allowed           getattr(x, <变量>) 不拒算
  python-spec-paths-ignored        spec_from_file_location 不折叠、不拒算
  python-attributes-unnarrowed     obj.m 连到全部同名方法（不按闭包里出现过的类收窄）
  shell-heredoc-split              here-doc 正文照样按缩进切
  shell-names-command-position-only 函数名只在命令位置上才连
  shell-unfolded-paths-ignored     折不出的路径不拒算
  shell-raw-text                   脚本体的指纹取原文
  external-programs-unchecked      闭包里点名的外部程序不要求调用方给
  rust-format-captures-ignored     串字面量里的 {名}（format! 一族的内联捕获）不算裸名
  shell-variable-first-line-only   顶层赋值只取第一行（$(cat <<'词' … 词) 的正文不进那个变量单元）
  tree-copy-tails-ignored          「折不出的目录 + 根里存在的相对路径」（临时树、仓副本里的同一份）不按根里那一份认（shell 与 python）
  git-repository-paths-unfolded    git rev-parse --show-toplevel / --git-common-dir 不折成根（shell 的 $(…) 与 python 的 subprocess …stdout）
  python-call-returns-unfolded     不带参数调仓内模块级函数，不按它各处 return 的值折
  shell-redirect-targets-loaded    重定向的目标当成一条命令留下（写数据的路径被当成装代码）
  shell-path-arguments-loaded      参数位置上长得像脚本的路径也当装代码
  python-command-arguments-loaded  subprocess 命令里脚本后面的路径参数也当装代码
  shell-generated-scripts-ignored  单元自己 printf / echo / cat <<词 写出来再起的脚本不认
  shell-inline-positionals-ignored bash -c '…' <名字> <参数…> 里的 $1…$9 不换成外面那几个词
  shell-copied-scripts-ignored     单元自己 cp <源> <它> 拷出来再起的脚本不连到源
  shell-enclosing-locals-ignored   套在外层函数里的函数看不见外层的 local
  shell-cd-first-directory-only    cd 过去的目录折出几个时，相对路径只按 target_of 的老办法找一处
  python-dispatch-tables-ignored   字典字面量里当值写的函数名不连边
  python-module-constants-unfolded import 进来的仓内模块常量 module.NAME 不折
  python-getattr-stdlib-refused    getattr(<标准库或外部模块>, 变量) 照样拒算
  python-subscript-targets-rebind  表[k] = … 当成重新绑定 k
  python-environment-defaults-unfolded os.environ.get(…) or <缺省> 不按缺省折
  python-written-scripts-ignored   单元自己 open(…, "w") 写出字面文本再起的脚本不认
  python-loop-variables-unfolded   for 循环变量不取遍元组 / 列表的元素
  python-imports-first-directory-only 同一个模块名在 sys.path 插的几个目录里各有一份时只连第一份
  python-import-names-unfolded     importlib.import_module(<折得出的名字>) 照样拒算
  python-path-tables-unfolded      路径表（模块级字典字面量）[键] 不折
  python-comprehension-scope-leaks 推导式里的循环变量算成函数里的赋值
  python-directory-wildcards-ignored os.path.join(<仓里的目录>, <折不出的文件名>) 起的脚本不连到那个目录里的每一份
  inline-scripts-ignored           -c 的串、喂给标准输入的脚本不当内联脚本认（回到既不连边也不拒算）
  shell-later-interpreters-ignored 一条命令里只认第一个解释器
  shell-substitutions-ignored      命令替换 $( … ) 与进程替换 <( … ) 里的命令不认
  external-program-words-unfiltered 带空白的串、消息命令（echo / printf / die / bad / howto）后面的词照样当外部程序名
  python-modules-ignored           python3 -m <模块> 不认
  shell-unfolded-commands-allowed  命令位置上折不出的程序不拒算
  shell-wrapped-commands-ignored   包装命令（env、nice、timeout…）后面直接执行的脚本不认
  shell-command-structure-ignored  [[ … ]]、(( … ))、case 模式、数组字面量照普通命令切，{ … } 后面的重定向不管组里的命令
  shell-array-commands-unfolded    "${数组[@]}" 当命令时不按字面赋值摊开
  shell-heredoc-captures-unfolded  $(cat <<'词' …) 与 $(只打字面行的函数) 不折成正文
  declared-checks-ignored          文件头登记的 check 判据不连、上游 preflight 跑它们的那一处照样拒算
  inline-argv-commands-ignored     内联 python exec sys.argv[k:] 时不把后面的词当命令
  python-unfolded-heads-allowed    python 命令位置上折不出的程序不拒算
  python-commands-literal-only     python 的命令只认字面列表与只赋过一次列表的名字（认不出的不拒算）
  python-path-programs-refused     shutil.which(…) 与 os.path.join(<折不出的目录>, <根外的字面尾巴>) 起的程序照样拒算
  python-command-dictionaries-unfolded 命令里的 字典[<字面键>] 不折回字典字面量
  python-dispatch-calls-ignored    分派表[键](…) 不算对表里那几个函数的调用（参数折不到那一处调用给的实参）
  与 crates/singlefs-unit-graph 对齐的那几条（自证里 ALIGNMENT_FIXTURE 与 CLOSURE_RULES_FIXTURE 那几格）：
  rust-module-heads-unlinked       单元不连模块头，也不连把它所在文件连进模块树的 mod 声明
  rust-module-declarations-unlinked 单元不连把它所在文件连进模块树的 mod 声明（模块头照连）
  rust-use-names-resolved          use 单元里的名字往下解（引进的名字全带进闭包）
  rust-use-aliases-unresolved      use a::b as c 里用到 c 的不按 b 再解
  rust-trait-impls-by-trait-arguments R-隐式另按 trait 参数里的类型认 trait impl
  rust-kind-names-hyphenated       输出行的种类名退回旧拼法（macro、item-macro、mod-decl、extern-block、extern-crate、wgsl-entry、wgsl-fn…）
  rust-qualified-names-file-local  输出行的限定名退回文件里的嵌套路径（不带 crate、目标与模块路径前缀）
  rust-doctest-fingerprint-from-rows 文档测试的指纹取代码块原文（随代码块里的注释变）
  rust-module-heads-dropped        不切模块头（行内模块头与文件开头的 #![…]），模块里的单元也就不连它
  rust-data-dump-unlisted          dump 不列数据行
  wgsl-external-names-by-long-identifiers WGSL 外名退回旧口径：串的值与闭包里长于 3 个字符的标识符里出现就算有着落
  wgsl-external-name-sources-unlinked WGSL 入口的闭包不带给外名着落的 Rust 单元（着落不做成边）
  rust-escaped-brace-captures-skipped 格式串里转义的 {{ 后面紧跟的捕获（{{{名}}}）不算裸名
  rust-union-as-keyword            条目级宏调用可能定义的名字里另去掉 union（把上下文关键字当关键字）
  4-3d（定案 2、3）那几条（自证里 unit_divergence_cells 那几格）：
  rust-cfg-atoms-unnarrowed        键里的 cfg 退回整套生效 cfg（不按闭包真引到的收窄）
  rust-cfg-macro-atoms-ignored     cfg 原子只取单元的条件链，不扫词法单元里的 cfg!、语句上的 #[cfg] 与 #[cfg_attr]
  wgsl-defined-name-suffixes-ignored WGSL 外名不认「常量、静态量、条目级宏调用定义的名字或它以 _ 加外名收尾」这一条
  wgsl-prefix-remainders-unchecked WGSL 外名的首段规则不核其余那一段
  wgsl-name-suppliers-not-pulled-in Rust 入口的闭包不把闭包外供外名的单元并进来（着落不做成边）
  wgsl-modules-per-naming-unit     WGSL 模块退回按点名读 .wgsl 的常量的那个单元一组一组拼
  rust-trait-self-paths-ignored    trait 成员里的 Self::x 不落到 trait 自己的成员（退回旧口径：落到自身类型名是 trait 名的 impl）
  rust-doctest-structure-linked    文档测试照普通单元连所挂的条目与那个模块的 use
  rust-use-self-imports-ignored    use 花括号里的 self 不引进 { 前面那一段
  rust-open-by-trait-arguments     R-开放另按 trait 参数里的名字找 impl（dyn Sink 带进 impl Wrap<dyn Sink> for H）
  rust-blanket-impls-ignored       R-方法、R-路径不放行 blanket impl，trait 头进闭包不带它的 blanket impl
  4-3h 那两条（自证里 unit_divergence_cells 里 4-3k 那两格之前的两格）：
  rust-underscore-items-named      const _ / static _ 照名字 _ 进 R-裸名的着落（let _ = …、_ => 里的 _ 把它们拉进闭包）
  rust-includes-outside-root-refused include_str! 一类读根外的文件一律拒算 include-missing（读得到也拒）
  4-3k 那两条（自证里 unit_divergence_cells 末尾两格）：
  rust-blanket-self-parameter-only blanket 退回旧判法：自身类型里头一个名字是泛型参数才算（Box<T>、Option<Box<T>>、Sealed<T> 的 impl 不算）
  wgsl-other-file-definitions-unlinked WGSL 外名在链接集里别的 include_str! 读进来的 .wgsl 里有定义时，不把读它的 Rust 单元与那份 .wgsl 拉进闭包

折叠认的写法（上面各开关管的那几条；推的，没有夹具钉住的不写在这里）：
  临时树、仓副本、worktree 里的「$目录/<相对路径>」「os.path.join(目录, <相对路径>)」按根里同一相对路径那一份认（至少两段、在根里存在）：
    拷进去的是它，替身只会让闭包多收、不会少收；尾巴在根里不在的照旧拒算。
  git rev-parse --git-common-dir 折成 <根>/.git：根是 worktree 时运行期指的是主仓，库只看给它的这一个根，按根里同一相对路径取，那一份不在照 missing 拒算。
  ${X:-缺省} 与 os.environ.get("X") or <缺省>：环境变量那一边看不见，按缺省值折（调用方给的值要与缺省同一份，靠写脚本的人守）。
  单元自己写出来的脚本（printf / echo / cat <<词 重定向到它、open(…, "w") 写字面文本）：代码就是单元里那几行；cat <<词 的正文接着看它装了什么，
    printf / echo 写进去的那几行不当代码看；写它的有一处别的写法（tee、cp、别的命令重定向）就不算。
  os.path.join(<仓里的目录>, <折不出的文件名>)：起的是那个目录里调用方点名的一份，目录里每一份同后缀的都连（判定对象就是那一层的工具：hook-swap、gate-structure-check）。

管不到的（推的，没有夹具钉住）：
  Rust：外部 crate 的泛型函数构造、回传工作区类型（Default::default() 经外部泛型、serde 反序列化）；宏展开出来的条目按宏单元里出现过的标识符认，
    展开出的名字由 concat_idents! / paste! 拼出来的拒算，别的拼法认不出；`mod` 的 #[path] 在内联 mod 里按「所在文件目录 + 内联路径」算。
  Python：exec / eval 的字符串、按字符串名字派活（{"x": "函数名"} 再 globals()[…] 以外的写法）、类属性之外的动态属性赋值；
    模块级语句只在那个模块有单元进闭包时连进来（import 时的副作用照算），模块被别的进程起时连的是 main-guard 与 module-body。
  起代码的写法（推的，有夹具钉住的是上面「起代码的写法逐种认」那几条）：内联 bash 串里自己赋的变量只在这个串里看得见（串之外的赋值照本单元的折）；
    跨脚本的参数不传（被起的脚本里 "$1" 起的程序，按 "$1" 一类认）；根外程序按「不带脚本后缀、根里没有这一截」认，同名的替身不分。
  shell：eval 的字符串、按变量名间接调用（"${handler}"）只在那个变量的值是本闭包可见的函数名、而这个名字作为一个词出现在闭包里时才连上；
    here-doc 的起止按规范文本里的 <<词 与单独一行的词认，引号里长得像 <<词 而后面又恰好有单独一行同一个词的会多跳一段。
"""
import ast
import bisect
import collections
import copy
import glob
import hashlib
import os
import posixpath
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

KEY_FORM = "singlefs-function-units-v1"
BREAK_VARIABLE = "FUNCTION_UNITS_BREAK"
REFUSED_EXIT = 4
SHELL_TIMEOUT_SECONDS = 600
SNIPPET_CHARACTERS = 160
FOLD_LIMIT = 64
SKIPPED_DIRECTORIES = frozenset((".git", "target", "__pycache__", "node_modules"))
DEFAULT_SCRIPT_DIRECTORIES = ("scripts", ".claude/gate.d", ".claude/hooks", ".claude/scripts")
EXTERNAL_PROGRAMS = ("herd7", "busybox", "cargo")
EXTERNAL_PROGRAM_PREFIXES = ("qemu-system-",)
PYTHON_INTERPRETERS = frozenset(("python3", "python"))
SHELL_INTERPRETERS = frozenset(("bash", "sh"))
INLINE_FORM = "inline-script-not-literal"
COMMAND_FORM = "subprocess-command-unfolded"
# 只打印的命令：后面的词是消息，不是程序名（外部程序那一条不看它们）
MESSAGE_HEADS = frozenset(("echo", "printf", "die", "bad", "howto"))
# 包装命令：名字 → (带一个值的选项, 选项之后先跳过几个操作数)；包装之后的那个词在命令位置上
COMMAND_WRAPPERS = {
    "env": (frozenset(("-u", "--unset", "-C", "--chdir")), 0),
    "nice": (frozenset(("-n", "--adjustment")), 0),
    "ionice": (frozenset(("-c", "--class", "-n", "--classdata", "-p", "-P", "-u")), 0),
    "timeout": (frozenset(("-s", "--signal", "-k", "--kill-after")), 1),
    "nohup": (frozenset(), 0),
    "setsid": (frozenset(), 0),
    "stdbuf": (frozenset(("-i", "-o", "-e")), 0),
    "taskset": (frozenset(), 1),
    "chrt": (frozenset(), 1),
    "sudo": (frozenset(("-u", "-g", "-C", "-D", "-h", "-p", "-r", "-t", "-U", "-T")), 0),
    "xargs": (frozenset(("-a", "-d", "-E", "-I", "-L", "-n", "-P", "-s", "--arg-file", "--delimiter", "--max-args", "--max-procs", "--max-lines",
                         "--max-chars")), 0),
    "exec": (frozenset(("-a",)), 0),
    "command": (frozenset(), 0),
    "time": (frozenset(), 0),
}
PYTHON_VALUE_OPTIONS = frozenset(("-X", "-W", "--check-hash-based-pycs"))
SHELL_VALUE_OPTIONS = frozenset(("-o", "-O", "+o", "+O", "--rcfile", "--init-file"))
TEST_RUNNER_MODULES = frozenset(("unittest", "pytest"))
PREFLIGHT_MODULE = "singlefs-ai-sop/scripts/preflight.py"
DECLARED_CHECK = re.compile(r"^#\s*(?:admission|run-condition):\s*check\s+(.*?)(?:\s+::\s.*)?$")


def declared_checks(root, path, rows):
    """一份脚本文件头（#! 之后、第一行代码之前的注释块）里登记的 check 判据 → [InlineScript…]：preflight 开跑前在仓根用 bash -c 跑它们，
    环境里有 PREFLIGHT_SCRIPT 与 PREFLIGHT_SCRIPT_DIRECTORY（preflight-discipline.md「写法」）。"""
    out = []
    absolute = posixpath.join(root, path)
    environment = (("PREFLIGHT_SCRIPT", frozenset((absolute,))), ("PREFLIGHT_SCRIPT_DIRECTORY", frozenset((posixpath.dirname(absolute),))))
    for number, row in enumerate(rows, 1):
        if number == 1 and row.startswith("#!"):
            continue
        stripped = row.strip()
        if stripped and not stripped.startswith("#"):
            break
        found = DECLARED_CHECK.match(stripped)
        if found and found.group(1).strip():
            out.append(InlineScript("shell", (found.group(1).strip(),), (None,), (root,), path, number, snippet_of(row), environment))
    return out
SHELL_POSITIONAL_WORD = re.compile(r'^"?\$(?:\{(?:[@*]|[1-9][0-9]*)(?::[^}]*)?\}|[@*1-9])"?$')


def external_program_name(value, breaks):
    """一个词的字面值 → 它点名的登记外部程序名（等于 cargo / herd7 / busybox，或以 qemu-system- 起头、后面还有字）；不是 → None。
    词里有空白的（消息串、带空格的串）不算，只是前缀本身（qemu-system-）也不算。"""
    if not value:
        return None
    base = posixpath.basename(value)
    if "external-program-words-unfiltered" in breaks:
        return base if base in EXTERNAL_PROGRAMS or base.startswith(EXTERNAL_PROGRAM_PREFIXES) else None
    if any(character.isspace() for character in value):
        return None
    if base in EXTERNAL_PROGRAMS:
        return base
    if any(base.startswith(prefix) and len(base) > len(prefix) for prefix in EXTERNAL_PROGRAM_PREFIXES):
        return base
    return None


def command_positions(values):
    """一条命令里在命令位置上的那几个下标：0 起，经包装命令（env、nice、timeout、sudo、xargs、exec…）跳过它们的选项与操作数之后的那一个。
    values 是逐词字面值（折不出的是 None：它就是命令位置上的词，后面不再往下认）；command -v / -V 是查询，后面不算命令位置。"""
    positions, position = [], 0
    while position < len(values):
        positions.append(position)
        value = values[position]
        name = posixpath.basename(value) if value else None
        if name not in COMMAND_WRAPPERS:
            break
        value_options, operands = COMMAND_WRAPPERS[name]
        position += 1
        while position < len(values):
            option = values[position]
            if option is None:
                break
            if name == "command" and option in ("-v", "-V"):
                return positions
            if option == "--":
                position += 1
                break
            if name == "env" and ASSIGNMENT_WORD.match(option):
                position += 1
                continue
            if option.startswith("-") and len(option) > 1:
                position += 2 if option in value_options else 1
                continue
            break
        position += operands
    return positions


def interpreter_mode(values, position, python):
    """解释器词（values[position]）之后：→ (写法, 下标)。写法：script（下标是脚本那个词）、inline（-c，下标是 -c）、module（python 的 -m）、
    stdin（- 或 bash 的 -s：脚本从标准输入来）、syntax-only（bash -n：只查语法不跑）、no-run（--version、--help、python3 -V：打完就退）、
    bare（后面没有词了，下标是末尾）。
    合写的短选项（bash -ec、python3 -Bc）按最后一个字母认；带值的选项（bash -o x、python3 -X y）跳两格；折不出的词当脚本。"""
    argument = position + 1
    value_options = PYTHON_VALUE_OPTIONS if python else SHELL_VALUE_OPTIONS
    while argument < len(values):
        option = values[argument]
        if option is None:
            return "script", argument
        if option == "-":
            return "stdin", argument
        if option in ("--version", "--help") or (option[:1] == "-" and len(option) > 1 and set(option[1:]) <= ({"V", "h"} if python else set())):
            return "no-run", argument
        if option in value_options:
            argument += 2
            continue
        if option.startswith("--"):
            argument += 1
            continue
        if option[:1] in "-+" and len(option) > 1:
            letters = option[1:]
            if letters.isalnum():
                last = letters[-1]
                if last == "c":
                    return "inline", argument
                if python and last == "m":
                    return "module", argument
                if not python and last == "s":
                    return "stdin", argument
                if not python and "n" in letters:
                    return "syntax-only", argument
            argument += 1
            continue
        return "script", argument
    return "bare", argument


ARGV_SPAWNS = frozenset(("os.execvp", "os.execv", "os.execve", "os.execvpe", "subprocess.run", "subprocess.call", "subprocess.check_call",
                         "subprocess.check_output", "subprocess.Popen"))


def argv_command_offsets(text):
    """python 内联脚本正文里把 sys.argv[k:] 当命令起的（os.execvp(sys.argv[1], sys.argv[1:])、subprocess.run(sys.argv[2:])）→ 那几个 k：
    起它的那条命令里第 k 个参数起就是一条命令，在起它的那边照命令认。正文解析不了 → 空。"""
    try:
        tree = ast.parse(text)
    except (SyntaxError, ValueError):
        return set()
    aliases = {"os": "os", "subprocess": "subprocess"}
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            for alias in node.names:
                aliases[alias.asname or alias.name] = alias.name
    offsets = set()
    for node in ast.walk(tree):
        if not (isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and isinstance(node.func.value, ast.Name)):
            continue
        if f"{aliases.get(node.func.value.id, node.func.value.id)}.{node.func.attr}" not in ARGV_SPAWNS:
            continue
        for argument in node.args[:2]:
            if isinstance(argument, ast.Subscript) and isinstance(argument.value, ast.Attribute) and argument.value.attr == "argv" and \
                    isinstance(argument.slice, ast.Slice) and isinstance(argument.slice.lower, ast.Constant) and type(argument.slice.lower.value) is int \
                    and argument.slice.upper is None and argument.slice.step is None and argument.slice.lower.value >= 1:
                offsets.add(argument.slice.lower.value)
    return offsets


def descriptor_of(script):
    """脚本那个词是 /dev/fd/<N> 或 /dev/stdin（从文件描述符读脚本）→ N；不是 → None。"""
    if script == "/dev/stdin":
        return 0
    found = re.match(r"^/dev/fd/([0-9]+)$", script or "")
    return int(found.group(1)) if found else None


InlineScript = collections.namedtuple("InlineScript", "language texts argv bases file line snippet environment", defaults=((),))
InlineScript.__doc__ = """内联脚本（python3 -c / bash -c 的串、喂给解释器标准输入的 here-doc、here-string）：texts 是它可能取到的几份正文，
argv 是 sys.argv[1:] / $1… 逐个折出来的值（frozenset，折不出 None），bases 是相对路径与 import 先试的目录（当时的工作目录），
file / line / snippet 是起它的那一行（拒算落在那里），environment 是同一条命令里给它设的环境变量（(名字, 取值或 None)…：
X=… python3、env X=… python3）。"""
OPAQUE = object()
POSITIONAL = "\0positional"   # 内联脚本的参数是 "$@" / "$1" 一类：取值就是调用处那几个词（那边已照命令认过）


class ValueElement:
    """命令里一个已经折好的元素（内联脚本的 sys.argv[k:] 摊开的那几个）：values 是 frozenset 或 None。"""
    __slots__ = ("values",)

    def __init__(self, values):
        self.values = values


class FunctionUnitsError(Exception):
    """算不出：根不在、bash 起不来、清单读不出。调用方按「判不出来」处置（要跑），退出码 3。"""


class UsageError(Exception):
    """参数不对，退出码 2。"""


def active_breaks():
    return frozenset(name for name in os.environ.get(BREAK_VARIABLE, "").split(",") if name)


def sha256_text(text):
    return hashlib.sha256(text.encode("utf-8", "surrogateescape")).hexdigest()


def sha256_bytes(content):
    return hashlib.sha256(content).hexdigest()


def snippet_of(text):
    return " ".join(str(text).split())[:SNIPPET_CHARACTERS]


class Refusal(collections.namedtuple("Refusal", "form file line snippet")):
    __slots__ = ()

    def text(self):
        return f"refused\t{self.form}\t{self.file}:{self.line}\t{self.snippet}"


UNIT_SLOTS = ("language", "kind", "name", "file", "first_line", "last_line", "fingerprint", "data",
              "short", "container", "conditions", "scope", "public", "first_token", "last_token",
              "plain", "methods", "qualified", "dyn_traits", "references", "data_references", "refusals",
              "self_types", "trait_name", "is_trait_impl", "blanket", "implicit_keys", "imports", "aliases", "import_glob", "defines",
              "declared", "node", "nodes", "facts", "body", "module", "cache", "full", "header", "module_head", "doc_module", "cfg_leaves",
              "open_keys")

# 弄坏开关 rust-kind-names-hyphenated 下输出行退回的旧拼法（与 crates/singlefs-unit-graph 对齐之前 python 这一份的写法）
HYPHENATED_KIND_NAMES = {"macro_rules": "macro", "item_macro": "item-macro", "mod": "mod-decl", "extern_block": "extern-block",
                         "extern_crate": "extern-crate", "broadcast_group": "broadcast-group", "mod_head": "mod-head",
                         "wgsl_fn": "wgsl-fn", "wgsl_compute": "wgsl-entry", "wgsl_const": "wgsl-const", "wgsl_override": "wgsl-override",
                         "wgsl_struct": "wgsl-struct", "wgsl_alias": "wgsl-alias", "wgsl_var": "wgsl-var", "wgsl_directive": "wgsl-directive",
                         "wgsl_const_assert": "wgsl-const-assert"}


class Unit:
    __slots__ = UNIT_SLOTS

    def __init__(self, language, kind, name, file, first_line, last_line, fingerprint, data=False):
        for slot in UNIT_SLOTS:
            setattr(self, slot, None)
        self.language, self.kind, self.name, self.file = language, kind, name, file
        self.first_line, self.last_line, self.fingerprint, self.data = first_line, last_line, fingerprint, data
        self.short = name
        self.refusals = []

    def output_kind(self):
        """输出行里的种类名：Rust 与 WGSL 照 crates/singlefs-unit-graph 的拼法（unit.kind 本身就是那一种）；弄坏开关下退回旧拼法。"""
        if self.language in ("rust", "wgsl") and "rust-kind-names-hyphenated" in active_breaks():
            return HYPHENATED_KIND_NAMES.get(self.kind, self.kind)
        return self.kind

    def output_name(self):
        """输出行里的限定名：Rust 与 WGSL 带 crate 与模块路径前缀（full，照 crates/singlefs-unit-graph）；python、shell 与数据单元是 name。
        弄坏开关 rust-qualified-names-file-local 下 Rust 与 WGSL 退回文件里的嵌套路径（name）。"""
        if self.full is not None and "rust-qualified-names-file-local" not in active_breaks():
            return self.full
        return self.name

    def text(self):
        """进 UnitKey 组成清单与各判法按（种类, 限定名, 文件）认单元用的那一行：限定名是文件里的嵌套路径（name）。"""
        return f"{self.kind}\t{self.name}\t{self.file}:{self.first_line}-{self.last_line}\t{self.fingerprint}"

    def dump_text(self):
        """dump 与命令行 closure / key 打的那一行：Rust 与 WGSL 照 crates/singlefs-unit-graph 的写法（种类名、带前缀的限定名），别的语言同 text()。"""
        return f"{self.output_kind()}\t{self.output_name()}\t{self.file}:{self.first_line}-{self.last_line}\t{self.fingerprint}"

    def sort_key(self):
        return (self.file, self.first_line, self.kind, self.name)


def data_file_unit(root, path):
    with open(os.path.join(root, path), "rb") as handle:
        content = handle.read()
    rows = content.count(b"\n") + (0 if content.endswith(b"\n") or not content else 1)
    return Unit("data", "data-file", path, path, 1, max(rows, 1), sha256_bytes(content), data=True)


def files_under(root, suffixes, under=None):
    """根下（或 under 给的几处下）后缀对得上的文件，相对根、排好序；跳过 .git、target、__pycache__、node_modules。"""
    tops = [os.path.join(root, place) for place in under] if under else [root]
    found = set()
    for top in tops:
        if os.path.isfile(top):
            if top.endswith(suffixes):
                found.add(os.path.relpath(top, root).replace(os.sep, "/"))
            continue
        for directory, names, files in os.walk(top):
            names[:] = sorted(name for name in names if name not in SKIPPED_DIRECTORIES)
            for name in files:
                if name.endswith(suffixes):
                    found.add(os.path.relpath(os.path.join(directory, name), root).replace(os.sep, "/"))
    return sorted(found)


def inside_root(root, absolute):
    """绝对路径 → 相对根的路径（在根里面时），否则 None。"""
    real_root = os.path.normpath(os.path.abspath(root))
    candidate = os.path.normpath(absolute)
    if candidate == real_root or candidate.startswith(real_root + os.sep):
        return os.path.relpath(candidate, real_root).replace(os.sep, "/")
    return None


# ── Rust 与 WGSL 的词法 ─────────────────────────────────────────────────────

IDENT, LITERAL, LIFETIME, PUNCT = "ident", "literal", "lifetime", "punct"
OPENING_OF = {")": "(", "]": "[", "}": "{"}


def block_document_rows(body):
    return [re.sub(r"^\s*\*? ?", "", row, count=1) for row in body.split("\n")]


def is_identifier_start(character):
    """标识符起头：ASCII 字母、下划线，或任何非 ASCII 字符（Rust 那一份 lexer.rs 的 is_identifier_start：字节 >= 0x80 都算）。"""
    return (character.isascii() and (character.isalpha() or character == "_")) or ord(character) >= 0x80


def is_identifier_continue(character):
    """标识符续字符：ASCII 字母数字、下划线，或任何非 ASCII 字符（lexer.rs 的 is_identifier_continue）。"""
    return (character.isascii() and (character.isalnum() or character == "_")) or ord(character) >= 0x80


def rust_lex(text):
    """→ (kinds, texts, lines, documents)；documents 是 [(行, outer|inner, [各行正文], 下一个词法单元的下标, [各行原文])]：
    各行正文去掉了块注释每行开头的 * 号，各行原文是注释记号之后的原样（与 crates/singlefs-unit-graph 的 documentation_comment_of 同一份，文档测试取它）。"""
    kinds, texts, lines, documents = [], [], [], []
    count = len(text)
    newlines = [match.start() for match in re.finditer("\n", text)]

    def line_at(position):
        return bisect.bisect_left(newlines, position) + 1

    def quoted_end(start, quote):
        j = start + 1
        while j < count:
            character = text[j]
            if character == "\\":
                j += 2
                continue
            if character == quote:
                return j + 1
            j += 1
        return count

    def raw_end(start):
        j, hashes = start, 0
        while j < count and text[j] == "#":
            hashes += 1
            j += 1
        if j >= count or text[j] != '"':
            return None
        closing = '"' + "#" * hashes
        found = text.find(closing, j + 1)
        return count if found < 0 else found + len(closing)

    def emit(kind, start, end):
        kinds.append(kind)
        texts.append(text[start:end])
        lines.append(line_at(start))

    i = 0
    while i < count:
        character = text[i]
        if character.isspace():
            i += 1
            continue
        following = text[i + 1] if i + 1 < count else ""
        if character == "/" and following == "/":
            end = text.find("\n", i)
            end = count if end < 0 else end
            body = text[i:end]
            if body.startswith("///") and not body.startswith("////"):
                documents.append((line_at(i), "outer", [body[3:]], len(texts), [body[3:]]))
            elif body.startswith("//!"):
                documents.append((line_at(i), "inner", [body[3:]], len(texts), [body[3:]]))
            i = end
            continue
        if character == "/" and following == "*":
            depth, j = 0, i
            while j < count:
                if text.startswith("/*", j):
                    depth += 1
                    j += 2
                elif text.startswith("*/", j):
                    depth -= 1
                    j += 2
                    if depth == 0:
                        break
                else:
                    j += 1
            body = text[i:j]
            style = None
            if body.startswith("/**") and not body.startswith("/***") and body != "/**/":
                style = "outer"
            elif body.startswith("/*!"):
                style = "inner"
            if style is not None:
                if style == "outer":
                    raw = body[3:-2] if body.endswith("*/") and len(body) >= 5 else ""
                else:
                    raw = body[3:-2] if body.endswith("*/") and len(body) >= 5 else body[3:]
                documents.append((line_at(i), style, block_document_rows(body[3:-2]), len(texts), raw.split("\n")))
            i = j
            continue
        start = i
        if character in "bc" and following == "r":
            end = raw_end(i + 2)
            if end is not None:
                emit(LITERAL, start, end)
                i = end
                continue
        if character == "r" and following in ('#', '"'):
            if following == "#" and i + 2 < count and is_identifier_start(text[i + 2]):
                j = i + 2
                while j < count and is_identifier_continue(text[j]):
                    j += 1
                # 原始标识符的词按源码原样取、带着 r#（与 Rust 那一份 lexer.rs 同一条：词是源码里的那一段）
                kinds.append(IDENT)
                texts.append(text[i:j])
                lines.append(line_at(i))
                i = j
                continue
            end = raw_end(i + 1)
            if end is not None:
                emit(LITERAL, start, end)
                i = end
                continue
        if character in "bc" and following in ('"', "'"):
            end = quoted_end(i + 1, following)
            emit(LITERAL, start, end)
            i = end
            continue
        if character == '"':
            end = quoted_end(i, '"')
            emit(LITERAL, start, end)
            i = end
            continue
        if character == "'":
            if following == "\\":
                end = quoted_end(i, "'")
                emit(LITERAL, start, end)
                i = end
                continue
            if i + 2 < count and text[i + 2] == "'":
                emit(LITERAL, start, i + 3)
                i += 3
                continue
            j = i + 1
            while j < count and (text[j].isalnum() or text[j] == "_"):
                j += 1
            emit(LIFETIME, start, j)
            i = j
            continue
        if is_identifier_start(character):
            j = i
            while j < count and is_identifier_continue(text[j]):
                j += 1
            emit(IDENT, start, j)
            i = j
            continue
        if character.isdigit():
            j = i
            hexadecimal = text.startswith(("0x", "0X"), i)
            while j < count and (text[j].isalnum() or text[j] == "_"
                                 or (text[j] == "." and j + 1 < count and text[j + 1].isdigit() and not hexadecimal)
                                 or (text[j] in "+-" and text[j - 1] in "eE" and not hexadecimal)):
                j += 1
            emit(LITERAL, start, j)
            i = j
            continue
        emit(PUNCT, start, i + 1)
        i += 1
    return kinds, texts, lines, documents


def match_brackets(kinds, texts):
    """→ (左括号下标 → 右括号下标, None)，配不上时 → (None, 出错的下标)。"""
    pairs, stack = {}, []
    for index, (kind, text) in enumerate(zip(kinds, texts)):
        if kind != PUNCT:
            continue
        if text in ("(", "[", "{"):
            stack.append(index)
        elif text in (")", "]", "}"):
            if not stack or texts[stack[-1]] != OPENING_OF[text]:
                return None, index
            pairs[stack.pop()] = index
    if stack:
        return None, stack[-1]
    return pairs, None


def rust_string_value(token):
    """串字面量的值（普通串、原始串、字节串、C 串）；不是串回 None。"""
    match = re.match(r'^[bc]?r(#*)"(.*)"\1$', token, re.S)
    if match:
        return match.group(2)
    if token[:2] in ('b"', 'c"'):
        token = token[1:]
    if len(token) >= 2 and token[0] == '"' and token[-1] == '"':
        escapes = {"n": "\n", "t": "\t", "\\": "\\", '"': '"', "'": "'", "0": "\0", "r": "\r", "\n": ""}
        return re.sub(r"\\(.)", lambda found: escapes.get(found.group(1), found.group(1)), token[1:-1], flags=re.S)
    return None


FORMAT_CAPTURE = re.compile(r"(?<!\{)\{([A-Za-z_][A-Za-z0-9_]*)(?=[:}])")


def format_captures(literal):
    """串字面量里 {名} / {名:…} 形式的格式捕获（照 crates/singlefs-unit-graph 的 add_format_captures 逐字节扫）：{{ 是转义、跳过两个字节，
    转义之后紧跟的 { 照样开一个捕获（{{{名}}} 里的名算）。弄坏开关 rust-escaped-brace-captures-skipped 下退回 (?<!\\{)\\{ 的口径（{{{名}}} 不算）。"""
    if "rust-escaped-brace-captures-skipped" in active_breaks():
        return set(FORMAT_CAPTURE.findall(literal))
    names, position, count = set(), 0, len(literal)
    while position < count:
        if literal[position] != "{":
            position += 1
            continue
        if position + 1 < count and literal[position + 1] == "{":
            position += 2
            continue
        start = end = position + 1
        while end < count and literal[end].isascii() and (literal[end].isalnum() or literal[end] == "_"):
            end += 1
        if end > start and not literal[start].isdigit() and end < count and literal[end] in "}:":
            names.add(literal[start:end])
        position = max(end, position + 1)
    return names


def token_references(kinds, texts, low, high):
    """→ (裸名, 方法名, (限定, 名), dyn trait)：R-裸名、R-方法、R-路径读前三样，dyn trait 只在弄坏开关 rust-dyn-traits-open-every-implementation 下读；
    字段访问 .f（后面不接 ( 或 ::）不收。
    串字面量里的 {名} / {名:…}（format! 一族的内联捕获）也算裸名：不认它，format!("{CONST}") 读的常量就漏了。"""
    plain, methods, qualified, dyn_traits = set(), set(), set(), set()
    count = len(texts)
    captures = "rust-format-captures-ignored" not in active_breaks()

    def text_at(index):
        return texts[index] if 0 <= index < count else ""

    for i in range(low, high):
        if kinds[i] == LITERAL and captures and "{" in texts[i] and texts[i][:1] in ('"', "r"):
            plain.update(format_captures(texts[i]))
            continue
        if kinds[i] != IDENT:
            continue
        name = texts[i]
        previous = text_at(i - 1)
        if previous == "." and text_at(i - 2) != ".":
            following = text_at(i + 1)
            if following == "(" or (following == ":" and text_at(i + 2) == ":"):
                methods.add(name)
            continue
        if previous == ":" and text_at(i - 2) == ":":
            if i >= 3 and kinds[i - 3] == IDENT:
                qualified.add((texts[i - 3], name))
            elif text_at(i - 3) == ">" and text_at(i - 5) == "as" and i >= 4 and kinds[i - 4] == IDENT:
                qualified.add((texts[i - 4], name))
        if name == "dyn":
            j = i + 1
            while j + 2 < count and kinds[j] == IDENT and texts[j + 1] == ":" and texts[j + 2] == ":":
                j += 3
            if j < count and kinds[j] == IDENT:
                dyn_traits.add(texts[j])
        plain.add(name)
    return plain, methods, qualified, dyn_traits


# ── Rust：一份文件切成条目单元 ───────────────────────────────────────────────

RUST_QUALIFIERS = frozenset(("async", "unsafe", "default", "safe", "open", "closed", "spec", "proof", "exec", "broadcast",
                             "tracked", "ghost", "uninterp"))
RUST_ITEM_STARTS = frozenset(("pub", "fn", "struct", "enum", "union", "impl", "trait", "mod", "use", "const", "static", "type",
                              "macro_rules", "extern", "auto")) | RUST_QUALIFIERS
RUST_NAMED_KINDS = frozenset(("fn", "const", "static", "struct", "enum", "union", "type", "trait", "macro_rules", "broadcast_group"))
RUST_TYPE_KINDS = frozenset(("struct", "enum", "union", "type"))
# 只是结构、里面的名字不往下解的种类（R-结构；与 crates/singlefs-unit-graph 的 UnitKind::resolves_its_own_names 为假的那几种同一份）
RUST_STRUCTURE_ONLY_KINDS = frozenset(("use", "mod", "mod_head", "extern_crate"))
RUST_ITEM_MACROS_HOLDING_ITEMS = frozenset(("thread_local",))
PRIMITIVE_TYPES = frozenset(("u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64",
                             "bool", "char", "str", "String", "Vec", "Option", "Result", "Box", "Self"))
SELF_TYPE_SKIPPED = frozenset(("dyn", "mut", "const", "impl", "for", "where", "as", "unsafe"))
RUST_KEYWORDS = frozenset(("as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false", "fn", "for",
                           "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static",
                           "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while", "macro_rules"))
# union 是上下文关键字（fn union() 合法），不在 RUST_KEYWORDS 里（与 crates/singlefs-unit-graph 的 RUST_KEYWORDS 同一份）；弄坏开关 rust-union-as-keyword 下另去掉它
UNION_KEYWORD = frozenset(("union",))


# #[test] 带来的条件（只在 cfg(test) 下编）的原文标记：键里的 cfg 原子只算 #[cfg] / #[cfg_attr] / cfg! 真引到的，它不算
TEST_ATTRIBUTE_RAW = "#[test]"


def cfg_tree_leaves(tree):
    """cfg 谓词树 → 叶子（("name", 名) 与 ("pair", 键, 值)），由左到右。"""
    if tree[0] in ("all", "any", "not"):
        return [leaf for item in tree[1] for leaf in cfg_tree_leaves(item)]
    return [tree]


def cfg_atom_text(leaf):
    """叶子写成原子：无值名字写名字，键值写 键="值"（与 crates/singlefs-unit-graph 的 atoms 同一种写法）。"""
    return leaf[1] if leaf[0] == "name" else f'{leaf[1]}="{leaf[2]}"'


def cfg_predicates_in_tokens(kinds, texts, lines, pairs, low, high):
    """一段词法单元里写出来的 cfg 谓词 → [(树或 None, 原文, 行)]：每个属性 #[…] / #![…] 里的 cfg(谓词)、cfg_attr(条件, 属性…) 的条件与
    它属性里套的 cfg(…) / cfg_attr(…)，以及 cfg!(谓词)（括号写成 ( [ { 都认）。树是 None 的是写不成谓词的原文。"""
    out = []

    def text(index):
        return texts[index] if 0 <= index < len(texts) else ""

    def predicate(start, end):
        raw = " ".join(texts[start:end])
        try:
            tree = parse_cfg_tokens(kinds[start:end], texts[start:end])
        except ValueError:
            tree = None
        out.append((tree, raw, lines[start] if start < len(lines) else (lines[-1] if lines else 1)))

    def parts(start, end):
        found, begin, index = [], start, start
        while index < end:
            if text(index) in ("(", "[", "{") and index in pairs:
                index = pairs[index] + 1
                continue
            if text(index) == ",":
                found.append((begin, index))
                begin = index + 1
            index += 1
        found.append((begin, end))
        return [part for part in found if part[0] < part[1]]

    def attribute(start, end):
        if text(start) == "cfg" and text(start + 1) == "(" and (start + 1) in pairs:
            predicate(start + 2, pairs[start + 1])
        elif text(start) == "cfg_attr" and text(start + 1) == "(" and (start + 1) in pairs:
            pieces = parts(start + 2, pairs[start + 1])
            if pieces:
                predicate(*pieces[0])
                for piece in pieces[1:]:
                    attribute(*piece)

    index = low
    while index < high:
        if text(index) == "#" and text(index + 1) == "[" and (index + 1) in pairs:
            attribute(index + 2, pairs[index + 1])
        elif text(index) == "#" and text(index + 1) == "!" and text(index + 2) == "[" and (index + 2) in pairs:
            attribute(index + 3, pairs[index + 2])
        elif kinds[index] == IDENT and text(index) == "cfg" and text(index + 1) == "!" and text(index + 2) in ("(", "[", "{") and (index + 2) in pairs:
            predicate(index + 3, pairs[index + 2])
        index += 1
    return out


def parse_cfg_tokens(kinds, texts):
    """cfg 谓词的词法单元 → 树：("name", 名) / ("pair", 键, 值) / ("all"|"any"|"not", [子树…])；认不出抛 ValueError。"""
    position = 0
    total = len(texts)

    def expression():
        nonlocal position
        if position >= total or kinds[position] != IDENT:
            raise ValueError("要一个名字")
        name = texts[position]
        position += 1
        if name in ("all", "any", "not") and position < total and texts[position] == "(":
            position += 1
            items = []
            while True:
                if position >= total:
                    raise ValueError("括号没收")
                if texts[position] == ")":
                    position += 1
                    break
                items.append(expression())
                if position < total and texts[position] == ",":
                    position += 1
                elif position < total and texts[position] == ")":
                    continue
                else:
                    raise ValueError("要逗号或右括号")
            if name == "not" and len(items) != 1:
                raise ValueError("not 只收一项")
            return (name, items)
        if position < total and texts[position] == "=":
            position += 1
            if position >= total or kinds[position] != LITERAL:
                raise ValueError("= 后面要串")
            value = rust_string_value(texts[position])
            position += 1
            if value is None:
                raise ValueError("= 后面要串")
            return ("pair", name, value)
        if position < total and texts[position] == "(":
            raise ValueError(f"认不出 {name}(…)")
        return ("name", name)

    tree = expression()
    if position != total:
        raise ValueError("后面还有")
    return tree


class RustFile:
    def __init__(self, root, path, text, crate_directory):
        self.root, self.path = root, path
        self.directory = posixpath.dirname(path)
        self.crate_directory = crate_directory
        self.breaks = active_breaks()
        self.source_rows = text.split("\n")
        self.kinds, self.texts, self.lines, self.documents = rust_lex(text)
        self.count = len(self.texts)
        self.units, self.refusals, self.mod_declarations, self.doctests = [], [], [], []
        # 文档测试挂在哪（照 crates/singlefs-unit-graph）：条目的外文档注释，与模块开头（文件开头、行内模块体开头到内属性收尾）的内文档注释
        self.module_documents = []      # [(内文档注释可落的下标区间 低, 高, 文件里的行内模块路径)]
        self.current_header, self.current_module_head = 0, None
        self.pairs, bad = match_brackets(self.kinds, self.texts)
        if self.pairs is None:
            line = self.lines[bad] if bad is not None and bad < self.count else 1
            self.refusals.append(Refusal("unbalanced-brackets", path, line, self.row(line)))
            self.pairs = {}
            return
        inner_end, inner_conditions = self.inner_attributes(0, self.count)
        if inner_end > 0 and "rust-module-heads-dropped" not in self.breaks:
            # 文件开头的内属性 #![…]：一个模块头单元（限定名 <module attributes>），这份文件里的单元都连它
            self.current_module_head = self.add("mod_head", "<module attributes>", 0, inner_end, [], [], None, inner_conditions, False)
        self.module_documents.append((0, inner_end, []))
        self.scan(0, self.count, [], [], None, [], False)
        self.units.sort(key=lambda unit: unit.first_token)
        for unit in self.units:
            self.analyse(unit)
        self.extract_doctests()
        self.units.extend(self.doctests)

    def row(self, line):
        return snippet_of(self.source_rows[line - 1]) if 0 < line <= len(self.source_rows) else ""

    def t(self, index):
        return self.texts[index] if 0 <= index < self.count else ""

    def kind(self, index):
        return self.kinds[index] if 0 <= index < self.count else ""

    def refuse(self, unit, form, index):
        line = self.lines[index] if 0 <= index < self.count else unit.first_line
        unit.refusals.append(Refusal(form, self.path, line, self.row(line)))

    def add(self, kind, name, start, end, context, module_path, container, conditions, public):
        end = max(end, start + 1)
        pieces = self.texts[start:end]
        if "rust-doc-comments-kept" in self.breaks:
            pieces = pieces + ["\n".join(rows) for _line, _style, rows, next_index, _raw in self.documents if start <= next_index < end]
        unit = Unit("rust", kind, "::".join(context + [name]), self.path, self.lines[start], self.lines[min(end, self.count) - 1],
                    sha256_text("\x1f".join(pieces)))
        unit.short = name
        unit.first_token, unit.last_token = start, min(end, self.count)
        unit.container, unit.conditions, unit.scope, unit.public = container, list(conditions), tuple(module_path), public
        unit.self_types, unit.implicit_keys, unit.is_trait_impl, unit.blanket = set(), set(), False, False
        # header：属性之后、可见性起的那个词法单元（外文档注释落在 [首词法单元, header] 里的挂这个条目）；module_head：所在模块的模块头单元
        unit.header, unit.module_head = max(self.current_header, start), self.current_module_head
        self.units.append(unit)
        return unit

    # 条目的边界

    def function_end(self, k, high):
        c = k + 2
        while c < high:
            text = self.t(c)
            if text in ("(", "[") and c in self.pairs:
                c = self.pairs[c] + 1
                continue
            if text == "{" and c in self.pairs:
                return self.pairs[c] + 1
            if text == ";":
                return c + 1
            c += 1
        return high

    def head_end(self, k, high):
        c = k
        while c < high and self.t(c) not in ("{", ";"):
            c = self.pairs[c] + 1 if self.t(c) in ("(", "[") and c in self.pairs else c + 1
        return c

    def statement_end(self, k, high):
        c = k
        while c < high:
            text = self.t(c)
            if text in ("(", "[", "{") and c in self.pairs:
                c = self.pairs[c] + 1
                continue
            if text == ";":
                return c + 1
            c += 1
        return high

    def type_end(self, c, high):
        while c < high:
            text = self.t(c)
            if text == "{" and c in self.pairs:
                end = self.pairs[c] + 1
                return end + 1 if self.t(end) == ";" else end
            if text in ("(", "[") and c in self.pairs:
                c = self.pairs[c] + 1
                continue
            if text == ";":
                return c + 1
            c += 1
        return high

    def unknown_end(self, k, high):
        c = k
        while c < high:
            text = self.t(c)
            if text == "{" and c in self.pairs:
                end = self.pairs[c] + 1
                return end + 1 if self.t(end) == ";" else end
            if text in ("(", "[") and c in self.pairs:
                c = self.pairs[c] + 1
                continue
            if text == ";":
                return c + 1
            c += 1
        return high

    def macro_path_at(self, c):
        """c 起是不是一个宏调用 a::b::m!(…)：是就回宏名那一个词法单元的下标，否则 None。"""
        j = c
        if self.t(j) == ":" and self.t(j + 1) == ":":
            j += 2
        while self.kind(j) == IDENT and self.t(j + 1) == ":" and self.t(j + 2) == ":":
            j += 3
        if self.kind(j) == IDENT and self.t(j + 1) == "!" and self.t(j + 2) in ("(", "[", "{") and (j + 2) in self.pairs:
            return j
        return None

    def item_starts_at(self, c, high):
        if c >= high:
            return True
        text = self.t(c)
        if text == "#" and self.t(c + 1) in ("[", "!"):
            return True
        if text == ";" or text in RUST_ITEM_STARTS:
            return True
        return self.macro_path_at(c) is not None

    def verus_end(self, k, high):
        """verus! 里一个条目：到下一个同层条目起头之前（; 或 } 之后接条目起头）。"""
        c = k
        while c < high:
            text = self.t(c)
            if text in ("(", "[", "{") and c in self.pairs:
                c = self.pairs[c] + 1
                if text == "{" and self.item_starts_at(c, high):
                    return c
                continue
            c += 1
            if text == ";" and self.item_starts_at(c, high):
                return c
        return high

    def top_level_parts(self, low, high):
        parts, start, c = [], low, low
        while c < high:
            text = self.t(c)
            if text in ("(", "[", "{") and c in self.pairs:
                c = self.pairs[c] + 1
                continue
            if text == ",":
                parts.append((start, c))
                start = c + 1
            c += 1
        parts.append((start, high))
        return [part for part in parts if part[0] < part[1]]

    # 属性

    def condition(self, low, high, line):
        raw = " ".join(self.texts[low:high])
        try:
            tree = parse_cfg_tokens(self.kinds[low:high], self.texts[low:high])
        except ValueError:
            tree = None
        return (tree, raw, line)

    def attribute_conditions(self, low, high):
        word = self.t(low)
        line = self.lines[low] if low < self.count else 1
        if word == "cfg" and self.t(low + 1) == "(" and (low + 1) in self.pairs:
            return [self.condition(low + 2, self.pairs[low + 1], line)]
        if word == "test" and high == low + 1:
            return [(("name", "test"), TEST_ATTRIBUTE_RAW, line)]
        if word == "cfg_attr" and self.t(low + 1) == "(" and (low + 1) in self.pairs:
            parts = self.top_level_parts(low + 2, self.pairs[low + 1])
            conditions = []
            if len(parts) >= 2:
                predicate = self.condition(parts[0][0], parts[0][1], line)
                for part_low, _part_high in parts[1:]:
                    if self.t(part_low) == "cfg" and self.t(part_low + 1) == "(" and (part_low + 1) in self.pairs:
                        inner = self.condition(part_low + 2, self.pairs[part_low + 1], line)
                        tree = None
                        if predicate[0] is not None and inner[0] is not None:
                            tree = ("any", [("not", [predicate[0]]), inner[0]])
                        conditions.append((tree, f"cfg_attr({predicate[1]}, cfg({inner[1]}))", line))
            return conditions
        return []

    def path_attribute(self, attributes):
        for low, _high in attributes:
            if self.t(low) == "path" and self.t(low + 1) == "=" and self.kind(low + 2) == LITERAL:
                return rust_string_value(self.t(low + 2))
        return None

    # 切

    def skip_qualifiers(self, k):
        while True:
            word = self.t(k)
            if word in RUST_QUALIFIERS and self.t(k + 1) == "(" and (k + 1) in self.pairs and self.kind(self.pairs[k + 1] + 1) == IDENT:
                k = self.pairs[k + 1] + 1
                continue
            if word in RUST_QUALIFIERS and self.kind(k + 1) == IDENT:
                k += 1
                continue
            if word == "const" and self.t(k + 1) in ("fn", "unsafe", "async", "extern"):
                k += 1
                continue
            if word == "extern" and self.kind(k + 1) == LITERAL and self.t(k + 2) in ("fn", "unsafe"):
                k += 2
                continue
            if word == "extern" and self.t(k + 1) == "fn":
                k += 1
                continue
            return k

    def inner_attributes(self, low, high):
        """low 起连着的内属性 #![…] → (它们之后的下标, 它们的 cfg 条件)；模块头单元从条目起头到这里。"""
        i, conditions = low, []
        while i < high and self.t(i) == "#" and self.t(i + 1) == "!" and self.t(i + 2) == "[" and (i + 2) in self.pairs:
            close = self.pairs[i + 2]
            conditions += self.attribute_conditions(i + 3, close)
            i = close + 1
        return i, conditions

    def scan(self, low, high, context, module_path, container, conditions, verus):
        i = low
        while i < high and self.t(i) == "#" and self.t(i + 1) == "!" and self.t(i + 2) == "[" and (i + 2) in self.pairs:
            close = self.pairs[i + 2]
            conditions = conditions + self.attribute_conditions(i + 3, close)
            i = close + 1
        while i < high:
            start = i
            item_conditions = list(conditions)
            attributes = []
            while i < high and self.t(i) == "#" and self.t(i + 1) == "[" and (i + 1) in self.pairs:
                close = self.pairs[i + 1]
                item_conditions += self.attribute_conditions(i + 2, close)
                attributes.append((i + 2, close))
                i = close + 1
            if i < high and self.t(i) == "#" and self.t(i + 1) == "!" and self.t(i + 2) == "[" and (i + 2) in self.pairs:
                i = self.pairs[i + 2] + 1
                continue
            if i >= high:
                break
            if self.t(i) == ";":
                i += 1
                continue
            self.current_header = i
            keyword, public = i, False
            if self.t(keyword) == "pub":
                keyword += 1
                if self.t(keyword) == "(" and keyword in self.pairs:
                    keyword = self.pairs[keyword] + 1
                else:
                    public = True
            keyword = self.skip_qualifiers(keyword)
            end = self.item(start, keyword, high, context, module_path, container, item_conditions, public, attributes, verus)
            i = max(end, i + 1)

    def item(self, start, k, high, context, module_path, container, conditions, public, attributes, verus):
        word = self.t(k)
        member_visible = container is not None and (container.kind == "trait" or bool(container.is_trait_impl))
        line = self.lines[k] if k < self.count else self.lines[start]
        if word == "fn" and self.kind(k + 1) == IDENT:
            end = self.verus_end(k, high) if verus else self.function_end(k, high)
            self.add("fn", self.t(k + 1), start, end, context, module_path, container, conditions, public or member_visible)
            return end
        keyword = k + 1 if word == "auto" and self.t(k + 1) == "trait" else k
        if self.t(keyword) in ("impl", "trait") and word in ("impl", "trait", "auto"):
            head_end = self.head_end(keyword, high)
            is_impl = self.t(keyword) == "impl"
            if self.t(head_end) != "{" or head_end not in self.pairs:
                end = head_end + 1 if self.t(head_end) == ";" else head_end
                if is_impl:
                    head = self.impl_head(keyword, head_end)
                    unit = self.add("impl", head["label"], start, end, context, module_path, container, conditions, True)
                    self.set_impl_head(unit, head)
                else:
                    self.add("trait", self.t(keyword + 1), start, end, context, module_path, container, conditions, public or member_visible)
                return end
            close = self.pairs[head_end]
            if is_impl:
                head = self.impl_head(keyword, head_end)
                unit = self.add("impl", head["label"], start, head_end, context, module_path, container, conditions, True)
                self.set_impl_head(unit, head)
                label = head["label"]
            else:
                name = self.t(keyword + 1)
                unit = self.add("trait", name, start, head_end, context, module_path, container, conditions, public or member_visible)
                label = f"<trait {name}>"
            self.scan(head_end + 1, close, context + [label], module_path, unit, conditions, verus)
            return close + 1
        if word == "mod" and self.kind(k + 1) == IDENT:
            name = self.t(k + 1)
            if self.t(k + 2) == "{" and (k + 2) in self.pairs:
                close = self.pairs[k + 2]
                # 行内模块头：条目起头（外属性）到 { 与紧跟的内属性 #![…]（verus! 里不收内属性，到 { 为止）；模块里的单元都连它
                inner_end, inner_conditions = (k + 3, []) if verus else self.inner_attributes(k + 3, close)
                outer_head = self.current_module_head
                if "rust-module-heads-dropped" not in self.breaks:
                    self.current_module_head = self.add("mod_head", name, start, inner_end, context, module_path, container,
                                                        conditions + inner_conditions, public)
                if not verus:
                    self.module_documents.append((k + 3, inner_end, list(context) + [name]))
                self.scan(k + 3, close, context + [name], module_path + [name], None, conditions, verus)
                self.current_module_head = outer_head
                return close + 1
            if self.t(k + 2) == ";":
                unit = self.add("mod", name, start, k + 3, context, module_path, container, conditions, public)
                self.mod_declarations.append((unit, name, self.path_attribute(attributes), list(module_path)))
                return k + 3
        if word == "macro_rules" and self.t(k + 1) == "!" and self.kind(k + 2) == IDENT and (k + 3) in self.pairs:
            end = self.pairs[k + 3] + 1
            if self.t(end) == ";":
                end += 1
            exported = any(self.t(low) == "macro_export" for low, _high in attributes)
            self.add("macro_rules", self.t(k + 2), start, end, context, module_path, container, conditions, exported)
            return end
        if word == "use":
            end = self.verus_end(k, high) if verus else self.statement_end(k, high)
            # verus! 里的 broadcast use 照 crates/singlefs-unit-graph 写 broadcast-use@<broadcast 那一行>
            label = f"broadcast-use@{self.lines[k - 1]}" if verus and k - 1 >= start and self.t(k - 1) == "broadcast" else f"use@{line}"
            unit = self.add("use", label, start, end, context, module_path, container, conditions, public)
            unit.imports, unit.aliases, unit.import_glob = self.use_imports(k + 1, end - 1 if self.t(end - 1) == ";" else end)
            return end
        if verus and word == "group" and k - 1 >= start and self.t(k - 1) == "broadcast" and self.kind(k + 1) == IDENT:
            end = self.verus_end(k, high)
            self.add("broadcast_group", self.t(k + 1), start, end, context, module_path, container, conditions, public or member_visible)
            return end
        if word in ("struct", "enum", "union") and self.kind(k + 1) == IDENT:
            end = self.verus_end(k, high) if verus else self.type_end(k + 2, high)
            self.add(word, self.t(k + 1), start, end, context, module_path, container, conditions, public or member_visible)
            return end
        if word == "type" and self.kind(k + 1) == IDENT:
            end = self.verus_end(k, high) if verus else self.statement_end(k, high)
            self.add("type", self.t(k + 1), start, end, context, module_path, container, conditions, public or member_visible)
            return end
        if word in ("const", "static") and self.kind(k + 1) == IDENT:
            name = self.t(k + 2) if word == "static" and self.t(k + 1) == "mut" else self.t(k + 1)
            end = self.verus_end(k, high) if verus else self.statement_end(k, high)
            self.add(word, name, start, end, context, module_path, container, conditions, public or member_visible)
            return end
        if word == "extern" and self.t(k + 1) == "crate":
            end = self.statement_end(k, high)
            self.add("extern_crate", self.t(k + 2), start, end, context, module_path, container, conditions, public)
            return end
        if word == "extern" and (self.t(k + 1) == "{" or (self.kind(k + 1) == LITERAL and self.t(k + 2) == "{")):
            brace = k + 1 if self.t(k + 1) == "{" else k + 2
            if brace in self.pairs:
                end = self.pairs[brace] + 1
                unit = self.add("extern_block", f"extern@{line}", start, end, context, module_path, container, conditions, True)
                # fn / static 之后的名字（static mut 跳过 mut，与 crates/singlefs-unit-graph 的 extern_block_declared_names 同一口径）
                unit.defines = {self.t(j + 2) if self.t(j) == "static" and self.t(j + 1) == "mut" else self.t(j + 1) for j in range(brace, end)
                                if self.t(j) in ("fn", "static") and self.kind(j + 2 if self.t(j) == "static" and self.t(j + 1) == "mut" else j + 1) == IDENT}
                return end
        name_index = self.macro_path_at(k)
        if name_index is not None:
            name = self.t(name_index)
            opening = name_index + 2
            close = self.pairs[opening]
            end = close + 1
            if self.t(end) == ";":
                end += 1
            if name == "verus" and self.t(opening) == "{":
                self.scan(opening + 1, close, context, module_path, container, conditions, "rust-verus-segments-ignored" not in self.breaks)
                return end
            if name in RUST_ITEM_MACROS_HOLDING_ITEMS and self.t(opening) == "{":
                self.scan(opening + 1, close, context, module_path, container, conditions, verus)
                return end
            unit = self.add("item_macro", f"{name}!@{self.lines[name_index]}", start, end, context, module_path, container, conditions, True)
            keywords = RUST_KEYWORDS | (UNION_KEYWORD if "rust-union-as-keyword" in self.breaks else frozenset())
            unit.defines = {self.texts[j] for j in range(opening + 1, close) if self.kinds[j] == IDENT} - keywords
            return end
        end = self.verus_end(k, high) if verus else self.unknown_end(k, high)
        self.refusals.append(Refusal("unknown-item", self.path, line, self.row(line)))
        return end

    def set_impl_head(self, unit, head):
        unit.self_types, unit.trait_name = head["self_types"], head["trait_name"]
        unit.is_trait_impl, unit.blanket, unit.implicit_keys = head["is_trait_impl"], head["blanket"], head["implicit_keys"]
        unit.open_keys = head["open_keys"]

    def impl_head(self, keyword, head_end):
        tokens = list(range(keyword + 1, head_end))
        generics = set()
        if tokens and self.t(tokens[0]) == "<":
            depth, cut = 0, len(tokens)
            for position, index in enumerate(tokens):
                text = self.t(index)
                if text == "<":
                    depth += 1
                elif text == ">" and self.t(index - 1) != "-":
                    depth -= 1
                elif depth == 1 and self.kind(index) == IDENT and self.t(index - 1) in ("<", ",", "const") and text != "const":
                    generics.add(text)
                if depth == 0:
                    cut = position + 1
                    break
            tokens = tokens[cut:]
        where = next((position for position, index in enumerate(tokens) if self.t(index) == "where"), None)
        if where is not None:
            tokens = tokens[:where]
        label = "<impl " + re.sub(r"\s*: :\s*", "::", " ".join(self.t(index) for index in tokens)) + ">"
        depth, split = 0, None
        for position, index in enumerate(tokens):
            text = self.t(index)
            if text == "<":
                depth += 1
            elif text == ">" and self.t(index - 1) != "-":
                depth -= 1
            elif text == "for" and depth == 0 and self.t(index + 1) != "<":
                split = position
                break
        trait_tokens = tokens[:split] if split is not None else []
        self_tokens = tokens[split + 1:] if split is not None else tokens
        trait_name, trait_arguments, depth = None, set(), 0
        for index in trait_tokens:
            text = self.t(index)
            if text == "<":
                depth += 1
            elif text == ">" and self.t(index - 1) != "-":
                depth -= 1
            elif self.kind(index) == IDENT:
                if depth == 0 and text != "dyn":
                    trait_name = text
                elif depth > 0:
                    trait_arguments.add(text)
        self_names = [self.t(index) for index in self_tokens if self.kind(index) == IDENT and self.t(index) not in SELF_TYPE_SKIPPED]
        self_types = set(self_names)
        # blanket：自身类型里任何深度出现 impl 的任一类型或常量泛型参数（for T、&T、Box<T>、(T, u32)、[T]、[T; N]、Option<Box<T>>）；全是具体类型的按被点名的类型。
        # 弄坏开关 rust-blanket-self-parameter-only 退回旧判法：自身类型里头一个名字是泛型参数才算（Box<T>、Option<Box<T>> 不算）
        if "rust-blanket-self-parameter-only" in self.breaks:
            blanket = split is not None and bool(self_names) and self_names[0] in generics
        else:
            blanket = split is not None and bool(self_types & generics)
        return {"label": label, "trait_name": trait_name, "is_trait_impl": split is not None, "self_types": self_types,
                "blanket": blanket,
                # R-隐式按自身类型的名字认 trait impl（crates/singlefs-unit-graph 的 implementations_by_self_type_name）：trait 参数里的类型被点名不带进来
                "implicit_keys": self_types | (trait_arguments if "rust-trait-impls-by-trait-arguments" in self.breaks else set()),
                # R-开放按 trait 自己的名字找 impl（定案 3：impl Wrap<dyn Sink> for H 不是 Sink 的 impl）；
                # 弄坏开关 rust-open-by-trait-arguments 下另按 trait 参数里的名字找（crates/singlefs-unit-graph 现在按 trait_part_names 建索引的写法）
                "open_keys": ({trait_name} if trait_name else set())
                             | (trait_arguments - {"dyn"} if "rust-open-by-trait-arguments" in self.breaks else set())}

    def use_imports(self, low, high):
        """→ (引进的名字, [(别名, 原名)], 有没有 *)：路径段（后面接 ::）与 self / super / crate 不算名字，a as b 两边都算名字、记一条别名
        （as 两边都是标识符才记）；花括号里的 self 引进的是 { 前面那一段（use std::io::{self, Seek} 引进 io，self as c 记别名 (c, io)；定案 3）。
        弄坏开关 rust-use-self-imports-ignored 下花括号里的 self 不引进名字（crates/singlefs-unit-graph 的 use_imports 现在的写法）。"""
        names, aliases, glob_import = set(), [], False
        self_imports = "rust-use-self-imports-ignored" not in self.breaks
        braces = []           # 套着的每一层 { 前面那一段（不是名字的记 None）

        def original(j):
            if self_imports and self.t(j) == "self" and braces and self.t(j + 1) != ":":
                return braces[-1]
            return self.t(j)

        for j in range(low, high):
            text = self.t(j)
            if text == "*":
                glob_import = True
            elif text == "{":
                segment = self.t(j - 3) if self.t(j - 1) == ":" and self.t(j - 2) == ":" and self.kind(j - 3) == IDENT else None
                braces.append(segment if segment not in ("self", "super", "crate") else None)
            elif text == "}":
                if braces:
                    braces.pop()
            elif self.kind(j) == IDENT:
                if text == "as":
                    if self.kind(j + 1) == IDENT and j > low and self.kind(j - 1) == IDENT and original(j - 1) is not None:
                        aliases.append((self.t(j + 1), original(j - 1)))
                    continue
                if self.t(j + 1) == ":" and self.t(j + 2) == ":":
                    continue
                # `a::b as c`：原名 b 不进当前作用域，进的只有别名 c（Rust 那一份 rust_items.rs 的 use_imports 同一条）
                if self.t(j + 1) == "as":
                    continue
                if text not in ("self", "super", "crate"):
                    names.add(text)
                elif text == "self" and original(j) not in (None, "self"):
                    names.add(original(j))
        return names, aliases, glob_import

    # 单元读到的名字与数据

    def analyse(self, unit):
        unit.plain, unit.methods, unit.qualified, unit.dyn_traits = token_references(self.kinds, self.texts, unit.first_token, unit.last_token)
        unit.cfg_leaves = cfg_predicates_in_tokens(self.kinds, self.texts, self.lines, self.pairs, unit.first_token, unit.last_token)
        unit.references = unit.plain | unit.methods
        unit.data_references = []
        self.data_references(unit)

    def data_references(self, unit):
        i, end = unit.first_token, unit.last_token
        while i < end:
            text, kind = self.texts[i], self.kinds[i]
            if kind == IDENT and self.t(i + 1) == "!" and self.t(i + 2) in ("(", "[", "{") and (i + 2) in self.pairs:
                close = self.pairs[i + 2]
                if text in ("include_str", "include_bytes"):
                    self.include_reference(unit, i, i + 3, close)
                    i = close + 1
                    continue
                if text == "include":
                    self.refuse(unit, "include-code", i)
                    i = close + 1
                    continue
                if text in ("concat_idents", "paste"):
                    self.refuse(unit, "macro-generated-names", i)
                    i = close + 1
                    continue
                if text == "concat":
                    self.runtime_concat(unit, i, i + 3, close)
                    i = close + 1
                    continue
                if text == "env" and self.kind(i + 3) == LITERAL:
                    value = rust_string_value(self.t(i + 3))
                    if value == "CARGO_MANIFEST_DIR":
                        self.runtime_manifest_path(unit, i, close)
                    elif value and value.startswith("CARGO_BIN_EXE_"):
                        unit.data_references.append(("bin-executable", value[len("CARGO_BIN_EXE_"):], i))
                    elif value == "OUT_DIR":
                        unit.data_references.append(("out-directory", None, i))
                    i = close + 1
                    continue
            if kind == IDENT and text == "current_exe" and self.t(i + 1) == "(":
                self.refuse(unit, "current-exe", i)
            if kind == IDENT and text in ("var_os", "var") and self.t(i + 1) == "(" and rust_string_value(self.t(i + 2)) == "CARGO_MANIFEST_DIR":
                self.refuse(unit, "runtime-path", i)
            i += 1

    def concat_value(self, low, high):
        """concat!(…) 的参数 → ("path", 相对根的路径) / ("relative", 串) / ("out", None) / None（认不出）。"""
        pieces, manifest = [], False
        for part_low, part_high in self.top_level_parts(low, high):
            if part_high == part_low + 1 and self.kind(part_low) == LITERAL:
                value = rust_string_value(self.t(part_low))
                if value is None:
                    return None
                pieces.append(value)
                continue
            if self.t(part_low) == "env" and self.t(part_low + 1) == "!" and self.t(part_low + 2) == "(" and part_high == part_low + 5:
                name = rust_string_value(self.t(part_low + 3))
                if name == "CARGO_MANIFEST_DIR" and not pieces and not manifest:
                    manifest = True
                    continue
                if name == "OUT_DIR":
                    return ("out", None)
            return None
        text = "".join(pieces)
        if manifest:
            return ("path", posixpath.normpath(posixpath.join(self.crate_directory or ".", text.lstrip("/"))))
        return ("relative", text)

    def include_reference(self, unit, macro_index, low, high):
        parts = self.top_level_parts(low, high)
        path = None
        if len(parts) == 1:
            part_low, part_high = parts[0]
            if part_high == part_low + 1 and self.kind(part_low) == LITERAL:
                value = rust_string_value(self.t(part_low))
                if value is not None:
                    path = posixpath.normpath(posixpath.join(self.directory, value))
            elif self.t(part_low) == "concat" and self.t(part_low + 1) == "!" and self.t(part_low + 2) == "(" and (part_low + 2) in self.pairs:
                folded = self.concat_value(part_low + 3, self.pairs[part_low + 2])
                if folded is not None and folded[0] == "out":
                    unit.data_references.append(("out-directory", None, macro_index))
                    return
                if folded is not None and folded[0] == "path":
                    path = folded[1]
                elif folded is not None:
                    path = posixpath.normpath(posixpath.join(self.directory, folded[1]))
        if path is None:
            self.refuse(unit, "include-argument", macro_index)
            return
        # 根外的文件（include_str!("../../.claude/heavy-thread-default") 从根下一层的工作区读仓根下的登记文件）照 crates/singlefs-unit-graph 收：
        #   读得到就是数据单元（键只取字节摘要），路径写成绝对路径（unit-graph 的 display_path 对根外的路径同样不去前缀），不当 .wgsl 模块拼；
        #   读不到才拒算 include-missing。弄坏开关 rust-includes-outside-root-refused 下照改前根外一律拒算
        outside = path == ".." or path.startswith("../")
        if outside and "rust-includes-outside-root-refused" not in active_breaks():
            path = os.path.normpath(os.path.join(os.path.abspath(self.root), path))
            if not os.path.isfile(path):
                self.refuse(unit, "include-missing", macro_index)
                return
            unit.data_references.append(("file", path, macro_index))
            return
        if outside or not os.path.isfile(os.path.join(self.root, path)):
            self.refuse(unit, "include-missing", macro_index)
            return
        unit.data_references.append(("wgsl" if path.endswith(".wgsl") else "file", path, macro_index))

    def runtime_concat(self, unit, macro_index, low, high):
        folded = self.concat_value(low, high)
        manifest_reads = [j for j in range(low, high) if self.t(j) == "env" and rust_string_value(self.t(j + 3)) == "CARGO_MANIFEST_DIR"]
        if not manifest_reads:
            return
        # 读的那一处落在 env! 那个词上（与 crates/singlefs-unit-graph 记的行相同：concat!( 与 env! 分两行写时取 env! 那一行）
        if folded is not None and folded[0] == "path" and os.path.isfile(os.path.join(self.root, folded[1])):
            unit.data_references.append(("file", folded[1], manifest_reads[0]))
            return
        self.refuse(unit, "runtime-path", manifest_reads[0])

    def runtime_manifest_path(self, unit, environment_index, environment_close):
        opening = environment_index - 1
        if self.t(opening) == "(" and self.pairs.get(opening) == environment_close + 1 and self.t(opening - 1) in ("new", "from"):
            path = self.crate_directory or "."
            c = environment_close + 2
            while self.t(c) == "." and self.t(c + 1) == "join" and self.t(c + 2) == "(" and self.kind(c + 3) == LITERAL and self.t(c + 4) == ")":
                value = rust_string_value(self.t(c + 3))
                if value is None:
                    break
                path = posixpath.normpath(posixpath.join(path, value))
                c += 5
            if self.t(c) != "." and not path.startswith("../") and os.path.isfile(os.path.join(self.root, path)):
                unit.data_references.append(("file", path, environment_index))
                return
        self.refuse(unit, "runtime-path", environment_index)

    # 文档测试（照 crates/singlefs-unit-graph 的 push_documentation_tests 与 rust_code_blocks）

    def extract_doctests(self):
        """条目的外文档注释（下一个词法单元落在 [条目首词法单元, header] 里的）与模块开头的内文档注释（文件开头、行内模块体开头到内属性收尾）
        各自拼起来找 Rust 代码块，一个代码块一个文档测试单元。字段、枚举变体、函数体里的条目与行内模块头上的外文档注释不挂（Rust 那一份也不挂）。"""
        if not self.documents:
            return
        outer = [(next_index, line, raw) for line, style, _rows, next_index, raw in self.documents if style == "outer"]
        inner = [(next_index, line, raw) for line, style, _rows, next_index, raw in self.documents if style == "inner"]
        for unit in list(self.units):
            if unit.kind == "mod_head" or unit.first_token is None:
                continue
            comments = [(line, raw) for next_index, line, raw in outer if unit.first_token <= next_index <= unit.header]
            for code, opening, closing in rust_code_blocks(comments):
                self.add_doctest(code, opening, closing, unit, None)
        for low, high, module_path in self.module_documents:
            comments = [(line, raw) for next_index, line, raw in inner if low <= next_index <= high]
            for code, opening, closing in rust_code_blocks(comments):
                self.add_doctest(code, opening, closing, None, module_path)

    def add_doctest(self, code, open_line, close_line, owner, module_path):
        """限定名：挂条目的 <条目>::<doctest line 行>，挂模块的 <模块路径>::<doctest line 行>（文件开头的就是 <doctest line 行>）。
        指纹：代码块（去掉隐藏行记号）重新切词、去注释，词法单元以 0x1f 连的 sha256；弄坏开关 rust-doctest-fingerprint-from-rows 下取代码块原文各行以 \\n 连。"""
        tail = f"<doctest line {open_line}>"
        name = f"{owner.name}::{tail}" if owner is not None else "::".join(list(module_path) + [tail])
        kinds, texts, _lines, _documents = rust_lex(code)
        if "rust-doctest-fingerprint-from-rows" in self.breaks:
            fingerprint = sha256_text(code)
        else:
            fingerprint = sha256_text("\x1f".join(texts))
        unit = Unit("rust", "doctest", name, self.path, open_line, close_line, fingerprint)
        unit.plain, unit.methods, unit.qualified, unit.dyn_traits = token_references(kinds, texts, 0, len(texts))
        unit.references = unit.plain | unit.methods
        code_pairs, _bad = match_brackets(kinds, texts)
        unit.cfg_leaves = cfg_predicates_in_tokens(kinds, texts, [open_line + line for line in _lines], code_pairs or {}, 0, len(texts))
        unit.container = owner
        unit.conditions = list(owner.conditions) if owner is not None else []
        unit.scope = owner.scope if owner is not None else tuple(module_path)
        unit.doc_module = None if owner is not None else tuple(module_path)
        unit.public, unit.data_references = False, []
        unit.self_types, unit.implicit_keys = set(), set()
        unit.first_token = unit.last_token = None
        self.doctests.append(unit)


RUST_CODE_BLOCK_WORDS = frozenset(("rust", "ignore", "should_panic", "no_run", "compile_fail", "test_harness", "allow_fail", "standalone_crate", "custom"))


def is_rust_code_block_information(information):
    """代码块的信息串是不是 rustdoc 当 Rust 文档测试编的那一种（空的也是）；照 crates/singlefs-unit-graph 的同名函数。"""
    for word in re.split(r"[, \t]", information):
        if not word:
            continue
        if word in RUST_CODE_BLOCK_WORDS or word.startswith("edition") or word.startswith("ignore-"):
            continue
        if len(word) > 1 and word[0] == "E" and all("0" <= character <= "9" for character in word[1:]):
            continue
        return False
    return True


def code_fence_of(trimmed):
    """行首围栏：三个以上的 ` 或 ~ → (围栏字符, 个数)，否则 None。"""
    if not trimmed or trimmed[0] not in "`~":
        return None
    count = len(trimmed) - len(trimmed.lstrip(trimmed[0]))
    return (trimmed[0], count) if count >= 3 else None


def without_hidden_line_marker(line):
    """rustdoc 的隐藏行：# 代码 照编；单独一个 # 是空行；## 开头的是一个字面的 #。"""
    trimmed = line.lstrip()
    if trimmed == "#":
        return ""
    if trimmed.startswith("# "):
        return trimmed[2:]
    if trimmed.startswith("##"):
        return trimmed[1:]
    return line


def rust_code_blocks(comments):
    """[(注释起行, [各行原文])] 拼起来找代码块 → [(代码, 开围栏那一行, 闭围栏那一行)]，只留 Rust 代码块；没闭上的到最后一行。"""
    blocks, opened = [], None
    for first_line, rows in comments:
        for offset, raw in enumerate(rows):
            line = first_line + offset
            trimmed = raw.lstrip()
            fence = code_fence_of(trimmed)
            if opened is None:
                if fence is not None:
                    opened = (fence[0], fence[1], line, [], is_rust_code_block_information(trimmed[fence[1]:].strip()))
                continue
            if fence is not None and fence[0] == opened[0] and fence[1] >= opened[1] and not trimmed[fence[1]:].strip():
                if opened[4]:
                    blocks.append(("\n".join(opened[3]), opened[2], line))
                opened = None
            else:
                opened[3].append(without_hidden_line_marker(raw))
    if opened is not None and opened[4]:
        blocks.append(("\n".join(opened[3]), opened[2], opened[2] + len(opened[3])))
    return blocks


# ── Rust：包、目标、模块树、名字表 ───────────────────────────────────────────

KNOWN_CFG_NAMES = frozenset(("test", "debug_assertions", "unix", "windows", "doc", "doctest", "miri", "proc_macro", "overflow_checks",
                             "panic", "target_arch", "target_os", "target_family", "target_env", "target_endian", "target_pointer_width",
                             "target_vendor", "target_has_atomic", "target_abi", "target_feature", "feature", "relocation_model", "ub_checks",
                             "fmt_debug", "contract_checks"))
# x86_64-unknown-linux-gnu 那一行（与 crates/singlefs-unit-graph 的 TARGET_PLATFORMS 同一份，target_abi 是空串）加上恒开的 debug_assertions
DEFAULT_RUST_CFG = ("debug_assertions", "unix", 'target_os="linux"', 'target_family="unix"', 'target_arch="x86_64"',
                    'target_pointer_width="64"', 'target_endian="little"', 'target_env="gnu"', 'target_vendor="unknown"', 'target_abi=""', 'panic="unwind"',
                    'target_has_atomic="8"', 'target_has_atomic="16"', 'target_has_atomic="32"', 'target_has_atomic="64"',
                    'target_has_atomic="ptr"')
TARGET_PREFERENCE = ("lib", "bin", "test", "bench", "example", "build")
# 输出行限定名前缀取「头一个目标」时的目标次序（crates/singlefs-unit-graph 的 discover：lib、bin、test、example、bench、build）
RUST_TARGET_ORDER = {"lib": 0, "bin": 1, "test": 2, "example": 3, "bench": 4, "build": 5}


def load_manifest(root, path):
    try:
        with open(os.path.join(root, path), "rb") as handle:
            return tomllib.load(handle)
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise FunctionUnitsError(f"读不出 {path}：{error}") from error


class RustDependency:
    __slots__ = ("name", "directory", "features", "default_features")

    def __init__(self, name, directory, features, default_features):
        self.name, self.directory, self.features, self.default_features = name, directory, features, default_features


class RustCrate:
    def __init__(self, root, directory, manifest, workspace_dependencies):
        self.directory = directory
        package = manifest.get("package", {})
        self.package = package.get("name", posixpath.basename(directory) or "root")
        self.label = posixpath.basename(directory) if directory else self.package
        lib = manifest.get("lib", {})
        self.lib_name = lib.get("name", self.package.replace("-", "_"))
        self.proc_macro = bool(lib.get("proc-macro") or lib.get("proc_macro"))
        self.features = {name: list(items) for name, items in manifest.get("features", {}).items()}
        self.manifest = manifest
        self.dependencies = {"normal": [], "dev": [], "build": []}
        tables = [manifest] + list(manifest.get("target", {}).values())
        for section, key in (("dependencies", "normal"), ("dev-dependencies", "dev"), ("build-dependencies", "build")):
            for table in tables:
                for name, specification in table.get(section, {}).items():
                    self.dependencies[key].append(self.dependency(root, name, specification, workspace_dependencies))
        self.check_cfg = set()
        lints = manifest.get("lints", {}).get("rust", {}).get("unexpected_cfgs", {})
        if isinstance(lints, dict):
            for entry in lints.get("check-cfg", []):
                found = re.match(r"^cfg\(([A-Za-z_][A-Za-z0-9_]*)", entry)
                if found:
                    self.check_cfg.add(found.group(1))

    def dependency(self, root, name, specification, workspace_dependencies):
        directory, features, default_features = None, [], True
        if isinstance(specification, dict):
            features = list(specification.get("features", []))
            default_features = specification.get("default-features", specification.get("default_features", True))
            if "path" in specification:
                directory = posixpath.normpath(posixpath.join(self.directory or ".", specification["path"]))
            elif specification.get("workspace"):
                shared = workspace_dependencies.get(name, {})
                if isinstance(shared, dict) and "path" in shared:
                    directory = posixpath.normpath(shared["path"])
                    features += list(shared.get("features", []))
        return RustDependency(name.replace("-", "_"), directory, features, default_features)


class RustTarget:
    def __init__(self, crate, kind, name, root_file):
        self.crate, self.kind, self.name, self.root_file = crate, kind, name, root_file
        self.id = f"{crate.label}:{kind}:{name}"
        self.files = {}       # 文件 → (模块路径, 那份文件的 cfg 条件链)
        self.declared_by = collections.defaultdict(list)    # 文件 → 这个目标里把它连进模块树的每一条 mod 声明单元（R-结构）
        self.refusals = []


def discover_rust_crates(root):
    manifest = load_manifest(root, "Cargo.toml") if os.path.isfile(os.path.join(root, "Cargo.toml")) else {}
    workspace = manifest.get("workspace")
    directories = []
    if workspace:
        excluded = {posixpath.normpath(path) for path in workspace.get("exclude", [])}
        for pattern in workspace.get("members", []):
            for match in sorted(glob.glob(os.path.join(root, pattern))):
                relative = os.path.relpath(match, root).replace(os.sep, "/")
                if relative not in excluded and os.path.isfile(os.path.join(match, "Cargo.toml")):
                    directories.append(relative)
    elif os.path.isdir(os.path.join(root, "crates")):
        for name in sorted(os.listdir(os.path.join(root, "crates"))):
            if os.path.isfile(os.path.join(root, "crates", name, "Cargo.toml")):
                directories.append(f"crates/{name}")
    if "package" in manifest:
        directories.insert(0, "")
    shared = workspace.get("dependencies", {}) if workspace else {}
    crates = []
    for directory in dict.fromkeys(directories):
        crate_manifest = manifest if directory == "" else load_manifest(root, posixpath.join(directory, "Cargo.toml"))
        crates.append(RustCrate(root, directory, crate_manifest, shared))
    return crates


def crate_targets(root, crate):
    def place(*parts):
        return posixpath.normpath(posixpath.join(crate.directory or ".", *parts))

    def exists(path):
        return os.path.isfile(os.path.join(root, path))

    def listed(directory, single_files=True):
        found = []
        absolute = os.path.join(root, place(directory))
        if not os.path.isdir(absolute):
            return found
        for name in sorted(os.listdir(absolute)):
            if single_files and name.endswith(".rs") and os.path.isfile(os.path.join(absolute, name)):
                found.append((name[:-3], place(directory, name)))
            elif os.path.isfile(os.path.join(absolute, name, "main.rs")):
                found.append((name, place(directory, name, "main.rs")))
        return found

    targets, seen = [], set()

    def add(kind, name, path):
        if (kind, name) not in seen and exists(path):
            seen.add((kind, name))
            targets.append(RustTarget(crate, kind, name, path))

    manifest = crate.manifest
    lib = manifest.get("lib", {})
    if "package" in manifest:
        add("lib", crate.lib_name, place(lib["path"]) if "path" in lib else place("src", "lib.rs"))
    for kind, section, directory in (("bin", "bin", "src/bin"), ("test", "test", "tests"), ("bench", "bench", "benches"), ("example", "example", "examples")):
        for entry in manifest.get(section, []):
            name = entry.get("name")
            path = place(entry["path"]) if "path" in entry else (place("src", "main.rs") if kind == "bin" and name == crate.package else place(directory, f"{name}.rs"))
            add(kind, name, path)
        if kind == "bin":
            add("bin", crate.package, place("src", "main.rs"))
        for name, path in listed(directory):
            add(kind, name, path)
    build = manifest.get("package", {}).get("build")
    if isinstance(build, str):
        add("build", crate.label, place(build))
    elif build is not False:
        add("build", crate.label, place("build.rs"))
    return targets


class CfgEvaluator:
    def __init__(self, names, pairs, features_by_crate, test_targets, known_extra):
        self.names, self.pairs, self.features_by_crate = names, pairs, features_by_crate
        self.test_targets, self.known_extra = test_targets, known_extra

    def value(self, tree, crate_directory, target_id):
        kind = tree[0]
        if kind == "all":
            return all(self.value(item, crate_directory, target_id) for item in tree[1])
        if kind == "any":
            return any(self.value(item, crate_directory, target_id) for item in tree[1])
        if kind == "not":
            return not self.value(tree[1][0], crate_directory, target_id)
        if kind == "name":
            name = tree[1]
            if name == "test":
                return target_id in self.test_targets
            if name in self.names:
                return True
            if name in KNOWN_CFG_NAMES or name in self.known_extra:
                return False
            raise ValueError(name)
        key, value = tree[1], tree[2]
        if key == "feature":
            return value in self.features_by_crate.get(crate_directory, ())
        if (key, value) in self.pairs:
            return True
        if key in KNOWN_CFG_NAMES or key in self.known_extra:
            return False
        raise ValueError(key)


def cfg_entries(entries):
    names, pairs = set(), set()
    for entry in entries:
        found = re.match(r'^([A-Za-z_][A-Za-z0-9_]*)\s*=\s*"(.*)"$', entry)
        if found:
            pairs.add((found.group(1), found.group(2)))
        elif re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", entry):
            names.add(entry)
        else:
            raise UsageError(f"认不出的 cfg：{entry}（写 <名> 或 <名>=\"<值>\"）")
    return names, pairs


def condition_mentions_test(tree):
    if tree is None:
        return False
    if tree[0] == "name":
        return tree[1] == "test"
    if tree[0] in ("all", "any"):
        return any(condition_mentions_test(item) for item in tree[1])
    return False


class RustWorkspace:
    def __init__(self, root):
        if not os.path.isdir(root):
            raise FunctionUnitsError(f"根不在：{root}")
        self.root = root
        self.breaks = active_breaks()
        self.crates = discover_rust_crates(root)
        self.crate_by_directory = {crate.directory: crate for crate in self.crates}
        self.files = {}
        self.targets = {}
        self.names_assigned = False
        for crate in self.crates:
            for target in crate_targets(root, crate):
                self.targets[target.id] = target
                directory = posixpath.dirname(target.root_file)
                self.walk(target, target.root_file, [], [], directory)
        for path in sorted(self.files):
            self.assign_full_names(self.files[path])
        self.names_assigned = True
        self.index()

    def file_prefix(self, path):
        """一份文件里单元限定名的前缀（照 crates/singlefs-unit-graph 的 module_prefix_of_file）：crate 名、这份文件所属的头一个目标的标记
        （目标次序照那一份：crate 按目录排，crate 里 lib、bin、test、example、bench、build，同种按名字与根文件排；lib 不写标记）、
        它在那个目标里的模块路径；哪个目标都不属于的写 <unlinked 路径>。"""
        crate = self.crate_of_path(path)
        prefix = [crate.lib_name] if crate is not None else []
        owners = [target for target in self.targets.values() if target.kind != "doctest" and path in target.files]
        if not owners:
            return prefix + [f"<unlinked {path}>"]
        first = min(owners, key=lambda target: (target.crate.directory, RUST_TARGET_ORDER.get(target.kind, len(RUST_TARGET_ORDER)),
                                                target.name, target.root_file))
        marker = {"bin": f"<bin {first.name}>", "test": f"<test {first.name}>", "example": f"<example {first.name}>",
                  "bench": f"<bench {first.name}>", "build": "<build>"}.get(first.kind)
        return prefix + ([marker] if marker else []) + list(first.files[path][0])

    def assign_full_names(self, rust_file):
        """单元的 full（输出行里的限定名）= 前缀 + 文件里的嵌套路径；文档测试挂条目的接在条目的 full 后面，挂模块的接在前缀加模块路径后面。"""
        prefix = self.file_prefix(rust_file.path)
        for unit in rust_file.units:
            if unit.kind != "doctest":
                unit.full = "::".join(prefix + [unit.name])
        for unit in rust_file.units:
            if unit.kind == "doctest":
                tail = unit.name.rsplit("::", 1)[-1]
                if unit.container is not None:
                    unit.full = f"{unit.container.full}::{tail}"
                else:
                    unit.full = "::".join(prefix + list(unit.doc_module or ()) + [tail])

    def crate_of_path(self, path):
        best = None
        for crate in self.crates:
            prefix = crate.directory + "/" if crate.directory else ""
            if path.startswith(prefix) and (best is None or len(crate.directory) > len(best.directory)):
                best = crate
        return best

    def file(self, path):
        if path not in self.files:
            crate = self.crate_of_path(path)
            with open(os.path.join(self.root, path), encoding="utf-8", errors="surrogateescape") as handle:
                text = handle.read()
            self.files[path] = RustFile(self.root, path, text, crate.directory if crate else posixpath.dirname(path))
            if self.names_assigned:
                self.assign_full_names(self.files[path])
        return self.files[path]

    def all_files(self):
        for crate in self.crates:
            for path in files_under(self.root, (".rs",), [crate.directory or "."]):
                if crate.directory == "" and any(path.startswith(other.directory + "/") for other in self.crates if other.directory):
                    continue
                self.file(path)
        return [self.files[path] for path in sorted(self.files)]

    def walk(self, target, path, module_path, chain, child_directory, declaring=None):
        if declaring is not None and declaring not in target.declared_by[path]:
            target.declared_by[path].append(declaring)
        if path in target.files:
            return
        rust_file = self.file(path)
        target.files[path] = (tuple(module_path), list(chain))
        for unit, name, path_attribute, inline in rust_file.mod_declarations:
            conditions = chain + unit.conditions
            if path_attribute is not None:
                candidate = posixpath.normpath(posixpath.join(posixpath.dirname(path), *inline, path_attribute))
                candidates = [candidate]
            else:
                base = posixpath.join(child_directory, *inline)
                candidates = [posixpath.normpath(posixpath.join(base, f"{name}.rs")), posixpath.normpath(posixpath.join(base, name, "mod.rs"))]
            found = next((candidate for candidate in candidates if os.path.isfile(os.path.join(self.root, candidate))), None)
            if found is None:
                target.refusals.append(Refusal("module-file-missing", path, unit.first_line, rust_file.row(unit.first_line)))
                continue
            if path_attribute is not None or found.endswith("/mod.rs"):
                next_directory = posixpath.dirname(found)
            else:
                next_directory = posixpath.normpath(posixpath.join(child_directory, *inline, name))
            self.walk(target, found, list(module_path) + list(inline) + [name], conditions, next_directory, unit)

    def index(self):
        self.unit_targets = collections.defaultdict(set)
        self.by_name = {}
        self.uses_by_scope = collections.defaultdict(list)
        self.members_by_name = collections.defaultdict(list)
        self.members_of = collections.defaultdict(list)
        self.implicit_impls = collections.defaultdict(list)
        self.inherent_impls = collections.defaultdict(list)
        self.impls_of_trait = collections.defaultdict(list)
        self.blanket_impls = collections.defaultdict(list)
        self.wgsl_const_file = {}
        indexed = set()
        # const _ / static _ 没有名字：R-裸名的着落里不收 _（let _ = …、_ => 里的 _ 不点它；条目级宏调用、extern 块定义的名字里的 _ 同样不收）；
        #   弄坏开关 rust-underscore-items-named 下照收
        underscore_named = "rust-underscore-items-named" in active_breaks()
        for target in self.targets.values():
            names = self.by_name.setdefault(target.id, collections.defaultdict(list))
            for path in target.files:
                for unit in self.files[path].units:
                    if unit.kind == "doctest":
                        continue
                    self.unit_targets[id(unit)].add(target.id)
                    if unit.container is None and unit.kind in RUST_NAMED_KINDS and (unit.short != "_" or underscore_named):
                        names[unit.short].append(unit)
                    if unit.kind in ("item_macro", "extern_block"):
                        for name in unit.defines:
                            if name != "_" or underscore_named:
                                names[name].append(unit)
            for path in target.files:
                if path in indexed:
                    continue
                indexed.add(path)
                for unit in self.files[path].units:
                    if unit.kind == "doctest":
                        continue
                    if unit.kind == "use":
                        self.uses_by_scope[(unit.file, unit.scope)].append(unit)
                    if unit.container is not None:
                        self.members_by_name[unit.short].append(unit)
                        self.members_of[id(unit.container)].append(unit)
                    if unit.kind == "impl":
                        if unit.is_trait_impl:
                            for key in unit.implicit_keys:
                                self.implicit_impls[key].append(unit)
                            for key in unit.open_keys or ():
                                self.impls_of_trait[key].append(unit)
                            if unit.trait_name and unit.blanket:
                                self.blanket_impls[unit.trait_name].append(unit)
                        else:
                            for key in unit.self_types:
                                self.inherent_impls[key].append(unit)
                    if unit.kind == "const":
                        wgsl = [reference[1] for reference in unit.data_references if reference[0] == "wgsl"]
                        if wgsl:
                            self.wgsl_const_file[id(unit)] = wgsl[0]

    def visible(self, target_id, development=False):
        target = self.targets[target_id]
        crate = target.crate
        out = [target_id]
        if target.kind == "doctest":
            lib = self.lib_target(crate)
            return [target_id] + (self.visible(lib.id, development=True) if lib else [])
        if target.kind == "build":
            direct = list(crate.dependencies["build"])
        else:
            direct = list(crate.dependencies["normal"])
            if development or target.kind in ("test", "bench", "example"):
                direct += crate.dependencies["dev"]
            lib = self.lib_target(crate)
            if target.kind != "lib" and lib is not None:
                out.append(lib.id)
        seen = set()
        todo = [dependency.directory for dependency in direct if dependency.directory in self.crate_by_directory]
        while todo:
            directory = todo.pop()
            if directory in seen:
                continue
            seen.add(directory)
            dependency_crate = self.crate_by_directory[directory]
            lib = self.lib_target(dependency_crate)
            if lib is not None and lib.id not in out:
                out.append(lib.id)
            todo += [dependency.directory for dependency in dependency_crate.dependencies["normal"] if dependency.directory in self.crate_by_directory]
        return out

    def lib_target(self, crate):
        return self.targets.get(f"{crate.label}:lib:{crate.lib_name}")

    def resolve_features(self, linked_crates, entry_crate, requested, no_default):
        enabled = {crate.directory: set() for crate in self.crates}

        def enable(crate, feature):
            have = enabled[crate.directory]
            if feature in have:
                return
            have.add(feature)
            for item in crate.features.get(feature, []):
                if item.startswith("dep:"):
                    continue
                if "/" in item:
                    dependency_name, sub_feature = item.split("/", 1)
                    dependency_name = dependency_name.rstrip("?").replace("-", "_")
                    for section in crate.dependencies.values():
                        for dependency in section:
                            if dependency.name == dependency_name and dependency.directory in self.crate_by_directory:
                                enable(self.crate_by_directory[dependency.directory], sub_feature)
                    continue
                enable(crate, item)

        if not no_default and "default" in entry_crate.features:
            enable(entry_crate, "default")
        for crate, feature in requested:
            enable(crate, feature)
        for crate in linked_crates:
            for section in crate.dependencies.values():
                for dependency in section:
                    dependency_crate = self.crate_by_directory.get(dependency.directory)
                    if dependency_crate is None or dependency_crate not in linked_crates:
                        continue
                    if dependency.default_features and "default" in dependency_crate.features:
                        enable(dependency_crate, "default")
                    for feature in dependency.features:
                        enable(dependency_crate, feature)
        return enabled

    def crate_by_package(self, name):
        for crate in self.crates:
            if name in (crate.package, crate.lib_name, crate.label):
                return crate
        raise UsageError(f"没有叫 {name} 的包")

    def find_entry(self, path, qualified, target_option):
        """→ (入口单元, 目标号)。path 为 None 时 qualified 是 fn 路径 <crate>::<模块…>::<名>。"""
        if path is None:
            first, _sep, rest = qualified.partition("::")
            crate = self.crate_by_package(first)
            lib = self.lib_target(crate)
            if lib is None:
                raise UsageError(f"{first} 没有 lib 目标")
            hits = []
            for file_path, (module_path, _chain) in lib.files.items():
                for unit in self.files[file_path].units:
                    full = "::".join(list(module_path) + [unit.name])
                    if full == rest:
                        hits.append(unit)
            if len(hits) != 1:
                raise UsageError(f"fn 路径 {qualified} 命中 {len(hits)} 个单元（要恰好一个）")
            return hits[0], lib.id
        if not os.path.isfile(os.path.join(self.root, path)):
            raise UsageError(f"入口文件不在：{path}")
        rust_file = self.file(path)
        hits = [unit for unit in rust_file.units if unit.name == qualified]
        if len(hits) != 1:
            raise UsageError(f"{path} 里限定名 {qualified} 命中 {len(hits)} 个单元（要恰好一个；dump 看限定名）")
        unit = hits[0]
        owners = [target for target in self.targets.values() if path in target.files]
        if unit.kind == "doctest":
            crate = self.crate_of_path(path)
            target = RustTarget(crate, "doctest", unit.name, path)
            self.targets[target.id] = target
            self.unit_targets[id(unit)].add(target.id)
            return unit, target.id
        if target_option:
            if target_option not in self.targets or path not in self.targets[target_option].files:
                raise UsageError(f"目标 {target_option} 里没有 {path}")
            return unit, target_option
        if not owners:
            raise UsageError(f"{path} 不在任何编译目标的模块树里")
        owners.sort(key=lambda target: (TARGET_PREFERENCE.index(target.kind), target.id))
        return unit, owners[0].id


def rust_unit_is_test(workspace, unit, target_id):
    target = workspace.targets[target_id]
    if target.kind in ("test", "bench"):
        return True
    chain = target.files.get(unit.file, ((), []))[1]
    probe = unit
    while probe is not None:
        if any(condition_mentions_test(condition[0]) for condition in probe.conditions):
            return True
        probe = probe.container
    return any(condition_mentions_test(condition[0]) for condition in chain)


class Closure:
    def __init__(self):
        self.members = {}
        self.order = []
        self.refusals = set()
        self.cfg_lines = []
        self.programs = {}
        self.via = {}
        # Rust 闭包另记（unit_graph_key 照 crates/singlefs-unit-graph 的 reuse_key 算键用）：入口目标的种类、入口目标的生效 cfg 集（无值名字写名字，
        # 键值写 键="值"）、读数据文件的每一处（文件、行、数据文件、摘要、读它的单元）、每个单元收进来时所在的目标
        self.entry_target_kind, self.entry_cfg_atoms = None, []
        # 键里的 cfg 成分（「<原子>\t<true|false>」的集合，rust_closure_of 填）；None 是没记（key_cfg_items 退回整套生效 cfg）
        self.cfg_atoms = None
        self.data_sites = set()
        self.unit_targets = {}

    def add(self, unit, via, rule):
        if id(unit) in self.members:
            return False
        self.members[id(unit)] = unit
        self.order.append(unit)
        self.via[id(unit)] = (via, rule)
        return True

    def units(self):
        return sorted(self.order, key=Unit.sort_key)

    def refusal_list(self):
        return sorted(self.refusals, key=lambda refusal: (refusal.file, refusal.line, refusal.form, refusal.snippet))


def rust_literal_texts(workspace, units):
    """旧口径（只给弄坏开关 wgsl-external-names-by-long-identifiers）：闭包里 Rust 单元的串字面量的值与长于 3 个字符的标识符，
    WGSL 外名是其中之一的子串、或以串里 <名>{ 的前缀起头就算有着落。"""
    texts = []
    for unit in units:
        if unit.language != "rust" or unit.first_token is None:
            continue
        rust_file = workspace.files[unit.file]
        for index in range(unit.first_token, unit.last_token):
            if rust_file.kinds[index] == LITERAL:
                value = rust_string_value(rust_file.texts[index])
                if value:
                    texts.append(value)
            elif rust_file.kinds[index] == IDENT and len(rust_file.texts[index]) > 3:
                texts.append(rust_file.texts[index])
    return texts


SHORTEST_WGSL_EXTERNAL_NAME_PREFIX_CHARACTERS = 2
# 按名字定义的名字能给 WGSL 外名着落的种类（crates/singlefs-unit-graph 的 defines_names_a_kernel_can_use）：常量、静态量与条目级宏调用
WGSL_NAME_DEFINING_KINDS = frozenset(("const", "static", "item_macro"))


def rust_declared_names(unit):
    """单元按名字定义的名字（crates/singlefs-unit-graph 的 declared_names）：条目级宏调用与 extern 块是它里面定义的那几个，别的是它自己的名字。"""
    if unit.kind in ("item_macro", "extern_block"):
        return unit.defines or set()
    return {unit.short}


def name_and_underscore_suffixes(name):
    """名字与它每个 _ 之后的尾段（GPU_FAILURE_POINT_NONE 交它自己、FAILURE_POINT_NONE、POINT_NONE、NONE）：外名以 _ 加它收尾的名字都算供它的名字。"""
    out = {name}
    for position, character in enumerate(name):
        if character == "_" and position + 1 < len(name):
            out.add(name[position + 1:])
    return out


def ascii_upper(text):
    """只把 ASCII 小写字母换成大写（与 Rust 的 to_ascii_uppercase 同；str.upper 会动非 ASCII 字符）。"""
    return "".join(character.upper() if "a" <= character <= "z" else character for character in text)


def rust_string_literal_words(token):
    """串字面量（原文以 " 或 r 起头）原文里 ASCII 字母、数字、_ 连成的每一段。"""
    return re.findall(r"[A-Za-z0-9_]+", token)


class WgslNameSources:
    """闭包里 Rust 单元给 WGSL 外名的着落（照 crates/singlefs-unit-graph 的 WgslExternalNameSources::supply）：
    ① 整名是某个 Rust 串字面量（原文以 " 或 r 起头）里的一个词（ASCII 字母、数字、_ 连成的一段）；
    ② 整名是闭包里常量、静态量、条目级宏调用按名字定义的名字，或这样一个名字以 _ 加整名收尾（GPU_FAILURE_POINT_NONE 供 FAILURE_POINT_NONE）；
    ③ 外名的首段连 _（按第一个 _ 切、连那个 _，T_HEADER 取 T_、FAILURE_POINT_NONE 取 FAILURE_）不短于 2 个字符、是某个 Rust 单元的串字面量原文
       或 macro_rules! 单元任一词法单元原文的子串，且其余那一段（不空，不分大小写）是闭包里某个串字面量的一个词，或闭包里某个单元定义或点名的名字。
    名字里没有 _ 的只认整名。suppliers(名字) 交（有没有着落, 给它着落的那几个单元）：WGSL 入口经 --rust-strings-entry 时把它们连同它们的闭包并进来。
    弄坏开关 wgsl-defined-name-suffixes-ignored 下不认 ②，wgsl-prefix-remainders-unchecked 下 ③ 不核其余那一段；
    wgsl-external-names-by-long-identifiers 下退回旧口径（rust_literal_texts：串的值与长于 3 个字符的标识符，不交单元）。"""

    def __init__(self, workspace, units):
        breaks = active_breaks()
        self.legacy = "wgsl-external-names-by-long-identifiers" in breaks
        self.check_remainders = "wgsl-prefix-remainders-unchecked" not in breaks
        self.words, self.texts, self.defined = {}, [], {}
        self.words_upper, self.names_upper = set(), set()
        if self.legacy:
            self.legacy_texts = rust_literal_texts(workspace, units)
            self.legacy_prefixes = literal_prefixes(self.legacy_texts)
            return
        for unit in units:
            if unit.language != "rust":
                continue
            declared = rust_declared_names(unit)
            if unit.kind in WGSL_NAME_DEFINING_KINDS and "wgsl-defined-name-suffixes-ignored" not in breaks:
                for name in declared:
                    for suffix in name_and_underscore_suffixes(name):
                        self.defined.setdefault(suffix, {})[id(unit)] = unit
            self.names_upper |= {ascii_upper(name) for name in declared}
            self.names_upper |= {ascii_upper(name) for name in unit.plain or () if name not in RUST_KEYWORDS}
            if unit.first_token is None:
                continue
            rust_file = workspace.files[unit.file]
            macro_body = unit.kind == "macro_rules"
            for index in range(unit.first_token, unit.last_token):
                token = rust_file.texts[index]
                literal = rust_file.kinds[index] == LITERAL and token[:1] in ('"', "r")
                if literal:
                    for word in rust_string_literal_words(token):
                        self.words.setdefault(word, {})[id(unit)] = unit
                if literal or macro_body:
                    self.texts.append((token, unit))
        self.words_upper = {ascii_upper(word) for word in self.words}

    def suppliers(self, name):
        """→ (有没有着落, 给它着落的单元列表)。"""
        if self.legacy:
            return literal_mentions(self.legacy_texts, self.legacy_prefixes, name), []
        if name in self.words or name in self.defined:
            found = dict(self.words.get(name, {}))
            found.update(self.defined.get(name, {}))
            return True, list(found.values())
        if "_" not in name:
            return False, []
        prefix = name[:name.index("_") + 1]
        remainder = ascii_upper(name[len(prefix):])
        if len(prefix) < SHORTEST_WGSL_EXTERNAL_NAME_PREFIX_CHARACTERS or not remainder:
            return False, []
        if self.check_remainders and remainder not in self.words_upper and remainder not in self.names_upper:
            return False, []
        found = {}
        for token, unit in self.texts:
            if prefix in token:
                found[id(unit)] = unit
        return bool(found), list(found.values())


def rust_closure_of(workspace, entry, target_id, cfg=None, no_default_cfg=False, features=(), no_default_features=False):
    """从入口收 Rust 闭包（3.3 八条规则），连同入口闭包拼出的 WGSL 模块（3.5）。"""
    breaks = active_breaks()
    closure = Closure()
    target = workspace.targets[target_id]
    test_compiled = rust_unit_is_test(workspace, entry, target_id) if target.kind != "doctest" else False
    linked = set(workspace.visible(target_id, development=test_compiled))
    linked_crates = {workspace.targets[identifier].crate for identifier in linked}
    requested = []
    for item in features:
        package, _slash, feature = item.partition("/")
        requested.append((workspace.crate_by_package(package), feature))
    enabled = workspace.resolve_features(linked_crates, target.crate, requested, no_default_features)
    names, pairs = cfg_entries(([] if no_default_cfg else list(DEFAULT_RUST_CFG)) + list(cfg or ()))
    known_extra = set(names) | {key for key, _value in pairs}
    for crate in workspace.crates:
        known_extra |= crate.check_cfg
    test_targets = {target_id} if test_compiled else set()
    evaluator = CfgEvaluator(names, pairs, enabled, test_targets, known_extra)
    closure.cfg_lines = sorted(set(names) | {f'{key}="{value}"' for key, value in pairs}
                               | {f'feature="{feature}"@{crate.package}' for crate in linked_crates for feature in enabled[crate.directory]}
                               | {f"test@{identifier}" for identifier in test_targets})
    closure.entry_target_kind = target.kind
    closure.entry_cfg_atoms = sorted(set(names) | {f'{key}="{value}"' for key, value in pairs} | ({"test"} if test_compiled else set())
                                     | {f'feature="{feature}"' for feature in enabled[target.crate.directory]})
    # 键里的 cfg 成分（定案 2）：闭包里各单元真引到的原子，各带收它进来那个目标下的真值（record_cfg_atoms 逐个记）
    closure.cfg_atoms = set()
    for identifier in linked:
        if workspace.targets[identifier].kind == "doctest":
            continue
        for refusal in workspace.targets[identifier].refusals:
            closure.refusals.add(refusal)
        for path in workspace.targets[identifier].files:
            closure.refusals.update(workspace.files[path].refusals)
        crate = workspace.targets[identifier].crate
        if crate.proc_macro:
            closure.refusals.add(Refusal("proc-macro-crate", posixpath.join(crate.directory, "Cargo.toml"), 1, f"proc-macro = true（{crate.package}）"))
    no_open = "rust-open-entry-traits-ignored" in breaks
    # R-开放只管入口签名；闭包里单元的 dyn Trait 不开放全部 impl（值来自闭包里点名的具体类型，R-隐式已连上它的 trait impl）。
    # 弄坏开关 rust-dyn-traits-open-every-implementation 下闭包里单元的 dyn Trait 照旧开放这个 trait 的全部 impl
    open_interior_dyn = "rust-dyn-traits-open-every-implementation" in breaks
    no_constants = "rust-constants-dropped" in breaks
    no_macros = "rust-macros-dropped" in breaks
    no_trait_impls = "rust-trait-impl-edges-dropped" in breaks
    ignore_cfg = "rust-cfg-ignored" in breaks
    no_module_heads = "rust-module-heads-unlinked" in breaks
    no_declarations = "rust-module-declarations-unlinked" in breaks
    no_aliases = "rust-use-aliases-unresolved" in breaks
    trait_self_paths = "rust-trait-self-paths-ignored" not in breaks
    # blanket impl（impl<T: …> Tr for T）够得到的方法算边（定案 3：python 保持，crates/singlefs-unit-graph 跟着改）；
    # 弄坏开关 rust-blanket-impls-ignored 下 R-方法、R-路径不放行 blanket impl，trait 头进闭包也不带它的 blanket impl（Rust 现在的写法）
    blanket = "rust-blanket-impls-ignored" not in breaks
    doctest_structure = "rust-doctest-structure-linked" in breaks
    body_cfg_atoms = "rust-cfg-macro-atoms-ignored" not in breaks
    raw_wgsl = "wgsl-raw-text" in breaks
    pull_suppliers = not ({"wgsl-name-suppliers-not-pulled-in", "wgsl-external-names-by-long-identifiers"} & breaks)
    pull_wgsl_definers = "wgsl-other-file-definitions-unlinked" not in breaks
    modules_per_naming_unit = "wgsl-modules-per-naming-unit" in breaks
    structure_only = RUST_STRUCTURE_ONLY_KINDS - ({"use"} if "rust-use-names-resolved" in breaks else set())
    dropped = set()
    if no_constants:
        dropped |= {"const", "static"}
    if no_macros:
        dropped |= {"macro_rules", "item_macro"}
    named = set(PRIMITIVE_TYPES)
    methods_seen, dyn_seen, seen = set(), set(), set()
    wgsl_modules = set()
    wgsl_paths = set()           # 闭包里单元 include_str! / include_bytes! 读的 .wgsl：合成一个模块（design 3.5，与 crates/singlefs-unit-graph 的 wgsl_files 同）
    suppliers_tried = set()
    data_units = {}
    todo = []

    def record_cfg_atoms(unit, identifier):
        """这个单元在这个目标下真引到的 cfg 原子与真值进 closure.cfg_atoms（「<原子>\\t<true|false>」）：它自己连外层的条件链（#[test] 不算），
        与它词法单元里每一处 #[cfg(…)] / #![cfg(…)] / #[cfg_attr(条件, …)] 的条件与里面套的 cfg(…)、cfg!(…)。认不出、判不了真假的拒算 cfg-unrecognized。"""
        if ignore_cfg:
            return
        crate_directory = workspace.targets[identifier].crate.directory
        sources = [condition for condition in unit.conditions or () if condition[1] != TEST_ATTRIBUTE_RAW]
        if body_cfg_atoms:
            sources += list(unit.cfg_leaves or ())
        for tree, raw, line in sources:
            if tree is None:
                closure.refusals.add(Refusal("cfg-unrecognized", unit.file, line, snippet_of(f"cfg({raw})")))
                continue
            for leaf in cfg_tree_leaves(tree):
                try:
                    holds = evaluator.value(leaf, crate_directory, identifier)
                except ValueError:
                    closure.refusals.add(Refusal("cfg-unrecognized", unit.file, line, snippet_of(f"cfg({raw})")))
                    continue
                closure.cfg_atoms.add(f"{cfg_atom_text(leaf)}\t{'true' if holds else 'false'}")

    def push_exact(unit, identifier, via, rule):
        if (id(unit), identifier) not in seen:
            todo.append((unit, identifier, via, rule))

    def push(unit, identifier, via, rule):
        owners = workspace.unit_targets.get(id(unit), ())
        if identifier in owners:
            push_exact(unit, identifier, via, rule)
            return
        for owner in sorted(owners):
            if owner in linked:
                push_exact(unit, owner, via, rule)

    def push_impl(head, identifier, via, rule):
        push(head, identifier, via, rule)
        for member in workspace.members_of.get(id(head), ()):
            push(member, identifier, head, "member")

    def active(unit, identifier):
        if ignore_cfg:
            return True
        crate_directory = workspace.targets[identifier].crate.directory
        chain = list(workspace.targets[identifier].files.get(unit.file, ((), []))[1]) + list(unit.conditions or ())
        for tree, raw, line in chain:
            if tree is None:
                closure.refusals.add(Refusal("cfg-unrecognized", unit.file, line, snippet_of(f"cfg({raw})")))
                continue
            try:
                if not evaluator.value(tree, crate_directory, identifier):
                    return False
            except ValueError:
                closure.refusals.add(Refusal("cfg-unrecognized", unit.file, line, snippet_of(f"cfg({raw})")))
        return True

    def name_type(type_name, identifier, via):
        if type_name in named:
            return
        named.add(type_name)
        if not no_trait_impls:
            for head in workspace.implicit_impls.get(type_name, ()):
                push_impl(head, identifier, via, f"trait-impl-of {type_name}")
        for head in workspace.inherent_impls.get(type_name, ()):
            for member in workspace.members_of.get(id(head), ()):
                if member.short in methods_seen:
                    push(member, identifier, via, f"method-late {member.short}")

    def dyn_trait(trait_name, identifier, via):
        if trait_name in dyn_seen:
            return
        dyn_seen.add(trait_name)
        for head in workspace.impls_of_trait.get(trait_name, ()):
            push_impl(head, identifier, via, f"open {trait_name}")

    def entry_signature_traits(unit):
        if unit.first_token is None:
            return set()
        rust_file = workspace.files[unit.file]
        out, i = set(), unit.first_token
        while i < unit.last_token and rust_file.t(i) != "{":
            if rust_file.t(i) in ("impl", "dyn", ":", "+") and rust_file.kind(i + 1) == IDENT:
                out.add(rust_file.t(i + 1))
            i += 1
        return out

    def collect_data(unit, identifier):
        """这个单元读的数据文件进闭包（一份文件一个 data-file 单元），另按读它的那一处记一条 data_sites（Rust 那一份的数据单元写法，unit_graph_key 用）；
        OUT_DIR 与 CARGO_BIN_EXE_<名> 连到构建脚本与那个 bin 的 main。"""
        crate = workspace.targets[identifier].crate
        for kind, value, index in unit.data_references or ():
            if kind == "file":
                if value not in data_units:
                    data_units[value] = data_file_unit(workspace.root, value)
                closure.add(data_units[value], unit, "data")
                closure.data_sites.add((unit.file, workspace.files[unit.file].lines[index], value, data_units[value].fingerprint, id(unit)))
            elif kind == "wgsl":
                if raw_wgsl:
                    if value not in data_units:
                        data_units[value] = data_file_unit(workspace.root, value)
                    closure.add(data_units[value], unit, "data")
                else:
                    wgsl_paths.add(value)
            elif kind == "out-directory":
                build = workspace.targets.get(f"{crate.label}:build:{crate.label}")
                mains = [candidate for candidate in workspace.files[build.root_file].units if candidate.kind == "fn" and candidate.short == "main"
                         and candidate.container is None] if build else []
                if not mains:
                    workspace.files[unit.file].refuse(unit, "include-argument", index)
                    closure.refusals.update(unit.refusals)
                for main in mains:
                    push_exact(main, build.id, unit, "out-directory")
            elif kind == "bin-executable":
                bin_target = workspace.targets.get(f"{crate.label}:bin:{value}")
                mains = [candidate for candidate in workspace.files[bin_target.root_file].units if candidate.kind == "fn" and candidate.short == "main"
                         and candidate.container is None] if bin_target else []
                if not mains:
                    closure.refusals.add(Refusal("bin-target-missing", unit.file, unit.first_line, f"CARGO_BIN_EXE_{value}"))
                for main in mains:
                    push_exact(main, bin_target.id, unit, "bin-executable")

    if not active(entry, target_id):
        raise UsageError(f"入口 {entry.file}::{entry.name} 在这一组 cfg 下不编（{'、'.join(closure.cfg_lines) or '空'}）：用 --feature <包名>/<特性> 或 --cfg 打开它")
    todo.append((entry, target_id, None, "entry"))
    for trait_name in entry_signature_traits(entry) if not no_open else ():
        dyn_trait(trait_name, target_id, entry)

    def drain():
        while todo:
            unit, identifier, via, rule = todo.pop()
            if (id(unit), identifier) in seen:
                continue
            seen.add((id(unit), identifier))
            if unit.kind in dropped or not active(unit, identifier):
                continue
            closure.add(unit, via, rule)
            closure.unit_targets.setdefault(id(unit), identifier)
            closure.refusals.update(unit.refusals)
            record_cfg_atoms(unit, identifier)
            if unit.kind == "extern_block":
                closure.refusals.add(Refusal("extern-block", unit.file, unit.first_line, workspace.files[unit.file].row(unit.first_line)))
            alias_originals = []
            if unit.kind != "doctest" or doctest_structure:
                # R-结构：成员连头；单元连所在模块的模块头、把它所在文件连进这个目标模块树的 mod 声明（经它再连上层模块的头与声明）、
                # 所在模块里引进了它用到的名字的 use（glob 一律连）；use a::b as c 里用到 c 的，b 按 R-裸名再解。
                # 文档测试是另编的一个 crate，挂在哪个条目下只管它叫什么：不连所挂的条目、也不连那个模块的 use（crates/singlefs-unit-graph 的 follow_structure 同）
                if unit.container is not None:
                    push(unit.container, identifier, unit, "container")
                if unit.module_head is not None and not no_module_heads:
                    push(unit.module_head, identifier, unit, "module-head")
                if unit.kind != "doctest" and not no_module_heads and not no_declarations:
                    for declaring in workspace.targets[identifier].declared_by.get(unit.file, ()):
                        push_exact(declaring, identifier, unit, "mod-declaration")
                references = unit.references or set()
                for use in workspace.uses_by_scope.get((unit.file, unit.scope), ()):
                    if use is not unit and (use.import_glob or (use.imports & references)):
                        push(use, identifier, unit, "use")
                        if not no_aliases:
                            alias_originals += [original for alias, original in use.aliases if alias in unit.plain]
            if unit.kind in structure_only:
                # use、mod 声明、模块头、extern crate 只是结构：读文件照收，里面的名字不往下解（crates/singlefs-unit-graph 的 resolves_its_own_names 同一口径）
                collect_data(unit, identifier)
                continue
            visible = workspace.visible(identifier, development=identifier in test_targets)
            for name in sorted(unit.plain) + alias_originals:
                for visible_id in visible:
                    table = workspace.by_name.get(visible_id)
                    if table is None:
                        continue
                    for candidate in table.get(name, ()):
                        if candidate is unit:
                            continue
                        if visible_id != identifier and not candidate.public:
                            continue
                        push_exact(candidate, visible_id, unit, f"name {name}")
            for name in unit.methods:
                methods_seen.add(name)
                for member in workspace.members_by_name.get(name, ()):
                    container = member.container
                    if container.kind == "trait" or (blanket and container.blanket) or (container.self_types & named):
                        push(member, identifier, unit, f"method {name}")
            own = unit.container
            for qualifier, name in unit.qualified:
                for member in workspace.members_by_name.get(name, ()):
                    container = member.container
                    if container.kind == "trait" and container.short == qualifier:
                        push(member, identifier, unit, f"path {qualifier}::{name}")
                    elif (trait_self_paths and container.kind == "trait" and qualifier == "Self" and own is not None and own.kind == "trait"
                          and own.short == container.short):
                        # trait 成员里的 Self::x 落到这个 trait 自己的成员 x（定案 3；crates/singlefs-unit-graph 的 follow_qualified_paths 同）
                        push(member, identifier, unit, f"path Self::{name}")
                    elif container.kind == "impl" and qualifier in container.self_types:
                        push(member, identifier, unit, f"path {qualifier}::{name}")
                    elif container.kind == "impl" and qualifier == "Self" and own is not None and (
                            container.self_types & own.self_types if trait_self_paths else container.self_types & (own.self_types or {own.short})):
                        push(member, identifier, unit, f"path Self::{name}")
                    elif container.kind == "impl" and container.is_trait_impl and container.trait_name == qualifier and ((blanket and container.blanket) or container.self_types & named):
                        push(member, identifier, unit, f"path {qualifier}::{name}")
            if unit.kind in RUST_TYPE_KINDS:
                name_type(unit.short, identifier, unit)
            if unit.kind == "trait":
                for head in workspace.blanket_impls.get(unit.short, ()) if blanket else ():
                    push_impl(head, identifier, unit, f"blanket {unit.short}")
                if not no_trait_impls:
                    for head in workspace.implicit_impls.get(unit.short, ()):
                        push_impl(head, identifier, unit, f"trait-impl-of {unit.short}")
            for trait_name in (unit.dyn_traits or ()) if open_interior_dyn else ():
                dyn_trait(trait_name, identifier, unit)
            collect_data(unit, identifier)
            if modules_per_naming_unit and not raw_wgsl and unit.plain:
                # 弄坏开关 wgsl-modules-per-naming-unit：退回旧口径，按点名读 .wgsl 的常量的那个单元一组一组拼模块
                module_files = set()
                for name in unit.plain:
                    for visible_id in visible:
                        for candidate in workspace.by_name.get(visible_id, {}).get(name, ()):
                            if id(candidate) in workspace.wgsl_const_file and candidate.kind not in dropped:
                                module_files.add(workspace.wgsl_const_file[id(candidate)])
                if unit.kind == "const" and id(unit) in workspace.wgsl_const_file:
                    module_files.discard(workspace.wgsl_const_file[id(unit)])
                if module_files:
                    wgsl_modules.add(frozenset(module_files))

    def wgsl_module_groups():
        if raw_wgsl:
            return []
        if modules_per_naming_unit:
            return sorted(wgsl_modules, key=sorted)
        return [frozenset(wgsl_paths)] if wgsl_paths else []

    def pull_in_wgsl_name_suppliers():
        """着落做成边（crates/singlefs-unit-graph 的 pull_in_wgsl_name_suppliers）：闭包里还没有着落的 WGSL 外名，把链接集里、闭包外供它的单元
        （按名字定义了它或以 _ 加它收尾的常量、静态量、条目级宏调用；不是这一种的，串字面量里有它这个词的单元）连同它们的闭包并进来；
        每个只试一次。另补一条着落边：还没有着落的外名在链接集里别的单元 include_str! / include_bytes! 读进来的一份 .wgsl（还不在这个模块里）
        定义了（那份文件的顶层条目：fn、const、override、alias、var、struct），把读它的那个 Rust 单元连同它的闭包并进来，那份 .wgsl 随之进模块、
        名字在合起来的名字空间里解；拉不进的才拒算 wgsl-undefined-name。弄坏开关 wgsl-other-file-definitions-unlinked 下不补这一条。
        → 这一趟有没有新推进去的单元。"""
        groups = wgsl_module_groups()
        if not groups or not (pull_suppliers or pull_wgsl_definers):
            return False
        sources = WgslNameSources(workspace, closure.order)
        unsupplied = set()
        for module_files in groups:
            _members, refusals, _suppliers = wgsl_closure([wgsl_file(workspace.root, path) for path in sorted(module_files)], None, sources)
            unsupplied |= {refusal.snippet for refusal in refusals if refusal.form == "wgsl-undefined-name"}
        if not unsupplied:
            return False
        pushed = False
        for identifier in sorted(linked):
            if workspace.targets[identifier].kind == "doctest":
                continue
            for path in sorted(workspace.targets[identifier].files):
                rust_file = workspace.files[path]
                for candidate in rust_file.units:
                    if candidate.kind == "doctest" or id(candidate) in closure.members or id(candidate) in suppliers_tried:
                        continue
                    if pull_wgsl_definers and any(
                            kind == "wgsl" and value not in wgsl_paths and wgsl_top_level_names_in(workspace.root, value) & unsupplied
                            for kind, value, _index in candidate.data_references or ()):
                        suppliers_tried.add(id(candidate))
                        push(candidate, identifier, None, "wgsl-file-definer")
                        pushed = True
                        continue
                    if not pull_suppliers:
                        continue
                    defines = candidate.kind in WGSL_NAME_DEFINING_KINDS and any(
                        name_and_underscore_suffixes(name) & unsupplied for name in rust_declared_names(candidate))
                    in_string = not defines and any(
                        rust_file.kinds[index] == LITERAL and rust_file.texts[index][:1] in ('"', "r")
                        and unsupplied & set(rust_string_literal_words(rust_file.texts[index]))
                        for index in range(candidate.first_token, candidate.last_token))
                    if defines or in_string:
                        suppliers_tried.add(id(candidate))
                        push(candidate, identifier, None, "wgsl-name-supplier")
                        pushed = True
        return pushed

    drain()
    while pull_in_wgsl_name_suppliers():
        drain()
    groups = wgsl_module_groups()
    if groups:
        sources = WgslNameSources(workspace, closure.order)
        for module_files in groups:
            wgsl_files = [wgsl_file(workspace.root, path) for path in sorted(module_files)]
            members, refusals, _suppliers = wgsl_closure(wgsl_files, None, sources)
            for member in members:
                closure.add(member, None, "wgsl-module")
            closure.refusals.update(refusals)
    return closure


RustClosureResult = collections.namedtuple("RustClosureResult", "members key refusals cfg_lines closure")


def rust_units(root):
    """→ [(种类, 限定名, 文件, (起, 止), 指纹, cfg 链)]，cfg 链是这个单元在它那份文件里的条件（外层模块与内属性的也在里面），
    原文写法、由外到里；mod 声明带进来的那一截按目标定，不在这里（closure 时按入口的目标求值）。"""
    workspace = RustWorkspace(root)
    return [(unit.kind, unit.name, unit.file, (unit.first_line, unit.last_line), unit.fingerprint, [raw for _tree, raw, _line in unit.conditions or ()])
            for rust_file in workspace.all_files() for unit in sorted(rust_file.units, key=Unit.sort_key)]


def rust_closure(root, entry, target=None, cfg=None):
    """入口（<文件>::<限定名> 或 fn 路径）在目标（默认按文件挑）与 cfg 集（加进默认全局 cfg 的几项）下 → 成员单元行、键（拒算时 None）、拒算。"""
    workspace = RustWorkspace(root)
    _language, path, qualified = parse_entry(root, entry)
    unit, target_id = workspace.find_entry(path, qualified, target)
    closure = rust_closure_of(workspace, unit, target_id, cfg)
    key, refusals = closure_key(closure)
    return RustClosureResult([member.text() for member in closure.units()], key, refusals, closure.cfg_lines, closure)


# ── WGSL ────────────────────────────────────────────────────────────────────

WGSL_PREDECLARED = frozenset("""
fn let var const override struct alias return if else loop for while break continue continuing switch case default discard true false
bitcast enable requires diagnostic const_assert
bool i32 u32 f32 f16 vec2 vec3 vec4 vec2i vec3i vec4i vec2u vec3u vec4u vec2f vec3f vec4f vec2h vec3h vec4h
mat2x2 mat2x3 mat2x4 mat3x2 mat3x3 mat3x4 mat4x2 mat4x3 mat4x4 mat2x2f mat3x3f mat4x4f array atomic ptr sampler sampler_comparison
texture_1d texture_2d texture_2d_array texture_3d texture_cube texture_cube_array texture_multisampled_2d texture_storage_1d
texture_storage_2d texture_storage_2d_array texture_storage_3d texture_depth_2d texture_depth_2d_array texture_depth_cube
texture_depth_cube_array texture_depth_multisampled_2d texture_external
function private workgroup uniform storage handle read write read_write
group binding compute workgroup_size vertex fragment builtin location interpolate invariant align size id must_use blend_src
global_invocation_id local_invocation_id local_invocation_index workgroup_id num_workgroups position vertex_index instance_index
front_facing frag_depth sample_index sample_mask subgroup_invocation_id subgroup_size
abs acos acosh all any arrayLength asin asinh atan atan2 atanh ceil clamp cos cosh countLeadingZeros countOneBits countTrailingZeros
cross degrees determinant distance dot dot4U8Packed dot4I8Packed exp exp2 extractBits faceForward firstLeadingBit firstTrailingBit
floor fma fract frexp insertBits inverseSqrt ldexp length log log2 max min mix modf normalize pow quantizeToF16 radians reflect refract
reverseBits round saturate select sign sin sinh smoothstep sqrt step tan tanh transpose trunc
atomicLoad atomicStore atomicAdd atomicSub atomicMax atomicMin atomicAnd atomicOr atomicXor atomicExchange atomicCompareExchangeWeak
workgroupBarrier storageBarrier textureBarrier workgroupUniformLoad
pack4x8snorm pack4x8unorm pack2x16snorm pack2x16unorm pack2x16float unpack4x8snorm unpack4x8unorm unpack2x16snorm unpack2x16unorm
unpack2x16float pack4xI8 pack4xU8 unpack4xI8 unpack4xU8
textureDimensions textureGather textureLoad textureNumLayers textureNumLevels textureNumSamples textureSample textureSampleLevel
textureStore dpdx dpdy fwidth
subgroupAdd subgroupAll subgroupAny subgroupBallot subgroupBroadcast subgroupMax subgroupMin
off on error warning info derivative_uniformity
_
""".split())


class WgslFile:
    def __init__(self, path, text):
        self.path = path
        self.source_rows = text.split("\n")
        self.kinds, self.texts, self.lines, _documents = rust_lex(text)
        self.count = len(self.texts)
        self.units, self.refusals = [], []
        self.pairs, bad = match_brackets(self.kinds, self.texts)
        if self.pairs is None:
            line = self.lines[bad] if bad is not None and bad < self.count else 1
            self.refusals.append(Refusal("unbalanced-brackets", path, line, self.row(line)))
            self.pairs = {}
            return
        self.scan()

    def row(self, line):
        return snippet_of(self.source_rows[line - 1]) if 0 < line <= len(self.source_rows) else ""

    def t(self, index):
        return self.texts[index] if 0 <= index < self.count else ""

    def kind(self, index):
        return self.kinds[index] if 0 <= index < self.count else ""

    def statement_end(self, i):
        c = i
        while c < self.count:
            text = self.t(c)
            if text in ("(", "[", "{") and c in self.pairs:
                c = self.pairs[c] + 1
                continue
            if text == ";":
                return c + 1
            c += 1
        return self.count

    def add(self, kind, name, start, end):
        end = max(end, start + 1)
        unit = Unit("wgsl", kind, name, self.path, self.lines[start], self.lines[min(end, self.count) - 1],
                    sha256_text("\x1f".join(self.texts[start:end])))
        unit.first_token, unit.last_token = start, min(end, self.count)
        plain, declared, name_lines = set(), set(), {}
        for index in range(start, unit.last_token):
            if self.kinds[index] != IDENT:
                continue
            text = self.texts[index]
            previous = self.t(index - 1)
            if previous in (".", "@"):
                continue
            if previous == "(" and self.t(index - 2) in ("builtin", "interpolate", "diagnostic"):
                continue
            if self.t(index + 1) == ":" and self.t(index + 2) != ":":
                declared.add(text)
            if previous in ("let", "var", "const", "override") or (previous == ">" and self.t(index - 2) != "-" and self.var_head(index)):
                declared.add(text)
            plain.add(text)
            name_lines.setdefault(text, self.lines[index])
        unit.plain, unit.declared, unit.cache = plain, declared, name_lines
        self.units.append(unit)
        return unit

    def var_head(self, index):
        """index 前面是不是 var<…> 的那个 >。"""
        j = index - 1
        depth = 0
        while j >= 0:
            text = self.t(j)
            if text == ">":
                depth += 1
            elif text == "<":
                depth -= 1
                if depth == 0:
                    return self.t(j - 1) == "var"
            elif text in (";", "{", "}"):
                return False
            j -= 1
        return False

    def scan(self):
        i = 0
        while i < self.count:
            start = i
            attributes = []
            while self.t(i) == "@" and self.kind(i + 1) == IDENT:
                attributes.append(self.t(i + 1))
                i += 2
                if self.t(i) == "(" and i in self.pairs:
                    i = self.pairs[i] + 1
            if i >= self.count:
                break
            word = self.t(i)
            line = self.lines[i]
            if word == ";":
                i += 1
                continue
            if word == "fn" and self.kind(i + 1) == IDENT:
                c, end = i + 2, self.count
                while c < self.count:
                    if self.t(c) == "(" and c in self.pairs:
                        c = self.pairs[c] + 1
                        continue
                    if self.t(c) == "{" and c in self.pairs:
                        end = self.pairs[c] + 1
                        break
                    c += 1
                # 只有 @compute 的是入口（wgsl_compute，与 crates/singlefs-unit-graph 同一种认法），@vertex / @fragment 照普通 fn
                kind = "wgsl_compute" if "compute" in attributes else "wgsl_fn"
                self.add(kind, self.t(i + 1), start, end)
                i = end
                continue
            if word in ("const", "override", "alias") and self.kind(i + 1) == IDENT:
                end = self.statement_end(i)
                self.add(f"wgsl_{word}", self.t(i + 1), start, end)
                i = end
                continue
            if word == "var":
                j = i + 1
                if self.t(j) == "<":
                    depth = 0
                    while j < self.count:
                        if self.t(j) == "<":
                            depth += 1
                        elif self.t(j) == ">":
                            depth -= 1
                            if depth == 0:
                                break
                        j += 1
                    j += 1
                end = self.statement_end(i)
                self.add("wgsl_var", self.t(j), start, end)
                i = end
                continue
            if word == "struct" and self.kind(i + 1) == IDENT:
                c, end = i + 2, self.count
                while c < self.count:
                    if self.t(c) == "{" and c in self.pairs:
                        end = self.pairs[c] + 1
                        break
                    c += 1
                if self.t(end) == ";":
                    end += 1
                self.add("wgsl_struct", self.t(i + 1), start, end)
                i = end
                continue
            if word in ("enable", "requires", "diagnostic"):
                end = self.statement_end(i)
                self.add("wgsl_directive", f"{word}@{line}", start, end)
                i = end
                continue
            if word == "const_assert":
                end = self.statement_end(i)
                self.add("wgsl_const_assert", f"const_assert@{line}", start, end)
                i = end
                continue
            end = self.statement_end(i)
            self.refusals.append(Refusal("unknown-item", self.path, line, self.row(line)))
            i = max(end, i + 1)


WGSL_FILES = {}
WGSL_TOP_LEVEL_DEFINING_KINDS = frozenset(("wgsl_fn", "wgsl_compute", "wgsl_const", "wgsl_override", "wgsl_alias", "wgsl_var", "wgsl_struct"))


def wgsl_top_level_names_in(root, path):
    """一份 .wgsl 顶层条目定义的名字（fn、const、override、alias、var、struct）；读不了的交空集（读它的那一处另由 include 那一条拒算）。"""
    try:
        item = wgsl_file(root, path)
    except OSError:
        return set()
    return {unit.short for unit in item.units if unit.kind in WGSL_TOP_LEVEL_DEFINING_KINDS}


def wgsl_file(root, path):
    absolute = os.path.join(root, path)
    with open(absolute, encoding="utf-8", errors="surrogateescape") as handle:
        text = handle.read()
    key = (absolute, sha256_text(text))
    if key not in WGSL_FILES:
        WGSL_FILES[key] = WgslFile(path, text)
        # 输出行里的限定名照 crates/singlefs-unit-graph：<所在 crate 名>::<文件名>::<名字>
        identifier = crate_identifier_of(root, path)
        for unit in WGSL_FILES[key].units:
            unit.full = f"{identifier}::{posixpath.basename(path)}::{unit.name}" if identifier else None
    return WGSL_FILES[key]


CRATE_IDENTIFIERS = {}


def crate_identifier_of(root, path):
    """path 所在 crate（工作区成员里目录是它最长前缀的那个）的 crate 名：[lib] name，没有就包名把 - 换成 _；不在哪个 crate 里交 None。"""
    real_root = os.path.realpath(root)
    if real_root not in CRATE_IDENTIFIERS:
        try:
            CRATE_IDENTIFIERS[real_root] = [(crate.directory, crate.lib_name) for crate in discover_rust_crates(root)]
        except FunctionUnitsError:
            CRATE_IDENTIFIERS[real_root] = []
    best = None
    for directory, identifier in CRATE_IDENTIFIERS[real_root]:
        prefix = directory + "/" if directory else ""
        if path.startswith(prefix) and (best is None or len(directory) > len(best[0])):
            best = (directory, identifier)
    return best[1] if best else None


def literal_mentions(literal_texts, prefixes, name):
    return any(name in text for text in literal_texts) or any(name.startswith(prefix) for prefix in prefixes)


def literal_prefixes(literal_texts):
    prefixes = set()
    for text in literal_texts:
        for found in re.finditer(r"([A-Za-z_][A-Za-z0-9_]*)\{", text):
            if len(found.group(1)) >= 2:
                prefixes.add(found.group(1))
    return prefixes


def wgsl_closure(wgsl_files, entry_units=None, sources=None):
    """一个 WGSL 模块（几份 .wgsl 拼成的）从入口按名字精确收；没有入口就收全部。没定义的名字（外名）要在 sources（WgslNameSources）里有着落，
    没有着落的拒算；sources 是 None（没给 Rust 入口）时一律拒算。→ (成员, 拒算, 给外名着落的 Rust 单元)。"""
    by_name = collections.defaultdict(list)
    all_units, refusals = [], []
    for item in wgsl_files:
        refusals += item.refusals
        for unit in item.units:
            by_name[unit.short].append(unit)
            all_units.append(unit)
    starts = list(entry_units) if entry_units else ([unit for unit in all_units if unit.kind == "wgsl_compute"] or list(all_units))
    starts += [unit for unit in all_units if unit.kind in ("wgsl_directive", "wgsl_const_assert")]
    declared = set()
    for unit in all_units:
        declared |= unit.declared
    seen, order, undefined = set(), [], {}
    todo = list(starts)
    while todo:
        unit = todo.pop()
        if id(unit) in seen:
            continue
        seen.add(id(unit))
        order.append(unit)
        for name in unit.plain:
            if name in by_name:
                todo.extend(by_name[name])
            elif name not in WGSL_PREDECLARED and name not in declared and name not in undefined:
                undefined[name] = (unit.file, unit.cache.get(name, unit.first_line))
    suppliers = {}
    for name in sorted(undefined):
        if sources is not None:
            supplied, units = sources.suppliers(name)
            if supplied:
                for unit in units:
                    suppliers[id(unit)] = unit
                continue
        file, line = undefined[name]
        refusals.append(Refusal("wgsl-undefined-name", file, line, name))
    return order, refusals, list(suppliers.values())


def wgsl_units(root):
    out = []
    for crate in discover_rust_crates(root):
        for path in files_under(root, (".wgsl",), [crate.directory or "."]):
            out.append(wgsl_file(root, path))
    return out


# ── Python ──────────────────────────────────────────────────────────────────

SUBPROCESS_CALLS = frozenset(("subprocess.run", "subprocess.call", "subprocess.check_call", "subprocess.check_output", "subprocess.Popen"))
EXEC_CALLS = frozenset(("os.execv", "os.execvp", "os.execve", "os.execvpe"))   # 第二个参数是 argv
SHELL_STRING_CALLS = frozenset(("os.system", "os.popen"))                       # 参数是一段交给 sh -c 的串
SOURCE_READERS = frozenset(("open", "io.open", "tokenize.open", "inspect.getsource", "inspect.getsourcelines", "linecache.getlines"))
PATH_TRANSFORMS = frozenset(("os.path.dirname", "os.path.abspath", "os.path.realpath", "os.path.normpath", "os.path.basename",
                             "posixpath.dirname", "posixpath.normpath", "posixpath.basename", "str", "os.fspath"))
PATH_CONSTRUCTORS = frozenset(("pathlib.Path", "pathlib.PurePath", "pathlib.PurePosixPath", "pathlib.PosixPath"))
PYTHON_BUILTIN_CALLS = frozenset(("open", "str", "getattr", "globals", "__import__"))


def is_docstring(statement):
    return isinstance(statement, ast.Expr) and isinstance(statement.value, ast.Constant) and isinstance(statement.value.value, str)


def without_docstrings(node, keep):
    if keep:
        return node
    clone = copy.deepcopy(node)
    for inner in ast.walk(clone):
        if isinstance(inner, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef, ast.Module)) and inner.body and is_docstring(inner.body[0]):
            inner.body = inner.body[1:]
    return clone


def dump_fingerprint(node):
    return sha256_text(ast.dump(node, include_attributes=False))


def assignment_names(statement):
    targets = statement.targets if isinstance(statement, ast.Assign) else [statement.target]
    names = []
    for target in targets:
        if isinstance(target, ast.Name):
            names.append(target.id)
        elif isinstance(target, (ast.Tuple, ast.List)):
            names += [element.id for element in target.elts if isinstance(element, ast.Name)]
    return list(dict.fromkeys(names))


def import_bound_names(statement):
    names = []
    for alias in statement.names:
        if isinstance(statement, ast.Import):
            names.append(alias.asname or alias.name.split(".")[0])
        else:
            names.append(alias.asname or alias.name)
    return names


def is_main_guard(statement):
    if not isinstance(statement, ast.If):
        return False
    test = statement.test
    return (isinstance(test, ast.Compare) and isinstance(test.left, ast.Name) and test.left.id == "__name__" and len(test.comparators) == 1
            and isinstance(test.comparators[0], ast.Constant) and test.comparators[0].value == "__main__")


def stored_names(node):
    names = set()
    for inner in ast.walk(node):
        if isinstance(inner, ast.Name) and isinstance(inner.ctx, ast.Store):
            names.add(inner.id)
        elif isinstance(inner, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            names.add(inner.name)
        elif isinstance(inner, (ast.Import, ast.ImportFrom)):
            names.update(import_bound_names(inner))
    return names


def dotted_name(node):
    parts = []
    while isinstance(node, ast.Attribute):
        parts.append(node.attr)
        node = node.value
    if isinstance(node, ast.Name):
        parts.append(node.id)
        return ".".join(reversed(parts))
    return None


class ElementsOf(ast.AST):
    """局部赋值表里的一项：for 循环变量取的是 sequence 里的元素（折叠时取遍那几个元素）。"""
    _fields = ("sequence",)


def bound_names(target):
    """赋值目标里重新绑定的名字：Name、拆包（Tuple / List / Starred）里的 Name；下标与属性目标不绑名字。"""
    if isinstance(target, ast.Name):
        return [target.id]
    if isinstance(target, ast.Starred):
        return bound_names(target.value)
    if isinstance(target, (ast.Tuple, ast.List)):
        return [name for element in target.elts for name in bound_names(element)]
    return []


def walk_nodes(nodes):
    for node in nodes:
        yield from ast.walk(node)


def unique_alternatives(alternatives):
    """命令的几种取法去重（同一串元素节点算一种），次序不变。"""
    out, seen = [], set()
    for alternative in alternatives:
        key = tuple(id(item[0]) if isinstance(item, tuple) else id(item) for item in alternative)
        if key not in seen:
            seen.add(key)
            out.append(alternative)
    return out


def paired_elements(target, value):
    """a, b = x, y：→ [(名字, 对上的值节点)…]；两边不是等长的元组 / 列表、有 *、目标里有不是名字的 → None。"""
    if not (isinstance(target, (ast.Tuple, ast.List)) and isinstance(value, (ast.Tuple, ast.List)) and len(target.elts) == len(value.elts)):
        return None
    if any(not isinstance(item, ast.Name) for item in target.elts) or any(isinstance(item, ast.Starred) for item in value.elts):
        return None
    return [(item.id, element) for item, element in zip(target.elts, value.elts)]


OWN_SCOPES = (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef, ast.Lambda, ast.ListComp, ast.SetComp, ast.DictComp, ast.GeneratorExp)


def walk_function_statements(function):
    """函数体里的每个节点，照源码次序，不进套在里面的函数、类、lambda 与推导式（它们自成作用域）。"""
    def visit(node):
        yield node
        for child in ast.iter_child_nodes(node):
            if not isinstance(child, OWN_SCOPES):
                yield from visit(child)
    for statement in function.body:
        if not isinstance(statement, OWN_SCOPES):
            yield from visit(statement)


def walk_targets(statement):
    """for 的循环变量、with 的 as 绑的那几个名字。"""
    targets = [statement.target] if isinstance(statement, (ast.For, ast.AsyncFor)) else [item.optional_vars for item in statement.items if item.optional_vars]
    for target in targets:
        yield from (inner for inner in ast.walk(target) if isinstance(inner, ast.Name))


class PythonFacts:
    """inline_scripts：起的内联脚本（InlineScript，python 的在闭包里切成单元、shell 的当一段 bash 认）；whole_modules：整份连上的模块
    （-m unittest 点名的测试模块）；handled：每个起子进程的调用各记一条认成了什么（只给量全仓的人看，不进判定）。"""
    __slots__ = ("names", "attributes", "loads", "program_links", "doc_reads", "source_read", "strings", "refusals", "import_targets", "inline_scripts",
                 "whole_modules", "handled")

    def __init__(self):
        self.names, self.attributes, self.loads, self.program_links = set(), [], [], []
        self.doc_reads, self.source_read, self.strings, self.refusals, self.import_targets = [], False, set(), [], []
        self.inline_scripts, self.whole_modules, self.handled = [], [], []


class PythonIndex:
    def __init__(self, root):
        self.root = os.path.abspath(root)
        self.breaks = active_breaks()
        self.modules = {}
        self.folding = set()
        self._repository_names = None

    def module(self, path):
        if path not in self.modules:
            self.modules[path] = PythonModule(self, path) if os.path.isfile(os.path.join(self.root, path)) else None
        return self.modules[path]

    def inline_modules(self, script):
        """InlineScript（python 的）→ 每份正文一个 PythonModule：单元都标成内联（闭包穿过它们连下去、自己不进闭包，正文已在起它的那个单元里），
        sys.argv[k] 折成 script.argv[k-1]，import 先在 script.bases（工作目录）里找，拒算落在起它的那一行。"""
        key = ("inline", script)
        if key not in self.modules:
            self.modules[key] = [PythonModule(self, f"{script.file}#inline@{script.line}#{position}", inline=script, text=text)
                                 for position, text in enumerate(script.texts)]
        return self.modules[key]

    def repository_names(self):
        if self._repository_names is None:
            names = set()
            for directory, subdirectories, files in os.walk(self.root):
                subdirectories[:] = [name for name in subdirectories if name not in SKIPPED_DIRECTORIES]
                for name in files:
                    if name.endswith(".py"):
                        names.add(name[:-3])
                if "__init__.py" in files:
                    names.add(os.path.basename(directory))
            self._repository_names = names
        return self._repository_names

    def relative(self, absolute):
        return inside_root(self.root, absolute)


class PythonModule:
    def __init__(self, index, path, inline=None, text=None):
        self.index, self.path, self.inline = index, path, inline
        if inline is not None:
            self.absolute = posixpath.join(inline.bases[0], "<inline>")
            self.raw = text.encode("utf-8", "surrogateescape")
        else:
            self.absolute = os.path.join(index.root, path)
            with open(self.absolute, "rb") as handle:
                self.raw = handle.read()
        self.text = self.raw.decode("utf-8", "surrogateescape")
        self.rows = self.text.split("\n")
        self.units, self.refusals, self.methods = [], [], []
        self.by_name = collections.defaultdict(list)
        self.body_defines, self.docstrings = set(), {}
        self.module_body = self.main_guard = None
        self.star_imports = []
        self._bindings = self._call_sites = self._sys_path = None
        self._data = {}
        breaks = index.breaks
        self.keep_comments = "python-comments-kept" in breaks
        self.keep_docstrings = "python-docstrings-kept" in breaks
        try:
            self.tree = ast.parse(self.raw, filename=path)
        except (SyntaxError, ValueError) as error:
            line = getattr(error, "lineno", None) or 1
            self.refusals.append(self.refusal("python-syntax", line))
            self.tree = None
            body_unit = Unit("python", "module-body", "<module-body>", path, 1, 1, sha256_bytes(self.raw))
            body_unit.module, body_unit.nodes = self, []
            self.module_body = body_unit
            self.units.append(body_unit)
        else:
            self.split()
        if inline is not None:
            for unit in self.units:
                unit.scope, unit.file = "inline", inline.file

    def row(self, line):
        if self.inline is not None:
            return self.inline.snippet
        return snippet_of(self.rows[line - 1]) if 0 < line <= len(self.rows) else ""

    def refusal(self, form, line):
        """拒算落在哪：内联脚本的落在起它的那一行（它的正文就在那个单元里），别的落在这份文件的那一行。"""
        if self.inline is not None:
            return Refusal(form, self.inline.file, self.inline.line, self.inline.snippet)
        return Refusal(form, self.path, line, self.row(line))

    def add_unit(self, kind, name, node, nodes, fingerprint, container=None, short=None):
        unit = Unit("python", kind, name, self.path, node.lineno, getattr(node, "end_lineno", node.lineno) or node.lineno, fingerprint)
        decorators = getattr(node, "decorator_list", None)
        if decorators:
            unit.first_line = min(unit.first_line, min(decorator.lineno for decorator in decorators))
        unit.node, unit.nodes, unit.module, unit.container = node, nodes, self, container
        unit.short = short or name
        self.units.append(unit)
        if container is None and kind != "main-guard":
            self.by_name[name].append(unit)
        return unit

    def function_fingerprint(self, node):
        if self.keep_comments:
            return sha256_text(ast.get_source_segment(self.text, node) or "")
        return dump_fingerprint(without_docstrings(node, self.keep_docstrings))

    def note_docstring(self, owner, node):
        if node.body and is_docstring(node.body[0]):
            statement = node.body[0]
            self.docstrings[owner] = (statement.value.value, statement.lineno, statement.end_lineno or statement.lineno)

    def split(self):
        body = list(self.tree.body)
        if body and is_docstring(body[0]):
            statement = body[0]
            self.docstrings["<module>"] = (statement.value.value, statement.lineno, statement.end_lineno or statement.lineno)
            body = body[1:]
        rest = []
        for statement in body:
            if isinstance(statement, (ast.FunctionDef, ast.AsyncFunctionDef)):
                self.add_unit("def", statement.name, statement, [statement], self.function_fingerprint(statement))
                self.note_docstring(statement.name, statement)
            elif isinstance(statement, ast.ClassDef):
                head_body = [member for member in statement.body if not isinstance(member, (ast.FunctionDef, ast.AsyncFunctionDef))]
                clone = copy.copy(statement)
                clone.body = head_body
                head = self.add_unit("class", statement.name, statement,
                                     list(statement.decorator_list) + list(statement.bases) + list(statement.keywords) + head_body,
                                     dump_fingerprint(without_docstrings(clone, self.keep_docstrings)))
                self.note_docstring(statement.name, statement)
                for member in statement.body:
                    if isinstance(member, (ast.FunctionDef, ast.AsyncFunctionDef)):
                        method = self.add_unit("def", f"{statement.name}.{member.name}", member, [member], self.function_fingerprint(member),
                                               container=head, short=member.name)
                        self.note_docstring(method.name, member)
                        self.methods.append(method)
            elif isinstance(statement, (ast.Assign, ast.AnnAssign, ast.AugAssign)) and assignment_names(statement):
                for name in assignment_names(statement):
                    self.add_unit("assign", name, statement, [statement], dump_fingerprint(statement))
            elif isinstance(statement, (ast.Import, ast.ImportFrom)):
                for name in import_bound_names(statement):
                    unit = self.add_unit("import", name, statement, [statement], dump_fingerprint(statement))
                    if name == "*":
                        self.star_imports.append(unit)
            elif is_main_guard(statement):
                self.main_guard = self.add_unit("main-guard", "<main-guard>", statement, [statement], dump_fingerprint(statement))
            else:
                rest.append(statement)
        module = ast.Module(body=without_docstrings(ast.Module(body=rest, type_ignores=[]), self.keep_docstrings).body, type_ignores=[])
        first = rest[0].lineno if rest else 1
        last = (rest[-1].end_lineno or rest[-1].lineno) if rest else 1
        self.module_body = Unit("python", "module-body", "<module-body>", self.path, first, last, dump_fingerprint(module))
        self.module_body.node, self.module_body.nodes, self.module_body.module = None, rest, self
        self.units.append(self.module_body)
        for statement in rest:
            self.body_defines |= stored_names(statement)

    def program_entries(self):
        return [unit for unit in (self.main_guard, self.module_body) if unit is not None]

    # 绑定与解析

    def bindings(self):
        if self._bindings is None:
            self._bindings = {}
            if self.tree is not None:
                statements = [unit.node for unit in self.units if unit.kind == "import"] + list(self.module_body.nodes)
                for node in walk_nodes(statements):
                    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda)):
                        continue
                    self.bind(node, self._bindings)
        return self._bindings

    @staticmethod
    def bind(node, table):
        if isinstance(node, ast.Import):
            for alias in node.names:
                if alias.asname:
                    table[alias.asname] = ("import", alias.name, 0, node)
                else:
                    table[alias.name.split(".")[0]] = ("import", alias.name.split(".")[0], 0, node)
        elif isinstance(node, ast.ImportFrom):
            for alias in node.names:
                if alias.name != "*":
                    table[alias.asname or alias.name] = ("from", node.module or "", node.level, node, alias.name)

    def local_bindings(self, unit):
        if unit.cache is None:
            unit.cache = {}
        if "bindings" not in unit.cache:
            table = {}
            if unit.kind in ("def", "class", "main-guard"):
                for node in walk_nodes(unit.nodes):
                    self.bind(node, table)
            unit.cache["bindings"] = table
        return unit.cache["bindings"]

    def binding(self, name, unit):
        local = self.local_bindings(unit).get(name) if unit is not None else None
        return local if local is not None else self.bindings().get(name)

    def search_directories(self):
        # 内联脚本（python3 -c、python3 - <<'PY'）的 sys.path[0] 是工作目录
        directories = list(self.inline.bases) if self.inline is not None else [posixpath.dirname(self.absolute)]
        if self._sys_path is None:
            self._sys_path = []
            if self.tree is not None:
                self._sys_path = ["computing"]
                found = []
                for unit in self.units:
                    for node in walk_nodes(unit.nodes):
                        arguments = []
                        if isinstance(node, ast.Call):
                            canonical = self.canonical(node.func, unit)
                            if canonical == "sys.path.insert" and len(node.args) >= 2:
                                arguments = [node.args[1]]
                            elif canonical == "sys.path.append" and node.args:
                                arguments = [node.args[0]]
                            elif canonical == "sys.path.extend" and node.args and isinstance(node.args[0], (ast.List, ast.Tuple)):
                                arguments = list(node.args[0].elts)
                        elif isinstance(node, ast.Assign) and isinstance(node.value, (ast.List, ast.Tuple)) and any(
                                isinstance(target, ast.Subscript) and self.canonical(target.value, unit) == "sys.path" for target in node.targets):
                            arguments = list(node.value.elts)
                        for argument in arguments:
                            values = self.path_values(argument, unit, directory=True)
                            if values:
                                found += sorted(values)
                self._sys_path = found
        return directories + [path for path in self._sys_path if path != "computing"]

    def resolve_module(self, dotted, level):
        """→ ("repo", 文件) / ("stdlib", None) / ("external", None) / ("unresolved", None)。"""
        parts = [part for part in dotted.split(".") if part] if dotted else []
        if level:
            base = posixpath.dirname(self.absolute)
            for _step in range(level - 1):
                base = posixpath.dirname(base)
            bases = [base]
        else:
            bases = self.search_directories()
        for base in bases:
            candidate = posixpath.join(base, *parts) if parts else base
            for option in (candidate + ".py", posixpath.join(candidate, "__init__.py")):
                if parts or option.endswith("__init__.py"):
                    if os.path.isfile(option):
                        relative = self.index.relative(option)
                        if relative is not None:
                            return ("repo", relative)
        if level:
            return ("unresolved", None)
        top = parts[0] if parts else ""
        if top in sys.stdlib_module_names or top in sys.builtin_module_names:
            return ("stdlib", None)
        if top in self.index.repository_names():
            return ("unresolved", None)
        return ("external", None)

    def all_resolutions(self, dotted, level):
        """同一个名字在各个搜索目录里各解到的仓内文件（sys.path 里插的目录折出几个，运行期是其中哪一个都可能）；resolve_module 只交第一个。"""
        if level or "python-imports-first-directory-only" in self.index.breaks:
            status, file = self.resolve_module(dotted, level)
            return [file] if status == "repo" else []
        parts = [part for part in dotted.split(".") if part] if dotted else []
        found = []
        for base in self.search_directories():
            candidate = posixpath.join(base, *parts) if parts else base
            for option in (candidate + ".py", posixpath.join(candidate, "__init__.py")):
                if (parts or option.endswith("__init__.py")) and os.path.isfile(option):
                    relative = self.index.relative(option)
                    if relative is not None and relative not in found:
                        found.append(relative)
                    break
        return found

    def canonical(self, node, unit):
        if isinstance(node, ast.Name):
            binding = self.binding(node.id, unit)
            if binding is None:
                return node.id if node.id in PYTHON_BUILTIN_CALLS else None
            if binding[0] == "import":
                return binding[1]
            if binding[2]:
                return None
            return f"{binding[1]}.{binding[4]}"
        if isinstance(node, ast.Attribute):
            base = self.canonical(node.value, unit)
            return f"{base}.{node.attr}" if base else None
        return None

    def binding_targets(self, binding, unit, facts):
        """一个绑定 → 它指到的仓内单元（那份模块的 module-body 与同名单元）；仓内认不出的记拒算。"""
        node = binding[3]
        if binding[0] == "import":
            status, file = self.resolve_module(binding[1], 0)
            if status == "unresolved":
                facts.refusals.append(self.refusal("import-unresolved", node.lineno))
            if status != "repo":
                return []
            targets = []
            for each in self.all_resolutions(binding[1], 0) or [file]:
                module = self.index.module(each)
                targets += module.program_entries()[-1:] if module else []
            return targets
        status, file = self.resolve_module(binding[1], binding[2])
        if status == "unresolved":
            facts.refusals.append(self.refusal("import-unresolved", node.lineno))
        if status != "repo":
            return []
        others = [each for each in self.all_resolutions(binding[1], binding[2]) if each != file]
        targets = []
        for each in others:
            other_module = self.index.module(each)
            if other_module is not None:
                targets += [other_module.module_body] + list(other_module.by_name.get(binding[4], ()))
        module = self.index.module(file)
        if module is None:
            return targets
        attribute = binding[4]
        targets += [module.module_body] + list(module.by_name.get(attribute, ()))
        if attribute in module.body_defines:
            targets.append(module.module_body)
        sub_status, sub_file = self.resolve_module(f"{binding[1]}.{attribute}" if binding[1] else attribute, binding[2])
        if sub_status == "repo" and sub_file != file:
            sub_module = self.index.module(sub_file)
            if sub_module is not None:
                targets.append(sub_module.module_body)
        return targets

    def module_of_binding(self, binding):
        if binding is None:
            return None
        if binding[0] == "import":
            status, file = self.resolve_module(binding[1], 0)
        else:
            dotted = f"{binding[1]}.{binding[4]}" if binding[1] else binding[4]
            status, file = self.resolve_module(dotted, binding[2])
        return self.index.module(file) if status == "repo" else None

    def resolve_name(self, name, unit, facts):
        binding = self.local_bindings(unit).get(name)
        if binding is not None:
            return self.binding_targets(binding, unit, facts)
        found = [candidate for candidate in self.by_name.get(name, ()) if candidate is not unit]
        if found:
            return found
        if name in self.body_defines and unit is not self.module_body:
            return [self.module_body]
        out = []
        for star in self.star_imports:
            for target in self.binding_targets(("from", star.node.module or "", star.node.level, star.node, "*"), unit, facts):
                module = target.module
                if module is not None:
                    out += list(module.by_name.get(name, ()))
        return out

    def resolve_attribute(self, base, attribute, unit, facts):
        """base.attribute：base 是 import 进来的仓内模块时 → 那份模块的同名单元；不是 → None（交 RTA）。"""
        if not base:
            return None
        parts = base.split(".")
        binding = self.binding(parts[0], unit)
        first = self.module_of_binding(binding)
        if first is None:
            return None
        modules = [first]
        if binding[0] == "import" and len(parts) == 1:
            # 同一个模块名在 sys.path 插的几个目录里各有一份：每一份的同名单元都连
            modules += [self.index.module(file) for file in self.all_resolutions(binding[1], 0) if file != first.path]
        targets = []
        for module in modules:
            if module is None:
                continue
            for part in parts[1:]:
                status, file = module.resolve_module(part, 1) if module.path.endswith("__init__.py") else ("unresolved", None)
                if status != "repo":
                    return None
                module = self.index.module(file)
                if module is None:
                    return None
            targets += [module.module_body] + list(module.by_name.get(attribute, ()))
            if module.path.endswith("__init__.py"):
                status, file = module.resolve_module(attribute, 1)
                if status == "repo":
                    sub_module = self.index.module(file)
                    if sub_module is not None:
                        targets.append(sub_module.module_body)
        return targets

    # 折叠

    def local_assignments(self, unit):
        if unit.cache is None:
            unit.cache = {}
        if "assignments" not in unit.cache:
            result = collections.defaultdict(list)
            if isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                stack = list(unit.node.body)
                own_scope = (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef, ast.Lambda)
                if "python-comprehension-scope-leaks" not in self.index.breaks:
                    # 推导式自成一层作用域：里面的循环变量不是这个函数的赋值
                    own_scope += (ast.ListComp, ast.SetComp, ast.DictComp, ast.GeneratorExp)
                while stack:
                    node = stack.pop()
                    if isinstance(node, own_scope):
                        continue
                    if isinstance(node, ast.Assign):
                        for target in node.targets:
                            if isinstance(target, ast.Name):
                                result[target.id].append(node.value)
                            elif paired_elements(target, node.value) is not None:
                                # a, b = x, y：逐个对上
                                for name, value in paired_elements(target, node.value):
                                    result[name].append(value)
                            else:
                                # 拆包（a, b = …）绑的是里面的名字；x[k] = …、x.y = … 只改 x 那个对象，不重新绑 k、x
                                rebound = ([inner.id for inner in ast.walk(target) if isinstance(inner, ast.Name)]
                                           if "python-subscript-targets-rebind" in self.index.breaks else bound_names(target))
                                for inner in rebound:
                                    result[inner].append(None)
                    elif isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
                        result[node.target.id].append(node.value)
                    elif isinstance(node, ast.AugAssign) and isinstance(node.target, ast.Name):
                        result[node.target.id].append(None)
                    elif isinstance(node, (ast.For, ast.AsyncFor, ast.comprehension)):
                        if isinstance(node.target, ast.Name) and "python-loop-variables-unfolded" not in self.index.breaks:
                            # for x in <元组 / 列表，或绑着元组 / 列表的名字>：x 取遍那几个元素
                            result[node.target.id].append(ElementsOf(sequence=node.iter))
                        else:
                            for inner in ast.walk(node.target):
                                if isinstance(inner, ast.Name):
                                    result[inner.id].append(None)
                    elif isinstance(node, (ast.With, ast.AsyncWith)):
                        for item in node.items:
                            if item.optional_vars is not None:
                                for inner in ast.walk(item.optional_vars):
                                    if isinstance(inner, ast.Name):
                                        result[inner.id].append(None)
                    elif isinstance(node, ast.NamedExpr):
                        result[node.target.id].append(node.value)
                    stack.extend(ast.iter_child_nodes(node))
            unit.cache["assignments"] = result
        return unit.cache["assignments"]

    def call_sites(self):
        if self._call_sites is None:
            self._call_sites = collections.defaultdict(list)
            for unit in self.units:
                for node in walk_nodes(unit.nodes or []):
                    if isinstance(node, ast.Call) and isinstance(node.func, ast.Name):
                        self._call_sites[node.func.id].append((node, unit))
                    elif isinstance(node, ast.Call) and "python-dispatch-calls-ignored" not in self.index.breaks:
                        # 分派表[键](…) / 分派表.get(键)(…)：表（模块级只赋过一次的字典字面量）里当值写的每个函数都算被这一处调了
                        table = node.func.value if isinstance(node.func, ast.Subscript) else (
                            node.func.func.value if isinstance(node.func, ast.Call) and isinstance(node.func.func, ast.Attribute)
                            and node.func.func.attr == "get" else None)
                        found = self.module_dictionary(table.id, None) if isinstance(table, ast.Name) else None
                        if found is not None:
                            for value in found[0].values:
                                if isinstance(value, ast.Name):
                                    self._call_sites[value.id].append((node, unit))
        return self._call_sites

    def fold_parameter(self, unit, name, depth):
        function = unit.node
        if unit.container is not None or not isinstance(function, (ast.FunctionDef, ast.AsyncFunctionDef)):
            return None
        arguments = function.args
        positional = [argument.arg for argument in arguments.posonlyargs + arguments.args]
        keyword_only = [argument.arg for argument in arguments.kwonlyargs]
        defaults = {}
        for argument, default in zip(positional[len(positional) - len(arguments.defaults):], arguments.defaults):
            defaults[argument] = default
        for argument, default in zip(keyword_only, arguments.kw_defaults):
            if default is not None:
                defaults[argument] = default
        if name not in positional and name not in keyword_only:
            return None
        position = positional.index(name) if name in positional else None
        sites = self.call_sites().get(function.name, [])
        if not sites:
            return None
        values = set()
        for call, caller in sites:
            argument = None
            if position is not None and position < len(call.args):
                argument = call.args[position]
                if any(isinstance(item, ast.Starred) for item in call.args[:position + 1]):
                    return None
            for keyword in call.keywords:
                if keyword.arg is None:
                    return None
                if keyword.arg == name:
                    argument = keyword.value
            if argument is None:
                argument = defaults.get(name)
            if argument is None:
                return None
            folded = self.fold(argument, caller, depth + 1)
            if folded is None:
                return None
            values |= folded
        return values

    def fold_name(self, name, unit, depth):
        if name == "__file__":
            return None if self.inline is not None else {self.absolute}
        key = (self.path, id(unit), name)
        if key in self.index.folding:
            return None
        self.index.folding.add(key)
        try:
            if unit is not None and isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                local = self.local_assignments(unit)
                if name in local:
                    return self.fold_all(local[name], unit, depth)
                arguments = unit.node.args
                if name in [argument.arg for argument in arguments.posonlyargs + arguments.args + arguments.kwonlyargs]:
                    return self.fold_parameter(unit, name, depth)
            values, found = set(), False
            for candidate in self.by_name.get(name, ()):
                if candidate.kind == "assign":
                    statement = candidate.node
                    value = statement.value if isinstance(statement, (ast.Assign, ast.AnnAssign)) and len(assignment_names(statement)) == 1 else None
                    if value is None and isinstance(statement, ast.Assign) and len(statement.targets) == 1:
                        # 模块级的 a, b = x, y：取对上 name 的那一个
                        value = dict(paired_elements(statement.targets[0], statement.value) or ()).get(name)
                        if value is not None:
                            folded = self.fold(value, candidate, depth + 1)
                            if folded is None:
                                return None
                            values |= folded
                            found = True
                            continue
                    if isinstance(statement, ast.Assign) and not all(isinstance(target, ast.Name) for target in statement.targets):
                        value = None
                    folded = self.fold(value, candidate, depth + 1) if value is not None else None
                    if folded is None:
                        return None
                    values |= folded
                    found = True
                elif candidate.kind == "import":
                    binding = self.bindings().get(name)
                    if binding is None or binding[0] != "from":
                        return None
                    status, file = self.resolve_module(binding[1], binding[2])
                    other = self.index.module(file) if status == "repo" else None
                    if other is None:
                        return None
                    folded = other.fold_name(binding[4], other.module_body, depth + 1)
                    if folded is None:
                        return None
                    values |= folded
                    found = True
                else:
                    return None
            if not found and name in self.body_defines:
                for node in walk_nodes(self.module_body.nodes):
                    if isinstance(node, ast.Assign) and any(isinstance(target, ast.Name) and target.id == name for target in node.targets):
                        folded = self.fold(node.value, self.module_body, depth + 1)
                        if folded is None:
                            return None
                        values |= folded
                        found = True
            return values if found else None
        finally:
            self.index.folding.discard(key)

    def fold_all(self, nodes, unit, depth):
        values = set()
        for node in nodes:
            if node is None:
                return None
            folded = self.fold(node, unit, depth + 1)
            if folded is None:
                return None
            values |= folded
        return values

    def fold(self, node, unit, depth=0):
        """表达式 → 它可能取到的字符串值（路径按 posix 规范化）；折叠不出 → None。"""
        if depth > 16 or node is None:
            return None
        values = self.fold_inner(node, unit, depth)
        if values is not None and len(values) > FOLD_LIMIT:
            return None
        return values

    def module_dictionary(self, name, unit):
        """name 是本模块里只赋过一次（模块级、不在函数里另赋）的字典字面量 → (那个 ast.Dict, 它的赋值单元)；别的 → None。"""
        if unit is not None and isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)) and name in self.local_assignments(unit):
            return None
        candidates = [candidate for candidate in self.by_name.get(name, ()) if candidate.kind == "assign"]
        if len(candidates) != 1 or not isinstance(candidates[0].node, ast.Assign) or not isinstance(candidates[0].node.value, ast.Dict):
            return None
        if name in self.body_defines:
            return None
        return candidates[0].node.value, candidates[0]

    def fold_elements(self, sequence, unit, depth):
        """元组 / 列表字面量，或绑着它的名字（本函数里只赋过一次的、模块级的）→ 各元素折出来的并；有一个折不出就整个折不出。"""
        if isinstance(sequence, ast.Name):
            assigned = self.local_assignments(unit).get(sequence.id, []) if unit is not None and isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)) else []
            module, owner = self, unit
            if not assigned:
                candidates = [candidate for candidate in self.by_name.get(sequence.id, ()) if candidate.kind == "assign"]
                assigned = [candidate.node.value for candidate in candidates if isinstance(candidate.node, ast.Assign)]
                owner = candidates[0] if len(candidates) == 1 else None
            if len(assigned) != 1 or not isinstance(assigned[0], (ast.Tuple, ast.List)):
                return None
            return module.fold_elements(assigned[0], owner, depth + 1)
        if not isinstance(sequence, (ast.Tuple, ast.List)):
            return None
        values = set()
        for element in sequence.elts:
            folded = self.fold(element, unit, depth + 1)
            if folded is None:
                return None
            values |= folded
        return values

    def fold_inner(self, node, unit, depth):
        if isinstance(node, ElementsOf):
            return self.fold_elements(node.sequence, unit, depth)
        if isinstance(node, ast.Constant):
            return {node.value} if isinstance(node.value, str) else None
        if isinstance(node, ast.JoinedStr):
            results = {""}
            for part in node.values:
                if isinstance(part, ast.Constant):
                    results = {result + str(part.value) for result in results}
                elif isinstance(part, ast.FormattedValue) and part.format_spec is None and part.conversion in (-1, 115):
                    folded = self.fold(part.value, unit, depth + 1)
                    if folded is None:
                        return None
                    results = {result + value for result in results for value in folded}
                else:
                    return None
            return results
        known = self.inline_environment(node, unit)
        if known is not None:
            return set(known)
        if isinstance(node, ast.BoolOp) and isinstance(node.op, ast.Or) and "python-environment-defaults-unfolded" not in self.index.breaks:
            # os.environ.get("X") or <缺省>：环境变量顶替的那一边照 shell 的 ${X:-缺省} 办，按缺省折；别的每一边都要折得出
            values = set()
            for operand in node.values:
                if self.environment_read(operand, unit):
                    continue
                folded = self.fold(operand, unit, depth + 1)
                if folded is None:
                    return None
                values |= folded
            return values or None
        if isinstance(node, ast.BinOp) and isinstance(node.op, (ast.Add, ast.Div)):
            left, right = self.fold(node.left, unit, depth + 1), self.fold(node.right, unit, depth + 1)
            if left is None or right is None:
                return None
            if isinstance(node.op, ast.Add):
                return {a + b for a in left for b in right}
            return {posixpath.normpath(posixpath.join(a, b)) for a in left for b in right}
        if isinstance(node, ast.Name):
            return self.fold_name(node.id, unit, depth)
        if isinstance(node, ast.Attribute) and node.attr == "stdout" and isinstance(node.value, ast.Call):
            return self.git_output(node.value, unit)
        if isinstance(node, ast.Attribute) and isinstance(node.value, ast.Name) and "python-module-constants-unfolded" not in self.index.breaks:
            # import 进来的仓内模块的模块级常量：module.NAME 按那份模块里 NAME 的赋值折
            other = self.module_of_binding(self.binding(node.value.id, unit))
            if other is not None and other is not self:
                return other.fold_name(node.attr, other.module_body, depth + 1)
        if isinstance(node, ast.Attribute):
            if node.attr in ("parent", "name", "stem"):
                folded = self.fold(node.value, unit, depth + 1)
                if folded is None:
                    return None
                if node.attr == "parent":
                    return {posixpath.dirname(value) for value in folded}
                if node.attr == "name":
                    return {posixpath.basename(value) for value in folded}
                return {posixpath.splitext(posixpath.basename(value))[0] for value in folded}
            return None
        if self.inline is not None and isinstance(node, ast.Subscript) and self.canonical(node.value, unit) == "sys.argv" and \
                isinstance(node.slice, ast.Constant) and type(node.slice.value) is int and node.slice.value >= 1:
            # 内联脚本的 sys.argv[k]：起它的那条命令里脚本之后的第 k 个词
            position = node.slice.value - 1
            values = self.inline.argv[position] if position < len(self.inline.argv) else None
            return set(values) if values is not None and values is not POSITIONAL else None
        if isinstance(node, ast.Subscript) and isinstance(node.value, ast.Name) and "python-path-tables-unfolded" not in self.index.breaks:
            # 路径表（模块级只赋过一次的字典字面量）[键]：键折得出取那几项，折不出取遍全部值
            found = self.module_dictionary(node.value.id, unit)
            if found is not None:
                table, owner = found
                keys = self.fold(node.slice, unit, depth + 1)
                if any(key is None for key in table.keys):
                    return None
                chosen = [value for key, value in zip(table.keys, table.values)
                          if keys is None or (isinstance(key, ast.Constant) and key.value in keys)]
                values = set()
                for value in chosen:
                    folded = self.fold(value, owner, depth + 1)
                    if folded is None:
                        return None
                    values |= folded
                return values or None
        if isinstance(node, ast.Subscript) and isinstance(node.value, ast.Attribute) and node.value.attr == "parents":
            index = node.slice
            if isinstance(index, ast.Constant) and isinstance(index.value, int):
                folded = self.fold(node.value.value, unit, depth + 1)
                if folded is None:
                    return None
                out = set()
                for value in folded:
                    for _step in range(index.value + 1):
                        value = posixpath.dirname(value)
                    out.add(value)
                return out
            return None
        if isinstance(node, ast.Call):
            canonical = self.canonical(node.func, unit)
            if canonical == "subprocess.check_output":
                return self.git_output(node, unit)
            if canonical == "shutil.which" and len(node.args) == 1 and isinstance(node.args[0], ast.Constant) and isinstance(node.args[0].value, str):
                # PATH 上找到的那一个程序：名字就是给的这个（装不装仓里的代码按名字认）
                return {node.args[0].value}
            if isinstance(node.func, ast.Name) and (node.args or node.keywords) and "python-call-returns-unfolded" not in self.index.breaks:
                # 带参数调仓内模块级函数：按它各处 return 的值折，参数按它的每一处调用给的实参折（不分是哪一处调用）
                returned = self.fold_call_return(node.func.id, unit, depth)
                if returned is not None:
                    return returned
            if isinstance(node.func, ast.Attribute) and node.func.attr in ("strip", "rstrip", "decode") and not node.keywords and (
                    not node.args or (node.func.attr == "decode" and len(node.args) == 1 and isinstance(node.args[0], ast.Constant))):
                return self.fold(node.func.value, unit, depth + 1)
            if node.keywords:
                return None
            if canonical in ("os.path.join", "posixpath.join") or canonical in PATH_CONSTRUCTORS:
                results = {""}
                for argument in node.args:
                    folded = self.fold(argument, unit, depth + 1)
                    if folded is None:
                        return None
                    results = {posixpath.join(result, value) if result else value for result in results for value in folded}
                return {posixpath.normpath(value) for value in results} if node.args else None
            if canonical in PATH_TRANSFORMS and len(node.args) == 1:
                folded = self.fold(node.args[0], unit, depth + 1)
                if folded is None:
                    return None
                if canonical.endswith("dirname"):
                    return {posixpath.dirname(value) for value in folded}
                if canonical.endswith("basename"):
                    return {posixpath.basename(value) for value in folded}
                if canonical in ("str", "os.fspath"):
                    return folded
                return {posixpath.normpath(value) for value in folded}
            if isinstance(node.func, ast.Attribute):
                method = node.func.attr
                if method in ("resolve", "absolute", "as_posix") and not node.args:
                    folded = self.fold(node.func.value, unit, depth + 1)
                    return {posixpath.normpath(value) for value in folded} if folded is not None else None
                if method == "joinpath":
                    folded = self.fold(node.func.value, unit, depth + 1)
                    for argument in node.args:
                        part = self.fold(argument, unit, depth + 1)
                        if folded is None or part is None:
                            return None
                        folded = {posixpath.join(a, b) for a in folded for b in part}
                    return {posixpath.normpath(value) for value in folded} if folded is not None else None
            if isinstance(node.func, ast.Name) and not node.args:
                return self.fold_call_return(node.func.id, unit, depth)
            return None
        return None

    def fold_call_return(self, name, unit, depth):
        """不带参数调一个仓内的模块级函数（本模块的，或 from 仓内模块 import 进来的）→ 它各处 return 的值的并；
        return / return None 那几处是「没找到」的出口，不算一种取值；有一处 return 折不出就整个折不出。"""
        if "python-call-returns-unfolded" in self.index.breaks:
            return None
        binding = self.binding(name, unit)
        module = self
        if binding is not None:
            if binding[0] != "from":
                return None
            status, file = self.resolve_module(binding[1], binding[2])
            module = self.index.module(file) if status == "repo" else None
            name = binding[4]
            if module is None:
                return None
        candidates = [candidate for candidate in module.by_name.get(name, ()) if candidate.kind == "def" and candidate.container is None]
        if len(candidates) != 1:
            return None
        callee = candidates[0]
        key = (module.path, id(callee), "<return>")
        if key in self.index.folding:
            return None
        self.index.folding.add(key)
        try:
            values, stack = set(), list(callee.node.body)
            while stack:
                statement = stack.pop()
                if isinstance(statement, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef, ast.Lambda)):
                    continue
                if isinstance(statement, ast.Return):
                    if statement.value is None or (isinstance(statement.value, ast.Constant) and statement.value.value is None):
                        continue
                    folded = module.fold(statement.value, callee, depth + 1)
                    if folded is None:
                        return None
                    values |= folded
                    continue
                stack.extend(ast.iter_child_nodes(statement))
            return values or None
        finally:
            self.index.folding.discard(key)

    def inline_environment(self, node, unit):
        """内联脚本里读起它的那条命令设的环境变量（os.environ["X"]、os.environ.get("X")、os.getenv("X")）→ 那个值（frozenset）；别的 → None。"""
        if self.inline is None or not self.inline.environment or not self.environment_read(node, unit):
            return None
        key = node.slice if isinstance(node, ast.Subscript) else (node.args[0] if node.args else None)
        if not (isinstance(key, ast.Constant) and isinstance(key.value, str)):
            return None
        found = None
        for name, values in self.inline.environment:
            if name == key.value:
                found = values
        return found

    def environment_read(self, node, unit):
        """os.environ.get(…) / os.getenv(…) / os.environ[…]：读环境变量的那一个表达式。"""
        if isinstance(node, ast.Call):
            return self.canonical(node.func, unit) in ("os.environ.get", "os.getenv")
        if isinstance(node, ast.Subscript):
            return self.canonical(node.value, unit) == "os.environ"
        return False

    def written_files(self, unit):
        """这个单元自己写出来的文件：open(<表达式>, "w" / "x" / "wb" / "xb") 打开、写进去的是折得出的字面文本（只写这一样）的那几个表达式（ast.dump）。
        子进程起的脚本正是其中之一，代码就是这个单元里的那段文本（不再往里看它装了什么）。"""
        if unit.cache is None:
            unit.cache = {}
        if "written" not in unit.cache:
            written = set()
            for node in walk_nodes(unit.nodes or []):
                if not isinstance(node, (ast.With, ast.AsyncWith)):
                    continue
                for item in node.items:
                    call = item.context_expr
                    if not (isinstance(call, ast.Call) and self.canonical(call.func, unit) in ("open", "io.open") and call.args
                            and isinstance(item.optional_vars, ast.Name)):
                        continue
                    mode = call.args[1] if len(call.args) > 1 else next((keyword.value for keyword in call.keywords if keyword.arg == "mode"), None)
                    if not (isinstance(mode, ast.Constant) and mode.value in ("w", "x", "wb", "xb")):
                        continue
                    handle = item.optional_vars.id
                    writes = [inner for inner in walk_nodes(node.body) if isinstance(inner, ast.Call) and isinstance(inner.func, ast.Attribute)
                              and isinstance(inner.func.value, ast.Name) and inner.func.value.id == handle]
                    if writes and all(inner.func.attr == "write" and len(inner.args) == 1 and self.fold(inner.args[0], unit) is not None for inner in writes):
                        written.add(ast.dump(call.args[0]))
            unit.cache["written"] = written
        return unit.cache["written"]

    def git_output(self, call, unit):
        """subprocess.run([...git… rev-parse… --show-toplevel|--git-common-dir…]).stdout 与 check_output 同样的命令 → {根} / {根/.git}。"""
        if self.canonical(call.func, unit) not in SUBPROCESS_CALLS or not call.args:
            return None
        elements = self.command_elements(call.args[0], unit, False)
        if elements is None:
            return None
        words = []
        for element in elements:
            folded = self.fold(element, unit) if not isinstance(element, ast.Starred) else None
            words.append(next(iter(folded)) if folded is not None and len(folded) == 1 else None)
        return git_repository_location(words, self.index.root, self.index.breaks)

    def literal_tail(self, node, unit, depth=0):
        """路径表达式 → 「折不出的前缀之后的字面尾巴」（不带前缀；整个折不出、又没有字面尾巴 → ""）；尾巴里夹着折不出的 → None。"""
        if depth > 8 or node is None:
            return None
        folded = self.fold(node, unit)
        if folded is not None:
            return next(iter(folded)) if len(folded) == 1 else None
        if isinstance(node, ast.Call) and not node.keywords and node.args and (
                self.canonical(node.func, unit) in ("os.path.join", "posixpath.join") or self.canonical(node.func, unit) in PATH_CONSTRUCTORS):
            parts = []
            for argument in reversed(node.args):
                value = self.fold(argument, unit)
                if value is None or len(value) != 1:
                    head = self.literal_tail(argument, unit, depth + 1)
                    if head is None:
                        return None
                    parts.append(head)
                    break
                parts.append(next(iter(value)))
            else:
                return None
            return posixpath.join(*reversed(parts)) if parts else None
        if isinstance(node, ast.BinOp) and isinstance(node.op, (ast.Add, ast.Div)):
            right = self.fold(node.right, unit)
            if right is None or len(right) != 1:
                return None
            left = self.literal_tail(node.left, unit, depth + 1)
            if left is None:
                return None
            value = next(iter(right))
            return left + value if isinstance(node.op, ast.Add) else posixpath.join(left, value)
        if isinstance(node, ast.JoinedStr) and node.values and isinstance(node.values[0], ast.FormattedValue) and all(
                isinstance(part, ast.Constant) for part in node.values[1:]):
            head = self.literal_tail(node.values[0].value, unit, depth + 1)
            return None if head is None else head + "".join(str(part.value) for part in node.values[1:])
        if isinstance(node, ast.Name) and unit is not None and isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            assigned = self.local_assignments(unit).get(node.id, [])
            if len(assigned) == 1 and assigned[0] is not None:
                return self.literal_tail(assigned[0], unit, depth + 1)
        return ""

    def outside_tail(self, value):
        """一个程序路径的字面尾巴（os.path.join(<折不出的目录>, "bin/x") 的 "bin/x"）是不是根外的程序：不带脚本后缀，根里也没有这一截。"""
        tail = value.strip("/")
        if not tail or tail.endswith((".py", ".sh", ".bash")) or posixpath.basename(tail) in PYTHON_INTERPRETERS | SHELL_INTERPRETERS:
            return False
        if tail.startswith("target/") or "/target/" in tail:
            # cargo 编出来的二进制：不是源码（它的源码由 Rust 那一份闭包管）
            return True
        return tree_copy_file(self.index.root, tail) is None and not os.path.exists(os.path.join(self.index.root, tail))

    def outside_head(self, node, unit, depth=0):
        """命令位置上折不出的元素是不是根外的程序：shutil.which(…)、os.path.join(<折不出的目录>, <outside_tail 认的字面尾巴>)，
        或经本函数里的赋值（连同 a, b = f() 按位置拆包）、函数参数（每一处调用给的实参）、调仓内函数的返回值到这几种；有一种取法不是 → False。"""
        if depth > 10 or node is None or "python-unfolded-heads-allowed" in self.index.breaks:
            return False
        if isinstance(node, ast.Call):
            canonical = self.canonical(node.func, unit)
            if canonical == "shutil.which":
                return True
            if canonical in ("os.path.join", "posixpath.join") and node.args:
                tail = self.fold(node.args[-1], unit)
                return tail is not None and len(tail) == 1 and self.outside_tail(next(iter(tail)))
            if isinstance(node.func, ast.Name):
                return self.returns_outside(node.func.id, unit, None, depth)
            return False
        if isinstance(node, ast.BoolOp):
            for operand in node.values:
                folded = self.fold(operand, unit)
                if not (self.outside_head(operand, unit, depth + 1) or (folded is not None and all(self.outside_tail(value) for value in folded))):
                    return False
            return True
        if isinstance(node, ast.Constant) and isinstance(node.value, str):
            return self.outside_tail(node.value)
        if self.inline is not None and isinstance(node, ast.Subscript) and self.canonical(node.value, unit) == "sys.argv" and \
                isinstance(node.slice, ast.Constant) and type(node.slice.value) is int and 1 <= node.slice.value <= len(self.inline.argv) and \
                self.inline.argv[node.slice.value - 1] is POSITIONAL:
            # 内联脚本的参数取的是起它那份脚本的 "$1" 一类：程序是调用处给的那个词（同 shell 那边命令位置上的 "$1"）
            return True
        if isinstance(node, ast.Name) and (unit is None or not isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef))):
            # 模块级的名字（连同 a, b = x, y 拆包）：每一处赋值都得是根外的程序
            candidates = [candidate for candidate in self.by_name.get(node.id, ()) if candidate.kind == "assign" and isinstance(candidate.node, ast.Assign)]
            if not candidates:
                return False
            for candidate in candidates:
                statement = candidate.node
                value = statement.value if len(statement.targets) == 1 and isinstance(statement.targets[0], ast.Name) else \
                    dict(paired_elements(statement.targets[0], statement.value) or ()).get(node.id) if len(statement.targets) == 1 else None
                if value is None or not self.outside_head(value, candidate, depth + 1):
                    return False
            return True
        if not isinstance(node, ast.Name) or unit is None or not isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            return False
        sources = []
        for statement in walk_function_statements(unit.node):
            if isinstance(statement, ast.Assign):
                for target in statement.targets:
                    if isinstance(target, ast.Name) and target.id == node.id:
                        sources.append((statement.value, None))
                    elif isinstance(target, (ast.Tuple, ast.List)) and node.id in [item.id for item in target.elts if isinstance(item, ast.Name)]:
                        names = [item.id if isinstance(item, ast.Name) else None for item in target.elts]
                        sources.append((statement.value, names.index(node.id)))
            elif isinstance(statement, ast.AugAssign) and isinstance(statement.target, ast.Name) and statement.target.id == node.id:
                return False
            elif isinstance(statement, (ast.For, ast.AsyncFor, ast.With, ast.AsyncWith)) and node.id in {inner.id for inner in walk_targets(statement)}:
                return False
        if not sources:
            arguments = unit.node.args
            if node.id not in [argument.arg for argument in arguments.posonlyargs + arguments.args + arguments.kwonlyargs] or unit.container is not None:
                return False
            positional = [argument.arg for argument in arguments.posonlyargs + arguments.args]
            position = positional.index(node.id) if node.id in positional else None
            sites = self.call_sites().get(unit.node.name, [])
            if not sites:
                return False
            for call, caller in sites:
                argument = call.args[position] if position is not None and position < len(call.args) else None
                argument = next((keyword.value for keyword in call.keywords if keyword.arg == node.id), argument)
                if argument is None:
                    return False
                if "python-written-scripts-ignored" not in self.index.breaks and ast.dump(argument) in self.written_files(caller):
                    # 调用方自己 open(…, "w") 写出字面文本的那个程序：代码就是调用方里那段文本
                    continue
                if not self.outside_head(argument, caller, depth + 1):
                    return False
            return True
        for value, index in sources:
            if index is None:
                if not self.outside_head(value, unit, depth + 1):
                    return False
            elif not (isinstance(value, ast.Call) and isinstance(value.func, ast.Name) and self.returns_outside(value.func.id, unit, index, depth)):
                return False
        return True

    def returns_outside(self, name, unit, index, depth):
        """调仓内模块级函数 name：它每一处 return 的值（index 不是 None 时取元组的第 index 个）都是 outside_head 认的根外程序。"""
        binding = self.binding(name, unit)
        module = self
        if binding is not None:
            if binding[0] != "from":
                return False
            status, file = self.resolve_module(binding[1], binding[2])
            module = self.index.module(file) if status == "repo" else None
            name = binding[4]
            if module is None:
                return False
        candidates = [candidate for candidate in module.by_name.get(name, ()) if candidate.kind == "def" and candidate.container is None]
        if len(candidates) != 1:
            return False
        callee = candidates[0]
        returns = [statement.value for statement in walk_function_statements(callee.node) if isinstance(statement, ast.Return) and statement.value is not None]
        if not returns:
            return False
        for value in returns:
            if index is not None:
                if not isinstance(value, (ast.Tuple, ast.List)) or index >= len(value.elts):
                    return False
                value = value.elts[index]
            if not module.outside_head(value, callee, depth + 1):
                return False
        return True

    def path_values(self, node, unit, directory=False):
        """折得出的 → 那几个值；折不出、但「折不出的目录」后面接着一段在根里存在的字面尾巴（临时树、仓副本里的同一相对路径）→ {根/尾巴}；都不行 → None。"""
        values = self.fold(node, unit)
        if values is not None or "tree-copy-tails-ignored" in self.index.breaks:
            return values
        tail = self.literal_tail(node, unit)
        copied = tree_copy_file(self.index.root, tail, directory) if tail else None
        return {posixpath.join(self.index.root, copied)} if copied is not None else None

    def directory_wildcard(self, node, unit, suffix, depth=0):
        """os.path.join(<目录>, <折不出的文件名>)，目录折得出（或照临时树尾巴认得出）是根里的一个目录 → 那个目录里全部 <suffix> 文件（绝对路径）；
        起的是「那个目录里调用方点名的那一份」，哪一份都可能，就都连。名字是本函数里只赋过一次的局部变量的，照它的值认。"""
        if "python-directory-wildcards-ignored" in self.index.breaks or depth > 4:
            return None
        if isinstance(node, ast.Name) and unit is not None and isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            assigned = self.local_assignments(unit).get(node.id, [])
            return self.directory_wildcard(assigned[0], unit, suffix, depth + 1) if len(assigned) == 1 and assigned[0] is not None else None
        if not (isinstance(node, ast.Call) and not node.keywords and len(node.args) >= 2
                and self.canonical(node.func, unit) in ("os.path.join", "posixpath.join")):
            return None
        if self.fold(node.args[-1], unit) is not None:
            return None
        prefix = ast.Call(func=node.func, args=list(node.args[:-1]), keywords=[]) if len(node.args) > 2 else node.args[0]
        directories = self.path_values(prefix, unit, directory=True)
        if not directories:
            return None
        files = set()
        for directory in directories:
            if self.index.relative(directory) is None or not os.path.isdir(directory):
                return None
            files |= {posixpath.join(directory, name) for name in os.listdir(directory)
                      if name.endswith(suffix) and os.path.isfile(os.path.join(directory, name))}
        return files or None

    def file_for_value(self, value, relative_bases):
        """折出来的路径值 → 相对根的文件；relative_bases 是相对路径依次试的几处。"""
        if posixpath.isabs(value):
            candidates = [value]
        else:
            candidates = [posixpath.normpath(posixpath.join(base, value)) for base in relative_bases]
        for candidate in candidates:
            relative = self.index.relative(candidate)
            if relative is not None and os.path.isfile(candidate):
                return relative
        return None

    # 单元读到的东西

    def facts(self, unit):
        if unit.facts is not None:
            return unit.facts
        facts = PythonFacts()
        unit.facts = facts
        if unit.data or self.tree is None:
            return facts
        breaks = self.index.breaks
        if unit.kind == "import":
            for name, binding in self.local_import_bindings(unit).items():
                if name == unit.name or unit.name == "*":
                    facts.import_targets += self.binding_targets(binding, unit, facts)
        dropped = set()
        if "python-dispatch-tables-ignored" in breaks:
            # 弄坏开关：分派表（字典字面量）里当值写的函数名不连边
            dropped = {id(value) for node in walk_nodes(unit.nodes or []) if isinstance(node, ast.Dict)
                       for value in node.values if isinstance(value, ast.Name)}
        for node in walk_nodes(unit.nodes or []):
            if isinstance(node, ast.Name):
                if isinstance(node.ctx, (ast.Load, ast.Del)) and id(node) not in dropped:
                    facts.names.add(node.id)
                if node.id == "__doc__":
                    facts.doc_reads.append("<module>")
            elif isinstance(node, ast.Attribute):
                facts.attributes.append((dotted_name(node.value), node.attr))
                if node.attr == "__doc__":
                    base = dotted_name(node.value)
                    facts.doc_reads.append(base if base else "*")
            elif isinstance(node, (ast.Import, ast.ImportFrom)) and unit.kind != "import":
                table = {}
                self.bind(node, table)
                for binding in table.values():
                    facts.import_targets += self.binding_targets(binding, unit, facts)
            elif isinstance(node, ast.Call):
                self.call_facts(node, unit, facts, breaks)
            elif isinstance(node, ast.Constant) and isinstance(node.value, str):
                facts.strings.add(node.value)
            elif isinstance(node, ast.Assign) and any(isinstance(target, ast.Subscript) and self.canonical(target.value, unit) == "sys.path"
                                                      for target in node.targets):
                elements = list(node.value.elts) if isinstance(node.value, (ast.List, ast.Tuple)) else [None]
                if any(element is None or self.path_values(element, unit, directory=True) is None for element in elements):
                    self.refuse(facts, "sys-path-unfolded", node)
        return facts

    def outside_module(self, node, unit):
        """node 是 import 进来的标准库或外部模块（os、subprocess…）：getattr(它, <变量>) 取到的只会是那个模块里的东西，够不到仓里的代码。"""
        if "python-getattr-stdlib-refused" in self.index.breaks or not isinstance(node, ast.Name):
            return False
        binding = self.binding(node.id, unit)
        if binding is None or binding[0] != "import":
            return False
        return self.resolve_module(binding[1], 0)[0] in ("stdlib", "external")

    def local_import_bindings(self, unit):
        table = {}
        self.bind(unit.node, table)
        return table

    def refuse(self, facts, form, node):
        facts.refusals.append(self.refusal(form, node.lineno))

    def call_facts(self, node, unit, facts, breaks):
        function = node.func
        canonical = self.canonical(function, unit)
        if isinstance(function, ast.Name) and function.id == "getattr" and len(node.args) >= 2 and self.binding("getattr", unit) is None:
            attribute = node.args[1]
            if isinstance(attribute, ast.Constant) and isinstance(attribute.value, str):
                facts.attributes.append((dotted_name(node.args[0]), attribute.value))
            elif "python-getattr-allowed" not in breaks and not self.outside_module(node.args[0], unit):
                self.refuse(facts, "getattr-variable", node)
        if isinstance(function, ast.Name) and function.id == "globals" and self.binding("globals", unit) is None:
            self.refuse(facts, "globals-lookup", node)
        if canonical == "importlib.util.spec_from_file_location" and "python-spec-paths-ignored" not in breaks:
            location = node.args[1] if len(node.args) > 1 else next((keyword.value for keyword in node.keywords if keyword.arg == "location"), None)
            values = self.path_values(location, unit) if location is not None else None
            if values is None:
                self.refuse(facts, "spec-path-unfolded", node)
            else:
                for value in sorted(values):
                    file = self.file_for_value(value, [posixpath.dirname(self.absolute), self.index.root])
                    if file is None:
                        self.refuse(facts, "spec-path-missing", node)
                    else:
                        facts.loads.append(file)
        if canonical in ("importlib.import_module", "__import__") and node.args:
            argument = node.args[0]
            names = {argument.value} if isinstance(argument, ast.Constant) and isinstance(argument.value, str) else (
                self.fold(argument, unit) if "python-import-names-unfolded" not in breaks else None)
            if names is not None:
                for name in sorted(names):
                    status, file = self.resolve_module(name, 0)
                    if status == "repo":
                        facts.loads += self.all_resolutions(name, 0) or [file]
                    elif status == "unresolved":
                        self.refuse(facts, "import-unresolved", node)
            else:
                self.refuse(facts, "import-module-variable", node)
        if canonical == "runpy.run_path" and node.args:
            values = self.path_values(node.args[0], unit)
            if values is None:
                self.refuse(facts, "subprocess-path-unfolded", node)
            else:
                for value in sorted(values):
                    file = self.file_for_value(value, [self.index.root, posixpath.dirname(self.absolute)])
                    if file is None:
                        self.refuse(facts, "subprocess-path-missing", node)
                    else:
                        facts.program_links.append(file)
        if canonical in ("sys.path.insert", "sys.path.append"):
            argument = node.args[1] if canonical.endswith("insert") and len(node.args) >= 2 else (node.args[0] if node.args else None)
            if argument is not None and self.path_values(argument, unit, directory=True) is None:
                self.refuse(facts, "sys-path-unfolded", node)
        if canonical == "sys.path.extend" and node.args:
            elements = list(node.args[0].elts) if isinstance(node.args[0], (ast.List, ast.Tuple)) else [None]
            if any(element is None or self.path_values(element, unit, directory=True) is None for element in elements):
                self.refuse(facts, "sys-path-unfolded", node)
        if canonical in SUBPROCESS_CALLS and node.args:
            shell = any(keyword.arg == "shell" and isinstance(keyword.value, ast.Constant) and keyword.value.value for keyword in node.keywords)
            self.subprocess_facts(node, node.args[0], unit, facts, shell)
        if canonical in EXEC_CALLS and len(node.args) >= 2:
            self.subprocess_facts(node, node.args[1], unit, facts, False)
        if canonical in SHELL_STRING_CALLS and node.args:
            self.subprocess_facts(node, node.args[0], unit, facts, True)
        if canonical in SOURCE_READERS or (isinstance(function, ast.Attribute) and function.attr in ("read_text", "read_bytes")):
            if canonical in ("inspect.getsource", "inspect.getsourcelines") or any(
                    isinstance(inner, ast.Name) and inner.id == "__file__" for inner in ast.walk(node)):
                facts.source_read = True

    def command_elements(self, command, unit, shell):
        if isinstance(command, (ast.List, ast.Tuple)):
            return list(command.elts)
        if isinstance(command, ast.Name):
            candidates = []
            if isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                candidates = self.local_assignments(unit).get(command.id, [])
            if not candidates:
                candidates = [candidate.node.value for candidate in self.by_name.get(command.id, ())
                              if candidate.kind == "assign" and isinstance(candidate.node, ast.Assign)]
            if len(candidates) == 1 and isinstance(candidates[0], (ast.List, ast.Tuple)):
                return list(candidates[0].elts)
            return None
        if isinstance(command, ast.Constant) and isinstance(command.value, str) and shell:
            try:
                return [ast.Constant(value=word) for word in shlex.split(command.value)]
            except ValueError:
                return None
        return None

    def subprocess_facts(self, node, command, unit, facts, shell):
        """起子进程的一次调用：命令是交给 sh -c 的串（shell=True、os.system）的当内联 bash 认；是列表的折出它的几种取法（字面列表、
        拼起来的、按调用处或返回值折的），逐种按命令认；认不出 → 拒算 subprocess-command-unfolded。"""
        breaks = self.index.breaks
        keywords = {keyword.arg: keyword.value for keyword in node.keywords if keyword.arg}
        where = self.refusal("", node.lineno)
        if shell:
            if "python-commands-literal-only" in breaks:
                elements = self.command_elements(command, unit, True)
                if elements is not None:
                    self.command_links([(element, self, unit) for element in elements], unit, node, facts, keywords)
                return
            texts = self.fold(command, unit)
            if texts is None:
                if "inline-scripts-ignored" not in breaks:
                    self.refuse(facts, INLINE_FORM, node)
                facts.handled.append(("shell-string", node.lineno))
                return
            facts.inline_scripts.append(InlineScript("shell", tuple(sorted(texts)), (None,), self.working_directories(keywords, unit), where.file, where.line,
                                                     where.snippet))
            facts.handled.append(("shell-string", node.lineno))
            return
        if self.inline is not None and isinstance(command, ast.Subscript) and self.canonical(command.value, unit) == "sys.argv" and \
                isinstance(command.slice, ast.Slice) and "inline-argv-commands-ignored" not in breaks:
            # 内联脚本起 sys.argv[k:]：那条命令在起这个内联脚本的那一边照命令认过（ShellIndex.resolve_inline、command_links 的 inline）
            facts.handled.append(("argv-command", node.lineno))
            return
        if "python-commands-literal-only" in breaks:
            elements = self.command_elements(command, unit, False)
            alternatives = None if elements is None else [[(element, self, unit) for element in elements]]
            if alternatives is None:
                return
        else:
            alternatives = self.command_alternatives(command, unit)
        if alternatives is None:
            self.refuse(facts, COMMAND_FORM, node)
            facts.handled.append(("command-unfolded", node.lineno))
            return
        for elements in alternatives:
            self.command_links(elements, unit, node, facts, keywords)

    def working_directories(self, keywords, unit):
        """subprocess 的 cwd=：折得出就是它（几个都算），不然是根与这份文件所在的目录（相对路径依次试）。"""
        fallback = (self.index.root, posixpath.dirname(self.absolute)) if self.inline is None else tuple(self.inline.bases)
        directory = keywords.get("cwd")
        folded = self.path_values(directory, unit, directory=True) if directory is not None else None
        if not folded:
            return fallback
        return tuple(sorted(value if posixpath.isabs(value) else posixpath.normpath(posixpath.join(self.index.root, value)) for value in folded))

    def command_alternatives(self, node, unit, depth=0):
        """命令表达式 → 它的几种取法 [[(元素节点, 所在模块, 所在单元) 或 OPAQUE…]…]；认不出 → None。
        认：列表 / 元组字面量（*展开的认得出就摊开，认不出占一格 OPAQUE）、a + b、x if c else y、推导式（OPAQUE）、list(…)；
        名字：本函数里的赋值（连同 += / .append / .extend 往后加的）、函数参数（按本模块里每一处调用给的实参）、模块级赋值；
        调本模块或 from 仓内模块 import 进来的函数：它各处 return 的值。"""
        found = self.command_alternatives_once(node, unit, depth)
        return unique_alternatives(found) if found is not None else None

    def command_alternatives_once(self, node, unit, depth):
        if depth > 20 or node is None:
            return None
        if isinstance(node, (ast.List, ast.Tuple)):
            alternatives = [[]]
            for element in node.elts:
                if isinstance(element, ast.Starred):
                    parts = self.command_alternatives(element.value, unit, depth + 1) or [[OPAQUE]]
                else:
                    parts = [[(element, self, unit)]]
                alternatives = unique_alternatives([left + right for left in alternatives for right in parts])
                if len(alternatives) > FOLD_LIMIT:
                    return None
            return alternatives
        if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Add):
            left, right = self.command_alternatives(node.left, unit, depth + 1), self.command_alternatives(node.right, unit, depth + 1)
            if left is None or right is None or len(left) * len(right) > FOLD_LIMIT:
                return None
            return [first + second for first in left for second in right]
        if isinstance(node, ast.IfExp):
            body, other = self.command_alternatives(node.body, unit, depth + 1), self.command_alternatives(node.orelse, unit, depth + 1)
            return None if body is None or other is None else body + other
        if isinstance(node, (ast.ListComp, ast.GeneratorExp)):
            return [[OPAQUE]]
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id in ("list", "tuple") and len(node.args) == 1 and \
                self.binding(node.func.id, unit) is None:
            return self.command_alternatives(node.args[0], unit, depth + 1)
        if self.inline is not None and isinstance(node, ast.Subscript) and self.canonical(node.value, unit) == "sys.argv" and \
                isinstance(node.slice, ast.Slice) and node.slice.upper is None and node.slice.step is None and \
                isinstance(node.slice.lower, ast.Constant) and type(node.slice.lower.value) is int and node.slice.lower.value >= 1:
            # 内联脚本的 sys.argv[k:]：起它的那条命令里脚本之后从第 k 个起的那几个词
            return [[ValueElement(values) for values in self.inline.argv[node.slice.lower.value - 1:]]]
        if isinstance(node, ast.Constant) and node.value is None:
            # [*options["launcher"], …] 里取到 None 的那一种：那一支跑不到这里（*None 会抛），不添词
            return [[]]
        if isinstance(node, ast.Subscript) and isinstance(node.slice, ast.Constant) and isinstance(node.slice.value, str) and \
                "python-command-dictionaries-unfolded" not in self.index.breaks:
            # 字典[<字面键>]：字典折回它的字面量（经本函数的赋值、函数参数的每一处调用、dict(底, 键=…)、字典[键] = … 的写入），取那个键的值
            sources = self.dictionary_values(node.value, node.slice.value, unit, depth + 1)
            if sources is None:
                return None
            alternatives = []
            for value, module, owner in sources:
                found = module.command_alternatives(value, owner, depth + 1)
                if found is None:
                    return None
                alternatives += found
            return alternatives or None
        if isinstance(node, ast.Name):
            return self.name_command_alternatives(node.id, unit, depth)
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name):
            return self.call_command_alternatives(node.func.id, unit, depth)
        return None

    def dictionary_values(self, node, key, unit, depth):
        """node 折回字典字面量时，各字面量里 key 的值 [(值节点, 所在模块, 所在单元)…]（连同 node[key] = … 写进去的）；有一种来路认不出 → None。"""
        if depth > 30 or node is None:
            return None
        if isinstance(node, ast.Constant) and node.value is None:
            # 取到 None 的那一支取不了键（会抛），不添取值
            return []
        if isinstance(node, ast.IfExp):
            body, other = self.dictionary_values(node.body, key, unit, depth + 1), self.dictionary_values(node.orelse, key, unit, depth + 1)
            return None if body is None or other is None else body + other
        if isinstance(node, ast.Dict):
            return [(value, self, unit) for name, value in zip(node.keys, node.values) if isinstance(name, ast.Constant) and name.value == key]
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id == "dict" and self.binding("dict", unit) is None:
            given = [(keyword.value, self, unit) for keyword in node.keywords if keyword.arg == key]
            if given:
                return given
            for keyword in node.keywords:
                if keyword.arg is None:
                    # dict(底, **改动)：改动那一份也折回字面量，给了这个键就是它
                    spread = self.dictionary_values(keyword.value, key, unit, depth + 1)
                    if spread is None:
                        return None
                    given += spread
            base = self.dictionary_values(node.args[0], key, unit, depth + 1) if node.args else []
            return None if base is None else given + base
        if not isinstance(node, ast.Name) or unit is None:
            return None
        overrides = self.__dict__.setdefault("dictionary_overrides", {})
        if (id(unit), node.id) in overrides:
            return overrides[(id(unit), node.id)]
        progress = self.index.__dict__.setdefault("dictionary_progress", set())
        marker = (self.path, id(unit), node.id, key)
        if marker in progress:
            # 绕回来了（saved = options["launcher"] … options["launcher"] = saved 一类存了再放回去）：这一圈不添新的取值
            return []
        progress.add(marker)
        try:
            found = self.dictionary_name_values(node, key, unit, depth)
        finally:
            progress.discard(marker)
        if found is None:
            return None
        unique, seen = [], set()
        for item in found:
            if id(item[0]) not in seen:
                seen.add(id(item[0]))
                unique.append(item)
        return unique

    def dictionary_name_values(self, node, key, unit, depth):
        overrides = self.__dict__.setdefault("dictionary_overrides", {})
        function = unit.node if isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)) else None
        found, written, unpacked = [], [], False
        statements = walk_function_statements(function) if function is not None else walk_nodes(self.module_body.nodes or [])
        for statement in statements:
            if isinstance(statement, ast.Assign):
                for target in statement.targets:
                    if isinstance(target, ast.Name) and target.id == node.id:
                        found.append(statement.value)
                    elif isinstance(target, ast.Subscript) and isinstance(target.value, ast.Name) and target.value.id == node.id and \
                            isinstance(target.slice, ast.Constant) and target.slice.value == key:
                        written.append((statement.value, self, unit))
                    elif isinstance(target, ast.Subscript) and isinstance(target.value, ast.Name) and target.value.id == node.id:
                        # 字典[<变量键>] = …：键折得出、又没有这个键的，与这个键无关；折不出就认不出写进去的是哪一项
                        keys = self.fold(target.slice, unit)
                        table = target.slice.value if isinstance(target.slice, ast.Subscript) and isinstance(target.slice.value, ast.Name) else None
                        if keys is None and table is not None and function is not None:
                            # 键取自本函数里只赋过一次的字典字面量（flags[argument] 一类）：键是那张表的各个值
                            assigned = self.local_assignments(unit).get(table.id, [])
                            if len(assigned) == 1 and isinstance(assigned[0], ast.Dict):
                                keys = {value.value for value in assigned[0].values if isinstance(value, ast.Constant)} \
                                    if all(isinstance(value, ast.Constant) for value in assigned[0].values) else None
                        if keys is None or key in keys:
                            return None
                    elif isinstance(target, (ast.Tuple, ast.List)):
                        for position, item in enumerate(target.elts):
                            if isinstance(item, ast.Name) and item.id == node.id:
                                pairs = paired_elements(target, statement.value)
                                if pairs is not None:
                                    # a, options = x, {…}：对上的那一项
                                    found.append(dict(pairs)[node.id])
                                    continue
                                # a, options = f(…)：取 f 各处 return 的元组里这个位置的那一项
                                if not (isinstance(statement.value, ast.Call) and isinstance(statement.value.func, ast.Name)):
                                    return None
                                returned = self.returned_elements(statement.value.func.id, unit, position)
                                if returned is None:
                                    return None
                                for value, module, owner in returned:
                                    inner = module.dictionary_values(value, key, owner, depth + 1)
                                    if inner is None:
                                        return None
                                    written += inner
                                unpacked = True
                            if isinstance(item, ast.Subscript) and isinstance(item.value, ast.Name) and item.value.id == node.id and \
                                    isinstance(item.slice, ast.Constant) and item.slice.value == key:
                                pairs = statement.value.elts if isinstance(statement.value, (ast.Tuple, ast.List)) else None
                                if pairs is None or position >= len(pairs):
                                    return None
                                written.append((pairs[position], self, unit))
        values = list(written)
        # options = dict(options, …) 一类引用自己的赋值：里面的 options 是之前的值（别的赋值，没有就是参数 / 模块级那一份）
        own = [value for value in found if any(isinstance(inner, ast.Name) and inner.id == node.id for inner in ast.walk(value))]
        plain = [value for value in found if not any(value is item for item in own)]
        if plain:
            base = []
            for value in plain:
                inner = self.dictionary_values(value, key, unit, depth + 1)
                if inner is None:
                    return None
                base += inner
        elif unpacked:
            base = []
        else:
            base = self.dictionary_parameter_values(node, key, unit, function, depth)
            if base is None:
                return None
        if not own:
            return values + base
        overrides[(id(unit), node.id)] = base
        try:
            for value in own:
                inner = self.dictionary_values(value, key, unit, depth + 1)
                if inner is None:
                    return None
                values += inner
        finally:
            overrides.pop((id(unit), node.id), None)
        return values + (base if plain else [])

    def returned_elements(self, name, unit, position):
        """调仓内模块级函数 name：它各处 return 的元组里第 position 项 [(节点, 模块, 单元)…]；有一处不是元组字面量 → None。"""
        binding = self.binding(name, unit)
        module = self
        if binding is not None:
            if binding[0] != "from":
                return None
            status, file = self.resolve_module(binding[1], binding[2])
            module = self.index.module(file) if status == "repo" else None
            name = binding[4]
            if module is None:
                return None
        candidates = [candidate for candidate in module.by_name.get(name, ()) if candidate.kind == "def" and candidate.container is None]
        if len(candidates) != 1:
            return None
        out = []
        for statement in walk_function_statements(candidates[0].node):
            if isinstance(statement, ast.Return) and statement.value is not None:
                if not isinstance(statement.value, (ast.Tuple, ast.List)) or position >= len(statement.value.elts):
                    return None
                out.append((statement.value.elts[position], module, candidates[0]))
        return out or None

    def dictionary_parameter_values(self, node, key, unit, function, depth):
        """dictionary_values 里名字不在本函数里赋值时：函数参数按每一处调用给的实参折，模块级的按它那一处赋值折。"""
        values = []
        if function is not None:
            arguments = function.args
            positional = [argument.arg for argument in arguments.posonlyargs + arguments.args]
            if node.id in positional and unit.container is None:
                position = positional.index(node.id)
                defaults = dict(zip(positional[len(positional) - len(arguments.defaults):], arguments.defaults))
                sites = self.call_sites().get(function.name, [])
                if not sites:
                    return None
                for call, caller in sites:
                    argument = call.args[position] if position < len(call.args) else next(
                        (keyword.value for keyword in call.keywords if keyword.arg == node.id), None)
                    if argument is None and node.id in defaults:
                        # 这一处调用没给：取缺省值（在被调函数那边认）
                        inner = self.dictionary_values(defaults[node.id], key, unit, depth + 1)
                        if inner is None:
                            return None
                        values += inner
                        continue
                    if argument is None:
                        return None
                    inner = self.dictionary_values(argument, key, caller, depth + 1)
                    if inner is None:
                        return None
                    values += inner
                return values
            # **名字：本函数或套在里面的函数收的关键字参数，这个键的值是各处调用给的同名关键字参数
            nested = [function] + [inner for inner in ast.walk(function) if isinstance(inner, (ast.FunctionDef, ast.AsyncFunctionDef)) and inner is not function]
            owner = next((definition for definition in nested if definition.args.kwarg is not None and definition.args.kwarg.arg == node.id), None)
            if owner is None:
                return None
            if owner is function:
                sites = self.call_sites().get(function.name, []) if unit.container is None else []
            else:
                sites = [(call, unit) for call in ast.walk(function) if isinstance(call, ast.Call) and isinstance(call.func, ast.Name) and call.func.id == owner.name]
            if not sites:
                return None
            for call, caller in sites:
                for keyword in call.keywords:
                    if keyword.arg == key:
                        values.append((keyword.value, self, caller))
                    elif keyword.arg is None:
                        inner = self.dictionary_values(keyword.value, key, caller, depth + 1)
                        if inner is None:
                            return None
                        values += inner
            return values
        candidates = [candidate for candidate in self.by_name.get(node.id, ()) if candidate.kind == "assign" and isinstance(candidate.node, ast.Assign)]
        if len(candidates) != 1:
            return None
        inner = self.dictionary_values(candidates[0].node.value, key, candidates[0], depth + 1)
        return None if inner is None else values + inner

    def name_command_alternatives(self, name, unit, depth):
        overridden = getattr(self, "command_overrides", {}).get((id(unit), name))
        if overridden is not None:
            return overridden
        progress = self.index.__dict__.setdefault("command_progress", set())
        marker = (self.path, id(unit), name)
        if marker in progress:
            # 绕回来了（存了再放回去的那一类）：这一圈不添新的取法
            return [[]]
        progress.add(marker)
        try:
            return self.name_command_alternatives_once(name, unit, depth)
        finally:
            progress.discard(marker)

    def name_command_alternatives_once(self, name, unit, depth):
        function = unit.node if unit is not None and isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)) else None
        if function is not None:
            assigned, appended = [], []
            for statement in walk_function_statements(function):
                if isinstance(statement, ast.Assign):
                    for target in statement.targets:
                        if isinstance(target, ast.Name) and target.id == name:
                            assigned.append(statement.value)
                        elif name in dict(paired_elements(target, statement.value) or ()):
                            assigned.append(dict(paired_elements(target, statement.value))[name])
                        elif name in bound_names(target):
                            return None
                elif isinstance(statement, (ast.AnnAssign, ast.NamedExpr)) and isinstance(statement.target, ast.Name) and statement.target.id == name:
                    assigned.append(statement.value)
                elif isinstance(statement, ast.AugAssign) and isinstance(statement.target, ast.Name) and statement.target.id == name:
                    if not isinstance(statement.op, ast.Add):
                        return None
                    appended.append(("extend", statement.value))
                elif isinstance(statement, (ast.For, ast.AsyncFor)) and isinstance(statement.target, ast.Name) and statement.target.id == name \
                        and isinstance(statement.iter, (ast.Tuple, ast.List)):
                    # for 命令 in (<列表>, …)：每个元素是一种取法
                    assigned += list(statement.iter.elts)
                elif isinstance(statement, (ast.For, ast.AsyncFor, ast.With, ast.AsyncWith)) and name in {inner.id for inner in walk_targets(statement)}:
                    return None
                elif isinstance(statement, ast.Call) and isinstance(statement.func, ast.Attribute) and isinstance(statement.func.value, ast.Name) \
                        and statement.func.value.id == name and statement.func.attr in ("append", "extend", "insert"):
                    if statement.func.attr == "insert" or len(statement.args) != 1:
                        return None
                    appended.append((statement.func.attr, statement.args[0]))
            arguments = function.args
            parameters = [argument.arg for argument in arguments.posonlyargs + arguments.args + arguments.kwonlyargs]
            if assigned:
                # command = [..., *command] 一类引用自己的赋值：里面的 command 是别的那几处赋的值
                own = [value for value in assigned if any(isinstance(inner, ast.Name) and inner.id == name for inner in ast.walk(value))]
                bases = []
                for value in assigned:
                    if any(value is item for item in own):
                        continue
                    found = self.command_alternatives(value, unit, depth + 1)
                    if found is None:
                        return None
                    bases += found
                if own and bases:
                    if not hasattr(self, "command_overrides"):
                        self.command_overrides = {}
                    self.command_overrides[(id(unit), name)] = list(bases)
                    try:
                        for value in own:
                            found = self.command_alternatives(value, unit, depth + 1)
                            if found is None:
                                return None
                            bases += found
                    finally:
                        self.command_overrides.pop((id(unit), name), None)
                elif own:
                    return None
            elif name in parameters:
                bases = self.parameter_command_alternatives(unit, name, depth)
                if bases is None:
                    return None
            elif appended:
                bases = [[]]
            else:
                bases = None
            if bases is not None:
                for method, value in appended:
                    parts = [[(value, self, unit)]] if method == "append" else (self.command_alternatives(value, unit, depth + 1) or [[OPAQUE]])
                    bases = [left + right for left in bases for right in parts]
                    if len(bases) > FOLD_LIMIT:
                        return None
                return bases
        candidates = [candidate for candidate in self.by_name.get(name, ()) if candidate.kind == "assign" and isinstance(candidate.node, ast.Assign)]
        if len(candidates) != 1 or name in self.body_defines:
            return None
        return self.command_alternatives(candidates[0].node.value, candidates[0], depth + 1)

    def parameter_command_alternatives(self, unit, name, depth):
        """命令是函数参数：本模块里调这个函数的每一处给的那个实参（在调用方那边认），合起来；有一处认不出、没有调用处 → None。
        类里的方法按同一个类里 self.<方法>(…) 那几处调用认（实参位置去掉 self）。"""
        function = unit.node
        arguments = function.args
        positional = [argument.arg for argument in arguments.posonlyargs + arguments.args]
        defaults = dict(zip(positional[len(positional) - len(arguments.defaults):], arguments.defaults))
        defaults.update({argument.arg: default for argument, default in zip(arguments.kwonlyargs, arguments.kw_defaults) if default is not None})
        position = positional.index(name) if name in positional else None
        if unit.container is not None:
            if not positional or position == 0:
                return None
            position = position - 1 if position is not None else None
            sites = [(node, method) for method in self.methods if method.container is unit.container for node in walk_nodes(method.nodes or [])
                     if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.func.attr == function.name
                     and isinstance(node.func.value, ast.Name) and node.func.value.id == "self"]
        else:
            sites = self.call_sites().get(function.name, [])
        if not sites:
            return None
        alternatives = []
        for call, caller in sites:
            argument = None
            if position is not None and position < len(call.args):
                if any(isinstance(item, ast.Starred) for item in call.args[:position + 1]):
                    return None
                argument = call.args[position]
            for keyword in call.keywords:
                if keyword.arg is None:
                    return None
                if keyword.arg == name:
                    argument = keyword.value
            if argument is None:
                argument = defaults.get(name)
            if argument is None:
                return None
            found = self.command_alternatives(argument, caller, depth + 1)
            if found is None:
                return None
            alternatives += found
            if len(alternatives) > FOLD_LIMIT:
                return None
        return alternatives

    def call_command_alternatives(self, name, unit, depth):
        """命令是调一个仓内模块级函数的返回值（本模块的，或 from 仓内模块 import 进来的）：它各处 return 的值（在它那边认）合起来。"""
        binding = self.binding(name, unit)
        module = self
        if binding is not None:
            if binding[0] != "from":
                return None
            status, file = self.resolve_module(binding[1], binding[2])
            module = self.index.module(file) if status == "repo" else None
            name = binding[4]
            if module is None:
                return None
        candidates = [candidate for candidate in module.by_name.get(name, ()) if candidate.kind == "def" and candidate.container is None]
        if len(candidates) != 1:
            return None
        callee = candidates[0]
        key = (module.path, id(callee), "<command>")
        if key in self.index.folding:
            return None
        self.index.folding.add(key)
        try:
            alternatives = []
            for statement in walk_function_statements(callee.node):
                if isinstance(statement, ast.Return):
                    if statement.value is None:
                        continue
                    found = module.command_alternatives(statement.value, callee, depth + 1)
                    if found is None:
                        return None
                    alternatives += found
            return alternatives or None
        finally:
            self.index.folding.discard(key)

    def command_links(self, elements, unit, node, facts, keywords=None):
        """一种取法的命令（[(元素节点, 所在模块, 所在单元) 或 OPAQUE…]）→ 连边、内联脚本与拒算。命令位置（经 env / nice / timeout… 之后）上的词
        折不出 → 拒算 subprocess-command-unfolded；解释器后面照它的写法认（脚本、-c 的串、-m 的模块、input= 喂的标准输入），脚本之后接着往后认。"""
        keywords = keywords or {}
        breaks = self.index.breaks
        values = []
        for item in elements:
            if item is OPAQUE:
                values.append(None)
                continue
            if isinstance(item, ValueElement):
                values.append(set(item.values) if item.values is not None and item.values is not POSITIONAL else None)
                continue
            element, module, owner = item
            if isinstance(element, ast.Starred):
                values.append(None)
            elif module.canonical(element, owner) == "sys.executable":
                values.append({"python3"})
            else:
                values.append(module.fold(element, owner))

        def single(position):
            value = values[position] if position < len(values) else None
            return next(iter(value)) if value is not None and len(value) == 1 else None

        def element_call(position, method, *arguments):
            item = elements[position]
            if item is OPAQUE or isinstance(item, ValueElement) or isinstance(item[0], ast.Starred):
                return None
            element, module, owner = item
            calls = {"path_values": module.path_values, "literal_tail": module.literal_tail, "directory_wildcard": module.directory_wildcard}
            return calls[method](element, owner, *arguments)

        def path_program(position):
            """命令位置上的词是 shutil.which(…) 找到的 PATH 上的程序（或本函数里只赋过一次这个的名字）。"""
            item = elements[position]
            if item is OPAQUE or isinstance(item, ValueElement):
                return False
            element, module, owner = item
            if isinstance(element, ast.Name) and owner is not None and isinstance(owner.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                assigned = module.local_assignments(owner).get(element.id, [])
                element = assigned[0] if len(assigned) == 1 else element
            return isinstance(element, ast.Call) and module.canonical(element.func, owner) == "shutil.which"

        bases = list(self.working_directories(keywords, unit))
        literal = [single(position) for position in range(len(values))]
        positions = command_positions(literal) if "python-unfolded-heads-allowed" not in breaks else [0]
        head = positions[-1] if positions else 0
        if head >= len(values):
            facts.handled.append(("empty", node.lineno))
            return
        if values[head] is None and "python-unfolded-heads-allowed" not in breaks:
            if isinstance(elements[head], ValueElement) and elements[head].values is POSITIONAL:
                # 内联脚本 exec 它收到的 "$@"：命令就是起它的那条 shell 命令后面那几个词（调用处照命令认过）
                facts.handled.append(("positional", node.lineno))
                return
            if "python-path-programs-refused" not in breaks and (path_program(head) or (
                    elements[head] is not OPAQUE and not isinstance(elements[head], ValueElement)
                    and elements[head][1].outside_head(elements[head][0], elements[head][2]))):
                # shutil.which(<名字>) 找到的、os.path.join(<折不出的目录>, "verus") 一类根外的程序：不是仓里的代码
                facts.handled.append(("path-program", node.lineno))
                return
            copied = element_call(head, "path_values")
            if not copied:
                tail = element_call(head, "literal_tail") if "python-path-programs-refused" not in breaks else None
                if tail and tail.strip("/") and not tail.endswith((".py", ".sh")) and tree_copy_file(self.index.root, tail.lstrip("/")) is None \
                        and not os.path.exists(os.path.join(self.index.root, tail.lstrip("/"))):
                    # os.path.join(<折不出的目录>, "bin/llvm-cov") 一类：根外的程序（不带脚本后缀、根里也没有这一截）
                    facts.handled.append(("outside-program", node.lineno))
                    return
                self.refuse(facts, COMMAND_FORM, node)
                facts.handled.append(("head-unfolded", node.lineno))
                return
            values[head] = copied
            literal[head] = single(head)
        if values[head] is not None and len(values[head]) > 1:
            # 命令位置上的词折出几个：都不是解释器的，是脚本的连上、别的是外部程序；有解释器的认不出后面怎么接，拒算
            if any(posixpath.basename(value) in PYTHON_INTERPRETERS | SHELL_INTERPRETERS for value in values[head]):
                self.refuse(facts, COMMAND_FORM, node)
                facts.handled.append(("head-several", node.lineno))
                return
            for value in sorted(values[head]):
                file = self.file_for_value(value, bases) if value.endswith((".py", ".sh")) or "/" in value else None
                if file is not None:
                    facts.program_links.append(file)
            facts.handled.append(("head-several", node.lineno))
            return
        position, interpreters = head, 0
        while position < len(values):
            word = literal[position]
            base = posixpath.basename(word) if word else None
            if base not in PYTHON_INTERPRETERS and base not in SHELL_INTERPRETERS:
                if position > head and values[position] is None and "python-command-arguments-loaded" in breaks and elements[position] is not OPAQUE:
                    # 弄坏开关：脚本后面传给它的路径参数也当装代码（那是给被起的程序的数据）
                    tail = element_call(position, "literal_tail")
                    if tail and tail.endswith((".py", ".sh")) and element_call(position, "path_values") is None:
                        self.refuse(facts, "subprocess-path-unfolded", node)
                position += 1
                continue
            interpreters += 1
            python = base in PYTHON_INTERPRETERS
            mode, argument = interpreter_mode(literal, position, python)
            facts.handled.append((f"{'python' if python else 'shell'}-{mode}", node.lineno))
            if mode == "script":
                if descriptor_of(literal[argument]) is not None:
                    self.refuse(facts, INLINE_FORM, node)
                    return
                if values[argument] is None:
                    values[argument] = element_call(argument, "path_values")
                if values[argument] is None:
                    values[argument] = element_call(argument, "directory_wildcard", ".py" if python else ".sh")
                if values[argument] is None and "python-written-scripts-ignored" not in breaks and elements[argument] is not OPAQUE \
                        and not isinstance(elements[argument], ValueElement) and elements[argument][2] is not None \
                        and ast.dump(elements[argument][0]) in elements[argument][1].written_files(elements[argument][2]):
                    position = argument + 1
                    continue
                if values[argument] is None:
                    self.refuse(facts, "subprocess-path-unfolded", node)
                else:
                    for value in sorted(values[argument]):
                        file = self.file_for_value(value, bases)
                        if file is None:
                            self.refuse(facts, "subprocess-path-missing", node)
                        else:
                            facts.program_links.append(file)
                position = argument + 1
                continue
            if mode == "inline":
                if "inline-scripts-ignored" in breaks:
                    return
                texts = values[argument + 1] if argument + 1 < len(values) else None
                if texts is None and not python and argument + 1 < len(values) and self.declared_check_runner(elements[argument + 1]):
                    # 上游 preflight 跑脚本文件头登记的 check 判据：判据正文在各份声明它的脚本里，从那些脚本连（ScriptClosure.declared_checks）
                    facts.handled.append(("declared-checks", node.lineno))
                    return
                after = [values[position] if values[position] is not None else
                         (element_call(position, "path_values") or element_call(position, "path_values", True))
                         for position in range(argument + 2, len(values))]
                self.inline_from_command(facts, node, "python" if python else "shell", texts, after, bases)
                if python and texts is not None and "inline-argv-commands-ignored" not in breaks:
                    # 正文把 sys.argv[k:] 当命令起：sys.argv[1] 是串之后那一个元素，从第 k 个起是一条命令
                    for offset in sorted(set().union(*(argv_command_offsets(text) for text in texts))):
                        if argument + 1 + offset < len(elements):
                            self.command_links(elements[argument + 1 + offset:], unit, node, facts, keywords)
                return
            if mode == "module":
                self.module_from_command(facts, node, values, argument, bases)
                position = argument + 2
                continue
            if mode == "stdin" or (mode == "bare" and position in positions):
                if "inline-scripts-ignored" in breaks:
                    return
                feed = keywords.get("input")
                texts = self.fold(feed, unit) if feed is not None else None
                self.inline_from_command(facts, node, "python" if python else "shell", texts, values[argument + 1:], bases, stdin=True)
                return
            return
        if interpreters == 0:
            word = literal[head]
            if word and (word.endswith((".py", ".sh")) or ("/" in word and not word.startswith(("/usr/", "/bin/")))):
                file = self.file_for_value(word, bases)
                if file is not None:
                    facts.program_links.append(file)
            facts.handled.append(("program", node.lineno))

    def declared_check_runner(self, item):
        """上游 preflight.py 里跑 check 判据的那一处：bash -c declaration.command（判据是被判脚本文件头里登记的那几行）。"""
        if item is OPAQUE or isinstance(item, ValueElement) or "declared-checks-ignored" in self.index.breaks:
            return False
        element = item[0]
        return self.path.endswith(PREFLIGHT_MODULE) and isinstance(element, ast.Attribute) and element.attr == "command" and \
            isinstance(element.value, ast.Name) and element.value.id == "declaration"

    def inline_from_command(self, facts, node, language, texts, after, bases, stdin=False):
        """命令里 -c 的串、input= 喂的标准输入：正文折得出记成一份 InlineScript，折不出拒算 inline-script-not-literal。
        after 是串之后（标准输入时是解释器选项之后）那几个元素的取值：python 的 sys.argv[1:]；bash 的 -c 是 $0 $1 …，标准输入是 $1 …。"""
        if texts is None:
            self.refuse(facts, INLINE_FORM, node)
            return
        argv = tuple(frozenset(value) if value is not None else None for value in after)
        if language == "shell" and stdin:
            argv = (None,) + argv
        where = self.refusal("", node.lineno)
        facts.inline_scripts.append(InlineScript(language, tuple(sorted(texts)), argv, tuple(bases), where.file, where.line, where.snippet))

    def module_from_command(self, facts, node, values, argument, bases):
        """[python3, -m, <模块>, …]：同 shell 那边的 resolve_module_run。"""
        if "python-modules-ignored" in self.index.breaks:
            return
        names = values[argument + 1] if argument + 1 < len(values) else None
        if names is None:
            self.refuse(facts, "import-module-variable", node)
            return
        for name in sorted(names):
            if name not in TEST_RUNNER_MODULES:
                status, file = repository_module(self.index.root, bases, name, main=True)
                if status == "repo":
                    facts.program_links.append(file)
                elif status == "unresolved":
                    self.refuse(facts, "import-unresolved", node)
                continue
            targets, position = [], argument + 2
            while position < len(values):
                value = values[position]
                one = next(iter(value)) if value is not None and len(value) == 1 else None
                if one is not None and one.startswith("-"):
                    position += 2 if one in ("-k", "-p", "-s", "-t", "-m", "--pattern", "--start-directory", "--top-level-directory") else 1
                    continue
                targets.append(value)
                position += 1
            if not targets or targets[0] == {"discover"}:
                self.refuse(facts, "import-unresolved", node)
                continue
            for target in targets:
                if target is None:
                    self.refuse(facts, "import-module-variable", node)
                    continue
                for value in sorted(target):
                    status, file = repository_module(self.index.root, bases, value)
                    if status == "repo":
                        facts.whole_modules.append(file)
                    elif status == "unresolved":
                        self.refuse(facts, "import-unresolved", node)

    def docstring_units(self, owner):
        owners = sorted(self.docstrings) if owner == "*" else [owner]
        out = []
        for name in owners:
            if name not in self.docstrings:
                continue
            if name not in self._data:
                text, first, last = self.docstrings[name]
                self._data[name] = Unit("python", "docstring", f"{name}.__doc__", self.path, first, last, sha256_text(text), data=True)
            out.append(self._data[name])
        return out

    def source_unit(self):
        if "<source-text>" not in self._data:
            self._data["<source-text>"] = Unit("python", "source-text", "<source-text>", self.path, 1, len(self.rows), sha256_bytes(self.raw), data=True)
        return self._data["<source-text>"]


def python_units(root, under=None):
    index = PythonIndex(root)
    modules = [index.module(path) for path in files_under(root, (".py",), under)]
    return index, [module for module in modules if module is not None]


# ── shell ───────────────────────────────────────────────────────────────────

SHELL_WRAPPER = "__singlefs_function_units_body"
SHELL_NORMALIZE_SCRIPT = r'''
for source_path in "$@"; do
  if bash -n "$source_path" 2>"$source_path.check" && [[ ! -s "$source_path.check" ]]; then
    source_text="$(<"$source_path")"
    if eval "__singlefs_function_units_body() {
$source_text
}" 2>/dev/null; then
      declare -f __singlefs_function_units_body >"$source_path.normal"
    fi
    unset -f __singlefs_function_units_body
  fi
done
'''
KEPT_SHELL_COMMENT = re.compile(r"(?:^|[\s;&|()])(#\s*(?:[A-Za-z][A-Za-z0-9_-]*:|shellcheck\b).*)$")
FUNCTION_HEADER = re.compile(r"^( *)function (\S+) \(\) $")
HEREDOC_START = re.compile(r"(?<!<)<<(?!<)(-?)\s*(['\"]?)([A-Za-z_][A-Za-z0-9_]*)\2")
TOP_ASSIGNMENT = re.compile(r"^    (?:(?:export|readonly|declare|typeset)(?: -[A-Za-z]+)* )?([A-Za-z_][A-Za-z0-9_]*)(?:\[[^\]]*\])?\+?=")
ASSIGNMENT_WORD = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)(?:\[(?:[^\[\]]|\[[^\]]*\])*\])?\+?=(.*)$", re.S)
SHELL_KEYWORDS = frozenset(("if", "then", "else", "elif", "fi", "do", "done", "while", "until", "!", "{", "}", "time", "esac", "coproc"))
SHELL_DECLARATIONS = frozenset(("local", "declare", "export", "readonly", "typeset"))
SHELL_WORDS = re.compile(r"[A-Za-z_][A-Za-z0-9_.:-]*")
VARIABLE_REFERENCE = re.compile(r"\$\{?([A-Za-z_][A-Za-z0-9_]*)")


def kept_shell_comments(text):
    rows = text.split("\n")
    kept = [rows[0]] if rows and rows[0].startswith("#!") else []
    for row in rows:
        found = KEPT_SHELL_COMMENT.search(row)
        if found:
            kept.append(found.group(1))
    return kept


def shell_normal_forms(contents):
    """每份内容 → declare -f 打出来的那一份（str）；bash -n 不过、有告警、包不进函数的 → None。bash 起不来抛 FunctionUnitsError。"""
    with tempfile.TemporaryDirectory(prefix="function-units-shell-") as directory:
        paths = []
        for position, content in enumerate(contents):
            path = os.path.join(directory, f"{position}.sh")
            with open(path, "wb") as handle:
                handle.write(content)
            paths.append(path)
        environment = {"PATH": os.environ.get("PATH", "/usr/bin:/bin"), "LC_ALL": "C", "HOME": directory}
        try:
            completed = subprocess.run(["bash", "--norc", "--noprofile", "-c", SHELL_NORMALIZE_SCRIPT, "function-units", *paths],
                                       cwd=directory, env=environment, capture_output=True, stdin=subprocess.DEVNULL, timeout=SHELL_TIMEOUT_SECONDS)
        except (OSError, subprocess.TimeoutExpired) as error:
            raise FunctionUnitsError(f"bash 解析 .sh 没跑完：{error}") from error
        if completed.returncode != 0:
            raise FunctionUnitsError(f"bash 解析 .sh 退 {completed.returncode}：{completed.stderr.decode('utf-8', 'replace').strip()[:300]}")
        forms = []
        for path in paths:
            if os.path.isfile(path + ".normal"):
                with open(path + ".normal", "rb") as handle:
                    forms.append(handle.read().decode("utf-8", "surrogateescape"))
            else:
                forms.append(None)
        return forms


def heredoc_mask(rows):
    """规范文本里 here-doc 正文那几行的下标（起止按 <<词 与单独一行的词认；后面找不到收尾那一行的不算）。"""
    mask, pending = set(), []
    for position, row in enumerate(rows):
        if pending:
            mask.add(position)
            delimiter, strip_tabs = pending[0]
            if (row.lstrip("\t") if strip_tabs else row) == delimiter:
                pending.pop(0)
            continue
        for found in HEREDOC_START.finditer(row):
            delimiter, strip_tabs = found.group(3), found.group(1) == "-"
            if any((later.lstrip("\t") if strip_tabs else later) == delimiter for later in rows[position + 1:]):
                pending.append((delimiter, strip_tabs))
    return mask


def heredoc_bodies(rows):
    """规范文本里每个 here-doc → {(<<词 所在行, 所在列): (词带不带引号, 正文)}；起止的认法与 heredoc_mask 同一份（找不到收尾那一行的不算）。"""
    bodies, pending = {}, []
    for position, row in enumerate(rows):
        if pending:
            place, delimiter, strip_tabs, quoted, lines = pending[0]
            if (row.lstrip("\t") if strip_tabs else row) == delimiter:
                bodies[place] = (quoted, "\n".join(lines))
                pending.pop(0)
            else:
                lines.append(row)
            continue
        for found in HEREDOC_START.finditer(row):
            delimiter, strip_tabs = found.group(3), found.group(1) == "-"
            if any((later.lstrip("\t") if strip_tabs else later) == delimiter for later in rows[position + 1:]):
                pending.append(((position, found.start()), delimiter, strip_tabs, bool(found.group(2)), []))
    return bodies


def command_inputs(details, base, visible, bodies):
    """split_commands 记的重定向细目 → {命令下标: [(文件描述符号, 写法, 内容)…]}，只收读进来的那几种：
    file（< 词：内容是目标词）、herestring（<<< 词）、heredoc（<<词：内容是 (带不带引号, 正文)，正文对不上是 None）。
    base 是这一段在 visible（整个单元的规范文本，here-doc 正文已抹空）里的起点，用来找 <<词 落在哪一行哪一列。"""
    out = collections.defaultdict(list)
    for index, operator, target, offset, descriptor in details:
        number = 0 if descriptor is None else descriptor
        if operator in ("<<", "<<-"):
            absolute = base + offset
            row = visible.count("\n", 0, absolute)
            column = absolute - (visible.rfind("\n", 0, absolute) + 1)
            out[index].append((number, "heredoc", bodies.get((row, column))))
        elif operator == "<<<":
            out[index].append((number, "herestring", target))
        elif operator == "<":
            out[index].append((number, "file", target))
    return out


def substitutions(text):
    """text 里最外一层的命令替换 $( … ) 与 `…` → [(里面的文本, 它在 text 里起头的下标)…]：双引号里的也算，单引号里的、$(( 算术 )) 与注释不算。"""
    out, i, total = [], 0, len(text)
    while i < total:
        character = text[i]
        if character == "\\":
            i += 2
            continue
        if character == "'":
            end = text.find("'", i + 1)
            i = total if end < 0 else end + 1
            continue
        if character == "#" and (i == 0 or text[i - 1] in " \t\n;&|("):
            end = text.find("\n", i)
            i = total if end < 0 else end
            continue
        if character == '"':
            j = i + 1
            while j < total and text[j] != '"':
                if text[j] == "\\":
                    j += 2
                    continue
                if text.startswith("$((", j):
                    j = matching_close(text, j + 1, "(", ")") + 1
                    continue
                if text.startswith("$(", j):
                    end = matching_close(text, j + 1, "(", ")")
                    out.append((text[j + 2:end], j + 2))
                    j = end + 1
                    continue
                if text[j] == "`":
                    end = text.find("`", j + 1)
                    if end < 0:
                        break
                    out.append((text[j + 1:end], j + 1))
                    j = end + 1
                    continue
                j += 1
            i = j + 1
            continue
        if text.startswith("$((", i):
            i = matching_close(text, i + 1, "(", ")") + 1
            continue
        if text.startswith("$(", i):
            end = matching_close(text, i + 1, "(", ")")
            out.append((text[i + 2:end], i + 2))
            i = end + 1
            continue
        if character == "`":
            end = text.find("`", i + 1)
            if end < 0:
                break
            out.append((text[i + 1:end], i + 1))
            i = end + 1
            continue
        i += 1
    return out


def matching_close(text, start, opening, closing):
    """text[start] 是 opening；回配对的 closing 的下标（跳过引号），配不上回 len(text) - 1。"""
    depth, i = 0, start
    while i < len(text):
        character = text[i]
        if character == "\\":
            i += 2
            continue
        if character == "'" and opening != "{":
            end = text.find("'", i + 1)
            i = len(text) if end < 0 else end + 1
            continue
        if character == '"':
            i = double_quote_end(text, i)
            continue
        if character == opening:
            depth += 1
        elif character == closing:
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return len(text) - 1


def double_quote_end(text, start):
    i = start + 1
    while i < len(text):
        character = text[i]
        if character == "\\":
            i += 2
            continue
        if character == '"':
            return i + 1
        if text.startswith("$(", i):
            i = matching_close(text, i + 1, "(", ")") + 1
            continue
        if text.startswith("${", i):
            i = matching_close(text, i + 1, "{", "}") + 1
            continue
        i += 1
    return len(text)


def split_commands(text, redirect_targets_as_commands=False, redirects=None, details=None):
    """规范文本 → [[词…]…]：按 ; && || | & ( ) 与换行切成简单命令，词里引号与 $(…) ${…} 原样留着；重定向和它的目标丢掉
    （重定向的目标是写数据、读数据的路径，不是装代码的位置）。redirect_targets_as_commands 只给弄坏开关
    shell-redirect-targets-loaded 用：目标单独当一条命令留下。redirects 给了一个列表的，每个重定向往里记
    （它属于第几条命令、写法如 > >> <<、目标词）；details 给了一个列表的，同样每个重定向记一条（第几条命令、写法、目标词、
    写法在 text 里起头的下标、前面写的文件描述符号或 None）。<( … ) 与 >( … ) 是进程替换，不是重定向：括号里的照命令切。
    不是命令的几段不切成命令：[[ … ]] 与 (( … )) 整段只留 [[ ]] / ((（里面的 || ( ) < 不是分隔），case 分支的模式（a | b) 那一段）跳过，
    数组字面量 x=( … ) 整个留在赋值词里（括号里的是元素，不是命令）。"""
    commands, words, word = [], [], []
    redirect_target = False
    redirect_operator = ""
    redirect_start, redirect_descriptor = 0, None
    i, total = 0, len(text)
    structured = "shell-command-structure-ignored" not in active_breaks()
    cases = []   # 每层 case 此刻在认什么：pattern（分支的模式）/ body（分支里的命令）

    def end_word():
        nonlocal redirect_target
        if word:
            if redirect_target:
                redirect_target = False
                if redirects is not None:
                    redirects.append((len(commands), redirect_operator, "".join(word)))
                if details is not None:
                    details.append((len(commands), redirect_operator, "".join(word), redirect_start, redirect_descriptor))
                if redirect_targets_as_commands:
                    commands.append(["".join(word)])
            else:
                words.append("".join(word))
            word.clear()

    def end_command():
        end_word()
        if words:
            if structured and words[0] == "case" and words[-1] == "in":
                cases.append("pattern")
            elif structured and words == ["esac"] and cases:
                cases.pop()
            commands.append(list(words))
            words.clear()

    while i < total:
        character = text[i]
        if structured and cases and cases[-1] == "pattern" and not words and not word:
            # case 分支的模式：跳到它的 ) 之后；碰到 esac 收这一层
            if character in " \t\n;":
                i += 1
                continue
            if text.startswith("esac", i) and (i + 4 >= total or text[i + 4] in " \t\n;&|)"):
                cases.pop()
                i += 4
                continue
            i = case_pattern_end(text, i) + 1
            cases[-1] = "body"
            continue
        if structured and not word and all(item in SHELL_KEYWORDS or item == "for" for item in words):
            if text.startswith("[[", i) and (i + 2 >= total or text[i + 2] in " \t\n"):
                words += ["[[", "]]"]
                i = test_command_end(text, i + 2)
                continue
            if text.startswith("((", i):
                words.append("((")
                i = matching_close(text, i, "(", ")") + 1
                continue
        if structured and character == ";" and cases and text.startswith((";;", ";&"), i):
            end_command()
            i += 3 if text.startswith(";;&", i) else 2
            cases[-1] = "pattern"
            continue
        if structured and character == "(" and word and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*(?:\[[^\]]*\])?\+?=", "".join(word)):
            end = matching_close(text, i, "(", ")")
            word.append(text[i:end + 1])
            i = end + 1
            continue
        if character == "\\" and i + 1 < total:
            if text[i + 1] == "\n":
                i += 2
                continue
            word.append(text[i:i + 2])
            i += 2
            continue
        if character == "'":
            end = text.find("'", i + 1)
            end = total - 1 if end < 0 else end
            word.append(text[i:end + 1])
            i = end + 1
            continue
        if character == '"':
            end = double_quote_end(text, i)
            word.append(text[i:end])
            i = end
            continue
        if character == "$" and text.startswith("$(", i):
            end = matching_close(text, i + 1, "(", ")")
            word.append(text[i:end + 1])
            i = end + 1
            continue
        if character == "$" and text.startswith("${", i):
            end = matching_close(text, i + 1, "{", "}")
            word.append(text[i:end + 1])
            i = end + 1
            continue
        if character == "`":
            end = text.find("`", i + 1)
            end = total - 1 if end < 0 else end
            word.append(text[i:end + 1])
            i = end + 1
            continue
        if character in " \t":
            end_word()
            i += 1
            continue
        if character in "\n;&|()":
            end_command()
            i += 1
            continue
        if character in "<>":
            if character in "<>" and text.startswith("(", i + 1) and "shell-substitutions-ignored" not in active_breaks():
                # 进程替换 <( … ) / >( … )：括号里是要跑的命令，照命令切（左括号在下一圈当分隔）；
                # 「< <( … )」前面那个 < 的目标就是这个进程替换，不是括号里的第一个词
                end_word()
                redirect_target = False
                i += 1
                continue
            redirect_descriptor = None
            if word and "".join(word).isdigit():
                redirect_descriptor = int("".join(word))
                word.clear()
            else:
                end_word()
            start = i
            while i < total and text[i] in "<>&|-":
                i += 1
            redirect_operator = text[start:i]
            redirect_start = start
            while i < total and text[i] in " \t":
                i += 1
            redirect_target = True
            continue
        if character == "#" and not word:
            end = text.find("\n", i)
            i = total if end < 0 else end
            continue
        word.append(character)
        i += 1
    end_command()
    if structured and details is not None:
        # { … } 3<<'词'：写在收尾那个 } 后面的重定向管着组里的每一条命令
        groups, copies = [], []
        for index, command in enumerate(commands):
            if command and command[0] == "{":
                groups.append(index)
            elif command and command[0] == "}" and groups:
                start = groups.pop()
                for detail in details:
                    if detail[0] == index:
                        copies += [(inner,) + tuple(detail[1:]) for inner in range(start, index)]
        details.extend(copies)
    return commands


def case_pattern_end(text, start):
    """case 分支模式（text[start] 起，可带一个开头的左括号）收尾的那个 ) 的下标；找不到回末尾。"""
    i = start + 1 if text.startswith("(", start) else start
    depth = 0
    while i < len(text):
        character = text[i]
        if character == "\\":
            i += 2
            continue
        if character == "'":
            end = text.find("'", i + 1)
            i = len(text) if end < 0 else end + 1
            continue
        if character == '"':
            i = double_quote_end(text, i)
            continue
        if character == "(":
            depth += 1
        elif character == ")":
            if depth == 0:
                return i
            depth -= 1
        i += 1
    return len(text) - 1


def test_command_end(text, start):
    """[[ … ]]：从 start 起找收尾的 ]]（前面是空白、后面是分隔或末尾），回它之后的下标；引号、$( … ) ${ … } 跳过；到换行就停。"""
    i = start
    while i < len(text):
        character = text[i]
        if character == "\\":
            i += 2
            continue
        if character == "'":
            end = text.find("'", i + 1)
            i = len(text) if end < 0 else end + 1
            continue
        if character == '"':
            i = double_quote_end(text, i)
            continue
        if text.startswith("$(", i):
            i = matching_close(text, i + 1, "(", ")") + 1
            continue
        if text.startswith("${", i):
            i = matching_close(text, i + 1, "{", "}") + 1
            continue
        if text.startswith("]]", i) and text[i - 1] in " \t" and (i + 2 >= len(text) or text[i + 2] in " \t\n;&|)"):
            return i + 2
        if character == "\n":
            return i
        i += 1
    return len(text)


def git_repository_location(words, root, breaks):
    """git … rev-parse … --show-toplevel / --git-common-dir 的输出 → {根} / {根/.git}；别的 git 用法 → None。
    words 是逐个词的字面值（折不出的词是 None，只要 git、rev-parse 与那个选项是字面的就认）。
    根是 worktree 时运行期 --git-common-dir 指的是主仓：库只看给它的这一个根，按根里同一相对路径取，那一份不在照 missing 拒算。"""
    if "git-repository-paths-unfolded" in breaks or not words or words[0] != "git" or "rev-parse" not in words:
        return None
    if "--show-toplevel" in words:
        return {root}
    if "--git-common-dir" in words:
        return {posixpath.join(root, ".git")}
    return None


TREE_COPY_TAIL = re.compile(r"^[A-Za-z0-9_.+@,=-]+(?:/[A-Za-z0-9_.+@,=-]+)+$")


def tree_copy_file(root, tail, directory=False):
    """「<折不出的目录>/<字面尾巴>」→ 尾巴里最长的、至少两段、在根里存在的那一截（相对根）；没有 → None。
    临时树、仓副本、worktree 里的同一相对路径按根里那一份算：拷进去的是它，替身也只会让闭包多收不会少收。"""
    tail = tail.lstrip("/")
    if not TREE_COPY_TAIL.match(tail):
        return None
    parts = tail.split("/")
    if any(part in (".", "..") for part in parts):
        return None
    for start in range(len(parts) - 1):
        candidate = "/".join(parts[start:])
        absolute = os.path.join(root, candidate)
        if (os.path.isdir(absolute) if directory else os.path.isfile(absolute)):
            return candidate
    return None


INLINE_POSITIONAL = re.compile(r"\$(?:\{([1-9])\}|([1-9]))")


def inline_positionals(inline, rest):
    """bash -c '<串>' <$0> <$1>…：串里的 $1…$9 换成外面那几个词（去掉外层双引号，留在串里原来的引号里；
    外面没给的、带单引号的照原样留着，照旧折不出）。"""
    def replace(found):
        position = int(found.group(1) or found.group(2))
        if position >= len(rest) or "'" in rest[position]:
            return found.group(0)
        return rest[position].replace('"', "")
    return INLINE_POSITIONAL.sub(replace, inline)


SHELL_TREE_PREFIX = re.compile(r"^(?:\$\{(?:[A-Za-z_][A-Za-z0-9_]*|[1-9])(?:[:]?[-=?][^{}]*)?\}|\$(?:[A-Za-z_][A-Za-z0-9_]*|[1-9]))(/.+)$")


def literal_word(word):
    """没有展开的词 → 去掉引号后的字面值；有 $ ` 展开的 → None。双引号里的单引号是字面的，双引号里的反斜杠只在 $ ` " \\ 前面才转义。"""
    out, i, quoted = [], 0, False
    while i < len(word):
        character = word[i]
        if character == "'" and not quoted:
            end = word.find("'", i + 1)
            if end < 0:
                return None
            out.append(word[i + 1:end])
            i = end + 1
            continue
        if character == '"':
            quoted = not quoted
            i += 1
            continue
        if character in "$`":
            return None
        if character == "\\" and i + 1 < len(word):
            if quoted and word[i + 1] not in '$`"\\\n':
                out.append(character)
                i += 1
                continue
            out.append(word[i + 1])
            i += 2
            continue
        out.append(character)
        i += 1
    return "".join(out)


class ShellFacts:
    """invocations：[(写法, 词, cd 过去的目录词或 None, 附带)]，写法 source / shell / python / exec / python-module / python-inline / shell-inline /
    inline-unknown（附带见 command_facts）；handled：每个解释器词各记一条它按哪一种写法认的（只给量全仓的人看，不进判定）。"""
    __slots__ = ("commands", "words", "command_words", "variable_references", "assignments", "local_names", "invocations", "programs", "directory",
                 "generated", "handled")

    def __init__(self):
        self.commands, self.words, self.variable_references, self.generated = [], set(), set(), {}
        self.command_words, self.assignments, self.invocations, self.programs = set(), collections.defaultdict(list), [], set()
        self.local_names, self.directory, self.handled = set(), None, []


GENERATOR_HEADS = frozenset(("printf", "echo"))
COPYING_HEADS = frozenset(("cp", "mv", "install", "ln", "rsync", "tee", "dd", "touch"))


def generated_scripts(commands, redirects, rows, mask):
    """单元自己写出来、拷出来的脚本：目标词（去掉双引号）→ [这个单元写进去的 here-doc 正文…]，或 ("copied", [源词…])。
    写出来的：写它的每一处都得是 printf / echo 的输出重定向，或不带别的参数的 cat 接 here-doc；有一处别的写法（tee、别的命令重定向、cp）就不算。
    printf / echo 写进去的那几行不当代码看（格式串里认不出哪是代码）；cat <<词 的正文照代码看。
    拷出来的：写它的只有「cp <源> <它>」（两个操作数），代码就是源那一份。"""
    writers = collections.defaultdict(list)
    for index, operator, target in redirects:
        if not operator.startswith(">") or operator.startswith(">&") and target.isdigit():
            continue
        command = commands[index] if index < len(commands) else []
        head = literal_word(command[0]) if command else None
        operators = {other for position, other, _target in redirects if position == index}
        if head in GENERATOR_HEADS or (head == "cat" and len(command) == 1 and any(other in ("<<", "<<-") for other in operators)):
            writers[target.replace('"', "")].append(head)
        else:
            writers[target.replace('"', "")].append(None)
    copies = collections.defaultdict(list)
    for command in commands:
        head = literal_word(command[0]) if command else None
        if head not in COPYING_HEADS:
            continue
        operands = [argument for argument in command[1:] if not (literal_word(argument) or "").startswith("-")]
        if head == "cp" and len(operands) == 2:
            copies[operands[1].replace('"', "")].append(operands[0])
            continue
        for argument in command[1:]:
            writers[argument.replace('"', "")].append(None)
    for target, heads in writers.items():
        if target in copies:
            copies[target].append(None)
    generated = {target: [] for target, heads in writers.items() if heads and None not in heads and target not in copies}
    generated.update({target: ("copied", sources) for target, sources in copies.items() if None not in sources})
    for position, row in enumerate(rows):
        if position in mask:
            continue
        starts = list(HEREDOC_START.finditer(row))
        if len(starts) != 1:
            continue
        delimiter, strip_tabs = starts[0].group(3), starts[0].group(1) == "-"
        body, later = [], position + 1
        while later < len(rows) and (rows[later].lstrip("\t") if strip_tabs else rows[later]) != delimiter:
            body.append(rows[later])
            later += 1
        if later >= len(rows):
            continue
        row_redirects = []
        row_commands = split_commands(row, False, row_redirects)
        for index, operator, target in row_redirects:
            key = target.replace('"', "")
            if operator.startswith(">") and isinstance(generated.get(key), list) and index < len(row_commands) and literal_word(row_commands[index][0]) == "cat":
                generated[key].append("\n".join(body))
    return generated


class ShellFile:
    def __init__(self, path, content, normal, breaks):
        self.path, self.raw = path, content
        self.text = content.decode("utf-8", "surrogateescape")
        self.rows = self.text.split("\n")
        self.refusals, self.units = [], []
        self.functions = collections.defaultdict(list)
        self.variables = collections.defaultdict(list)
        self.script_body = self.kept = None
        self.breaks = breaks
        if normal is None:
            self.refusals.append(Refusal("bash-syntax", path, 1, snippet_of(self.rows[0] if self.rows else "")))
            return
        self.parse(normal)

    def parse(self, normal):
        lines = normal.split("\n")
        while lines and lines[-1] == "":
            lines.pop()
        if len(lines) < 3 or not lines[0].startswith(SHELL_WRAPPER) or lines[1].strip() != "{" or lines[-1] != "}":
            self.refusals.append(Refusal("bash-syntax", self.path, 1, snippet_of(self.rows[0] if self.rows else "")))
            return
        body = lines[2:-1]
        mask = set() if "shell-heredoc-split" in self.breaks else heredoc_mask(body)
        functions = []
        for position, row in enumerate(body):
            if position in mask:
                continue
            found = FUNCTION_HEADER.match(row)
            if not found or position + 1 >= len(body) or body[position + 1] != found.group(1) + "{ ":
                continue
            indent = found.group(1)
            end = None
            for later in range(position + 2, len(body)):
                if later in mask:
                    continue
                candidate = body[later]
                if candidate.startswith(indent + "}") and (len(candidate) == len(indent) + 1 or candidate[len(indent) + 1] in " ;|&<>)"):
                    end = later
                    break
            if end is not None:
                functions.append((position, end, len(indent), found.group(2)))
        functions.sort()
        parents, stack, by_range = {}, [], {}
        for function in functions:
            while stack and stack[-1][1] < function[0]:
                stack.pop()
            parents[function] = stack[-1] if stack else None
            stack.append(function)
        cursor = 0
        for function in functions:
            start, end, _indent, name = function
            parent = by_range.get(parents[function])
            qualified = name if parent is None else f"{parent.name}::{name}"
            text = "\n".join(body[start:end + 1])
            first_line, last_line, cursor = self.original_lines(name, cursor)
            unit = Unit("shell", "function", qualified, self.path, first_line, last_line, sha256_text(text))
            unit.short, unit.body, unit.container = name, text, parent
            by_range[function] = unit
            self.functions[name].append(unit)
            self.units.append(unit)
        covered = set()
        for function in functions:
            if function[2] == 4:
                covered |= set(range(function[0], function[1] + 1))
        body_text = "\n".join(row for position, row in enumerate(body) if position not in covered)
        fingerprint = sha256_bytes(self.raw) if "shell-raw-text" in self.breaks else sha256_text(body_text)
        self.script_body = Unit("shell", "script-body", "<script-body>", self.path, 1, max(len(self.rows), 1), fingerprint)
        self.script_body.body = body_text
        self.units.append(self.script_body)
        cursors = collections.defaultdict(int)
        position = 0
        while position < len(body):
            found = TOP_ASSIGNMENT.match(body[position]) if position not in covered and position not in mask else None
            if not found:
                position += 1
                continue
            end = position
            while end + 1 < len(body) and (end in mask or not body[end].rstrip().endswith(";")) and "shell-variable-first-line-only" not in self.breaks:
                end += 1
            if any(index in covered for index in range(position, end + 1)):
                end = position
            text = "\n".join(body[position:end + 1]).strip()
            name = found.group(1)
            pattern = re.compile(rf"^(?:(?:export|readonly|declare|typeset)(?:\s+-\w+)*\s+)?{re.escape(name)}(?:\[[^\]]*\])?\+?=")
            row_index = next((number for number in range(cursors[name], len(self.rows)) if pattern.match(self.rows[number])), None)
            if row_index is None:
                row_index = max(cursors[name] - 1, 0)
            cursors[name] = row_index + 1
            unit = Unit("shell", "shell-variable", name, self.path, row_index + 1, min(row_index + 1 + end - position, len(self.rows)), sha256_text(text))
            unit.body = text.rstrip(";")
            self.variables[name].append(unit)
            self.units.append(unit)
            position = end + 1
        kept = kept_shell_comments(self.text)
        if kept:
            self.kept = Unit("shell", "kept-comments", "<kept-comments>", self.path, 1, max(len(self.rows), 1), sha256_text("\n".join(kept)), data=True)
            self.units.append(self.kept)

    def original_lines(self, name, cursor):
        header = re.compile(rf"(?:^|[;&|{{(]\s*)\s*((?:function\s+{re.escape(name)}(?:\s*\(\s*\))?|{re.escape(name)}\s*\(\s*\))\s*(?:\{{|\(|$|#))")
        found = next((position for position in range(cursor, len(self.rows)) if header.search(self.rows[position])), None)
        if found is None:
            found = next((position for position in range(0, len(self.rows)) if header.search(self.rows[position])), None)
        if found is None:
            return 1, 1, cursor
        column = header.search(self.rows[found]).start(1)
        return found + 1, shell_block_end(self.rows, found, column) + 1, found + 1


def shell_block_end(rows, start, column=0):
    """原文里 rows[start] 第 column 列起的函数定义到哪一行收尾（只当索引；认不出回起头那一行）。逐行扫，带一个语境栈：
    单引号、双引号、$( … ) 可以跨行、可以互相套，here-doc（含 "$(… <<'词'" 这种写在命令替换里的）的正文整段跳过。"""
    header = re.compile(r"\s*(?:function\s+)?[^\s(]+\s*(?:\(\s*\))?\s*").match(rows[start], column)
    row_index, column = start, header.end() if header else column
    while row_index < len(rows):
        rest = rows[row_index][column:]
        if rest.strip():
            column += len(rest) - len(rest.lstrip())
            break
        row_index, column = row_index + 1, 0
    if row_index >= len(rows) or rows[row_index][column] not in "{(":
        return start
    opener = rows[row_index][column]
    depth = 0
    stack = []          # 语境："'"、'"'、"$(" 各一层；空的时候在函数体本层
    parens = [0]        # 每一层 $( 里还开着几个 (
    while row_index < len(rows):
        row, i, heredocs = rows[row_index], column, []
        while i < len(row):
            character = row[i]
            top = stack[-1] if stack else None
            if top == "'":
                if character == "'":
                    stack.pop()
                i += 1
                continue
            if top == '"':
                if character == "\\":
                    i += 2
                    continue
                if character == '"':
                    stack.pop()
                elif row.startswith("$(", i):
                    stack.append("$(")
                    parens.append(0)
                    i += 2
                    continue
                i += 1
                continue
            if character == "\\":
                i += 2
                continue
            if character in ("'", '"'):
                stack.append(character)
                i += 1
                continue
            if character == "#" and (i == 0 or row[i - 1] in " \t;"):
                break
            if row.startswith("<<", i) and not row.startswith("<<<", i):
                found = HEREDOC_START.match(row, i)
                if found:
                    heredocs.append((found.group(3), found.group(1) == "-"))
                    i = found.end()
                    continue
            if row.startswith("${", i):
                i = matching_close(row, i + 1, "{", "}") + 1
                continue
            if row.startswith("$(", i):
                stack.append("$(")
                parens.append(0)
                i += 2
                continue
            if top == "$(":
                if character == "(":
                    parens[-1] += 1
                elif character == ")":
                    if parens[-1] == 0:
                        stack.pop()
                        parens.pop()
                    else:
                        parens[-1] -= 1
                i += 1
                continue
            if opener == "{":
                before = i == 0 or row[i - 1] in " \t;&|()"
                after = i + 1 >= len(row) or row[i + 1] in " \t;)|&<>"
                if character == "{" and before and after:
                    depth += 1
                elif character == "}" and before and after:
                    depth -= 1
                    if depth == 0:
                        return row_index
            elif character == "(":
                depth += 1
            elif character == ")":
                depth -= 1
                if depth == 0:
                    return row_index
            i += 1
        row_index, column = row_index + 1, 0
        for delimiter, strip_tabs in heredocs:
            while row_index < len(rows) and (rows[row_index].lstrip("\t") if strip_tabs else rows[row_index]) != delimiter:
                row_index += 1
            row_index += 1
    return start


class ShellIndex:
    def __init__(self, root, paths):
        self.root = os.path.abspath(root)
        self.breaks = active_breaks()
        self.files = {}
        self.sources_of, self.sourcers_of = {}, collections.defaultdict(set)
        self.folding = set()
        # 正在折的内联 bash 串里自己赋的变量（一层一份，里层先看）；$(cat <<'词' … 词) 那段命令替换的输出（here-doc 正文）
        self.overlays, self.substitution_outputs = [], collections.defaultdict(set)
        self.add(paths)

    def add(self, paths):
        new = [path for path in dict.fromkeys(paths) if path not in self.files and os.path.isfile(os.path.join(self.root, path))]
        if not new:
            return
        contents = []
        for path in new:
            with open(os.path.join(self.root, path), "rb") as handle:
                contents.append(handle.read())
        forms = shell_normal_forms(contents)
        for path, content, form in zip(new, contents, forms):
            self.files[path] = ShellFile(path, content, form, self.breaks)
        self.compute_groups()

    def compute_groups(self):
        """source 边（有向：谁 source 了谁）。折叠路径要用变量，变量又可能赋在被 source 的那份里，所以来回算到不再变（至多三趟）。"""
        self.sources_of = {path: frozenset() for path in self.files}
        for _round in range(3):
            edges = collections.defaultdict(set)
            for path, shell_file in self.files.items():
                for unit in shell_file.units:
                    if unit.data:
                        continue
                    for kind, targets, _refusal in self.links(unit):
                        if kind == "source":
                            edges[path].update(target for target in targets if target in self.files and target != path)
            sources = {path: frozenset(edges.get(path, ())) for path in self.files}
            if sources == self.sources_of:
                break
            self.sources_of = sources
            for shell_file in self.files.values():
                for unit in shell_file.units:
                    if unit.cache is not None:
                        unit.cache.pop("links", None)
        self.sourcers_of = collections.defaultdict(set)
        for path, targets in self.sources_of.items():
            for target in targets:
                self.sourcers_of[target].add(path)

    def downward(self, paths):
        """这几份与它们（传递地）source 进来的全部文件。"""
        seen, todo = set(), list(paths)
        while todo:
            path = todo.pop()
            if path in seen:
                continue
            seen.add(path)
            todo += list(self.sources_of.get(path, ()))
        return seen

    def upward(self, path):
        """（传递地）source 了这一份的全部文件，不含它自己。"""
        seen, todo = set(), list(self.sourcers_of.get(path, ()))
        while todo:
            item = todo.pop()
            if item in seen or item == path:
                continue
            seen.add(item)
            todo += list(self.sourcers_of.get(item, ()))
        return seen

    def facts(self, unit):
        if unit.facts is not None:
            return unit.facts
        facts = ShellFacts()
        text = unit.body or ""
        rows = text.split("\n")
        mask = set() if "shell-heredoc-split" in self.breaks else heredoc_mask(rows)
        visible = "\n".join("" if position in mask else row for position, row in enumerate(rows))
        redirects = []
        details = []
        facts.commands = split_commands(visible, "shell-redirect-targets-loaded" in self.breaks, redirects, details)
        facts.generated = {} if "shell-generated-scripts-ignored" in self.breaks else generated_scripts(facts.commands, redirects, rows, mask)
        facts.words = set(SHELL_WORDS.findall(text))
        facts.variable_references = set(VARIABLE_REFERENCE.findall(text))
        bodies = heredoc_bodies(rows) if mask else {}
        self.scan_commands(facts.commands, details, 0, visible, bodies, facts)
        self.scan_substitutions(visible, 0, visible, bodies, facts, 0)
        unit.facts = facts
        return facts

    def scan_commands(self, commands, details, base, visible, bodies, facts):
        inputs = command_inputs(details, base, visible, bodies)
        for index, command in enumerate(commands):
            self.command_facts(command, facts, inputs.get(index, ()))

    def scan_substitutions(self, fragment, base, visible, bodies, facts, depth):
        """命令替换 $( … ) 与 `…` 里的命令照样是要跑的命令（x="$(bash y.sh)"、if out="$(python3 - <<'PY' …)"）：逐层切开再认。"""
        if "shell-substitutions-ignored" in self.breaks or depth > 8:
            return
        for inner, offset in substitutions(fragment):
            details = []
            commands = split_commands(inner, "shell-redirect-targets-loaded" in self.breaks, None, details)
            self.scan_commands(commands, details, base + offset, visible, bodies, facts)
            if len(commands) == 1 and commands[0] and literal_word(commands[0][0]) == "cat" and len(commands[0]) == 1:
                heredocs = command_inputs(details, base + offset, visible, bodies).get(0, [])
                if len(heredocs) == 1 and heredocs[0][:2] == (0, "heredoc") and heredocs[0][2] is not None:
                    self.substitution_outputs[inner].add(heredocs[0][2])
            self.scan_substitutions(inner, base + offset, visible, bodies, facts, depth + 1)

    def scan_text(self, text, facts):
        """一段独立的 shell 正文（bash -c 的串、喂给 bash 的 here-doc）：切命令、认 here-doc 与命令替换，记进 facts。"""
        rows = text.split("\n")
        mask = set() if "shell-heredoc-split" in self.breaks else heredoc_mask(rows)
        visible = "\n".join("" if position in mask else row for position, row in enumerate(rows))
        details = []
        commands = split_commands(visible, "shell-redirect-targets-loaded" in self.breaks, None, details)
        bodies = heredoc_bodies(rows) if mask else {}
        self.scan_commands(commands, details, 0, visible, bodies, facts)
        self.scan_substitutions(visible, 0, visible, bodies, facts, 0)

    def command_facts(self, command, facts, inputs=()):
        words = list(command)
        while words and words[0] in SHELL_KEYWORDS:
            words = words[1:]
        if not words:
            return
        if words[0] == "for" and len(words) >= 3 and words[2] == "in":
            for value in words[3:]:
                facts.assignments[words[1]].append(value)
            return
        if words[0] in ("case", "[[", "[", "]]"):
            pass
        declaration = words[0] in SHELL_DECLARATIONS
        scoped = words[0] == "local" or (words[0] in ("declare", "typeset") and "-g" not in words[1:])
        start = 1 if declaration else 0
        position = start
        while position < len(words):
            found = ASSIGNMENT_WORD.match(words[position])
            if found or (scoped and re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", words[position])):
                name = found.group(1) if found else words[position]
                if found:
                    facts.assignments[name].append(found.group(2))
                if scoped:
                    facts.local_names.add(name)
                position += 1
                continue
            if declaration and words[position].startswith("-"):
                position += 1
                continue
            break
        if declaration:
            return
        prefix = [(found.group(1), found.group(2)) for found in (ASSIGNMENT_WORD.match(item) for item in words[:position]) if found]
        words = words[position:]
        if not words:
            return
        first = literal_word(words[0])
        facts.command_words.add(first if first is not None else words[0])
        if first == "cd":
            arguments = [word for word in words[1:] if not (literal_word(word) or "").startswith("-")]
            facts.directory = arguments[0] if len(arguments) == 1 else None
            return
        if first not in MESSAGE_HEADS or "external-program-words-unfiltered" in self.breaks:
            for word in words:
                program = external_program_name(literal_word(word), self.breaks)
                if program is not None:
                    facts.programs.add(program)
        if first in ("source", ".") and len(words) > 1:
            facts.invocations.append(("source", words[1], facts.directory, None))
            return
        if self.interpreter_runs(words, facts, inputs, prefix):
            return
        values = [literal_word(word) for word in words]
        positions = [0] if "shell-wrapped-commands-ignored" in self.breaks else command_positions(values)
        head = positions[-1]
        if head >= len(words):
            return
        raw = words[head]
        value = values[head]
        unquoted = raw.replace('"', "").replace("'", "")
        if unquoted.endswith((".sh", ".py", ".bash")) or (value is not None and "/" in value and not value.startswith("/usr") and not value.startswith("/bin")):
            facts.invocations.append(("exec", raw, facts.directory, None))
        elif value is None and "shell-unfolded-commands-allowed" not in self.breaks:
            # 命令位置上折不出的程序：到 links 里再折，折出解释器照解释器认、折出仓里的脚本连上、折不出拒算（"$@" 一类除外：命令就是调用处那几个词）
            facts.invocations.append(("command", raw, facts.directory, (words[head + 1:], tuple(inputs))))
        if "shell-path-arguments-loaded" in self.breaks:
            # 弄坏开关：把参数位置上长得像脚本的路径也当装代码（命令位置之外的路径是传给那条命令的数据，不装代码）
            for argument in words[1:]:
                if argument.replace('"', "").replace("'", "").endswith((".sh", ".py", ".bash")):
                    facts.invocations.append(("exec", argument, facts.directory, None))

    def interpreter_runs(self, words, facts, inputs, prefix=()):
        """这条命令里每一个解释器词（bash / sh / python3 / python，在哪个位置都算：包装命令 env、sudo、xargs、capped.sh、
        run-with-memory-cap.sh 后面的照样认）各按它的写法记一条要装的代码；→ 认到了几个解释器词（0 交给直接执行那一条）。
        脚本之后接着往后认（bash capped.sh N python3 x.py 里的 python3 x.py）；-c 的串、标准输入喂的脚本之后的词是它的参数，不再往后认。"""
        values = [literal_word(word) for word in words]
        positions = command_positions(values)
        found, position = 0, 0
        while position < len(words):
            value = values[position]
            base = posixpath.basename(value) if value else None
            if base not in SHELL_INTERPRETERS and base not in PYTHON_INTERPRETERS:
                position += 1
                continue
            found += 1
            python = base in PYTHON_INTERPRETERS
            language = "python" if python else "shell"
            mode, argument = interpreter_mode(values, position, python)
            facts.handled.append((language, mode, position))
            # 同一条命令里设给它的环境变量：命令前的 X=…（prefix）与 env 后面的 X=…
            environment = tuple(prefix) + tuple((found_word.group(1), found_word.group(2)) for found_word in
                                                (ASSIGNMENT_WORD.match(item) for item in words[:position]) if found_word)
            if mode == "script":
                descriptor = descriptor_of(values[argument])
                if descriptor is not None:
                    self.stdin_script(language, words[position], words[argument + 1:], inputs, descriptor, facts, environment)
                    return found
                facts.invocations.append((language, words[argument], facts.directory, None))
                if "shell-later-interpreters-ignored" in self.breaks:
                    return found
                position = argument + 1
                continue
            if mode == "inline":
                if argument + 1 < len(words):
                    # bash -c '串' $0 $1 …；python3 -c '串' 之后的词是 sys.argv[1:]（sys.argv[0] 是 -c）
                    rest = (["-c"] if python else []) + words[argument + 2:]
                    self.inline_run(language, words[position], words[argument + 1], rest, facts, environment)
                return found
            if mode == "module":
                if argument + 1 < len(words) and "python-modules-ignored" not in self.breaks:
                    facts.invocations.append(("python-module", words[argument + 1], facts.directory, (words[position], words[argument + 2:])))
                if "shell-later-interpreters-ignored" in self.breaks:
                    return found
                position = argument + 2
                continue
            if mode == "stdin":
                self.stdin_script(language, words[position], words[argument + 1:], inputs, 0, facts, environment)
                return found
            if mode == "bare" and position in positions:
                # 命令位置上的解释器后面没有脚本：脚本从标准输入来（here-doc、<<<、< 文件；都没有就是管道，拒算）
                self.stdin_script(language, words[position], [], inputs, 0, facts, environment)
            # syntax-only（bash -n 只查语法）与参数位置上光一个解释器名（command -v python3、which bash）：不装代码
            return found
        return found

    def stdin_script(self, language, interpreter, argv, inputs, descriptor, facts, environment=()):
        """解释器从文件描述符 descriptor（0 是标准输入）读脚本：同一条命令里给这个描述符的 here-doc、here-string、< 文件就是脚本；
        找不到（管道喂的、外面给的）拒算。argv 是脚本之后的那几个词（$1… / sys.argv[1:]）。"""
        source = None
        for number, form, payload in inputs:
            if number == descriptor:
                source = (form, payload)
        if "inline-scripts-ignored" in self.breaks:
            return
        if source is None:
            facts.invocations.append(("inline-unknown", interpreter, facts.directory, (language,)))
        elif source[0] == "file":
            facts.invocations.append((language, source[1], facts.directory, None))
        elif source[0] == "herestring":
            self.inline_run(language, interpreter, source[1], ["-"] + list(argv), facts, environment)
        elif source[1] is None:
            facts.invocations.append(("inline-unknown", interpreter, facts.directory, (language,)))
        else:
            quoted, body = source[1]
            payload = ("text", body) if quoted else ("heredoc", body)
            facts.invocations.append((f"{language}-inline", interpreter, facts.directory, (payload, ["-"] + list(argv), tuple(environment))))

    def inline_run(self, language, interpreter, word, rest, facts, environment=()):
        """-c 的串（rest 是串之后的词：$0、$1…；python 的 sys.argv[1:] 是 rest[1:]）。字面的 bash 串当场切开再认（串里的赋值算进这个单元）；
        python 的串与带展开的串到 links 里再折。"""
        if "inline-scripts-ignored" in self.breaks:
            if language == "python":
                return
        inline = literal_word(word)
        if language == "shell" and inline is not None:
            if "shell-inline-positionals-ignored" not in self.breaks:
                inline = inline_positionals(inline, rest)
            self.scan_text(inline, facts)
            return
        if "inline-scripts-ignored" in self.breaks:
            return
        payload = ("text", inline) if inline is not None else ("word", word)
        facts.invocations.append((f"{language}-inline", interpreter, facts.directory, (payload, list(rest), tuple(environment))))

    def assignments_of(self, name, unit):
        """一个变量的赋值词：这个单元自己赋过的只看自己的（local 的作用域）；没有就取它那份文件里全部单元的（别的函数里 local 的不算），
        还没有就取它（传递地）source 进来的那几份的，再没有取 source 了它的那几份的。"""
        for overlay in reversed(self.overlays):
            if overlay.get(name):
                return overlay[name]
        if unit is None:
            return []
        own = self.facts(unit).assignments.get(name) if unit.kind == "function" else None
        if own:
            return own
        container = unit.container if "shell-enclosing-locals-ignored" not in self.breaks else None
        while container is not None:
            # 套在外层函数里的函数：外层的 local 它看得见（bash 按调用链找变量，套着定义的只在外层里调）
            enclosing = self.facts(container).assignments.get(name)
            if enclosing:
                return enclosing
            container = container.container

        def gathered(paths):
            found = []
            for path in sorted(paths):
                for other in self.files[path].units:
                    if other.data or other.kind == "shell-variable":
                        continue
                    other_facts = self.facts(other)
                    if other.kind == "function" and name in other_facts.local_names:
                        continue
                    found += other_facts.assignments.get(name, [])
            return found

        for paths in ([unit.file], self.downward([unit.file]) - {unit.file}, self.upward(unit.file)):
            found = gathered([path for path in paths if path in self.files])
            if found:
                return found
        return []

    def fold_word(self, word, unit, progress):
        results = {""}
        i, total, quoted = 0, len(word), False

        def extend(values):
            nonlocal results
            results = {result + value for result in results for value in values}

        while i < total:
            character = word[i]
            if character == '"':
                quoted = not quoted
                i += 1
                continue
            if character == "'" and not quoted:
                end = word.find("'", i + 1)
                if end < 0:
                    return None
                extend({word[i + 1:end]})
                i = end + 1
                continue
            if character == "\\" and i + 1 < total:
                if quoted and word[i + 1] not in '$`"\\\n':
                    extend({character})
                    i += 1
                    continue
                extend({word[i + 1]})
                i += 2
                continue
            if character == "$":
                if word.startswith("$((", i):
                    return None
                if word.startswith("$(", i):
                    end = matching_close(word, i + 1, "(", ")")
                    values = self.fold_substitution(word[i + 2:end], unit, progress)
                    i = end + 1
                elif word.startswith("${", i):
                    end = matching_close(word, i + 1, "{", "}")
                    values = self.fold_parameter(word[i + 2:end], unit, progress)
                    i = end + 1
                else:
                    found = re.match(r"\$([A-Za-z_][A-Za-z0-9_]*|[0-9@*#?$!-])", word[i:])
                    if not found:
                        extend({"$"})
                        i += 1
                        continue
                    values = self.fold_parameter(found.group(1), unit, progress)
                    i += found.end()
                if values is None:
                    return None
                extend(values)
                if len(results) > FOLD_LIMIT:
                    return None
                continue
            if character == "`":
                end = word.find("`", i + 1)
                if end < 0:
                    return None
                values = self.fold_substitution(word[i + 1:end], unit, progress)
                if values is None:
                    return None
                extend(values)
                i = end + 1
                continue
            if character == "~" and i == 0 and not quoted:
                return None
            extend({character})
            i += 1
        return results

    def fold_parameter(self, inner, unit, progress):
        found = re.match(r"^([A-Za-z_][A-Za-z0-9_]*|[0-9]+|[@*#?$!-])(\[[^\]]*\])?(.*)$", inner, re.S)
        if not found:
            return None
        name, subscript, rest = found.group(1), found.group(2), found.group(3)
        absolute = os.path.join(self.root, unit.file) if unit is not None else None
        if name == "BASH_SOURCE" and subscript in (None, "[0]"):
            base = {absolute}
        elif name == "0":
            base = {absolute}
        elif name == "CLAUDE_PROJECT_DIR":
            base = {self.root}
        elif name.isdigit() or name in "@*#?$!-" or subscript is not None:
            base = None
        elif name in progress:
            base = set()
        else:
            words = self.assignments_of(name, unit)
            if not words:
                base = None
            else:
                base = set()
                for value in words:
                    folded = self.fold_word(value, unit, progress | {name})
                    if folded is None:
                        base = None
                        break
                    base |= folded
                if base is not None and len(base) > 1:
                    base.discard("")
        if rest == "":
            return base
        for operator in (":-", ":=", "-", "="):
            if rest.startswith(operator):
                default = self.fold_word(rest[len(operator):], unit, progress)
                if default is None:
                    return None
                return default if not base else base | default
        if rest.startswith((":?", "?")):
            return base
        if rest == "%/*" and base:
            return {posixpath.dirname(value) for value in base}
        return None

    def fold_substitution(self, inner, unit, progress):
        captured = self.substitution_outputs.get(inner)
        if not captured and "shell-heredoc-captures-unfolded" not in self.breaks:
            captured = self.function_output(inner, unit)
        if captured:
            # $(cat <<'词' … 词)：输出就是 here-doc 正文（不带引号的词照 bash 展开正文）
            values = set()
            for quoted, body in sorted(captured):
                folded = {body} if quoted else self.fold_heredoc(body, unit, progress)
                if folded is None:
                    return None
                values |= folded
            return values
        commands = split_commands(inner)
        if len(commands) == 1:
            words = commands[0]
            head = literal_word(words[0])
            arguments = [word for word in words[1:] if not (literal_word(word) or "").startswith("-")]
            if head in ("dirname", "realpath", "readlink", "basename") and len(arguments) == 1:
                folded = self.fold_word(arguments[0], unit, progress)
                if folded is None:
                    return None
                if head == "dirname":
                    return {posixpath.dirname(value) for value in folded}
                if head == "basename":
                    return {posixpath.basename(value) for value in folded}
                return {posixpath.normpath(value) for value in folded}
            if head == "git":
                return git_repository_location([literal_word(word) for word in words], self.root, self.breaks)
            return None
        if len(commands) == 2 and literal_word(commands[0][0]) == "cd" and literal_word(commands[1][0]) == "pwd":
            arguments = [word for word in commands[0][1:] if not (literal_word(word) or "").startswith("-")]
            if len(arguments) == 1:
                folded = self.fold_word(arguments[0], unit, progress)
                if folded is None:
                    return None
                # 相对目录照 target_of 认相对路径的办法往下传（先根、再这份文件所在的目录）：
                # 「d=${BASH_SOURCE[0]%/*}; [[ $d == ${BASH_SOURCE[0]} ]] && d=.」那个 . 就是这份文件自己的目录
                return {posixpath.normpath(value) for value in folded}
        return None

    def target_of(self, value, unit, directories=None):
        """折出来的路径值 → ("ok", 相对根的文件) / ("outside", None) / ("missing", None)；相对路径先按前面 cd 过去的目录试。"""
        if posixpath.isabs(value):
            relative = inside_root(self.root, value)
            if relative is None:
                return ("outside", None)
            return ("ok", relative) if os.path.isfile(os.path.join(self.root, relative)) else ("missing", None)
        bases = sorted(directories) if directories else []
        for base in bases + [self.root, os.path.dirname(os.path.join(self.root, unit.file))]:
            candidate = os.path.normpath(os.path.join(base, value))
            relative = inside_root(self.root, candidate)
            if relative is not None and os.path.isfile(candidate):
                return ("ok", relative)
        return ("missing", None)

    def links(self, unit):
        """→ [(source|shell|python, [文件…], 拒算或 None)]。"""
        if unit.cache is None:
            unit.cache = {}
        if "links" in unit.cache:
            return unit.cache["links"]
        out = []
        shell_file = self.files.get(unit.file)
        placed = {}
        facts = self.facts(unit)
        # 写出来、拷出来的脚本：这个单元与套着它的外层函数里写的都算（嵌套函数用的是外层的局部变量）
        generated, container = {}, unit
        while container is not None:
            for target, written in self.facts(container).generated.items():
                generated.setdefault(target, written)
            container = container.container
        work = [(kind, word, directory, extra, None, None) for kind, word, directory, extra in facts.invocations]
        state = {"unit": unit, "shell_file": shell_file, "facts": facts, "generated": generated, "expanded": set(), "placed": placed, "out": out,
                 "work": work}
        while work:
            kind, word, directory, extra, generator_line, overlay = work.pop(0)
            self.overlays.append(overlay or {})
            try:
                self.resolve_invocation(state, kind, word, directory, extra, generator_line, overlay)
            finally:
                self.overlays.pop()
        unit.cache["links"] = out
        return out

    def row_snippet(self, shell_file, unit, line, word):
        if shell_file is not None and 0 < line <= len(shell_file.rows):
            return snippet_of(shell_file.rows[line - 1])
        if unit.scope == "inline" and unit.qualified:
            return unit.qualified
        return snippet_of(word)

    def place(self, state, kind, word, generator_line, loader=None):
        """拒算与内联脚本落在哪一行（同一个词装了几次各落各的行）。"""
        if generator_line:
            return generator_line
        placed = state["placed"]
        line = self.word_line(state["shell_file"], state["unit"], word, loader or kind, placed.get((kind, word), 0))
        placed[(kind, word)] = line
        return line

    def resolve_invocation(self, state, kind, word, directory, extra, generator_line, overlay):
        unit, shell_file, facts, out, work = state["unit"], state["shell_file"], state["facts"], state["out"], state["work"]
        if kind in ("python-inline", "shell-inline", "inline-unknown"):
            self.resolve_inline(state, kind, word, directory, extra, generator_line)
            return
        if kind == "python-module":
            self.resolve_module_run(state, word, directory, extra)
            return
        if kind == "command":
            self.resolve_command(state, word, directory, extra, generator_line, overlay)
            return
        if generator_line is not None and literal_word(word) is None:
            # 生成脚本正文里带展开的路径：那是生成出来的脚本自己运行时的变量，这个单元的赋值替它作不了保
            if "shell-unfolded-paths-ignored" not in self.breaks:
                out.append((kind, [], Refusal("shell-path-unfolded", unit.file, generator_line, self.row_snippet(shell_file, unit, generator_line, word))))
            return
        values = self.fold_word(word, unit, frozenset())
        directories = self.fold_word(directory, unit, frozenset()) if directory is not None else None
        if values is not None and directory is not None and directories is None and any(not posixpath.isabs(value) for value in values):
            values = None
        line = self.place(state, kind, word, generator_line)
        if values is None:
            unquoted = word.replace('"', "").replace("'", "")
            if kind == "exec" and not unquoted.endswith((".sh", ".py", ".bash")):
                return
            written = state["generated"].get(unquoted)
            if isinstance(written, tuple) and "shell-copied-scripts-ignored" not in self.breaks:
                # 这个单元自己 cp 出来的脚本：代码就是源那一份，源折得出就连过去
                files = self.copied_sources(written[1], unit)
                if files is not None:
                    if files:
                        out.append((kind, files, None))
                    return
            elif written is not None and not isinstance(written, tuple):
                # 这个单元自己用 printf / echo / cat <<词 写出来的脚本：代码就是这个单元里的那几行，cat <<词 的正文接着看它装了什么
                if unquoted not in state["expanded"]:
                    state["expanded"].add(unquoted)
                    for body in written:
                        inner = ShellFacts()
                        self.scan_text(body, inner)
                        work.extend((inner_kind, inner_word, inner_directory, inner_extra, line, None)
                                    for inner_kind, inner_word, inner_directory, inner_extra in inner.invocations)
                        facts.programs |= inner.programs
                return
            copied = self.tree_copy_targets(unquoted, unit)
            if copied:
                out.append((kind, sorted(copied), None))
                return
            if "shell-unfolded-paths-ignored" not in self.breaks:
                out.append((kind, [], Refusal("shell-path-unfolded", unit.file, line, self.row_snippet(shell_file, unit, line, word))))
            return
        files = []
        for value in sorted(values):
            # 前面 cd 过去的目录折出几个，相对路径就在每一个底下各找一次（运行期是其中哪一个都可能）
            tries = [{directory} for directory in sorted(directories)] if directories and not posixpath.isabs(value) else [directories]
            if "shell-cd-first-directory-only" in self.breaks:
                tries = [directories]
            for base in tries:
                status, file = self.target_of(value, unit, base)
                if status == "ok":
                    files.append(file)
                elif kind != "exec":
                    form = "shell-path-outside-root" if status == "outside" else "shell-path-missing"
                    out.append((kind, [], Refusal(form, unit.file, line, self.row_snippet(shell_file, unit, line, word))))
        if files:
            out.append((kind, files, None))

    def resolve_inline(self, state, kind, interpreter, directory, extra, generator_line):
        """内联脚本：-c 的串、喂给标准输入的 here-doc / here-string。正文折得出 → python 的交一份 InlineScript（闭包里切成单元、沿它的 import
        与 sys.argv 连下去），bash 的切开再认；折不出、或标准输入从哪来认不出 → 拒算 inline-script-not-literal。"""
        unit, shell_file, facts, out, work = state["unit"], state["shell_file"], state["facts"], state["out"], state["work"]
        language = extra[0] if kind == "inline-unknown" else ("python" if kind == "python-inline" else "shell")
        line = self.place(state, kind, interpreter, generator_line, language)
        snippet = self.row_snippet(shell_file, unit, line, interpreter)
        if kind == "inline-unknown":
            out.append((kind, [], Refusal(INLINE_FORM, unit.file, line, snippet)))
            return
        payload, rest, environment = extra
        texts = self.inline_texts(payload, unit)
        if texts is None:
            out.append((kind, [], Refusal(INLINE_FORM, unit.file, line, snippet)))
            return
        if language == "python":
            if "inline-argv-commands-ignored" not in self.breaks:
                # 正文把 sys.argv[k:] 当命令起：起它的这条命令里第 k 个参数起是一条命令，在这边照命令认
                for offset in sorted(set().union(*(argv_command_offsets(text) for text in texts))):
                    if offset < len(rest):
                        inner = ShellFacts()
                        inner.directory = directory
                        self.command_facts(list(rest[offset:]), inner)
                        work.extend((inner_kind, inner_word, inner_directory, inner_extra, generator_line, None)
                                    for inner_kind, inner_word, inner_directory, inner_extra in inner.invocations)
                        facts.programs |= inner.programs
            argv = tuple(self.argument_values(argument, unit) for argument in rest[1:])
            settings = tuple((name, self.argument_values(value, unit)) for name, value in environment)
            out.append(("python-inline", [InlineScript("python", tuple(sorted(texts)), argv, self.directory_bases(directory, unit), unit.file, line, snippet,
                                                       settings)], None))
            return
        for text in sorted(texts):
            inner = ShellFacts()
            self.scan_text(text if "shell-inline-positionals-ignored" in self.breaks else inline_positionals(text, rest), inner)
            work.extend((inner_kind, inner_word, inner_directory, inner_extra, generator_line, inner.assignments)
                        for inner_kind, inner_word, inner_directory, inner_extra in inner.invocations)
            facts.programs |= inner.programs

    def inline_texts(self, payload, unit):
        form, value = payload
        if form == "text":
            return {value}
        if form == "word":
            return self.fold_word(value, unit, frozenset())
        if form == "heredoc":
            return self.fold_heredoc(value, unit, frozenset())
        return None

    def fold_heredoc(self, body, unit, progress):
        """不带引号的 here-doc 正文：$名字、${…}、$( … )、`…` 逐处折（\\$ \\` \\\\ 与续行照 bash 的规矩），别的字原样；有一处折不出 → None。"""
        results, literal, i, total = {""}, [], 0, len(body)
        while i < total:
            character = body[i]
            if character == "\\" and i + 1 < total and body[i + 1] in "$`\\\n":
                if body[i + 1] != "\n":
                    literal.append(body[i + 1])
                i += 2
                continue
            if character in "$`":
                if body.startswith("$((", i):
                    return None
                if body.startswith("$(", i):
                    end = matching_close(body, i + 1, "(", ")") + 1
                elif body.startswith("${", i):
                    end = matching_close(body, i + 1, "{", "}") + 1
                elif character == "`":
                    close = body.find("`", i + 1)
                    if close < 0:
                        return None
                    end = close + 1
                else:
                    found = re.match(r"\$([A-Za-z_][A-Za-z0-9_]*|[0-9@*#?$!-])", body[i:])
                    if not found:
                        literal.append("$")
                        i += 1
                        continue
                    end = i + found.end()
                values = self.fold_word(body[i:end], unit, progress)
                if values is None:
                    return None
                prefix = "".join(literal)
                literal = []
                results = {result + prefix + value for result in results for value in values}
                if len(results) > FOLD_LIMIT:
                    return None
                i = end
                continue
            literal.append(character)
            i += 1
        prefix = "".join(literal)
        return {result + prefix for result in results}

    def argument_values(self, word, unit):
        """传给内联脚本的一个词 → 它的取值（frozenset）；折不出时「$目录/<仓里的相对路径>」按根里那一份认；都不行 → None。"""
        values = self.fold_word(word, unit, frozenset())
        if values is not None:
            return frozenset(values)
        if SHELL_POSITIONAL_WORD.match(word):
            return POSITIONAL
        unquoted = word.replace('"', "")
        bare = re.match(r"^\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?$", unquoted)
        if bare:
            # 只取位置参数（与字面值）的变量："$1" 一类，取值是调用处给的那几个词
            assigned = self.assignments_of(bare.group(1), unit)
            if assigned and any(SHELL_POSITIONAL_WORD.match(value) for value in assigned) and all(
                    SHELL_POSITIONAL_WORD.match(value) or literal_word(value) is not None for value in assigned):
                return POSITIONAL
        copied = self.tree_copy_targets(unquoted, unit)
        if copied:
            return frozenset(posixpath.join(self.root, file) for file in copied)
        found = SHELL_TREE_PREFIX.match(unquoted)
        directory = tree_copy_file(self.root, found.group(1), directory=True) if found and "tree-copy-tails-ignored" not in self.breaks else None
        return frozenset((posixpath.join(self.root, directory),)) if directory else None

    def directory_bases(self, directory, unit):
        """内联脚本的工作目录：前面 cd 过去的目录折得出就是它（几个都算），不然是根与这份文件所在的目录（相对路径与 import 依次试）。"""
        fallback = (self.root, posixpath.dirname(posixpath.join(self.root, unit.file)))
        if directory is None:
            return fallback
        folded = self.fold_word(directory, unit, frozenset())
        if not folded:
            return fallback
        return tuple(sorted(value if posixpath.isabs(value) else posixpath.normpath(posixpath.join(self.root, value)) for value in folded))

    def resolve_module_run(self, state, word, directory, extra):
        """python3 -m <模块>：仓里的模块连到它的程序入口（包连 __main__.py）；-m unittest / pytest 后面点名的测试模块整份连上
        （测试方法按名字反射着调，闭包认不出调的是哪几个）；标准库与外部的不装仓里的代码；名字折不出、找不到仓里那一份拒算。"""
        unit, shell_file, out = state["unit"], state["shell_file"], state["out"]
        interpreter, rest = extra
        line = self.place(state, "python-module", interpreter, None, "python")
        snippet = self.row_snippet(shell_file, unit, line, interpreter)
        names = self.fold_word(word, unit, frozenset())
        bases = self.directory_bases(directory, unit)
        if names is None:
            out.append(("python-module", [], Refusal("import-module-variable", unit.file, line, snippet)))
            return
        for name in sorted(names):
            if name not in TEST_RUNNER_MODULES:
                status, file = repository_module(self.root, bases, name, main=True)
                if status == "repo":
                    out.append(("python", [file], None))
                elif status == "unresolved":
                    out.append(("python-module", [], Refusal("import-unresolved", unit.file, line, snippet)))
                continue
            targets, position = [], 0
            while position < len(rest):
                value = literal_word(rest[position])
                if value is not None and value.startswith("-"):
                    position += 2 if value in ("-k", "-p", "-s", "-t", "-m", "--pattern", "--start-directory", "--top-level-directory") else 1
                    continue
                targets.append(rest[position])
                position += 1
            if not targets or literal_word(targets[0]) == "discover":
                out.append(("python-module", [], Refusal("import-unresolved", unit.file, line, snippet)))
                continue
            for target in targets:
                folded = self.fold_word(target, unit, frozenset())
                if folded is None:
                    out.append(("python-module", [], Refusal("import-module-variable", unit.file, line, snippet)))
                    continue
                for value in sorted(folded):
                    status, file = repository_module(self.root, bases, value)
                    if status == "repo":
                        out.append(("python-whole", [file], None))
                    elif status == "unresolved":
                        out.append(("python-module", [], Refusal("import-unresolved", unit.file, line, snippet)))

    def resolve_command(self, state, word, directory, extra, generator_line, overlay):
        """命令位置上折不出字面值的程序词：折得出 → 解释器照解释器认、仓里的脚本连上、PATH 上的程序名与函数名不装仓里的代码（函数按名字连）；
        "$@" "$1" "${@:2}" 一类：命令就是调用处那几个词（调用处已照命令认过）；折不出 → 拒算 shell-command-unfolded。"""
        unit, shell_file, out, work = state["unit"], state["shell_file"], state["out"], state["work"]
        rest, inputs = extra
        if SHELL_POSITIONAL_WORD.match(word):
            return
        array = re.match(r'^(?:\$\{[A-Za-z_][A-Za-z0-9_]*\[[@*]\]\+)?"?\$\{([A-Za-z_][A-Za-z0-9_]*)\[[@*]\]\}"?\}?$', word)
        if array and "shell-array-commands-unfolded" not in self.breaks:
            # "${命令[@]}"：数组的每一处字面赋值（x=( … )、x+=( … )）摊开当命令词，后面接这条命令余下的词。同一个数组在一个单元里只摊一次：
            # 再碰到它（x=("${x[@]}" y) 里套着自己）时它的每一处赋值已经认过
            seen = state.setdefault("arrays", collections.Counter())
            if seen[array.group(1)] >= 8 or (array.group(1), tuple(rest)) in seen:
                return
            seen[array.group(1)] += 1
            seen[(array.group(1), tuple(rest))] += 1
            assigned = self.assignments_of(array.group(1), unit)
            if assigned and all(value.startswith("(") and value.endswith(")") for value in assigned):
                for value in assigned:
                    inner = ShellFacts()
                    inner.directory = directory
                    self.command_facts([item for command in split_commands(value[1:-1]) for item in command] + list(rest), inner, inputs)
                    work.extend((inner_kind, inner_word, inner_directory, inner_extra, generator_line, overlay)
                                for inner_kind, inner_word, inner_directory, inner_extra in inner.invocations)
                return
        values = self.fold_word(word, unit, frozenset())
        if values is None:
            copied = self.tree_copy_targets(word.replace('"', ""), unit)
            if copied:
                out.append(("exec", sorted(copied), None))
                return
            unquoted = word.replace('"', "")
            prefix = re.match(r"^([A-Za-z_][A-Za-z0-9_]*)\$", unquoted)
            if prefix and "/" not in unquoted and "shell-unfolded-commands-allowed" not in self.breaks:
                # "前缀_$1" 一类：按名字前缀派给本闭包里看得见的函数（前缀相同的都连）
                out.append(("function-prefix", [prefix.group(1)], None))
                return
            outside = self.outside_program(unquoted, unit, state["facts"], 0)
            if outside:
                return
            line = self.place(state, "command", word, generator_line, "exec")
            out.append(("command", [], Refusal("shell-command-unfolded", unit.file, line, self.row_snippet(shell_file, unit, line, word))))
            return
        for value in sorted(values):
            base = posixpath.basename(value)
            if base in SHELL_INTERPRETERS or base in PYTHON_INTERPRETERS:
                inner = ShellFacts()
                inner.directory = directory
                self.command_facts([shlex.quote(value)] + list(rest), inner, inputs)
                work.extend((inner_kind, inner_word, inner_directory, inner_extra, generator_line, overlay)
                            for inner_kind, inner_word, inner_directory, inner_extra in inner.invocations)
            elif "/" in value or value.endswith((".sh", ".py", ".bash")):
                status, file = self.target_of(value, unit)
                if status == "ok":
                    out.append(("exec", [file], None))

    def function_output(self, inner, unit):
        """$(函数名)：看得见的那几个同名函数，整个函数体只有一条「printf '%s\\n' <字面的行…>」或「cat <<'词' … 词」的 → 它打的那几行
        （(带不带引号, 正文) 的集合，同 substitution_outputs）；有一个不是这种写法 → None。"""
        commands = split_commands(inner)
        if unit is None or len(commands) != 1 or len(commands[0]) != 1:
            return None
        name = literal_word(commands[0][0])
        if not name or not re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", name):
            return None
        paths = [unit.file] + sorted(self.downward([unit.file]) - {unit.file}) + sorted(self.upward(unit.file))
        functions = [function for path in paths if path in self.files for function in self.files[path].functions.get(name, [])]
        if not functions:
            return None
        outputs = set()
        for function in functions:
            commands = [command for command in self.facts(function).commands if command not in (["function", name], ["{"], ["}"])]
            if len(commands) != 1:
                return None
            command = commands[0]
            values = [literal_word(word) for word in command]
            if len(command) >= 2 and values[0] == "printf" and values[1] == "%s\\n" and None not in values:
                outputs.add((True, "\n".join(values[2:])))
                continue
            body = self.function_heredoc(function)
            if values == ["cat"] and body is not None:
                outputs.add(body)
                continue
            return None
        return outputs

    def function_heredoc(self, function):
        """函数体是一条「cat <<词 … 词」：→ (带不带引号, 正文)；不是 → None。"""
        rows = (function.body or "").split("\n")
        bodies = heredoc_bodies(rows)
        return next(iter(bodies.values())) if len(bodies) == 1 else None

    def outside_program(self, unquoted, unit, facts, depth):
        """命令位置上折不出的词是不是根外的程序：每一种取法要么折得出（不是解释器、不是仓里的脚本），要么是「$目录/<字面尾巴>」而尾巴
        不带脚本后缀、在根里也找不到（工具链、补好的 QEMU 这类根外的二进制）；变量的每一处赋值是这种写法，或只取位置参数（"$1"、"$@"：
        命令就是调用处那几个词）也算。尾巴是登记的外部程序名的，记进闭包点名的外部程序。"""
        if depth > 4 or "shell-unfolded-commands-allowed" in self.breaks:
            return False
        pieces = re.split(r"\$\{[^{}]*\}|\$[A-Za-z_][A-Za-z0-9_]*|\$[0-9@*#?$!-]", unquoted)
        if len(pieces) > 1 and pieces[0] == "" and pieces[-1].startswith("/") and len(pieces[-1]) > 1 and "`" not in unquoted and "$(" not in unquoted:
            tail = pieces[-1].lstrip("/")
            if tail.endswith((".sh", ".py", ".bash")) or tree_copy_file(self.root, tail) is not None:
                return False
            if os.path.exists(os.path.join(self.root, tail)):
                return False
            program = external_program_name(posixpath.basename(tail), self.breaks)
            if program is not None:
                facts.programs.add(program)
            return True
        bare = re.match(r"^\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?$", unquoted)
        if not bare:
            return False
        assigned = self.assignments_of(bare.group(1), unit)
        if not assigned:
            return False
        for value in assigned:
            if SHELL_POSITIONAL_WORD.match(value):
                continue
            folded = self.fold_word(value, unit, frozenset())
            if folded is not None:
                if any(posixpath.basename(item) in SHELL_INTERPRETERS | PYTHON_INTERPRETERS or item.endswith((".sh", ".py", ".bash"))
                       or self.target_of(item, unit)[0] == "ok" for item in folded):
                    return False
                for item in folded:
                    program = external_program_name(item, self.breaks)
                    if program is not None:
                        facts.programs.add(program)
                continue
            if not self.outside_program(value.replace('"', ""), unit, facts, depth + 1):
                return False
        return True

    def copied_sources(self, sources, unit):
        """cp 的源词 → 仓里的文件；有一个源折不出或不在根里 → None。"""
        files = set()
        for source in sources:
            values = self.fold_word(source, unit, frozenset())
            if values is None:
                return None
            for value in values:
                status, file = self.target_of(value, unit)
                if status != "ok":
                    return None
                files.add(file)
        return sorted(files)

    def tree_copy_targets(self, unquoted, unit, depth=0):
        """「$目录/<字面尾巴>」→ 根里同一相对路径那一份（tree_copy_file）；「$变量」→ 它每一处赋值各自折得出或各自是这种写法时的并。都不行 → None。"""
        if "tree-copy-tails-ignored" in self.breaks or depth > 4:
            return None
        found = SHELL_TREE_PREFIX.match(unquoted)
        if found:
            copied = tree_copy_file(self.root, found.group(1))
            return {copied} if copied is not None else None
        bare = re.match(r"^\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?$", unquoted)
        if not bare:
            return None
        words = self.assignments_of(bare.group(1), unit)
        if not words:
            return None
        files = set()
        for word in words:
            values = self.fold_word(word, unit, frozenset())
            if values is not None:
                for value in values:
                    status, file = self.target_of(value, unit)
                    if status != "ok":
                        return None
                    files.add(file)
                continue
            inner = self.tree_copy_targets(word.replace('"', ""), unit, depth + 1)
            if inner is None:
                return None
            files |= inner
        return files

    @staticmethod
    def word_line(shell_file, unit, word, kind=None, after=0):
        """拒算落在哪一行：先找这个词与装代码的那个命令词（source / . / bash / sh / python3 / 直接执行）同在的一行，
        找不到再退回第一处提到这个词的行（赋值、重定向、cp 这类只是提到路径的行排在后面）。
        同一个单元里同一个词装了几次，after 是上一次落的行：先往后找，各落各的行。"""
        if shell_file is None:
            return unit.first_line
        first = unit.first_line if unit.kind == "function" else 1
        last = unit.last_line if unit.kind == "function" else len(shell_file.rows)
        ranges = [(first, last), (first, len(shell_file.rows))]
        if after and first <= after < last:
            ranges.insert(0, (after + 1, last))
        needles = [word.strip('"')] + [f"${name}" for name in VARIABLE_REFERENCE.findall(word)] + [f"${{{name}" for name in VARIABLE_REFERENCE.findall(word)]
        tail = word.strip('"').rsplit("/", 1)[-1]
        if tail and "$" not in tail:
            needles.append(tail)
        loaders = {"source": r"(?:^|[\s;&|({])(?:source|\.)\s+", "shell": r"(?:^|[\s;&|({\"'=])(?:\S*/)?(?:bash|sh)\s",
                   "python": r"(?:^|[\s;&|({\"'=])(?:\S*/)?python3?\s", "exec": r""}
        loader = loaders.get(kind)
        tiers = [loader] if loader is not None else []
        for pattern in tiers + [None]:
            for low, high in ranges:
                for needle in needles:
                    for number in range(low, high + 1):
                        if not 0 < number <= len(shell_file.rows):
                            continue
                        row = shell_file.rows[number - 1]
                        if needle not in row or row.lstrip().startswith("#"):
                            continue
                        if pattern is None or (pattern == "" and row.lstrip().startswith((needle, f'"{needle}'))) or (
                                pattern and re.search(pattern + r".*" + re.escape(needle), row)):
                            return number
        return unit.first_line

    def functions_in(self, paths, name):
        out = []
        for member in sorted(paths):
            if member in self.files:
                out += self.files[member].functions.get(name, [])
        return out

    def function_names_in(self, paths):
        names = set()
        for member in paths:
            if member in self.files:
                names |= set(self.files[member].functions)
        return names


REPOSITORY_MODULE_NAMES = {}


def repository_module_names(root):
    """根里每份 .py 的模块名与每个包目录名（import 认不出时分「仓里该有却找不到」与「外部的」）。"""
    if root not in REPOSITORY_MODULE_NAMES:
        names = set()
        for directory, subdirectories, files in os.walk(root):
            subdirectories[:] = [name for name in subdirectories if name not in SKIPPED_DIRECTORIES]
            names |= {name[:-3] for name in files if name.endswith(".py")}
            if "__init__.py" in files:
                names.add(os.path.basename(directory))
        REPOSITORY_MODULE_NAMES[root] = names
    return REPOSITORY_MODULE_NAMES[root]


def repository_module(root, bases, name, main=False):
    """python3 -m 的模块名、unittest 点名的测试模块（点分的名字或 .py 路径）在 bases（工作目录）里 → ("repo", 相对根的文件) /
    ("stdlib" 或 "external", None) / ("unresolved", None)。main=True 时包取它的 __main__.py。"""
    if name.endswith(".py") or "/" in name:
        for base in bases:
            candidate = posixpath.normpath(name if posixpath.isabs(name) else posixpath.join(base, name))
            relative = inside_root(root, candidate)
            if relative is not None and os.path.isfile(candidate):
                return "repo", relative
        return "unresolved", None
    parts = [part for part in name.split(".") if part]
    if not parts or not all(re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", part) for part in parts):
        return "unresolved", None
    for base in bases:
        stem = posixpath.join(base, *parts)
        for option in ((stem + ".py", posixpath.join(stem, "__main__.py")) if main else (stem + ".py", posixpath.join(stem, "__init__.py"))):
            relative = inside_root(root, option)
            if relative is not None and os.path.isfile(option):
                return "repo", relative
    if parts[0] in sys.stdlib_module_names or parts[0] in sys.builtin_module_names:
        return "stdlib", None
    if parts[0] in repository_module_names(root):
        return "unresolved", None
    return "external", None


def shell_files_under(root, under=None):
    found = files_under(root, (".sh", ".bash"), under)
    return found


def shell_units(root, under=None):
    return ShellIndex(root, shell_files_under(root, under or list(DEFAULT_SCRIPT_DIRECTORIES)))


def language_of_file(root, path):
    if path.endswith(".py"):
        return "python"
    if path.endswith((".sh", ".bash")):
        return "shell"
    try:
        with open(os.path.join(root, path), "rb") as handle:
            first = handle.readline()
    except OSError:
        return None
    if first.startswith(b"#!") and (b"bash" in first or first.rstrip().endswith(b"/sh")):
        return "shell"
    if first.startswith(b"#!") and b"python" in first:
        return "python"
    return None


# ── python 与 shell 的闭包（两种语言互相连） ────────────────────────────────

def inline_shell_units(cache, script):
    """InlineScript（shell 的：python 起的 bash -c、shell=True、os.system）→ 每份正文一个内联 shell 单元（$0 $1… 折得出单值的换进正文）。"""
    if script not in cache:
        words = []
        for value in script.argv:
            single = next(iter(value)) if value is not None and len(value) == 1 else None
            words.append(single if single is not None and "'" not in single and '"' not in single else "'")
        # 起它时设的环境变量（单值的）写成正文前面的赋值，正文里读它们照本单元的赋值折
        settings = "".join(f"{name}={shlex.quote(next(iter(values)))}\n" for name, values in script.environment
                           if values is not None and len(values) == 1 and re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", name))
        units = []
        for position, text in enumerate(script.texts):
            body = settings + (text if "shell-inline-positionals-ignored" in active_breaks() else inline_positionals(text, words))
            unit = Unit("shell", "function", f"<inline@{script.line}#{position}>", script.file, script.line, script.line, sha256_text(body))
            unit.body, unit.scope, unit.qualified = body, "inline", script.snippet
            units.append(unit)
        cache[script] = units
    return cache[script]


def inline_refusals(root, python, shell, scripts, depth=0):
    """dump 用：几份内联脚本里收得到的拒算（python 的切成单元逐个看，shell 的看它的连边），连同它们里面再起的内联脚本。"""
    refusals, cache = set(), {}
    for script in scripts:
        if depth > 4:
            break
        if script.language == "python":
            for module in python.inline_modules(script):
                refusals.update(module.refusals)
                for unit in module.units:
                    facts = module.facts(unit)
                    refusals.update(facts.refusals)
                    refusals |= inline_refusals(root, python, shell, facts.inline_scripts, depth + 1)
            continue
        for unit in inline_shell_units(cache, script):
            nested = []
            for kind, files, refusal in shell.links(unit):
                if refusal is not None:
                    refusals.add(refusal)
                if kind == "python-inline":
                    nested += files
            refusals |= inline_refusals(root, python, shell, nested, depth + 1)
    return refusals


class ScriptClosure:
    def __init__(self, root, under):
        self.root = os.path.abspath(root)
        self.breaks = active_breaks()
        self.python = PythonIndex(self.root)
        self.under = list(under)
        self.shell = None
        self.closure = Closure()
        self.named_classes, self.attributes_seen, self.loaded_modules, self.modules_seen = set(), set(), set(), set()
        self.methods_by_name = collections.defaultdict(list)
        self.closure_sourcers, self.processed_shell = collections.defaultdict(set), collections.defaultdict(list)
        self.todo = []
        # 内联脚本切出来的单元：穿过它们连下去，自己不进闭包（正文已在起它的那个单元里）
        self.inline_seen, self.inline_units = set(), {}

    def shell_index(self, extra=()):
        if self.shell is None:
            self.shell = ShellIndex(self.root, shell_files_under(self.root, self.under) + list(extra))
        elif extra:
            self.shell.add(list(extra))
        return self.shell

    def push(self, unit, via, rule):
        if unit is not None and id(unit) not in self.closure.members:
            self.todo.append((unit, via, rule))

    def push_program(self, file, via):
        language = language_of_file(self.root, file)
        if language == "python":
            module = self.python.module(file)
            if module is not None:
                self.closure.refusals.update(module.refusals)
                for unit in module.program_entries():
                    self.push(unit, via, "program")
        elif language == "shell":
            index = self.shell_index([file])
            shell_file = index.files.get(file)
            if shell_file is not None:
                self.closure.refusals.update(shell_file.refusals)
                self.push(shell_file.script_body, via, "program")

    def run(self, entries):
        for entry in entries:
            self.push(entry, None, "entry")
        while self.todo:
            unit, via, rule = self.todo.pop()
            if unit.scope == "inline":
                if id(unit) in self.inline_seen:
                    continue
                self.inline_seen.add(id(unit))
            elif not self.closure.add(unit, via, rule):
                continue
            if unit.data:
                continue
            if unit.language == "python":
                self.python_edges(unit)
            elif unit.language == "shell":
                self.shell_edges(unit)
        return self.closure

    def push_inline(self, script, via):
        """一份内联脚本：python 的切成单元，从它的程序入口穿过去；bash 的当一个内联 shell 单元（sys.argv / $1… 已折进去）。"""
        if script.language == "python":
            for module in self.python.inline_modules(script):
                self.closure.refusals.update(module.refusals)
                for unit in module.program_entries():
                    self.push(unit, via, "inline")
            return
        for unit in inline_shell_units(self.inline_units, script):
            self.push(unit, via, "inline")

    def push_whole(self, file, via):
        """整份连上的模块（-m unittest 点名的测试模块：测试方法按名字反射着调）：它的每个单元。"""
        module = self.python.module(file)
        if module is None:
            return
        self.closure.refusals.update(module.refusals)
        for unit in module.units:
            if not unit.data:
                self.push(unit, via, "whole-module")

    # python

    def module_entered(self, module):
        self.modules_seen.add(module.path)
        self.closure.refusals.update(module.refusals)
        for method in module.methods:
            self.methods_by_name[method.short].append(method)
            if id(method.container) in self.named_classes and (method.short in self.attributes_seen or "python-attributes-unnarrowed" in self.breaks):
                self.push(method, method.container, f"method {method.short}")

    def class_entered(self, head):
        self.named_classes.add(id(head))
        for method in head.module.methods:
            if method.container is head and (method.short in self.attributes_seen or (method.short.startswith("__") and method.short.endswith("__"))):
                self.push(method, head, f"method {method.short}")

    def attribute_seen(self, name, via):
        if name in self.attributes_seen:
            return
        self.attributes_seen.add(name)
        unnarrowed = "python-attributes-unnarrowed" in self.breaks
        for method in self.methods_by_name.get(name, ()):
            if unnarrowed or id(method.container) in self.named_classes:
                self.push(method, via, f"method {name}")
        for path in sorted(self.loaded_modules):
            module = self.python.module(path)
            if module is not None:
                for unit in module.by_name.get(name, ()):
                    self.push(unit, via, f"loaded {name}")

    def python_edges(self, unit):
        module = unit.module
        if module.path not in self.modules_seen:
            self.module_entered(module)
        facts = module.facts(unit)
        self.closure.refusals.update(facts.refusals)
        if unit.container is not None:
            self.push(unit.container, unit, "container")
        if unit.kind != "module-body":
            self.push(module.module_body, unit, "module-body")
        if unit.kind == "class":
            self.class_entered(unit)
        for target in facts.import_targets:
            self.push(target, unit, "import")
        for name in sorted(facts.names):
            for target in module.resolve_name(name, unit, facts):
                self.push(target, unit, f"name {name}")
        self.closure.refusals.update(facts.refusals)
        for base, attribute in facts.attributes:
            targets = module.resolve_attribute(base, attribute, unit, facts)
            if targets is not None:
                for target in targets:
                    self.push(target, unit, f"attribute {base}.{attribute}")
            else:
                self.attribute_seen(attribute, unit)
        if "python-attributes-unnarrowed" in self.breaks:
            for base, attribute in facts.attributes:
                for method in self.methods_by_name.get(attribute, ()):
                    self.push(method, unit, f"method {attribute}")
        for file in facts.loads:
            loaded = self.python.module(file)
            if loaded is None:
                continue
            if file not in self.loaded_modules:
                self.loaded_modules.add(file)
                for name in sorted(self.attributes_seen):
                    for target in loaded.by_name.get(name, ()):
                        self.push(target, unit, f"loaded {name}")
            self.push(loaded.module_body, unit, "load")
        for file in facts.program_links:
            self.push_program(file, unit)
        for script in facts.inline_scripts:
            self.push_inline(script, unit)
        for file in facts.whole_modules:
            self.push_whole(file, unit)
        if unit.kind == "main-guard" and module.inline is None and "declared-checks-ignored" not in self.breaks:
            # 程序入口调 preflight：它先在仓根跑这份文件头登记的 check 判据
            for script in declared_checks(self.root, module.path, module.rows):
                self.push_inline(script, unit)
        if module.inline is not None:
            # 内联脚本没有文档串可读、也没有自己的原文文件：读 __doc__ / __file__ 的那几样不收
            for text in facts.strings:
                program = external_program_name(text, self.breaks)
                if program is not None:
                    self.closure.programs.setdefault(program, (unit.file, unit.first_line))
            return
        if "python-docstring-readers-ignored" not in self.breaks:
            for owner in facts.doc_reads:
                if owner in module.docstrings or owner in ("<module>", "*"):
                    for target in module.docstring_units(owner):
                        self.closure.add(target, unit, "docstring")
                    continue
                other = module.module_of_binding(module.binding(owner.split(".")[0], unit))
                for target in (other.docstring_units("<module>") if other is not None else module.docstring_units("*")):
                    self.closure.add(target, unit, "docstring")
        if facts.source_read:
            self.closure.add(module.source_unit(), unit, "source-text")
        for text in facts.strings:
            program = external_program_name(text, self.breaks)
            if program is not None:
                self.closure.programs.setdefault(program, (unit.file, unit.first_line))

    # shell

    def visible_files(self, path):
        """一个单元能叫到哪几份文件里的函数与变量：它那份文件、这个闭包里（传递地）source 了它的那几份，连同这些（传递地）source 进来的。
        同一份库被别的脚本也 source 了，那些脚本的函数不算可见（它们不在这一次的调用链上）。"""
        roots, todo = {path}, [path]
        while todo:
            item = todo.pop()
            for sourcer in self.closure_sourcers.get(item, ()):
                if sourcer not in roots:
                    roots.add(sourcer)
                    todo.append(sourcer)
        return self.shell.downward(roots)

    def shell_names(self, unit, facts, visible):
        index = self.shell
        shell_file = index.files[unit.file]
        names = facts.command_words if "shell-names-command-position-only" in self.breaks else facts.words
        for name in sorted(names & index.function_names_in(visible)):
            for target in index.functions_in(visible, name):
                if target is not unit:
                    self.push(target, unit, f"name {name}")
        for name in sorted(facts.variable_references):
            targets = shell_file.variables.get(name) or [variable for path in sorted(visible) if path in index.files
                                                          for variable in index.files[path].variables.get(name, [])]
            for target in targets:
                self.push(target, unit, f"variable {name}")

    def shell_edges(self, unit):
        index = self.shell_index()
        inline = unit.scope == "inline"
        shell_file = index.files.get(unit.file) if not inline else None
        if shell_file is not None:
            self.closure.refusals.update(shell_file.refusals)
        facts = index.facts(unit)
        if not inline:
            self.processed_shell[unit.file].append(unit)
        if unit.container is not None:
            self.push(unit.container, unit, "container")
        if shell_file is not None and shell_file.kept is not None:
            self.closure.add(shell_file.kept, unit, "kept-comments")
        if not inline:
            self.shell_names(unit, facts, self.visible_files(unit.file))
        if unit.kind == "script-body" and shell_file is not None and "declared-checks-ignored" not in self.breaks:
            # 脚本体调 preflight：它先在仓根跑这份文件头登记的 check 判据
            for script in declared_checks(self.root, unit.file, shell_file.rows):
                self.push_inline(script, unit)
        sourced, prefixes = set(), []
        for kind, files, refusal in index.links(unit):
            if refusal is not None:
                self.closure.refusals.add(refusal)
            for file in files:
                if kind == "function-prefix":
                    prefixes.append(file)
                elif kind == "python-inline":
                    self.push_inline(file, unit)
                elif kind == "python-whole":
                    self.push_whole(file, unit)
                elif kind == "source":
                    added = index.files.get(file)
                    if added is None:
                        index.add([file])
                        added = index.files.get(file)
                    if added is None:
                        continue
                    self.closure.refusals.update(added.refusals)
                    self.push(added.script_body, unit, "source")
                    sourced.add(file)
                    if not inline and unit.file not in self.closure_sourcers[file]:
                        self.closure_sourcers[file].add(unit.file)
                        for path in sorted(index.downward([file])):
                            for processed in list(self.processed_shell.get(path, ())):
                                self.shell_names(processed, index.facts(processed), self.visible_files(path))
                else:
                    self.push_program(file, unit)
        if prefixes:
            # "前缀_$1" 派活：看得见的函数里名字以这个前缀起头的都连
            visible = index.downward(sourced) if inline else self.visible_files(unit.file)
            for prefix in sorted(set(prefixes)):
                for name in sorted(name for name in index.function_names_in(visible) if name.startswith(prefix)):
                    for target in index.functions_in(visible, name):
                        if target is not unit:
                            self.push(target, unit, f"prefix {prefix}")
        if inline:
            # 内联 bash（python 起的 bash -c）：叫得到的函数与变量只在它 source 进来的那几份里
            visible = index.downward(sourced)
            for name in sorted(facts.words & index.function_names_in(visible)):
                for target in index.functions_in(visible, name):
                    self.push(target, unit, f"name {name}")
            for name in sorted(facts.variable_references):
                for path in sorted(visible):
                    for target in index.files[path].variables.get(name, []) if path in index.files else []:
                        self.push(target, unit, f"variable {name}")
        for program in facts.programs:
            self.closure.programs.setdefault(program, (unit.file, unit.first_line))


# ── 入口、键与输出 ───────────────────────────────────────────────────────────

ENTRY_PATTERN = re.compile(r"^(.+?\.(?:rs|wgsl|py|sh|bash))(?:::(.+))?$")


def parse_entry(root, entry):
    """→ (语言, 文件或 None, 限定名或 None)。"""
    found = ENTRY_PATTERN.match(entry)
    if found:
        path, qualified = found.group(1), found.group(2)
        suffix = path.rsplit(".", 1)[1]
        language = {"rs": "rust", "wgsl": "wgsl", "py": "python", "sh": "shell", "bash": "shell"}[suffix]
        return language, path, qualified
    path, _separator, qualified = entry.partition("::")
    if "/" in path and os.path.isfile(os.path.join(root, path)):
        language = language_of_file(root, path)
        if language is None:
            raise UsageError(f"认不出 {path} 是哪种语言")
        return language, path, qualified or None
    if "::" in entry and "/" not in entry:
        return "rust", None, entry
    raise UsageError(f"认不出的入口：{entry}（写 <文件>::<限定名>、<文件> 或 Rust 的 <crate>::<模块…>::<名>）")


class Options:
    def __init__(self):
        self.language, self.under, self.target = None, [], None
        self.cfg, self.no_default_cfg, self.features, self.no_default_features = [], False, [], False
        self.toolchain, self.dependencies, self.externals = [], [], {}
        self.wgsl_modules, self.rust_strings_entry, self.explain = [], None, False


def closure_of(root, entry, options=None):
    options = options or Options()
    if not os.path.isdir(root):
        raise FunctionUnitsError(f"根不在：{root}")
    language, path, qualified = parse_entry(root, entry)
    if language == "rust":
        workspace = RustWorkspace(root)
        unit, target_id = workspace.find_entry(path, qualified, options.target)
        return rust_closure_of(workspace, unit, target_id, options.cfg, options.no_default_cfg, options.features, options.no_default_features)
    if language == "wgsl":
        files = [wgsl_file(root, item) for item in [path] + list(options.wgsl_modules)]
        entries = [unit for unit in files[0].units if unit.name == qualified] if qualified else None
        if qualified and not entries:
            raise UsageError(f"{path} 里没有 {qualified}")
        sources, workspace, rust = None, None, None
        closure = Closure()
        if options.rust_strings_entry:
            rust_language, rust_path, rust_qualified = parse_entry(root, options.rust_strings_entry)
            workspace = RustWorkspace(root)
            if rust_language == "rust":
                rust_unit, rust_target = workspace.find_entry(rust_path, rust_qualified, options.target)
                rust = rust_closure_of(workspace, rust_unit, rust_target, options.cfg, options.no_default_cfg, options.features, options.no_default_features)
            else:
                rust = closure_of(root, options.rust_strings_entry, options)
                for unit in rust.order:
                    if unit.language == "rust" and unit.first_token is not None:
                        workspace.file(unit.file)
            sources = WgslNameSources(workspace, rust.order)
        members, refusals, suppliers = wgsl_closure(files, entries, sources)
        for member in members:
            closure.add(member, None, "wgsl")
        closure.refusals.update(refusals)
        if suppliers and "wgsl-external-name-sources-unlinked" not in active_breaks():
            # 着落做成边：给外名着落的 Rust 单元连同它自己的闭包并进来（改了拼名字的那个 Rust 单元或它调到的，WGSL 入口的键就变）
            for supplier in sorted(suppliers, key=Unit.sort_key):
                target_id = rust.unit_targets.get(id(supplier))
                if target_id is None:
                    continue
                supplied = rust_closure_of(workspace, supplier, target_id, options.cfg, options.no_default_cfg, options.features, options.no_default_features)
                for member in supplied.order:
                    closure.add(member, supplier, "wgsl-name-source")
                closure.refusals.update(supplied.refusals)
                closure.data_sites |= supplied.data_sites
                closure.cfg_atoms = (closure.cfg_atoms or set()) | supplied.cfg_atoms
        return closure
    under = options.under or [place for place in DEFAULT_SCRIPT_DIRECTORIES if os.path.isdir(os.path.join(root, place))]
    under = list(dict.fromkeys(list(under) + [posixpath.dirname(path) or "."]))
    script = ScriptClosure(root, under)
    if language == "python":
        module = script.python.module(path)
        if module is None:
            raise UsageError(f"入口文件不在：{path}")
        if qualified:
            entries = [unit for unit in module.units if unit.name == qualified and not unit.data]
            if len(entries) != 1:
                raise UsageError(f"{path} 里限定名 {qualified} 命中 {len(entries)} 个单元（要恰好一个；dump 看限定名）")
        else:
            entries = module.program_entries()[:1]
        script.closure.refusals.update(module.refusals)
        return script.run(entries)
    index = script.shell_index([path])
    shell_file = index.files.get(path)
    if shell_file is None:
        raise UsageError(f"入口文件不在：{path}")
    script.closure.refusals.update(shell_file.refusals)
    if shell_file.script_body is None:
        return script.closure
    if qualified:
        entries = [unit for unit in shell_file.units if unit.name == qualified and not unit.data]
        if len(entries) != 1:
            raise UsageError(f"{path} 里限定名 {qualified} 命中 {len(entries)} 个单元（要恰好一个；dump 看限定名）")
    else:
        entries = [shell_file.script_body]
    return script.run(entries)


def closure_key(closure, toolchain=(), dependencies=(), externals=None, allow_refused=False):
    """→ (键或 None, 拒算列表)。拒算不空时键是 None（allow_refused 只给对照用：照算出键，拒算照样交回）。"""
    externals = dict(externals or {})
    refusals = set(closure.refusals)
    if "external-programs-unchecked" not in active_breaks():
        for program, (file, line) in sorted(closure.programs.items()):
            if program not in externals:
                refusals.add(Refusal("external-program-unkeyed", file, line, program))
    ordered = sorted(refusals, key=lambda refusal: (refusal.file, refusal.line, refusal.form, refusal.snippet))
    if ordered and not allow_refused:
        return None, ordered
    lines = [f"form\t{KEY_FORM}"]
    lines += sorted(f"cfg\t{item}" for item in key_cfg_items(closure, closure.cfg_lines))
    lines += sorted(f"toolchain\t{item}" for item in toolchain)
    lines += sorted(f"dependency\t{item}" for item in dependencies)
    lines += sorted(f"external\t{name}\t{sha}\t{version}" for name, (sha, version) in externals.items())
    lines += sorted(f"unit\t{unit.kind}\t{unit.fingerprint}" for unit in closure.order if not unit.data)
    lines += sorted(f"data\t{unit.kind}\t{unit.fingerprint}" for unit in closure.order if unit.data)
    return sha256_text("".join(line + "\n" for line in lines)), ordered


def key_cfg_items(closure, unnarrowed):
    """键里的 cfg 那几项（定案 2）：闭包记了真引到的原子（cfg_atoms，「<原子>\\t<true|false>」）就只取它们；没记的（别的脚本自己拼的闭包、
    并起来的闭包没并 cfg_atoms）退回 unnarrowed（整套生效 cfg：只多不少，键照样对，只是带不带特性的两趟分开存）。
    弄坏开关 rust-cfg-atoms-unnarrowed 下一律退回 unnarrowed。"""
    narrowed = getattr(closure, "cfg_atoms", None)
    if narrowed is None or "rust-cfg-atoms-unnarrowed" in active_breaks():
        return list(unnarrowed)
    return list(narrowed)


def unit_graph_key(closure, toolchain=(), lock_lines=()):
    """照 crates/singlefs-unit-graph 的 reuse_key 算 Rust 闭包的键（两份实现逐入口对拍用；python 这一份自己的复用键仍是 closure_key）→ (键或 None, 拒算)。
    行以 \\n 连、末尾不带：entry-target-kind\\t<目标种类>、cfg\\t<原子>\\t<true|false>（闭包里各单元真引到的 cfg 原子与它在收那个单元的目标下的真值，
    排好序；key_cfg_items）、unit\\t<种类>\\t<指纹>（闭包里每个 Rust 与 WGSL 单元，
    排序）、data\\t<摘要>（每一处读数据文件，排序）、toolchain\\t<行>、lock\\t<行>（调用方给的次序）。闭包里有拒算不交键。"""
    ordered = sorted(closure.refusals, key=lambda refusal: (refusal.file, refusal.line, refusal.form, refusal.snippet))
    if ordered:
        return None, ordered
    lines = [f"entry-target-kind\t{closure.entry_target_kind}"]
    lines += [f"cfg\t{atom}" for atom in sorted(key_cfg_items(closure, closure.entry_cfg_atoms))]
    lines += sorted(f"unit\t{unit.kind}\t{unit.fingerprint}" for unit in closure.order if not unit.data)
    lines += sorted(f"data\t{site[3]}" for site in closure.data_sites)
    lines += [f"toolchain\t{line}" for line in toolchain]
    lines += [f"lock\t{line}" for line in lock_lines]
    return sha256_text("\n".join(lines)), []


def dump_lines(root, language=None, under=None):
    """→ (单元行, 拒算行, 各语言单元数)。"""
    units, refusals, counts = [], set(), collections.Counter()
    if language in (None, "rust"):
        workspace = RustWorkspace(root)
        names, pairs = cfg_entries(list(DEFAULT_RUST_CFG))
        known = set(KNOWN_CFG_NAMES)
        for crate in workspace.crates:
            known |= crate.check_cfg
            if crate.proc_macro:
                refusals.add(Refusal("proc-macro-crate", posixpath.join(crate.directory, "Cargo.toml"), 1, f"proc-macro = true（{crate.package}）"))
        for target in workspace.targets.values():
            refusals.update(target.refusals)
        data_digests = {}
        for rust_file in workspace.all_files():
            refusals.update(rust_file.refusals)
            for unit in rust_file.units:
                units.append(unit)
                counts["rust"] += 1
                refusals.update(unit.refusals)
                if "rust-data-dump-unlisted" not in active_breaks():
                    # 数据单元照 crates/singlefs-unit-graph 的 dump：每一处读（include_str! 一类与运行期拼得出字面路径的读，.wgsl 不算）一行
                    # data\t<数据文件>\t<读它的文件>:<行>-<行>\t<字节 sha256>
                    for kind, value, index in unit.data_references or ():
                        if kind != "file":
                            continue
                        if value not in data_digests:
                            data_digests[value] = data_file_unit(root, value).fingerprint
                        line = rust_file.lines[index]
                        site = Unit("data", "data", value, unit.file, line, line, data_digests[value], data=True)
                        units.append(site)
                        counts["data"] += 1
                if unit.kind == "extern_block":
                    refusals.add(Refusal("extern-block", unit.file, unit.first_line, rust_file.row(unit.first_line)))
                for tree, raw, line in unit.conditions or ():
                    if tree is None or not cfg_tree_known(tree, known):
                        refusals.add(Refusal("cfg-unrecognized", unit.file, line, snippet_of(f"cfg({raw})")))
    if language in (None, "wgsl"):
        for item in wgsl_units(root):
            refusals.update(item.refusals)
            units += item.units
            counts["wgsl"] += len(item.units)
    if language in (None, "python"):
        python_index, modules = python_units(root, under)
        empty_shell = ShellIndex(root, [])
        for module in modules:
            refusals.update(module.refusals)
            refusals |= inline_refusals(root, python_index, empty_shell, declared_checks(root, module.path, module.rows))
            for unit in module.units:
                units.append(unit)
                counts["python"] += 1
                facts = module.facts(unit)
                refusals.update(facts.refusals)
                refusals |= inline_refusals(root, python_index, empty_shell, facts.inline_scripts)
    if language in (None, "shell"):
        index = shell_units(root, under)
        inline_python = PythonIndex(root)
        for path in sorted(index.files):
            shell_file = index.files[path]
            refusals.update(shell_file.refusals)
            refusals |= inline_refusals(root, inline_python, index, declared_checks(root, path, shell_file.rows))
            for unit in shell_file.units:
                units.append(unit)
                counts["shell"] += 1
                if not unit.data:
                    for kind, files, refusal in index.links(unit):
                        if refusal is not None:
                            refusals.add(refusal)
                        if kind == "python-inline":
                            refusals |= inline_refusals(root, inline_python, index, files)
    unit_lines = [unit.dump_text() for unit in sorted(units, key=dump_sort_key)]
    refusal_lines = [refusal.text() for refusal in sorted(refusals, key=lambda refusal: (refusal.file, refusal.line, refusal.form, refusal.snippet))]
    return unit_lines, refusal_lines, counts


def dump_sort_key(unit):
    """dump 的次序：Rust、WGSL 与数据单元照 crates/singlefs-unit-graph 的 sorted_texts（文件、起行、止行、整行）；python 与 shell 照 Unit.sort_key。
    同一份文件里只有一种语言，两种排法不会比到一起。"""
    if unit.language in ("rust", "wgsl", "data"):
        return (unit.file, unit.first_line, unit.last_line, unit.dump_text(), "")
    return (unit.file, unit.first_line, unit.kind, unit.name, "")


def cfg_tree_known(tree, known):
    if tree[0] in ("all", "any", "not"):
        return all(cfg_tree_known(item, known) for item in tree[1])
    return tree[1] in known


def explain_lines(closure):
    lines = []
    for unit in closure.units():
        via, rule = closure.via.get(id(unit), (None, ""))
        source = f"{via.file}:{via.name}" if via is not None else "-"
        lines.append(f"via\t{unit.file}:{unit.name}\t{source}\t{rule}")
    return lines


# ── V5 / V11 对照（design 报告 2.2 节） ──────────────────────────────────────

CONTROL_ENTRIES = (
    ("v5_test", "crates/singlefs-harness/tests/new_pool_file_creation_publish.rs::root_slots_system_configurations_and_journal_ring_hold_the_published_state"),
    ("v11_test", "crates/singlefs-format/src/lib.rs::tests::pointer_record_and_entry_width_literals_equal_the_field_sums_they_stand_for"),
    ("cpu_judge", "crates/singlefs-checker-tier/src/crash.rs::judge_crash_image"),
)
CONTROL_VARIANTS = (
    ("V5", "const JOURNAL_PAYLOAD_CHECKSUM_OFFSET: usize = 95;", {"v5_test": "changed", "v11_test": "same", "cpu_judge": "changed"}),
    ("V11", "pub const DATA_POINTER_BYTES: u64 = 88;", {"v5_test": "changed", "v11_test": "changed", "cpu_judge": "changed"}),
)


def mutation_row(root, original):
    with open(os.path.join(root, "crates", "mutations.tsv"), encoding="utf-8") as handle:
        for number, row in enumerate(handle, 1):
            fields = row.rstrip("\n").split("\t")
            if len(fields) >= 4 and fields[2].replace("\\n", "\n") == original:
                return number, fields[1], fields[2].replace("\\n", "\n"), fields[3].replace("\\n", "\n")
    raise FunctionUnitsError(f"crates/mutations.tsv 里找不到原文是「{original}」的一行")


def control_keys(copy_root):
    workspace = RustWorkspace(copy_root)
    out = {}
    for label, entry in CONTROL_ENTRIES:
        _language, path, qualified = parse_entry(copy_root, entry)
        unit, target_id = workspace.find_entry(path, qualified, None)
        closure = rust_closure_of(workspace, unit, target_id)
        key, refusals = closure_key(closure, allow_refused=True)
        out[label] = (key, len(closure.order), len(refusals))
    return out


def run_controls(root):
    breaks = active_breaks()
    with tempfile.TemporaryDirectory(prefix="function-units-controls-") as work:
        copy_root = os.path.join(work, "copy")
        os.makedirs(copy_root)
        shutil.copytree(os.path.join(root, "crates"), os.path.join(copy_root, "crates"), ignore=shutil.ignore_patterns("target"))
        for name in ("Cargo.toml", "Cargo.lock"):
            if os.path.isfile(os.path.join(root, name)):
                shutil.copy2(os.path.join(root, name), os.path.join(copy_root, name))
        started = time.monotonic()
        baseline = control_keys(copy_root)
        for label, (key, members, refused) in baseline.items():
            print(f"base\t{label}\tkey={key[:16]}\tunits={members}\trefused={refused}")
        failures = 0
        for variant, original, expected in CONTROL_VARIANTS:
            number, file, before, after = mutation_row(root, original)
            path = os.path.join(copy_root, file)
            with open(path, encoding="utf-8") as handle:
                text = handle.read()
            if text.count(before) != 1:
                raise FunctionUnitsError(f"{file} 里「{before}」命中 {text.count(before)} 次（要恰好一次）")
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(text.replace(before, after))
            now = control_keys(copy_root)
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(text)
            for label, _entry in CONTROL_ENTRIES:
                got = "changed" if now[label][0] != baseline[label][0] else "same"
                want = "same" if "rust-constants-dropped" in breaks else expected[label]
                mark = "✓" if got == want else "✗"
                failures += got != want
                print(f"  {mark} {variant}（crates/mutations.tsv:{number}）\t{label}\t{got}\t期望 {want}")  # gate-lint:detail
        elapsed = time.monotonic() - started
    if failures:
        print(f"  ✗ 对照 {failures} 格与 design 报告 2.2 节的表不符（共 {len(CONTROL_VARIANTS) * len(CONTROL_ENTRIES)} 格，{elapsed:.1f} 秒）")
        print("    → 怎么办：看不符那一格的入口闭包（closure <根> <入口> --explain）里有没有那个常量，补第三节规则里漏的那一条")
        return 1
    print(f"  ✓ 对照 {len(CONTROL_VARIANTS) * len(CONTROL_ENTRIES)} 格与 design 报告 2.2 节的表一致（{elapsed:.1f} 秒）")
    return 0


# ── 自证 ────────────────────────────────────────────────────────────────────

RUST_FIXTURE = {
    "crates/fcore/Cargo.toml": '[package]\nname = "fcore"\n',
    "crates/fcore/src/lib.rs": '''pub const WIDTH: usize = 4;
pub trait Device {
    fn read(&self) -> u32;
    fn doubled(&self) -> u32 {
        self.read() * 2
    }
}
pub fn recover(device: &dyn Device) -> u32 {
    device.read() + 1
}
pub fn recover_generic<D: Device>(device: &D) -> u32 {
    device.doubled()
}
pub struct Money(pub u32);
impl std::ops::Add for Money {
    type Output = Money;
    fn add(self, other: Money) -> Money {
        Money(self.0 + other.0)
    }
}
pub struct Guard;
impl Drop for Guard {
    fn drop(&mut self) {
        let _ = 1;
    }
}
macro_rules! make_constant_function {
    ($name:ident) => {
        pub fn $name() -> u32 {
            5
        }
    };
}
make_constant_function!(five);
pub trait Limit {
    const LIMIT: u32;
}
pub fn limit_of<T: Limit>() -> u32 {
    T::LIMIT
}
pub fn helper() -> u32 {
    3
}
pub static TABLE: [fn() -> u32; 1] = [helper];
pub fn unrelated() -> u32 {
    99
}
pub struct Buffer {
    pub bytes: [u8; WIDTH],
}
''',
    "crates/fharness/Cargo.toml": '[package]\nname = "fharness"\n[dependencies]\nfcore = { path = "../fcore" }\n',
    "crates/fharness/src/lib.rs": '''use fcore::{Device, Limit};
pub struct Memory;
impl Device for Memory {
    fn read(&self) -> u32 {
        7
    }
}
pub struct Small;
impl Limit for Small {
    const LIMIT: u32 = 10;
}
pub struct Other;
impl Device for Other {
    fn read(&self) -> u32 {
        8
    }
}
''',
    "crates/ftier/Cargo.toml": '[package]\nname = "ftier"\n[dependencies]\nfcore = { path = "../fcore" }\nfharness = { path = "../fharness" }\nfextra = { path = "../fextra" }\n',
    "crates/ftier/src/lib.rs": '''use fcore::{recover, recover_generic, Device, Money};
use fharness::{Memory, Small};
pub fn judge_through_dyn() -> u32 {
    recover(&Memory)
}
pub fn judge_through_generic() -> u32 {
    recover_generic(&Memory)
}
pub fn judge_operator() -> u32 {
    (Money(1) + Money(2)).0
}
pub fn judge_drop() -> u32 {
    let _guard = fcore::Guard;
    0
}
pub fn judge_macro() -> u32 {
    fcore::five()
}
pub fn judge_associated_constant() -> u32 {
    fcore::limit_of::<Small>()
}
pub fn judge_table() -> u32 {
    (fcore::TABLE[0])()
}
pub fn judge_open(device: &dyn Device) -> u32 {
    device.read()
}
pub fn judge_array_length() -> usize {
    std::mem::size_of::<fcore::Buffer>()
}
pub fn judge_listing() -> u32 {
    fextra::shade_count()
}
pub fn judge_speed() -> u32 {
    fextra::speed()
}
pub fn judge_proven() -> u32 {
    fextra::proven()
}
pub fn judge_documented() -> u32 {
    fextra::documented()
}
pub fn judge_table_rows() -> usize {
    fextra::table_rows()
}
pub fn judge_kernel() -> String {
    fextra::kernel_source()
}
pub fn judge_greeting() -> String {
    fextra::greet()
}
''',
    "crates/fextra/Cargo.toml": '[package]\nname = "fextra"\n[features]\ndefault = ["fast"]\nfast = []\nslow = []\n',
    "crates/fextra/src/lib.rs": '''#[macro_export]
macro_rules! listing {
    ($($item:item)*) => { $($item)* };
}
crate::listing! {
    pub enum Shade {
        Light,
        Dark,
    }
}
pub fn shade_count() -> u32 {
    Shade::Dark as u32 + 1
}
#[cfg(feature = "fast")]
pub fn speed() -> u32 {
    11
}
#[cfg(feature = "slow")]
pub fn speed() -> u32 {
    22
}
verus! {
pub fn bounded(x: u32) -> (r: u32)
    ensures
        if x > 0 { r > 0 } else { r == 0 },
{
    x
}
pub fn proven() -> u32 {
    9
}
} // verus!
/// 文档说明。
///
/// ```
/// assert_eq!(fextra::documented(), 4);
/// ```
pub fn documented() -> u32 {
    4
}
pub const TABLE_TEXT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/table.txt"));
pub fn table_rows() -> usize {
    TABLE_TEXT.lines().count()
}
pub const KERNEL_COMMON: &str = include_str!("common.wgsl");
pub const KERNEL_MAIN: &str = include_str!("main.wgsl");
pub fn prelude() -> String {
    format!("const GENERATED_{}: u32 = 1u;", "LIMIT")
}
pub fn kernel_source() -> String {
    format!("{}\\n{KERNEL_COMMON}\\n{KERNEL_MAIN}", prelude())
}
const GREETING: &str = "hi";
pub fn greet() -> String {
    format!("{GREETING}!")
}
''',
    "crates/fextra/data/table.txt": "row\nrow\n",
    "crates/fextra/src/common.wgsl": '''// 共用的那一半
fn helper_used(x: u32) -> u32 {
    return x + GENERATED_LIMIT;
}
fn helper_unused(x: u32) -> u32 {
    return x * 7u;
}
''',
    "crates/fextra/src/main.wgsl": '''@group(0) @binding(0) var<storage, read_write> output: array<u32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    output[id.x] = helper_used(id.x);
}
''',
}
RUST_FIXTURE_ENTRY_FILE = "crates/ftier/src/lib.rs"
RUST_FIXTURE_CASES = (
    ("dyn 分派到下游 crate 的 impl", "judge_through_dyn", "crates/fharness/src/lib.rs", "    fn read(&self) -> u32 {\n        7", "    fn read(&self) -> u32 {\n        70", "changed"),
    ("dyn 分派：闭包里的 dyn 不连没被点名的另一个 impl，改它不作废", "judge_through_dyn", "crates/fharness/src/lib.rs", "        8\n", "        80\n", "same"),
    ("泛型约束分派", "judge_through_generic", "crates/fharness/src/lib.rs", "    fn read(&self) -> u32 {\n        7", "    fn read(&self) -> u32 {\n        70", "changed"),
    ("trait 默认方法", "judge_through_generic", "crates/fcore/src/lib.rs", "self.read() * 2", "self.read() * 3", "changed"),
    ("运算符重载", "judge_operator", "crates/fcore/src/lib.rs", "Money(self.0 + other.0)", "Money(self.0 * other.0)", "changed"),
    ("Drop 隐式调用", "judge_drop", "crates/fcore/src/lib.rs", "let _ = 1;", "let _ = 2;", "changed"),
    ("声明宏展开出的函数", "judge_macro", "crates/fcore/src/lib.rs", "        pub fn $name() -> u32 {\n            5", "        pub fn $name() -> u32 {\n            6", "changed"),
    ("关联常量（经泛型）", "judge_associated_constant", "crates/fharness/src/lib.rs", "const LIMIT: u32 = 10;", "const LIMIT: u32 = 11;", "changed"),
    ("函数指针表", "judge_table", "crates/fcore/src/lib.rs", "pub fn helper() -> u32 {\n    3", "pub fn helper() -> u32 {\n    4", "changed"),
    ("开放入口：参数是 dyn", "judge_open", "crates/fharness/src/lib.rs", "    fn read(&self) -> u32 {\n        7", "    fn read(&self) -> u32 {\n        70", "changed"),
    ("数组长度里的常量", "judge_array_length", "crates/fcore/src/lib.rs", "pub const WIDTH: usize = 4;", "pub const WIDTH: usize = 5;", "changed"),
    ("阴性：够不到的函数", "judge_macro", "crates/fcore/src/lib.rs", "    99\n", "    98\n", "same"),
    ("阴性：只改注释", "judge_operator", "crates/fcore/src/lib.rs", "        Money(self.0 + other.0)", "        // 只加注释\n        Money(self.0 + other.0)", "same"),
    ("带路径的条目级宏调用定义的枚举：改了作废", "judge_listing", "crates/fextra/src/lib.rs", "        Light,\n        Dark,", "        Dark,\n        Light,", "changed"),
    ("verus! 里 ensures 带花括号：下一个函数照样切开，改它作废", "judge_proven", "crates/fextra/src/lib.rs", "    9\n", "    10\n", "changed"),
    ("verus! 里够不到的函数改了不作废", "judge_proven", "crates/fextra/src/lib.rs", "{\n    x\n}", "{\n    x + 0\n}", "same"),
    ("cfg(feature) 关着的单元改了不作废", "judge_speed", "crates/fextra/src/lib.rs", "    22\n", "    23\n", "same"),
    ("cfg(feature) 开着的单元改了作废", "judge_speed", "crates/fextra/src/lib.rs", "    11\n", "    12\n", "changed"),
    ("只改文档注释不作废", "judge_documented", "crates/fextra/src/lib.rs", "/// 文档说明。", "/// 文档说明，改了措辞。", "same"),
    ("改文档注释里的代码块：入口 fn 不作废", "judge_documented", "crates/fextra/src/lib.rs", "documented(), 4);", "documented(), 5);", "same"),
    ("include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), …)) 读的文件改了作废", "judge_table_rows", "crates/fextra/data/table.txt", "row\nrow\n", "row\nrow\nrow\n", "changed"),
    ("WGSL 只改注释不作废", "judge_kernel", "crates/fextra/src/common.wgsl", "// 共用的那一半", "// 共用的那一半，改了措辞", "same"),
    ("WGSL 够不到的函数改了不作废", "judge_kernel", "crates/fextra/src/common.wgsl", "x * 7u", "x * 8u", "same"),
    ("WGSL 够得到的函数改了作废", "judge_kernel", "crates/fextra/src/common.wgsl", "x + GENERATED_LIMIT", "x - GENERATED_LIMIT", "changed"),
    ("format! 串里内联捕获的常量改了作废", "judge_greeting", "crates/fextra/src/lib.rs", 'GREETING: &str = "hi";', 'GREETING: &str = "hello";', "changed"),
)

# 与 crates/singlefs-unit-graph 对齐的那几条（dump 写法、模块头、文档测试指纹、数据单元、WGSL 外名着落、{{{名}}}、union）的小工作区
ALIGNMENT_FIXTURE = {
    "crates/falign/Cargo.toml": '[package]\nname = "falign"\n',
    "crates/falign/src/lib.rs": '''#![allow(dead_code)]
//! 模块说明。
//!
//! ```
//! // 只是注释
//! assert_eq!(falign::width(), 4);
//! ```
pub mod nested {
    #![allow(unused)]
    pub fn inside() -> u32 {
        1
    }
}
mod helpers;
pub const WIDTH: u32 = 4;
pub fn width() -> u32 {
    WIDTH
}
macro_rules! declare_named {
    ($name:ident) => {
        pub fn $name() -> u32 {
            2
        }
    };
}
declare_named!(union);
pub fn call_union() -> u32 {
    union()
}
pub fn wrapped() -> String {
    format!("{{{WIDTH}}}")
}
pub const TABLE_TEXT: &str = include_str!("table.txt");
pub fn table() -> usize {
    TABLE_TEXT.len()
}
pub const KERNEL: &str = include_str!("kernel.wgsl");
macro_rules! failure_name {
    ($name:ident) => {
        concat!(stringify!(FAILURE_), stringify!($name))
    };
}
pub fn kernel_source() -> String {
    format!("const {}: u32 = 0u;\\nconst T_{}: u32 = 1u;\\n{KERNEL}", failure_name!(POINT_NONE), "HEADER")
}
pub const KERNEL_COUNT: &str = include_str!("kernel_count.wgsl");
pub fn count_kernel_source() -> String {
    let COUNT_LIMIT = 3;
    format!("const X: u32 = {}u;\\n{KERNEL_COUNT}", COUNT_LIMIT)
}
''',
    "crates/falign/src/helpers.rs": "pub fn help() -> u32 {\n    5\n}\n",
    "crates/falign/src/main.rs": "fn main() {\n    println!(\"{}\", falign::width());\n}\n",
    "crates/falign/src/bin/tool.rs": "fn main() {}\n",
    "crates/falign/src/table.txt": "row\n",
    "crates/falign/src/kernel.wgsl": '''@group(0) @binding(0) var<storage, read_write> output: array<u32>;
@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    output[id.x] = T_HEADER + FAILURE_POINT_NONE;
}
''',
    "crates/falign/src/kernel_count.wgsl": '''@compute @workgroup_size(1)
fn main() {
    _ = COUNT_LIMIT;
}
''',
}
ALIGNMENT_ENTRY_FILE = "crates/falign/src/lib.rs"


PYTHON_FIXTURE = {
    "pkg/a.py": '''"""模块文档串"""
import importlib.util
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("b_module", os.path.join(HERE, "b.py"))
b_module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(b_module)


def helper():
    """helper 的文档串"""
    # 只是注释
    return 1


def entry():
    return helper() + b_module.value()


def described():
    return helper.__doc__


def dynamic(name):
    return getattr(b_module, name)()


def launch():
    return subprocess.run([sys.executable, os.path.join(HERE, "c.py")], check=False)


class Reader:
    def read(self):
        return 1


class Writer:
    def read(self):
        return 2


def use_reader():
    return Reader().read()


if __name__ == "__main__":
    print(entry())
''',
    "pkg/b.py": '''def value():
    return 2


def unused():
    return 3
''',
    "pkg/c.py": '''def main():
    return 5


if __name__ == "__main__":
    main()
''',
    "pkg/d.py": '''import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))


def handle_alpha():
    return 11


def handle_beta():
    return 12


HANDLERS = {"alpha": handle_alpha, "beta": handle_beta}


def dispatch(name):
    return HANDLERS[name]()


def run_copy(tree):
    return subprocess.run([sys.executable, os.path.join(tree, "pkg", "c.py")], check=False)


def write_data(tree):
    target = os.path.join(tree, "pkg", "out.py")
    with open(target, "w", encoding="utf-8") as handle:
        handle.write("data")
    return subprocess.run([sys.executable, os.path.join(HERE, "c.py"), target], check=False)


def common_directory():
    return subprocess.run(["git", "-C", HERE, "rev-parse", "--path-format=absolute", "--git-common-dir"], capture_output=True, text=True).stdout.strip()


def main_repository_value():
    sys.path.insert(0, os.path.join(os.path.dirname(common_directory()), "pkg"))
    import b
    return b.value()
''',
    "pkg/constants.py": '''TOOLS_RELATIVE = "pkg/tools"
''',
    "pkg/g.py": '''import importlib.util
import os

HERE = os.path.dirname(os.path.abspath(__file__))
CACHE = {}


def cached_b():
    path = os.path.join(HERE, "b.py")
    if path not in CACHE:
        specification = importlib.util.spec_from_file_location("cached_b", path)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        CACHE[path] = module
    return CACHE[path].value()
''',
    "pkg/k.py": '''import os
import subprocess
import sys


def run_tool_named(root, name):
    return subprocess.run([sys.executable, os.path.join(root, "pkg", "tools", name)], check=False)
''',
    "pkg/j.py": '''import importlib.util
import os

HERE = os.path.dirname(os.path.abspath(__file__))
MODULE_PATHS = {"b": os.path.join(HERE, "b.py"), "c": os.path.join(HERE, "c.py")}


def load_named(name):
    specification = importlib.util.spec_from_file_location(name, MODULE_PATHS[name])
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module.value()
''',
    "pkg/h.py": '''import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
STOP_SCRIPT = os.environ.get("FIXTURE_STOP_SCRIPT") or os.path.join(HERE, "c.py")
STUB = "print('stub')\\n"


def stop():
    return subprocess.run([sys.executable, STOP_SCRIPT, "stop"], check=False)


def run_stub(work):
    stub = os.path.join(work, "stub.py")
    with open(stub, "w") as handle:
        handle.write(STUB)
    return subprocess.run([sys.executable, stub], check=False)
''',
    "pkg/f.py": '''import os


def patched_os_function(name, replacement):
    original = getattr(os, name)
    setattr(os, name, replacement)
    return original
''',
    "pkg/e.py": '''import os
import sys

import constants


def load_tool(repository):
    sys.path.insert(0, os.path.join(repository, constants.TOOLS_RELATIVE))
    import tool
    return tool.run()
''',
    "pkg/tools/tool.py": '''def run():
    return 7
''',
    "pkg/more_tools/tool.py": '''def run():
    return 8
''',
    "pkg/tools/main_tool.py": '''def main():
    return 9


if __name__ == "__main__":
    main()
''',
    "pkg/i.py": '''import importlib
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TOOL_DIRECTORIES = (os.path.join(HERE, "tools"), os.path.join(HERE, "more_tools"))
TOOL_MODULE = "tool"


def load_named(name):
    for directory in TOOL_DIRECTORIES:
        if os.path.basename(directory) == name:
            sys.path.insert(0, directory)
            break
    import tool
    return tool.run()


def load_by_constant(name):
    for directory in TOOL_DIRECTORIES:
        if os.path.basename(directory) == name:
            sys.path.insert(0, directory)
    return importlib.import_module(TOOL_MODULE).run()


def load_each():
    values = []
    for name in MODULE_FILES:
        specification = importlib.util.spec_from_file_location("each", os.path.join(HERE, name))
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        values.append(module.value())
    return ",".join(name for name, _code in [("x", 1)])


MODULE_FILES = ("b.py",)
''',
    "pkg/m.py": '''import os
import shutil
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))


def run_inline():
    return subprocess.run([sys.executable, "-c", "import sys; sys.path.insert(0, sys.argv[1]); import b; b.value()", HERE], check=False)


def run_dynamic(code):
    return subprocess.run([sys.executable, "-c", code], check=False)


def run_unknown(interpreter):
    return subprocess.run([interpreter, os.path.join(HERE, "c.py")], check=False)


def launch_with(extra):
    return subprocess.run([sys.executable] + extra, check=False)


def launch_tool():
    return launch_with([os.path.join(HERE, "c.py")])


def launch_job(job):
    return subprocess.run(job.command, check=False)


def run_tests():
    return subprocess.run([sys.executable, "-m", "unittest", "test_sample"], cwd=HERE, check=False)


def version_of(name, toolchain):
    found = subprocess.run([shutil.which(name), "--version"], check=False)
    return found, subprocess.run([os.path.join(toolchain, "bin", "llvm-cov"), "--version"], check=False)


def run_with_options(options):
    return subprocess.run([*options["launcher"], os.path.join(HERE, "c.py")], check=False)


def run_default():
    return run_with_options({"launcher": ["python3"]})


def launch_listed(command):
    return subprocess.run(command, check=False)


LAUNCHERS = {"listed": launch_listed}


def launch_from_table():
    return LAUNCHERS["listed"]([sys.executable, os.path.join(HERE, "c.py")])
''',
    "pkg/test_sample.py": '''import unittest


class SampleTest(unittest.TestCase):
    def test_one(self):
        return 3
''',
    ".claude/singlefs-ai-sop/scripts/preflight.py": '''import subprocess


def run_check(declaration):
    return subprocess.run(["bash", "-c", declaration.command], check=False)
''',
}

SHELL_FIXTURE = {
    "s/main.sh": '''#!/usr/bin/env bash
# admission: always 样本
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$here/lib.sh"
emit() {
  cat <<'EOF'
first
    };
second
EOF
}
cell_alpha() { echo alpha; }
cell_beta() { echo beta; }
LIBRARY_TEXT="$(cat <<'PY'
print("library")
PY
)"
use_library() { printf '%s\\n' "$LIBRARY_TEXT"; }
run_cell() { "$1"; }
run_cell cell_alpha
bash "$here/child.sh"
''',
    "s/lib.sh": '''#!/usr/bin/env bash
lib_function() { echo lib; }
''',
    "s/child.sh": '''#!/usr/bin/env bash
echo child
''',
    "s/dynamic.sh": '''#!/usr/bin/env bash
x="$1"
bash "$x"
''',
    "s/broken.sh": '''#!/usr/bin/env bash
if then fi (
''',
    "s/tools.sh": '''#!/usr/bin/env bash
build() {
  cargo build
}
build
''',
    "s/copies.sh": '''#!/usr/bin/env bash
tree="$1"
bash "$tree/s/child.sh"
''',
    "s/versions.sh": '''#!/usr/bin/env bash
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
run_in() {
  local directory
  case "$1" in
    alpha) directory="$here/alpha" ;;
    beta) directory="$here/beta" ;;
  esac
  (cd "$directory" && bash tool.sh)
}
run_in alpha
run_in beta
''',
    "s/alpha/tool.sh": '''#!/usr/bin/env bash
echo alpha tool
''',
    "s/beta/tool.sh": '''#!/usr/bin/env bash
echo beta tool
''',
    "s/nested.sh": '''#!/usr/bin/env bash
outer() {
  local helper
  helper="$(dirname "${BASH_SOURCE[0]}")/child.sh"
  inner() { bash "$helper"; }
  inner
}
outer
''',
    "s/selfcopy.sh": '''#!/usr/bin/env bash
copy="$1/run.sh"
cp "$(dirname "${BASH_SOURCE[0]}")/child.sh" "$copy"
bash "$copy"
''',
    "s/common.sh": '''#!/usr/bin/env bash
if common="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)"; then
  source "${common%/*}/s/lib.sh"
fi
lib_function
''',
    "s/redirect.sh": '''#!/usr/bin/env bash
out="$1"
grep -v '^#' "$2" > "$out/filtered.sh"
''',
    "s/arguments.sh": '''#!/usr/bin/env bash
write_stage() { printf '%s\\n' "$2" | tee "$1" >/dev/null; }
write_stage "$1/stage.sh" 'echo stage'
''',
    "s/generator.sh": '''#!/usr/bin/env bash
out="$1"
cat > "$out/run.sh" <<'RUN'
bash s/child.sh
RUN
bash "$out/run.sh"
''',
    "s/inline.sh": '''#!/usr/bin/env bash
tree="$1"
bash -c 'source "$1"; lib_function' inline-case "$tree/s/lib.sh"
''',
    "s/py/helper_mod.py": '''def run():
    return 1
''',
    "s/py/tool.py": '''def main():
    return 2


if __name__ == "__main__":
    main()
''',
    "s/inline_python.sh": '''#!/usr/bin/env bash
python3 -B -c 'import sys; sys.path.insert(0, sys.argv[1]); import helper_mod; helper_mod.run()' "$(dirname "${BASH_SOURCE[0]}")/py"
''',
    "s/heredoc_python.sh": '''#!/usr/bin/env bash
here="$(dirname "${BASH_SOURCE[0]}")"
python3 - "$here/py" <<'PY'
import sys
sys.path.insert(0, sys.argv[1])
import helper_mod
helper_mod.run()
PY
''',
    "s/inline_expanded.sh": '''#!/usr/bin/env bash
code="$1"
python3 -c "$code"
bash -c "$2"
''',
    "s/inline_bash_folded.sh": '''#!/usr/bin/env bash
step="bash $(dirname "${BASH_SOURCE[0]}")/child.sh"
bash -c "$step"
''',
    "s/second.sh": '''#!/usr/bin/env bash
here="$(dirname "${BASH_SOURCE[0]}")"
bash "$here/lib.sh" 2 python3 -B "$here/py/tool.py"
''',
    "s/substitution.sh": '''#!/usr/bin/env bash
check_output="$(bash "$(dirname "${BASH_SOURCE[0]}")/child.sh" 2>&1)" || exit 1
printf '%s\\n' "$check_output"
''',
    "s/process_substitution.sh": '''#!/usr/bin/env bash
file="$(pick_file)"
mapfile -t lines < <(read_lines "$file")
''',
    "s/message.sh": '''#!/usr/bin/env bash
die() { echo "$1" >&2; exit 1; }
command -v true >/dev/null || die "qemu-system-x86_64 缺失" "装上 qemu-system-x86 再跑"
[[ -n "${1:-}" ]] || die "缺 qemu-system-x86_64" "装上 qemu-system-x86 再跑"
''',
    "s/module.sh": '''#!/usr/bin/env bash
cd "$(dirname "${BASH_SOURCE[0]}")/py" && python3 -m tool
''',
    "s/head.sh": '''#!/usr/bin/env bash
runner="$(find_runner)"
"$runner" --flag
''',
    "s/wrapped.sh": '''#!/usr/bin/env bash
timeout 60 "$(dirname "${BASH_SOURCE[0]}")/child.sh"
''',
    "s/structure.sh": '''#!/usr/bin/env bash
x="$(pick_x)" y="$(pick_y)"
if [[ -n "$x" || "$y" == ok ]]; then echo yes; fi
case "$1" in
  "$x") echo first ;;
  *) echo other ;;
esac
pair=( "$x" "$y" )
(( $# > 0 )) && echo "${pair[0]}"
''',
    "s/array.sh": '''#!/usr/bin/env bash
command_words=(bash "$(dirname "${BASH_SOURCE[0]}")/child.sh")
"${command_words[@]}"
''',
    "s/capture.sh": '''#!/usr/bin/env bash
code_of() {
  printf '%s\\n' "import sys" "sys.path.insert(0, sys.argv[1])" "import helper_mod" "helper_mod.run()"
}
code="$(code_of)"
python3 -c "$code" "$(dirname "${BASH_SOURCE[0]}")/py"
''',
    "s/declared.sh": '''#!/usr/bin/env bash
# run-condition: check bash "$PREFLIGHT_SCRIPT_DIRECTORY/child.sh" :: 先让同目录的 child.sh 跑通
echo declared
''',
    "s/argv_exec.sh": '''#!/usr/bin/env bash
extra=(SAMPLE=1)
python3 -c 'import os, sys; os.execvp(sys.argv[1], sys.argv[1:])' env "${extra[@]}" bash "$(dirname "${BASH_SOURCE[0]}")/child.sh"
''',
    "s/positional_tool.sh": '''#!/usr/bin/env bash
tool_path="$1"
python3 - "$tool_path" <<'PY'
import subprocess
import sys
tool = sys.argv[1]
subprocess.run([tool, "--version"], check=False)
PY
''',
    "s/group.sh": '''#!/usr/bin/env bash
here="$(dirname "${BASH_SOURCE[0]}")"
{
  python3 /dev/fd/3 "$here/py"
} 3<<'PY'
import sys
sys.path.insert(0, sys.argv[1])
import helper_mod
helper_mod.run()
PY
''',
}


def write_tree(base, files):
    for path, text in files.items():
        absolute = os.path.join(base, path)
        os.makedirs(os.path.dirname(absolute), exist_ok=True)
        with open(absolute, "w", encoding="utf-8") as handle:
            handle.write(text)


def edit_file(base, path, old, new):
    absolute = os.path.join(base, path)
    with open(absolute, encoding="utf-8") as handle:
        text = handle.read()
    if text.count(old) != 1:
        raise AssertionError(f"自证样本 {path} 里「{old}」命中 {text.count(old)} 次（要恰好一次）")
    with open(absolute, "w", encoding="utf-8") as handle:
        handle.write(text.replace(old, new))


def fixture_key(root, entry, options=None):
    """→ (键或 None, 拒算形态的集合)。"""
    closure = closure_of(root, entry, options)
    key, refusals = closure_key(closure)
    return key, {refusal.form for refusal in refusals}


def compare_cell(fixture, entry, path, old, new, want):
    def judge(work):
        base = os.path.join(work, "tree")
        write_tree(base, fixture)
        before, before_refused = fixture_key(base, entry)
        edit_file(base, path, old, new)
        after, after_refused = fixture_key(base, entry)
        if before is None or after is None:
            return False, f"拒算了：改前 {sorted(before_refused)}，改后 {sorted(after_refused)}"
        got = "changed" if before != after else "same"
        return got == want, f"期望 {want}，实际 {got}"
    return judge


def refused_cell(fixture, entry, form, edit=None):
    def judge(work):
        base = os.path.join(work, "tree")
        write_tree(base, fixture)
        if edit is not None:
            edit_file(base, *edit)
        key, forms = fixture_key(base, entry)
        return key is None and form in forms, f"要拒算 {form}，实际键 {'有' if key else '无'}、拒算 {sorted(forms)}"
    return judge


def rust_doctest_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, RUST_FIXTURE)
    rust_file = RustWorkspace(base).file("crates/fextra/src/lib.rs")
    doctests = [unit.name for unit in rust_file.units if unit.kind == "doctest"]
    if len(doctests) != 1:
        return False, f"要恰好一个文档测试单元，实际 {doctests}"
    entry = f"crates/fextra/src/lib.rs::{doctests[0]}"
    before, _ = fixture_key(base, entry)
    edit_file(base, "crates/fextra/src/lib.rs", "documented(), 4);", "documented(), 5);")
    after, _ = fixture_key(base, entry)
    return before is not None and after is not None and before != after, f"文档测试入口 {entry} 的键改前 {before}、改后 {after}"


def rust_dump_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, RUST_FIXTURE)
    unit_lines, refusal_lines, counts = dump_lines(base, "rust")
    shape = re.compile(r"^[a-z_]+\t[^\t]+\t[^\t]+:\d+-\d+\t[0-9a-f]{64}$")
    bad = [line for line in unit_lines if not shape.match(line)]
    wanted = ("item_macro\tfextra::listing!@5\tcrates/fextra/src/lib.rs:5-10", "fn\tfextra::bounded\tcrates/fextra/src/lib.rs:",
              "fn\tfextra::proven\tcrates/fextra/src/lib.rs:", "impl\tfcore::<impl std::ops::Add for Money>\tcrates/fcore/src/lib.rs:15-15",
              "fn\tfcore::<impl std::ops::Add for Money>::add\t")
    missing = [prefix for prefix in wanted if not any(line.startswith(prefix) for line in unit_lines)]
    return not bad and not missing and not refusal_lines and counts["rust"] > 30, f"形态不对 {bad[:2]}，缺 {missing}，拒算 {refusal_lines[:3]}"


def alignment_dump(work):
    base = os.path.join(work, "tree")
    write_tree(base, ALIGNMENT_FIXTURE)
    unit_lines = dump_lines(base, "rust")[0] + dump_lines(base, "wgsl")[0]
    return base, unit_lines


def line_with(lines, prefix):
    return any(line.startswith(prefix) for line in lines)


def alignment_kind_names_cell(work):
    """dump 的种类名照 crates/singlefs-unit-graph 的拼法：macro_rules、item_macro、mod、mod_head、wgsl_compute、wgsl_var。"""
    _base, lines = alignment_dump(work)
    wanted = ("macro_rules\tfalign::declare_named\t", "item_macro\tfalign::declare_named!@26\t", "mod\tfalign::helpers\t", "mod_head\tfalign::nested\t",
              "wgsl_compute\tfalign::kernel.wgsl::main\t", "wgsl_var\tfalign::kernel.wgsl::output\t")
    missing = [prefix for prefix in wanted if not line_with(lines, prefix)]
    return not missing, f"dump 里缺 {missing}"


def alignment_qualified_names_cell(work):
    """dump 的限定名带 crate、目标与模块路径前缀（lib 不写目标，bin 写 <bin 名>，mod 声明连进来的文件带模块路径）；文档测试写 <模块>::<doctest line 行>。"""
    _base, lines = alignment_dump(work)
    wanted = ("fn\tfalign::helpers::help\tcrates/falign/src/helpers.rs:1-3\t", "fn\tfalign::<bin falign>::main\tcrates/falign/src/main.rs:1-3\t",
              "fn\tfalign::<bin tool>::main\tcrates/falign/src/bin/tool.rs:1-1\t", "fn\tfalign::nested::inside\tcrates/falign/src/lib.rs:10-12\t",
              "doctest\tfalign::<doctest line 4>\tcrates/falign/src/lib.rs:4-7\t", "wgsl_compute\tfalign::kernel_count.wgsl::main\t")
    missing = [prefix for prefix in wanted if not line_with(lines, prefix)]
    return not missing, f"dump 里缺 {missing}"


def doctest_fingerprint(base):
    units = [unit for unit in RustWorkspace(base).file(ALIGNMENT_ENTRY_FILE).units if unit.kind == "doctest"]
    return units[0].fingerprint if len(units) == 1 else f"文档测试单元 {len(units)} 个"


def alignment_doctest_fingerprint_cell(work):
    """文档测试的指纹按代码块重新切词、去注释：代码块里只改注释不变，改代码变（不随原文各行变）。"""
    base = os.path.join(work, "tree")
    write_tree(base, ALIGNMENT_FIXTURE)
    before = doctest_fingerprint(base)
    edit_file(base, ALIGNMENT_ENTRY_FILE, "//! // 只是注释\n", "//! // 只是注释，改了措辞\n")
    commented = doctest_fingerprint(base)
    edit_file(base, ALIGNMENT_ENTRY_FILE, "falign::width(), 4);", "falign::width(), 5);")
    changed = doctest_fingerprint(base)
    return before == commented and commented != changed and len(before) == 64, f"改注释前 {before}、改注释后 {commented}、改代码后 {changed}"


def alignment_module_head_cell(work):
    """模块头：文件开头的 #![…] 与行内模块头（连内属性）各切成一个 mod_head 单元；模块里的单元连它，改内属性作废。"""
    base, lines = alignment_dump(work)
    wanted = ("mod_head\tfalign::<module attributes>\tcrates/falign/src/lib.rs:1-1\t", "mod_head\tfalign::nested\tcrates/falign/src/lib.rs:8-9\t")
    missing = [prefix for prefix in wanted if not line_with(lines, prefix)]
    before, _ = fixture_key(base, f"{ALIGNMENT_ENTRY_FILE}::nested::inside")
    edit_file(base, ALIGNMENT_ENTRY_FILE, "#![allow(unused)]", "#![allow(unused_variables)]")
    after, _ = fixture_key(base, f"{ALIGNMENT_ENTRY_FILE}::nested::inside")
    return not missing and before is not None and before != after, f"dump 里缺 {missing}；nested::inside 的键改内属性前 {before}、后 {after}"


def alignment_data_dump_cell(work):
    """dump 按读它的那一处列数据单元：data<TAB>数据文件<TAB>读它的文件:行-行<TAB>字节 sha256（.wgsl 不列，按 WGSL 单元列）。"""
    _base, lines = alignment_dump(work)
    want = f"data\tcrates/falign/src/table.txt\tcrates/falign/src/lib.rs:33-33\t{sha256_bytes(b'row' + bytes([10]))}"
    wgsl_listed = [line for line in lines if line.startswith("data\t") and ".wgsl" in line.split("\t")[1]]
    return want in lines and not wgsl_listed, f"要有 {want}，读 .wgsl 的不列：{wgsl_listed}"


def alignment_external_prefix_cell(work):
    """WGSL 外名的首段连 _ 落在闭包里 macro_rules! 体的词法单元里（FAILURE_POINT_NONE 取 FAILURE_）或串字面量里（T_HEADER 取 T_）就有着落、不拒算。"""
    base = os.path.join(work, "tree")
    write_tree(base, ALIGNMENT_FIXTURE)
    key, forms = fixture_key(base, f"{ALIGNMENT_ENTRY_FILE}::kernel_source")
    return key is not None, f"不该拒算，实际拒算 {sorted(forms)}"


def alignment_external_identifier_cell(work):
    """WGSL 外名只出现在闭包里别的 Rust 单元的长标识符里（let COUNT_LIMIT）、不在串字面量与宏体里：拒算（不再按「标识符长于 3 个字符」认）。"""
    base = os.path.join(work, "tree")
    write_tree(base, ALIGNMENT_FIXTURE)
    closure = closure_of(base, f"{ALIGNMENT_ENTRY_FILE}::count_kernel_source")
    refused = [refusal for refusal in closure.refusals if refusal.form == "wgsl-undefined-name"]
    return [refusal.snippet for refusal in refused] == ["COUNT_LIMIT"], f"要恰好拒算 COUNT_LIMIT，实际 {[refusal.text() for refusal in closure.refusals]}"


def alignment_external_edge_cell(work):
    """外名的着落做成边：WGSL 入口（--rust-strings-entry 给拼内核的 Rust 函数）的闭包带上给名字着落的 Rust 单元与它的闭包，改那个单元键变。"""
    base = os.path.join(work, "tree")
    write_tree(base, ALIGNMENT_FIXTURE)
    options = Options()
    options.rust_strings_entry = f"{ALIGNMENT_ENTRY_FILE}::kernel_source"
    entry = "crates/falign/src/kernel.wgsl::main"
    before, before_refusals = closure_key(closure_of(base, entry, options))
    edit_file(base, ALIGNMENT_ENTRY_FILE, "const T_{}: u32 = 1u;", "const T_{}: u32 = 2u;")
    after, _ = closure_key(closure_of(base, entry, options))
    return before is not None and before != after, f"改给 T_ 着落的那一串，WGSL 入口的键改前 {before}（拒算 {[r.text() for r in before_refusals]}）、改后 {after}"


def alignment_union_cell(work):
    """union 是上下文关键字：declare_named!(union) 这条条目级宏调用可能定义 union，调 union() 的函数连到它。"""
    base = os.path.join(work, "tree")
    write_tree(base, ALIGNMENT_FIXTURE)
    closure = closure_of(base, f"{ALIGNMENT_ENTRY_FILE}::call_union")
    macros = [unit.name for unit in closure.order if unit.kind == "item_macro"]
    return macros == ["declare_named!@26"], f"call_union 的闭包里条目级宏调用：{macros}"


def alignment_cells():
    return [
        ("dump：种类名照 crates/singlefs-unit-graph 的拼法（macro_rules、item_macro、mod、mod_head、wgsl_compute）", alignment_kind_names_cell),
        ("dump：限定名带 crate、目标与模块路径前缀，文档测试写 <模块>::<doctest line 行>", alignment_qualified_names_cell),
        ("文档测试：代码块里只改注释指纹不变、改代码指纹变", alignment_doctest_fingerprint_cell),
        ("模块头：#![…] 与行内模块头切成单元，模块里的单元连它，改内属性作废", alignment_module_head_cell),
        ("dump：include_str! 读的数据文件按读它的那一处列一行 data", alignment_data_dump_cell),
        ("WGSL 外名：首段连 _ 落在 macro_rules! 体或串字面量里有着落", alignment_external_prefix_cell),
        ("WGSL 外名：只在别的单元的长标识符里出现的拒算", alignment_external_identifier_cell),
        ("WGSL 外名的着落做成边：给名字的 Rust 单元改了 WGSL 入口作废", alignment_external_edge_cell),
        ("format! 串里 {{{名}}}（转义的 {{ 后紧跟的捕获）改了作废", compare_cell(ALIGNMENT_FIXTURE, f"{ALIGNMENT_ENTRY_FILE}::wrapped", ALIGNMENT_ENTRY_FILE,
                                                         "pub const WIDTH: u32 = 4;", "pub const WIDTH: u32 = 5;", "changed")),
        ("条目级宏调用里的 union 算它可能定义的名字", alignment_union_cell),
    ]


# R-结构（模块头、mod 声明、use）与 R-隐式按自身类型认 trait impl 的小工作区（与 crates/singlefs-unit-graph 的 closure.rs 同一口径）
CLOSURE_RULES_FIXTURE = {
    "crates/frules/Cargo.toml": '[package]\nname = "frules"\n',
    "crates/frules/src/lib.rs": '''#![allow(dead_code)]
mod sub;
pub mod inline_area {
    #![allow(unused)]
    pub fn inside() -> u32 {
        1
    }
}
pub fn outside() -> u32 {
    2
}
pub mod tools {
    pub fn used() -> u32 {
        3
    }
    pub fn unused() -> u32 {
        4
    }
    pub fn original() -> u32 {
        5
    }
}
use tools::*;
pub fn glob_caller() -> u32 {
    used()
}
pub mod listing {
    use crate::tools::{used, unused};
    pub fn list_caller() -> u32 {
        used()
    }
}
pub mod renaming {
    use crate::tools::original as renamed;
    pub fn alias_caller() -> u32 {
        renamed()
    }
}
pub struct Inner;
pub struct Outer;
impl From<Inner> for Outer {
    fn from(_value: Inner) -> Outer {
        Outer
    }
}
pub fn make_inner() -> Inner {
    Inner
}
pub fn make_outer() -> Outer {
    Outer
}
''',
    "crates/frules/src/sub.rs": "pub fn in_sub() -> u32 {\n    6\n}\n",
}
CLOSURE_RULES_FILE = "crates/frules/src/lib.rs"


def closure_member_names(base, entry):
    return {f"{unit.kind} {unit.name}" for unit in closure_of(base, entry).order if not unit.data}


def closure_rules_file_head_cell(work):
    """改文件开头的 #![…]：经 mod sub; 连进来的 sub.rs 里的入口作废（入口连声明它所在文件的 mod，mod 再连 lib.rs 的模块头）。"""
    base = os.path.join(work, "tree")
    write_tree(base, CLOSURE_RULES_FIXTURE)
    entry = "crates/frules/src/sub.rs::in_sub"
    members = closure_member_names(base, entry)
    wanted = {"mod sub", "mod_head <module attributes>"}
    before, _ = fixture_key(base, entry)
    edit_file(base, CLOSURE_RULES_FILE, "#![allow(dead_code)]", "#![allow(dead_code, unused_imports)]")
    after, _ = fixture_key(base, entry)
    return wanted <= members and before is not None and before != after, f"in_sub 的闭包缺 {sorted(wanted - members)}；改 #![…] 前键 {before}、后 {after}"


def closure_rules_inline_head_cell(work):
    """改行内模块头的内属性：只那个模块里的入口作废，模块外同一份文件里的入口不变。"""
    base = os.path.join(work, "tree")
    write_tree(base, CLOSURE_RULES_FIXTURE)
    inside, outside = f"{CLOSURE_RULES_FILE}::inline_area::inside", f"{CLOSURE_RULES_FILE}::outside"
    inside_before, outside_before = fixture_key(base, inside)[0], fixture_key(base, outside)[0]
    edit_file(base, CLOSURE_RULES_FILE, "#![allow(unused)]", "#![allow(unused_variables)]")
    inside_after, outside_after = fixture_key(base, inside)[0], fixture_key(base, outside)[0]
    ok = None not in (inside_before, outside_before) and inside_before != inside_after and outside_before == outside_after
    return ok, f"inline_area::inside 改前 {inside_before}、后 {inside_after}；outside 改前 {outside_before}、后 {outside_after}"


def closure_rules_glob_cell(work):
    """use tools::* 之后只用到 tools 里一个 fn：闭包有那条 use 与那个 fn，没有 tools 里别的 fn；改没用到的 fn 不作废。"""
    base = os.path.join(work, "tree")
    write_tree(base, CLOSURE_RULES_FIXTURE)
    entry = f"{CLOSURE_RULES_FILE}::glob_caller"
    members = closure_member_names(base, entry)
    uses = sorted(name for name in members if name.startswith("use "))
    extra = sorted({"fn tools::unused", "fn tools::original"} & members)
    before, _ = fixture_key(base, entry)
    edit_file(base, CLOSURE_RULES_FILE, "        4\n", "        40\n")
    after, _ = fixture_key(base, entry)
    ok = len(uses) == 1 and "fn tools::used" in members and not extra and before is not None and before == after
    return ok, f"glob_caller 的闭包里 use {uses}、有没有 tools::used {'fn tools::used' in members}、多出 {extra}；改 unused 前键 {before}、后 {after}"


def closure_rules_listed_cell(work):
    """use crate::tools::{used, unused} 只用 used：use 只结构上连、引进的名字不往下解，闭包里没有 unused，改它不作废。"""
    base = os.path.join(work, "tree")
    write_tree(base, CLOSURE_RULES_FIXTURE)
    entry = f"{CLOSURE_RULES_FILE}::listing::list_caller"
    members = closure_member_names(base, entry)
    before, _ = fixture_key(base, entry)
    edit_file(base, CLOSURE_RULES_FILE, "        4\n", "        40\n")
    after, _ = fixture_key(base, entry)
    uses = sorted(name for name in members if name.startswith("use "))
    ok = uses and "fn tools::unused" not in members and before is not None and before == after
    return ok, f"list_caller 的闭包里 use {uses}、有没有 tools::unused {'fn tools::unused' in members}；改 unused 前键 {before}、后 {after}"


def closure_rules_alias_cell(work):
    """use crate::tools::original as renamed 之后调 renamed()：按原名 original 再解，闭包含 tools::original，改它作废。"""
    base = os.path.join(work, "tree")
    write_tree(base, CLOSURE_RULES_FIXTURE)
    entry = f"{CLOSURE_RULES_FILE}::renaming::alias_caller"
    members = closure_member_names(base, entry)
    before, _ = fixture_key(base, entry)
    edit_file(base, CLOSURE_RULES_FILE, "        5\n", "        50\n")
    after, _ = fixture_key(base, entry)
    return "fn tools::original" in members and before is not None and before != after, \
        f"alias_caller 的闭包里有没有 tools::original {'fn tools::original' in members}；改 original 前键 {before}、后 {after}"


def closure_rules_trait_argument_cell(work):
    """R-隐式按自身类型认 trait impl：只点名 impl From<Inner> for Outer 里 trait 参数的 Inner，这个 impl 不进闭包；点名 Outer 的进。"""
    base = os.path.join(work, "tree")
    write_tree(base, CLOSURE_RULES_FIXTURE)
    impl_name = "impl <impl From < Inner > for Outer>"
    inner, outer = closure_member_names(base, f"{CLOSURE_RULES_FILE}::make_inner"), closure_member_names(base, f"{CLOSURE_RULES_FILE}::make_outer")
    return impl_name not in inner and impl_name in outer, f"make_inner 的闭包里有没有那个 impl {impl_name in inner}，make_outer 的 {impl_name in outer}"


def closure_rule_cells():
    return [
        ("R-结构：改文件开头的 #![…]，经 mod 声明连进来的子模块里的入口作废", closure_rules_file_head_cell),
        ("R-结构：改行内模块头的内属性，只那个模块里的入口作废", closure_rules_inline_head_cell),
        ("R-结构：use a::* 只用到 a 里一个 fn，闭包有那条 use 与那个 fn、没有 a 里别的 fn", closure_rules_glob_cell),
        ("R-结构：use 引进的名字不往下解，没用到的那个改了不作废", closure_rules_listed_cell),
        ("R-结构：use a::b as c 之后调 c()，按 b 再解", closure_rules_alias_cell),
        ("R-隐式：只点名 trait 参数里的类型，那个 trait impl 不进闭包", closure_rules_trait_argument_cell),
    ]


# 4-3d：键的 cfg 成分收窄（定案 2）、WGSL 外名的 _ 后缀与供名字的边、trait 成员里的 Self::x、文档测试不连结构、use a::{self}（定案 3）的小工作区；
# 闭包成员与 crates/singlefs-unit-graph 的 unit-graph closure 逐行核过（报告里有命令与原样输出）
CFG_ATOMS_FIXTURE = {
    "crates/fcfg/Cargo.toml": '[package]\nname = "fcfg"\n\n[features]\ngpu = []\n',
    "crates/fcfg/src/lib.rs": '''pub fn plain_entry() -> u32 {
    helper() + 1
}
fn helper() -> u32 {
    2
}
pub fn macro_entry() -> u32 {
    if cfg!(feature = "gpu") { 3 } else { 4 }
}
pub fn statement_entry() -> u32 {
    #[cfg(feature = "gpu")]
    let value = 8;
    #[cfg(not(feature = "gpu"))]
    let value = 9;
    value
}
#[cfg_attr(feature = "gpu", inline)]
pub fn attribute_only_entry() -> u32 {
    7
}
''',
}
CFG_ATOMS_FILE = "crates/fcfg/src/lib.rs"


def cfg_keys(base, entry, features):
    """入口在给的特性下 → (unit_graph_key, closure_key)；拒算的那一把是 None。"""
    workspace = RustWorkspace(base)
    language, path, qualified = parse_entry(base, entry)
    unit, target_id = workspace.find_entry(path, qualified, None)
    closure = rust_closure_of(workspace, unit, target_id, None, False, list(features), False)
    return unit_graph_key(closure, ["host: x86_64-unknown-linux-gnu"])[0], closure_key(closure)[0]


def cfg_unreferenced_cell(work):
    """闭包里没有单元引到 gpu 的入口：带不带 gpu 特性两种目标下两把键（unit_graph_key 与 closure_key）都相同。"""
    base = os.path.join(work, "tree")
    write_tree(base, CFG_ATOMS_FIXTURE)
    without = cfg_keys(base, f"{CFG_ATOMS_FILE}::plain_entry", [])
    with_gpu = cfg_keys(base, f"{CFG_ATOMS_FILE}::plain_entry", ["fcfg/gpu"])
    return None not in without + with_gpu and without == with_gpu, f"不带 gpu {without}、带 gpu {with_gpu}"


def cfg_referenced_cell(work):
    """闭包里有单元引到 gpu（函数体里的 cfg!、语句上的 #[cfg]、入口自己的 #[cfg_attr]）：带不带 gpu 特性两把键都不同。"""
    base = os.path.join(work, "tree")
    write_tree(base, CFG_ATOMS_FIXTURE)
    same = []
    for name in ("macro_entry", "statement_entry", "attribute_only_entry"):
        without = cfg_keys(base, f"{CFG_ATOMS_FILE}::{name}", [])
        with_gpu = cfg_keys(base, f"{CFG_ATOMS_FILE}::{name}", ["fcfg/gpu"])
        if None in without + with_gpu or without[0] == with_gpu[0] or without[1] == with_gpu[1]:
            same.append(f"{name}：不带 {without}、带 {with_gpu}")
    return not same, "；".join(same)


# 外名由 Rust 现拼的三种写法（与 crates/singlefs-unit-graph 的 tests/unit_graph_wgsl_modules_close_over_compute_entries.rs 的三份夹具同形）
WGSL_SUPPLY_FIXTURE = {
    "crates/fsupply/Cargo.toml": '[package]\nname = "fsupply"\n',
    "crates/fsupply/src/lib.rs": '''pub const CODE_TABLE_KERNEL_SOURCE: &str = include_str!("failure_points.wgsl");
pub const UNRELATED_KERNEL_SOURCE: &str = include_str!("unrelated.wgsl");
pub const SUPPLIED_KERNEL_SOURCE: &str = include_str!("supplied.wgsl");
pub const PRELUDE_SOURCE: &str = include_str!("prelude.wgsl");
pub const SPLIT_KERNEL_SOURCE: &str = include_str!("split.wgsl");
macro_rules! failure_points {
    ($($constant:ident = $code:literal,)*) => {
        $(pub const $constant: u32 = $code;)*
        pub const GPU_FAILURE_POINTS: &[(&str, u32)] = &[$((stringify!($constant), $code),)*];
    };
}
failure_points! {
    GPU_FAILURE_POINT_NONE = 0,
    GPU_FAILURE_POINT_UNREADABLE = 9,
}
pub fn failure_point_constants() -> String {
    GPU_FAILURE_POINTS
        .iter()
        .map(|(name, code)| format!("const {}: u32 = {code}u;\\n", name.strip_prefix("GPU_").expect("都以 GPU_ 起头")))
        .collect()
}
pub fn failure_point_kernel_text() -> String {
    assert_eq!(GPU_FAILURE_POINT_NONE, 0, "无失败点的码是 0");
    format!("{}{CODE_TABLE_KERNEL_SOURCE}", failure_point_constants())
}
pub fn supplied_kernel_text() -> String {
    format!("{}{SUPPLIED_KERNEL_SOURCE}", failure_point_constants())
}
pub fn unrelated_kernel_text() -> String {
    let note = "FOO_ 起头的说明串，与内核里的名字无关";
    format!("// {note}\\n{UNRELATED_KERNEL_SOURCE}")
}
pub fn prelude_text() -> &'static str {
    PRELUDE_SOURCE
}
pub fn split_kernel_text() -> String {
    format!("{}{SPLIT_KERNEL_SOURCE}", prelude_text())
}
''',
    "crates/fsupply/src/failure_points.wgsl": '''@group(0) @binding(0) var<storage, read_write> words: array<u32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    words[id.x] = FAILURE_POINT_NONE + FAILURE_POINT_UNREADABLE;
}
''',
    "crates/fsupply/src/unrelated.wgsl": '''@group(0) @binding(0) var<storage, read_write> words: array<u32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    words[id.x] = FOO_BAR;
}
''',
    "crates/fsupply/src/supplied.wgsl": '''@group(0) @binding(0) var<storage, read_write> words: array<u32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    words[id.x] = FAILURE_POINT_UNREADABLE;
}
''',
    "crates/fsupply/src/prelude.wgsl": '''fn doubled(value: u32) -> u32 {
    return value * 2u;
}
''',
    "crates/fsupply/src/split.wgsl": '''@group(0) @binding(0) var<storage, read_write> words: array<u32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    words[id.x] = doubled(id.x);
}
''',
}
WGSL_SUPPLY_FILE = "crates/fsupply/src/lib.rs"


def wgsl_defined_suffix_cell(work):
    """条目级宏调用定义的 GPU_FAILURE_POINT_NONE 以 _ 加外名收尾：FAILURE_POINT_NONE 有着落、不拒算（不靠首段规则：其余那一段 POINT_NONE 哪儿都没有）。"""
    base = os.path.join(work, "tree")
    write_tree(base, WGSL_SUPPLY_FIXTURE)
    key, forms = fixture_key(base, f"{WGSL_SUPPLY_FILE}::failure_point_kernel_text")
    return key is not None, f"不该拒算，实际拒算 {sorted(forms)}"


def wgsl_prefix_remainder_cell(work):
    """外名 FOO_BAR 的首段 FOO_ 只落在一个无关的说明串里、其余那一段 BAR 哪儿都没有：拒算 wgsl-undefined-name。"""
    base = os.path.join(work, "tree")
    write_tree(base, WGSL_SUPPLY_FIXTURE)
    closure = closure_of(base, f"{WGSL_SUPPLY_FILE}::unrelated_kernel_text")
    refused = [refusal.snippet for refusal in closure.refusals if refusal.form == "wgsl-undefined-name"]
    return refused == ["FOO_BAR"], f"要恰好拒算 FOO_BAR，实际 {[refusal.text() for refusal in closure.refusals]}"


def wgsl_supplier_edge_cell(work):
    """供外名的条目级宏调用不经任何名字点到（拼内核的函数只用宏体里定义的 GPU_FAILURE_POINTS）：着落做成边，宏调用并进闭包，改它里面的码键变。"""
    base = os.path.join(work, "tree")
    write_tree(base, WGSL_SUPPLY_FIXTURE)
    entry = f"{WGSL_SUPPLY_FILE}::supplied_kernel_text"
    members = closure_member_names(base, entry)
    before, before_refused = fixture_key(base, entry)
    edit_file(base, WGSL_SUPPLY_FILE, "GPU_FAILURE_POINT_UNREADABLE = 9,", "GPU_FAILURE_POINT_UNREADABLE = 99,")
    after, _ = fixture_key(base, entry)
    macro_in = any(name.startswith("item_macro failure_points!@") for name in members)
    return macro_in and before is not None and after is not None and before != after, \
        f"闭包里有没有 failure_points! 调用 {macro_in}；改码前键 {before}（拒算 {sorted(before_refused)}）、后 {after}"


def wgsl_single_module_cell(work):
    """闭包读到的几份 .wgsl 合成一个模块：split.wgsl 用的 doubled 在 prelude.wgsl 里（拼它的函数与点名 PRELUDE_SOURCE 的函数是两个），不拒算、doubled 进闭包。"""
    base = os.path.join(work, "tree")
    write_tree(base, WGSL_SUPPLY_FIXTURE)
    closure = closure_of(base, f"{WGSL_SUPPLY_FILE}::split_kernel_text")
    names = sorted(unit.name for unit in closure.order if unit.language == "wgsl")
    return not closure.refusals and "doubled" in names, f"WGSL 成员 {names}，拒算 {[refusal.text() for refusal in closure.refusals]}"


TRAIT_SELF_FIXTURE = {
    "crates/ftrait/Cargo.toml": '[package]\nname = "ftrait"\n',
    "crates/ftrait/src/lib.rs": '''pub trait Shape {
    fn area(&self) -> u32 {
        Self::scale() * 2
    }
    fn scale() -> u32 {
        3
    }
}
pub struct Square;
impl Shape for Square {}
pub fn measure(square: &Square) -> u32 {
    square.area()
}
''',
}


def trait_self_path_cell(work):
    """trait 成员里的 Self::scale() 落到这个 trait 自己的成员 scale：改 scale 的默认实现，measure 的键变。"""
    base = os.path.join(work, "tree")
    write_tree(base, TRAIT_SELF_FIXTURE)
    entry = "crates/ftrait/src/lib.rs::measure"
    members = closure_member_names(base, entry)
    before, _ = fixture_key(base, entry)
    edit_file(base, "crates/ftrait/src/lib.rs", "        3\n", "        4\n")
    after, _ = fixture_key(base, entry)
    return "fn <trait Shape>::scale" in members and before is not None and before != after, \
        f"measure 的闭包里有没有 <trait Shape>::scale {'fn <trait Shape>::scale' in members}；改 scale 前键 {before}、后 {after}"


DOCTEST_STRUCTURE_FIXTURE = {
    "crates/fdoc/Cargo.toml": '[package]\nname = "fdoc"\n',
    "crates/fdoc/src/lib.rs": '''use std::fmt::Debug;
/// ```
/// assert_eq!(fdoc::shown(), 1);
/// ```
pub fn owner() -> u32 {
    10
}
pub fn shown() -> u32 {
    1
}
pub fn debug_text(value: &dyn Debug) -> String {
    format!("{value:?}")
}
''',
}


def doctest_structure_cell(work):
    """文档测试是另编的一个 crate：不连它挂的条目、也不连那个模块的 use；改所挂条目 owner 的函数体，文档测试入口的键不变，改它调的 shown 键变。"""
    base = os.path.join(work, "tree")
    write_tree(base, DOCTEST_STRUCTURE_FIXTURE)
    path = "crates/fdoc/src/lib.rs"
    doctests = [unit.name for unit in RustWorkspace(base).file(path).units if unit.kind == "doctest"]
    if len(doctests) != 1:
        return False, f"要恰好一个文档测试单元，实际 {doctests}"
    entry = f"{path}::{doctests[0]}"
    members = closure_member_names(base, entry)
    before, _ = fixture_key(base, entry)
    edit_file(base, path, "    10\n", "    11\n")
    owner_edited, _ = fixture_key(base, entry)
    edit_file(base, path, "    1\n", "    2\n")
    shown_edited, _ = fixture_key(base, entry)
    linked = sorted(name for name in members if name in ("fn owner",) or name.startswith("use "))
    ok = not linked and None not in (before, owner_edited, shown_edited) and before == owner_edited and owner_edited != shown_edited
    return ok, f"文档测试的闭包里连上了 {linked}；键：改前 {before}、改 owner 后 {owner_edited}、改 shown 后 {shown_edited}"


USE_SELF_FIXTURE = {
    "crates/fuse/Cargo.toml": '[package]\nname = "fuse"\n',
    "crates/fuse/src/lib.rs": '''pub mod store {
    pub fn open() -> u32 {
        1
    }
}
pub mod user {
    use crate::store::{self};
    pub fn reader() -> u32 {
        store::open()
    }
}
''',
}


def use_self_import_cell(work):
    """use crate::store::{self} 引进名字 store：只写 store::open() 的单元连上这条 use，改这条 use 作废。"""
    base = os.path.join(work, "tree")
    write_tree(base, USE_SELF_FIXTURE)
    entry = "crates/fuse/src/lib.rs::user::reader"
    members = closure_member_names(base, entry)
    uses = sorted(name for name in members if name.startswith("use "))
    before, _ = fixture_key(base, entry)
    edit_file(base, "crates/fuse/src/lib.rs", "use crate::store::{self};", "use crate::store::{self, open};")
    after, _ = fixture_key(base, entry)
    return uses == ["use user::use@7"] and before is not None and before != after, f"reader 的闭包里 use {uses}；改那条 use 前键 {before}、后 {after}"


OPEN_TRAIT_ARGUMENT_FIXTURE = {
    "crates/fopen/Cargo.toml": '[package]\nname = "fopen"\n',
    "crates/fopen/src/lib.rs": '''pub trait Sink {
    fn level(&self) -> u32;
}
pub trait Wrap<T: ?Sized> {
    fn wrap(&self) -> u32;
}
pub struct Meter;
impl Sink for Meter {
    fn level(&self) -> u32 {
        1
    }
}
pub struct Holder;
impl Wrap<dyn Sink> for Holder {
    fn wrap(&self) -> u32 {
        2
    }
}
pub fn feed(sink: &dyn Sink) -> u32 {
    sink.level()
}
''',
}
OPEN_TRAIT_ARGUMENT_FILE = "crates/fopen/src/lib.rs"


def open_trait_argument_cell(work):
    """R-开放只按 trait 自己的名字找 impl：入口签名里的 dyn Sink 带进 impl Sink for Meter（改它键变），不带只在 trait 参数里写了 Sink 的 impl Wrap<dyn Sink> for Holder（改它键不变）。"""
    base = os.path.join(work, "tree")
    write_tree(base, OPEN_TRAIT_ARGUMENT_FIXTURE)
    entry = f"{OPEN_TRAIT_ARGUMENT_FILE}::feed"
    members = closure_member_names(base, entry)
    before, _ = fixture_key(base, entry)
    edit_file(base, OPEN_TRAIT_ARGUMENT_FILE, "        2\n", "        3\n")
    holder_edited, _ = fixture_key(base, entry)
    edit_file(base, OPEN_TRAIT_ARGUMENT_FILE, "        1\n", "        4\n")
    meter_edited, _ = fixture_key(base, entry)
    holder_in = sorted(name for name in members if "Holder" in name)
    ok = not holder_in and None not in (before, holder_edited, meter_edited) and before == holder_edited and holder_edited != meter_edited
    return ok, f"feed 的闭包里有 Holder 的 {holder_in}；键：改前 {before}、改 Holder 的 wrap 后 {holder_edited}、改 Meter 的 level 后 {meter_edited}"


BLANKET_FIXTURE = {
    "crates/fblanket/Cargo.toml": '[package]\nname = "fblanket"\n',
    "crates/fblanket/src/lib.rs": '''pub trait Shout {
    fn shout(&self) -> u32;
}
impl<T: Copy> Shout for T {
    fn shout(&self) -> u32 {
        6
    }
}
pub fn yell(value: &u32) -> u32 {
    value.shout()
}
''',
}
BLANKET_FILE = "crates/fblanket/src/lib.rs"


def blanket_method_cell(work):
    """blanket impl（impl<T: Copy> Shout for T）够得到的方法算边：yell 调 value.shout()，闭包里有 blanket impl 的 shout，改它键变。"""
    base = os.path.join(work, "tree")
    write_tree(base, BLANKET_FIXTURE)
    entry = f"{BLANKET_FILE}::yell"
    members = closure_member_names(base, entry)
    before, _ = fixture_key(base, entry)
    edit_file(base, BLANKET_FILE, "        6\n", "        7\n")
    after, _ = fixture_key(base, entry)
    impl_members = sorted(name for name in members if name.startswith("fn <impl"))
    return bool(impl_members) and before is not None and after is not None and before != after, \
        f"yell 的闭包里 impl 成员 {impl_members}；改 blanket impl 的 shout 前键 {before}、后 {after}"


# 4-3k：自身类型里任何深度出现 impl 的泛型参数就是 blanket（Box<T>、Option<Box<T>> 旧判法认不出：头一个名字不是泛型参数）；
# 同一份写法在 code-tooling 的 function-units-two-implementations-agree-green 样本里与 unit-graph 对拍
BLANKET_SHAPES_FIXTURE = {
    "crates/fshapes/Cargo.toml": '[package]\nname = "fshapes"\n',
    "crates/fshapes/src/lib.rs": '''pub trait Weigh {
    fn weigh(&self) -> u32;
}
impl<T> Weigh for (T, u32) {
    fn weigh(&self) -> u32 {
        self.1
    }
}
impl<T> Weigh for [T] {
    fn weigh(&self) -> u32 {
        self.len() as u32
    }
}
impl<T, const N: usize> Weigh for [T; N] {
    fn weigh(&self) -> u32 {
        N as u32
    }
}
impl<T: Weigh> Weigh for Box<T> {
    fn weigh(&self) -> u32 {
        (**self).weigh() + 1
    }
}
impl<T: Weigh> Weigh for Option<Box<T>> {
    fn weigh(&self) -> u32 {
        match self {
            Some(inner) => inner.weigh() + 2,
            None => 0,
        }
    }
}
pub struct Sealed<T>(pub T);
impl<T> Weigh for Sealed<T> {
    fn weigh(&self) -> u32 {
        5
    }
}
pub fn heaviest(pair: &(u8, u32)) -> u32 {
    pair.weigh()
}
''',
}
BLANKET_SHAPES_FILE = "crates/fshapes/src/lib.rs"
# Box、Option 与 u32 在 PRIMITIVE_TYPES 里、本来就算被点名，旧判法下这五份也进闭包；Sealed<T> 没被点名，只有按 blanket 放行才进
BLANKET_SHAPES_SELF_TYPES = ("(T, u32)", "[T]", "[T; N]", "Box<T>", "Option<Box<T>>", "Sealed<T>")


def blanket_shapes_cell(work):
    """heaviest 调 pair.weigh()：六份 impl 的自身类型里都出现了 impl 的泛型参数，都按 blanket 放行、都进闭包；改 Sealed<T> 那一份的 weigh 键变。"""
    base = os.path.join(work, "tree")
    write_tree(base, BLANKET_SHAPES_FIXTURE)
    entry = f"{BLANKET_SHAPES_FILE}::heaviest"
    closure = closure_of(base, entry)
    heads = {unit.name for unit in closure.order if unit.kind == "impl"}
    missing = [shape for shape in BLANKET_SHAPES_SELF_TYPES
               if not any("for" + shape.replace(" ", "") in name.replace(" ", "") for name in heads)]
    before, _ = fixture_key(base, entry)
    edit_file(base, BLANKET_SHAPES_FILE, "        5\n", "        6\n")
    after, _ = fixture_key(base, entry)
    return not missing and before is not None and after is not None and before != after, \
        f"闭包里的 impl 头 {sorted(heads)}，缺 {missing}；改 Sealed<T> 那一份的 weigh 前键 {before}、后 {after}"


# 4-3k：WGSL 外名在同一链接集里别的 include_str! 读进来的 .wgsl 里有定义（入口只读 kernel.wgsl，SHARED_LIMIT 在 shared.wgsl 里）
WGSL_OTHER_FILE_FIXTURE = {
    "crates/fkernels/Cargo.toml": '[package]\nname = "fkernels"\n',
    "crates/fkernels/src/lib.rs": '''pub const KERNEL_SOURCE: &str = include_str!("kernel.wgsl");
pub const SHARED_SOURCE: &str = include_str!("shared.wgsl");
pub fn kernel_text() -> &'static str {
    KERNEL_SOURCE
}
''',
    "crates/fkernels/src/kernel.wgsl": '''@compute @workgroup_size(1)
fn main() {
    let limit = SHARED_LIMIT + 1u;
}
''',
    "crates/fkernels/src/shared.wgsl": "const SHARED_LIMIT: u32 = 7u;\n",
}
WGSL_OTHER_FILE_RUST = "crates/fkernels/src/lib.rs"


def wgsl_other_file_definition_cell(work):
    """kernel_text 的闭包只读 kernel.wgsl，它用的 SHARED_LIMIT 定义在 shared.wgsl：读 shared.wgsl 的 SHARED_SOURCE 并进闭包、shared.wgsl 进模块，
    不拒算；改 shared.wgsl 里那个常量键变。"""
    base = os.path.join(work, "tree")
    write_tree(base, WGSL_OTHER_FILE_FIXTURE)
    entry = f"{WGSL_OTHER_FILE_RUST}::kernel_text"
    closure = closure_of(base, entry)
    names = {f"{unit.language} {unit.name}" for unit in closure.order}
    refused = sorted(refusal.text() for refusal in closure.refusals)
    before, _ = fixture_key(base, entry)
    edit_file(base, "crates/fkernels/src/shared.wgsl", "7u", "8u")
    after, _ = fixture_key(base, entry)
    pulled = {"rust SHARED_SOURCE", "wgsl SHARED_LIMIT"} <= names
    return pulled and not refused and before is not None and after is not None and before != after, \
        f"闭包里有 SHARED_SOURCE 与 SHARED_LIMIT：{pulled}（成员 {sorted(names)}）；拒算 {refused}；改常量前键 {before}、后 {after}"


# 4-3h：const _ 不是 R-裸名的着落（gpu 特性下才编的 const _ 编译期断言被 let _ = …、_ => 里的 _ 拉进闭包，带不带 gpu 的两趟键就分了）；
# include_str! 读根外的文件（工作区在根下一层，读仓根下的登记文件）照 crates/singlefs-unit-graph 收成数据单元
UNDERSCORE_FIXTURE = {
    "crates/funderscore/Cargo.toml": '[package]\nname = "funderscore"\n\n[features]\ngpu = []\n',
    "crates/funderscore/src/lib.rs": '''pub const LIMIT: u32 = 4;
#[cfg(feature = "gpu")]
const _: () = assert!(LIMIT == 4);
pub fn pick(value: u32) -> u32 {
    let _ = value;
    match value {
        0 => 1,
        _ => LIMIT,
    }
}
''',
}
UNDERSCORE_FILE = "crates/funderscore/src/lib.rs"


def underscore_const_cell(work):
    """const _ 没有名字：pick 里的 let _ 与 _ => 不把 gpu 特性下的 const _ 拉进闭包，带不带 gpu 特性两种目标下两把键都相同、闭包里没有 const _。"""
    base = os.path.join(work, "tree")
    write_tree(base, UNDERSCORE_FIXTURE)
    entry = f"{UNDERSCORE_FILE}::pick"
    without = cfg_keys(base, entry, [])
    with_gpu = cfg_keys(base, entry, ["funderscore/gpu"])
    workspace = RustWorkspace(base)
    _language, path, qualified = parse_entry(base, entry)
    unit, target_id = workspace.find_entry(path, qualified, None)
    closure = rust_closure_of(workspace, unit, target_id, None, False, ["funderscore/gpu"], False)
    underscores = sorted(f"{member.kind} {member.name}" for member in closure.order if not member.data and member.short == "_")
    return None not in without + with_gpu and without == with_gpu and not underscores, \
        f"不带 gpu {without}、带 gpu {with_gpu}；带 gpu 时闭包里的 _ 单元 {underscores}"


OUTSIDE_INCLUDE_FIXTURE = {
    "workspace/crates/finclude/Cargo.toml": '[package]\nname = "finclude"\n',
    "workspace/crates/finclude/src/lib.rs": '''pub fn threads() -> usize {
    include_str!("../../../../thread-default").trim().len()
}
pub fn missing() -> usize {
    include_str!("../../../../not-there").len()
}
''',
    "thread-default": "31\n",
}
OUTSIDE_INCLUDE_FILE = "crates/finclude/src/lib.rs"


def outside_include_cell(work):
    """根是 <树>/workspace：include_str! 读根外 <树>/thread-default 的入口出键、改那份文件键变；读根外不在的文件拒算 include-missing。"""
    tree = os.path.join(work, "tree")
    write_tree(tree, OUTSIDE_INCLUDE_FIXTURE)
    base = os.path.join(tree, "workspace")
    before, before_refused = fixture_key(base, f"{OUTSIDE_INCLUDE_FILE}::threads")
    edit_file(tree, "thread-default", "31\n", "16\n")
    after, after_refused = fixture_key(base, f"{OUTSIDE_INCLUDE_FILE}::threads")
    missing, missing_refused = fixture_key(base, f"{OUTSIDE_INCLUDE_FILE}::missing")
    ok = None not in (before, after) and before != after and missing is None and "include-missing" in missing_refused
    return ok, (f"threads：改前键 {before}（拒算 {sorted(before_refused)}）、改根外文件后 {after}（拒算 {sorted(after_refused)}）；"
                f"missing：键 {missing}、拒算 {sorted(missing_refused)}")


def unit_divergence_cells():
    return [
        ("cfg 成分：闭包没引到 gpu 的入口，带不带 gpu 特性两种目标下键相同", cfg_unreferenced_cell),
        ("cfg 成分：闭包里有 cfg!、语句上的 #[cfg] 或 #[cfg_attr] 引到 gpu 的入口，带不带 gpu 特性键不同", cfg_referenced_cell),
        ("WGSL 外名：条目级宏调用定义的名字以 _ 加外名收尾就有着落", wgsl_defined_suffix_cell),
        ("WGSL 外名：首段只落在无关的串里、其余那一段哪儿都没有的拒算", wgsl_prefix_remainder_cell),
        ("WGSL 外名的着落做成边：只靠外名点到的条目级宏调用并进闭包，改它里面的码键变", wgsl_supplier_edge_cell),
        ("WGSL：闭包读到的几份 .wgsl 合成一个模块，名字跨文件解", wgsl_single_module_cell),
        ("R-路径：trait 成员里的 Self::x 落到这个 trait 自己的成员 x", trait_self_path_cell),
        ("R-结构：文档测试不连所挂的条目与那个模块的 use", doctest_structure_cell),
        ("R-结构：use a::{self} 引进名字 a，只写 a::f() 的单元连上这条 use", use_self_import_cell),
        ("R-开放：dyn Trait 只带 Trait 自己的 impl，不带只在 trait 参数里写了它的 impl", open_trait_argument_cell),
        ("R-方法：blanket impl 够得到的方法算边，改它作废", blanket_method_cell),
        ("R-裸名：const _ 没有名字，let _ 与 _ => 不把 gpu 特性下的 const _ 拉进闭包，带不带 gpu 特性键相同", underscore_const_cell),
        ("include_str! 读根外的文件：读得到收成数据单元、改它键变，读不到拒算 include-missing", outside_include_cell),
        ("R-方法：自身类型里出现泛型参数的 impl（(T, u32)、[T]、[T; N]、Box<T>、Option<Box<T>>、Sealed<T>）都按 blanket 放行，改 Sealed<T> 那一份作废",
         blanket_shapes_cell),
        ("WGSL 外名在链接集里别的 include_str! 读进来的 .wgsl 里有定义：读它的 Rust 单元与那份 .wgsl 拉进闭包，改那个常量作废", wgsl_other_file_definition_cell),
    ]


def rust_unbalanced_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, RUST_FIXTURE)
    edit_file(base, "crates/fextra/src/lib.rs", "    Shade::Dark as u32 + 1\n}", "    Shade::Dark as u32 + 1\n")
    key, forms = fixture_key(base, f"{RUST_FIXTURE_ENTRY_FILE}::judge_listing")
    return key is None and "unbalanced-brackets" in forms, f"要拒算 unbalanced-brackets，实际 {sorted(forms)}"


def wgsl_undefined_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, RUST_FIXTURE)
    edit_file(base, "crates/fextra/src/common.wgsl", "x + GENERATED_LIMIT", "x + MISSING_EVERYWHERE")
    key, forms = fixture_key(base, f"{RUST_FIXTURE_ENTRY_FILE}::judge_kernel")
    return key is None and "wgsl-undefined-name" in forms, f"要拒算 wgsl-undefined-name，实际 {sorted(forms)}"


def python_rta_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, PYTHON_FIXTURE)
    closure = closure_of(base, "pkg/a.py::use_reader")
    names = {unit.name for unit in closure.order}
    return "Reader.read" in names and "Writer.read" not in names, f"闭包里方法：{sorted(name for name in names if '.' in name)}"


def python_spec_cell(work):
    ok, detail = compare_cell(PYTHON_FIXTURE, "pkg/a.py::entry", "pkg/b.py", "    return 2", "    return 20", "changed")(work)
    return ok, detail


def shell_heredoc_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, SHELL_FIXTURE)
    index = ShellIndex(base, ["s/main.sh"])
    shell_file = index.files["s/main.sh"]
    emits = shell_file.functions.get("emit", [])
    if len(emits) != 1 or "second" not in emits[0].body or "second" in shell_file.script_body.body:
        return False, f"emit 单元 {len(emits)} 个，含 second：{[('second' in unit.body) for unit in emits]}"
    before, _ = fixture_key(base, "s/main.sh::emit")
    beta_before, _ = fixture_key(base, "s/main.sh::cell_beta")
    edit_file(base, "s/main.sh", "second\n", "second changed\n")
    after, _ = fixture_key(base, "s/main.sh::emit")
    beta_after, _ = fixture_key(base, "s/main.sh::cell_beta")
    return before != after and beta_before == beta_after and before is not None, "here-doc 正文改了：emit 的键要变、cell_beta 的不变"


def shell_names_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, SHELL_FIXTURE)
    closure = closure_of(base, "s/main.sh")
    names = {unit.name for unit in closure.order if unit.file == "s/main.sh"}
    return "cell_alpha" in names and "cell_beta" not in names, f"main.sh 的闭包里：{sorted(names)}"


def shell_folding_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, SHELL_FIXTURE)
    closure = closure_of(base, "s/main.sh")
    files = {unit.file for unit in closure.order}
    key, refusals = closure_key(closure)
    return {"s/lib.sh", "s/child.sh"} <= files and key is not None, f"闭包碰到的文件 {sorted(files)}，拒算 {[refusal.form for refusal in refusals]}"


def no_refusal_cell(fixture, entry):
    def judge(work):
        base = os.path.join(work, "tree")
        write_tree(base, fixture)
        key, forms = fixture_key(base, entry)
        return key is not None, f"不该拒算，实际拒算 {sorted(forms)}"
    return judge


LOADER_FILES = (".claude/scripts/preflight.sh", ".claude/scripts/project_preflight.py")
CANONICAL_FILES = ("preflight.sh", "preflight.py", "proc.py")   # preflight.py 按名字载入同目录的 proc.py（load_package_module）


def loader_tree(base, canonical):
    """在 base 下拷一棵只有 .claude/ 的小树：两份装载器，canonical 给了就连同规范副本的 preflight.sh / preflight.py。"""
    repository = os.path.normpath(os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", ".."))
    for path in LOADER_FILES:
        os.makedirs(os.path.join(base, os.path.dirname(path)), exist_ok=True)
        shutil.copyfile(os.path.join(repository, path), os.path.join(base, path))
    if canonical is not None:
        target = os.path.join(base, ".claude", "singlefs-ai-sop", "scripts")
        os.makedirs(target, exist_ok=True)
        for name in CANONICAL_FILES:
            shutil.copyfile(os.path.join(canonical, name), os.path.join(target, name))


def loader_resolutions(tree, work):
    """→ (source 装载器之后 PREFLIGHT_PROGRAM 指的文件, import 装载器之后 preflight 模块的文件, 出错说明)，都取真实路径。"""
    environment = {"PATH": os.environ.get("PATH", "/usr/bin:/bin"), "HOME": work, "LC_ALL": "C", "PYTHONDONTWRITEBYTECODE": "1"}
    shell = subprocess.run(["bash", "--norc", "--noprofile", "-c", 'source "$1/.claude/scripts/preflight.sh" && printf "%s\\n" "$PREFLIGHT_PROGRAM"',
                            "loader", tree], cwd=work, env=environment, capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=60)
    python = subprocess.run([sys.executable, "-B", "-c", "import sys; sys.path.insert(0, sys.argv[1]); import project_preflight; "
                             "print(project_preflight.shared_preflight_module('loader').__file__)", os.path.join(tree, ".claude", "scripts")],
                            cwd=work, env=environment, capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=60)
    got = [os.path.realpath(completed.stdout.strip()) if completed.returncode == 0 and completed.stdout.strip() else None for completed in (shell, python)]
    return got[0], got[1], f"bash 退 {shell.returncode}：{shell.stderr.strip()[:200]}；python 退 {python.returncode}：{python.stderr.strip()[:200]}"


def preflight_loader_cell(work):
    """只拷了 .claude/ 的小树：source / import 两份装载器都解到这棵树里的规范副本（不靠 git、不靠主仓），两份装载器的闭包不拒算、收进副本。"""
    canonical = sys.modules["project_preflight"].shared_preflight_directory()
    if canonical is None:
        return False, "本仓找不到规范副本（.claude/singlefs-ai-sop/scripts/），造不出小树"
    tree = os.path.join(work, "tree")
    loader_tree(tree, canonical)
    want = os.path.realpath(os.path.join(tree, ".claude", "singlefs-ai-sop", "scripts", "preflight.py"))
    shell, python, detail = loader_resolutions(tree, work)
    if shell != want or python != want:
        return False, f"要都解到 {want}，source 解到 {shell}，import 解到 {python}（{detail}）"
    for entry, member in ((".claude/scripts/preflight.sh", ".claude/singlefs-ai-sop/scripts/preflight.sh"),
                          (".claude/scripts/project_preflight.py::shared_preflight_module", ".claude/singlefs-ai-sop/scripts/preflight.py")):
        closure = closure_of(tree, entry)
        files = {unit.file for unit in closure.order}
        if closure.refusals or member not in files:
            return False, f"{entry} 的闭包拒算 {sorted(refusal.text() for refusal in closure.refusals)}，碰到的文件 {sorted(files)}"
    return True, ""


def preflight_worktree_cell(work):
    """git worktree 里没有规范副本（.gitignore 挡着）：source / import 两份装载器都解到主仓那一份。"""
    canonical = sys.modules["project_preflight"].shared_preflight_directory()
    if canonical is None:
        return False, "本仓找不到规范副本（.claude/singlefs-ai-sop/scripts/），造不出主仓"
    main, linked = os.path.join(work, "main"), os.path.join(work, "linked")
    loader_tree(main, canonical)
    with open(os.path.join(main, ".gitignore"), "w", encoding="utf-8") as handle:
        handle.write("/.claude/singlefs-ai-sop/\n")
    environment = {"PATH": os.environ.get("PATH", "/usr/bin:/bin"), "HOME": work, "LC_ALL": "C", "GIT_CONFIG_NOSYSTEM": "1"}
    git = ["git", "-c", "user.name=selftest", "-c", "user.email=selftest@example.invalid", "-c", "commit.gpgsign=false", "-c", "init.defaultBranch=main"]
    for step in (["init", "-q", main], ["-C", main, "add", ".gitignore", ".claude/scripts"], ["-C", main, "commit", "-q", "-m", "loaders"],
                 ["-C", main, "worktree", "add", "-q", "--detach", linked]):
        completed = subprocess.run(git + step, env=environment, capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=60)
        if completed.returncode != 0:
            return False, f"git {' '.join(step[:3])} 退 {completed.returncode}：{completed.stderr.strip()[:200]}"
    if os.path.exists(os.path.join(linked, ".claude", "singlefs-ai-sop")):
        return False, "worktree 里不该有规范副本，样本没造对"
    want = os.path.realpath(os.path.join(main, ".claude", "singlefs-ai-sop", "scripts", "preflight.py"))
    shell, python, detail = loader_resolutions(linked, work)
    return shell == want and python == want, f"要都解到主仓的 {want}，source 解到 {shell}，import 解到 {python}（{detail}）"


def key_toolchain_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, PYTHON_FIXTURE)
    closure = closure_of(base, "pkg/a.py::entry")
    plain, _ = closure_key(closure)
    with_toolchain, _ = closure_key(closure, toolchain=["python 3.12.3"])
    return plain is not None and plain != with_toolchain, "加一行工具链键要变"


def key_external_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, SHELL_FIXTURE)
    closure = closure_of(base, "s/tools.sh")
    missing, refusals = closure_key(closure)
    given, _ = closure_key(closure, externals={"cargo": ("0" * 64, "cargo 1.90.0")})
    return missing is None and any(refusal.form == "external-program-unkeyed" for refusal in refusals) and given is not None, \
        f"没给 cargo 时 {[refusal.form for refusal in refusals]}；给了之后键 {given}"


def selftest_cells():
    cells = []
    for name, entry, path, old, new, want in RUST_FIXTURE_CASES:
        cells.append((name, compare_cell(RUST_FIXTURE, f"{RUST_FIXTURE_ENTRY_FILE}::{entry}", path, old, new, want)))
    cells += [
        ("改文档注释里的代码块：文档测试入口作废", rust_doctest_cell),
        ("认不出的 cfg 谓词拒算", refused_cell(RUST_FIXTURE, f"{RUST_FIXTURE_ENTRY_FILE}::judge_speed", "cfg-unrecognized",
                                         ("crates/fextra/src/lib.rs", '#[cfg(feature = "fast")]', "#[cfg(accessible(fast))]"))),
        ("括号配不上的文件整份拒算", rust_unbalanced_cell),
        ("WGSL 里没定义、Rust 串里也没有的名字拒算", wgsl_undefined_cell),
        ("dump 的单元行形态（带路径的宏、verus! 两个 fn、impl 头与方法）", rust_dump_cell),
        ("python：只改注释不作废", compare_cell(PYTHON_FIXTURE, "pkg/a.py::entry", "pkg/a.py", "    # 只是注释\n", "    # 只是注释，改了措辞\n", "same")),
        ("python：改文档串不作废", compare_cell(PYTHON_FIXTURE, "pkg/a.py::entry", "pkg/a.py", '"""helper 的文档串"""', '"""helper 的文档串，改了"""', "same")),
        ("python：有单元读 __doc__ 时改文档串作废", compare_cell(PYTHON_FIXTURE, "pkg/a.py::described", "pkg/a.py", '"""helper 的文档串"""',
                                                         '"""helper 的文档串，改了"""', "changed")),
        ("python：getattr 变量名拒算", refused_cell(PYTHON_FIXTURE, "pkg/a.py::dynamic", "getattr-variable")),
        ("python：getattr(<标准库模块>, 变量名) 不拒算（够不到仓里的代码）", no_refusal_cell(PYTHON_FIXTURE, "pkg/f.py::patched_os_function")),
        ("python：os.environ.get(…) or <缺省路径> 按缺省折，连到那份脚本", compare_cell(PYTHON_FIXTURE, "pkg/h.py::stop", "pkg/c.py", "    return 5", "    return 50", "changed")),
        ("python：自己写出字面文本再起的脚本不拒算", no_refusal_cell(PYTHON_FIXTURE, "pkg/h.py::run_stub")),
        ("python：for 循环变量取遍元组里的目录，插进 sys.path 不拒算、各目录里的同名模块都连", compare_cell(PYTHON_FIXTURE, "pkg/i.py::load_named", "pkg/tools/tool.py",
                                                                                       "    return 7", "    return 70", "changed")),
        ("python：推导式里的循环变量不算函数里的赋值，for 取遍元组拼出的脚本连上", compare_cell(PYTHON_FIXTURE, "pkg/i.py::load_each", "pkg/b.py", "    return 2",
                                                                           "    return 20", "changed")),
        ("python：os.path.join(<仓里的目录>, <折不出的文件名>) 起的脚本连到那个目录里每一份", compare_cell(PYTHON_FIXTURE, "pkg/k.py::run_tool_named", "pkg/tools/main_tool.py",
                                                                                  "    return 9", "    return 90", "changed")),
        ("python：路径表（字典字面量）[键] 取遍表里的路径，spec 载入连上", compare_cell(PYTHON_FIXTURE, "pkg/j.py::load_named", "pkg/b.py", "    return 2",
                                                                      "    return 20", "changed")),
        ("python：importlib.import_module(<模块级常量>) 折出模块名", compare_cell(PYTHON_FIXTURE, "pkg/i.py::load_by_constant", "pkg/more_tools/tool.py",
                                                                            "    return 8", "    return 80", "changed")),
        ("python：表[路径] = … 不当成重新绑定路径，spec 载入照样折得出", compare_cell(PYTHON_FIXTURE, "pkg/g.py::cached_b", "pkg/b.py", "    return 2", "    return 20",
                                                                    "changed")),
        ("python：spec_from_file_location 字面路径折叠出跨模块边", python_spec_cell),
        ("python：spec 载入的模块里够不到的函数改了不作废", compare_cell(PYTHON_FIXTURE, "pkg/a.py::entry", "pkg/b.py", "    return 3", "    return 30", "same")),
        ("python：subprocess 起 sys.executable 字面路径连到那份脚本的程序入口", compare_cell(PYTHON_FIXTURE, "pkg/a.py::launch", "pkg/c.py", "    return 5", "    return 50", "changed")),
        ("python：obj.方法 按闭包里出现过的类收窄", python_rta_cell),
        ("shell：here-doc 正文里有一行 `};` 不许被切开", shell_heredoc_cell),
        ("shell：函数名当参数传照样连上", shell_names_cell),
        ("shell：`bash \"$x\"` 折不出拒算", refused_cell(SHELL_FIXTURE, "s/dynamic.sh", "shell-path-unfolded")),
        ("shell：$(cd \"$(dirname \"${BASH_SOURCE[0]}\")\" && pwd) 折叠出 source 与 bash 的边", shell_folding_cell),
        ("shell：bash -n 不过的整份拒算", refused_cell(SHELL_FIXTURE, "s/broken.sh", "bash-syntax")),
        ("shell：只改注释不作废", compare_cell(SHELL_FIXTURE, "s/main.sh::cell_beta", "s/main.sh", "set -euo pipefail\n", "# 一行新注释\nset -euo pipefail\n", "same")),
        ("shell：脚本体只改注释不作废", compare_cell(SHELL_FIXTURE, "s/main.sh", "s/main.sh", "set -euo pipefail\n", "# 一行新注释\nset -euo pipefail\n", "same")),
        ("shell：顶层变量里 here-doc 的正文改了，读它的函数作废", compare_cell(SHELL_FIXTURE, "s/main.sh::use_library", "s/main.sh", 'print("library")',
                                                                'print("library changed")', "changed")),
        ("键：工具链行进键", key_toolchain_cell),
        ("键：点名了 cargo 而调用方没给 --external 拒算、给了出键", key_external_cell),
        ("shell：`bash \"$tree/<仓里的相对路径>\"` 连到仓里那一份", compare_cell(SHELL_FIXTURE, "s/copies.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        ("python：os.path.join(<折不出的目录>, <仓里的相对路径>) 连到仓里那一份", compare_cell(PYTHON_FIXTURE, "pkg/d.py::run_copy", "pkg/c.py", "    return 5",
                                                                                "    return 50", "changed")),
        ("shell：git rev-parse --git-common-dir 折成根，经它 source 的库连上", compare_cell(SHELL_FIXTURE, "s/common.sh", "s/lib.sh", "echo lib", "echo lib changed", "changed")),
        ("python：git rev-parse --git-common-dir 经无参函数的返回值折成根，插进 sys.path 的目录不拒算", no_refusal_cell(PYTHON_FIXTURE, "pkg/d.py::main_repository_value")),
        ("shell：重定向到变量路径不拒算", no_refusal_cell(SHELL_FIXTURE, "s/redirect.sh")),
        ("shell：把路径当参数传给自己的函数不拒算", no_refusal_cell(SHELL_FIXTURE, "s/arguments.sh")),
        ("python：写数据的路径与传给 subprocess 的参数不拒算", no_refusal_cell(PYTHON_FIXTURE, "pkg/d.py::write_data")),
        ("shell：自己用 cat <<词 写出来再起的脚本不拒算、正文里装的连上", compare_cell(SHELL_FIXTURE, "s/generator.sh", "s/child.sh", "echo child",
                                                                       "echo child changed", "changed")),
        ("shell：bash -c '…\"$1\"…' <名字> <路径> 里的 $1 换成那个路径", compare_cell(SHELL_FIXTURE, "s/inline.sh", "s/lib.sh", "echo lib", "echo lib changed", "changed")),
        ("shell：自己 cp 出来再起的脚本连到源那一份", compare_cell(SHELL_FIXTURE, "s/selfcopy.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        ("shell：套在外层函数里的函数看得见外层的 local", compare_cell(SHELL_FIXTURE, "s/nested.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        ("shell：cd 过去的目录折出几个，相对路径在每一个底下都连", compare_cell(SHELL_FIXTURE, "s/versions.sh", "s/beta/tool.sh", "echo beta tool",
                                                              "echo beta tool changed", "changed")),
        ("python：分派表（字典字面量）里的函数改了作废", compare_cell(PYTHON_FIXTURE, "pkg/d.py::dispatch", "pkg/d.py", "    return 12", "    return 13", "changed")),
        ("python：import 进来的模块常量（module.NAME）折进路径，插进 sys.path 的目录连上", compare_cell(PYTHON_FIXTURE, "pkg/e.py::load_tool", "pkg/tools/tool.py",
                                                                                     "    return 7", "    return 70", "changed")),
        ("preflight 装载器：只拷了 .claude/ 的小树里 source 与 import 都解到树里的规范副本，两份装载器的闭包不拒算", preflight_loader_cell),
        ("preflight 装载器：worktree 里没有规范副本时 source 与 import 解到主仓那一份", preflight_worktree_cell),
    ]
    cells += [(name, judge) for name, judge in silent_form_cells()]
    cells += alignment_cells()
    cells += closure_rule_cells()
    cells += unit_divergence_cells()
    return cells


def silent_form_cells():
    """以前既不连边也不拒算的几种写法：每种一格连边、一格拒算（或「认出、不装仓里的代码」），各配弄坏开关（BREAK_CELLS）。"""
    helper = ("s/py/helper_mod.py", "    return 1", "    return 10", "changed")
    child = ("s/child.sh", "echo child", "echo child changed", "changed")
    return [
        ("shell：python3 -c '<字面串>' 当内联脚本切，sys.argv 折成后面的词，连到它 import 的仓内模块", compare_cell(SHELL_FIXTURE, "s/inline_python.sh", *helper)),
        ("shell：python3 - <<'PY' 喂的 here-doc 当内联脚本切，连到它 import 的仓内模块", compare_cell(SHELL_FIXTURE, "s/heredoc_python.sh", *helper)),
        ("shell：{ python3 /dev/fd/3 …; } 3<<'PY'（重定向写在组后面）照内联脚本切", compare_cell(SHELL_FIXTURE, "s/group.sh", *helper)),
        ("shell：python3 -c / bash -c 的串带折不出的展开拒算 inline-script-not-literal", refused_cell(SHELL_FIXTURE, "s/inline_expanded.sh", INLINE_FORM)),
        ("shell：bash -c \"$串\" 的串折得出就切开再认，连到它起的脚本", compare_cell(SHELL_FIXTURE, "s/inline_bash_folded.sh", *child)),
        ("shell：一条命令里第二个解释器（bash 包装脚本 N python3 x.py）照第一个的规则认，连到 x.py", compare_cell(SHELL_FIXTURE, "s/second.sh", "s/py/tool.py",
                                                                                         "    return 2", "    return 20", "changed")),
        ("shell：起同目录脚本连边（写在命令替换里：x=\"$(bash 同目录/y.sh)\"）", compare_cell(SHELL_FIXTURE, "s/substitution.sh", *child)),
        ("shell：< <(命令 \"$x\") 里的命令照命令认，前面那个 < 不吞掉它的第一个词", no_refusal_cell(SHELL_FIXTURE, "s/process_substitution.sh")),
        ("shell：die 消息里出现 qemu-system-x86_64 不算外部程序", no_refusal_cell(SHELL_FIXTURE, "s/message.sh")),
        ("shell：python3 -m <仓内模块> 连到那份模块的程序入口", compare_cell(SHELL_FIXTURE, "s/module.sh", "s/py/tool.py", "    return 2", "    return 20", "changed")),
        ("shell：命令位置上折不出的程序拒算 shell-command-unfolded", refused_cell(SHELL_FIXTURE, "s/head.sh", "shell-command-unfolded")),
        ("shell：包装命令（timeout 60）后面直接执行的脚本连上", compare_cell(SHELL_FIXTURE, "s/wrapped.sh", *child)),
        ("shell：[[ … || … ]]、(( … ))、case 分支的模式、数组字面量里的词不当命令", no_refusal_cell(SHELL_FIXTURE, "s/structure.sh")),
        ("shell：\"${数组[@]}\" 当命令时按数组的字面赋值摊开，连到它起的脚本", compare_cell(SHELL_FIXTURE, "s/array.sh", *child)),
        ("shell：python3 -c \"$(打印正文的函数)\" 折出函数打的那几行当内联脚本", compare_cell(SHELL_FIXTURE, "s/capture.sh", *helper)),
        ("shell：文件头登记的 check 判据（preflight 开跑前跑）照内联 bash 认，连到它起的脚本", compare_cell(SHELL_FIXTURE, "s/declared.sh", *child)),
        ("shell：内联 python exec sys.argv[1:] 时后面那几个词照命令认", compare_cell(SHELL_FIXTURE, "s/argv_exec.sh", *child)),
        ("python：[sys.executable, \"-c\", <字面串>, …] 当内联脚本切，连到它 import 的仓内模块", compare_cell(PYTHON_FIXTURE, "pkg/m.py::run_inline", "pkg/b.py",
                                                                                         "    return 2", "    return 20", "changed")),
        ("python：-c 的串折不出拒算 inline-script-not-literal", refused_cell(PYTHON_FIXTURE, "pkg/m.py::run_dynamic", INLINE_FORM)),
        ("python：变量解释器折不出拒算 subprocess-command-unfolded", refused_cell(PYTHON_FIXTURE, "pkg/m.py::run_unknown", COMMAND_FORM)),
        ("python：命令是 [解释器] + 参数（按调用处给的列表折）连到那份脚本", compare_cell(PYTHON_FIXTURE, "pkg/m.py::launch_tool", "pkg/c.py", "    return 5",
                                                                        "    return 50", "changed")),
        ("python：认不出的命令（对象的属性）拒算 subprocess-command-unfolded", refused_cell(PYTHON_FIXTURE, "pkg/m.py::launch_job", COMMAND_FORM)),
        ("python：-m unittest <测试模块> 整份连上（测试方法改了作废）", compare_cell(PYTHON_FIXTURE, "pkg/m.py::run_tests", "pkg/test_sample.py", "        return 3",
                                                                    "        return 30", "changed")),
        ("python：shutil.which(…) 与 os.path.join(<折不出的目录>, \"bin/x\") 起的根外程序不拒算", no_refusal_cell(PYTHON_FIXTURE, "pkg/m.py::version_of")),
        ("python：内联脚本起 sys.argv[k]（取的是起它那份脚本的 \"$1\"）不拒算：程序是调用处给的词", no_refusal_cell(SHELL_FIXTURE, "s/positional_tool.sh")),
        ("python：上游 preflight 跑文件头登记的判据（bash -c declaration.command）不拒算，判据从声明它的脚本连",
         no_refusal_cell(PYTHON_FIXTURE, ".claude/singlefs-ai-sop/scripts/preflight.py::run_check")),
        ("python：命令里的 *选项[\"键\"] 折回调用处给的字典字面量，连到那份脚本", compare_cell(PYTHON_FIXTURE, "pkg/m.py::run_default", "pkg/c.py", "    return 5",
                                                                               "    return 50", "changed")),
        ("python：经分派表 表[键](…) 调到的函数，命令参数按那一处调用折", compare_cell(PYTHON_FIXTURE, "pkg/m.py::launch_from_table", "pkg/c.py", "    return 5",
                                                                     "    return 50", "changed")),
    ]


BREAK_CELLS = (
    ("rust-open-entry-traits-ignored", ("开放入口：参数是 dyn", "R-开放：dyn Trait 只带 Trait 自己的 impl，不带只在 trait 参数里写了它的 impl")),
    ("rust-dyn-traits-open-every-implementation", ("dyn 分派：闭包里的 dyn 不连没被点名的另一个 impl，改它不作废",)),
    ("rust-constants-dropped", ("关联常量（经泛型）", "函数指针表", "数组长度里的常量")),
    ("rust-macros-dropped", ("声明宏展开出的函数", "带路径的条目级宏调用定义的枚举：改了作废")),
    ("rust-trait-impl-edges-dropped", ("运算符重载", "Drop 隐式调用", "关联常量（经泛型）")),
    ("rust-verus-segments-ignored", ("verus! 里 ensures 带花括号：下一个函数照样切开，改它作废", "verus! 里够不到的函数改了不作废")),
    ("rust-cfg-ignored", ("cfg(feature) 关着的单元改了不作废", "认不出的 cfg 谓词拒算")),
    ("rust-doc-comments-kept", ("只改文档注释不作废", "改文档注释里的代码块：入口 fn 不作废")),
    ("wgsl-raw-text", ("WGSL 只改注释不作废", "WGSL 够不到的函数改了不作废")),
    ("python-comments-kept", ("python：只改注释不作废",)),
    ("python-docstrings-kept", ("python：改文档串不作废",)),
    ("python-docstring-readers-ignored", ("python：有单元读 __doc__ 时改文档串作废",)),
    ("python-getattr-allowed", ("python：getattr 变量名拒算",)),
    ("python-spec-paths-ignored", ("python：spec_from_file_location 字面路径折叠出跨模块边",)),
    ("python-attributes-unnarrowed", ("python：obj.方法 按闭包里出现过的类收窄",)),
    ("shell-heredoc-split", ("shell：here-doc 正文里有一行 `};` 不许被切开",)),
    ("shell-names-command-position-only", ("shell：函数名当参数传照样连上",)),
    ("shell-unfolded-paths-ignored", ("shell：`bash \"$x\"` 折不出拒算",)),
    ("shell-raw-text", ("shell：脚本体只改注释不作废",)),
    ("external-programs-unchecked", ("键：点名了 cargo 而调用方没给 --external 拒算、给了出键",)),
    ("rust-format-captures-ignored", ("format! 串里内联捕获的常量改了作废", "WGSL 够得到的函数改了作废")),
    ("shell-variable-first-line-only", ("shell：顶层变量里 here-doc 的正文改了，读它的函数作废",)),
    ("tree-copy-tails-ignored", ("shell：`bash \"$tree/<仓里的相对路径>\"` 连到仓里那一份", "python：os.path.join(<折不出的目录>, <仓里的相对路径>) 连到仓里那一份")),
    ("git-repository-paths-unfolded", ("shell：git rev-parse --git-common-dir 折成根，经它 source 的库连上",
                                       "python：git rev-parse --git-common-dir 经无参函数的返回值折成根，插进 sys.path 的目录不拒算",
                                       "preflight 装载器：只拷了 .claude/ 的小树里 source 与 import 都解到树里的规范副本，两份装载器的闭包不拒算")),
    ("python-call-returns-unfolded", ("python：git rev-parse --git-common-dir 经无参函数的返回值折成根，插进 sys.path 的目录不拒算",
                                      "preflight 装载器：只拷了 .claude/ 的小树里 source 与 import 都解到树里的规范副本，两份装载器的闭包不拒算")),
    ("shell-redirect-targets-loaded", ("shell：重定向到变量路径不拒算",)),
    ("shell-path-arguments-loaded", ("shell：把路径当参数传给自己的函数不拒算",)),
    ("python-command-arguments-loaded", ("python：写数据的路径与传给 subprocess 的参数不拒算",)),
    ("shell-generated-scripts-ignored", ("shell：自己用 cat <<词 写出来再起的脚本不拒算、正文里装的连上",)),
    ("shell-inline-positionals-ignored", ("shell：bash -c '…\"$1\"…' <名字> <路径> 里的 $1 换成那个路径",)),
    ("shell-copied-scripts-ignored", ("shell：自己 cp 出来再起的脚本连到源那一份",)),
    ("shell-enclosing-locals-ignored", ("shell：套在外层函数里的函数看得见外层的 local",)),
    ("shell-cd-first-directory-only", ("shell：cd 过去的目录折出几个，相对路径在每一个底下都连",)),
    ("python-dispatch-tables-ignored", ("python：分派表（字典字面量）里的函数改了作废",)),
    ("python-module-constants-unfolded", ("python：import 进来的模块常量（module.NAME）折进路径，插进 sys.path 的目录连上",)),
    ("python-getattr-stdlib-refused", ("python：getattr(<标准库模块>, 变量名) 不拒算（够不到仓里的代码）",)),
    ("python-subscript-targets-rebind", ("python：表[路径] = … 不当成重新绑定路径，spec 载入照样折得出",)),
    ("python-environment-defaults-unfolded", ("python：os.environ.get(…) or <缺省路径> 按缺省折，连到那份脚本",)),
    ("python-written-scripts-ignored", ("python：自己写出字面文本再起的脚本不拒算",)),
    ("python-loop-variables-unfolded", ("python：for 循环变量取遍元组里的目录，插进 sys.path 不拒算、各目录里的同名模块都连",)),
    ("python-imports-first-directory-only", ("python：for 循环变量取遍元组里的目录，插进 sys.path 不拒算、各目录里的同名模块都连",)),
    ("python-import-names-unfolded", ("python：importlib.import_module(<模块级常量>) 折出模块名",)),
    ("python-path-tables-unfolded", ("python：路径表（字典字面量）[键] 取遍表里的路径，spec 载入连上",)),
    ("python-comprehension-scope-leaks", ("python：推导式里的循环变量不算函数里的赋值，for 取遍元组拼出的脚本连上",)),
    ("python-directory-wildcards-ignored", ("python：os.path.join(<仓里的目录>, <折不出的文件名>) 起的脚本连到那个目录里每一份",)),
    ("inline-scripts-ignored", ("shell：python3 -c '<字面串>' 当内联脚本切，sys.argv 折成后面的词，连到它 import 的仓内模块",
                                "shell：python3 - <<'PY' 喂的 here-doc 当内联脚本切，连到它 import 的仓内模块",
                                "shell：python3 -c / bash -c 的串带折不出的展开拒算 inline-script-not-literal",
                                "shell：bash -c \"$串\" 的串折得出就切开再认，连到它起的脚本",
                                "python：[sys.executable, \"-c\", <字面串>, …] 当内联脚本切，连到它 import 的仓内模块",
                                "python：-c 的串折不出拒算 inline-script-not-literal")),
    ("shell-later-interpreters-ignored", ("shell：一条命令里第二个解释器（bash 包装脚本 N python3 x.py）照第一个的规则认，连到 x.py",)),
    ("shell-substitutions-ignored", ("shell：起同目录脚本连边（写在命令替换里：x=\"$(bash 同目录/y.sh)\"）",
                                     "shell：< <(命令 \"$x\") 里的命令照命令认，前面那个 < 不吞掉它的第一个词")),
    ("external-program-words-unfiltered", ("shell：die 消息里出现 qemu-system-x86_64 不算外部程序",)),
    ("python-modules-ignored", ("shell：python3 -m <仓内模块> 连到那份模块的程序入口", "python：-m unittest <测试模块> 整份连上（测试方法改了作废）")),
    ("shell-unfolded-commands-allowed", ("shell：命令位置上折不出的程序拒算 shell-command-unfolded",)),
    ("shell-wrapped-commands-ignored", ("shell：包装命令（timeout 60）后面直接执行的脚本连上",)),
    ("shell-command-structure-ignored", ("shell：[[ … || … ]]、(( … ))、case 分支的模式、数组字面量里的词不当命令",
                                         "shell：{ python3 /dev/fd/3 …; } 3<<'PY'（重定向写在组后面）照内联脚本切")),
    ("shell-array-commands-unfolded", ("shell：\"${数组[@]}\" 当命令时按数组的字面赋值摊开，连到它起的脚本",)),
    ("shell-heredoc-captures-unfolded", ("shell：python3 -c \"$(打印正文的函数)\" 折出函数打的那几行当内联脚本",)),
    ("declared-checks-ignored", ("shell：文件头登记的 check 判据（preflight 开跑前跑）照内联 bash 认，连到它起的脚本",
                                 "python：上游 preflight 跑文件头登记的判据（bash -c declaration.command）不拒算，判据从声明它的脚本连")),
    ("inline-argv-commands-ignored", ("shell：内联 python exec sys.argv[1:] 时后面那几个词照命令认",)),
    ("python-unfolded-heads-allowed", ("python：变量解释器折不出拒算 subprocess-command-unfolded",)),
    ("python-commands-literal-only", ("python：命令是 [解释器] + 参数（按调用处给的列表折）连到那份脚本", "python：认不出的命令（对象的属性）拒算 subprocess-command-unfolded")),
    ("python-path-programs-refused", ("python：shutil.which(…) 与 os.path.join(<折不出的目录>, \"bin/x\") 起的根外程序不拒算",
                                      "python：内联脚本起 sys.argv[k]（取的是起它那份脚本的 \"$1\"）不拒算：程序是调用处给的词")),
    ("python-command-dictionaries-unfolded", ("python：命令里的 *选项[\"键\"] 折回调用处给的字典字面量，连到那份脚本",)),
    ("python-dispatch-calls-ignored", ("python：经分派表 表[键](…) 调到的函数，命令参数按那一处调用折",)),
    ("rust-kind-names-hyphenated", ("dump：种类名照 crates/singlefs-unit-graph 的拼法（macro_rules、item_macro、mod、mod_head、wgsl_compute）",)),
    ("rust-qualified-names-file-local", ("dump：限定名带 crate、目标与模块路径前缀，文档测试写 <模块>::<doctest line 行>",)),
    ("rust-doctest-fingerprint-from-rows", ("文档测试：代码块里只改注释指纹不变、改代码指纹变",)),
    ("rust-module-heads-dropped", ("模块头：#![…] 与行内模块头切成单元，模块里的单元连它，改内属性作废",)),
    ("rust-data-dump-unlisted", ("dump：include_str! 读的数据文件按读它的那一处列一行 data",)),
    ("wgsl-external-names-by-long-identifiers", ("WGSL 外名：首段连 _ 落在 macro_rules! 体或串字面量里有着落", "WGSL 外名：只在别的单元的长标识符里出现的拒算")),
    ("wgsl-external-name-sources-unlinked", ("WGSL 外名的着落做成边：给名字的 Rust 单元改了 WGSL 入口作废",)),
    ("rust-escaped-brace-captures-skipped", ("format! 串里 {{{名}}}（转义的 {{ 后紧跟的捕获）改了作废",)),
    ("rust-union-as-keyword", ("条目级宏调用里的 union 算它可能定义的名字",)),
    ("rust-module-heads-unlinked", ("R-结构：改文件开头的 #![…]，经 mod 声明连进来的子模块里的入口作废", "R-结构：改行内模块头的内属性，只那个模块里的入口作废")),
    ("rust-module-declarations-unlinked", ("R-结构：改文件开头的 #![…]，经 mod 声明连进来的子模块里的入口作废",)),
    ("rust-use-names-resolved", ("R-结构：use 引进的名字不往下解，没用到的那个改了不作废",)),
    ("rust-use-aliases-unresolved", ("R-结构：use a::b as c 之后调 c()，按 b 再解",)),
    ("rust-trait-impls-by-trait-arguments", ("R-隐式：只点名 trait 参数里的类型，那个 trait impl 不进闭包",)),
    ("rust-cfg-atoms-unnarrowed", ("cfg 成分：闭包没引到 gpu 的入口，带不带 gpu 特性两种目标下键相同",)),
    ("rust-cfg-macro-atoms-ignored", ("cfg 成分：闭包里有 cfg!、语句上的 #[cfg] 或 #[cfg_attr] 引到 gpu 的入口，带不带 gpu 特性键不同",)),
    ("wgsl-defined-name-suffixes-ignored", ("WGSL 外名：条目级宏调用定义的名字以 _ 加外名收尾就有着落",
                                            "WGSL 外名的着落做成边：只靠外名点到的条目级宏调用并进闭包，改它里面的码键变")),
    ("wgsl-prefix-remainders-unchecked", ("WGSL 外名：首段只落在无关的串里、其余那一段哪儿都没有的拒算",)),
    ("wgsl-name-suppliers-not-pulled-in", ("WGSL 外名的着落做成边：只靠外名点到的条目级宏调用并进闭包，改它里面的码键变",)),
    ("wgsl-modules-per-naming-unit", ("WGSL：闭包读到的几份 .wgsl 合成一个模块，名字跨文件解",)),
    ("rust-trait-self-paths-ignored", ("R-路径：trait 成员里的 Self::x 落到这个 trait 自己的成员 x",)),
    ("rust-doctest-structure-linked", ("R-结构：文档测试不连所挂的条目与那个模块的 use",)),
    ("rust-use-self-imports-ignored", ("R-结构：use a::{self} 引进名字 a，只写 a::f() 的单元连上这条 use",)),
    ("rust-open-by-trait-arguments", ("R-开放：dyn Trait 只带 Trait 自己的 impl，不带只在 trait 参数里写了它的 impl",)),
    ("rust-blanket-impls-ignored", ("R-方法：blanket impl 够得到的方法算边，改它作废",)),
    ("rust-underscore-items-named", ("R-裸名：const _ 没有名字，let _ 与 _ => 不把 gpu 特性下的 const _ 拉进闭包，带不带 gpu 特性键相同",)),
    ("rust-includes-outside-root-refused", ("include_str! 读根外的文件：读得到收成数据单元、改它键变，读不到拒算 include-missing",)),
    ("rust-blanket-self-parameter-only", ("R-方法：自身类型里出现泛型参数的 impl（(T, u32)、[T]、[T; N]、Box<T>、Option<Box<T>>、Sealed<T>）都按 blanket 放行，改 Sealed<T> 那一份作废",)),
    ("wgsl-other-file-definitions-unlinked", ("WGSL 外名在链接集里别的 include_str! 读进来的 .wgsl 里有定义：读它的 Rust 单元与那份 .wgsl 拉进闭包，改那个常量作废",)),
)


def run_cell(judge):
    with tempfile.TemporaryDirectory(prefix="function-units-selftest-") as work:
        try:
            return judge(work)
        except (FunctionUnitsError, UsageError, AssertionError, OSError, KeyError, ValueError) as error:
            return False, f"抛了 {type(error).__name__}：{error}"


def with_break(switch, judge):
    previous = os.environ.get(BREAK_VARIABLE)
    os.environ[BREAK_VARIABLE] = switch
    try:
        return run_cell(judge)
    finally:
        if previous is None:
            os.environ.pop(BREAK_VARIABLE, None)
        else:
            os.environ[BREAK_VARIABLE] = previous


def selftest():
    started = time.monotonic()
    failures, checked = [], 0
    if active_breaks():
        print(f"  ℹ {BREAK_VARIABLE}={os.environ.get(BREAK_VARIABLE)} 设着：点名的那几格本来就该红")
    cells = selftest_cells()
    by_name = dict(cells)
    for name, judge in cells:
        checked += 1
        ok, detail = run_cell(judge)
        print(f"  {'✓' if ok else '✗'} {name}" + ("" if ok else f"：{detail}"))  # gate-lint:detail
        if not ok:
            failures.append(name)
    for switch, red_cells in BREAK_CELLS:
        checked += 1
        missing = [cell for cell in red_cells if cell not in by_name]
        flipped = [cell for cell in red_cells if cell in by_name and not with_break(switch, by_name[cell])[0]]
        ok = not missing and flipped == list(red_cells)
        label = f"弄坏开关 {switch} 下「{'」「'.join(red_cells)}」转红"
        print(f"  {'✓' if ok else '✗'} {label}" + ("" if ok else f"：只红了 {flipped}，表里没有 {missing}"))  # gate-lint:detail
        if not ok:
            failures.append(label)
    elapsed = time.monotonic() - started
    if failures:
        print(f"  ✗ lib_function_units.py 自证没过：{len(failures)} 格判错（共 {checked} 格，{elapsed:.1f} 秒）")
        print("    → 怎么办：照上面每一格改对应语言那一段的切分或闭包规则；FUNCTION_UNITS_BREAK 设着的话点名的那几格本来就该红")
        return 1
    print(f"  ✓ lib_function_units.py 自证通过：{checked} 格都对（Rust 夹具 {len(RUST_FIXTURE_CASES)} 格加拒算与形态、python、shell、键；"
          f"弄坏开关 {len(BREAK_CELLS)} 个各红在点名的格；{elapsed:.1f} 秒）")
    return 0


# ── 命令行 ──────────────────────────────────────────────────────────────────

VALUE_OPTIONS = ("--language", "--under", "--target", "--cfg", "--feature", "--toolchain", "--dependency", "--external", "--wgsl-module",
                 "--rust-strings-entry")


def parse_options(arguments):
    options, positional = Options(), []
    repeated = {"--under": options.under, "--cfg": options.cfg, "--feature": options.features, "--toolchain": options.toolchain,
                "--dependency": options.dependencies, "--wgsl-module": options.wgsl_modules}
    i = 0
    while i < len(arguments):
        argument = arguments[i]
        if argument == "--no-default-cfg":
            options.no_default_cfg = True
            i += 1
            continue
        if argument == "--no-default-features":
            options.no_default_features = True
            i += 1
            continue
        if argument == "--explain":
            options.explain = True
            i += 1
            continue
        if argument in VALUE_OPTIONS:
            if i + 1 >= len(arguments):
                raise UsageError(f"{argument} 后面要一个值")
            value = arguments[i + 1]
            if argument == "--external":
                parts = value.split("=", 2)
                if len(parts) != 3 or not re.match(r"^[0-9a-f]{64}$", parts[1]):
                    raise UsageError(f"--external 写成 <程序名>=<sha256>=<--version 那一行>：{value}")
                options.externals[parts[0]] = (parts[1], parts[2])
            elif argument in repeated:
                repeated[argument].append(value)
            elif argument == "--language":
                options.language = value
            elif argument == "--target":
                options.target = value
            else:
                options.rust_strings_entry = value
            i += 2
            continue
        if argument.startswith("--"):
            raise UsageError(f"认不出的选项：{argument}")
        positional.append(argument)
        i += 1
    if options.language not in (None, "rust", "python", "shell", "wgsl"):
        raise UsageError(f"--language 只认 rust、python、shell、wgsl：{options.language}")
    return options, positional


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    try:
        if not arguments:
            raise UsageError("要一个子命令")
        command, rest = arguments[0], arguments[1:]
        options, positional = parse_options(rest)
        if command == "dump" and len(positional) == 1:
            root = positional[0]
            if not os.path.isdir(root):
                raise FunctionUnitsError(f"根不在：{root}")
            started = time.monotonic()
            unit_lines, refusal_lines, counts = dump_lines(root, options.language, options.under or None)
            for line in unit_lines + refusal_lines:
                print(line)
            summary = "、".join(f"{language} {count}" for language, count in sorted(counts.items()))
            print(f"  ✓ 切出 {len(unit_lines)} 个单元（{summary}），拒算 {len(refusal_lines)} 处，{time.monotonic() - started:.1f} 秒", file=sys.stderr)
            return 0
        if command in ("closure", "key") and len(positional) == 2:
            root, entry = positional
            closure = closure_of(root, entry, options)
            if command == "closure":
                for unit in closure.units():
                    print(unit.dump_text())
                refusals = closure.refusal_list()
                for refusal in refusals:
                    print(refusal.text())
                if options.explain:
                    for line in explain_lines(closure):
                        print(line)
                return REFUSED_EXIT if refusals else 0
            key, refusals = closure_key(closure, options.toolchain, options.dependencies, options.externals)
            if key is None:
                for refusal in refusals:
                    print(refusal.text())
                return REFUSED_EXIT
            print(f"key\t{key}")
            for unit in closure.units():
                print(unit.dump_text())
            return 0
        if command == "controls" and len(positional) == 1:
            return run_controls(positional[0])
        raise UsageError(f"认不出的用法：{' '.join(arguments)}")
    except UsageError as error:
        print(f"  ✗ 用法错：{error}", file=sys.stderr)
        print("    → 怎么办：照文件头「用法」一段给参数（dump <根> | closure <根> <入口> | key <根> <入口> | controls <根> | --selftest）", file=sys.stderr)
        return 2
    except (FunctionUnitsError, OSError) as error:
        print(f"  ✗ 算不出：{error}", file=sys.stderr)
        print("    → 怎么办：看上一句点的是哪一样（根、bash、Cargo.toml、读不了的文件），补齐再跑；调用方按要跑处置，不当没变", file=sys.stderr)
        return 3


if __name__ == '__main__':
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
