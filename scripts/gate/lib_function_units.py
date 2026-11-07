#!/usr/bin/env python3
# admission: always 判的是这一次给的根下此刻的源码，上一次算出来的键不替这一次作保
# run-condition: command bash
# gate-similar: lib_source_digest.py 那一份按文件算不随注释变的摘要，不切单元、不收调用图；这里的 bash 规范化与它同法（搬过来没 import：它下一批改成调这里）
# gate-similar: mutation-prune.py 它的 FileIndex 切 fn 项与 impl 方法、没有调用图，函数外部分与宏按整文件取；这里照设计第三节另写切分与八条闭包规则
# gate-similar: admission.py 它的 crash_case_judging_digest_text 那份 python 闭包只看一个模块、只认 ast.Name、单元原文带注释；这里扩到跨模块、方法收窄与拒算
"""函数单元：把源码切成函数一级的单元，从入口顺调用图收闭包，算一把只随闭包里单元内容变的键（Rust、WGSL、Python、shell）。
设计照 design 路报告第三节（3.1 键、3.3 Rust 八条闭包规则 R-目标 / R-裸名 / R-路径 / R-方法 / R-隐式 / R-开放 / R-结构 / R-宏、
3.4 拒算、3.5 WGSL、3.6 Python、3.7 shell）。Rust 这一份与 crates/singlefs-unit-graph 是两条不共用代码的路子：这里不 import、不读那个 crate，
两边照同一份规则各写各的；门禁 code-tooling 的 function-units-two-implementations-agree 格逐单元对拍 dump（种类、限定名、起止、指纹），
再逐个比登记入口的闭包成员与键（python 这一边的键照 unit-graph 的写法算：unit_graph_key）。dump 与闭包输出的写法以 crates/singlefs-unit-graph 为准。

输出（一行一项、制表符分隔、次序确定；Rust 与 WGSL 的单元行与 crates/singlefs-unit-graph 的 unit-graph 逐字相同）：
  单元行   <种类>\t<限定名>\t<文件>:<起>-<止>\t<指纹>（Unit.dump_text；Unit.text 是进 UnitKey 组成清单与各判法按（种类, 限定名, 文件）认单元的那一行，
           限定名是文件里的嵌套路径、不带前缀）
  数据行   data\t<数据文件>\t<读它的文件>:<行>-<行>\t<字节 sha256>（dump 里每一处读一行，与 unit-graph 的 dump 同一种写法）；
           读的是目录的写 data-directory\t<目录>\t<读它的文件>:<行>-<行>\t<目录指纹>（下面「数据目录」）
  键行     key\t<sha256>
  拒算行   refused\t<形态>\t<文件>:<行>\t<原样片段>
  Rust、WGSL 与数据行按（文件，起行，止行，整行）排（与 unit-graph 的 dump 同一种次序），python 与 shell 的单元行按（文件，起行，种类，限定名）排，
  拒算行按（文件，行，形态，片段）排；文件是相对根的路径，起止是 1 起算的行号；
  原样片段是那一行原文，空白折成一个空格，至多 160 个字符。closure --explain 另打 via\t<文件>:<限定名>\t<从哪个单元来>\t<规则>，只当诊断看。

种类与限定名：
  Rust   种类名照 crates/singlefs-unit-graph（src/units.rs 的 output_name）：fn（自由 fn 与 impl / trait 里的方法）、impl（impl 头：属性起、到 { 之前）、
         trait（trait 头）、const、static、struct、enum、union、type、macro_rules（macro_rules! 整个定义）、use、item_macro（条目级宏调用）、mod（mod x;）、
         mod_head（模块头：行内模块从外属性起、到 { 与紧跟的内属性 #![…]；文件开头的内属性 #![…] 另一个，限定名 <module attributes>；模块里的单元都连它）、
         extern_block、extern_crate、broadcast_group（verus! 里的 broadcast group）、assume_specification（verus! 里的 assume_specification[ 路径 ] …;
         整条连属性一个单元，按方括号里路径末段的名字找得到它、不按它自己的名字；产品行为视图里剥掉）、doctest（文档注释里的代码块）；
         闭包里的数据单元 data-file（include_str! / include_bytes! 与运行期拼得出字面路径读的文件：限定名是那份文件相对根的路径，起止 1-<行数>，指纹是字节 sha256）、
         data-directory（清单目录折出的 crates/ 之下一个目录，下面「数据目录」：限定名是目录相对根的路径，起止 1-<文件数>，指纹见 directory_fingerprint），
         dump 与命令行 closure、key 照 unit-graph 按读它的那一处列（上面「数据行」：data-directory\t<目录>\t<读它的文件>:<行>-<行>\t<指纹>，closure_unit_lines）。
         弄坏开关 closure-directory-lines-span-files：closure、key 把数据目录打成它自己那一行（<目录>:1-<文件数>）；
         自证「命令行 closure：数据目录单元按读它的那一处列…」那一格必须判错。
         文件里的嵌套路径（Unit.name，入口 <文件>::<限定名> 照它找）用 :: 连：内联 mod 写名字；impl 写 <impl 词法单元…>（impl 关键字之后、去掉紧跟它的泛型参数表与 where 子句，
         逐个词法单元用一个空格连，相邻两个 : 连成 :: 且两边不留空格，例 <impl std::ops::Add for Money>、<impl Wrapper < T >>）；trait 头写名字，它的成员挂在 <trait 名> 下；use 写 use@<起行>
         （verus! 里的 broadcast use 写 broadcast-use@<broadcast 那一行>）；条目级宏调用写 <宏名>!@<宏名那一行>；extern 块写 extern@<起行>；
         assume_specification 写 assume_specification@<关键字那一行>；
         文档测试写 <所挂条目的嵌套路径>::<doctest line 代码块起行>，挂模块的写 <模块路径>::<doctest line 行>（文件开头的就是 <doctest line 行>）。
         输出行里的限定名（Unit.full）照 unit-graph 带前缀：<crate 名>::<目标标记>::<模块路径>::<嵌套路径>——目标标记取这份文件所属的头一个目标
         （crate 里 lib、bin、test、example、bench、build 的次序，同种按名字与根文件排；lib 不写，bin 写 <bin 名>，构建脚本写 <build>），模块路径是它在那个目标的
         模块树里经 mod 声明连进来的那一截；哪个目标都不属于的写 <unlinked 文件>。例 singlefs_verification::<bin e158_root_choice_repair>::main。
         verus! { … } 与 thread_local! { … } 展开成里面的条目再切；verus! 里一个条目从它起头到下一个同层条目起头之前（括号配平，
         同层条目起头 = ; 或 } 之后接属性、条目关键字、宏调用或后面接 [ / < 的 assume_specification；; 本身不是起头，} 之后紧跟的 ; 归前一个条目）。
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
         带 #[test] 的 fn 与 #[cfg(test)] 模块里的单元不当名字供给者收（测试函数没人调，它的串到不了拼内核的函数；入口本身不在此列，早就在闭包里）；
         外名在链接集里别的单元 include_str! / include_bytes! 读进来、还不在模块里的一份 .wgsl 的顶层条目里有定义时，同样补一条着落边：
         读它的那个 Rust 单元连同它的闭包并进来，那份 .wgsl 进模块、名字在合起来的名字空间里解；拉不进的才拒算 wgsl-undefined-name。
         WGSL 入口（--rust-strings-entry 给 Rust 入口）的闭包带上给名字着落的 Rust 单元连同它的闭包。
  Python def（模块级 def / async def，类里的方法写 <类>.<方法>）、class（类头：装饰器、基类、类体里不是 def 的语句）、assign（模块级赋值，
         按目标名各一单元）、import（模块级 import，按绑定的名字各一单元）、module-body（其余模块级语句，限定名 <module-body>，每份文件一个，
         没有语句也有）、main-guard（if __name__ == "__main__": 那一段，限定名 <main-guard>；认法与几段合一见下面「Python 闭包」）、def-head（带装饰器或缺省值的模块级 def 与模块级类里的方法
         另切的 def 头：只收装饰器与缺省值那几个表达式，限定名 <def 的限定名>.<head>，不进按名字的解析，下面「import 时执行的模块级语句」）；
         指纹 = ast.dump(节点, include_attributes=False)
         的 sha256，节点里各层 def / class 的文档串先去掉。数据单元 docstring（被读 __doc__ 的文档串，限定名 <所属>.__doc__，模块的写 <module>.__doc__）、
         source-text（读自己原文的那份文件整份字节，限定名 <source-text>）、data-file（运行期读的仓内文件，下面「运行期读的数据文件」）。
  shell  function（规范文本里 function <名> () 起、同缩进的 } 止；嵌套的另算一个，限定名 <外层>::<名>，外层文本里留着它；here-doc 正文跳过不切）、
         script-body（规范文本去掉顶层函数，限定名 <script-body>）、shell-variable（顶层赋值语句，按变量名各一单元）、
         kept-comments（数据单元：程序读的注释「# <词>:」与 # shellcheck，限定名 <kept-comments>）。规范文本 = bash -n 过了且没有告警、
         整份包进一个函数定义 eval（不执行）、取 declare -f；指纹 = 单元规范文本的 sha256。起止行是回到原文按函数头找的，只当索引。

Python 闭包：import 时执行的模块级语句（欠账 C890（判法单元闭包少收 import 时执行的模块级语句））：
  一个模块有单元进闭包（ScriptClosure.module_entered），它就被载入过：它的模块级 import、assign、类头本身与 def-head 一起进闭包（规则名 import-time）；
  类头走 import-time-class：不算被点名、不连方法，之后被别的规则点到再照被点名的类连方法。其余模块级语句在 module-body 里，本来就连。
  内联 python（here-doc、-c）照样：这几样穿过去连边，自己不进闭包。不收的：只带注解、没装饰器也没缺省值的 def 头（注解在 import 时也求值）；main-guard 不在 import 时执行。
  main-guard 认两种写法（is_main_guard）：if __name__ == "__main__":，与不带 else 的 if __name__ == "__main__" and <别的条件>:（and 从左往右短路，import 时
  后面的条件与块都不执行；scripts/hosts/memory_peaks_lib.py 那一种）；and 写法里 __name__ 那一项不在最左、带 else 的认不出，留在 module-body，import 时照收。
  头一种带 else（或 elif）的：if 那一支是 main-guard，else 那一支在 import 时执行，照别的模块级语句进 module-body。
  被 import、被 spec_from_file_location 载入的模块只连 module-body，不连 main-guard（欠账 C947：不然被 import 的模块的自证分支连进判法闭包）；
  这份文件被当程序起（子进程边、runpy.run_path、内联脚本、只写 <文件> 的入口）时连 main-guard；块里按 sys.argv 分派的分支只在给了参数时剪（下面「程序入口带参数」），
  没给参数的照旧整份收。
  一份文件里几段 main-guard 合成一个单元（被当程序起时每一段都执行）：节点是那几段，起行取头一段、止行取末一段，只有一段时指纹照旧取那一段。
程序入口带参数（ScriptClosure.run 的 program_arguments；命令行 --program-argument <词>，admission.py 判定请求的 program-argument:<词>；
  弄坏开关 python-program-arguments-unnarrowed 下照旧整份收）：判定是「这份脚本带这组参数跑」时，只收这组参数真走到的那一支：
  入口是 python 文件的 main-guard，sys.argv 取 [<文件>, <参数…>]；按已知的值逐个单元剪（PythonModule.narrowing_plan）：
    已知的值：sys.argv、它的整数下标与不带步长的切片、字面串与字面列表、函数里只赋过一次的名字、调用处带进来的参数；不知道的词记成「不知道」，
      长度不知道（*展开、认不出的元素之后）的记成开口的一串，下标落在开口那一截上的不知道。
    剪：if 的条件判得出真假的只收走到的那一支（==、!=、in、not in、not、and、or 与一串参数非空，in 的右边是字面列表、集合、本模块只赋过一次的字典字面量的键）；
      一支以 return、raise、sys.exit(…)、os._exit(…) 收尾（两支都收尾的 if 也算）的，同一串语句里它后面的不收；判不出的两支都收。
    分派表：本模块只赋过一次的字典字面量[已知的串] 只连那一项的值，不连整张表；表[已知的串](实参) 与 本模块的函数(实参) 带着已知的实参往下剪（叫到的函数
      按带进来的参数再剪，同一个函数带不同的已知参数各收一遍，没带已知参数的照整份收；函数里给那个参数另赋过值的不带）。只在单元自己那一层作用域里剪，
      套在里面的函数不剪。<词> in 表 / <词> not in 表（表是本模块只赋过一次的字典字面量）只用到表的键：这一处不当对整张表的引用，表的赋值单元
      照下一条「只收赋值本身」收（值里写的函数名不连）；弄坏开关 python-membership-tables-followed 下照旧当普通引用连整张表。
    给了参数时 import 时执行的赋值只收赋值本身（PythonModule.table_reference_names）：值里的字典、列表、元组、集合字面量当元素写的函数名不在 import 时调，
      不沿它们连；取这张表的那一处照上面连（取的是整张表的连全部，取一项的连那一项）。弄坏开关 python-import-time-tables-followed 下照旧沿它们连。
    子进程边：起 python 脚本的命令里脚本之后的词照样带过去（折得出的是已知的词），被起的那份照它们剪；脚本之后有认不出个数的元素的记成开口的一串。
    参数另进键：admission.py 的键加一行 program-argument（每个词照次序），参数不同的两条判定闭包相同也不共用键。
  只认 python 的 main-guard；shell 的 script-body 与没有 main-guard 的 python 文件照旧整份收。
运行期读的数据文件（python；PythonModule.read_target / data_read_origins，ScriptClosure.python_data_reads；弄坏开关 python-data-reads-ignored 下回到既不收也不拒算）：
  读文件的写法：open / io.open / codecs.open（模式里没有 w a x +）、<路径>.read_text() / .read_bytes()、Path(…).open()、shutil.copy / copy2 / copyfile 的源。
  被读的文件分两类，再加一种拒算；每一处读都落进其中一种（判据逐条写在后面）：
  一、判法自己的数据：收进这一份闭包的键（数据单元，规则名 data-read）。判据：路径推得出落在根里的固定位置——
    整个折得出（fold，连同「<折不出的目录>/<仓里的相对路径>」那一种 tree_copy_file）→ 根里的文件收成 data-file（限定名是那份文件相对根的路径，
      指纹是字节 sha256，同 Rust 的 data-file）；根外的、盘上没有的跳过；
    只认得出字面尾巴（path_tails：os.path.join(<折不出>, "pkg", "table.tsv")、f"{目录}/notes.txt"）→ 仓里以这一截收尾的文件整组收；
      一份都没有的不是仓里的文件（这一趟自己写的、根外的），跳过；尾巴只剩后缀（".json"）的不算认得出；
      折不出的前缀推得到本函数的一个参数的（PythonModule.prefix_parameter：os.path.join(<参数>, …)、os.path.dirname 一类、本函数里给它的赋值），
      那一组留到收完闭包再按调用处认（ScriptClosure.tail_read_supplied）：闭包里每一处给的都是这一趟自己建的临时目录、判法入口的参数（被判的输入），
      或又是这样的参数，就不收（归二）；有一处给的是 sys.argv、环境变量、根里的路径或推不出的（仓副本、根本身都可能），照尾巴整组收。
      弄坏开关 python-tail-reads-unanchored 下读的时候就整组收；
    os.path.join(<折得出、是根里一个目录（不是根本身）>, <折不出、出处推不出>…)（fixed_directory_read）→ 那个目录整棵收成 data-directory
      （指纹同 directory_fingerprint；读的是那个目录里登记表、夹具、模板中的某一份，哪一份都可能）；
    名字照它的赋值逐个认，模块里只赋过一次的常量照它的值认；函数参数照每一处调用给的实参认：先本模块里的调用（方法按 <对象>.<方法>(…)、套着的函数按外层里的调用），
      本模块里没有调用处的，在闭包里按名字找调用处（别的模块里的调用，ScriptClosure.resolve_parameter_reads），逐处照实参认；
      self.<属性> 照本类方法里给 self.<属性> 的每一处赋值认（self_attribute_values）；a or b、a if c else b 每一边各认各的、并起来。
  二、调用方交进来的输入：不进这一份键、不拒算（调用方已把它折进这一条判定的内容输入或单元键：请求的 line:、file:、program-argument:、被判那棵树的单元指纹）。判据：
    路径的出处推得出是下面之一——这一次闭包的入口函数的参数、闭包里找不到调用处的函数参数（入口经引用、反射、pool.map 交出去的）；
    sys.argv、argparse 解出来的属性、环境变量（第 3 条：调用方在哪一行把它们折进键，写在 admission.py 文件头「单元键」）；
    tempfile 新建的临时目录与临时文件（读的是这一趟自己写的）；根外的字面绝对前缀（/proc/…）；
    os.path.join(<折得出的固定目录或根>, <出处是上面这几样的一截>)（被判的输入、计划、请求、日志、被判树单元清单里的源文件：样本根、计划文件、unit.file 这一类）；
    出处推不出、前缀也推不出的（对象属性、读进来的值、循环变量、调用的返回值）：照调用方交进来的认，不收也不拒算（头一类的几种都试过之后才落到这里）；
    os.path.join(<根本身>, <折不出的一截>) 而那一截不是读根里的文件得来的（清单遍历、算出来的名单、参数给的：被判树的成员、拷副本的那几份）同样落在这里；
    拷文件（shutil.copy / copy2 / copyfile）的源折不出的那一截一律落在这里（拷进副本、样本树的那几份由起在副本上的那一趟按它自己的键管）。
  三、拒算 data-path-unfolded（落在读它的那一行）：os.path.join(<折得出、就是根本身>, <折不出的一截>)，那一截是读根里的文件得来的
    （PythonModule.repository_content_derived：沿名字的赋值、for 与推导式的迭代对象、with … as 的对象往回找，碰到一处读根里文件或目录的读）——
    读的是根里某一份由判法自己的数据点名的文件（读进来的登记行点名的这一类），整个根当数据不成样子，哪一份又推不出。
    弄坏开关 python-root-joins-refused 下回到「根本身 + 折不出」一律拒算、拷文件的源照拒算。
  shell 闭包读的文件（重定向 <、cat 的参数这类）这一份还不收、也不拒算；读目录（os.listdir / os.walk / glob）同样还不收。
按行进键的登记表（ROW_KEYED_TABLES，row_keyed_closure；弄坏开关 row-keyed-tables-whole 下照旧整份进键）：
  判法读一张登记表、只按其中几行取用的（按被起的那份脚本查它自己那一行、按门禁阶段查样本档那一行），那张表在这一条判定的键里只进那几行。
  ROW_KEYED_TABLES 写：表（相对根）、读整张表交解析结果的函数（读表函数）、只按几行取用的函数（取用函数）与它看的那几行的正则（{entry} 是这一条判定的入口文件）。
  调用方（admission.py 算判法键时带上请求的入口文件）照它换：闭包里调读表函数的每个单元（读表函数自己除外）都是登记的取用函数时，那张表的 data-file 单元
  换成 data-rows 单元（限定名照旧是表的路径，起止 1-<选中的行数>，指纹是那几个正则与选中的行按原次序连起来的 sha256）；有一个调用者不在登记里
  （读整张表、逐行核的）照旧整份。登记的一张：.claude/hooks/selftest-registry.tsv（门禁 code-tooling 的 research-script-selftests 那几条的判法
  scripts/gate/selftest-process-guard.py 经 .claude/hooks/lib_project_code_runs.py 读它）。
不走的子进程边（EXCLUDED_PROGRAM_EDGES；同一份判决末节第 3、5 条）：登记了的（起它的文件, 起它的单元的限定名, 被起的程序）那一条子进程边不连
  （python 单元与 shell 单元起的都认；shell 函数的限定名同单元表，嵌套的写 <外层>::<名>）。
  按实测排：那一条边只打提示、不改结局；与 scripts/crash/lib_verdict_library.py 的 NOT_JUDGING_FILES（算键的库自己，按角色排、整份文件不进键）是两种排除。
  登记了一条：E183 跑法脚本计划那一步（plan_step）起 scripts/gate/admission.py 的准入边。准入是闸，退出码只定这一件跑不跑，标准输出只有 E7INPUT 行，
  读产物的判法先 sed 去掉这几行；它不改读数，所以不沿它把准入模块收进实验键。登记一条边之前先现查起点单元与被起的程序都在，不在的不登记。
  自证 EXCLUDED_EDGE_PYTHON_CELL 与 EXCLUDED_EDGE_SHELL_CELL 在夹具上证这套机制：自证期间临时往表里登记夹具里的一条边，核登记了的那一条不连、
  同一个单元起的另一个程序照连、不登记时那一条照连。EXCLUDED_EDGE_EXIT_CELL 在夹具上证「键不变而退出码变」必红。
  登记的准入边配三格（真仓）：ADMISSION_EDGE_ANOMALY_CELL（准入这条路上的异常只让计划那一步不跑、读数出不来）、ADMISSION_EDGE_STRIP_CELL
  （准入打的只有 E7INPUT 行，判法去掉之后读数一行不差）、ADMISSION_EDGE_REACH_CELL（切边前后少掉的单元都只经这条边可达）。
判法入口的登记（registered_judging_entries；命令行 entries <根>）：存判定的各路判法入口从各份脚本的登记现取，不手抄——广谱变异的 JUDGING_ENTRIES 与
  GPU_JUDGING_ENTRIES、crates 变异的 JUDGING_ENTRIES、admission.py 的 REPLAY_SCRIPT 与 REPLAY_JUDGING_FUNCTIONS、e165 的 JUDGING_ENTRY、
  verus-proof-check.py 的 JUDGING_ENTRIES，加门禁阶段（.claude/gate.d/*.sh）里写死的 "judge:<文件>::<函数>" 令牌；自证真仓那几格的入口从它取。
  登记值可以是字面量，也可以是前面模块级字面量的名字与 + 拼成的式子。自证 REAL_JUDGING_CLOSURES_CELL 逐个收登记的入口的闭包，有一个拒算就红
  （拒算的那一路判定写不进库、每趟照判）；口径照 judging_unit_key：crates 变异的入口全算，别的入口去掉 lib_verdict_library.py 的 NOT_JUDGING_FILES。

Rust 闭包（3.3，与 crates/singlefs-unit-graph 的 src/closure.rs 逐条同义；这里只写 R-结构、R-路径、R-隐式与 R-开放的口径，别的几条照设计原文）：
  入口名字的定义那一处（只给 scripts/gate/admission.py 用，rust_closure_of 的 entry_definition_name_unresolved=True）：入口单元自己的名字在它的词法单元里只出现在
          定义那一处（fn <名>）时，这个名字不按 R-裸名解、也不拿来判连不连 use，同一目标里短名相同、不在同一模块的单元不因此收进入口的闭包
          （两条同短名的测试函数各算各的键）；名字在函数体里另有出现（递归、经路径点名同名的另一个）照解。不给是 False：与 crates/singlefs-unit-graph
          同义的那一份（两份实现对拍、dump、命令行 closure / key）照旧把定义那一处当裸名解
  到这些单元就停（只给 scripts/mutation/crates-mutation-rows.py 的 static-check --reachability 用，rust_closure_of 的 stop_at）：stop_at 里每一项是
          (来处单元, 单元)。来处是 None 的，那个单元不进闭包、也不从它往下连；来处是一个单元的，只是从来处连到它的那一条边不走，别的单元连到它照连
          （方法调用经「类型后点名」连进来的那一条边，来处记的是点名类型的单元，按边停截不住，要截就按单元停）。入口不停。不给（空）就是全闭包。
          截过的闭包只拿来判够不够得着，不算键、不进缓存；crates/singlefs-unit-graph 那一份没有这个参数，两份实现对拍不带它
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
  闭包外的四类边（records/2026-11-02-一百七十六节欠账.md；索引在 RustWorkspace.index_macro_and_foreign_edges，crates/singlefs-unit-graph 那一份还没有，
  两份实现对拍要等它跟上）：
  条目级宏调用生成的条目  调用体里跟在 enum / struct / union / type 后面的名字是宏生成的类型：调用进闭包时这些名字算被点名（R-方法、R-隐式照连到写给它的 impl）；
          同名 macro_rules! 体里展开式顶层（宏体的括号一层、展开式的括号一层；impl $name { … } 里的关联项不算）跟在 const / static / fn / struct / enum /
          union / type 后面、按字面写出的名字（不是 $ 起头的捕获）登到这个宏的每一次调用名下，
          点名这个名字的单元连到调用（宏名同名的几份 macro_rules! 合起来算）。
          弄坏开关 rust-macro-generated-types-unnamed：宏生成的类型不算被点名；rust-macro-rules-literal-names-unlinked：宏体里的字面名字不登
  #[global_allocator]  带这个属性的 static 管整个链接出来的程序：入口的链接集里有它所在的目标，它就进闭包（规则名 global-allocator），经它的类型
          R-隐式带上 GlobalAlloc 的 impl。弄坏开关 rust-global-allocators-unlinked：不进
  外部类型上的 trait impl  trait 是本仓的、自身类型是外部类型（写成路径、头一段不是 crate / self / super / Self 与本仓 crate 名；或自身类型里的名字没有一个是本仓
          定义的类型）的 trait impl，经泛型约束分派、自身类型永远不会被点名：R-方法落到 .x( 时它的成员 x 照连，R-路径 Tr::x 同样。
          弄坏开关 rust-foreign-self-trait-impls-unlinked：不连
  构建脚本交的值  env!("X") / option_env!("X") 的 X 不是 cargo 给的（不以 CARGO 起头、不是 OUT_DIR）：连到本 crate 构建脚本的 main（它经 cargo::rustc-env 交这个值），
          规则名 build-environment；没有构建脚本的不连。弄坏开关 rust-build-environment-unlinked：不连
  工作区外的 path 依赖  discover_rust_crates 从工作区成员顺 path 依赖（normal、dev、build）收落在仓根底下、有 Cargo.toml 的 crate，根清单 exclude 里的也收
          （prune-server/ 这一类自己是工作区根的独立工程）；落在仓根外的不收。弄坏开关 rust-excluded-path-dependencies-dropped：不收。
          这样收进来的 crate 按调用关系连，不按同名连（工作区成员之间跨目标照旧按 R-裸名同名连，与 crates/singlefs-unit-graph 同）：
          别的包里的裸名要这个名字在工作区里经 use 或路径从这个 crate 引进来过（RustWorkspace.names_reaching_outside_crate：use / extern crate 语句里
          库名或已引进的名字后面的名字、根名::a::b 路径里的段、它的 macro_rules! 名后接 !；根名后面有 glob 的全收），才连到这个 crate 的同名条目，
          只是同名的字段、局部变量不连（弄坏开关 rust-outside-crate-names-linked-unimported：照旧按同名连）；同一个目标里按模块筛照下面「R-裸名同一个目标里按模块筛」
          （committer 里 impl 成员 fn run 不连到 command_line::run；弄坏开关 rust-outside-crate-names-linked-across-modules：照旧按同名连）
  R-裸名同一个目标里按模块筛  crates/ 下的包与工作区外的 crate 一样，同一个目标里的裸名只连单元所在模块看得见的那几份
          （RustWorkspace.candidates_seen_from_module：同一模块的、所在模块的 use 引进了它且路径点到候选所在模块的、写成 模块::名 的，super 照个数往上数层；
          use 路径或 …::名 的根名是 std、core、alloc 或看得见的别的 crate 的库名的指别的 crate，跳过那一处）；点不准的退回全连，
          字段名、局部变量与 impl 成员自己的名字一份都不连；名字是 use 改名的原名、单元是 macro_rules!（体里的名字在调用处解）的不筛；跨目标的只认 pub、不按模块筛。
          验证包里 CPU 判器的单元经 use 从格式库引进 SLOT_BYTES，不连到 gpu_unit_checks 里私有的同名常量。
          弄坏开关 rust-crates-names-linked-across-modules：crates/ 下的包照旧按同名连

数据目录（RustFile.runtime_manifest_path、fold_manifest_chain；crates/singlefs-unit-graph 的 fold_manifest_directory_read、fold_path_chain 同写）：
  Path::new(env!("CARGO_MANIFEST_DIR")) / PathBuf::from(…) 后接 .join("字面") 链、链后不再接别的方法，
  折出的仓内路径是文件的收成 data-file；是目录、落在仓根 crates/ 之下（不含 crates/ 整个、不含仓根与根外）的，收成 data-directory：目录下递归全部文件的字节进键
  （directory_fingerprint：什么目录都不跳过，指向目录的符号链接不进去；每个文件一行「<相对目录的路径>\t<字节 sha256>\n」，按相对路径排序，连起来取 sha256）。
  折出的是仓根本身、crates/ 整个、根外的目录而直接用在表达式位置的，照旧拒算 runtime-path（读整棵仓，有键也复用不上）；接别的方法（.parent()）、.join(<不是字面>) 的也照旧拒算。
  写法的三种：直接写在用目录的位置（read_dir(Path::new(env!(…)).join("rules"))）；整个是 let [mut] <名> [: 类型] = …; 右边的目录，不论是不是能收的目录
  （prune-server/src/preflight.rs 绑清单目录本身，再 .join("..").join("…") 取仓里一份文件），这个单元里后面每一处用到 <名> 的写法各折一样：
  <名> 或 Path::new(<名>) / PathBuf::from(<名>) 接 .join("字面") 链的折出那条路径，单独写 <名> 的取绑出的目录本身、照直接写的那一条判，
  又整个是 let <别名> = …; 右边的照样跟 <别名>（最多跟 DEEPEST_MANIFEST_BINDING_CHAIN 层，再深的拒算）；.<名> 与 <名>:: 不算用法，碰到再 let 同名就停。
  每一处各自结局：接别的方法、.join(<不是字面>)、折出的不是文件也不是能收的目录的，那一处拒算 runtime-path，别的处照收；读与拒算都记在 env! 那一处
  （一处折不出、一处折出文件的，这个单元拒算、那份文件照记一条 data 读）。根外的路径与盘上没有的路径那一处拒算 runtime-path
  （与 Rust 那一份不同写：那一份根外的文件照收，盘上没有的拒算 data-file-unreadable）。
  const <名>: &str = env!("CARGO_MANIFEST_DIR");（私有、初始值整个就是那个宏）本身不读什么，同一份文件里
  Path::new(<名>) / PathBuf::from(<名>) 照 env! 同一处折，别的写法用到 <名> 的拒算。记的行是 env! 那一处（常量写法记 <名> 那一处）的行。
  弄坏开关 rust-manifest-directories-refused：目录一律拒算；自证「Rust：清单目录折出 crates/ 之下的目录收成数据目录…」那一格必须判错。
  弄坏开关 rust-manifest-binding-refused-whole：不是能收的目录不跟 let、拒算，绑给 let 的有一处折不出就整处拒算、一条读都不记；
  自证「Rust：清单目录绑给 let 变量的，一处 .join(变量) 折不出、一处折出文件…」那一格必须判错。

测试二进制起自己（RustFile.current_exe_child；规则名 current-exe-child；crates/singlefs-unit-graph 同写）：
  Command::new(…current_exe()…) 后接 .arg(…) / .args([…]) 链，参数里恰好有一个字面 "--exact"，紧跟在它后面的那个参数折得出测试名的，
  从起子进程的那个单元连一条边到同一编译目标里 libtest 全名（文件在目标里的模块路径加限定名）恰好等于它的那一个 #[test] fn，不拒算 current-exe。
  测试名折法：字面串；常量名（名字记下来，值在收闭包时解：先找起子进程那个单元自己里面的 const <名>: &str = "…";，再找同一个包、在这一次链接集里叫这个名字的
  常量单元，几个都得写成 = "…";、值相同；单元里另有 let 绑定的名字不认，是参数的按参数）；全由这两样组成的 concat!(…) 与 format!("…{}…{名}…", …)；
  起子进程那个 fn 自己的参数名：闭包里每一处调用它（不是 .方法 调用）的那个实参各折一遍，各自再对 #[test] fn，每趟 drain 之后、WGSL 供给之前跑，
  一处都没有、折不出、实参又是参数的拒算。测试名写在 --exact 前面、用 module_path!() 拼的、套着 as_str() 的、单元里另有 let 绑定的，折不出。
  同一个函数里起了子进程的（函数里出现 SPAWNING_WORDS 里的词：Command、spawn、posix_spawn、fork、execv、execve、execvp），对不上恰好一个 #[test] fn、折不出名字、
  不是 Command::new(current_exe()) 的写法的（含路径先存进变量再交给 Command::new），照旧拒算 current-exe。
  同一个函数里不起子进程、只问 current_exe() 路径的（取路径读文件大小、current_exe().is_ok()）不拒算，也不连边：路径拿去做什么与测试要点名的那一条无关。
  不判：路径经参数交给别的函数、由那个函数起子进程的（那个函数里没有 current_exe，本来就不在这条规则里）。
  弄坏开关 rust-current-exe-refused：Command::new(current_exe()) 的写法一律不认（起了子进程的函数里折不出测试名，拒算 current-exe）；
  rust-current-exe-children-unlinked：折得出也不连边（子进程测试的函数体改了键不变）；
  自证「Rust：current_exe() 交给 Command::new 起子进程…」那一格两个开关下都必须判错。
  弄坏开关 rust-current-exe-path-only-refused：只问路径的也拒算；自证「Rust：current_exe() 只问路径…」那一格必须判错。
  弄坏开关 rust-current-exe-spawn-unrefused：起了子进程的函数里折不出测试名的也不拒算；自证「Rust：同一函数里起 Command 的 current_exe() 照旧按起自己判…」那一格必须判错。
  弄坏开关 rust-current-exe-constants-same-file-only：常量名只在起子进程那个单元所在的那一份文件里找（常量定义在同包别的文件里的折不出）；
  自证「Rust：current_exe() 的测试名里的常量定义在同包另一份文件里…」那一格必须判错。

Rust 闭包读到的 shell 脚本（rust_closure_of 的 collect_shell_sources，shell_sourced_files）：数据文件是 shell 脚本（language_of_file：.sh / .bash 或 #! 行是 bash / sh）的，
  它在自己那个 shell 里（传递地）source / . 进来的仓内文件各收成一个 data-file 单元（规则名 shell-source，从那份脚本的 data-file 单元来），字节进键。
  路径照 ShellIndex.links 的折法（字面路径、"$(dirname "${BASH_SOURCE[0]}")/…"、前面赋过字面值的变量、$(cd … && pwd)、git rev-parse 折成根……），
  每份文件的每个单元都看（读到它的是整份字节，不按函数调没调到挑），被 source 进来的那几份接着看，到不再多出文件为止。
  折不出、落在根外、指到不在的文件拒算（shell-path-unfolded / shell-path-outside-root / shell-path-missing），bash 过不了的那几份拒算 bash-syntax。
  不走的：bash -c '… source …' 这类子进程里的 source，与脚本起的 bash / python 子进程（这一份只沿 source 收）。
  这几个单元另按读那份脚本的那一处（读它的单元、那一行）各记一条 data_sites（crates/singlefs-unit-graph 的 follow_shell_sources 同）：
  与那一份对拍的闭包行（data\t<被 source 的文件>\t<读脚本的文件>:<行>-<行>\t<摘要>）与 unit_graph_key 都带它们；closure_key 照旧按文件各一个单元。
  一个 RustWorkspace 里一份脚本只折一趟（RustWorkspace.shell_sourced）。

键（3.1）：下面这些行各以 \\n 收尾、连起来取 sha256：
  form\tsinglefs-function-units-v1
  cfg\t<原子>\t<true|false>           Rust 闭包里各单元真引到的 cfg 原子：单元自己连外层的条件链（#[test] 不算）与它词法单元里每一处
                                        #[cfg(…)]、#![cfg(…)]、#[cfg_attr(条件, …)]（条件与里面套的 cfg）、cfg!(…) 的谓词的每个叶子，写 <名> 或 <名>="<值>"，
                                        真值按收这个单元进来的那个目标的生效 cfg 判（特性按这个单元所在的包；包开了哪些特性照 cargo resolver 2
                                        顺依赖边统一：入口 crate 照入口目标顺依赖段，构建脚本只顺 build 依赖，按 cfg(test) 编的 lib 与 bin、集成测试、
                                        example、bench、文档测试连 dev 依赖，别的只顺普通依赖；别的 crate 只顺普通依赖）；没引到的原子不进键；判不了真假的拒算
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
  entry\t<种类>\t<指纹>[\t<种类>\t<指纹>…]   入口单元自己（Closure.entry_units：rust_closure_of 的入口、ScriptClosure.run 的入口、WGSL 点名的入口）
                                        连同包它的 impl / trait 头（Unit.container）一层层往外，一个入口一行，排序去重（entry_key_lines）。闭包是单元的多重集，
                                        几个入口互相收进对方的闭包（一个 trait impl 里的几个方法经 R-结构、R-隐式；python 里互相调用的两个 def）时闭包一模一样，
                                        没有这一行就共用一把键、一份判定（与 scripts/gate/verus-proof-check.py 的 entry_unit_component 同一种写法）。
                                        别处自己拼的闭包没记入口（lib_verdict_library.py 的 FunctionUnitIndex.key 并起来的那一份），不出这一行。
                                        弄坏开关 key-entry-units-dropped：不出这一行。
                                        文档测试的入口只取它自己：它的 Unit.container 记的是所挂的条目，不是包它的 impl / trait 头，
                                        R-结构也不连（文档测试另编一个 crate），不往外取；弄坏开关 key-entry-doctest-owner-chained：照别的单元往外取。
                                        文档测试入口（rust_closure_of）照 crates/singlefs-unit-graph 的 resolve_entry_units 文档测试那一支，所挂条目或模块的
                                        全部文档测试一起当入口、各记一行（doctest_entry_group）；弄坏开关 rust-doctest-entry-alone：只记交进来的那一个
  不带路径、名字与行号；组成清单（单元行）随键打出当索引。
目标写法：<包目录名>:<lib|bin|test|bench|example|build|doctest>:<名>。

入口：<文件>::<限定名>（按后缀认语言：.rs .wgsl .py .sh / .bash）；只写 <文件> 的，Python 是程序入口（main-guard，没有就 module-body），
  shell 是 script-body；Rust 另认 fn 路径 <crate 名>::<模块…>::<名>（不带文件，按 lib 目标的模块树找）。

用法：
  lib_function_units.py dump <根> [--language rust|python|shell|wgsl] [--under <目录>]… [--judgment-kind product-behavior|formal-verification]
  lib_function_units.py closure <根> <入口> [选项] [--explain]
  lib_function_units.py key <根> <入口> [选项]
  lib_function_units.py controls <根>     V5 / V11 两处变异（crates/mutations.tsv 里原文是那两行的）在临时拷贝上施加，
                                           三个入口的键变不变与 design 报告 2.2 节那张表比；只读根，不写根
  lib_function_units.py entries <根>      判法入口的登记现取（registered_judging_entries），一行一个；只读根
  lib_function_units.py cross-run-caches  跨轮缓存的登记（cross_run_cache_registry，下面「跨轮缓存的登记」）照此刻的环境现取每一份的位置：先打 tempdir\t<tempfile.gettempdir()>，
                                           再一份一行 cache\t<名字>\t<脚本>\t<位置>（算不出的打 refused-cache\t<名字>\t<脚本>\t<原因>、退 3）；只读，不建目录
  lib_function_units.py --selftest        临时目录里的夹具，另把本仓的 scripts/ 与 .claude/（不带 fixtures、kb）拷进临时目录量真仓那几格；不写本仓
  选项：--target <目标>  --cfg <名或 名="值">（加进全局 cfg）  --no-default-cfg  --feature <包名>/<特性>  --no-default-features
        --toolchain <行>  --dependency <行>  --external <程序名>=<sha256>=<--version 那一行>
        --wgsl-module <文件>（WGSL 入口拼进同一个模块的别的 .wgsl）  --rust-strings-entry <Rust 入口>（WGSL 里没定义的名字到这个入口闭包的串字面量里找）
        --under <目录>（python / shell 扫哪几处，默认 scripts、.claude/gate.d、.claude/hooks、.claude/scripts 与入口所在目录）
        --judgment-kind product-behavior|formal-verification（Rust 单元在哪个视图上切、收、算，不给是 formal-verification；下面「判定种类」）
        --program-argument <词>（可给几次，照次序是 python 程序入口的参数；上面「程序入口带参数」，只对只写 <文件> 的 python 入口起作用）
退出码：0 判完；2 用法错；3 算不出（根不在、bash 起不来、清单读不出）；4 拒算（closure / key 的闭包里有拒算：打拒算行，key 不打键行）。

库接口（门禁里 python 写的判法直接 import 它算键，不为算键先编 Rust；产品行为视图例外，下面「判定种类」）：
  RustWorkspace(根, 判定种类=FORMAL_VERIFICATION, product_dump=None)：判定种类给 PRODUCT_BEHAVIOR 的是产品行为视图；
  rust_units(根) → [单元]；rust_closure(根, 入口, 目标=None, cfg=None) → 闭包；wgsl_units(根)、python_units(根, under)、shell_units(根, under)；
  closure_of(根, 入口, 选项) → 闭包（.members 成员与数据单元、.refusals、.cfg_lines、.programs）；
  closure_key(闭包, toolchain, dependencies, externals) → (键或 None, 拒算)；row_keyed_closure(闭包, 根, 入口文件) → 按行进键的登记表换过的闭包；unit.text() 是进 UnitKey 组成清单的单元行（嵌套路径），
  unit.dump_text() 是 dump 与命令行输出的那一行（unit-graph 的写法）；unit_graph_key(Rust 闭包, 工具链行, 依赖行) → 照 unit-graph 的 reuse_key 算的键（对拍用），
  取 sha256 的那几行是 unit_graph_key_lines（entry 行在数据行之后、工具链行之前，同 unit-graph 的 entry_key_lines；弄坏开关 unit-graph-key-entry-lines-dropped：不出 entry 行）。

判定种类（D13（验证路线） 已定项 17：剥规格与证明块只用于判产品行为的那几类判定的键，形式化判定的键保留规格）：
  formal-verification（缺省）照源码原样切、收、算，verus! 里的规格与证明都进键（scripts/gate/verus-proof-check.py、
    scripts/experiment-apparatus/e165-pin-red-examples.py 经 scripts/crash/lib_verdict_library.py 的 FunctionUnitIndex 用这一种）。
  product-behavior 是判产品行为的判定用的视图（验证包按条复用与 admission.py 算的别的单元键：KeyContext.workspace；
    crates 变异的行键与点名测试：scripts/mutation/lib_mutation_units.py 的 UnitKeys.workspace）。剥法以 crates/singlefs-unit-graph 的
    unit-graph dump <根> --judgment-kind product-behavior（那一份用 verus_syn 剥）为准，按它交的行改这一份：
    · 只动有 verus! 块的文件，与写着外属性 #[verifier::…] / #[verifier(…)] 的文件（没有 verus! 块的也算）；别的文件两个视图逐字相同。
      一个工作区里这两样文件都没有时不起 unit-graph。
    · 剥过的那一份：载入时照那一份的剥法按词法单元认同样的几截（VerusGhostCode：spec / proof / axiom 函数与常量整条、broadcast、assume_specification，
      执行函数签名里的规格子句与具名返回值的名字、Ghost / Tracked 参数与字段、函数体里的 proof 块与 assert / assume / reveal 一族、let ghost、
      循环头与闭包的子句、实参与字面量里的 Ghost(…) / Tracked(…)；另有 verus! 里外的外属性 #[verifier::…] / #[verifier(…)]，从 # 到配对的 ]，
      不剥内属性 #![verifier::…]、#[cfg_attr(…)] 里套的、落在别的宏调用与 macro_rules! 词法单元组里的，没有 verus! 块的文件也剥；剥掉打头的属性之后
      单元的起始行下挪，Rust 那一份同样下挪），换成空格（换行留着）重切；重切出来的单元与 Rust 那一份交的这份文件的单元
      逐个相同（种类、文件:起-止、指纹，限定名以嵌套路径收尾）就用它：只给 Verus 看的单元压根不切，闭包从剥过的词法单元收边（规格与证明里点到的
      执行代码不连），与 unit-graph 剥完再切、再收同一做法。核不上的、Rust 那一份解不开这份文件的，照原样那一份走下面两条。
    · 这份文件里逐单元按「种类、限定名、文件:起-止」（dump 行去掉指纹那一段）配对：配上的，指纹换成 Rust 那一份的；
      Rust 那一份没交、单元落在 verus! 块里的，记成只给 Verus 看的（Unit.ghost_only）：闭包收到它就停，不进键、不往下连（入口本身照收）；dump 不列。
    · 配不上的（Rust 交的单元这里没有、verus! 块外的单元 Rust 没交、同一配对键两边个数不同或几个指纹分不清谁对谁）记文件级拒算
      product-view-unmatched；Rust 那一份解不开的块（拒算行 verus-ghost-code-unparsed）照抄成这份文件的拒算；拿不到 Rust 那一份
      （找不到或编不出 unit-graph、它退非 0）记文件级拒算 product-view-unavailable。文件级拒算照 Rust 的口径进链接到那份文件的每个闭包。
    · 照原样那一份走的文件，闭包照原样的词法单元收边：规格与证明里点到的执行代码照收，只多不少；门禁 code-tooling 的 function-units-two-implementations-agree 格
      另有产品行为视图那一趟，逐入口比这一份与 unit-graph closure --judgment-kind product-behavior 的闭包与键。
  unit-graph 从哪来：设了 FUNCTION_UNITS_UNIT_GRAPH_DIRECTORY 起那个目录里的 unit-graph（调用方保证它是被判那一版的 crates/singlefs-unit-graph 编的；对拍格与自证用）；
    没设照这份库所在仓的 crates/singlefs-unit-graph、.claude/heavy-thread-default 与 Cargo.lock 的内容摘要找
    <跨轮缓存目录>/singlefs-unit-graph-product-view/<摘要>/unit-graph（跨轮缓存目录：设了 GATE_CROSS_RUN_TMPDIR 取它，没设取这份库所在仓的 git common-dir（git worktree 里是主仓的 .git），不在 git 仓里取 TMPDIR），没有就在那里的临时工作区经 scripts/hosts/capped.sh default
    起 cargo build --release --offline 编一份（几个进程同时要时排他锁，编完删工作区与编译目录，只留最近 3 份程序）。
  RustWorkspace 构造时取一次 Rust 那一份（fork 出来的子进程共用父进程那一份），之后载入的有 verus! 块或写着外属性 #[verifier…] 的文件照同一份改。

Rust 词法单元的缓存（rust_lex_cached）：RustFile 载入一份 .rs 先切词法单元（rust_lex 按字符逐个走，是载入整个工作区最费的一步，全仓约几秒到几十秒），
  结果只随这份文本变，而点名几行的 row-plan 每趟都是新进程、要把全仓的 .rs 重新切一遍。切出来的 (kinds, texts, lines, documents) 按文本的 sha256 存成一个 zlib 压缩的 json 文件
  （去重的词表加 base64 的序号表，不用 pickle：缓存目录在可共写的临时目录里；全仓约 20 MB），放 <跨轮缓存目录>/singlefs-rust-lex-cache/<库的内容摘要>/（同产品行为视图那一段的跨轮缓存目录）；
  FUNCTION_UNITS_LEX_CACHE_DIRECTORY 设了放那里，写 off 不用缓存（自证除缓存那一格外都设 off）。目录名里的库内容摘要把 rust_lex 与它的帮手都罩住：库的内容一变，旧目录整个不再取，
  只留最近 3 个；文件只看文本内容，不看路径、修改时刻、不设时限；读不了、写坏了、各表对不上的照常现切再存；写不成不影响结果。

跨轮缓存的登记（cross_run_cache_registry；命令行 cross-run-caches）：仓里有意跨轮复用的缓存都登记在这一处（.claude/singlefs-ai-sop/rules/command-safety.md
  「测试镜像一律放临时目录」末段：放 GATE_CROSS_RUN_TMPDIR 底下或项目定的位置，不放这一趟的 TMPDIR）。一行（名字、放它的脚本、现取它的位置的函数）；
  现取 = 调那份脚本自己算位置的那一段：python 载入那份脚本调它的函数（crates_mutation_judging_cache_directory 这一类），shell 读那份脚本里恰好一行的双引号赋值、
  照此刻的环境展开（shell_assignment_row；expand_shell_parameters 只认 $名、${名}、${名:-词}、${名-词}），不在这里另抄一份位置。
  test-environment 那一行位置分两段算（脚本里先取 GATE_CROSS_RUN_TMPDIR，没设再调 .claude/hooks/lib_work_root.py 的 shared <名>，再接文件名），不是一行赋值，
  现取 test_environment_stage_run_times_file 逐段从那份脚本里读：「local run_times_file cross_run_cache=」那一行读 GATE_CROSS_RUN_TMPDIR，没设时读脚本里恰好一处
  「python3 "$STAGE_DIR/../hooks/lib_work_root.py" shared <名>)」取名字、调同一个库（载入这份库所在仓里的 .claude/hooks/lib_work_root.py）的 shared_data_path，
  「run_times_file=」那一行照这个目录展开成文件；名字、目录、文件名都不在这里抄，哪一段脚本改了写法，对应那一段抛 ValueError、点名那份脚本，自证那一格红。登记的是：本库的 Rust 词法单元缓存
  与产品行为视图编出的 unit-graph 程序、scripts/mutation/crates-mutation-rows.py 的判法单元指纹缓存、.claude/gate.d/test-environment.sh 记的上一趟各阶段用时、
  .claude/scripts/lkmm.sh 与 .claude/scripts/fetch-deps.sh 的下载缓存。Rust 一侧 crates/singlefs-verification/src/gpu_unit_checks.rs 的 driver_shader_disk_cache_directory
  （没设 GATE_CROSS_RUN_TMPDIR 时放家目录的 .cache/singlefs 底下）取位置要编 验证包，不登记。新加一份跨轮缓存，在 cross_run_cache_registry 加一行。
  自证两格：CROSS_RUN_CACHE_CELL 把本仓 scripts/ 与 .claude/ 拷进临时目录（real_tree）、建空的 .git（在 git 仓里），另建一个空目录当 TMPDIR，摘掉 GATE_CROSS_RUN_TMPDIR
  与几份缓存自己的改放处变量（FUNCTION_UNITS_LEX_CACHE_DIRECTORY、crates-mutation-rows.py 的 JUDGING_CACHE_DIRECTORY_VARIABLE 指的那个），起子进程跑拷贝里
  这份库的命令行 cross-run-caches：子进程的 tempfile.gettempdir() 要是那个目录、登记的每一份都算得出、没有一份落在它之下。
  同一格再用 git 造一个主仓与它的 worktree（git worktree add --detach，worktree 里 .git 是文件），scripts/ 与 .claude/ 拷进 worktree 照样起一趟：
  照样不落在那一趟的 TMPDIR 之下，头一趟落在 <拷贝>/.git 底下的那几份，这一趟要落在主仓 .git（git common-dir）底下同一相对路径（与主仓共用同一份）。
  登记表写成函数（cross_run_cache_registry），不写成模块级的表：表里载入 crates-mutation-rows.py 的那一项不随 import 时执行的模块级语句进别的闭包。
  CROSS_RUN_CACHE_SCAN_CELL 扫本仓 CROSS_RUN_CACHE_SCANNED_DIRECTORIES 下（跳过 fixtures、__pycache__、target、.git）提到 GATE_CROSS_RUN_TMPDIR 的 .py / .sh / .bash：
  要么是登记表里的脚本，要么在 CROSS_RUN_CACHE_NOT_CACHES 里写明为什么不登记；不登记表里的每一份都在、都还提到它（只缩不涨）；夹具里没登记的一份判得出来。
  弄坏开关 cross-run-cache-under-tmpdir、cross-run-cache-run-times-old-spelling 与 cross-run-cache-mentions-unscanned 见下面「弄坏开关」。

拒算（3.4，认不出不静默跳过：这一条判定不复用、每趟现判）。形态名：
  Rust   unbalanced-brackets unknown-item cfg-unrecognized include-argument include-missing include-code runtime-path current-exe
         macro-generated-names proc-macro-crate extern-block module-file-missing bin-target-missing
         产品行为视图另有 product-view-unavailable product-view-unmatched verus-ghost-code-unparsed（「判定种类」）
  WGSL   unbalanced-brackets unknown-item wgsl-undefined-name
  Python python-syntax getattr-variable globals-lookup import-unresolved import-module-variable spec-path-unfolded spec-path-missing
         sys-path-unfolded subprocess-path-unfolded subprocess-path-missing subprocess-command-unfolded（命令认不出、命令位置上的程序折不出）
         data-path-unfolded（运行期读的文件路径折不出，下面「运行期读的数据文件」）
  shell  bash-syntax shell-path-unfolded shell-path-missing shell-path-outside-root shell-command-unfolded（命令位置上的程序折不出）
  公共   external-program-unkeyed inline-script-not-literal（-c 的串、喂给标准输入的脚本折不出，或标准输入从哪来认不出）
  起代码的写法逐种认（shell 里每一个解释器词，python 里每一次起子进程），每一种要么连边、要么拒算：
    解释器 + 脚本：连到那份脚本；脚本之后接着往后认（bash capped.sh N python3 x.py、run-with-memory-cap.sh 上限 bash y.sh 里的第二个解释器）。
    -c 的串、- / /dev/fd/N / bash -s 喂的 here-doc、here-string（含 { … } 3<<'词' 写在组后面的）：正文字面的、或折得出的
      （$变量、$(cat <<'词' …)、$(只打字面行的函数)）当内联脚本：python 的切成单元（不进闭包，穿过它连下去；sys.argv、命令前设的环境变量
      折成那条命令里的词），bash 的切开再认；折不出拒算 inline-script-not-literal。内联 python exec sys.argv[k:] 时第 k 个词起照命令认。
      拆包切片（a, b = sys.argv[1:3]，下标是不小于 0 的整数字面量、不带步长，上界写了就等于下界加目标个数）逐个对上 a = sys.argv[1]、b = sys.argv[2]，
      别的序列的切片照同一条对上（paired_elements）；上界对不上、带步长、下标折不出的照旧不对上（该拒算的照拒算）。
    -m <模块>：仓里的连到程序入口；-m unittest / pytest 点名的测试模块整份连上；标准库与外部的不装仓里的代码。
    --version、--help、python3 -V、bash -n：不跑代码。参数位置上光一个解释器名（command -v python3）：不是要跑的命令。
    命令位置（经 env、nice、timeout、sudo、xargs、exec 这几个包装之后）：仓里的脚本连上；折不出的程序词拒算，
      除了 "$@" "$1" 一类（命令就是调用处那几个词）、"前缀_$1"（按名字前缀连看得见的函数）、"${数组[@]}"（按数组的字面赋值摊开）、
      「$目录/<不带脚本后缀、根里没有的字面尾巴>」与 shutil.which(…)（根外的程序）。
    shell 里解释器后面的脚本、source 的路径另认这几种（ShellIndex.resolve_invocation）：
      $PWD 照这份文件脚本体顶层恰好一处的 cd 折（cd "$(dirname "$0")/../.." 一类；working_directory_of；脚本体自己里的 $PWD 不折）；
      ${名%词} ${名%%词} ${名#词} ${名##词} 的词也折得出时照 bash 去掉后缀、前缀（remove_shell_pattern）；
      cd "$目录/<字面尾巴>" 折不出而尾巴在根里是一个目录：照根里那个目录认（tree_copy_directories，临时树、仓副本里的同一相对路径）；
      cd 到折不出的目录之后的相对路径（bash scripts/x.sh，至少两段）：根里同一相对路径那一份；cd 到相对目录的，那个目录照根认（target_of）；
      「<折得出的根里的目录>/<带展开的文件名>」：那个目录里每一份同种脚本（python 起 .py，bash 与 source 起 .sh、.bash；directory_wildcard_targets）；
      函数里的 "$@" "$1" "${@:2}"：脚本由调用处给，这份文件与 source 它的几份里每一处以函数名起头的命令取对应的那个词，在调用处折
      （positional_scripts；调用处又是位置参数的往上找，至多 3 层），有一处折不出、一处调用都没有的照旧拒算 shell-path-unfolded。
      「$<折不出的目录>/<折得出的尾巴>」（尾巴里的变量照赋值折："$ROOT/$SAMPLE_DIRECTORY/driver.sh"）：仓里以这一截收尾的脚本整组连上
      （shell_tail_group，同 python 的 script_tail_group）；一份都没有的不是仓里的脚本，不连也不拒算。弄坏开关 shell-script-tail-groups-ignored 下照旧拒算。
    shell 的函数切分（ShellFile.parse）：函数头与收尾的 } 不在引号与命令替换里认（rows_inside_quotes：多行 awk 程序、内联脚本照原样留着缩进，
      它们的「    }」与 declare -f 打的函数收尾长得一样）。弄坏开关 shell-function-bounds-quote-blind 下照旧按行认，函数切在引号里、余下的正文当命令认。
    shell 起法带字面参数：bash <脚本> <词…> 的词全是字面的（python 起 shell 的、程序入口带参数时照它带过去的参数），脚本体照
      narrowed_shell_body 剪过的正文收边：头一处 shift、set --、循环之前的 "$1" "${1:-}" 一类代成字面参数（引号里照引号转义、单引号与 here-doc 里的不代），
      代过之后判得出真假的 if [[ <字面> == / != <字面> ]]（不带 elif）、case <字面> in（只留头一个对得上的分支）走不到的那几行不收边；
      脚本体单元与指纹照原来那一份（键照整份脚本体算），只剪边。有一处起法带了折不出的词，那份脚本体整份收（与 python 一侧「程序入口带参数」同一条规则）。
      弄坏开关 shell-program-arguments-unnarrowed 下不剪。
    python 的命令：字面列表、a + b、x if c else y、*展开、本函数里的赋值与 .append / .extend / +=、函数参数（本模块每一处调用给的实参）、
      调仓内函数的返回值都折；shell=True 与 os.system 的串当内联 bash；认不出拒算 subprocess-command-unfolded。另认：
      套在函数里的函数自成一层作用域（PythonModule.nested_unit / scope_of）：它的局部赋值与参数照它自己的，参数按外层函数里调它的那几处折，别的名字到外层找；
      a + b 有一边认不出那一段占一格（同 *展开认不出），命令位置落在它上面照旧拒算；f(…)[k] 取 f 各处 return 的元组第 k 项，x[a:b] 照切
      （b 可以是 x.index(<字面词>)：到头一个等于它的词之前，切点前折不出的词能断定不等于它的照过）；env 后面 f"{名字}={折不出}" 名字折得出照一条赋值认；
      类里方法的参数按本模块每一处 <对象>.<方法>(…) 折；推导式的循环变量取遍它迭代的元素（序列可以是调仓内函数的返回值、{…}.get(键, 缺省)）；
      只赋过一次的元组 / 列表字面量[整数]；spec_from_file_location 载入的仓内模块的 module.NAME；pool.submit(f, 实参) 算对 f 的一处调用；
      命令位置上折出几个取值的，每个取值当一种取法各认一遍。
      解释器后面的脚本折不出、只认得出字面尾巴（os.path.join(<折不出的目录>, "tools", "runner.py")）：仓里以这一截收尾的文件整组连上
      （script_tail_group）；一份都没有的不是仓里的脚本（这一份自己写出来的夹具、根外的程序），不连也不拒算；连尾巴都认不出的照旧拒算 subprocess-path-unfolded。
      cwd= 折不出、脚本写的是相对路径（["bash", "setup.sh"], cwd=<拷出来的样本>）同样按 /<它> 收尾整组连上。
      版本询问（[<折不出的程序>, "-V"]、"--version"、"-vV"，命令里只有这一个词）：程序只打版本、不跑仓里的代码，不连、不拒算，记成工具链询问
      （Closure.toolchain_queries）；它的输出归工具链：收闭包的一侧自己折，调用方不另做——闭包里收一个工具链单元（语言 data、种类 rust-toolchain，
      文件与行是询问那一处，指纹是 rust_toolchain_lines 那几行的 sha256：根里起的 PATH 上 rustc -vV 与 cargo -V，环境变量 RUSTC、CARGO_BUILD_RUSTC
      与根往上每一层、CARGO_HOME 下 cargo 配置的 build.rustc 指的编译器各自的 -V，起不来的记一行「起不来」），照数据单元进键的 data 行，
      并起来的闭包（lib_verdict_library.py 的 FunctionUnitIndex.key）照单元带过去。闭包里没有询问的不收它，换 rustc 键不变。
      弄坏开关 python-toolchain-queries-refused 下照旧拒算命令折不出；python-toolchain-queries-unkeyed 下不收工具链单元（换 rustc 键不变）。
      调用方交进来的命令（refuse_launch）：命令、-c 的串、脚本路径折不出，折不出的那几截（unfolded_part_origins）出处都推得出是入口函数的参数或 sys.argv、
      argparse 解出来的属性、临时目录、根外前缀（调用方交的那一条命令，由请求那一侧的单元键与 line:command= 管）：不拒算；别的函数的参数照闭包里每一处
      调它的实参接着往上认（闭包里没人调它的推不出）；出处是环境变量、推不出的照旧拒算。是不是入口在收闭包时判（ScriptClosure.caller_launch_refusals）。弄坏开关 python-caller-launches-refused 下一律照旧拒算。
      别的函数的参数在每个调用点照实参折（ScriptClosure.launch_argument_supplied）：折得出的字面值是根里的 .py、.sh 就当程序连上（连完再判一轮新收进来的），
      python3、bash 这类解释器词不算折出（脚本在命令的别处）；根外的程序（outside_head）、调用处那一层的参数（接着往上一层认）、本函数里的赋值、
      外层函数的变量（套着的函数）、调仓内函数的返回值（在它那边逐个认）照样认。弄坏开关 python-launch-arguments-unfolded 下只认调用方交进来的那几样。
      登记表点名的程序（REGISTRY_LISTED_PROGRAMS）：起子进程的模块在模块级写一张只赋过一次的字典字面量
      REGISTRY_LISTED_PROGRAMS = {"<函数名>": (<登记表相对根的路径>, <正则，头一组是相对根的脚本路径>)}，那个函数里脚本路径折不出、登记表现读得出对得上的脚本的，
      起的是环境探针：探针打的是这台机器上的工具版本、退出码是开跑前的前提，结局由调用方处置（admission.py 的 gate-preconditions 每趟现判、不复用；
      environment= 打的版本经请求项 environment-of: 进各条判定的键），不连、不拒算、登记表不进键（探针连进来是整道门禁阶段的脚本体，
      连带它起的一切把每个够到 run_probe 的闭包撑大、拒算）。登记表读不了的拒算 subprocess-path-missing。只认这一张表、只认写在里面的函数。
      弄坏开关 python-registry-programs-ignored 下照旧拒算 subprocess-path-unfolded；python-registry-programs-linked 下登记表点名的每一份都连、登记表收成 data-file。
    命令替换 $( … ) 与 `…`、进程替换 <( … ) 里的命令照命令认；[[ … ]]、(( … ))、case 分支的模式、数组字面量 x=( … ) 里的词不当命令。
    文件头登记的 check 判据（# admission: check / # run-condition: check，上游 preflight 开跑前在仓根 bash -c 跑它们）：从声明它的脚本的
      程序入口当内联 bash 连；上游 preflight.py 里跑它们的那一处（bash -c declaration.command）因此不拒算。
  文件级的（括号配不上、认不出的条目、语法错）：Rust 在那份文件进了入口的链接集时算进来，Python / shell 在闭包碰到那份文件时算进来；
  单元级的在那个单元进了闭包时算进来。dump 把扫到的全部列出来。

弄坏开关（FUNCTION_UNITS_BREAK=<项>[,<项>…]，只给自证证红用：--selftest 本身与它起的子进程认，平常跑设了退 2、点名只在自证里认；
自证逐个打开、核点名的那几格转红，点名的格就是 BREAK_CELLS 里这一项的那几句，自证逐条打「✓ 弄坏开关 <项> 下「<格>」红」）：
  FUNCTION_UNITS_BREAK=rust-open-entry-traits-ignored  去掉 R-开放（入口签名里的 dyn 与 trait 约束不连全部 impl）
      「开放入口：参数是 dyn」「R-开放：dyn Trait 只带 Trait 自己的 impl，不带只在 trait 参数里写了它的 impl」
  FUNCTION_UNITS_BREAK=rust-dyn-traits-open-every-implementation  闭包里单元的 dyn Trait 照旧连这个 trait 的全部 impl（CHA）
      「dyn 分派：闭包里的 dyn 不连没被点名的另一个 impl，改它不作废」
  FUNCTION_UNITS_BREAK=rust-constants-dropped  const 与 static 单元不进闭包
      「关联常量（经泛型）」「函数指针表」「数组长度里的常量」
  FUNCTION_UNITS_BREAK=rust-macros-dropped  macro 与 item-macro 单元不进闭包
      「声明宏展开出的函数」「带路径的条目级宏调用定义的枚举：改了作废」
  FUNCTION_UNITS_BREAK=rust-trait-impl-edges-dropped  去掉 R-隐式（类型被点名时给它写的 trait impl）
      「运算符重载」「Drop 隐式调用」「关联常量（经泛型）」
  闭包外的四类边（自证里 closure_edge_cells 那几格，文件头 Rust 闭包「闭包外的四类边」）：
  FUNCTION_UNITS_BREAK=rust-macro-generated-types-unnamed  条目级宏调用生成的类型不算被点名
      「R-宏：条目级宏调用里 enum 定义的类型算被点名，.方法() 连到写给它的 impl，改方法作废」
  FUNCTION_UNITS_BREAK=rust-macro-rules-literal-names-unlinked  macro_rules! 体里按字面写出的名字不登到调用名下
      「R-宏：macro_rules! 体里按字面写出的常量名登到这个宏的每次调用名下，点名它的入口连上调用，改调用体作废」
  FUNCTION_UNITS_BREAK=rust-global-allocators-unlinked  #[global_allocator] 的 static 不进闭包
      「#[global_allocator]：链接集里的分配器 static 进闭包，连上它的 GlobalAlloc impl，改 realloc 作废」
  FUNCTION_UNITS_BREAK=rust-foreign-self-trait-impls-unlinked  外部类型上的本仓 trait impl 不经 R-方法、R-路径连
      「R-方法：本仓 trait 写在外部类型上的 impl（impl Tr for std::time::Duration）经泛型约束调到，方法连上，改它作废」
  FUNCTION_UNITS_BREAK=rust-build-environment-unlinked  env!("X")（X 不是 cargo 给的）不连构建脚本的 main
      「构建脚本经 cargo::rustc-env 交的值：用 env! 读它的单元连到本 crate 构建脚本的 main，改 main 里算的值作废」
  FUNCTION_UNITS_BREAK=rust-excluded-path-dependencies-dropped  不顺 path 依赖收工作区外的 crate
      「工作区外的 path 依赖（根清单 exclude 里）：顺 path 依赖收进来，调到它的函数连上，改它作废」
  FUNCTION_UNITS_BREAK=rust-outside-crate-names-linked-unimported  别的包里的裸名照旧按同名连进工作区外的 crate，不看引没引进
      「工作区外的 crate 按引进的名字连：别的包里只是同名的字段 halted 不连到它的 fn halted，改它键不变；use 引进的 restarts、use 引进的模块下的
      control::paused、写成路径的 stopped_connections 照连，改它们键变」
  FUNCTION_UNITS_BREAK=rust-outside-crate-names-linked-across-modules  工作区外 crate 同一个目标里的裸名照旧按同名连到各个模块
      「工作区外的 crate 里按模块连：use 引进 log::Log 的只连它、不连 bench 模块里同名的 Log，Worker::launch 自己的名字不连 bench::launch；
      改 bench 模块里这两样键不变，改 log::Log、改 Worker::launch 键变」
  FUNCTION_UNITS_BREAK=rust-crates-names-linked-across-modules  crates/ 下的包同一个目标里的裸名照旧按同名连到各个模块
      「R-裸名：crates/ 下的包同一个目标里按模块连：经 use 引进 format::SLOT_BYTES 的单元不连 gpu_checks 里私有的同名常量，改它键不变」
  FUNCTION_UNITS_BREAK=rust-verus-segments-ignored  verus! 里照普通 Rust 切（签名后第一个 { 当函数体）
      「verus! 里 ensures 带花括号：下一个函数照样切开，改它作废」「verus! 里够不到的函数改了不作废」
  FUNCTION_UNITS_BREAK=rust-cfg-ignored  cfg 一律当真
      「cfg(feature) 关着的单元改了不作废」「认不出的 cfg 谓词拒算」
  FUNCTION_UNITS_BREAK=rust-doc-comments-kept  文档注释进词法单元
      「只改文档注释不作废」「改文档注释里的代码块：入口 fn 不作废」
  FUNCTION_UNITS_BREAK=wgsl-raw-text  include_str! 读的 .wgsl 取整份字节、不切单元
      「WGSL 只改注释不作废」「WGSL 够不到的函数改了不作废」
  FUNCTION_UNITS_BREAK=python-comments-kept  def 的指纹取原文（带注释）
      「python：只改注释不作废」
  FUNCTION_UNITS_BREAK=python-docstrings-kept  文档串不从指纹里去掉
      「python：改文档串不作废」
  FUNCTION_UNITS_BREAK=python-docstring-readers-ignored  读 __doc__ 不收文档串数据单元
      「python：有单元读 __doc__ 时改文档串作废」
  FUNCTION_UNITS_BREAK=python-getattr-allowed  getattr(x, <变量>) 不拒算
      「python：getattr 变量名拒算」
  FUNCTION_UNITS_BREAK=python-spec-paths-ignored  spec_from_file_location 不折叠、不拒算
      「python：spec_from_file_location 字面路径折叠出跨模块边」
  FUNCTION_UNITS_BREAK=python-attributes-unnarrowed  obj.m 连到全部同名方法（不按闭包里出现过的类收窄）
      「python：obj.方法 按闭包里出现过的类收窄」
  FUNCTION_UNITS_BREAK=shell-heredoc-split  here-doc 正文照样按缩进切
      「shell：here-doc 正文里有一行 `};` 不许被切开」
  FUNCTION_UNITS_BREAK=shell-names-command-position-only  函数名只在命令位置上才连
      「shell：函数名当参数传照样连上」
  FUNCTION_UNITS_BREAK=shell-unfolded-paths-ignored  折不出的路径不拒算
      「shell：`bash "$x"` 折不出拒算」
  FUNCTION_UNITS_BREAK=shell-raw-text  脚本体的指纹取原文
      「shell：脚本体只改注释不作废」
  FUNCTION_UNITS_BREAK=external-programs-unchecked  闭包里点名的外部程序不要求调用方给
      「键：点名了 cargo 而调用方没给 --external 拒算、给了出键」
  FUNCTION_UNITS_BREAK=key-entry-units-dropped  键里不出 entry 行（几个入口互相收进对方的闭包时共用一把键）
      「键：两个互相收进对方闭包的入口，键不同（一个 trait impl 里的两个方法、python 里互相调用的两个 def：闭包的单元多重集相同，键各是各的）」
  FUNCTION_UNITS_BREAK=key-entry-doctest-owner-chained  entry 行里文档测试照别的单元沿 Unit.container 往外取所挂条目的指纹（改所挂条目的函数体，文档测试入口的键跟着变）
      「R-结构：文档测试不连所挂的条目与那个模块的 use」
  FUNCTION_UNITS_BREAK=unit-graph-key-entry-lines-dropped  unit_graph_key（与 unit-graph 对拍的那一把）不出 entry 行（Rust 那一份的 reuse_key 带了，两边的键对不上）
      「键：unit_graph_key 照 crates/singlefs-unit-graph 的 reuse_key 带 entry 行（数据行之后、工具链行之前，排序去重；impl 方法入口连 impl 头；文档测试入口记所挂条目的全部文档测试、各一行，闭包从它们一起收）」
      「键：夹具上 unit_graph_key 与 crates/singlefs-unit-graph 的 unit-graph closure 打的键逐字相同（一个 impl 方法入口、一个文档测试入口；Rust 那一侧是录下的键 ENTRY_LINES_RUST_KEYS）」
  FUNCTION_UNITS_BREAK=rust-doctest-entry-alone  文档测试入口只取交进来的那一个文档测试（Rust 那一份取所挂条目的全部文档测试，entry 行与闭包对不上）
      「键：unit_graph_key 照 crates/singlefs-unit-graph 的 reuse_key 带 entry 行（数据行之后、工具链行之前，排序去重；impl 方法入口连 impl 头；文档测试入口记所挂条目的全部文档测试、各一行，闭包从它们一起收）」
      「键：夹具上 unit_graph_key 与 crates/singlefs-unit-graph 的 unit-graph closure 打的键逐字相同（一个 impl 方法入口、一个文档测试入口；Rust 那一侧是录下的键 ENTRY_LINES_RUST_KEYS）」
  FUNCTION_UNITS_BREAK=cross-run-cache-under-tmpdir  cross_run_cache_directory 没设 GATE_CROSS_RUN_TMPDIR 时放这一趟的 TMPDIR 底下（缓存被一例自证的临时目录带走）
      「真仓：跨轮缓存登记表（cross_run_cache_registry）里的每一份，在 TMPDIR 指到一个临时目录、GATE_CROSS_RUN_TMPDIR 没设、在 git 仓里（.git 是目录，或 git worktree 里 .git 是文件）时现取的位置都不落在 TMPDIR 之下；worktree 里落在 git 目录底下的几份与主仓共用 common-dir 那一处」
  FUNCTION_UNITS_BREAK=cross-run-cache-git-directory-only  git_common_directory_of 只认 <仓>/.git 是目录（worktree 里 .git 是文件，本库两份与
      crates-mutation-rows.py 的判法单元指纹缓存退回这一趟的 TMPDIR、每趟重算）
      「真仓：跨轮缓存登记表（cross_run_cache_registry）里的每一份，在 TMPDIR 指到一个临时目录、GATE_CROSS_RUN_TMPDIR 没设、在 git 仓里（.git 是目录，或 git worktree 里 .git 是文件）时现取的位置都不落在 TMPDIR 之下；worktree 里落在 git 目录底下的几份与主仓共用 common-dir 那一处」
  FUNCTION_UNITS_BREAK=cross-run-cache-run-times-old-spelling  test-environment 那一行只认「local run_times_file=」起头的写法（脚本把取目录与赋值拆开，这种写法在脚本里一行都找不到，现取抛 ValueError）
      「真仓：跨轮缓存登记表（cross_run_cache_registry）里的每一份，在 TMPDIR 指到一个临时目录、GATE_CROSS_RUN_TMPDIR 没设、在 git 仓里（.git 是目录，或 git worktree 里 .git 是文件）时现取的位置都不落在 TMPDIR 之下；worktree 里落在 git 目录底下的几份与主仓共用 common-dir 那一处」
  FUNCTION_UNITS_BREAK=cross-run-cache-mentions-unscanned  cross_run_cache_mentions 一份都不扫（提到 GATE_CROSS_RUN_TMPDIR 的脚本漏登记也看不出）
      「真仓：提到 GATE_CROSS_RUN_TMPDIR 的脚本都登记进跨轮缓存表或在不登记表里写明为什么，不登记表里的每一份都在、都还提到它；夹具里没登记的一份判得出来」
  FUNCTION_UNITS_BREAK=rust-format-captures-ignored  串字面量里的 {名}（format! 一族的内联捕获）不算裸名
      「format! 串里内联捕获的常量改了作废」「WGSL 够得到的函数改了作废」
  FUNCTION_UNITS_BREAK=shell-variable-first-line-only  顶层赋值只取第一行（$(cat <<'词' … 词) 的正文不进那个变量单元）
      「shell：顶层变量里 here-doc 的正文改了，读它的函数作废」
  FUNCTION_UNITS_BREAK=tree-copy-tails-ignored  「折不出的目录 + 根里存在的相对路径」（临时树、仓副本里的同一份）不按根里那一份认（shell 与 python）
      「shell：`bash "$tree/<仓里的相对路径>"` 连到仓里那一份」「python：os.path.join(<折不出的目录>, <仓里的相对路径>) 连到仓里那一份」
  FUNCTION_UNITS_BREAK=git-repository-paths-unfolded  git rev-parse --show-toplevel / --git-common-dir 不折成根（shell 的 $(…) 与 python 的 subprocess …stdout）
      「shell：git rev-parse --git-common-dir 折成根，经它 source 的库连上」「python：git rev-parse --git-common-dir 经无参函数的返回值折成根，插进 sys.path 的目录不拒算」「preflight 装载器：只拷了 .claude/ 的小树里 source 与 import 都解到树里的规范副本，两份装载器的闭包不拒算」
  FUNCTION_UNITS_BREAK=python-call-returns-unfolded  不带参数调仓内模块级函数，不按它各处 return 的值折
      「python：git rev-parse --git-common-dir 经无参函数的返回值折成根，插进 sys.path 的目录不拒算」「preflight 装载器：只拷了 .claude/ 的小树里 source 与 import 都解到树里的规范副本，两份装载器的闭包不拒算」
  FUNCTION_UNITS_BREAK=shell-redirect-targets-loaded  重定向的目标当成一条命令留下（写数据的路径被当成装代码）
      「shell：重定向到变量路径不拒算」
  FUNCTION_UNITS_BREAK=shell-path-arguments-loaded  参数位置上长得像脚本的路径也当装代码
      「shell：把路径当参数传给自己的函数不拒算」
  FUNCTION_UNITS_BREAK=python-command-arguments-loaded  subprocess 命令里脚本后面的路径参数也当装代码
      「python：写数据的路径与传给 subprocess 的参数不拒算」
  FUNCTION_UNITS_BREAK=shell-generated-scripts-ignored  单元自己 printf / echo / cat <<词 写出来再起的脚本不认
      「shell：自己用 cat <<词 写出来再起的脚本不拒算、正文里装的连上」
  FUNCTION_UNITS_BREAK=shell-inline-positionals-ignored  bash -c '…' <名字> <参数…> 里的 $1…$9 不换成外面那几个词
      「shell：bash -c '…"$1"…' <名字> <路径> 里的 $1 换成那个路径」
  FUNCTION_UNITS_BREAK=shell-inline-positionals-escapes-ignored  外层 bash -c 的串里转义的 \\$1 与单引号里的 $1 也换成外面那几个词（
      套着的里层 bash -c "…\\$1…" 拿到的是外层的 $1，内存包装自证里 python3 -c "\\$1" 折成了包装自己的路径、拒算 python-syntax）
      「shell：bash -c '…' 里套着的里层 bash -c "…\\$1…"：外层只换没转义的 $1…$9，转义的 \\$1 留给里层照它自己的参数折（里层的 $1 是外层的 $2），连到它 import 的仓内模块」
  FUNCTION_UNITS_BREAK=shell-copied-scripts-ignored  单元自己 cp <源> <它> 拷出来再起的脚本不连到源
      「shell：自己 cp 出来再起的脚本连到源那一份」
  FUNCTION_UNITS_BREAK=shell-enclosing-locals-ignored  套在外层函数里的函数看不见外层的 local
      「shell：套在外层函数里的函数看得见外层的 local」
  FUNCTION_UNITS_BREAK=shell-cd-first-directory-only  cd 过去的目录折出几个时，相对路径只按 target_of 找一处
      「shell：cd 过去的目录折出几个，相对路径在每一个底下都连」
  FUNCTION_UNITS_BREAK=shell-data-sources-unfollowed  Rust 闭包读到的 shell 脚本只收它自己的字节，不沿 source / . 收
      「Rust 闭包读到的 shell 脚本：它 source 进来的文件收成 data-file 单元，改一处读这份脚本的入口键变、读另一份不 source 它的脚本的入口键不变；被 source 的每一份按读这份脚本的那一处记一条数据位置，unit_graph_key（与 crates/singlefs-unit-graph 对拍的那一把）随它变」
      「Rust 闭包读到的 shell 脚本：. <路径> 照 source 收（路径经 $(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd) 赋的变量），改被引的文件键变」
      「Rust 闭包读到的 shell 脚本：两层（脚本 source 的库再 source 一份）逐层收，改里层那份键变」「Rust 闭包读到的 shell 脚本：source 的路径折不出拒算 shell-path-unfolded」
  FUNCTION_UNITS_BREAK=shell-data-sources-unsited  Rust 闭包读到的 shell 脚本 source 进来的文件照收进闭包，不按读那份脚本的那一处记 data_sites（
      对拍的闭包行与 unit_graph_key 不带它们，crates/singlefs-unit-graph 那一份带，门禁 code-tooling 的 function-units-two-implementations-agree 报「只在 Rust 闭包里」）
      「Rust 闭包读到的 shell 脚本：它 source 进来的文件收成 data-file 单元，改一处读这份脚本的入口键变、读另一份不 source 它的脚本的入口键不变；被 source 的每一份按读这份脚本的那一处记一条数据位置，unit_graph_key（与 crates/singlefs-unit-graph 对拍的那一把）随它变」
  FUNCTION_UNITS_BREAK=shell-data-sources-nested-unfollowed  Rust 闭包读到的 shell 脚本只收它自己那一层 source 的，被 source 的那几份再 source 的不收
      「Rust 闭包读到的 shell 脚本：两层（脚本 source 的库再 source 一份）逐层收，改里层那份键变」
  FUNCTION_UNITS_BREAK=shell-dot-source-unrecognized  . <路径> 不当 source 认（shell 闭包与 Rust 闭包读到的 shell 脚本都波及）
      「Rust 闭包读到的 shell 脚本：. <路径> 照 source 收（路径经 $(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd) 赋的变量），改被引的文件键变」
  FUNCTION_UNITS_BREAK=shell-data-source-refusals-dropped  Rust 闭包读到的 shell 脚本里 source 的路径折不出不拒算
      「Rust 闭包读到的 shell 脚本：source 的路径折不出拒算 shell-path-unfolded」
  FUNCTION_UNITS_BREAK=python-dispatch-tables-ignored  字典字面量里当值写的函数名不连边
      「python：分派表（字典字面量）里的函数改了作废」
  FUNCTION_UNITS_BREAK=python-module-constants-unfolded  import 进来的仓内模块常量 module.NAME 不折
      「python：import 进来的模块常量（module.NAME）折进路径，插进 sys.path 的目录连上」
  FUNCTION_UNITS_BREAK=python-getattr-stdlib-refused  getattr(<标准库或外部模块>, 变量) 照样拒算
      「python：getattr(<标准库模块>, 变量名) 不拒算（够不到仓里的代码）」
  FUNCTION_UNITS_BREAK=python-subscript-targets-rebind  表[k] = … 当成重新绑定 k
      「python：表[路径] = … 不当成重新绑定路径，spec 载入照样折得出」
  FUNCTION_UNITS_BREAK=python-environment-defaults-unfolded  os.environ.get(…) or <缺省> 不按缺省折
      「python：os.environ.get(…) or <缺省路径> 按缺省折，连到那份脚本」
  FUNCTION_UNITS_BREAK=python-written-scripts-ignored  单元自己 open(…, "w") 写出字面文本再起的脚本不认
      「python：自己写出字面文本再起的脚本不拒算」
  FUNCTION_UNITS_BREAK=python-loop-variables-unfolded  for 循环变量不取遍元组 / 列表的元素
      「python：for 循环变量取遍元组里的目录，插进 sys.path 不拒算、各目录里的同名模块都连」
  FUNCTION_UNITS_BREAK=python-imports-first-directory-only  同一个模块名在 sys.path 插的几个目录里各有一份时只连第一份
      「python：for 循环变量取遍元组里的目录，插进 sys.path 不拒算、各目录里的同名模块都连」
  FUNCTION_UNITS_BREAK=python-import-names-unfolded  importlib.import_module(<折得出的名字>) 照样拒算
      「python：importlib.import_module(<模块级常量>) 折出模块名」
  FUNCTION_UNITS_BREAK=python-path-tables-unfolded  路径表（模块级字典字面量）[键] 不折
      「python：路径表（字典字面量）[键] 取遍表里的路径，spec 载入连上」
  FUNCTION_UNITS_BREAK=python-comprehension-scope-leaks  推导式里的循环变量算成函数里的赋值
      「python：推导式里的循环变量不算函数里的赋值，for 取遍元组拼出的脚本连上」
  FUNCTION_UNITS_BREAK=python-directory-wildcards-ignored  os.path.join(<仓里的目录>, <折不出的文件名>) 起的脚本不连到那个目录里的每一份
      「python：os.path.join(<仓里的目录>, <折不出的文件名>) 起的脚本连到那个目录里每一份」
  FUNCTION_UNITS_BREAK=inline-scripts-ignored  -c 的串、喂给标准输入的脚本不当内联脚本认（回到既不连边也不拒算）
      「shell：python3 -c '<字面串>' 当内联脚本切，sys.argv 折成后面的词，连到它 import 的仓内模块」「shell：python3 - <<'PY' 喂的 here-doc 当内联脚本切，连到它 import 的仓内模块」「shell：python3 -c / bash -c 的串带折不出的展开拒算 inline-script-not-literal」「shell：bash -c "$串" 的串折得出就切开再认，连到它起的脚本」「python：[sys.executable, "-c", <字面串>, …] 当内联脚本切，连到它 import 的仓内模块」「python：-c 的串折不出（出处是环境变量）拒算 inline-script-not-literal」
  FUNCTION_UNITS_BREAK=shell-later-interpreters-ignored  一条命令里只认第一个解释器
      「shell：一条命令里第二个解释器（bash 包装脚本 N python3 x.py）照第一个的规则认，连到 x.py」
  FUNCTION_UNITS_BREAK=shell-substitutions-ignored  命令替换 $( … ) 与进程替换 <( … ) 里的命令不认
      「shell：起同目录脚本连边（写在命令替换里：x="$(bash 同目录/y.sh)"）」「shell：< <(命令 "$x") 里的命令照命令认，前面那个 < 不吞掉它的第一个词」
  FUNCTION_UNITS_BREAK=external-program-words-unfiltered  带空白的串、消息命令（echo / printf / die / bad / howto）后面的词照样当外部程序名
      「shell：die 消息里出现 qemu-system-x86_64 不算外部程序」
  FUNCTION_UNITS_BREAK=python-modules-ignored  python3 -m <模块> 不认
      「shell：python3 -m <仓内模块> 连到那份模块的程序入口」「python：-m unittest <测试模块> 整份连上（测试方法改了作废）」
  FUNCTION_UNITS_BREAK=shell-unfolded-commands-allowed  命令位置上折不出的程序不拒算
      「shell：命令位置上折不出的程序拒算 shell-command-unfolded」
  FUNCTION_UNITS_BREAK=shell-wrapped-commands-ignored  包装命令（env、nice、timeout…）后面直接执行的脚本不认
      「shell：包装命令（timeout 60）后面直接执行的脚本连上」
  FUNCTION_UNITS_BREAK=shell-command-structure-ignored  [[ … ]]、(( … ))、case 模式、数组字面量照普通命令切，{ … } 后面的重定向不管组里的命令
      「shell：[[ … || … ]]、(( … ))、case 分支的模式、数组字面量里的词不当命令」「shell：{ python3 /dev/fd/3 …; } 3<<'PY'（重定向写在组后面）照内联脚本切」
  FUNCTION_UNITS_BREAK=shell-subscript-assignment-split  名字[下标]=值 的下标里有空格时按空格切开，前半截当命令
      「shell：名字[下标]=值、名字[下标]+=值（下标里带空格与展开）照赋值认、不当命令」
  FUNCTION_UNITS_BREAK=shell-array-commands-unfolded  "${数组[@]}" 当命令时不按字面赋值摊开
      「shell："${数组[@]}" 当命令时按数组的字面赋值摊开，连到它起的脚本」
  FUNCTION_UNITS_BREAK=shell-heredoc-captures-unfolded  $(cat <<'词' …) 与 $(只打字面行的函数) 不折成正文
      「shell：python3 -c "$(打印正文的函数)" 折出函数打的那几行当内联脚本」
  FUNCTION_UNITS_BREAK=declared-checks-ignored  文件头登记的 check 判据不连、上游 preflight 跑它们的那一处照样拒算
      「shell：文件头登记的 check 判据（preflight 开跑前跑）照内联 bash 认，连到它起的脚本」「python：上游 preflight 跑文件头登记的判据（bash -c declaration.command）不拒算，判据从声明它的脚本连」
  FUNCTION_UNITS_BREAK=inline-argv-commands-ignored  内联 python exec sys.argv[k:] 时不把后面的词当命令
      「shell：内联 python exec sys.argv[1:] 时后面那几个词照命令认」
  FUNCTION_UNITS_BREAK=python-unfolded-heads-allowed  python 命令位置上折不出的程序不拒算
      「python：变量解释器折不出（出处是环境变量）拒算 subprocess-command-unfolded」
  FUNCTION_UNITS_BREAK=python-commands-literal-only  python 的命令只认字面列表与只赋过一次列表的名字（认不出的不拒算）
      「python：命令是 [解释器] + 参数（按调用处给的列表折）连到那份脚本」「python：认不出的命令（对象的属性）拒算 subprocess-command-unfolded」
  FUNCTION_UNITS_BREAK=python-path-programs-refused  shutil.which(…) 与 os.path.join(<折不出的目录>, <根外的字面尾巴>) 起的程序照样拒算
      「python：shutil.which(…) 与 os.path.join(<折不出的目录>, "bin/x") 起的根外程序不拒算」「python：内联脚本起 sys.argv[k]（取的是起它那份脚本的 "$1"）不拒算：程序是调用处给的词」
  FUNCTION_UNITS_BREAK=python-command-dictionaries-unfolded  命令里的 字典[<字面键>] 不折回字典字面量
      「python：命令里的 *选项["键"] 折回调用处给的字典字面量，连到那份脚本」
  FUNCTION_UNITS_BREAK=python-dispatch-calls-ignored  分派表[键](…) 不算对表里那几个函数的调用（参数折不到那一处调用给的实参）
      「python：经分派表 表[键](…) 调到的函数，命令参数按那一处调用折」
  FUNCTION_UNITS_BREAK=python-nested-scopes-unfolded  套在函数里的函数不自成作用域（它的局部变量、参数照外层函数找，折不出）
      「python：套在函数里的函数起子进程，它的局部变量与外层的变量照那一层作用域折，连到那份脚本」
  FUNCTION_UNITS_BREAK=python-command-concatenation-unfolded  命令 a + b 有一边认不出就整条认不出
      「python：git + step（step 是循环里取的、折不出）命令位置上是 git，不拒算」
  FUNCTION_UNITS_BREAK=python-command-slices-unfolded  命令里的 f(…)[k] 与 x[a:b] 不认
      「python：f(…)[k] 取返回的元组那一项、command[:command.index(<字面词>)] 照切，连到那份脚本」
  FUNCTION_UNITS_BREAK=python-command-heads-split-ignored  命令位置上折出几个、其中有解释器的照旧拒算
      「python：方法参数按每一处 .方法(…) 调用折，推导式变量取遍函数返回的字典里的列表，命令位置上折出几个各认一遍（python3 / bash --version 不跑代码）」
  FUNCTION_UNITS_BREAK=python-method-parameters-unfolded  类里方法的参数不按调用处折
      「python：方法参数按每一处 .方法(…) 调用折，推导式变量取遍函数返回的字典里的列表，命令位置上折出几个各认一遍（python3 / bash --version 不跑代码）」
  FUNCTION_UNITS_BREAK=python-comprehension-variables-unfolded  推导式的循环变量不取遍它迭代的元素
      「python：方法参数按每一处 .方法(…) 调用折，推导式变量取遍函数返回的字典里的列表，命令位置上折出几个各认一遍（python3 / bash --version 不跑代码）」
  FUNCTION_UNITS_BREAK=python-element-calls-unfolded  序列是调仓内函数的返回值、{…}.get(键, 缺省) 的不取元素
      「python：方法参数按每一处 .方法(…) 调用折，推导式变量取遍函数返回的字典里的列表，命令位置上折出几个各认一遍（python3 / bash --version 不跑代码）」
  FUNCTION_UNITS_BREAK=python-sequence-indexes-unfolded  只赋过一次的元组 / 列表字面量[整数] 不折
      「python：spec_from_file_location 载入的模块的常量（module.NAME）与模块级元组[整数] 折得出，sh -c 的串连到它起的脚本」
  FUNCTION_UNITS_BREAK=python-loaded-module-constants-unfolded  spec_from_file_location 载入的模块的 module.NAME 不折
      「python：spec_from_file_location 载入的模块的常量（module.NAME）与模块级元组[整数] 折得出，sh -c 的串连到它起的脚本」
  FUNCTION_UNITS_BREAK=python-script-tail-groups-ignored  解释器后面的脚本只认得出字面尾巴的照旧拒算（不按尾巴整组连，仓里没有这一截的也拒算）
      「python：os.path.join(<折不出的目录>, "tools", "runner.py") 连到仓里以 tools/runner.py 收尾的那一份」
      「python：os.path.join(<折不出的目录>, "generated", "made.py") 仓里没有以它收尾的文件：不是仓里的脚本，不拒算」
      「python：["bash", "setup.sh"] 在折不出的 cwd= 里起：仓里以 /setup.sh 收尾的整组连上」
  FUNCTION_UNITS_BREAK=python-submitted-functions-unfolded  pool.submit(f, 实参) 不算对 f 的一处调用
      「python：pool.submit(f, 实参) 算一处调用，f 的命令参数按它折，连到那份脚本」
  FUNCTION_UNITS_BREAK=python-environment-assignments-unfolded  env 后面折不出值的 NAME=VALUE 当命令位置（照旧拒算）
      「python：env -u X NAME={折不出} 之后的命令照命令位置认，连到那份脚本」
  FUNCTION_UNITS_BREAK=python-data-reads-ignored  运行期读的文件既不收进键也不拒算
      「python：运行期 open 读的仓内文件（路径折得出）进键：改它键变」「python：读 <sys.argv 给的目录>/pkg/table.tsv 照字面尾巴收仓里那一份：改它键变」
      「python：读 os.path.join(<根本身>, <读进来的登记行点名的一截>) 拒算 data-path-unfolded」
      「python：读 os.path.join(<根里的目录>, <出处推不出的一截>) 收那个目录整棵：改目录里的一份键变」
      「python：别的模块里的函数读它参数给的文件：闭包里的调用处给仓里的常量收进键（改它键变），给入口参数的不收也不拒算」
      「python：读 self.<属性>：照本类方法里给它的赋值认，赋的是仓里的常量就收进键（改它键变）」
  FUNCTION_UNITS_BREAK=python-parsed-arguments-unread  argparse 解出来的路径不当调用方的输入（照拒算）
      「python：读 argparse 解出来的路径（调用方的输入）不拒算」
  FUNCTION_UNITS_BREAK=python-directory-reads-refused  os.path.join(<根里的目录>, <出处推不出的一截>) 不收目录、照拒算
      「python：读 os.path.join(<根里的目录>, <出处推不出的一截>) 收那个目录整棵：改目录里的一份键变」
  FUNCTION_UNITS_BREAK=python-root-joins-refused  os.path.join(<根本身>, <折不出的一截>) 一律拒算（不看那一截是不是读根里的文件得来的），拷文件的源照拒算
      「python：读 os.path.join(<根本身>, <算出来的名单里的一截>)（不是读根里的文件得来的）：被判的输入，不拒算」
      「python：拷文件的源 os.path.join(<根本身>, <读进来的登记行点名的一截>)：拷副本，不拒算」
  FUNCTION_UNITS_BREAK=python-parameter-callers-unresolved  本模块里没有调用处的函数参数不在闭包里按名字找调用处：不是入口的拒算
      「python：别的模块里的函数读它参数给的文件：闭包里的调用处给仓里的常量收进键（改它键变），给入口参数的不收也不拒算」
  FUNCTION_UNITS_BREAK=python-attribute-reads-unfolded  读 self.<属性> 不照本类里给它的赋值认（落到调用方交进来的输入，不收）
      「python：读 self.<属性>：照本类方法里给它的赋值认，赋的是仓里的常量就收进键（改它键变）」
  FUNCTION_UNITS_BREAK=python-tail-reads-unanchored  读 <参数>/<字面尾巴> 不按调用处认，读的时候就照尾巴收仓里那一组
      「python：读 <参数>/<字面尾巴>，闭包里的调用处给的是这一趟的临时目录：仓里这一截尾巴的那一组不收，改其中一份键不变」
      「python：读 <判法入口的参数>/<字面尾巴>（被判的输入）：仓里这一截尾巴的那一组不收，改其中一份键不变」
  FUNCTION_UNITS_BREAK=python-launch-arguments-unfolded  命令参数不在调用点照实参折，只认调用方交进来的那几样（字面的不连、根外程序与返回值不认）
      「python：命令是别的函数的参数（subprocess.Popen(command)、[program, *arguments]）：照闭包里每个调用点的实参折（字面程序、根外程序、外层函数的变量、调仓内函数的返回值），不拒算」
      「python：命令参数在调用点折出根里的脚本（run_tool("pkg/tool.sh", …)）：连上那一份，改它键变」
  FUNCTION_UNITS_BREAK=python-caller-launches-refused  入口参数给的命令、-c 的串、解释器照旧拒算
      「python：判法入口的参数给的命令（-c 的串、解释器）是调用方交进来的：不拒算」
  FUNCTION_UNITS_BREAK=python-toolchain-queries-refused  [<折不出的程序>, "-V"] 照旧拒算 subprocess-command-unfolded
      「python：[<环境给的编译器>, "-V"] 是工具链询问：不拒算，工具链单元进键：换一版假 rustc 键变，闭包里没有询问的换 rustc 键不变」
  FUNCTION_UNITS_BREAK=python-toolchain-queries-unkeyed  工具链询问不收工具链单元（询问打的版本不进键）
      「python：[<环境给的编译器>, "-V"] 是工具链询问：不拒算，工具链单元进键：换一版假 rustc 键变，闭包里没有询问的换 rustc 键不变」
  FUNCTION_UNITS_BREAK=python-registry-programs-ignored  REGISTRY_LISTED_PROGRAMS 不认，登记表点名的脚本照旧拒算 subprocess-path-unfolded
      「python：登记表点名的程序（REGISTRY_LISTED_PROGRAMS）是环境探针：不连、不拒算，改登记表点名的一份探针，起探针的入口键不变」
  FUNCTION_UNITS_BREAK=python-registry-programs-linked  登记表点名的每一份照程序连（整份脚本体）、登记表收成 data-file
      「python：登记表点名的程序（REGISTRY_LISTED_PROGRAMS）是环境探针：不连、不拒算，改登记表点名的一份探针，起探针的入口键不变」
  FUNCTION_UNITS_BREAK=python-membership-tables-followed  给了参数时 <词> in 表 照旧把整张表当普通引用连（值里写的函数都连进来）
      「python：同一程序入口带两组参数：只改另一组参数才走到的函数，这一组键不变、改这一组走到的键变；起它的脚本把参数带过去照样剪」
  FUNCTION_UNITS_BREAK=shell-function-bounds-quote-blind  shell 函数头与收尾的 } 照旧逐行认，不看引号（多行 awk 程序里的「    }」把函数切断）
      「shell：函数里多行 awk 程序带一行与函数收尾同缩进的 }：函数照引号外的 } 收尾，awk 之后的 bash s/child.sh 连上（改它键变）」
  FUNCTION_UNITS_BREAK=shell-script-tail-groups-ignored  「$<折不出的目录>/<折得出的尾巴>」照旧拒算 shell-path-unfolded
      「shell：bash "$<折不出的目录>/$<折得出的变量>/driver.sh" 连到仓里以 .sample/driver.sh 收尾的那一组（改它键变）」
      「shell：bash "$<折不出的目录>/.nowhere/run.sh" 仓里没有以它收尾的脚本：不是仓里的脚本，不拒算」
  FUNCTION_UNITS_BREAK=shell-program-arguments-unnarrowed  bash <脚本> <字面参数…> 起的脚本体照旧整份收边
      「shell：bash s/dispatch.sh run：if [[ "${1:-}" == --selftest ]] 那一支走不到，它起的 s/child.sh 不连（改它键不变）」
      「shell：bash s/dispatch.sh run：case "$1" 里 *) 那一支走不到，它起的 s/tools/two.py 不连（改它键不变）」
  FUNCTION_UNITS_BREAK=shell-pwd-unfolded  shell 的 $PWD 不折
      「shell：$PWD 照脚本体顶层那一处 cd 折（cd "$(dirname "$0")/.."），函数里 bash "$PWD/<文件>" 连到那一份」
  FUNCTION_UNITS_BREAK=shell-pattern-removals-unfolded  ${名%词} 一类只认 %/*，别的去后缀、前缀不折
      「shell：${名%词} 与 ${名##词} 的词折得出就照 bash 去掉后缀、前缀，连到折出的那一份」
  FUNCTION_UNITS_BREAK=shell-directory-copies-ignored  cd "$目录/<根里的目录>" 不照根里那个目录认
      「shell：cd "$目录/<根里的目录>" 之后的相对路径照根里那个目录认，连到那一份」
  FUNCTION_UNITS_BREAK=shell-relative-tree-copies-ignored  cd 到折不出的目录之后的相对路径不按根里同一相对路径认
      「shell：cd 到折不出的目录之后 bash <根里的相对路径>（至少两段）连到根里那一份」
  FUNCTION_UNITS_BREAK=shell-directory-wildcards-ignored  「<根里的目录>/<带展开的文件名>」不连那个目录里的脚本
      「shell：python3 "<根里的目录>/$1" 连到那个目录里每一份 .py」
  FUNCTION_UNITS_BREAK=shell-positional-scripts-refused  函数里解释器后面的 "$@" "$1" 不照调用处找，照旧拒算
      「shell：函数里 python3 "$@" 的脚本照每一处调用给的那个词认，连到那一份」
  FUNCTION_UNITS_BREAK=shell-relative-directories-unrooted  cd 到相对目录之后的相对路径不把那个目录接到根上（拼出来是相对路径，报不在）
      「shell：cd <根里的相对目录> 之后 python3 <那个目录里的文件> 连到那一份（不报不在）」
  FUNCTION_UNITS_BREAK=row-keyed-tables-whole  按行进键的登记表照旧整份进键（row_keyed_closure 一张都不换）
      「按行进键的登记表：判法经登记的取用函数按入口查表里一行，改表里与这一条无关的行键不变、改它查的那一行键变；读整张表的不换」
  FUNCTION_UNITS_BREAK=python-program-arguments-unnarrowed  给了参数也照整份收
      「python：同一程序入口带两组参数：只改另一组参数才走到的函数，这一组键不变、改这一组走到的键变；起它的脚本把参数带过去照样剪」
  FUNCTION_UNITS_BREAK=python-import-time-tables-followed  给了参数时 import 时执行的赋值照旧沿字面量里当元素写的函数名连
      「python：同一程序入口带两组参数：只改另一组参数才走到的函数，这一组键不变、改这一组走到的键变；起它的脚本把参数带过去照样剪」
  FUNCTION_UNITS_BREAK=python-slice-unpacking-unpaired  a, b = s[k:k+2] 不逐个对上 s[k]、s[k+1]（内联脚本拆包 sys.argv[1:13] 折不出，sys.path.insert 拒算 sys-path-unfolded）
      「shell：exec python3 -B /dev/fd/3 … 3<<'PY' 里 (目录, 标签) = sys.argv[1:3] 拆包逐个对上，sys.path.insert(0, 目录) 折得出、连到它 import 的仓内模块」
  与 crates/singlefs-unit-graph 对齐的那几条（自证里 ALIGNMENT_FIXTURE 与 CLOSURE_RULES_FIXTURE 那几格）：
  FUNCTION_UNITS_BREAK=rust-module-heads-unlinked  单元不连模块头，也不连把它所在文件连进模块树的 mod 声明
      「R-结构：改文件开头的 #![…]，经 mod 声明连进来的子模块里的入口作废」「R-结构：改行内模块头的内属性，只那个模块里的入口作废」
  FUNCTION_UNITS_BREAK=rust-module-declarations-unlinked  单元不连把它所在文件连进模块树的 mod 声明（模块头照连）
      「R-结构：改文件开头的 #![…]，经 mod 声明连进来的子模块里的入口作废」
  FUNCTION_UNITS_BREAK=rust-use-names-resolved  use 单元里的名字往下解（引进的名字全带进闭包）
      「R-结构：use 引进的名字不往下解，没用到的那个改了不作废」
  FUNCTION_UNITS_BREAK=rust-use-aliases-unresolved  use a::b as c 里用到 c 的不按 b 再解
      「R-结构：use a::b as c 之后调 c()，按 b 再解」
  FUNCTION_UNITS_BREAK=rust-trait-impls-by-trait-arguments  R-隐式另按 trait 参数里的类型认 trait impl
      「R-隐式：只点名 trait 参数里的类型，那个 trait impl 不进闭包」
  FUNCTION_UNITS_BREAK=rust-kind-names-hyphenated  输出行的种类名退回旧拼法（macro、item-macro、mod-decl、extern-block、extern-crate、wgsl-entry、wgsl-fn…）
      「dump：种类名照 crates/singlefs-unit-graph 的拼法（macro_rules、item_macro、mod、mod_head、wgsl_compute）」
  FUNCTION_UNITS_BREAK=rust-qualified-names-file-local  输出行的限定名退回文件里的嵌套路径（不带 crate、目标与模块路径前缀）
      「dump：限定名带 crate、目标与模块路径前缀，文档测试写 <模块>::<doctest line 行>」
  FUNCTION_UNITS_BREAK=rust-doctest-fingerprint-from-rows  文档测试的指纹取代码块原文（随代码块里的注释变）
      「文档测试：代码块里只改注释指纹不变、改代码指纹变」
  FUNCTION_UNITS_BREAK=rust-module-heads-dropped  不切模块头（行内模块头与文件开头的 #![…]），模块里的单元也就不连它
      「模块头：#![…] 与行内模块头切成单元，模块里的单元连它，改内属性作废」
  FUNCTION_UNITS_BREAK=rust-data-dump-unlisted  dump 不列数据行
      「dump：include_str! 读的数据文件按读它的那一处列一行 data」
  FUNCTION_UNITS_BREAK=wgsl-external-names-by-long-identifiers  WGSL 外名退回旧口径：串的值与闭包里长于 3 个字符的标识符里出现就算有着落
      「WGSL 外名：首段连 _ 落在 macro_rules! 体或串字面量里有着落」「WGSL 外名：只在别的单元的长标识符里出现的拒算」
  FUNCTION_UNITS_BREAK=wgsl-external-name-sources-unlinked  WGSL 入口的闭包不带给外名着落的 Rust 单元（着落不做成边）
      「WGSL 外名的着落做成边：给名字的 Rust 单元改了 WGSL 入口作废」
  FUNCTION_UNITS_BREAK=rust-escaped-brace-captures-skipped  格式串里转义的 {{ 后面紧跟的捕获（{{{名}}}）不算裸名
      「format! 串里 {{{名}}}（转义的 {{ 后紧跟的捕获）改了作废」
  FUNCTION_UNITS_BREAK=rust-union-as-keyword  条目级宏调用可能定义的名字里另去掉 union（把上下文关键字当关键字）
      「条目级宏调用里的 union 算它可能定义的名字」
  FUNCTION_UNITS_BREAK=rust-assume-specifications-unrecognized  verus! 里的 assume_specification 不算条目起头、不切成单元（带 pub 的拒算 unknown-item，不带 pub 的吞进前一条）
      「verus!：assume_specification 带属性带 pub、不带 pub 带参数表与 ensures、带泛型参数表三种写法各切成一个单元（种类、名字、起止、按点名路径末段的名字找得到），dump 一条拒算都没有，后面的 spec fn 照旧自成单元」「verus!：点到常量的入口按名字连上它的 assume_specification，改它交回的值形式化键变」
  FUNCTION_UNITS_BREAK=rust-assume-specification-names-unlinked  assume_specification 不按点名路径末段的名字找（点到那个名字的闭包不收它，改它交回的值键不变）
      「verus!：assume_specification 带属性带 pub、不带 pub 带参数表与 ensures、带泛型参数表三种写法各切成一个单元（种类、名字、起止、按点名路径末段的名字找得到），dump 一条拒算都没有，后面的 spec fn 照旧自成单元」「verus!：点到常量的入口按名字连上它的 assume_specification，改它交回的值形式化键变」
  FUNCTION_UNITS_BREAK=rust-dependency-features-from-every-section  解特性时每个链进来的 crate 的普通、dev、build 三段依赖都顺（dev 依赖里才开的特性进了 bin 与别的 crate）
      「R-目标：入口 crate 的 dev 依赖里才开的特性，不按 cfg(test) 编的 bin 当它没开，集成测试当它开着」「R-目标：经普通依赖链进来的 crate，它的 dev 依赖里开的特性哪个目标都不顺」
  FUNCTION_UNITS_BREAK=rust-verus-semicolon-split-off  verus! 里 } 之后紧跟的 ; 当下一个条目起头（不进前一个条目，const X: S = S { … }; 的指纹少它）
      「verus!：} 之后紧跟的 ; 归前一个条目（结构体字面量当初值的 const 与 impl 里的关联 const），指纹是连 ; 在内的词法单元以 0x1f 连的 sha256，后面的条目照旧切开」
    （这四条在自证里 unit_graph_parity_cells 那几格）
  4-3d 那几条（自证里 unit_divergence_cells 那几格）：
  FUNCTION_UNITS_BREAK=rust-cfg-atoms-unnarrowed  键里的 cfg 退回整套生效 cfg（不按闭包真引到的收窄）
      「cfg 成分：闭包没引到 gpu 的入口，带不带 gpu 特性两种目标下键相同」
  FUNCTION_UNITS_BREAK=rust-cfg-macro-atoms-ignored  cfg 原子只取单元的条件链，不扫词法单元里的 cfg!、语句上的 #[cfg] 与 #[cfg_attr]
      「cfg 成分：闭包里有 cfg!、语句上的 #[cfg] 或 #[cfg_attr] 引到 gpu 的入口，带不带 gpu 特性键不同」
  FUNCTION_UNITS_BREAK=wgsl-defined-name-suffixes-ignored  WGSL 外名不认「常量、静态量、条目级宏调用定义的名字或它以 _ 加外名收尾」这一条
      「WGSL 外名：条目级宏调用定义的名字以 _ 加外名收尾就有着落」「WGSL 外名的着落做成边：只靠外名点到的条目级宏调用并进闭包，改它里面的码键变」
  FUNCTION_UNITS_BREAK=wgsl-prefix-remainders-unchecked  WGSL 外名的首段规则不核其余那一段
      「WGSL 外名：首段只落在无关的串里、其余那一段哪儿都没有的拒算」
  （「WGSL 外名的着落做成边：只靠外名点到的…」「WGSL 外名的着落不收测试函数：…」两格，自证判时另关 rust-macro-rules-literal-names-unlinked，只看着落那条边）
  FUNCTION_UNITS_BREAK=wgsl-name-suppliers-not-pulled-in  Rust 入口的闭包不把闭包外供外名的单元并进来（着落不做成边）
      「WGSL 外名的着落做成边：只靠外名点到的条目级宏调用并进闭包，改它里面的码键变」
  FUNCTION_UNITS_BREAK=wgsl-name-suppliers-include-test-code  名字供给者连带 #[test] fn 与 #[cfg(test)] 模块里的单元一起收
      「WGSL 外名的着落不收测试函数：串里有外名的 #[test] fn 不并进闭包，改它的函数体键不变，改真给名字的宏调用键变」
  FUNCTION_UNITS_BREAK=wgsl-modules-per-naming-unit  WGSL 模块退回按点名读 .wgsl 的常量的那个单元一组一组拼
      「WGSL：闭包读到的几份 .wgsl 合成一个模块，名字跨文件解」
  FUNCTION_UNITS_BREAK=rust-trait-self-paths-ignored  trait 成员里的 Self::x 不落到 trait 自己的成员（退回旧口径：落到自身类型名是 trait 名的 impl）
      「R-路径：trait 成员里的 Self::x 落到这个 trait 自己的成员 x」
  FUNCTION_UNITS_BREAK=rust-doctest-structure-linked  文档测试照普通单元连所挂的条目与那个模块的 use
      「R-结构：文档测试不连所挂的条目与那个模块的 use」
  FUNCTION_UNITS_BREAK=rust-use-self-imports-ignored  use 花括号里的 self 不引进 { 前面那一段
      「R-结构：use a::{self} 引进名字 a，只写 a::f() 的单元连上这条 use」
  FUNCTION_UNITS_BREAK=rust-open-by-trait-arguments  R-开放另按 trait 参数里的名字找 impl（dyn Sink 带进 impl Wrap<dyn Sink> for H）
      「R-开放：dyn Trait 只带 Trait 自己的 impl，不带只在 trait 参数里写了它的 impl」
  FUNCTION_UNITS_BREAK=rust-blanket-impls-ignored  R-方法、R-路径不放行 blanket impl，trait 头进闭包不带它的 blanket impl
      「R-方法：blanket impl 够得到的方法算边，改它作废」
  4-3h 那两条（自证里 unit_divergence_cells 里 4-3k 那两格之前的两格）：
  FUNCTION_UNITS_BREAK=rust-underscore-items-named  const _ / static _ 照名字 _ 进 R-裸名的着落（let _ = …、_ => 里的 _ 把它们拉进闭包）
      「R-裸名：const _ 没有名字，let _ 与 _ => 不把 gpu 特性下的 const _ 拉进闭包，带不带 gpu 特性键相同」
  FUNCTION_UNITS_BREAK=rust-includes-outside-root-refused  include_str! 一类读根外的文件一律拒算 include-missing（读得到也拒）
      「include_str! 读根外的文件：读得到收成数据单元、改它键变，读不到拒算 include-missing」
  实验准入入口那一条（自证里「Rust：清单目录往上拼出仓根」那一格；admission.py --selftest 拿真仓 scripts/experiment-apparatus/lib.rs 的那一格同样转红）：
  FUNCTION_UNITS_BREAK=rust-manifest-directory-reads-unrefused  Path::new(env!("CARGO_MANIFEST_DIR")).join(…) 拼出的是目录（仓根这一类）也不拒算 runtime-path（「读路径都不拒算」那种放宽）
      「Rust：清单目录往上拼出仓根（目录）拒算 runtime-path；点名清单目录里一份文件再往上取仓根的不拒算、那份文件的字节进键」
  FUNCTION_UNITS_BREAK=rust-manifest-directories-refused  清单目录折出 crates/ 之下的目录也一律拒算 runtime-path（不收成数据目录）
      「Rust：清单目录折出 crates/ 之下的目录收成数据目录（直接写、绑 let 变量、经私有常量），目录里一份文件改了键变、闭包外的改了键不变；crates/ 整个与变量上接别的方法的拒算 runtime-path」
  FUNCTION_UNITS_BREAK=rust-manifest-binding-refused-whole  清单目录绑给 let 的，不是能收的目录不跟、有一处用法折不出就整处拒算、一条读都不记
      「Rust：清单目录绑给 let 变量的，一处 .join(变量) 折不出、一处折出文件：这个单元拒算 runtime-path、那份文件照记一条 data 读（绑的是清单目录本身与 crates/ 整个都一样）；别名再绑一层、Path::new(别名) 接的链照跟」
  FUNCTION_UNITS_BREAK=rust-current-exe-refused  Command::new(current_exe()) 的写法一律不认（起了子进程的函数里折不出测试名，拒算 current-exe）
      「Rust：current_exe() 交给 Command::new 起子进程、--exact 后面点名的测试（常量、参数、字面与常量拼的）连到那条 #[test]，改它的函数体键变、改别的测试键不变；点名不存在的、测试名写在 --exact 前面的照旧拒算 current-exe」
  FUNCTION_UNITS_BREAK=rust-current-exe-children-unlinked  折得出子进程测试名也不连边（子进程测试的函数体改了键不变）
      「Rust：current_exe() 交给 Command::new 起子进程、--exact 后面点名的测试（常量、参数、字面与常量拼的）连到那条 #[test]，改它的函数体键变、改别的测试键不变；点名不存在的、测试名写在 --exact 前面的照旧拒算 current-exe」
  FUNCTION_UNITS_BREAK=rust-current-exe-path-only-refused  只问路径、同一函数里不起子进程的 current_exe() 也拒算 current-exe
      「Rust：current_exe() 只问路径、同一函数里不起 Command 的不拒算：键算得出，改它的函数体键变、改别的测试键不变」
  FUNCTION_UNITS_BREAK=rust-current-exe-spawn-unrefused  起了子进程的函数里，折不出测试名的 current_exe() 也不拒算 current-exe
      「Rust：同一函数里起 Command 的 current_exe() 照旧按起自己判：折得出测试名的连边，折不出的（没有 --exact、路径先存进变量再交 Command::new、点名不存在的）拒算 current-exe」
  FUNCTION_UNITS_BREAK=rust-current-exe-constants-same-file-only  常量名只在起子进程那个单元所在的那一份文件里找（常量定义在同包别的文件里的拒算 current-exe）
      「Rust：current_exe() 的测试名里的常量定义在同包另一份文件里，--exact 后面用它：连到那条 #[test]、不拒算，改它的函数体键变、改别的测试键不变；同名常量在同包两份文件里值不同的照旧拒算 current-exe」
  FUNCTION_UNITS_BREAK=closure-directory-lines-span-files  命令行 closure 与 key 把数据目录单元打成它自己那一行（<目录>:1-<文件数>）
      「命令行 closure：数据目录单元按读它的那一处列（data-directory 加目录、读它的文件与行、目录指纹），不打成 <目录>:1-<文件数>」
  FUNCTION_UNITS_BREAK=rust-lex-cache-unverified  Rust 词法单元的磁盘缓存文件名只取文本长度（缓存不核内容）
      「Rust 词法单元的磁盘缓存：同一份文本第二次取缓存、与现切的逐项相同；改一个字符（长度不变）不沿用、切出来的随文本变；缓存文件坏了照常现切」
  4-3k 那两条（自证里 unit_divergence_cells 末尾两格）：
  FUNCTION_UNITS_BREAK=rust-blanket-self-parameter-only  blanket 退回旧判法：自身类型里头一个名字是泛型参数才算（Box<T>、Option<Box<T>>、Sealed<T> 的 impl 不算）
      「R-方法：自身类型里出现泛型参数的 impl（(T, u32)、[T]、[T; N]、Box<T>、Option<Box<T>>、Sealed<T>）都按 blanket 放行，改 Sealed<T> 那一份作废」
  FUNCTION_UNITS_BREAK=wgsl-other-file-definitions-unlinked  WGSL 外名在链接集里别的 include_str! 读进来的 .wgsl 里有定义时，不把读它的 Rust 单元与那份 .wgsl 拉进闭包
      「WGSL 外名在链接集里别的 include_str! 读进来的 .wgsl 里有定义：读它的 Rust 单元与那份 .wgsl 拉进闭包，改那个常量作废」
  import 时执行的模块级语句与不走的子进程边（自证里 import_time_cells 那几格）：
  FUNCTION_UNITS_BREAK=python-import-time-statements-ignored  模块有单元进闭包时不收它的模块级 import、赋值、类头与 def 头（C890 的样子）
      「python：import 时执行的模块级 import、赋值、类头与 def 缺省值改了让程序退出码变的，入口的键都变（键不变而退出码变的必红，对照那一种必须认出）」
  FUNCTION_UNITS_BREAK=python-def-heads-ignored  不切 def-head（装饰器与缺省值不进闭包）
      「python：import 时执行的模块级 import、赋值、类头与 def 缺省值改了让程序退出码变的，入口的键都变（键不变而退出码变的必红，对照那一种必须认出）」
  FUNCTION_UNITS_BREAK=python-import-time-classes-named  只因 import 时执行收进来的类头也算被点名、连方法
      「python：只因 import 时执行收进来的类头不算被点名，类里的 __init__ 改了不作废」
  FUNCTION_UNITS_BREAK=python-main-guard-conjunctions-unrecognized  if __name__ == "__main__" and …: 不当 main-guard，留在 module-body（被 import 的模块的自证分支连进闭包）
      「python：被 import 的模块里 if __name__ == "__main__" and …: 那一段不进闭包（自证分支不收），块外的模块级语句与认不出的形态（带 else、__name__ 那一项不在最左）照收」
      「真仓：以 scripts/hosts/memory_peaks_lib.py 的 command_key 为判法入口，闭包不收它的自证（run_self_test、self_test_cells），改 self_test_cells 的函数体判法单元指纹不变」
  FUNCTION_UNITS_BREAK=python-main-guards-last-only  一份文件里几段 main-guard 只留末一段（前面几段被后面的盖掉、哪儿都不收）
      「python：被起的程序里每一段 main-guard 都收（if __name__ == "__main__" and …: 那一段与后面另一段），起它的参数不是字面量也连自证分支」
  FUNCTION_UNITS_BREAK=python-main-guard-else-dropped  if __name__ == "__main__": … else: … 连 else 整段当 main-guard（被 import 时 else 那一支哪儿都不收）
      「python：if __name__ == "__main__": … else: … 被 import 时 else 那一支照收、if 那一支不收；被当程序起时 if 那一支照收」
  FUNCTION_UNITS_BREAK=program-edge-exclusions-ignored  EXCLUDED_PROGRAM_EDGES 登记的子进程边照连
      「不走的子进程边（夹具）：登记了的 python 单元起的那一条不连、同一个单元起的另一个程序照连、不登记时那一条照连」
      「不走的子进程边（夹具）：登记了的 shell 函数起的那一条不连、同一个函数起的另一个程序照连、不登记时那一条照连」
      「不走的子进程边（真仓 E183 准入）：切边前后少掉的单元都在 admission.py 当程序起的闭包里（只经这条边可达），且确实少了 admission.py」
  FUNCTION_UNITS_BREAK=program-edge-exclusions-shell-ignored  EXCLUDED_PROGRAM_EDGES 只认 python 单元起的，shell 函数起的照连
      「不走的子进程边（夹具）：登记了的 shell 函数起的那一条不连、同一个函数起的另一个程序照连、不登记时那一条照连」
      「不走的子进程边（真仓 E183 准入）：切边前后少掉的单元都在 admission.py 当程序起的闭包里（只经这条边可达），且确实少了 admission.py」
  FUNCTION_UNITS_BREAK=excluded-edge-exit-unmeasured  改坏被切的程序之后不重跑包装，退出码照改坏之前记
      「不走的子进程边（夹具）：切了边的程序改坏后包装退出码变了而键不变的必红（对照 leak.py 必须认出），吞掉退出码的 quiet.py 不红」
  FUNCTION_UNITS_BREAK=admission-edge-admitted-sample  准入异常那几个假准入里多跑一个放行的（退 0、打带 sha256 的 E7INPUT 行），计划往下走、起编译
      「不走的子进程边（真仓 E183 准入）：准入这条路上的异常（崩、被杀、退 77、退 3、退 0 不打行、打别的行）只让计划那一步退非 0、不起编译、不写计划，读数出不来」
  FUNCTION_UNITS_BREAK=admission-edge-header-unstripped  判法读产物前不去掉 E7INPUT 行
      「不走的子进程边（真仓 E183 准入）：准入往标准输出打的只有 E7INPUT 行（键算出、拒算两样），判法读产物前去掉这几行之后读数一行不差」
  FUNCTION_UNITS_BREAK=admission-edge-reach-unmeasured  不量 admission.py 当程序起的闭包（当它是空的）
      「不走的子进程边（真仓 E183 准入）：切边前后少掉的单元都在 admission.py 当程序起的闭包里（只经这条边可达），且确实少了 admission.py」
  FUNCTION_UNITS_BREAK=judging-entry-literals-unfolded  判法入口的登记只认字面量（JUDGING_ENTRIES = A + (...) 折不出）
      「真仓：判法入口从登记现取（registered_judging_entries），形式化入口与包装入口都在里面、都认得出」「真仓：登记的每个判法入口的闭包都不拒算（拒算的那一路判定一条都写不进库），夹具里的 globals() 查找判得出来」
  FUNCTION_UNITS_BREAK=judging-entry-refusals-ignored  判法入口闭包里的拒算一条都不报
      「真仓：登记的每个判法入口的闭包都不拒算（拒算的那一路判定一条都写不进库），夹具里的 globals() 查找判得出来」
  FUNCTION_UNITS_BREAK=judging-entry-workers-head-word  判法入口拒算的并行进程数取登记文件的头一个词，读不出就不用登记的数（退回 1 个进程；那份文件头一个词是 #，总读不出）
      「判法入口拒算的并行进程数：没设 SINGLEFS_THREAD_CAP 取登记的每台默认重活线程数（头一行是注释的登记文件照样取到），设了照它，不退回按核数算」
  产品行为视图那五条（自证里 product_view_cells 那几格）：
  FUNCTION_UNITS_BREAK=product-view-kept-formal  不剥：产品行为视图照原样那一份（不取 Rust 那一份，规格与证明照进键）
      「产品行为视图：引理里加一句 assert(true);，产品行为键不变、形式化键变」「产品行为视图：verus! 里执行函数的 ensures 改了，产品行为键不变」
  FUNCTION_UNITS_BREAK=product-view-strips-executable-code  剥到执行代码：verus! 块里的单元一律当只给 Verus 看的（被调到的执行代码也不进键）
      「产品行为视图：verus! 里被调到的执行代码改了，产品行为键变」
  FUNCTION_UNITS_BREAK=product-view-edges-from-the-written-tokens  不用剥过的那一份：有 verus! 块的文件一律照原样那一份（规格与证明里点到的执行侧条目照进闭包）
      「产品行为视图：只在规格里点到的执行侧条目（ghost 字段的类型、ensures、let ghost、proof 块、循环头里点到的结构体）不进闭包，闭包与 unit-graph 剥完再收的那一份逐个相同」
  FUNCTION_UNITS_BREAK=product-view-blanking-unverified  剥过的那一份不与 Rust 那一份核就用
      「产品行为视图：这一份剥出来的与 Rust 那一份交的核不上，照原样那一份收（闭包只多不少，规格里点到的结构体照收）」
  FUNCTION_UNITS_BREAK=product-view-verifier-attributes-kept  不剥外属性 #[verifier::…] / #[verifier(…)]（Rust 那一份剥了，单元起始行对不上，拒算 product-view-unmatched）
      「产品行为视图：verus! 里的执行函数只多一行 #[verifier::external_body]，产品行为视图配得上、产品行为键不变」
      「产品行为视图：没有 verus! 块的文件里的函数只多一行 #[verifier::external_body]，产品行为视图配得上、产品行为键不变」

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
import array
import ast
import base64
import bisect
import collections
import copy
import fcntl
import fnmatch
import glob
import hashlib
import importlib.util
import json
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
import zlib
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

KEY_FORM = "singlefs-function-units-v1"
# 判定种类（文件头「判定种类」）：与 crates/singlefs-unit-graph 的 --judgment-kind 同两个写法
FORMAL_VERIFICATION = "formal-verification"
PRODUCT_BEHAVIOR = "product-behavior"
JUDGMENT_KINDS = (FORMAL_VERIFICATION, PRODUCT_BEHAVIOR)
UNIT_GRAPH_DIRECTORY_VARIABLE = "FUNCTION_UNITS_UNIT_GRAPH_DIRECTORY"
UNIT_GRAPH_KEPT_PROGRAMS = 3
VERUS_GHOST_CODE_UNPARSED = "verus-ghost-code-unparsed"
BREAK_VARIABLE = "FUNCTION_UNITS_BREAK"
# 函数里出现这几个词算起了子进程：此时 current_exe() 折不出要点名的那一条测试，拒算 current-exe（文件头「测试二进制起自己」）
SPAWNING_WORDS = frozenset(("Command", "spawn", "posix_spawn", "fork", "execv", "execve", "execvp"))
REFUSED_EXIT = 4
SHELL_TIMEOUT_SECONDS = 600
SNIPPET_CHARACTERS = 160
FOLD_LIMIT = 64
# 清单目录拼出的目录绑到 let 变量、变量的用法再绑到变量，最多跟这么多层，再深的拒算（crates/singlefs-unit-graph 的 DEEPEST_DIRECTORY_BINDING_CHAIN 同值）
DEEPEST_MANIFEST_BINDING_CHAIN = 4
# fold_manifest_chain 交出的「这一处折不出」，记成一条 runtime-path 拒算
MANIFEST_FOLD_REFUSED = (None, None)
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
MEMO_IN_PROGRESS = object()   # PythonModule.memoized：这一项正在算
COMPREHENSION_UNPACKED = object()   # 推导式拆包绑的循环变量（for a, b in …）：PythonModule.comprehension_binding 交它，折不出
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
              "open_keys", "ghost_only", "self_path_root")

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


def directory_fingerprint(root, path):
    """数据目录的指纹：目录下（递归，什么目录都不跳过）全部文件按相对目录的路径排序，每个文件一行「<相对路径>\\t<字节 sha256>\\n」连起来的 sha256 → (指纹, 文件数)。
    指向目录的符号链接不进去，指向文件的读它指到的内容，断了的不算（crates/singlefs-unit-graph 的 directory_digest 同）。"""
    top = os.path.join(root, path)
    entries = []
    for directory, _names, files in os.walk(top):
        for name in files:
            absolute = os.path.join(directory, name)
            if os.path.islink(absolute) and not os.path.isfile(absolute):
                continue
            with open(absolute, "rb") as handle:
                entries.append((os.path.relpath(absolute, top).replace(os.sep, "/"), sha256_bytes(handle.read())))
    entries.sort()
    return sha256_text("".join(f"{relative}\t{digest}\n" for relative, digest in entries)), len(entries)


def data_directory_unit(root, path):
    """数据目录单元：种类 data-directory，限定名与文件都是目录相对根的路径，起止 1-<文件数>（至少 1），指纹 directory_fingerprint。"""
    fingerprint, count = directory_fingerprint(root, path)
    return Unit("data", "data-directory", path, path, 1, max(count, 1), fingerprint, data=True)


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


def rust_lex(text, spans=None, document_spans=None):
    """→ (kinds, texts, lines, documents)；documents 是 [(行, outer|inner, [各行正文], 下一个词法单元的下标, [各行原文])]：
    各行正文去掉了块注释每行开头的 * 号，各行原文是注释记号之后的原样（与 crates/singlefs-unit-graph 的 documentation_comment_of 同一份，文档测试取它）。
    spans / document_spans 给了列表的，逐个词法单元 / 文档注释往里追加它在 text 里的 (起, 止) 字符下标（剥只给 Verus 看的代码用：verus_ghost_code_blanked）。"""
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
        if spans is not None:
            spans.append((start, end))

    def document(entry, start, end):
        documents.append(entry)
        if document_spans is not None:
            document_spans.append((start, end))

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
                document((line_at(i), "outer", [body[3:]], len(texts), [body[3:]]), i, end)
            elif body.startswith("//!"):
                document((line_at(i), "inner", [body[3:]], len(texts), [body[3:]]), i, end)
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
                document((line_at(i), style, block_document_rows(body[3:-2]), len(texts), raw.split("\n")), i, j)
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
                emit(IDENT, i, j)
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


# Rust 词法单元的磁盘缓存（文件头「Rust 词法单元的缓存」）
RUST_LEX_CACHE_FORM = "singlefs-rust-lex-cache-v1"
RUST_LEX_CACHE_DIRECTORY_NAME = "singlefs-rust-lex-cache"
RUST_LEX_CACHE_DIRECTORY_VARIABLE = "FUNCTION_UNITS_LEX_CACHE_DIRECTORY"   # 设了放这里；写 off 不用缓存（自证里除缓存那一格外都设它）
RUST_LEX_CACHE_KEPT_NAMESPACES = 3
RUST_LEX_CACHE_STATS = {"lexed": 0, "reused": 0}   # 本进程里 rust_lex_cached 现切了几份、取了几份缓存（自证数它）
_RUST_LEX_CACHE_NAMESPACES = {}


def git_common_directory_of(root):
    """root 这棵工作树的 git common-dir，照 git 自己在盘上的约定读、不起 git：root/.git 是目录就是它；是文件（git worktree、子模块）读它的
    「gitdir: <路径>」一行（相对 root），那个目录里有 commondir 文件就照它的内容（相对那个目录）找到主仓的 git 目录，没有就是那个目录自己。
    不是 git 工作树、读不出、指到的目录不在的交 None。scripts/mutation/crates-mutation-rows.py 的 judging_cache_directory 也调它。
    弄坏开关 cross-run-cache-git-directory-only：只认 root/.git 是目录（worktree 里 .git 是文件，判成不在 git 仓里）。"""
    dot_git = os.path.join(root, ".git")
    if os.path.isdir(dot_git):
        return dot_git
    if "cross-run-cache-git-directory-only" in active_breaks() or not os.path.isfile(dot_git):
        return None
    try:
        with open(dot_git, encoding="utf-8") as handle:
            pointer = handle.readline().strip()
        if not pointer.startswith("gitdir:"):
            return None
        git_directory = os.path.normpath(os.path.join(root, pointer[len("gitdir:"):].strip()))
        common_file = os.path.join(git_directory, "commondir")
        common = git_directory
        if os.path.isfile(common_file):
            with open(common_file, encoding="utf-8") as handle:
                common = os.path.normpath(os.path.join(git_directory, handle.readline().strip()))
    except (OSError, UnicodeDecodeError):
        return None
    return common if os.path.isdir(common) else None


def cross_run_cache_directory(name):
    """跨轮缓存 name 放哪：设了 GATE_CROSS_RUN_TMPDIR 放它底下；没设放这份库所在仓的 git common-dir 底下（git_common_directory_of；与各道判定库一处，
    git worktree 里是主仓那一份、与主仓共用；不随这一趟的 TMPDIR 走，跑完不留在那一趟的临时目录里）；这份库不在 git 仓里才放 TMPDIR 底下。"""
    configured = os.environ.get("GATE_CROSS_RUN_TMPDIR", "").strip()
    if configured:
        return os.path.join(configured, name)
    if "cross-run-cache-under-tmpdir" in active_breaks():
        return os.path.join(tempfile.gettempdir(), name)   # 弄坏开关：没设 GATE_CROSS_RUN_TMPDIR 就落在这一趟的 TMPDIR 里
    common = git_common_directory_of(os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__)))))
    return os.path.join(common if common is not None else tempfile.gettempdir(), name)


UNIT_GRAPH_PRODUCT_VIEW_CACHE_DIRECTORY_NAME = "singlefs-unit-graph-product-view"


def unit_graph_product_view_cache_directory():
    """产品行为视图编出的 unit-graph 程序放哪（文件头「判定种类」里「unit-graph 从哪来」）：照 cross_run_cache_directory 放 singlefs-unit-graph-product-view。"""
    return cross_run_cache_directory(UNIT_GRAPH_PRODUCT_VIEW_CACHE_DIRECTORY_NAME)


# ── 跨轮缓存的登记（文件头「跨轮缓存的登记」）──

def crates_mutation_judging_cache_directory():
    """crates 变异的判法单元指纹缓存放哪：载入这份库所在仓里的 scripts/mutation/crates-mutation-rows.py，调它自己的 judging_cache_directory（不在这里另抄一份）。"""
    specification = importlib.util.spec_from_file_location("crates_mutation_rows_for_cross_run_caches",
                                                           os.path.join(LIBRARY_ROOT, "scripts", "mutation", "crates-mutation-rows.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module.judging_cache_directory(dict(os.environ))


SHELL_PARAMETER_NAME = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")


def expand_shell_parameters(text, environment):
    """双引号串里的参数展开，照 environment 取值：只认 $名、${名}、${名:-词}、${名-词}（词里照样展开）与反斜杠转义；
    $(…)、反引号、${名:=词} 这类别的写法抛 ValueError（登记的那一行被写成认不出的写法时，照新写法改 cross_run_cache_registry）。"""
    pieces, index = [], 0
    while index < len(text):
        character = text[index]
        if character == "\\" and index + 1 < len(text):
            pieces.append(text[index + 1])
            index += 2
            continue
        if character == "`" or text.startswith("$(", index):
            raise ValueError(f"认不出的展开（命令替换）：{text}")
        if character != "$":
            pieces.append(character)
            index += 1
            continue
        if text.startswith("${", index):
            depth, end = 1, index + 2
            while end < len(text) and depth:
                if text.startswith("${", end):
                    depth += 1
                    end += 2
                    continue
                if text[end] == "}":
                    depth -= 1
                end += 1
            if depth:
                raise ValueError(f"${{ 没有配对的 }}：{text}")
            inner = text[index + 2:end - 1]
            matched = SHELL_PARAMETER_NAME.match(inner)
            if matched is None:
                raise ValueError(f"认不出的展开（${{{inner}}}）：{text}")
            name, operator_and_word = matched.group(0), inner[matched.end():]
            value = environment.get(name)
            if not operator_and_word:
                pieces.append(value or "")
            elif operator_and_word.startswith(":-"):
                pieces.append(value if value else expand_shell_parameters(operator_and_word[2:], environment))
            elif operator_and_word.startswith("-"):
                pieces.append(value if value is not None else expand_shell_parameters(operator_and_word[1:], environment))
            else:
                raise ValueError(f"认不出的展开（${{{inner}}}）：{text}")
            index = end
            continue
        matched = SHELL_PARAMETER_NAME.match(text, index + 1)
        if matched is None:
            pieces.append(character)
            index += 1
            continue
        pieces.append(environment.get(matched.group(0)) or "")
        index = matched.end()
    return "".join(pieces)


def shell_assignment_value(relative, prefix, environment=None):
    """这份库所在仓里 relative 那份 shell 脚本：恰好一行（去掉首尾空白）以 prefix 起头、后面是一个双引号串，照 environment（不给取此刻的环境）展开那个串交回。
    命中的不是恰好一行、后面不是双引号串、串里有认不出的展开，抛 ValueError。"""
    with open(os.path.join(LIBRARY_ROOT, relative), encoding="utf-8") as handle:
        matches = [line.strip()[len(prefix):] for line in handle if line.strip().startswith(prefix)]
    if len(matches) != 1:
        raise ValueError(f"{relative} 里以「{prefix}」起头的行有 {len(matches)} 行（要恰好一行）：那份脚本改了写法，照新写法改 cross_run_cache_registry 里它那一行")
    quoted = matches[0]
    if len(quoted) < 2 or not quoted.startswith('"') or not quoted.endswith('"'):
        raise ValueError(f"{relative} 里「{prefix}」后面不是一个双引号串：{quoted}")
    return expand_shell_parameters(quoted[1:-1], os.environ if environment is None else environment)


def shell_assignment_row(name, relative, prefix):
    """登记表里 shell 那一种的一行：现取 = shell_assignment_value(relative, prefix)。"""
    return (name, relative, lambda: shell_assignment_value(relative, prefix))


TEST_ENVIRONMENT_STAGE_SCRIPT = ".claude/gate.d/test-environment.sh"
TEST_ENVIRONMENT_STAGE_SHARED_ENTRY = re.compile(r'python3 "\$STAGE_DIR/\.\./hooks/lib_work_root\.py" shared ([a-z][a-z0-9-]*)\)"')


def work_root_library():
    """这份库所在仓里的 .claude/hooks/lib_work_root.py（数据根的算法只在它那一处）：按路径载入、不进 sys.modules。"""
    specification = importlib.util.spec_from_file_location("lib_work_root_for_cross_run_caches",
                                                           os.path.join(LIBRARY_ROOT, ".claude", "hooks", "lib_work_root.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def test_environment_stage_run_times_file():
    """test-environment 记的上一趟各阶段用时放哪：照 TEST_ENVIRONMENT_STAGE_SCRIPT 此刻的算法逐段取，位置的每一段都读自那份脚本：
    「local run_times_file cross_run_cache=」那一行读 GATE_CROSS_RUN_TMPDIR（设了取它）；没设时那份脚本里恰好一处 `python3 "$STAGE_DIR/../hooks/lib_work_root.py" shared <名>)"`
    给出名字，目录交 work_root_library().shared_data_path(<名>)（脚本是调同一个库的同名命令行，.claude/gate.d 的上一层是 .claude，所以是这份库所在仓的 .claude/hooks）；
    「run_times_file=」那一行照这个目录展开成文件。任何一段在脚本里找不到恰好一处就抛 ValueError（脚本改了写法，照新写法改这个函数）。
    弄坏开关 cross-run-cache-run-times-old-spelling：只认「local run_times_file=」起头的写法（脚本里一行都没有，抛 ValueError）。"""
    if "cross-run-cache-run-times-old-spelling" in active_breaks():
        return shell_assignment_value(TEST_ENVIRONMENT_STAGE_SCRIPT, "local run_times_file=")
    cross_run_cache = shell_assignment_value(TEST_ENVIRONMENT_STAGE_SCRIPT, "local run_times_file cross_run_cache=")
    if not cross_run_cache:
        with open(os.path.join(LIBRARY_ROOT, TEST_ENVIRONMENT_STAGE_SCRIPT), encoding="utf-8") as handle:
            entries = TEST_ENVIRONMENT_STAGE_SHARED_ENTRY.findall(handle.read())
        if len(entries) != 1:
            raise ValueError(f"{TEST_ENVIRONMENT_STAGE_SCRIPT} 里调 lib_work_root.py shared <名> 取目录的写法有 {len(entries)} 处（要恰好一处）："
                             "那份脚本改了写法，照新写法改 test_environment_stage_run_times_file")
        cross_run_cache = work_root_library().shared_data_path(entries[0])
    return shell_assignment_value(TEST_ENVIRONMENT_STAGE_SCRIPT, "run_times_file=", {"cross_run_cache": cross_run_cache})


def cross_run_cache_registry():
    """仓里有意跨轮复用的缓存（文件头「跨轮缓存的登记」）：[(名字, 放它的脚本, 现取它的位置的函数)]；现取的是那份脚本自己算位置的那一段，不在这里另抄一份位置。
    写成函数、不写成模块级的表：表里载入 crates-mutation-rows.py 的那一项不随 import 时执行的模块级语句进别的闭包（文件头「Python 闭包」）。"""
    return (
        ("Rust 词法单元缓存", "scripts/gate/lib_function_units.py", rust_lex_cache_directory),
        ("产品行为视图编出的 unit-graph 程序", "scripts/gate/lib_function_units.py", unit_graph_product_view_cache_directory),
        ("crates 变异的判法单元指纹缓存", "scripts/mutation/crates-mutation-rows.py", crates_mutation_judging_cache_directory),
        ("test-environment 记的上一趟各阶段用时", TEST_ENVIRONMENT_STAGE_SCRIPT, test_environment_stage_run_times_file),
        shell_assignment_row("LKMM 判法取的内核内存模型", ".claude/scripts/lkmm.sh", "CACHE="),
        shell_assignment_row("取依赖脚本的下载缓存", ".claude/scripts/fetch-deps.sh", "CACHE="),
    )


def cross_run_cache_rows():
    """→ [(名字, 脚本, 路径或 None, 算不出的原因或 None)]，照 cross_run_cache_registry 的次序、照此刻的环境现取。"""
    rows = []
    for name, script, locate in cross_run_cache_registry():
        try:
            path, problem = locate(), None
        except Exception as error:  # 现取的那一段在别的脚本里：载入或调用抛什么都照「算不出」报，不让一份带垮别的几份
            path, problem = None, f"{type(error).__name__}：{error}"
        if path is None and problem is None:
            problem = "取到 None（缓存关着：FUNCTION_UNITS_LEX_CACHE_DIRECTORY=off 这一类改放处变量设着）"
        rows.append((name, script, path, problem))
    return rows


def print_cross_run_cache_rows():
    """命令行 cross-run-caches（自证 CROSS_RUN_CACHE_CELL 在拷贝里起它）：先打 tempdir 一行（此刻的 tempfile.gettempdir()），再每份缓存一行，交退出码（有算不出的交 3）。"""
    print(f"tempdir\t{tempfile.gettempdir()}")
    rows = cross_run_cache_rows()
    for name, script, path, problem in rows:
        print(f"cache\t{name}\t{script}\t{path}" if problem is None else f"refused-cache\t{name}\t{script}\t{problem}")
    unresolved = [name for name, _script, _path, problem in rows if problem is not None]
    if unresolved:
        print(f"  ✗ {len(unresolved)} 份跨轮缓存的位置算不出：{'、'.join(unresolved)}", file=sys.stderr)
        print("    → 怎么办：看 refused-cache 那几行的原因，照那份脚本此刻的写法改 cross_run_cache_registry 里它那一行", file=sys.stderr)
        return 3
    print(f"  ✓ 现取 {len(rows)} 份跨轮缓存的位置（登记表 cross_run_cache_registry）", file=sys.stderr)
    return 0


def rust_lex_cache_directory():
    """缓存放哪个目录；交 None 是不用缓存。设了 FUNCTION_UNITS_LEX_CACHE_DIRECTORY 取它（off 是不用），没设照 cross_run_cache_directory 放 singlefs-rust-lex-cache。"""
    configured = os.environ.get(RUST_LEX_CACHE_DIRECTORY_VARIABLE, "").strip()
    if configured == "off":
        return None
    return configured or cross_run_cache_directory(RUST_LEX_CACHE_DIRECTORY_NAME)


def rust_lex_cache_namespace():
    """缓存按「切词法单元的代码」分目录：这份库自己的内容 sha256 加此刻的弄坏开关。库的内容一变（rust_lex 与它的帮手都在里面），旧目录整个不再取。"""
    breaks = ",".join(sorted(active_breaks()))
    if breaks not in _RUST_LEX_CACHE_NAMESPACES:
        with open(os.path.realpath(__file__), "rb") as handle:
            _RUST_LEX_CACHE_NAMESPACES[breaks] = hashlib.sha256(handle.read() + b"\0" + breaks.encode("utf-8")).hexdigest()[:24]
    return _RUST_LEX_CACHE_NAMESPACES[breaks]


def rust_lex_cache_path(text):
    """这份文本的缓存文件；不用缓存交 None。文件名是文本的 sha256（只看内容，不看路径与修改时刻）；
    弄坏开关 rust-lex-cache-unverified：文件名只取文本长度（缓存不核内容）。"""
    root = rust_lex_cache_directory()
    if root is None:
        return None
    name = f"length-{len(text)}" if "rust-lex-cache-unverified" in active_breaks() else hashlib.sha256(text.encode("utf-8", "surrogateescape")).hexdigest()
    return os.path.join(root, rust_lex_cache_namespace(), name + ".json.z")


def load_rust_lex(path):
    """缓存文件读成 (kinds, texts, lines, documents)；读不了、不是这一版的写法、各表对不上都交 None（照常现切）。"""
    try:
        with open(path, "rb") as handle:
            stored = json.loads(zlib.decompress(handle.read()))
        if stored["form"] != RUST_LEX_CACHE_FORM:
            return None
        kind_names, table = stored["kind_names"], stored["table"]
        kinds = [kind_names[code] for code in base64.b64decode(stored["kinds"])]
        text_codes, line_numbers = array.array("I"), array.array("I")
        text_codes.frombytes(base64.b64decode(stored["texts"]))
        line_numbers.frombytes(base64.b64decode(stored["lines"]))
        texts = [table[code] for code in text_codes]
        documents = [tuple(entry) for entry in stored["documents"]]
        if not (len(kinds) == len(texts) == len(line_numbers)) or any(len(entry) != 5 for entry in documents):
            return None
        return kinds, texts, list(line_numbers), documents
    except (OSError, ValueError, KeyError, IndexError, TypeError, EOFError, zlib.error):
        return None


def store_rust_lex(path, result):
    """rust_lex 的结果原子写成缓存文件（同目录排他建临时文件再改名）；新开一个目录时只留最近 RUST_LEX_CACHE_KEPT_NAMESPACES 个；写不成不影响这一趟。"""
    kinds, texts, lines, documents = result
    kind_names = sorted(set(kinds))
    kind_codes = {name: code for code, name in enumerate(kind_names)}
    table = list(dict.fromkeys(texts))
    text_codes = {text: code for code, text in enumerate(table)}
    payload = json.dumps(dict(form=RUST_LEX_CACHE_FORM, kind_names=kind_names, kinds=base64.b64encode(bytes(kind_codes[kind] for kind in kinds)).decode("ascii"),
                              table=table, texts=base64.b64encode(array.array("I", [text_codes[text] for text in texts]).tobytes()).decode("ascii"),
                              lines=base64.b64encode(array.array("I", lines).tobytes()).decode("ascii"), documents=documents))
    directory = os.path.dirname(path)
    try:
        fresh = not os.path.isdir(directory)
        os.makedirs(directory, exist_ok=True)
        handle, temporary = tempfile.mkstemp(prefix=".partial-", dir=directory)
        try:
            with os.fdopen(handle, "wb") as stream:
                stream.write(zlib.compress(payload.encode("ascii"), 1))
            os.replace(temporary, path)
        except BaseException:
            if os.path.exists(temporary):
                os.unlink(temporary)
            raise
        if fresh:
            root = os.path.dirname(directory)
            kept = sorted((name for name in os.listdir(root) if os.path.isdir(os.path.join(root, name))),
                          key=lambda name: os.path.getmtime(os.path.join(root, name)), reverse=True)
            for name in kept[RUST_LEX_CACHE_KEPT_NAMESPACES:]:
                shutil.rmtree(os.path.join(root, name), ignore_errors=True)
    except OSError:
        return


def rust_lex_cached(text):
    """rust_lex(text) 的磁盘缓存版：→ 与 rust_lex(text) 逐项相同的 (kinds, texts, lines, documents)，每次是新的列表。有缓存取缓存，没有现切再存。"""
    path = rust_lex_cache_path(text)
    loaded = load_rust_lex(path) if path is not None else None
    if loaded is not None:
        RUST_LEX_CACHE_STATS["reused"] += 1
        return loaded
    result = rust_lex(text)
    RUST_LEX_CACHE_STATS["lexed"] += 1
    if path is not None:
        store_rust_lex(path, result)
    return result


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
# 按名字找得到时用的是 defines 里那几个名字、不是自己名字的种类（crates/singlefs-unit-graph 的 declared_names）：条目级宏调用与 extern 块
# 是它里面定义的名字，assume_specification 是方括号里点名路径末段的名字
RUST_KINDS_DEFINING_NAMES = frozenset(("item_macro", "extern_block", "assume_specification"))
RUST_TYPE_KINDS = frozenset(("struct", "enum", "union", "type"))
def use_path_root(rust_file, first, last):
    """use 语句（词 first..last）的路径根名：use 之后头一个不是 : 的词，是标识符才算（use {…} 没有根名）→ 文本或 None。"""
    position = first
    while position < last and not (rust_file.kinds[position] == IDENT and rust_file.texts[position] == "use"):
        position += 1
    position += 1
    while position < last and rust_file.texts[position] == ":":
        position += 1
    return rust_file.texts[position] if position < last and rust_file.kinds[position] == IDENT else None


# R-裸名按模块筛候选时，路径根名是这几个的指标准库，不点本目标里的哪一份（另几个是看得见的别的 crate 的库名；RustWorkspace.candidates_seen_from_module）
STANDARD_CRATE_PATH_ROOTS = ("std", "core", "alloc")
RUST_TYPE_OR_TRAIT_KINDS = RUST_TYPE_KINDS | {"trait"}   # 前缀::名 的前缀是这几种之一的，名是关联项（RustWorkspace.candidates_seen_from_module）
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
        self._manifest_constants = None   # manifest_constants 第一次用到时算
        self.source_rows = text.split("\n")
        self.kinds, self.texts, self.lines, self.documents = rust_lex_cached(text)
        self.count = len(self.texts)
        self.units, self.refusals, self.mod_declarations, self.doctests = [], [], [], []
        self.verus_spans = []           # [(verus! 块的 { 下标, 配对的 } 下标)]：产品行为视图认只给 Verus 看的单元用（文件头「判定种类」）
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

    def inside_verus(self, unit):
        """单元落在某一块 verus! { … } 里。"""
        return unit.first_token is not None and any(opening < unit.first_token < close for opening, close in self.verus_spans)

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
        if text == ";":
            # ; 本身不是起头：} 之后紧跟的 ; 归前一个条目（const X: S = S { … }; 的指纹带它，crates/singlefs-unit-graph 的 verus_item_starts 同）
            return "rust-verus-semicolon-split-off" in self.breaks
        if text in RUST_ITEM_STARTS:
            return True
        if self.is_assume_specification_keyword(c):
            return True
        return self.macro_path_at(c) is not None

    def is_assume_specification_keyword(self, c):
        """c 是 verus! 里 assume_specification 条目的关键字：这个词后面接 [（点名的路径）或 <（泛型参数表）
        （crates/singlefs-unit-graph 的 is_assume_specification_keyword）。弄坏开关 rust-assume-specifications-unrecognized 下不认。"""
        return (self.t(c) == "assume_specification" and self.t(c + 1) in ("[", "<")
                and "rust-assume-specifications-unrecognized" not in self.breaks)

    def assume_specification_target_name(self, after_keyword, end):
        """assume_specification 之后（可带一段 <…> 泛型参数表）那对方括号里点名的路径，末段的名字（尖括号里的名字不算）：
        [ singlefs_format::WIDE_CHECKSUM_BYTES ] 交 WIDE_CHECKSUM_BYTES，[ Vec::<Element>::len ] 交 len；
        认不出那对方括号、方括号里尖括号外一个名字都没有的交 None（crates/singlefs-unit-graph 的 assume_specification_target_name）。"""
        open_bracket = after_keyword
        if self.t(after_keyword) == "<":
            depth, open_bracket = 0, None
            for position in range(after_keyword, end):
                if self.t(position) == "<":
                    depth += 1
                elif self.t(position) == ">" and (position == 0 or self.t(position - 1) != "-"):
                    depth -= 1
                    if depth < 0:
                        return None
                    if depth == 0:
                        open_bracket = position + 1
                        break
            if open_bracket is None:
                return None
        if self.t(open_bracket) != "[" or open_bracket not in self.pairs:
            return None
        angle_depth, last_name = 0, None
        for position in range(open_bracket + 1, self.pairs[open_bracket]):
            text = self.t(position)
            if text == "<":
                angle_depth += 1
            elif text == ">" and self.t(position - 1) != "-":
                angle_depth = max(angle_depth - 1, 0)
            elif angle_depth == 0 and self.kind(position) == IDENT:
                last_name = text
        return last_name

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
        if verus and self.is_assume_specification_keyword(k):
            # 整条（连属性）一个单元，名字 assume_specification@<关键字那一行>，按点名路径末段的名字找得到它（defines）；
            # 名字认不出时这份文件里谁连得到它说不全，记文件级拒算（Rust 那一份记 unrecognized-item-start）
            end = self.verus_end(k, high)
            target_name = self.assume_specification_target_name(k + 1, end)
            if target_name is None:
                self.refusals.append(Refusal("unknown-item", self.path, line, self.row(line)))
                return end
            unit = self.add("assume_specification", f"assume_specification@{line}", start, end, context, module_path, container, conditions,
                            public or member_visible)
            unit.defines = set() if "rust-assume-specification-names-unlinked" in self.breaks else {target_name}
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
                self.verus_spans.append((opening, close))
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
        unit.open_keys, unit.self_path_root = head["open_keys"], head["self_path_root"]

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
        # 自身类型写成路径 a::B 时的头一段（外部类型的 impl 认法用，RustWorkspace.index 的 foreign_trait_impls）；不是路径的是 None
        first_name = next((index for index in self_tokens if self.kind(index) == IDENT and self.t(index) not in SELF_TYPE_SKIPPED), None)
        self_path_root = self.t(first_name) if first_name is not None and self.t(first_name + 1) == ":" and self.t(first_name + 2) == ":" else None
        # blanket：自身类型里任何深度出现 impl 的任一类型或常量泛型参数（for T、&T、Box<T>、(T, u32)、[T]、[T; N]、Option<Box<T>>）；全是具体类型的按被点名的类型。
        # 弄坏开关 rust-blanket-self-parameter-only 退回旧判法：自身类型里头一个名字是泛型参数才算（Box<T>、Option<Box<T>> 不算）
        if "rust-blanket-self-parameter-only" in self.breaks:
            blanket = split is not None and bool(self_names) and self_names[0] in generics
        else:
            blanket = split is not None and bool(self_types & generics)
        return {"label": label, "trait_name": trait_name, "is_trait_impl": split is not None, "self_types": self_types,
                "blanket": blanket, "self_path_root": self_path_root,
                # R-隐式按自身类型的名字认 trait impl（crates/singlefs-unit-graph 的 implementations_by_self_type_name）：trait 参数里的类型被点名不带进来
                "implicit_keys": self_types | (trait_arguments if "rust-trait-impls-by-trait-arguments" in self.breaks else set()),
                # R-开放按 trait 自己的名字找 impl（impl Wrap<dyn Sink> for H 不是 Sink 的 impl）；
                # 弄坏开关 rust-open-by-trait-arguments 下另按 trait 参数里的名字找（crates/singlefs-unit-graph 现在按 trait_part_names 建索引的写法）
                "open_keys": ({trait_name} if trait_name else set())
                             | (trait_arguments - {"dyn"} if "rust-open-by-trait-arguments" in self.breaks else set())}

    def use_imports(self, low, high):
        """→ (引进的名字, [(别名, 原名)], 有没有 *)：路径段（后面接 ::）与 self / super / crate 不算名字，a as b 两边都算名字、记一条别名
        （as 两边都是标识符才记）；花括号里的 self 引进的是 { 前面那一段（use std::io::{self, Seek} 引进 io，self as c 记别名 (c, io)）。
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
                    elif value and not value.startswith("CARGO"):
                        # cargo 不给的变量：值只能来自构建脚本的 cargo::rustc-env（文件头 Rust 闭包「构建脚本交的值」）
                        unit.data_references.append(("build-environment", value, i))
                    i = close + 1
                    continue
                if text == "option_env" and self.kind(i + 3) == LITERAL:
                    value = rust_string_value(self.t(i + 3))
                    if value and value != "OUT_DIR" and not value.startswith("CARGO"):
                        unit.data_references.append(("build-environment", value, i))
                    i = close + 1
                    continue
            if kind == IDENT and text in self.manifest_constants()[0] and self.t(i - 1) != "const":
                self.manifest_constant_use(unit, i)
            if kind == IDENT and text == "current_exe" and self.t(i + 1) == "(":
                child = None if "rust-current-exe-refused" in self.breaks else self.current_exe_child(unit, i)
                if child is not None:
                    unit.data_references.append(("current-exe-child", child, i))
                elif "rust-current-exe-path-only-refused" in self.breaks or (self.starts_a_process(unit) and "rust-current-exe-spawn-unrefused" not in self.breaks):
                    self.refuse(unit, "current-exe", i)
            if kind == IDENT and text in ("var_os", "var") and self.t(i + 1) == "(" and rust_string_value(self.t(i + 2)) == "CARGO_MANIFEST_DIR":
                self.refuse(unit, "runtime-path", i)
            i += 1

    # current_exe()：测试二进制起自己（子进程里只跑一条点名的测试）

    def current_exe_child(self, unit, index):
        """current_exe 这个词在 index。写成 Command::new(…current_exe()…) 后接 .arg(…) / .args([…]) 链、参数里恰好有一个字面 "--exact"、
        紧跟在它后面的那个参数折得出测试名的，→ 要连的测试：("pieces", 几段)（几段是 ("literal", 串) 与 ("constant", 常量名)：字面串、常量名、
        全由这两样组成的 format!("…{}…{名}…", …) 与 concat!(…)；常量名在 rust_closure_of 的 constant_string 里解，不在这里找）、("parameter", 函数名, 位置)
        （名字是本单元这个 fn 自己的参数：rust_closure_of 取闭包里每一处调用它的那个实参各折一遍）；折不出的回 None（拒算 current-exe）。
        测试名写在 --exact 前面、用 module_path!() 拼的、套着 as_str() 的、在单元里另有 let 绑定的，都折不出。"""
        j = index
        while self.t(j - 1) == ":" and self.t(j - 2) == ":" and self.kind(j - 3) == IDENT:
            j -= 3
        opening = j - 1
        if not (self.t(opening) == "(" and self.t(opening - 1) == "new" and self.t(opening - 2) == ":" and self.t(opening - 3) == ":"
                and self.t(opening - 4) == "Command" and opening in self.pairs):
            return None
        arguments, c = [], self.pairs[opening] + 1
        while self.t(c) == "." and self.kind(c + 1) == IDENT and self.t(c + 2) == "(" and (c + 2) in self.pairs:
            method, inner_low, inner_close = self.t(c + 1), c + 3, self.pairs[c + 2]
            if method == "arg":
                arguments += self.top_level_parts(inner_low, inner_close)
            elif method == "args":
                k = inner_low + 1 if self.t(inner_low) == "&" else inner_low
                if not (self.t(k) == "[" and self.pairs.get(k) == inner_close - 1):
                    return None
                arguments += self.top_level_parts(k + 1, inner_close - 1)
            c = inner_close + 1
        flags = [position for position, (part_low, part_high) in enumerate(arguments)
                 if part_high == part_low + 1 and self.kind(part_low) == LITERAL and rust_string_value(self.t(part_low)) == "--exact"]
        if len(flags) != 1 or flags[0] + 1 >= len(arguments):
            return None
        return self.fold_test_name(unit, arguments[flags[0] + 1][0], arguments[flags[0] + 1][1], 0)

    def starts_a_process(self, unit):
        """这个单元里有没有起子进程的词（SPAWNING_WORDS）。没有的话 current_exe() 只是在问路径，不拒算；单元没有词法范围的按起了算。"""
        if unit.first_token is None:
            return True
        return any(self.kind(i) == IDENT and self.t(i) in SPAWNING_WORDS for i in range(unit.first_token, unit.last_token))

    def fold_test_name(self, unit, low, high, depth):
        """测试名那一段词法单元 [low, high) → ("pieces", 几段) | ("parameter", 函数名, 位置) | None（折不出）；几段是 ("literal", 串) 与 ("constant", 常量名)。
        认：串字面量；常量名（只记名字，值在 rust_closure_of 的 constant_string 里解）；concat!(…)；format!("…{}…{名}…", …)（只认 {} 与 {名}，各块都折得出）；
        本单元这个 fn 的参数名。"""
        if depth > 6 or low >= high:
            return None
        if high == low + 1 and self.kind(low) == LITERAL:
            value = rust_string_value(self.t(low))
            return ("pieces", (("literal", value),)) if value is not None else None
        if self.kind(low) == IDENT and self.t(low + 1) == "!" and self.t(low + 2) == "(" and self.pairs.get(low + 2) == high - 1:
            return self.fold_test_name_macro(unit, self.t(low), low + 3, high - 1, depth)
        names = [low]
        while names[-1] + 3 < high and self.t(names[-1] + 1) == ":" and self.t(names[-1] + 2) == ":" and self.kind(names[-1] + 3) == IDENT:
            names.append(names[-1] + 3)
        if self.kind(low) == IDENT and names[-1] + 1 == high:
            return self.fold_test_name_binding(unit, self.t(names[-1]), depth)
        return None

    def fold_test_name_macro(self, unit, macro, low, high, depth):
        parts = self.top_level_parts(low, high)
        if macro == "concat":
            pieces = [self.fold_test_name(unit, part_low, part_high, depth + 1) for part_low, part_high in parts]
            if not pieces or any(piece is None or piece[0] != "pieces" for piece in pieces):
                return None
            return ("pieces", tuple(segment for piece in pieces for segment in piece[1]))
        if macro != "format" or not parts or parts[0][1] != parts[0][0] + 1 or self.kind(parts[0][0]) != LITERAL:
            return None
        template = rust_string_value(self.t(parts[0][0]))
        positional = parts[1:]
        if template is None or any(part[1] - part[0] >= 3 and self.kind(part[0]) == IDENT and self.t(part[0] + 1) == "=" for part in positional):
            return None
        pieces, next_argument = [], 0
        for found in re.finditer(r"\{\{|\}\}|\{([A-Za-z_][A-Za-z0-9_]*)?\}|[{}]|[^{}]+", template):
            text = found.group(0)
            if text in ("{{", "}}"):
                pieces.append(("literal", text[0]))
            elif text.startswith("{") and text.endswith("}") and len(text) >= 2:
                if found.group(1) is None:
                    if next_argument >= len(positional):
                        return None
                    piece = self.fold_test_name(unit, positional[next_argument][0], positional[next_argument][1], depth + 1)
                    next_argument += 1
                else:
                    piece = self.fold_test_name_binding(unit, found.group(1), depth + 1)
                if piece is None or piece[0] != "pieces":
                    return None
                pieces.extend(piece[1])
            elif text in ("{", "}"):
                return None
            else:
                pieces.append(("literal", text))
        if next_argument != len(positional):
            return None
        return ("pieces", tuple(pieces))

    def fold_test_name_binding(self, unit, name, depth):
        """一个名字折成什么：单元里另有 let 绑定的 None（折不出）；是本单元这个 fn 的参数的，("parameter", 函数名, 位置)；
        别的当常量名，("pieces", (("constant", 名),))，值由 rust_closure_of 的 constant_string 解（本单元里的 const，再是同一个包链接集里的常量单元）。"""
        if unit.first_token is not None and any(
                self.texts[i] == "let" and (self.t(i + 1) == name or (self.t(i + 1) == "mut" and self.t(i + 2) == name))
                for i in range(unit.first_token, unit.last_token)):
            return None
        position = self.parameter_position(unit, name)
        return ("parameter", unit.short, position) if position is not None else ("pieces", (("constant", name),))

    def string_constant_in(self, low, high, name):
        """[low, high) 里第一处 const <名> : … = "…" ; 的串值（普通串字面量，不折表达式）；没有这一处、或写法不是它回 None
        （crates/singlefs-unit-graph 的 constant_string_value 里 definition_value 同写）。"""
        if low is None:
            return None
        for i in range(low, high):
            if self.texts[i] == "const" and self.t(i + 1) == name and self.t(i + 2) == ":":
                equals = next((c for c in range(i + 3, high) if self.texts[c] == "="), None)
                if equals is not None and self.kind(equals + 1) == LITERAL and self.t(equals + 1)[:1] == '"' and self.t(equals + 2) == ";":
                    return rust_string_value(self.t(equals + 1))
                return None
        return None

    def parameter_position(self, unit, name):
        """unit 是 fn 单元、<名> 是它的参数（不数 self）时，它是第几个参数（从 0 数）；不是回 None。"""
        if unit.kind != "fn" or unit.first_token is None:
            return None
        i = unit.first_token
        while i < unit.last_token and self.texts[i] != "fn":
            i += 1
        i += 2
        if self.t(i) == "<":
            depth = 0
            while i < unit.last_token:
                depth += 1 if self.texts[i] == "<" else -1 if self.texts[i] == ">" and self.t(i - 1) != "-" else 0
                i += 1
                if depth == 0:
                    break
        if self.t(i) != "(" or i not in self.pairs:
            return None
        close, parts, start, depth, c = self.pairs[i], [], i + 1, 0, i + 1
        while c < close:
            text = self.texts[c]
            if text in ("(", "[", "{") and c in self.pairs:
                c = self.pairs[c] + 1
                continue
            if text == "<":
                depth += 1
            elif text == ">" and self.t(c - 1) != "-":
                depth -= 1
            elif text == "," and depth == 0:
                parts.append((start, c))
                start = c + 1
            c += 1
        parts.append((start, close))
        count = 0
        for low, high in parts:
            words = [self.texts[k] for k in range(low, high) if self.kinds[k] != LIFETIME and self.texts[k] not in ("&", "mut")]
            if not words or words[0] == "self":
                continue
            if len(words) >= 2 and words[0] == name and words[1] == ":":
                return count
            count += 1
        return None

    def call_site_test_names(self, unit, function_name, position):
        """unit 的词法单元里每一处 <函数名>(…) 调用（不是 fn 定义、不是 .方法 调用）的第 position 个实参折出的测试名的几段 → [几段]（常量名由调用方在 unit 的上下文里解）；
        有一处折不出（实参不够、又是参数）回 None。"""
        found = []
        for i in range(unit.first_token or 0, unit.last_token or 0):
            if self.texts[i] != function_name or self.t(i + 1) != "(" or (i + 1) not in self.pairs or self.t(i - 1) in ("fn", "."):
                continue
            parts = self.top_level_parts(i + 2, self.pairs[i + 1])
            if position >= len(parts):
                return None
            reached = self.fold_test_name(unit, parts[position][0], parts[position][1], 0)
            if reached is None or reached[0] != "pieces":
                return None
            found.append(reached[1])
        return found

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
        #   读不到才拒算 include-missing。弄坏开关 rust-includes-outside-root-refused 下根外一律拒算
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
        """env!("CARGO_MANIFEST_DIR")（或指到它的私有常量，下标取常量名那一个词）所在的一处（crates/singlefs-unit-graph 的 fold_manifest_directory_read）：
        Path::new(…) / PathBuf::from(…) 里再接 .join("字面") 链的，交 fold_manifest_chain 折（文件头「数据目录」）：折出文件的收成数据文件（文件的字节进键），
        crates/ 之下（不含 crates/ 整个）的目录收成数据目录，绑给 let 变量的目录按这个变量每一处用法各折一样；
        折出来的每一样都记在这一处：读记一条（同一样只记一次），折不出的那一样拒算 runtime-path、别的照收。别的写法拒算 runtime-path。"""
        opening = environment_index - 1
        if self.t(opening) == "(" and self.pairs.get(opening) == environment_close + 1 and self.t(opening - 1) in ("new", "from"):
            folds = self.fold_manifest_chain(unit, self.crate_directory or ".", environment_close + 2, self.manifest_path_expression_start(opening - 1), 0)
            recorded = []
            for fold in folds:
                if fold == MANIFEST_FOLD_REFUSED:
                    self.refuse(unit, "runtime-path", environment_index)
                elif fold not in recorded:
                    recorded.append(fold)
            for kind, path in recorded:
                unit.data_references.append((kind, path, environment_index))
            return
        if environment_index in self.manifest_constants()[1]:
            return   # const <名>: &str = env!("CARGO_MANIFEST_DIR");：常量本身不读什么，用到它的每一处各自折（manifest_constant_use）
        self.refuse(unit, "runtime-path", environment_index)

    def fold_manifest_chain(self, unit, base, chain_start, expression_start, depth):
        """从 chain_start 起接字面 .join("…") 链拼在 base 后面（crates/singlefs-unit-graph 的 fold_path_chain）→ [("file" 或 "directory", 路径) 或 MANIFEST_FOLD_REFUSED]。
        链后还接任何方法（.parent()、.join(<不是字面>) 这一类）、折出根外或盘上没有的路径：MANIFEST_FOLD_REFUSED。折出文件：那份文件。
        折出目录、表达式 [expression_start, 链尾) 又整个是 let <名> [: 类型] = …; 的右边（不论是不是能收的目录）：<名> 每一处用法各折的（fold_manifest_binding_use；
        后面没用到的照目录本身判），已经跟了 DEEPEST_MANIFEST_BINDING_CHAIN 层的拒算；不绑 let 的目录：在 crates/ 之下的（is_data_directory）收成目录，别的拒算。
        depth 是已经跟过的 let 层数（env! 那一处是 0）。
        弄坏开关 rust-manifest-binding-refused-whole：不是能收的目录不跟 let、拒算；有一处用法折不出就整处只交一个 MANIFEST_FOLD_REFUSED。"""
        path, c = base, chain_start
        while self.t(c) == "." and self.t(c + 1) == "join" and self.t(c + 2) == "(" and self.kind(c + 3) == LITERAL and self.t(c + 4) == ")":
            value = rust_string_value(self.t(c + 3))
            if value is None:
                break
            path = posixpath.normpath(posixpath.join(path, value))
            c += 5
        if self.t(c) == "." or path == ".." or path.startswith("../"):
            return [MANIFEST_FOLD_REFUSED]
        if os.path.isfile(os.path.join(self.root, path)):
            return [("file", path)]
        if not os.path.isdir(os.path.join(self.root, path)):
            return [MANIFEST_FOLD_REFUSED]
        # 弄坏开关：env! 那一处拼出的是目录也照放行（目录底下读什么键里都看不见）；平常目录一律拒算，要仓根的点名清单目录里的一份文件再往上取
        if depth == 0 and "rust-manifest-directory-reads-unrefused" in active_breaks():
            return []
        directory_itself = ("directory", path) if self.is_data_directory(path) else MANIFEST_FOLD_REFUSED
        variable = self.manifest_binding_name(expression_start, c)
        refused_whole = "rust-manifest-binding-refused-whole" in self.breaks
        if variable is None or (refused_whole and directory_itself == MANIFEST_FOLD_REFUSED):
            return [directory_itself]
        if depth >= DEEPEST_MANIFEST_BINDING_CHAIN:
            return [MANIFEST_FOLD_REFUSED]
        uses = self.manifest_binding_uses(variable, c + 1, unit.last_token)
        if not uses:
            return [directory_itself]
        folds = [fold for use_index in uses for fold in self.fold_manifest_binding_use(unit, path, use_index, depth + 1)]
        if refused_whole and MANIFEST_FOLD_REFUSED in folds:
            return [MANIFEST_FOLD_REFUSED]
        return folds

    def is_data_directory(self, path):
        """折出的路径是不是能收成数据目录的目录：仓内、真是目录、在 crates/ 之下、不是 crates/ 整个。"""
        if "rust-manifest-directories-refused" in self.breaks:
            return False
        return path.startswith("crates/") and not path.startswith("../") and os.path.isdir(os.path.join(self.root, path))

    def manifest_path_expression_start(self, constructor_index):
        """std::path::Path::new 这类写法起头的那个词法单元：从 new / from 往回跨过 <名字> ::（crates/singlefs-unit-graph 的 path_expression_start）。"""
        start = constructor_index
        while self.t(start - 1) == ":" and self.t(start - 2) == ":" and self.kind(start - 3) == IDENT:
            start -= 3
        return start

    def manifest_binding_name(self, expression_start, after_expression):
        """[expression_start, after_expression) 整个是 let [mut] <名> [: 类型] = …; 的右边时交 <名>，别的交 None（crates/singlefs-unit-graph 的 let_binding_name）。"""
        if self.t(after_expression) != ";" or self.t(expression_start - 1) != "=":
            return None
        equals = expression_start - 1
        let_index = next((position for position in range(equals - 1, -1, -1) if self.texts[position] in ("let", ";", "{", "}", "=")), None)
        if let_index is None or self.texts[let_index] != "let":
            return None
        name_index = let_index + 1
        if self.t(name_index) == "mut":
            name_index += 1
        if self.kind(name_index) != IDENT:
            return None
        plain = name_index + 1 == equals
        typed = self.t(name_index + 1) == ":" and self.t(name_index + 2) != ":"
        return self.t(name_index) if plain or typed else None

    def manifest_binding_uses(self, variable, low, high):
        """<变量> 在 [low, high) 里的每一处用法的下标（crates/singlefs-unit-graph 的 uses_of_the_binding）：.<变量> 的字段或方法、<变量>:: 的路径段不算；
        碰到再 let 这个名字就停（之后是另一个值）。"""
        uses = []
        for j in range(low, high):
            if self.kind(j) != IDENT or self.t(j) != variable:
                continue
            previous = self.t(j - 1)
            if previous == "let" or (previous == "mut" and self.t(j - 2) == "let"):
                break
            if previous == "." or (self.t(j + 1) == ":" and self.t(j + 2) == ":"):
                continue
            uses.append(j)
        return uses

    def fold_manifest_binding_use(self, unit, value, use_index, depth):
        """绑着路径 value 的变量的一处用法（crates/singlefs-unit-graph 的 fold_use_of_a_path）：Path::new(<变量>) / PathBuf::from(<变量>) 之后、
        或变量本身之后接的字面 .join 链照 fold_manifest_chain 折；变量单独交给别的函数的，折出的就是它绑着的目录。"""
        if self.t(use_index - 1) == "(" and self.t(use_index - 2) in ("new", "from") and self.t(use_index + 1) == ")":
            return self.fold_manifest_chain(unit, value, use_index + 2, self.manifest_path_expression_start(use_index - 2), depth)
        return self.fold_manifest_chain(unit, value, use_index + 1, use_index, depth)

    def manifest_constants(self):
        """→ ({私有常量名}, {它初始值里那个 env 词的下标})：const <名>: <类型> = env!("CARGO_MANIFEST_DIR"); 初始值整个就是那一个宏、没写 pub 的。
        公开的常量别的文件也能用到，折不全，照拒算。"""
        if self._manifest_constants is None:
            sites = {}   # env 词的下标 → 常量名
            for i in range(self.count):
                if not (self.texts[i] == "env" and self.t(i + 1) == "!" and self.t(i + 2) == "(" and self.kind(i + 3) == LITERAL
                        and rust_string_value(self.t(i + 3)) == "CARGO_MANIFEST_DIR" and self.t(i + 4) == ")" and self.t(i - 1) == "=" and self.t(i + 5) == ";"):
                    continue
                j = i - 2
                while j >= 0 and self.texts[j] not in (";", "{", "}", "]") and self.texts[j] != "const":
                    j -= 1
                if self.t(j) != "const" or self.kind(j + 1) != IDENT:
                    continue
                before = 0
                while before < 6 and j - 1 - before >= 0 and self.texts[j - 1 - before] not in (";", "{", "}", "]"):
                    before += 1
                if any(self.texts[k] == "pub" for k in range(j - before, j)):
                    continue
                sites[i] = self.texts[j + 1]
            # 串字面量里有 {<名> 的（format! 一族的内联捕获）：名字不经 Path::new 就用到了，照拒算
            captured = {name for name in sites.values() if any(self.kinds[k] == LITERAL and "{" + name in self.texts[k] for k in range(self.count))}
            sites = {i: name for i, name in sites.items() if name not in captured}
            self._manifest_constants = (set(sites.values()), set(sites))
        return self._manifest_constants

    def manifest_constant_use(self, unit, index):
        """私有常量（manifest_constants）出现在 Path::new(<常量>) / PathBuf::from(<常量>) 里的，照 env!("CARGO_MANIFEST_DIR") 同一处折；别的写法拒算 runtime-path。"""
        opening = index - 1
        if self.t(opening) == "(" and self.pairs.get(opening) == index + 1 and self.t(opening - 1) in ("new", "from"):
            self.runtime_manifest_path(unit, index, index)
        else:
            self.refuse(unit, "runtime-path", index)

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
        self.outside_workspace = False   # discover_rust_crates 顺 path 依赖收进来的工作区外 crate 设真（文件头 Rust 闭包「工作区外的 path 依赖」）
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
    crates, pending = [], list(dict.fromkeys(directories))
    known = set(pending)
    workspace_members = set(pending)
    follow_path_dependencies = "rust-excluded-path-dependencies-dropped" not in active_breaks()
    while pending:
        directory = pending.pop(0)
        crate_manifest = manifest if directory == "" else load_manifest(root, posixpath.join(directory, "Cargo.toml"))
        crate = RustCrate(root, directory, crate_manifest, shared)
        crate.outside_workspace = directory not in workspace_members
        crates.append(crate)
        # 工作区外的 path 依赖（根清单 exclude 里、或没列进 members、落在仓根底下的；文件头 Rust 闭包「工作区外的 path 依赖」）：顺依赖边收成一个 crate，
        # 调到它的入口照连；落在仓根外的不收。弄坏开关 rust-excluded-path-dependencies-dropped：不收
        for dependency in (item for section in ("normal", "dev", "build") for item in crate.dependencies[section]) if follow_path_dependencies else ():
            place = dependency.directory
            if (place is None or place in known or place in (".", "..") or place.startswith("../") or posixpath.isabs(place)
                    or not os.path.isfile(os.path.join(root, place, "Cargo.toml"))):
                continue
            known.add(place)
            pending.append(place)
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
    def __init__(self, root, judgment_kind=FORMAL_VERIFICATION, product_dump=None):
        if not os.path.isdir(root):
            raise FunctionUnitsError(f"根不在：{root}")
        if judgment_kind not in JUDGMENT_KINDS:
            raise UsageError(f"判定种类只认 {'、'.join(JUDGMENT_KINDS)}：{judgment_kind}")
        self.root = root
        self.breaks = active_breaks()
        # 产品行为视图（文件头「判定种类」）：product_dump 是调用方交的 unit-graph dump --judgment-kind product-behavior 的行（对拍格与自证用），
        # 不给就要用时起 unit-graph；product_view 是按文件分好的那一份（{文件: ([单元行], [拒算行])}），拿不到时是一句原因
        self.judgment_kind, self.product_dump, self.product_view = judgment_kind, product_dump, None
        self.crates = discover_rust_crates(root)
        self.crate_by_directory = {crate.directory: crate for crate in self.crates}
        self.files = {}
        self.targets = {}
        self.names_assigned = False
        self.shell_sources = {}   # 读到的 shell 脚本 → shell_sourced_files 的结果（shell_sourced）
        for crate in self.crates:
            for target in crate_targets(root, crate):
                self.targets[target.id] = target
                directory = posixpath.dirname(target.root_file)
                self.walk(target, target.root_file, [], [], directory)
        for path in sorted(self.files):
            self.assign_full_names(self.files[path])
        self.names_assigned = True
        self.index()
        for path in sorted(self.files):
            self.apply_product_view(self.files[path])

    def shell_sourced(self, path):
        """shell_sourced_files(根, path)，一个工作区里一份脚本只折一趟（几十个入口读同一份脚本）。"""
        if path not in self.shell_sources:
            self.shell_sources[path] = shell_sourced_files(self.root, path)
        return self.shell_sources[path]

    def product_view_lines(self):
        """Rust 那一份产品行为视图的 dump，按文件分好：{文件: ([单元行], [拒算行])}；拿不到交一句原因。一个工作区只取一次。"""
        if self.product_view is None:
            lines = self.product_dump
            if lines is None:
                try:
                    lines = product_view_dump(self.root)
                except ProductViewUnavailable as error:
                    self.product_view = str(error)
                    return self.product_view
            grouped = {}
            for line in lines:
                fields = line.split("\t")
                if len(fields) < 3 or fields[0] in ("data", "data-directory"):
                    continue
                place = grouped.setdefault(fields[2].rpartition(":")[0], ([], []))
                if fields[0] == "refused":
                    place[1].append(fields)
                elif len(fields) == 4:
                    place[0].append(line)
            self.product_view = grouped
        return self.product_view

    def product_view_file(self, rust_file, text, directory):
        """产品行为视图里有 verus! 块、或写着 #[verifier::…] / #[verifier(…)] 外属性的文件，载入时先照 Rust 那一份的剥法在这里剥一遍（verus_ghost_code_blanked）、
        重切：重切出来的单元与 Rust 那一份交的这份文件的单元逐个核得上（product_view_agrees）就用剥过的这一份，闭包的边、cfg、数据与文档测试都从剥过的词法单元取
        （与 unit-graph 剥完再切、再收闭包同一做法）；核不上、Rust 那一份解不开这份文件、拿不到 Rust 那一份的，照原样那一份交给 apply_product_view（文件头「判定种类」）。
        弄坏开关 product-view-edges-from-the-written-tokens 下一律照原样那一份；product-view-blanking-unverified 下剥过的那一份不核就用。"""
        breaks = self.breaks
        if (self.judgment_kind != PRODUCT_BEHAVIOR or not (rust_file.verus_spans or verifier_attribute_token_ranges(rust_file.kinds, rust_file.texts))
                or breaks & {"product-view-kept-formal", "product-view-strips-executable-code", "product-view-edges-from-the-written-tokens"}):
            return rust_file
        blanked_text = verus_ghost_code_blanked(text)
        if blanked_text is None or blanked_text == text:
            return rust_file
        blanked = RustFile(self.root, rust_file.path, blanked_text, directory)
        if "product-view-blanking-unverified" in breaks:
            return blanked
        view = self.product_view_lines()
        if isinstance(view, str):
            return rust_file
        theirs_lines, theirs_refusals = view.get(rust_file.path, ([], []))
        if any(fields[1] == VERUS_GHOST_CODE_UNPARSED for fields in theirs_refusals):
            return rust_file
        return blanked if product_view_agrees(blanked, theirs_lines) else rust_file

    def apply_product_view(self, rust_file):
        """产品行为视图（文件头「判定种类」）：有 verus! 块、或写着 #[verifier::…] / #[verifier(…)] 外属性的文件照 Rust 那一份换指纹、认只给 Verus 看的单元、
        记配不上的拒算。弄坏开关 product-view-kept-formal 下照原样（不剥）；product-view-strips-executable-code 下 verus! 块里的单元一律当只给 Verus 看的（剥到执行代码）。"""
        attribute_heads = [hash_index for hash_index, _close in verifier_attribute_token_ranges(rust_file.kinds, rust_file.texts)]
        if self.judgment_kind != PRODUCT_BEHAVIOR or not (rust_file.verus_spans or attribute_heads) or "product-view-kept-formal" in self.breaks:
            return
        if "product-view-strips-executable-code" in self.breaks:
            for unit in rust_file.units:
                if rust_file.inside_verus(unit):
                    unit.ghost_only = True
            return
        view = self.product_view_lines()
        block_line = rust_file.lines[rust_file.verus_spans[0][0] if rust_file.verus_spans else attribute_heads[0]]
        if isinstance(view, str):
            rust_file.refusals.append(Refusal("product-view-unavailable", rust_file.path, block_line, snippet_of(view)))
            return
        theirs_lines, theirs_refusals = view.get(rust_file.path, ([], []))
        for fields in theirs_refusals:
            if fields[1] == VERUS_GHOST_CODE_UNPARSED:
                at = fields[2].rpartition(":")[2]
                rust_file.refusals.append(Refusal(VERUS_GHOST_CODE_UNPARSED, rust_file.path, int(at) if at.isdigit() else block_line,
                                                  fields[3] if len(fields) > 3 else ""))
        theirs, ours = {}, {}
        for line in theirs_lines:
            placed, _tab, fingerprint = line.rpartition("\t")
            theirs.setdefault(placed, []).append(fingerprint)
        for unit in rust_file.units:
            ours.setdefault(unit.dump_text().rpartition("\t")[0], []).append(unit)
        for placed in sorted(set(theirs) | set(ours)):
            fingerprints, units = theirs.get(placed, []), ours.get(placed, [])
            if len(fingerprints) == len(units) and len(set(fingerprints)) <= 1:
                for unit, fingerprint in zip(units, fingerprints):
                    unit.fingerprint = fingerprint
                continue
            if not fingerprints and all(rust_file.inside_verus(unit) for unit in units):
                for unit in units:
                    unit.ghost_only = True
                continue
            line = units[0].first_line if units else block_line
            rust_file.refusals.append(Refusal("product-view-unmatched", rust_file.path, line,
                                              snippet_of(f"{placed}：unit-graph 交 {len(fingerprints)} 个、这一份切出 {len(units)} 个")))

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
            directory = crate.directory if crate else posixpath.dirname(path)
            self.files[path] = self.product_view_file(RustFile(self.root, path, text, directory), text, directory)
            if self.names_assigned:
                self.assign_full_names(self.files[path])
                self.apply_product_view(self.files[path])
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
        # 用到时才算（names_reaching_outside_crate、candidates_seen_from_module）
        self.outside_crate_reaching_names, self.outside_crate_module_candidates, self.outside_crate_type_names = {}, {}, {}
        self.index_macro_and_foreign_edges()
        indexed = set()
        # const _ / static _ 没有名字：R-裸名的着落里不收 _（let _ = …、_ => 里的 _ 不点它；条目级宏调用、extern 块定义的名字里的 _ 同样不收）；
        #   弄坏开关 rust-underscore-items-named 下照收
        underscore_named = "rust-underscore-items-named" in active_breaks()
        literal_names_linked = "rust-macro-rules-literal-names-unlinked" not in active_breaks()
        for target in self.targets.values():
            names = self.by_name.setdefault(target.id, collections.defaultdict(list))
            for path in target.files:
                for unit in self.files[path].units:
                    if unit.kind == "doctest":
                        continue
                    self.unit_targets[id(unit)].add(target.id)
                    if unit.container is None and unit.kind in RUST_NAMED_KINDS and (unit.short != "_" or underscore_named):
                        names[unit.short].append(unit)
                    if unit.kind in RUST_KINDS_DEFINING_NAMES:
                        for name in unit.defines:
                            if name != "_" or underscore_named:
                                names[name].append(unit)
                    if unit.kind == "item_macro" and literal_names_linked:
                        # 宏体里按字面写出的名字登到这次调用名下（文件头 Rust 闭包「条目级宏调用生成的条目」）；调用体里已有的不重登
                        for name in sorted(self.macro_rules_literal_names.get(unit.short.partition("!@")[0], frozenset()) - set(unit.defines or ())):
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

    def index_macro_and_foreign_edges(self):
        """文件头 Rust 闭包「条目级宏调用生成的条目」「#[global_allocator]」「外部类型上的 trait impl」那三条要的索引，扫进了目标的每份文件一趟：
          macro_declared_types   条目级宏调用（id）→ 调用体里跟在 enum / struct / union / type 后面的名字（宏生成的类型）
          macro_rules_literal_names  宏名 → 同名 macro_rules! 体里展开式顶层跟在 const / static / fn / struct / enum / union / type 后面、按字面写出的名字
                                 （不是 $ 起头的捕获）
          global_allocators      带 #[global_allocator] 的 static 单元
          foreign_trait_impls    自身类型是外部类型、trait 是本仓 trait 的 trait impl（id）：自身类型写成路径、头一段不是 crate / self / super / Self 与本仓
                                 crate 名的，或自身类型里的名字没有一个是本仓定义的类型（struct、enum、union、type 与宏生成的类型）"""
        self.macro_declared_types, self.macro_rules_literal_names = {}, collections.defaultdict(set)
        self.global_allocators, self.foreign_trait_impls = [], set()
        paths = sorted({path for target in self.targets.values() for path in target.files})
        type_names, trait_names, impls = set(), set(), []
        for path in paths:
            rust_file = self.files[path]
            for unit in rust_file.units:
                if unit.first_token is None or unit.kind == "doctest":
                    continue
                span = range(unit.first_token, unit.last_token)
                if unit.kind == "item_macro":
                    declared = frozenset(rust_file.t(index + 1) for index in span if rust_file.t(index) in ("enum", "struct", "union", "type")
                                         and rust_file.kind(index + 1) == IDENT and rust_file.t(index + 1) not in RUST_KEYWORDS)
                    self.macro_declared_types[id(unit)] = declared
                    type_names |= declared
                elif unit.kind == "macro_rules":
                    depth = 0
                    for index in span:
                        word = rust_file.t(index)
                        depth += (word in ("{", "(", "[")) - (word in ("}", ")", "]"))
                        # 只收展开结果顶层的条目（宏体的括号一层、展开式的括号一层）：impl $name { fn new … } 里的关联项按类型连，不按名字登
                        if depth != 2 or word not in ("const", "static", "fn", "struct", "enum", "union", "type"):
                            continue
                        name_index = index + 2 if word == "static" and rust_file.t(index + 1) == "mut" else index + 1
                        name = rust_file.t(name_index)
                        if rust_file.kind(name_index) == IDENT and name not in RUST_KEYWORDS and name != "_":
                            self.macro_rules_literal_names[unit.short].add(name)
                elif unit.kind == "static" and any(rust_file.t(index) == "global_allocator" and rust_file.t(index - 1) == "[" and rust_file.t(index - 2) == "#"
                                                   for index in span):
                    self.global_allocators.append(unit)
                elif unit.kind in RUST_TYPE_KINDS:
                    type_names.add(unit.short)
                elif unit.kind == "trait":
                    trait_names.add(unit.short)
                elif unit.kind == "impl" and unit.is_trait_impl and not unit.blanket:
                    impls.append(unit)
        local_roots = {"crate", "self", "super", "Self"} | {crate.lib_name for crate in self.crates}
        for unit in impls:
            qualified_outside = unit.self_path_root is not None and unit.self_path_root not in local_roots
            if unit.trait_name in trait_names and (qualified_outside or not unit.self_types & type_names):
                self.foreign_trait_impls.add(id(unit))

    def names_reaching_outside_crate(self, crate):
        """别的包按裸名连得到工作区外这个 crate 的哪些名字（文件头 Rust 闭包「工作区外的 path 依赖」）→ (全收, 名字集)。扫的是这个 crate 以外每个目标的文件：
        根名是这个 crate 的库名，use / extern crate 语句里出现根名的，它后面的名字都算引进来、也都当根名（引进来的可能是模块或别名）；收到不再变；
        语句里根名后面有 * 的全收。路径 根名::a::b 里根名后面的段都算；名字后面接 ! 与括号、又是这个 crate 里 macro_rules! 的名字的也算。
        引进来的名字按整个工作区收、不分模块：经别处 pub use 转出去再 glob 引进来的照样认得。一个 crate 只算一趟。"""
        cached = self.outside_crate_reaching_names.get(crate.directory)
        if cached is not None:
            return cached
        own_paths = {path for target in self.targets.values() if target.crate is crate for path in target.files}
        paths = sorted({path for target in self.targets.values() if target.crate is not crate for path in target.files} - own_paths)
        macro_names = {unit.short for path in own_paths for unit in self.files[path].units if unit.kind == "macro_rules"}
        statements = []
        for path in paths:
            rust_file = self.files[path]
            texts, kinds = rust_file.texts, rust_file.kinds
            for index in (i for i, text in enumerate(texts) if text in ("use", "crate") and kinds[i] == IDENT):
                if texts[index] == "crate" and rust_file.t(index - 1) != "extern":
                    continue
                end = index + 1
                while end < rust_file.count and texts[end] != ";":
                    end += 1
                statements.append((rust_file, index + 1, end))
        roots, every_name = {crate.lib_name}, False
        grown = True
        while grown:
            grown = False
            for rust_file, low, high in statements:
                start = next((i for i in range(low, high) if rust_file.kinds[i] == IDENT and rust_file.texts[i] in roots), None)
                if start is None:
                    continue
                for i in range(start + 1, high):
                    text = rust_file.texts[i]
                    if text == "*":
                        every_name = True
                    elif rust_file.kinds[i] == IDENT and text not in ("self", "super", "crate", "as") and text not in roots:
                        roots.add(text)
                        grown = True
        names = set(roots)
        for path in paths:
            rust_file = self.files[path]
            texts, kinds = rust_file.texts, rust_file.kinds
            if not (roots | macro_names) & set(texts):
                continue
            for index, text in enumerate(texts):
                if kinds[index] != IDENT:
                    continue
                if text in macro_names and rust_file.t(index + 1) == "!" and rust_file.t(index + 2) in ("(", "[", "{"):
                    names.add(text)
                if text not in roots:
                    continue
                following = index + 1
                while rust_file.t(following) == ":" and rust_file.t(following + 1) == ":" and rust_file.kind(following + 2) == IDENT:
                    names.add(texts[following + 2])
                    following += 3
        self.outside_crate_reaching_names[crate.directory] = (every_name, frozenset(names))
        return self.outside_crate_reaching_names[crate.directory]

    def module_of(self, unit, identifier):
        """单元在目标 identifier 里所在的模块路径：文件在目标模块树里的路径接文件里的行内模块路径。"""
        return tuple(self.targets[identifier].files[unit.file][0]) + tuple(unit.scope or ())

    def candidates_seen_from_module(self, unit, identifier, name, candidates, other_crate_roots=frozenset()):
        """同一个目标里，单元按裸名 name 连得到 candidates 里的哪几份（文件头 Rust 闭包「R-裸名同一个目标里按模块筛」；crates/singlefs-unit-graph 的
        candidates_seen_from_the_module_of_the_unit 同一个判法）：与单元同一模块的；所在模块的 use 引进了 name（或是 glob）、use 路径里有候选所在模块的末段
        （crate::x 是根模块、super 是上一层、super::super 是上两层，self 起头是本模块）的；单元里写成 前缀::name、前缀是候选所在模块末段（crate、self 照算；
        前缀是 super 时按整条路径里 super 的个数往上数层）的。use 路径或 …::name 整条路径的根名在 other_crate_roots 里的（std、core、alloc 与看得见的
        别的 crate 的库名）指的是别的 crate，跳过这一处。认不出落在哪一份的退回全连：use 或路径点到的模块里一份都没有（经 pub use 转出来的）、
        前缀不是类型也不是哪一份所在模块、单元里另有 use 语句（函数体里的 use）、name 后面接 !。
        前缀是类型或 trait（Self、Tr::x）的是关联项，R-路径另管，不按这一处连。哪一样都不沾的（字段名、局部变量）一份都不连。"""
        key = (id(unit), identifier, name, other_crate_roots)
        cached = self.outside_crate_module_candidates.get(key)
        if cached is not None:
            return cached
        own = self.module_of(unit, identifier)
        modules = {id(candidate): self.module_of(candidate, identifier) for candidate in candidates}
        seen = {id(candidate) for candidate in candidates if modules[id(candidate)] == own}
        every = False

        def placed(segments):
            """路径（不含 name 本身）点到的候选；一份都没点到交 None。路径里写了几个 super 就往上几层（超出根模块的按根模块算）。"""
            found = set()
            levels_up = segments.count("super")
            for candidate in candidates:
                module = modules[id(candidate)]
                if ((module and module[-1] in segments) or (not module and segments[:1] == ["crate"])
                        or (levels_up and module == own[:max(len(own) - levels_up, 0)]) or (segments[:1] == ["self"] and module == own)):
                    found.add(id(candidate))
            return found or None

        rust_file = self.files[unit.file]
        for use in self.uses_by_scope.get((unit.file, unit.scope), ()):
            if use.first_token is None or not (use.import_glob or name in use.imports):
                continue
            if use_path_root(rust_file, use.first_token, use.last_token) in other_crate_roots:
                continue
            segments = [rust_file.texts[index] for index in range(use.first_token, use.last_token)
                        if rust_file.kinds[index] == IDENT and rust_file.texts[index] not in ("use", "pub", "as", name)]
            hit = placed(segments)
            if hit is None:
                every = True
            else:
                seen |= hit
        if unit.first_token is not None and not every:
            if identifier not in self.outside_crate_type_names:
                self.outside_crate_type_names[identifier] = frozenset(
                    short for short, found in self.by_name.get(identifier, {}).items() if any(other.kind in RUST_TYPE_OR_TRAIT_KINDS for other in found))
            types = self.outside_crate_type_names[identifier]
            for index in range(unit.first_token, unit.last_token):
                text = rust_file.texts[index]
                if text == "use" and rust_file.kinds[index] == IDENT and index != unit.first_token:
                    every = True
                    break
                if text != name or rust_file.kinds[index] != IDENT:
                    continue
                if rust_file.t(index + 1) == "!" and rust_file.t(index + 2) in ("(", "[", "{"):
                    every = True
                    break
                if rust_file.t(index - 1) == ":" and rust_file.t(index - 2) == ":" and rust_file.kind(index - 3) == IDENT:
                    prefix = rust_file.t(index - 3)
                    # 整条路径 根::…::前缀::name，从前缀往前顺着「标识符 ::」收到根
                    path_before, position = [prefix], index - 3
                    while position >= 3 and rust_file.t(position - 1) == ":" and rust_file.t(position - 2) == ":" and rust_file.kind(position - 3) == IDENT:
                        position -= 3
                        path_before.append(rust_file.t(position))
                    path_before.reverse()
                    if path_before[0] in other_crate_roots:
                        continue
                    if prefix == "Self" or (prefix in types and prefix not in ("crate", "self", "super")):
                        continue
                    hit = placed(path_before if prefix == "super" else [prefix])
                    if hit is None:
                        every = True
                        break
                    seen |= hit
        found = list(candidates) if every else [candidate for candidate in candidates if id(candidate) in seen]
        self.outside_crate_module_candidates[key] = found
        return found

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

    def test_functions(self, target_id):
        """编译目标里带 #[test] 的 fn 单元：[(libtest 全名, 单元)]，全名 = 文件在目标里的模块路径加单元的限定名。"""
        cache = self.__dict__.setdefault("_test_functions", {})
        if target_id not in cache:
            found = []
            for path, (module_path, _chain) in sorted(self.targets[target_id].files.items()):
                for candidate in self.files[path].units:
                    if candidate.kind == "fn" and any(condition[1] == TEST_ATTRIBUTE_RAW for condition in candidate.conditions or ()):
                        found.append(("::".join(list(module_path) + [candidate.name]), candidate))
            cache[target_id] = found
        return cache[target_id]

    def exact_test(self, full_name, target_id):
        """同一编译目标里 libtest 全名恰好是 full_name 的 #[test] 单元；不是恰好一个回 None（拒算 current-exe）。"""
        hits = [candidate for name, candidate in self.test_functions(target_id) if name == full_name]
        return hits[0] if len(hits) == 1 else None

    def resolve_features(self, linked_crates, entry_crate, requested, no_default, entry_sections):
        """照 cargo 的特性统一（resolver 2，与 crates/singlefs-unit-graph 的 resolve_features 同一口径）：依赖边上请求的特性，
        入口那个 crate 只顺 entry_sections 那几段（rust_entry_dependency_sections），别的 crate 只顺普通依赖——它们的 dev 依赖不编，
        dev 依赖里才开的特性不进 bin 与库目标。弄坏开关 rust-dependency-features-from-every-section 下退回旧口径：每个 crate 每一段都顺。"""
        every_section = "rust-dependency-features-from-every-section" in active_breaks()
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
            sections = tuple(crate.dependencies) if every_section else entry_sections if crate is entry_crate else ("normal",)
            for section in sections:
                for dependency in crate.dependencies[section]:
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


def doctest_entry_paths(library, doctest):
    """能点到这个文档测试的入口路径（不带 crate 名的段，元组），照 crates/singlefs-unit-graph 的 resolve_entry_units 文档测试那一支与 matches_entry_path 认的写法：
    挂模块的是那个模块在 lib 模块树里的路径；挂条目的是条目的模块路径接条目名，条目是 impl / trait 里的方法时是模块路径、行内模块、类型名、方法名，
    类型名取 impl 的各个 self 类型与头自己的名字。头一项是门禁 code-tooling 的 function-units-two-implementations-agree 格拼 unit-graph closure 时取的那一种
    （impl 取排序后头一个 self 类型）。所在文件不在 lib 目标模块树里的交空表（Rust 那一份只认 lib 里的文档测试）。"""
    placement = library.files.get(doctest.file)
    if placement is None:
        return []
    module_path = list(placement[0])
    owner = doctest.container
    if owner is None:
        return [tuple(module_path + list(doctest.doc_module or ()))]
    if owner.container is None:
        return [tuple(module_path + owner.name.split("::"))]
    head = owner.container
    named = sorted(head.self_types)[0] if head.kind == "impl" and head.self_types else head.short
    others = sorted((set(head.self_types or ()) | {head.short}) - {named})
    return [tuple(module_path + list(owner.scope or ()) + [type_name, owner.short]) for type_name in [named] + others]


def doctest_entry_group(workspace, entry):
    """文档测试入口一起当入口的那一组文档测试单元（rust_closure_of 的入口与 Closure.entry_units）：照 crates/singlefs-unit-graph 的 resolve_entry_units，
    文档测试目标的入口是所挂条目或模块的路径，解到那个路径点到的条目与模块挂的全部文档测试；这里拿 entry 的头一条入口路径（doctest_entry_paths）
    在 lib 目标模块树的每份文件里找能被它点到的文档测试，按 Unit.sort_key 排。lib 目标没有、entry 不在里面的，只交 entry 自己。"""
    crate = workspace.crate_of_path(entry.file)
    library = workspace.lib_target(crate) if crate is not None else None
    paths = doctest_entry_paths(library, entry) if library is not None else []
    if not paths:
        return [entry]
    group = [unit for path in sorted(library.files) for unit in workspace.file(path).units
             if unit.kind == "doctest" and paths[0] in doctest_entry_paths(library, unit)]
    if not any(unit is entry for unit in group):
        return [entry]
    return sorted(group, key=Unit.sort_key)


def rust_unit_is_test(workspace, unit, target_id):
    if workspace.targets[target_id].kind in ("test", "bench"):
        return True
    return rust_unit_is_test_code(workspace, unit, target_id)


def rust_unit_is_test_code(workspace, unit, target_id):
    """单元自己带 #[test]、或它（连同外层的 impl、行内模块与把它所在文件连进来的 mod 声明）带 cfg(test)：只在按测试编时才有的代码。
    不含整个目标都是测试的目标种类（tests/ 下的集成测试里没有 #[test] 的单元不算）。"""
    chain = workspace.targets[target_id].files.get(unit.file, ((), []))[1]
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
        # 键值写 键="值"）、读数据文件或数据目录的每一处（文件、行、数据文件或目录、摘要、读它的单元、种类 data 或 data-directory）、每个单元收进来时所在的目标
        self.entry_target_kind, self.entry_cfg_atoms = None, []
        # 键里的 cfg 成分（「<原子>\t<true|false>」的集合，rust_closure_of 填）；None 是没记（key_cfg_items 退回整套生效 cfg）
        self.cfg_atoms = None
        self.data_sites = set()
        self.unit_targets = {}
        # 从哪几个入口收的（键里的 entry 行，entry_key_lines）：rust_closure_of、ScriptClosure.run 与 WGSL 点名的入口各记；别处自己拼的闭包不记，不出那几行
        self.entry_units = []
        # 工具链询问（文件头「版本询问」）：{(文件, 行)}；每一处另有一个工具链单元在 order 里（ScriptClosure.toolchain_unit）
        self.toolchain_queries = set()

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
    """单元按名字定义的名字（crates/singlefs-unit-graph 的 declared_names）：条目级宏调用与 extern 块是它里面定义的那几个，
    assume_specification 是它点名路径末段的名字，别的是它自己的名字。"""
    if unit.kind in RUST_KINDS_DEFINING_NAMES:
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


def rust_entry_dependency_sections(target_kind, test_compiled):
    """入口目标顺自己 crate 的哪几段依赖解特性（crates/singlefs-unit-graph 的 dependency_sections_of）：构建脚本只顺 build 依赖；
    按 cfg(test) 编的 lib 与 bin、集成测试、example、bench、文档测试连 dev 依赖；不按 cfg(test) 编的 lib 与 bin 只顺普通依赖。"""
    if target_kind == "build":
        return ("build",)
    if test_compiled or target_kind in ("test", "bench", "example", "doctest"):
        return ("normal", "dev")
    return ("normal",)


def entry_names_only_defined(workspace, entry):
    """入口单元自己的名字在它的词法单元里只出现在定义那一处：交 {名字}，不然交空集（文件头 Rust 闭包「入口名字的定义那一处」）。"""
    if entry.first_token is None or entry.last_token is None or not entry.short:
        return frozenset()
    rust_file = workspace.files[entry.file]
    occurrences = sum(1 for index in range(entry.first_token, entry.last_token)
                      if rust_file.kind(index) == IDENT and rust_file.t(index) == entry.short)
    return frozenset((entry.short,)) if occurrences <= 1 else frozenset()


def rust_closure_of(workspace, entry, target_id, cfg=None, no_default_cfg=False, features=(), no_default_features=False,
                    entry_definition_name_unresolved=False, stop_at=()):
    """从入口收 Rust 闭包（3.3 八条规则），连同入口闭包拼出的 WGSL 模块（3.5）。entry_definition_name_unresolved 见文件头 Rust 闭包「入口名字的定义那一处」，
    stop_at 见同一节「到这些单元就停」。"""
    breaks = active_breaks()
    closure = Closure()
    stopped_units = {id(unit) for source, unit in stop_at if source is None}
    stopped_edges = {(id(source), id(unit)) for source, unit in stop_at if source is not None}
    # 文档测试入口照 crates/singlefs-unit-graph 的 resolve_entry_units：所挂条目或模块的全部文档测试一起当入口、一起记进 entry 行（doctest_entry_group）；
    # 弄坏开关 rust-doctest-entry-alone：只取交进来的那一个（Rust 那一份记一组、这里记一个，两边的 entry 行与闭包对不上）
    roots = doctest_entry_group(workspace, entry) if entry.kind == "doctest" and "rust-doctest-entry-alone" not in breaks else [entry]
    closure.entry_units.extend(roots)
    entry_unresolved_names = entry_names_only_defined(workspace, entry) if entry_definition_name_unresolved else frozenset()
    target = workspace.targets[target_id]
    test_compiled = rust_unit_is_test(workspace, entry, target_id) if target.kind != "doctest" else False
    linked = set(workspace.visible(target_id, development=test_compiled))
    linked_crates = {workspace.targets[identifier].crate for identifier in linked}
    requested = []
    for item in features:
        package, _slash, feature = item.partition("/")
        requested.append((workspace.crate_by_package(package), feature))
    enabled = workspace.resolve_features(linked_crates, target.crate, requested, no_default_features,
                                         rust_entry_dependency_sections(target.kind, test_compiled))
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
    # 键里的 cfg 成分：闭包里各单元真引到的原子，各带收它进来那个目标下的真值（record_cfg_atoms 逐个记）
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
    # blanket impl（impl<T: …> Tr for T）够得到的方法算边（python 与 crates/singlefs-unit-graph 两份都这样算）；
    # 弄坏开关 rust-blanket-impls-ignored 下 R-方法、R-路径不放行 blanket impl，trait 头进闭包也不带它的 blanket impl（Rust 现在的写法）
    blanket = "rust-blanket-impls-ignored" not in breaks
    # 文件头 Rust 闭包「条目级宏调用生成的条目」「#[global_allocator]」「外部类型上的 trait impl」：各一个弄坏开关去掉这些连边
    macro_types_named = "rust-macro-generated-types-unnamed" not in breaks
    allocators_linked = "rust-global-allocators-unlinked" not in breaks
    foreign_impls = workspace.foreign_trait_impls if "rust-foreign-self-trait-impls-unlinked" not in breaks else frozenset()
    # 文件头 Rust 闭包「工作区外的 path 依赖」：别的包按裸名连进来要经 use 或路径；同一个目标里只连所在模块看得见的那几份。各一个弄坏开关去掉这些限制
    outside_names_imported_only = "rust-outside-crate-names-linked-unimported" not in breaks
    outside_names_by_module = "rust-outside-crate-names-linked-across-modules" not in breaks
    # 文件头 Rust 闭包「R-裸名同一个目标里按模块筛」：crates/ 下的包同一个目标里同样只连所在模块看得见的那几份
    crates_names_by_module = "rust-crates-names-linked-across-modules" not in breaks
    doctest_structure = "rust-doctest-structure-linked" in breaks
    body_cfg_atoms = "rust-cfg-macro-atoms-ignored" not in breaks
    raw_wgsl = "wgsl-raw-text" in breaks
    pull_suppliers = not ({"wgsl-name-suppliers-not-pulled-in", "wgsl-external-names-by-long-identifiers"} & breaks)
    pull_wgsl_definers = "wgsl-other-file-definitions-unlinked" not in breaks
    suppliers_skip_test_code = "wgsl-name-suppliers-include-test-code" not in breaks
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
    pending_children = []        # 起子进程的测试名是参数的：(单元, 函数名, 位置, 目标, current_exe 的词法单元下标)
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

    def collect_shell_sources(path, reader, line):
        """读到的数据文件是 shell 脚本：它（传递地）source / . 进来的文件各收成一个 data-file 单元，折不出的拒算（文件头「Rust 闭包读到的 shell 脚本」）；
        每读一处这份脚本（reader 那个单元的第 line 行），被 source 的每一份在那一处记一条 data_sites（crates/singlefs-unit-graph 的 follow_shell_sources 同）。
        弄坏开关 shell-data-sources-unfollowed：一份都不收；shell-data-sources-unsited：照收、不记 data_sites。"""
        if "shell-data-sources-unfollowed" in breaks or language_of_file(workspace.root, path) != "shell":
            return
        sourced, refusals = workspace.shell_sourced(path)
        closure.refusals.update(refusals)
        for file in sourced:
            if file not in data_units:
                data_units[file] = data_file_unit(workspace.root, file)
            closure.add(data_units[file], data_units[path], "shell-source")
            if "shell-data-sources-unsited" not in breaks:
                closure.data_sites.add((reader.file, line, file, data_units[file].fingerprint, id(reader), "data"))

    def collect_data(unit, identifier):
        """这个单元读的数据文件进闭包（一份文件一个 data-file 单元），另按读它的那一处记一条 data_sites（Rust 那一份的数据单元写法，unit_graph_key 用）；
        OUT_DIR 与 CARGO_BIN_EXE_<名> 连到构建脚本与那个 bin 的 main。"""
        crate = workspace.targets[identifier].crate
        for kind, value, index in unit.data_references or ():
            if kind == "file":
                if value not in data_units:
                    data_units[value] = data_file_unit(workspace.root, value)
                line = workspace.files[unit.file].lines[index]
                collect_shell_sources(value, unit, line)   # 每一处读都记：被 source 的那几份的数据位置落在读这份脚本的那一处
                closure.add(data_units[value], unit, "data")
                closure.data_sites.add((unit.file, line, value, data_units[value].fingerprint, id(unit), "data"))
            elif kind == "current-exe-child":
                if value[0] == "parameter":
                    pending_children.append((unit, value[1], value[2], identifier, index))
                else:
                    name = folded_test_name(unit, value[1])
                    test = workspace.exact_test(name, identifier) if name is not None else None
                    if test is None:
                        refuse_child(unit, index)
                    elif "rust-current-exe-children-unlinked" not in breaks:
                        push_exact(test, identifier, unit, "current-exe-child")
            elif kind == "directory":
                if value not in data_units:
                    data_units[value] = data_directory_unit(workspace.root, value)
                closure.add(data_units[value], unit, "data")
                closure.data_sites.add((unit.file, workspace.files[unit.file].lines[index], value, data_units[value].fingerprint, id(unit), "data-directory"))
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
            elif kind == "build-environment":
                # env!("X") 的 X 不是 cargo 给的：连到本 crate 构建脚本的 main（它经 cargo::rustc-env 交这个值）；没有构建脚本的不连
                # （值来自编译时的进程环境，不在仓里的代码里）。弄坏开关 rust-build-environment-unlinked：不连
                if "rust-build-environment-unlinked" in breaks:
                    continue
                build = workspace.targets.get(f"{crate.label}:build:{crate.label}")
                for main in ([candidate for candidate in workspace.files[build.root_file].units if candidate.kind == "fn" and candidate.short == "main"
                              and candidate.container is None] if build else []):
                    push_exact(main, build.id, unit, "build-environment")
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
    for root in roots:
        todo.append((root, target_id, None, "entry"))
        for trait_name in entry_signature_traits(root) if not no_open else ():
            dyn_trait(trait_name, target_id, root)
    for allocator in workspace.global_allocators if allocators_linked else ():
        # #[global_allocator] 的 static 管整个链接出来的程序：链接集里有它所在目标的，它进闭包（连同它的类型、R-隐式带上 GlobalAlloc 的 impl）
        for owner in sorted(workspace.unit_targets.get(id(allocator), ())):
            if owner in linked:
                push_exact(allocator, owner, None, "global-allocator")

    def drain():
        while todo:
            unit, identifier, via, rule = todo.pop()
            if (id(unit), identifier) in seen:
                continue
            if unit is not entry and (id(unit) in stopped_units or (via is not None and (id(via), id(unit)) in stopped_edges)):
                # 到这些单元就停（文件头 Rust 闭包那一节）：按边停的不记进 seen，别的单元连到它照连
                continue
            seen.add((id(unit), identifier))
            if unit.kind in dropped or not active(unit, identifier):
                continue
            if unit.ghost_only and unit is not entry:
                # 产品行为视图里只给 Verus 看的单元（文件头「判定种类」）：不进闭包、不往下连
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
                references = (unit.references or set()) - (entry_unresolved_names if unit is entry else frozenset())
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
            unit_crate = workspace.targets[identifier].crate
            # macro_rules! 体里的裸名在调用处解，定义所在模块看不出它落在哪一份：一律全连
            resolved_at_call_site = unit.kind == "macro_rules"
            # 路径根名是这几个的指别的 crate（标准库与看得见的别的 crate 的库名），不点本目标里的哪一份（candidates_seen_from_module）
            other_crate_roots = frozenset(STANDARD_CRATE_PATH_ROOTS) | {
                workspace.targets[visible_id].crate.lib_name for visible_id in visible if workspace.targets[visible_id].crate is not unit_crate}
            for name in sorted(unit.plain) + alias_originals:
                if unit is entry and name in entry_unresolved_names:
                    continue
                for visible_id in visible:
                    table = workspace.by_name.get(visible_id)
                    if table is None:
                        continue
                    candidates = table.get(name, ())
                    visible_crate = workspace.targets[visible_id].crate
                    if candidates and visible_crate.outside_workspace and outside_names_imported_only and visible_crate is not unit_crate:
                        # 工作区外的 crate 按调用关系连（文件头 Rust 闭包「工作区外的 path 依赖」）：从别的包连进来，这个名字要在工作区里经 use 或路径
                        # 从它引进来过（验证包的 run 字段不连到 prune-server 的 command_line::run）
                        every_name, reaching = workspace.names_reaching_outside_crate(visible_crate)
                        if not every_name and name not in reaching:
                            continue
                    by_module = outside_names_by_module if visible_crate.outside_workspace else crates_names_by_module
                    if candidates and by_module and visible_id == identifier and name not in alias_originals and not resolved_at_call_site:
                        # 同一个目标里只连这个单元所在模块看得见的那几份（文件头 Rust 闭包「R-裸名同一个目标里按模块筛」：crates/ 下的包与工作区外的 crate 一样；
                        # 验证包里 CPU 判器的单元经 use 从格式库引进 SLOT_BYTES，不连到 gpu_unit_checks 里私有的同名常量）
                        candidates = workspace.candidates_seen_from_module(unit, identifier, name, candidates, other_crate_roots)
                    for candidate in candidates:
                        if candidate is unit:
                            continue
                        if visible_id != identifier and not candidate.public:
                            continue
                        push_exact(candidate, visible_id, unit, f"name {name}")
            for name in unit.methods:
                methods_seen.add(name)
                for member in workspace.members_by_name.get(name, ()):
                    container = member.container
                    if container.kind == "trait" or (blanket and container.blanket) or (container.self_types & named) or id(container) in foreign_impls:
                        push(member, identifier, unit, f"method {name}")
            own = unit.container
            for qualifier, name in unit.qualified:
                for member in workspace.members_by_name.get(name, ()):
                    container = member.container
                    if container.kind == "trait" and container.short == qualifier:
                        push(member, identifier, unit, f"path {qualifier}::{name}")
                    elif (trait_self_paths and container.kind == "trait" and qualifier == "Self" and own is not None and own.kind == "trait"
                          and own.short == container.short):
                        # trait 成员里的 Self::x 落到这个 trait 自己的成员 x（crates/singlefs-unit-graph 的 follow_qualified_paths 同）
                        push(member, identifier, unit, f"path Self::{name}")
                    elif container.kind == "impl" and qualifier in container.self_types:
                        push(member, identifier, unit, f"path {qualifier}::{name}")
                    elif container.kind == "impl" and qualifier == "Self" and own is not None and (
                            container.self_types & own.self_types if trait_self_paths else container.self_types & (own.self_types or {own.short})):
                        push(member, identifier, unit, f"path Self::{name}")
                    elif container.kind == "impl" and container.is_trait_impl and container.trait_name == qualifier and (
                            (blanket and container.blanket) or container.self_types & named or id(container) in foreign_impls):
                        push(member, identifier, unit, f"path {qualifier}::{name}")
            if unit.kind in RUST_TYPE_KINDS:
                name_type(unit.short, identifier, unit)
            if unit.kind == "item_macro" and macro_types_named:
                # 条目级宏调用生成的类型算被点名：R-方法、R-隐式照连到写给它的 impl（文件头 Rust 闭包「条目级宏调用生成的条目」）
                for type_name in sorted(workspace.macro_declared_types.get(id(unit), ())):
                    name_type(type_name, identifier, unit)
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

    def refuse_child(unit, index):
        rust_file = workspace.files[unit.file]
        line = rust_file.lines[index]
        closure.refusals.add(Refusal("current-exe", unit.file, line, rust_file.row(line)))

    def constant_string(context, name):
        """常量 name 的串值（crates/singlefs-unit-graph 的 constant_string_value）：先找 context 这个单元自己的词法单元里的 const <名>: … = "…";，
        再找同一个包、在这一次链接集里（同一包的别的 bin 不算）叫这个名字的常量单元（几个都得写成 = "…";、值相同）；找不到、写法不是串字面量或几个值不同回 None。
        弄坏开关 rust-current-exe-constants-same-file-only：第二步只找 context 所在的那一份文件。"""
        found = workspace.files[context.file].string_constant_in(context.first_token, context.last_token, name)
        if found is not None:
            return found
        context_crate = workspace.crate_of_path(context.file)
        values, searched = set(), set()
        for candidate_target in sorted(linked):
            if workspace.targets[candidate_target].kind == "doctest":
                continue
            for path in sorted(workspace.targets[candidate_target].files):
                if path in searched or workspace.crate_of_path(path) is not context_crate:
                    continue
                searched.add(path)
                if path != context.file and "rust-current-exe-constants-same-file-only" in breaks:
                    continue
                rust_file = workspace.files[path]
                for candidate in rust_file.units:
                    if candidate.kind == "const" and candidate.short == name:
                        value = rust_file.string_constant_in(candidate.first_token, candidate.last_token, name)
                        if value is None:
                            return None
                        values.add(value)
        return next(iter(values)) if len(values) == 1 else None

    def folded_test_name(context, pieces):
        """几段连成测试名：常量名在 context 这个单元里解（constant_string）；有一个解不出回 None。"""
        texts = []
        for piece_kind, text in pieces:
            if piece_kind == "literal":
                texts.append(text)
                continue
            value = constant_string(context, text)
            if value is None:
                return None
            texts.append(value)
        return "".join(texts)

    def child_tests_of(function_name, position, identifier):
        """名字是起子进程那个函数的参数：闭包里每一处调用它的那个实参各折一遍 → (对上的 #[test] 单元, 全都折得出又各对上恰好一个)；一处调用都没有也算不行。"""
        names, complete = set(), True
        for member in list(closure.order):
            if member.language != "rust" or member.first_token is None or member.kind == "doctest":
                continue
            found = workspace.files[member.file].call_site_test_names(member, function_name, position)
            if found is None:
                complete = False
                continue
            for pieces in found:
                name = folded_test_name(member, pieces)
                if name is None:
                    complete = False
                else:
                    names.add(name)
        tests = []
        for name in sorted(names):
            test = workspace.exact_test(name, identifier)
            if test is None:
                complete = False
            else:
                tests.append(test)
        return tests, complete and bool(names)

    def pull_in_child_process_tests():
        """参数写法的测试名（规则 current-exe-child）：闭包里每一处调用点交的实参对上的 #[test] 单元连进来；每趟 drain 之后、WGSL 供给之前跑。→ 有没有新推进去的。"""
        if "rust-current-exe-children-unlinked" in breaks:
            return False
        pushed = False
        for _unit, function_name, position, identifier, _index in pending_children:
            for test in child_tests_of(function_name, position, identifier)[0]:
                if (id(test), identifier) not in seen:
                    push_exact(test, identifier, _unit, "current-exe-child")
                    pushed = True
        return pushed

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
                    if suppliers_skip_test_code and rust_unit_is_test_code(workspace, candidate, identifier):
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
    while True:
        if pull_in_child_process_tests() or pull_in_wgsl_name_suppliers():
            drain()
            continue
        break
    for unit, function_name, position, identifier, index in pending_children:
        if not child_tests_of(function_name, position, identifier)[1]:
            refuse_child(unit, index)
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


# 产品行为视图里只给 Verus 看的那几截（crates/singlefs-unit-graph 的 verus_ghost_code.rs 文件头列的那几种；那一份按 verus_syn 的语法树挑，
# 这里按词法单元认同样的写法，认出来的要逐单元核得上 Rust 那一份才用：RustWorkspace.product_view_file）
VERUS_GHOST_FUNCTION_MODES = frozenset(("spec", "proof", "axiom"))
VERUS_SIGNATURE_CLAUSES = frozenset(("requires", "recommends", "ensures", "default_ensures", "returns", "decreases", "opens_invariants",
                                     "no_unwind", "with", "by", "atomically"))
VERUS_LOOP_CLAUSES = frozenset(("invariant", "invariant_except_break", "invariant_ensures", "ensures", "decreases"))
VERUS_GHOST_CALLS = frozenset(("assume", "reveal", "reveal_with_fuel", "hide"))
VERUS_GHOST_WRAPPERS = frozenset(("Ghost", "Tracked"))
# verus! 里同层条目的起头词（crates/singlefs-unit-graph 的 is_verus_item_start）
VERUS_ITEM_START_WORDS = frozenset(("pub", "fn", "spec", "proof", "exec", "open", "closed", "broadcast", "struct", "enum", "union", "impl",
                                    "trait", "const", "static", "type", "use", "mod", "unsafe", "extern", "macro_rules", "global", "tracked",
                                    "ghost", "uninterp", "axiom", "async", "default"))
# 前一个词是这些时后面的 ( 不是调用的实参表
VERUS_NOT_CALLEES = frozenset(("if", "while", "match", "return", "in", "for", "let", "loop", "else", "move", "mut", "ref", "as", "fn", "impl",
                               "where", "dyn", "break", "continue", "unsafe", "await", "yield", "box"))
# 一对 { … } 之后接这些时，它还在循环头的子句里（不是循环体）
VERUS_CLAUSE_CONTINUATIONS = frozenset((",", "{", ".", "?", "=", "!", "<", ">", "&", "|", "+", "-", "*", "/", "%", "^", "@", "as", "implies",
                                        "is", "has", "else")) | VERUS_LOOP_CLAUSES


class VerusGhostCode:
    """一份 Rust 文件里每一块 verus! { … } 中只给 Verus 看的那几截（照 crates/singlefs-unit-graph 的 GhostCodeCollector 挑）：
    spec、proof、axiom 函数与常量整条（连属性与文档注释）、broadcast group、broadcast use、assume_specification；执行函数签名里的规格子句
    （requires 一族到函数体之前）、具名返回值的「(名:」与「)」、类型是 Ghost<…> / Tracked<…> 的参数；函数体与常量初值里的 proof { … }、assert(…)（连 by …）、
    assert forall、assume、reveal 一族、let ghost / let tracked、循环头的 invariant 一族与 ensures、decreases、for 循环给迭代器起的名；
    ghost / tracked 字段与类型是 Ghost<…> / Tracked<…> 的字段，调用、方法调用的实参与结构体字面量、结构体模式里整个是 Ghost(…) / Tracked(…) 的那几样
    （一列里剥几样时连逗号，照那一份 blank_the_chosen_elements_of_a_list）。宏调用里面不看（verus_syn 不解它）。
    collect() → [(首词法单元, 末词法单元之后, 连不连紧挨在前面的文档注释)]。"""

    def __init__(self, kinds, texts, pairs):
        self.kinds, self.texts, self.pairs = kinds, texts, pairs
        self.count = len(texts)
        self.openings = {close: opening for opening, close in pairs.items()}
        self.ranges = []

    def t(self, index):
        return self.texts[index] if 0 <= index < self.count else ""

    def ident(self, index):
        return 0 <= index < self.count and self.kinds[index] == IDENT

    def group(self, index):
        return self.t(index) in ("(", "[", "{") and index in self.pairs

    def blank(self, first, end, with_documents=False):
        if first < end:
            self.ranges.append((first, end, with_documents))

    def collect(self):
        index = 0
        while index + 2 < self.count:
            if self.ident(index) and self.t(index) == "verus" and self.t(index + 1) == "!" and self.t(index + 2) == "{" and (index + 2) in self.pairs:
                close = self.pairs[index + 2]
                self.items(index + 3, close)
                index = close + 1
                continue
            index += 1
        return self.ranges

    # 条目

    def is_item_start(self, index):
        text = self.t(index)
        if text == "#":
            return self.t(index + 1) == "["
        if not self.ident(index):
            return False
        if text in VERUS_ITEM_START_WORDS:
            return True
        if text == "assume_specification":
            return self.t(index + 1) in ("[", "<")
        return self.t(index + 1) == "!" and self.group(index + 2)

    def items(self, low, high):
        """low..high 里同层的条目：块首，或顶层 ; / } 之后接条目起头词或属性（crates/singlefs-unit-graph 的 verus_item_starts）。"""
        starts, boundary, index = [], True, low
        while index < high:
            if boundary and self.is_item_start(index):
                starts.append(index)
            if self.group(index):
                close = self.pairs[index]
                boundary = self.t(close) == "}"
                index = close + 1
                continue
            boundary = self.t(index) == ";"
            index += 1
        for position, start in enumerate(starts):
            self.item(start, starts[position + 1] if position + 1 < len(starts) else high)

    def skip_attributes(self, index, end):
        while index < end and self.t(index) == "#" and self.t(index + 1) == "[" and (index + 1) in self.pairs:
            index = self.pairs[index + 1] + 1
        return index

    def skip_visibility(self, index):
        if self.t(index) == "pub":
            index += 1
            if self.t(index) == "(" and index in self.pairs:
                index = self.pairs[index] + 1
        return index

    def skip_qualifiers(self, index):
        """→ (条目关键字的下标, 跳过的修饰词)；照 crates/singlefs-unit-graph 的 skip_qualifiers（verus 为真）。"""
        skipped = set()
        while True:
            word, following = self.t(index), self.t(index + 1)
            if word == "const":
                qualifier = following in ("fn", "unsafe", "async", "extern")
            elif word == "async":
                qualifier = following in ("fn", "unsafe")
            elif word == "unsafe":
                qualifier = following in ("fn", "impl", "trait", "extern", "auto")
            elif word == "extern":
                qualifier = following == "fn" or (0 <= index + 1 < self.count and self.kinds[index + 1] == LITERAL and self.t(index + 2) == "fn")
                if qualifier and following != "fn":
                    index += 1
            elif word == "default":
                qualifier = following in ("fn", "type", "const", "unsafe", "async", "impl")
            elif word == "safe":
                qualifier = following in ("fn", "static")
            elif word == "auto":
                qualifier = following == "trait"
            elif word in ("open", "closed", "spec", "proof", "exec", "tracked", "ghost", "uninterp", "axiom"):
                qualifier = True
            elif word == "broadcast":
                qualifier = following in ("proof", "fn", "spec", "axiom", "uninterp", "open", "closed")
            else:
                qualifier = False
            if not qualifier:
                return index, skipped
            skipped.add(word)
            index += 1

    def item(self, start, end):
        header = self.skip_attributes(start, end)
        keyword, modes = self.skip_qualifiers(self.skip_visibility(header))
        word = self.t(keyword)
        ghost = bool(modes & VERUS_GHOST_FUNCTION_MODES)
        if word in ("fn", "const", "static") and ghost:
            self.blank(start, end, True)
        elif word == "fn":
            self.function(keyword, end)
        elif word in ("const", "static"):
            self.expressions(keyword, end)
        elif (word == "broadcast" and self.t(keyword + 1) in ("use", "group")) or (word == "assume_specification" and self.t(keyword + 1) in ("[", "<")):
            self.blank(start, end, True)
        elif word in ("struct", "union", "enum") and self.ident(keyword + 1):
            self.data_type(word, keyword, end)
        elif word in ("impl", "trait"):
            body = self.first_brace(keyword + 1, end)
            if body is not None:
                self.items(body + 1, self.pairs[body])
        elif word == "mod" and self.t(keyword + 2) == "{" and (keyword + 2) in self.pairs:
            self.items(keyword + 3, self.pairs[keyword + 2])

    def first_brace(self, low, high):
        """low 起顶层第一个 {（先遇到 ; 交 None）。"""
        index = low
        while index < high:
            text = self.t(index)
            if text == "{" and index in self.pairs:
                return index
            if text == ";":
                return None
            index = self.pairs[index] + 1 if self.group(index) else index + 1
        return None

    def after_angle_brackets(self, opening, end):
        """opening 那个 < 配到它的 >，交 > 之后的下标（-> 与 => 的 > 不算）；配不上交 None。"""
        depth = 0
        for position in range(opening, end):
            text = self.t(position)
            if text == "<":
                depth += 1
            elif text == ">" and self.t(position - 1) not in ("-", "="):
                depth -= 1
                if depth == 0:
                    return position + 1
                if depth < 0:
                    return None
        return None

    def single_colon(self, low, high):
        """low..high 顶层第一个单独的 :（不是 :: 的一半）。"""
        index = low
        while index < high:
            if self.group(index):
                index = self.pairs[index] + 1
                continue
            if self.t(index) == ":" and self.t(index - 1) != ":" and self.t(index + 1) != ":":
                return index
            index += 1
        return None

    def first_word(self, low, high, words):
        index = low
        while index < high:
            if self.group(index):
                index = self.pairs[index] + 1
                continue
            if self.ident(index) and self.t(index) in words and self.t(index - 1) not in (".", ":"):
                return index
            index += 1
        return None

    def elements(self, low, high, types):
        """low..high 按顶层逗号分开的几样 → [(起, 止, 后面那个逗号的下标或 None)]；types 为真时尖括号里的逗号不算（类型里的 Map<K, V>），
        否则只有 ::< 起的尖括号（turbofish）里的不算。"""
        parts, start, index, angles = [], low, low, 0
        while index < high:
            text = self.t(index)
            if self.group(index):
                index = self.pairs[index] + 1
                continue
            if text == "<" and (types or angles > 0 or (self.t(index - 1) == ":" and self.t(index - 2) == ":")):
                angles += 1
            elif text == ">" and angles > 0 and self.t(index - 1) not in ("-", "="):
                angles -= 1
            elif text == "," and angles == 0:
                parts.append((start, index, index))
                start = index + 1
            index += 1
        if start < high:
            parts.append((start, high, None))
        return parts

    def chosen_elements(self, low, high, is_chosen, types=False, with_documents=False):
        """一列里剥掉挑中的那几样连它们后面的逗号；挑中的有一样在列尾、后面没有逗号的，留下的那几样里最后一样后面的逗号也剥
        （crates/singlefs-unit-graph 的 blank_the_chosen_elements_of_a_list：剥完与压根没写那几样时逐个词法单元相同）。"""
        last_kept_comma, chosen_last_without_comma = None, False
        for first, end, comma in self.elements(low, high, types):
            if not is_chosen(first, end):
                last_kept_comma = comma
                continue
            self.blank(first, end, with_documents)
            if comma is None:
                chosen_last_without_comma = True
            else:
                self.blank(comma, comma + 1)
        if chosen_last_without_comma and last_kept_comma is not None:
            self.blank(last_kept_comma, last_kept_comma + 1)

    def is_wrapper_type(self, first, end):
        """first..end 整个是 Ghost<…> / Tracked<…>：路径（可带 :: 起头）末段是包装名、带尖括号参数，尖括号配到 end 为止。"""
        index = first + 2 if self.t(first) == ":" and self.t(first + 1) == ":" else first
        while self.ident(index) and self.t(index + 1) == ":" and self.t(index + 2) == ":":
            index += 3
        if not (self.ident(index) and self.t(index) in VERUS_GHOST_WRAPPERS and self.t(index + 1) == "<"):
            return False
        return self.after_angle_brackets(index + 1, end) == end

    def is_wrapper_parameter(self, first, end):
        """类型是 Ghost<…> / Tracked<…> 的参数（连模式）；self 不算。"""
        index = self.skip_attributes(first, end)
        colon = self.single_colon(index, end)
        if colon is None or any(self.t(position) == "self" for position in range(index, colon)):
            return False
        return self.is_wrapper_type(colon + 1, end)

    def is_wrapper_field(self, first, end, named):
        """ghost / tracked 模式的字段，或类型是 Ghost<…> / Tracked<…> 的字段。"""
        index = self.skip_visibility(self.skip_attributes(first, end))
        if named:
            mode = self.t(index) in ("ghost", "tracked") and self.ident(index + 1) and self.t(index + 2) == ":" and self.t(index + 3) != ":"
            name = index + 1 if mode else index
            if not (self.ident(name) and self.t(name + 1) == ":"):
                return False
            return mode or self.is_wrapper_type(name + 2, end)
        mode = self.t(index) in ("ghost", "tracked") and index + 1 < end
        return mode or self.is_wrapper_type(index, end)

    def is_wrapper_call(self, first, end):
        """first..end 整个是 Ghost(…) / Tracked(…)（可带 turbofish）：不带限定、只有一段路径、恰好一个实参。"""
        if not (self.ident(first) and self.t(first) in VERUS_GHOST_WRAPPERS):
            return False
        index = first + 1
        if self.t(index) == ":" and self.t(index + 1) == ":" and self.t(index + 2) == "<":
            index = self.after_angle_brackets(index + 2, end)
            if index is None:
                return False
        if self.t(index) != "(" or self.pairs.get(index) != end - 1:
            return False
        return len(self.elements(index + 1, end - 1, False)) == 1

    def is_wrapper_field_value(self, first, end):
        """结构体字面量里初值、结构体模式里子模式整个是 Ghost(…) / Tracked(…) 的字段。"""
        return (self.ident(first) and self.t(first + 1) == ":" and self.t(first + 2) != ":"
                and self.is_wrapper_call(first + 2, end))

    def data_type(self, word, keyword, end):
        index = keyword + 2
        if self.t(index) == "<":
            index = self.after_angle_brackets(index, end)
            if index is None:
                return
        if word == "struct" and self.t(index) == "(" and index in self.pairs:
            self.fields(index, False)
            return
        body = self.first_brace(index, end)
        if body is None:
            return
        if word != "enum":
            self.fields(body, True)
            return
        for first, variant_end, _comma in self.elements(body + 1, self.pairs[body], False):
            name = self.skip_attributes(first, variant_end)
            if self.t(name + 1) in ("{", "(") and (name + 1) in self.pairs:
                self.fields(name + 1, self.t(name + 1) == "{")

    def fields(self, opening, named):
        self.chosen_elements(opening + 1, self.pairs[opening], lambda first, end: self.is_wrapper_field(first, end, named),
                             types=True, with_documents=True)

    # 执行函数

    def function(self, keyword, end):
        index = keyword + 2
        if self.t(index) == "<":
            index = self.after_angle_brackets(index, end)
        if index is None or self.t(index) != "(" or index not in self.pairs:
            return
        parameters_close = self.pairs[index]
        self.chosen_elements(index + 1, parameters_close, self.is_wrapper_parameter, types=True)
        last = end - 1
        if self.t(last) == "}" and last in self.openings:
            body, signature_end = self.openings[last], self.openings[last]
        elif self.t(last) == ";":
            body, signature_end = None, last
        else:
            return
        after = parameters_close + 1
        if self.t(after) == "-" and self.t(after + 1) == ">" and self.t(after + 2) == "(" and (after + 2) in self.pairs:
            opening = after + 2
            colon = self.single_colon(opening + 1, self.pairs[opening])
            if colon is not None:
                self.blank(opening, colon + 1)
                self.blank(self.pairs[opening], self.pairs[opening] + 1)
        clause = self.first_word(after, signature_end, VERUS_SIGNATURE_CLAUSES)
        if clause is not None:
            self.blank(clause, signature_end)
        if body is not None:
            self.expressions(body + 1, last)

    def statement_end(self, index, high):
        """index 起顶层第一个 ; 之后（没有就是 high）。"""
        while index < high:
            if self.group(index):
                index = self.pairs[index] + 1
                continue
            if self.t(index) == ";":
                return index + 1
            index += 1
        return high

    def after_by(self, index, high):
        """assert(…) 之后的 by …：by 后面可带 (…)，再带 requires … 与 { … }，或直接 { … }。交整条之后的下标。"""
        if self.t(index) != "by":
            return index
        index += 1
        if self.t(index) == "(" and index in self.pairs:
            index = self.pairs[index] + 1
        if self.t(index) == "requires":
            while index < high:
                if self.t(index) == "{" and index in self.pairs:
                    return self.pairs[index] + 1
                if self.t(index) == ";":
                    return index
                index = self.pairs[index] + 1 if self.group(index) else index + 1
            return high
        if self.t(index) == "{" and index in self.pairs:
            return self.pairs[index] + 1
        return index

    def assert_forall_end(self, index, high):
        """assert forall … 到顶层的 ;（不含）或 by { … } 收尾。"""
        while index < high:
            text = self.t(index)
            if text == ";":
                return index
            if text == "by":
                after = index + 1
                if self.t(after) == "(" and after in self.pairs:
                    after = self.pairs[after] + 1
                if self.t(after) == "{" and after in self.pairs:
                    return self.pairs[after] + 1
            index = self.pairs[index] + 1 if self.group(index) else index + 1
        return high

    def ghost_expression_end(self, index, high):
        """index 起是整条只给 Verus 看的表达式（proof { … }、assert、assert forall、assume、reveal 一族）就交它之后的下标，否则 None。"""
        text, following = self.t(index), self.t(index + 1)
        if text == "proof" and following == "{" and (index + 1) in self.pairs:
            return self.pairs[index + 1] + 1
        if text == "assert" and following == "(" and (index + 1) in self.pairs:
            return self.after_by(self.pairs[index + 1] + 1, high)
        if text == "assert" and following == "forall":
            return self.assert_forall_end(index + 2, high)
        if text in VERUS_GHOST_CALLS and following == "(" and (index + 1) in self.pairs:
            return self.pairs[index + 1] + 1
        return None

    def ends_a_loop_head(self, close, high):
        """循环头里一对 { … } 之后接的不像子句的接续（逗号、运算符、子句词……）：它就是循环体。"""
        following = close + 1
        return following >= high or self.t(following) not in VERUS_CLAUSE_CONTINUATIONS

    def clause_body(self, clause, high):
        """clause 起的子句之后那个体的 {（照 ends_a_loop_head 认）；先遇到 ; 或到 high 都没认出交 None。"""
        index = clause + 1
        while index < high:
            if self.t(index) == "{" and index in self.pairs:
                if self.ends_a_loop_head(self.pairs[index], high):
                    return index
                index = self.pairs[index] + 1
                continue
            if self.t(index) == ";":
                return None
            index = self.pairs[index] + 1 if self.group(index) else index + 1
        return None

    def loop_head(self, keyword, high):
        """while / loop / for 的头：for 给迭代器起的名（in 之后的「名:」）与 invariant 一族、ensures、decreases 那几截到循环体之前。"""
        is_for, index, clause = self.t(keyword) == "for", keyword + 1, None
        while index < high:
            text = self.t(index)
            if (is_for and clause is None and text == "in" and self.ident(index + 1) and self.t(index + 2) == ":"
                    and self.t(index + 3) != ":"):
                self.blank(index + 1, index + 3)
            if clause is None and self.ident(index) and text in VERUS_LOOP_CLAUSES:
                clause = index
            if text == "{" and index in self.pairs:
                if clause is None or self.ends_a_loop_head(self.pairs[index], high):
                    break
                index = self.pairs[index] + 1
                continue
            if text == ";":
                return
            index = self.pairs[index] + 1 if self.group(index) else index + 1
        else:
            return
        if clause is not None:
            self.blank(clause, index)

    def is_call(self, opening):
        """opening 那个 ( 是调用或方法调用的实参表：前面是名字（不是关键字、不是 fn 名）、) 或 ]，或 ::<…> 的 >。"""
        before = self.t(opening - 1)
        if before in (")", "]"):
            return True
        if before == ">":
            depth, index = 0, opening - 1
            while index >= 0:
                if self.t(index) == ">":
                    depth += 1
                elif self.t(index) == "<":
                    depth -= 1
                    if depth == 0:
                        return self.t(index - 1) == ":" and self.t(index - 2) == ":"
                index -= 1
            return False
        return self.ident(opening - 1) and before not in VERUS_NOT_CALLEES and self.t(opening - 2) != "fn"

    def expressions(self, low, high):
        """函数体、常量初值这一段：剥整条只给 Verus 看的表达式与语句（当语句时连分号），循环头的子句，实参与字面量字段里的 Ghost(…) / Tracked(…)。"""
        index = low
        while index < high:
            text = self.t(index)
            if self.ident(index) and self.t(index + 1) == "!" and self.group(index + 2):
                index = self.pairs[index + 2] + 1
                continue
            if self.ident(index) and self.t(index - 1) not in (".", ":", "fn"):
                end = self.ghost_expression_end(index, high)
                if end is None and ((text == "let" and self.t(index + 1) in ("ghost", "tracked"))
                                    or (text == "broadcast" and self.t(index + 1) == "use")):
                    end = self.statement_end(index, high)
                elif end is not None and end < high and self.t(end) == ";" and (index == low or self.t(index - 1) in (";", "{", "}")):
                    end += 1
                if end is None and text in ("requires", "ensures"):
                    # 闭包的 requires / ensures（循环头里的已由 loop_head 剥过，这里再剥一遍是同一截）：到闭包体之前
                    end = self.clause_body(index, high)
                if end is not None:
                    self.blank(index, end)
                    index = end
                    continue
                if text in ("while", "loop", "for"):
                    self.loop_head(index, high)
            if self.group(index):
                close = self.pairs[index]
                if text == "(" and self.is_call(index):
                    self.chosen_elements(index + 1, close, self.is_wrapper_call)
                elif text == "{":
                    self.chosen_elements(index + 1, close, self.is_wrapper_field_value)
                self.expressions(index + 1, close)
                index = close + 1
                continue
            index += 1


VERIFIER_ATTRIBUTE_HEAD = "verifier"
OPENING_PUNCTUATION = ("{", "(", "[")
CLOSING_PUNCTUATION = ("}", ")", "]")


def verifier_attribute_token_ranges(kinds, texts):
    """外属性 #[verifier::…] 与括号写法 #[verifier(…)] 各从 # 到配对的 ] 的词法单元下标区间 [(# 的下标, ] 的下标)]，
    照 crates/singlefs-unit-graph 的 verifier_attribute_ranges_of：verus! 里外同一套认法；不认的有内属性 #![verifier::…]、#[cfg_attr(…, verifier::…)]、
    落在别的宏调用与 macro_rules! 的词法单元组里的（左括号前面紧挨 名 !，或 macro_rules ! 名；verus ! { 不算别的宏）、配不上 ] 的。"""
    if VERIFIER_ATTRIBUTE_HEAD not in texts:
        return []
    count = len(texts)

    def punctuation(index, wanted):
        return 0 <= index < count and kinds[index] == PUNCT and texts[index] == wanted

    def closing_bracket(opening):
        depth = 0
        for index in range(opening, count):
            if kinds[index] != PUNCT:
                continue
            if texts[index] in OPENING_PUNCTUATION:
                depth += 1
            elif texts[index] in CLOSING_PUNCTUATION:
                depth -= 1
                if depth == 0:
                    return index
        return None

    def attribute_closing_bracket(hash_index):
        opens_a_verifier_attribute = (punctuation(hash_index, "#") and punctuation(hash_index + 1, "[") and hash_index + 2 < count
                                      and kinds[hash_index + 2] == IDENT and texts[hash_index + 2] == VERIFIER_ATTRIBUTE_HEAD
                                      and ((punctuation(hash_index + 3, ":") and punctuation(hash_index + 4, ":")) or punctuation(hash_index + 3, "(")))
        return closing_bracket(hash_index + 1) if opens_a_verifier_attribute else None

    def macro_name_opening(opening):
        if opening >= 2 and punctuation(opening - 1, "!") and kinds[opening - 2] == IDENT:
            return texts[opening - 2]
        if (opening >= 3 and kinds[opening - 1] == IDENT and punctuation(opening - 2, "!") and kinds[opening - 3] == IDENT
                and texts[opening - 3] == "macro_rules"):
            return "macro_rules"
        return None

    # 每一层还开着的括号：这一层（连外层）落没落在别的宏的词法单元组里
    inside_another_macro, ranges, index = [], [], 0
    while index < count:
        if not (inside_another_macro and inside_another_macro[-1]):
            close = attribute_closing_bracket(index)
            if close is not None:
                ranges.append((index, close))
                index = close + 1
                continue
        if kinds[index] == PUNCT and texts[index] in OPENING_PUNCTUATION:
            name = macro_name_opening(index)
            opens_another_macro = name is not None and (name != "verus" or texts[index] != "{")
            inside_another_macro.append(bool(inside_another_macro and inside_another_macro[-1]) or opens_another_macro)
        elif kinds[index] == PUNCT and texts[index] in CLOSING_PUNCTUATION and inside_another_macro:
            inside_another_macro.pop()
        index += 1
    return ranges


def verus_ghost_code_blanked(text):
    """text 里每一块 verus! 中只给 Verus 看的那几截、verus! 里外的 #[verifier::…] / #[verifier(…)] 外属性（verifier_attribute_token_ranges）换成空格
    （换行留着，行号不变；VerusGhostCode 挑的那几截，各从首词法单元起到末词法单元止，整条剥的连紧挨在前面的文档注释）→ 剥过的源码；
    括号配不上的交 None（那份文件照原样那一份办）。弄坏开关 product-view-verifier-attributes-kept 下不剥属性。"""
    spans, document_spans = [], []
    kinds, texts, _lines, documents = rust_lex(text, spans, document_spans)
    pairs, _bad = match_brackets(kinds, texts)
    if pairs is None:
        return None
    ranges = VerusGhostCode(kinds, texts, pairs).collect()
    attributes = [] if "product-view-verifier-attributes-kept" in active_breaks() else [
        (spans[hash_index][0], spans[close][1]) for hash_index, close in verifier_attribute_token_ranges(kinds, texts)]
    if not ranges and not attributes:
        return text
    leading_documents = {}
    for (_line, _style, _rows, next_index, _raw), (start, _end) in zip(documents, document_spans):
        leading_documents[next_index] = min(start, leading_documents.get(next_index, start))
    characters = list(text)
    for first, end, with_documents in ranges:
        start = spans[first][0]
        if with_documents and first in leading_documents:
            start = min(start, leading_documents[first])
        for position in range(start, spans[end - 1][1]):
            if characters[position] != "\n":
                characters[position] = " "
    for start, end in attributes:
        for position in range(start, end):
            if characters[position] != "\n":
                characters[position] = " "
    return "".join(characters)


def product_view_agrees(rust_file, theirs_lines):
    """剥过的那一份重切出来的单元与 Rust 那一份交的这份文件的单元行逐个相同：种类、文件:起-止、指纹的多重集相同，
    Rust 那一份的限定名以这一份的嵌套路径收尾（前缀要等模块树连好才有，这里不比；文档测试不比名字）。"""
    theirs, names = collections.Counter(), {}
    for line in theirs_lines:
        kind, name, place, fingerprint = line.split("\t")
        theirs[(kind, place, fingerprint)] += 1
        names.setdefault((kind, place, fingerprint), []).append(name)
    ours = collections.Counter((unit.output_kind(), f"{unit.file}:{unit.first_line}-{unit.last_line}", unit.fingerprint) for unit in rust_file.units)
    if ours != theirs:
        return False
    return all(unit.kind == "doctest" or any(name == unit.name or name.endswith("::" + unit.name)
                                             for name in names[(unit.output_kind(), f"{unit.file}:{unit.first_line}-{unit.last_line}", unit.fingerprint)])
               for unit in rust_file.units)


class ProductViewUnavailable(Exception):
    """拿不到 Rust 那一份产品行为视图（找不到或编不出 unit-graph、它退非 0）：有 verus! 块的文件记拒算 product-view-unavailable。"""


def product_view_dump(root):
    """unit-graph dump <根> --judgment-kind product-behavior 的输出行（文件头「判定种类」）；拿不到抛 ProductViewUnavailable。"""
    program = os.path.join(unit_graph_directory(), "unit-graph")
    try:
        completed = subprocess.run([program, "dump", os.path.abspath(root), "--judgment-kind", PRODUCT_BEHAVIOR],
                                   capture_output=True, text=True, errors="surrogateescape", stdin=subprocess.DEVNULL)
    except OSError as error:
        raise ProductViewUnavailable(f"起不了 unit-graph：{error}") from error
    if completed.returncode != 0:
        tail = " | ".join(line for line in completed.stderr.strip().split("\n")[-2:] if line.strip())
        raise ProductViewUnavailable(f"unit-graph dump --judgment-kind {PRODUCT_BEHAVIOR} 退 {completed.returncode}：{tail[:200]}")
    return completed.stdout.splitlines()


def unit_graph_directory():
    """产品行为视图用的 unit-graph 所在的目录：设了 FUNCTION_UNITS_UNIT_GRAPH_DIRECTORY 取它，没设取这份库所在仓那一份编出来的（built_unit_graph）。"""
    return os.environ.get(UNIT_GRAPH_DIRECTORY_VARIABLE) or built_unit_graph(LIBRARY_ROOT)


def built_unit_graph(repository):
    """repository 里 crates/singlefs-unit-graph 编出来的 unit-graph 所在的目录（文件头「判定种类」里「unit-graph 从哪来」）；编不出抛 ProductViewUnavailable。"""
    source = os.path.join(repository, "crates", "singlefs-unit-graph")
    if not os.path.isfile(os.path.join(source, "Cargo.toml")):
        raise ProductViewUnavailable(f"{repository} 里没有 crates/singlefs-unit-graph")
    thread_registry = os.path.join(repository, ".claude", "heavy-thread-default")
    lock_file = os.path.join(repository, "Cargo.lock")
    inputs = [thread_registry, lock_file]
    for directory, subdirectories, names in os.walk(source):
        subdirectories[:] = sorted(name for name in subdirectories if name not in SKIPPED_DIRECTORIES)
        inputs += [os.path.join(directory, name) for name in sorted(names)]
    digest = hashlib.sha256()
    for path in inputs:
        digest.update(os.path.relpath(path, repository).encode("utf-8", "surrogateescape") + b"\0")
        try:
            with open(path, "rb") as handle:
                digest.update(handle.read())
        except OSError:
            digest.update(b"<missing>")
        digest.update(b"\0")
    cache = unit_graph_product_view_cache_directory()
    directory = os.path.join(cache, digest.hexdigest())
    program = os.path.join(directory, "unit-graph")
    if os.access(program, os.X_OK):
        return directory
    try:
        os.makedirs(cache, exist_ok=True)
        with open(os.path.join(cache, ".lock"), "a", encoding="utf-8") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
            if not os.access(program, os.X_OK):
                build_unit_graph(repository, source, thread_registry, lock_file, cache, directory, program)
    except OSError as error:
        raise ProductViewUnavailable(f"编不了 unit-graph（{cache}）：{error}") from error
    return directory


def build_unit_graph(repository, source, thread_registry, lock_file, cache, directory, program):
    """在 cache 下的临时工作区里编 unit-graph、换上 program；编完删工作区与编译目录，cache 里只留最近 UNIT_GRAPH_KEPT_PROGRAMS 份程序。"""
    build = tempfile.mkdtemp(prefix="build-", dir=cache)
    try:
        workspace = os.path.join(build, "workspace")
        shutil.copytree(source, os.path.join(workspace, "crates", "singlefs-unit-graph"), ignore=shutil.ignore_patterns("target"))
        os.makedirs(os.path.join(workspace, ".claude"))
        shutil.copy2(thread_registry, os.path.join(workspace, ".claude", "heavy-thread-default"))
        if os.path.isfile(lock_file):
            shutil.copy2(lock_file, os.path.join(workspace, "Cargo.lock"))
        with open(os.path.join(workspace, "Cargo.toml"), "w", encoding="utf-8") as handle:
            handle.write('[workspace]\nmembers = ["crates/singlefs-unit-graph"]\nresolver = "2"\n\n[profile.release]\nopt-level = 3\n'
                         'overflow-checks = true\ndebug-assertions = true\n')
        # 直接起 cargo：线程是服务给这一条套的 capped.sh 设好的 CARGO_BUILD_JOBS（capped.sh 只认服务套的那一层）
        completed = subprocess.run(["cargo", "build", "--release", "--offline",
                                    "--manifest-path", os.path.join(workspace, "Cargo.toml"), "-p", "singlefs-unit-graph", "--bin", "unit-graph"],
                                   capture_output=True, text=True, errors="surrogateescape", stdin=subprocess.DEVNULL,
                                   env=dict(os.environ, CARGO_TARGET_DIR=os.path.join(build, "target")))
        if completed.returncode != 0:
            tail = " | ".join(line for line in completed.stderr.strip().split("\n")[-3:] if line.strip())
            raise ProductViewUnavailable(f"cargo build --release -p singlefs-unit-graph 退 {completed.returncode}：{tail[:300]}")
        os.makedirs(directory, exist_ok=True)
        partial = os.path.join(directory, "unit-graph.partial")
        shutil.copy2(os.path.join(build, "target", "release", "unit-graph"), partial)
        os.replace(partial, program)
    finally:
        shutil.rmtree(build, ignore_errors=True)
    kept = sorted((name for name in os.listdir(cache) if not name.startswith((".", "build-"))),
                  key=lambda name: os.path.getmtime(os.path.join(cache, name)), reverse=True)
    for name in kept[UNIT_GRAPH_KEPT_PROGRAMS:]:
        shutil.rmtree(os.path.join(cache, name), ignore_errors=True)


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
# 新建临时目录、临时文件的调用：读它们底下的文件读的是这一趟自己写的（文件头「运行期读的数据文件」）
TEMPORARY_CALLS = frozenset(("tempfile.mkdtemp", "tempfile.mkstemp", "tempfile.gettempdir", "tempfile.TemporaryDirectory", "tempfile.NamedTemporaryFile",
                             "tempfile.TemporaryFile", "tempfile.SpooledTemporaryFile"))
# 路径对象上只换形状、不换出处的方法：前缀从哪来照点前面那个对象
PATH_METHODS = frozenset(("resolve", "absolute", "as_posix", "expanduser", "joinpath", "with_suffix", "with_name", "relative_to"))
DATA_FORM = "data-path-unfolded"
# 拷文件的调用（文件头「运行期读的数据文件」）：源照读文件认，折不出的那一截不拒算（拷进副本、样本树的那一份归起在副本上的那一趟管）
COPY_CALLS = frozenset(("shutil.copy", "shutil.copy2", "shutil.copyfile"))
# 版本询问（文件头「版本询问」）：命令位置上的程序折不出、后面只有这里头的一个词的，记成工具链询问
TOOLCHAIN_QUERY_WORDS = frozenset(("-V", "--version", "-vV"))
# 工具链单元的种类与名字（文件头「版本询问」）；rust_toolchain_lines 另问的编译器：这两个环境变量与 cargo 配置的 build.rustc
TOOLCHAIN_UNIT_KIND = "rust-toolchain"
TOOLCHAIN_PROGRAM_VARIABLES = ("RUSTC", "CARGO_BUILD_RUSTC")
TOOLCHAIN_QUERY_SECONDS = 60
# 登记表点名的程序（文件头「登记表点名的程序」）：起子进程的模块在模块级写的那张表的名字
REGISTRY_LISTED_PROGRAMS_TABLE = "REGISTRY_LISTED_PROGRAMS"
# 程序入口带参数（文件头「程序入口带参数」）：已知值表里 sys.argv 的名字（不是合法的 python 名字，撞不上参数名）
ARGV_BINDING = "<sys.argv>"
# 给了参数时 import 时执行的赋值只收赋值本身（PythonModule.table_reference_names）：ScriptClosure 里这一种走法记成这个视图
SHALLOW_IMPORT_VIEW = "import-time-shallow"
# 运行期读的文件、起子进程的命令里折不出的那一截，出处是这几样的算调用方交进来的输入（文件头「运行期读的数据文件」二、「调用方交进来的命令」）
CALLER_INPUT_ORIGINS = frozenset(("input", "temporary", "outside"))
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


def is_main_name_test(test):
    return (isinstance(test, ast.Compare) and isinstance(test.left, ast.Name) and test.left.id == "__name__" and len(test.comparators) == 1
            and isinstance(test.comparators[0], ast.Constant) and test.comparators[0].value == "__main__")


def is_main_guard(statement, breaks=frozenset()):
    """只在这份文件被当程序起时才执行的那一段（文件头「Python 闭包：import 时执行的模块级语句」的 main-guard 两种写法）：
    if __name__ == "__main__":，与不带 else 的 if __name__ == "__main__" and <别的条件>:（and 从左往右短路，import 时头一项是假的，后面的条件与块都不执行）。
    and 写法里 __name__ 那一项不在最左、带 else 的认不出，留在 module-body 照收。弄坏开关 python-main-guard-conjunctions-unrecognized：只认头一种。
    头一种带 else（或 elif）的照认：PythonModule.split 只把 if 那一支切成 main-guard，else 那一支放进 module-body（被 import 时执行）。"""
    if not isinstance(statement, ast.If):
        return False
    test = statement.test
    if isinstance(test, ast.BoolOp) and isinstance(test.op, ast.And) and not statement.orelse \
            and "python-main-guard-conjunctions-unrecognized" not in breaks:
        test = test.values[0]
    return is_main_name_test(test)


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


def walk_nodes_skipping(nodes, skipped):
    """同 walk_nodes（每个根各自按层走），skipped 里（按 id）的节点连同它底下的都不走（程序入口带参数时剪掉的那几支）。"""
    for root in nodes:
        if id(root) in skipped:
            continue
        pending = collections.deque([root])
        while pending:
            node = pending.popleft()
            yield node
            pending.extend(child for child in ast.iter_child_nodes(node) if id(child) not in skipped)


class KnownWords(collections.namedtuple("KnownWords", "words exact")):
    """程序入口带参数时已知的一串词（文件头「程序入口带参数」）：words 逐个是词（不知道的写 None），exact 是长度知不知道（不知道的是开口的一串）。"""

    def index(self, position):
        """[k]：k 是整数；落在不知道的那一截上交 None。"""
        if position >= 0:
            return self.words[position] if position < len(self.words) else None
        return self.words[position] if self.exact and -position <= len(self.words) else None

    def sliced(self, lower, upper):
        """[lower:upper]（都是不小于 0 的整数或 None，不带步长）。"""
        if upper is None:
            return KnownWords(self.words[lower:], self.exact)
        if self.exact or upper <= len(self.words):
            return KnownWords(self.words[lower:upper], True)
        return KnownWords(self.words[lower:], False)


def launch_arguments(elements, values, start):
    """起 python 脚本的命令里脚本之后那几个元素 → KnownWords（文件头「程序入口带参数」子进程边）：折得出、恰好一个取值的是那个词，别的是 None；
    遇到 OPAQUE、* 展开、POSITIONAL（个数不知道）就到此为止、记成开口的一串。"""
    words = []
    for position in range(start, len(elements)):
        item = elements[position]
        if item is OPAQUE or (isinstance(item, ValueElement) and item.values is POSITIONAL) or (
                isinstance(item, tuple) and isinstance(item[0], ast.Starred)):
            return KnownWords(tuple(words), False)
        value = values[position] if position < len(values) else None
        words.append(next(iter(value)) if value is not None and len(value) == 1 else None)
    return KnownWords(tuple(words), True)


def known_equal(left, right):
    """两个已知值相不相等：True / False；判不出交 None。"""
    if left is None or right is None:
        return None
    if isinstance(left, str) and isinstance(right, str):
        return left == right
    if isinstance(left, str) or isinstance(right, str):
        return False
    if left.exact and right.exact and len(left.words) != len(right.words):
        return False
    for mine, theirs in zip(left.words, right.words):
        if mine is not None and theirs is not None and mine != theirs:
            return False
    if left.exact and right.exact and None not in left.words and None not in right.words:
        return True
    return None


def literal_integer(node):
    """不小于 0 的整数字面量 → 它；不写（None）→ None 交 None 的区分由调用方做；别的 → False。"""
    if isinstance(node, ast.Constant) and type(node.value) is int and node.value >= 0:
        return node.value
    return False


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
    """a, b = x, y：→ [(名字, 对上的值节点)…]；两边不是等长的元组 / 列表、有 *、目标里有不是名字的 → None。
    a, b = s[k:k+2] 照 a = s[k]、b = s[k+1] 对上（sliced_elements；文件头「起代码的写法逐种认」）。"""
    if isinstance(target, (ast.Tuple, ast.List)) and isinstance(value, ast.Subscript) and isinstance(value.slice, ast.Slice) \
            and "python-slice-unpacking-unpaired" not in active_breaks():
        return sliced_elements(target, value)
    if not (isinstance(target, (ast.Tuple, ast.List)) and isinstance(value, (ast.Tuple, ast.List)) and len(target.elts) == len(value.elts)):
        return None
    if any(not isinstance(item, ast.Name) for item in target.elts) or any(isinstance(item, ast.Starred) for item in value.elts):
        return None
    return [(item.id, element) for item, element in zip(target.elts, value.elts)]


def sliced_elements(target, value):
    """a, b = s[k:k+2]：→ [(a, s[k]), (b, s[k+1])]，下标节点是另造的 s[<整数>]（挂在切片节点上，同一处拆包每次交同一批）。
    下界不写或是不小于 0 的整数字面量、不带步长、上界不写或等于下界加目标个数、目标全是名字，才对上；别的 → None。
    上界不写时 s 的长度对不上，跑起来就抛 ValueError、走不到后面，所以照对上算。"""
    bounds, names = value.slice, target.elts
    if bounds.step is not None or not names or any(not isinstance(item, ast.Name) for item in names):
        return None
    lower = 0 if bounds.lower is None else (
        bounds.lower.value if isinstance(bounds.lower, ast.Constant) and type(bounds.lower.value) is int else None)
    if lower is None or lower < 0:
        return None
    if bounds.upper is not None and not (isinstance(bounds.upper, ast.Constant) and type(bounds.upper.value) is int
                                         and bounds.upper.value == lower + len(names)):
        return None
    elements = getattr(value, "sliced_elements", None)
    if elements is None:
        elements = [ast.copy_location(ast.Subscript(value=value.value, slice=ast.copy_location(ast.Constant(value=lower + offset), value),
                                                    ctx=ast.Load()), value) for offset in range(len(names))]
        value.sliced_elements = elements   # 不在 _fields 里：ast.dump 与 walk_nodes 都看不见它，指纹不变
    return [(item.id, element) for item, element in zip(names, elements)]


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


def guard_assignments(name, unit):
    """main-guard 单元里给 name 的赋值的值（照写的次序）；单元不是 main-guard 或没赋过 → []。"""
    if unit is None or unit.kind != "main-guard":
        return []
    return [node.value for node in walk_nodes(unit.nodes or []) if isinstance(node, (ast.Assign, ast.AnnAssign)) and node.value is not None
            and any(isinstance(target, ast.Name) and target.id == name for target in (node.targets if isinstance(node, ast.Assign) else [node.target]))]


def enclosing_unit(unit):
    """PythonModule.nested_unit 造的作用域单元：它外面那一层（模块级的单元交 None）。"""
    return unit.cache.get("enclosing") if unit is not None and unit.cache else None


def walk_targets(statement):
    """for 的循环变量、with 的 as 绑的那几个名字。"""
    targets = [statement.target] if isinstance(statement, (ast.For, ast.AsyncFor)) else [item.optional_vars for item in statement.items if item.optional_vars]
    for target in targets:
        yield from (inner for inner in ast.walk(target) if isinstance(inner, ast.Name))


class PythonFacts:
    """inline_scripts：起的内联脚本（InlineScript，python 的在闭包里切成单元、shell 的当一段 bash 认）；whole_modules：整份连上的模块
    （-m unittest 点名的测试模块）；handled：每个起子进程的调用各记一条认成了什么（只给量全仓的人看，不进判定）。"""
    __slots__ = ("names", "attributes", "loads", "program_links", "doc_reads", "source_read", "strings", "refusals", "import_targets", "inline_scripts",
                 "whole_modules", "handled", "data_reads", "toolchain_queries", "caller_launches", "narrowed_calls", "registry_reads",
                 "program_arguments", "shallow_tables")

    def __init__(self):
        self.names, self.attributes, self.loads, self.program_links = set(), [], [], []
        self.doc_reads, self.source_read, self.strings, self.refusals, self.import_targets = [], False, set(), [], []
        self.inline_scripts, self.whole_modules, self.handled = [], [], []
        # 运行期读的数据文件（文件头「运行期读的数据文件」）：[(取法集合, 行, 是不是拷文件的源)]，取法见 PythonModule.data_read_origins
        self.data_reads = []
        # 版本询问的行（文件头「版本询问」）；调用方交进来的命令：[(拒算, 折不出那几截的出处)]，收闭包时判是不是入口参数（ScriptClosure.caller_launch_refusals）
        self.toolchain_queries, self.caller_launches = [], []
        # 程序入口带参数（文件头「程序入口带参数」）：带着已知实参往下剪的调用 [(被调的函数单元, 已知值表)]；
        # 子进程边带过去的参数：{被起的文件: {KnownWords 或 None（不知道）…}}
        self.narrowed_calls, self.program_arguments = [], {}
        # 程序入口带参数时只问键在不在（<词> in 表）的那几张本模块字典字面量的赋值单元：照 SHALLOW_IMPORT_VIEW 收（值里的函数名不连）
        self.shallow_tables = []
        # 登记表点名的程序（文件头「登记表点名的程序」）读的登记表（相对根的路径）
        self.registry_reads = []


class PythonIndex:
    def __init__(self, root):
        self.root = os.path.abspath(root)
        self.breaks = active_breaks()
        self.modules = {}
        self.folding = set()
        self._repository_names = None
        self._repository_files = None

    def repository_files(self):
        """根里的全部文件（相对根，跳过 SKIPPED_DIRECTORIES）：运行期读的路径只认得出字面尾巴时，按尾巴对上的那一组（PythonModule.read_origins 的 tail）。"""
        if self._repository_files is None:
            self._repository_files = files_under(self.root, ("",))
        return self._repository_files

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

    def add_def_head(self, statement, name):
        """import 时就求值的 def 头（文件头「Python」一段的 def-head）：带装饰器或缺省值的 def（模块级的与模块级类里的方法）另切一个单元，
        只收装饰器与缺省值那几个表达式（指纹与边都只看它们），不收函数体、不进 by_name（不被按名字点到）、不挂类头（不让类算被点名）。
        限定名 <def 的限定名>.<head>。弄坏开关 python-def-heads-ignored 下不切。"""
        arguments = statement.args
        evaluated = list(statement.decorator_list) + list(arguments.defaults) + [value for value in arguments.kw_defaults if value is not None]
        if not evaluated or "python-def-heads-ignored" in self.index.breaks:
            return None
        first = min(node.lineno for node in evaluated)
        last = max(getattr(node, "end_lineno", None) or node.lineno for node in evaluated)
        head = Unit("python", "def-head", f"{name}.<head>", self.path, first, last,
                    dump_fingerprint(ast.Module(body=[ast.Expr(value=node) for node in evaluated], type_ignores=[])))
        head.node, head.nodes, head.module, head.container = None, evaluated, self, None
        self.units.append(head)
        return head

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
        rest, guards = [], []
        for statement in body:
            if isinstance(statement, (ast.FunctionDef, ast.AsyncFunctionDef)):
                self.add_unit("def", statement.name, statement, [statement], self.function_fingerprint(statement))
                self.add_def_head(statement, statement.name)
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
                        self.add_def_head(member, f"{statement.name}.{member.name}")
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
            elif is_main_guard(statement, self.index.breaks):
                if statement.orelse and "python-main-guard-else-dropped" not in self.index.breaks:
                    # 标准写法带 else（或 elif）：只有 if 那一支是 main-guard；else 那一支被 import 时执行，照别的模块级语句进 module-body。
                    # 弄坏开关 python-main-guard-else-dropped：连 else 整段当 main-guard（被 import 时 else 那一支哪儿都不收）
                    main_branch = copy.copy(statement)
                    main_branch.orelse = []
                    main_branch.end_lineno = statement.body[-1].end_lineno or statement.body[-1].lineno
                    guards.append(main_branch)
                    rest.extend(statement.orelse)
                else:
                    guards.append(statement)
            else:
                rest.append(statement)
        if "python-main-guards-last-only" in self.index.breaks:
            guards = guards[-1:]
        if guards:
            # 一份文件里几段 main-guard 合成一个单元（被当程序起时每一段都执行）：节点是那几段，只有一段时指纹照旧取那一段；
            # 弄坏开关 python-main-guards-last-only：只留末一段（前面几段被后面的盖掉、哪儿都不收）
            fingerprint = dump_fingerprint(guards[0]) if len(guards) == 1 else dump_fingerprint(ast.Module(body=guards, type_ignores=[]))
            self.main_guard = self.add_unit("main-guard", "<main-guard>", guards[0], guards, fingerprint)
            self.main_guard.last_line = guards[-1].end_lineno or guards[-1].lineno
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
        if local is None and unit is not None and enclosing_unit(unit) is not None:
            # 套在函数里的函数（nested_unit）：自己没绑的名字到外层函数里找
            return self.binding(name, enclosing_unit(unit))
        return local if local is not None else self.bindings().get(name)

    def nested_unit(self, definition, enclosing):
        """套在 enclosing 里的函数 definition 自成一层作用域（文件头「起代码的写法逐种认」嵌套函数）：一个只用来折叠的单元，不进单元表、不进闭包，
        局部赋值与参数照它自己的，别的名字到 enclosing 里找（enclosing_unit），参数按 enclosing 里调它的那几处折（function_call_sites）。"""
        nested = self.__dict__.setdefault("_nested_units", {})
        if id(definition) not in nested:
            unit = Unit("python", "def", f"{enclosing.name}.<locals>.{definition.name}", self.path, definition.lineno,
                        getattr(definition, "end_lineno", None) or definition.lineno, "")
            unit.node, unit.nodes, unit.module, unit.container = definition, [definition], self, None
            unit.cache = {"enclosing": enclosing}
            nested[id(definition)] = unit
        return nested[id(definition)]

    def scope_of(self, node, unit):
        """单元里一个节点所在的那一层函数作用域：套在单元里的 def 里面的落到那个 def 的 nested_unit（最里层那一个），别的就是单元自己。
        弄坏开关 python-nested-scopes-unfolded 下一律是单元自己（套着的函数里的局部变量与参数折不出）。"""
        if unit is None or "python-nested-scopes-unfolded" in self.index.breaks:
            return unit
        if unit.cache is None:
            unit.cache = {}
        scopes = unit.cache.get("scopes")
        if scopes is None:
            scopes = {}
            own = unit.node if isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)) else None

            def visit(current, scope):
                for child in ast.iter_child_nodes(current):
                    inner = scope
                    if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef)) and child is not own:
                        inner = self.nested_unit(child, scope)
                    scopes[id(child)] = inner
                    visit(child, inner)
            for top in unit.nodes or []:
                scopes[id(top)] = unit
                visit(top, unit)
            unit.cache["scopes"] = scopes
        return scopes.get(id(node), unit)

    def function_call_sites(self, unit):
        """调 unit 这个函数的那几处 [(调用节点, 调用处的作用域单元)]：模块级函数取本模块里按名字调它的每一处（call_sites）；
        套在函数里的（nested_unit）只取外层函数里调它的那几处。"""
        function = unit.node
        enclosing = enclosing_unit(unit)
        if enclosing is None:
            return self.call_sites().get(function.name, [])
        return [(node, self.scope_of(node, enclosing)) for node in walk_nodes(enclosing.nodes or [])
                if isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id == function.name]

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
                        # 调用处在套着的函数里的，实参照那一层作用域折（scope_of）
                        self._call_sites[node.func.id].append((node, self.scope_of(node, unit)))
                    elif isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.args and \
                            node.func.attr in ("submit", "map") and isinstance(node.args[0], ast.Name):
                        if "python-submitted-functions-unfolded" not in self.index.breaks:
                            # pool.submit(f, a, b, k=v)：照 f(a, b, k=v) 算一处调用；pool.map(f, 序列)：实参是逐个取的元素，记成 *序列（折不出，照拒算）
                            submitted = node.func.attr == "submit"
                            call = ast.Call(func=node.args[0], args=list(node.args[1:]) if submitted else [ast.Starred(value=item) for item in node.args[1:]],
                                            keywords=list(node.keywords) if submitted else [], lineno=node.lineno, col_offset=node.col_offset)
                            self._call_sites[node.args[0].id].append((call, self.scope_of(node, unit)))
                    elif isinstance(node, ast.Call) and "python-dispatch-calls-ignored" not in self.index.breaks:
                        # 分派表[键](…) / 分派表.get(键)(…)：表（模块级只赋过一次的字典字面量）里当值写的每个函数都算被这一处调了
                        table = node.func.value if isinstance(node.func, ast.Subscript) else (
                            node.func.func.value if isinstance(node.func, ast.Call) and isinstance(node.func.func, ast.Attribute)
                            and node.func.func.attr == "get" else None)
                        found = self.module_dictionary(table.id, None) if isinstance(table, ast.Name) else None
                        if found is not None:
                            for value in found[0].values:
                                if isinstance(value, ast.Name):
                                    self._call_sites[value.id].append((node, self.scope_of(node, unit)))
        return self._call_sites

    def method_call_sites(self, unit):
        """类里的方法 unit：本模块里每一处 <任何东西>.<方法名>(…)（不只 self.，经别的对象调的同样算）→ [(调用节点, 调用处的作用域单元)]。"""
        cached = self.__dict__.setdefault("_method_sites", {})
        if unit.node.name not in cached:
            cached[unit.node.name] = [(node, self.scope_of(node, owner)) for owner in self.units for node in walk_nodes(owner.nodes or [])
                                      if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.func.attr == unit.node.name]
        return cached[unit.node.name]

    def fold_parameter(self, unit, name, depth):
        function = unit.node
        if not isinstance(function, (ast.FunctionDef, ast.AsyncFunctionDef)):
            return None
        method = unit.container is not None
        if method and "python-method-parameters-unfolded" in self.index.breaks:
            return None
        arguments = function.args
        positional = [argument.arg for argument in arguments.posonlyargs + arguments.args]
        if method and not any(isinstance(decorator, ast.Name) and decorator.id == "staticmethod" for decorator in function.decorator_list):
            # 方法的头一个参数（self / cls）是点前面那个对象，调用处的实参从第二个参数起对
            if not positional or positional[0] == name:
                return None
            positional = positional[1:]
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
        # 方法：本模块里按方法名调它的每一处（method_call_sites）；套在函数里的：外层函数里调它的那几处（function_call_sites）
        sites = self.method_call_sites(unit) if method else self.function_call_sites(unit)
        if not sites:
            return None
        values = set()
        for call, caller in sites:
            argument = None
            if position is not None and position < len(call.args):
                argument = call.args[position]
                if any(isinstance(item, ast.Starred) for item in call.args[:position + 1]):
                    return None
            elif any(isinstance(item, ast.Starred) for item in call.args):
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

    def comprehension_binding(self, node, unit):
        """一个读名字的节点落在绑这个名字的推导式里（[… name … for name in <序列>]）→ 最里层那个推导式里绑它的那一节 generator；
        绑它的是拆包（for name, x in …）交 COMPREHENSION_UNPACKED；不在这样的推导式里 → None。
        弄坏开关 python-comprehension-variables-unfolded 下一律 None（推导式的循环变量折不出）。"""
        if unit is None or "python-comprehension-variables-unfolded" in self.index.breaks:
            return None
        if unit.cache is None:
            unit.cache = {}
        bindings = unit.cache.get("comprehension-bindings")
        if bindings is None:
            bindings = {}
            # ast.walk 先走外层：里层的推导式后记，同一个名字节点落在里层的那一节上
            for comprehension in walk_nodes(unit.nodes or []):
                if not isinstance(comprehension, (ast.ListComp, ast.SetComp, ast.GeneratorExp, ast.DictComp)):
                    continue
                # 头一节的序列在外层作用域里求值，里面的名字不归这个推导式
                outer = {id(inner) for inner in ast.walk(comprehension.generators[0].iter)}
                for generator in comprehension.generators:
                    names = bound_names(generator.target)
                    for inner in ast.walk(comprehension):
                        if isinstance(inner, ast.Name) and isinstance(inner.ctx, ast.Load) and inner.id in names and id(inner) not in outer:
                            bindings[id(inner)] = generator if isinstance(generator.target, ast.Name) else COMPREHENSION_UNPACKED
            unit.cache["comprehension-bindings"] = bindings
        return bindings.get(id(node))

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
                if enclosing_unit(unit) is not None:
                    # 套着的函数里没赋、不是参数的名字：外层函数的局部变量（闭包变量）或模块级的
                    return self.fold_name(name, enclosing_unit(unit), depth + 1)
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

    def single_assignment(self, name, unit):
        """name 只赋过一次时的那个值与它所在的单元：本函数（连同外层函数）里的局部赋值优先，没有再看模块级；赋过几次、拆包赋的 → None。"""
        scope = unit
        while scope is not None and isinstance(scope.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            assigned = self.local_assignments(scope).get(name)
            if assigned is not None:
                return (assigned[0], scope) if len(assigned) == 1 and assigned[0] is not None and not isinstance(assigned[0], ElementsOf) else None
            arguments = scope.node.args
            if name in [argument.arg for argument in arguments.posonlyargs + arguments.args + arguments.kwonlyargs]:
                return None
            scope = enclosing_unit(scope)
        candidates = [candidate for candidate in self.by_name.get(name, ()) if candidate.kind == "assign"]
        if len(candidates) != 1 or not isinstance(candidates[0].node, ast.Assign) or len(candidates[0].node.targets) != 1 \
                or not isinstance(candidates[0].node.targets[0], ast.Name):
            return None
        return candidates[0].node.value, candidates[0]

    def single_sequence(self, node, unit):
        """元组 / 列表字面量，或只赋过一次这样一个字面量的名字 → (字面量, 所在单元)；别的 → None。"""
        if isinstance(node, (ast.Tuple, ast.List)):
            return node, unit
        if isinstance(node, ast.Name):
            found = self.single_assignment(node.id, unit)
            if found is not None and isinstance(found[0], (ast.Tuple, ast.List)):
                return found
        return None

    def loaded_module_of(self, name, unit):
        """name 绑的是 importlib.util.module_from_spec(<spec>) 载入的模块、<spec> 是 spec_from_file_location(…, <路径>) 而路径折出恰好一份仓内 .py
        → 那份模块（PythonModule）；别的 → None。"""
        found = self.single_assignment(name, unit)
        if found is None or not isinstance(found[0], ast.Call) or self.canonical(found[0].func, found[1]) != "importlib.util.module_from_spec" \
                or len(found[0].args) != 1 or not isinstance(found[0].args[0], ast.Name):
            return None
        specification = self.single_assignment(found[0].args[0].id, found[1])
        if specification is None or not isinstance(specification[0], ast.Call) or \
                self.canonical(specification[0].func, specification[1]) != "importlib.util.spec_from_file_location":
            return None
        call, owner = specification
        location = call.args[1] if len(call.args) > 1 else next((keyword.value for keyword in call.keywords if keyword.arg == "location"), None)
        values = self.path_values(location, owner) if location is not None else None
        if not values or len(values) != 1:
            return None
        file = self.file_for_value(next(iter(values)), [posixpath.dirname(self.absolute), self.index.root])
        return self.index.module(file) if file is not None and file.endswith(".py") else None

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
        if isinstance(sequence, ast.Call) and depth <= 16 and "python-element-calls-unfolded" not in self.index.breaks:
            function = sequence.func
            if isinstance(function, ast.Attribute) and function.attr == "get" and isinstance(function.value, ast.Dict) and 1 <= len(sequence.args) <= 2 \
                    and not sequence.keywords:
                # {…}.get(键, 缺省)：键折不折得出都取遍字典里每个值的元素，连同缺省（没写缺省时取到 None、迭代就抛，不添元素）
                values = set()
                for part in list(function.value.values) + list(sequence.args[1:]):
                    elements = self.fold_elements(part, unit, depth + 1)
                    if elements is None:
                        return None
                    values |= elements
                return values
            if isinstance(function, ast.Name) and not sequence.keywords:
                # 调仓内模块级函数：它各处 return 的序列的元素
                return self.elements_of_call_returns(function.id, unit, depth)
            return None
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
            generator = self.comprehension_binding(node, unit)
            if generator is COMPREHENSION_UNPACKED:
                return None
            if generator is not None:
                # 推导式的循环变量：取遍它那一节迭代的元素
                return self.fold_elements(generator.iter, unit, depth + 1)
            return self.fold_name(node.id, unit, depth)
        if isinstance(node, ast.Attribute) and node.attr == "stdout" and isinstance(node.value, ast.Call):
            return self.git_output(node.value, unit)
        if isinstance(node, ast.Attribute) and isinstance(node.value, ast.Name) and "python-module-constants-unfolded" not in self.index.breaks:
            # import 进来的仓内模块的模块级常量：module.NAME 按那份模块里 NAME 的赋值折
            other = self.module_of_binding(self.binding(node.value.id, unit))
            if other is not None and other is not self:
                return other.fold_name(node.attr, other.module_body, depth + 1)
        if isinstance(node, ast.Attribute) and isinstance(node.value, ast.Name) and "python-loaded-module-constants-unfolded" not in self.index.breaks:
            # spec_from_file_location 载入的仓内模块的模块级常量：module.NAME 按那份模块里 NAME 的赋值折
            loaded = self.loaded_module_of(node.value.id, unit)
            if loaded is not None and loaded is not self:
                return loaded.fold_name(node.attr, loaded.module_body, depth + 1)
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
        if isinstance(node, ast.Subscript) and isinstance(node.slice, ast.Constant) and type(node.slice.value) is int and \
                "python-sequence-indexes-unfolded" not in self.index.breaks:
            # 只赋过一次的元组 / 列表字面量[整数]（模块级常量、本函数里的局部变量）：取那一项
            found = self.single_sequence(node.value, unit)
            if found is not None:
                literal, owner = found
                index = node.slice.value
                if -len(literal.elts) <= index < len(literal.elts) and not any(isinstance(element, ast.Starred) for element in literal.elts):
                    return self.fold(literal.elts[index], owner, depth + 1)
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

    def elements_of_call_returns(self, name, unit, depth):
        """调仓内模块级函数 name（本模块的，或 from 仓内模块 import 进来的）：它各处 return 的序列（fold_elements 认的写法）的元素的并；
        return / return None 不添元素；有一处折不出 → None。"""
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
        key = (module.path, id(callee), "<elements>")
        if key in self.index.folding:
            return None
        self.index.folding.add(key)
        try:
            values = set()
            for statement in walk_function_statements(callee.node):
                if isinstance(statement, ast.Return):
                    if statement.value is None or (isinstance(statement.value, ast.Constant) and statement.value.value is None):
                        continue
                    elements = module.fold_elements(statement.value, callee, depth + 1)
                    if elements is None:
                        return None
                    values |= elements
            return values
        finally:
            self.index.folding.discard(key)

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
            sites = self.function_call_sites(unit)
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

    def path_tails(self, node, unit, depth=0):
        """路径表达式 → 它可能取到的「折不出的前缀之后那一截字面尾巴」的集合（几处调用各给一种的取遍；整个折得出交它的取值）；
        尾巴认不出、或有一种取法一点字面尾巴都没有 → None。os.path.join / Path(…) / a / b / a + b / f 串、本函数里只赋过一次的名字、
        函数参数（调它的每一处给的实参）照认。"""
        if depth > 10 or node is None:
            return None
        folded = self.fold(node, unit)
        if folded is not None:
            return set(folded) if folded else None
        if isinstance(node, ast.Call) and not node.keywords and node.args and (
                self.canonical(node.func, unit) in ("os.path.join", "posixpath.join") or self.canonical(node.func, unit) in PATH_CONSTRUCTORS):
            suffixes = {""}
            for argument in reversed(node.args):
                value = self.fold(argument, unit)
                if value is None:
                    heads = self.path_tails(argument, unit, depth + 1) or {""}
                    joined = {posixpath.join(head, suffix) if suffix else head for head in heads for suffix in suffixes}
                    return joined if all(item.strip("/") for item in joined) and len(joined) <= FOLD_LIMIT else None
                suffixes = {posixpath.join(item, suffix) if suffix else item for item in value for suffix in suffixes}
                if len(suffixes) > FOLD_LIMIT:
                    return None
            return suffixes or None
        if isinstance(node, ast.BinOp) and isinstance(node.op, (ast.Add, ast.Div)):
            right = self.fold(node.right, unit)
            if right is None:
                return None
            heads = self.path_tails(node.left, unit, depth + 1) or {""}
            joined = {(head + value if isinstance(node.op, ast.Add) else posixpath.join(head, value) if head else value) for head in heads for value in right}
            return joined if all(item.strip("/") for item in joined) and len(joined) <= FOLD_LIMIT else None
        if isinstance(node, ast.JoinedStr):
            tail = self.literal_tail(node, unit)
            return {tail} if tail and tail.strip("/") else None
        if isinstance(node, ast.Name) and unit is not None and isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            assigned = self.local_assignments(unit).get(node.id)
            if assigned is not None:
                tails = set()
                for value in assigned:
                    if value is None or isinstance(value, ElementsOf):
                        return None
                    found = self.path_tails(value, unit, depth + 1)
                    if not found:
                        return None
                    tails |= found
                return tails if len(tails) <= FOLD_LIMIT else None
            arguments = unit.node.args
            if node.id in [argument.arg for argument in arguments.posonlyargs + arguments.args] and unit.container is None:
                position = [argument.arg for argument in arguments.posonlyargs + arguments.args].index(node.id)
                sites = self.function_call_sites(unit)
                if not sites:
                    return None
                tails = set()
                for call, caller in sites:
                    if any(isinstance(item, ast.Starred) for item in call.args):
                        return None
                    argument = call.args[position] if position < len(call.args) else next(
                        (keyword.value for keyword in call.keywords if keyword.arg == node.id), None)
                    found = self.path_tails(argument, caller, depth + 1) if argument is not None else None
                    if not found:
                        return None
                    tails |= found
                return tails if len(tails) <= FOLD_LIMIT else None
            if enclosing_unit(unit) is not None:
                return self.path_tails(node, enclosing_unit(unit), depth + 1)
        return None

    def script_tail_group(self, node, unit):
        """解释器后面的脚本路径折不出、只认得出字面尾巴（path_tails）时，仓里以这几截尾巴收尾的文件（绝对路径的集合；一份都没有交空集合）；
        尾巴认不出、只剩后缀（".py"）的 → None（照旧拒算）。"""
        tails = self.path_tails(node, unit)
        if not tails or any(not tail.strip("/") or posixpath.basename(tail.rstrip("/")).startswith(".") for tail in tails):
            return None
        files = set()
        for tail in tails:
            stripped = tail.strip("/")
            files |= {posixpath.join(self.index.root, relative) for relative in self.index.repository_files() if relative.endswith(stripped)}
        return files

    # 运行期读的数据文件（文件头「运行期读的数据文件」）

    def read_target(self, node, unit):
        """一次调用是不是在读文件：open / io.open / codecs.open（模式里没有 w a x +）、<路径>.read_text() / .read_bytes()、Path(…).open()（同样看模式）、
        shutil.copy / copy2 / copyfile 的源 → 被读的路径表达式；别的 → None。"""
        canonical = self.canonical(node.func, unit)
        keywords = {keyword.arg: keyword.value for keyword in node.keywords if keyword.arg}

        def writes(mode):
            return isinstance(mode, ast.Constant) and isinstance(mode.value, str) and any(letter in mode.value for letter in "wax+")
        if canonical in ("open", "io.open", "codecs.open"):
            path = node.args[0] if node.args else keywords.get("file", keywords.get("filename"))
            if writes(node.args[1] if len(node.args) > 1 else keywords.get("mode")):
                return None
            return path
        function = node.func
        if isinstance(function, ast.Attribute) and function.attr in ("read_text", "read_bytes") and not node.args:
            return function.value
        if isinstance(function, ast.Attribute) and function.attr == "open" and isinstance(function.value, ast.Call) and \
                self.canonical(function.value.func, unit) in PATH_CONSTRUCTORS:
            return None if writes(node.args[0] if node.args else keywords.get("mode")) else function.value
        if canonical in COPY_CALLS and node.args:
            return node.args[0]
        return None

    def data_read_origins(self, node, unit, depth=0):
        """data_read_origins_once 的结果按（节点, 作用域单元）记下：同一处实参经几条调用链问到只算一次；正在算的又问到自己（绕回来）交推不出（照调用方交进来的输入认）。"""
        return self.memoized("_read_origins", node, unit, lambda: self.data_read_origins_once(node, unit, depth), frozenset({("skip", "unknown")}))

    def memoized(self, table, node, unit, compute, cycle):
        """按（节点, 单元）记下 compute() 的结果（存在这份模块上的 table 里）；正在算的又问到自己交 cycle。"""
        memo = self.__dict__.setdefault(table, {})
        key = (id(node), id(unit))
        if key in memo:
            return cycle if memo[key] is MEMO_IN_PROGRESS else memo[key]
        memo[key] = MEMO_IN_PROGRESS
        try:
            memo[key] = compute()
        except BaseException:
            del memo[key]
            raise
        return memo[key]

    def data_read_origins_once(self, node, unit, depth=0):
        """被读的路径表达式 → 取法的集合：("file", 相对根的文件) 收进键；("skip", 为什么) 不收也不拒算（不在根里、临时目录、调用方交进来的输入）；
        ("directory", 相对根的目录) 那个目录整棵收进键；("param", 函数单元, 参数名) 是本模块里没有调用处的函数参数（收闭包时在闭包里按名字找调用处，
        找不到或是入口的算调用方的输入；ScriptClosure.resolve_parameter_reads）；("refuse",) 拒算 data-path-unfolded（文件头「运行期读的数据文件」三）。
        次序：整个折得出（path_values）→ 名字照赋值、参数照每一处调用的实参 → self.<属性> 照本类里的赋值 → 字面尾巴（path_tails：仓里以它收尾的文件整组收，
        一份都没有的不是仓里的文件）→ 前缀折得出的固定目录（fixed_directory_read）→ 折不出的前缀从哪来（head_origin）→ 都推不出的照调用方交进来的输入认。"""
        if depth > 10 or node is None:
            return frozenset({("skip", "unknown")})
        if isinstance(node, ast.Constant) and not isinstance(node.value, str):
            return frozenset({("skip", "descriptor")})
        values = self.path_values(node, unit)
        if values is not None:
            out = set()
            for value in values:
                expanded = os.path.expanduser(value) if value.startswith("~") else value
                file = self.file_for_value(expanded, [self.index.root, posixpath.dirname(self.absolute)])
                out.add(("file", file) if file is not None else ("skip", "not-a-repository-file"))
            return frozenset(out) or frozenset({("skip", "no-value")})
        if isinstance(node, ast.Name):
            aliased = self.name_read_origins(node.id, unit, depth)
            if aliased is not None:
                return aliased
        if isinstance(node, ast.Attribute) and "python-attribute-reads-unfolded" not in self.index.breaks:
            assigned = self.self_attribute_values(node, unit)
            if assigned:
                return frozenset().union(*(method.module.data_read_origins(value, method, depth + 1) for value, method in assigned))
        if isinstance(node, (ast.BoolOp, ast.IfExp)):
            # registration_file or os.path.join(<根>, "固定表")、a if c else b：每一边都可能是读的那一份，各认各的、并起来
            sides = node.values if isinstance(node, ast.BoolOp) else [node.body, node.orelse]
            return frozenset().union(*(self.data_read_origins(side, unit, depth + 1) for side in sides))
        tails = self.path_tails(node, unit)
        if tails and all(tail.strip("/") and not posixpath.basename(tail.rstrip("/")).startswith(".") for tail in tails):
            out = set()
            for tail in tails:
                stripped = tail.strip("/")
                matched = [relative for relative in self.index.repository_files() if relative == stripped or relative.endswith("/" + stripped)
                           or (not tail.startswith("/") and relative.endswith(stripped))]
                out |= {("file", relative) for relative in matched}
                if not matched:
                    out.add(("skip", "not-a-repository-file"))
            anchor = self.prefix_parameter(node, unit) if "python-tail-reads-unanchored" not in self.index.breaks else None
            if anchor is not None and any(origin[0] == "file" for origin in out):
                # 折不出的前缀是本函数的参数：仓里这一截尾巴的那一组留到收完闭包再按调用处认（ScriptClosure.tail_read_supplied）
                return frozenset({("param-tail", anchor[0], anchor[1], frozenset(origin[1] for origin in out if origin[0] == "file"))})
            return frozenset(out)
        directory = self.fixed_directory_read(node, unit, depth)
        if directory is not None:
            return directory
        origins = self.head_origin(node, unit, depth + 1)
        # 出处推不出、前缀也推不出的：照调用方交进来的输入认（文件头「运行期读的数据文件」二的末一种），不收也不拒算
        return frozenset(origins) if origins is not None else frozenset({("skip", "unknown")})

    def prefix_parameter(self, node, unit, depth=0):
        """被读路径最左那一截折不出的前缀是本函数的哪个参数（os.path.join / Path 的头一个实参、os.path.dirname 这类、x[…]、x / …、本函数里给它的赋值，
        赋值里引用它自己的那几处不算）→ (函数单元, 参数名)；推不到一个参数上 → None。"""
        if depth > 8 or node is None or unit is None or not isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            return None
        if isinstance(node, ast.Call) and node.args and (self.canonical(node.func, unit) in ("os.path.join", "posixpath.join")
                                                         or self.canonical(node.func, unit) in PATH_CONSTRUCTORS | PATH_TRANSFORMS):
            return self.prefix_parameter(node.args[0], unit, depth + 1)
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.func.attr in PATH_METHODS:
            return self.prefix_parameter(node.func.value, unit, depth + 1)
        if isinstance(node, ast.Subscript):
            return self.prefix_parameter(node.value, unit, depth + 1)
        if isinstance(node, ast.BinOp) and isinstance(node.op, (ast.Add, ast.Div)):
            return self.prefix_parameter(node.left, unit, depth + 1)
        if not isinstance(node, ast.Name):
            return None
        assigned = self.local_assignments(unit).get(node.id)
        arguments = unit.node.args
        parameter = node.id in [argument.arg for argument in arguments.posonlyargs + arguments.args + arguments.kwonlyargs]
        if assigned is None:
            return (unit, node.id) if parameter else None
        # 参数自己算一处（arguments = [a for a in arguments if …] 这种只引用自己的赋值不另算）
        anchors, anchor = ({(id(unit), node.id)}, (unit, node.id)) if parameter else (set(), None)
        for value in assigned:
            if value is not None and not isinstance(value, ElementsOf) and any(
                    isinstance(inner, ast.Name) and inner.id == node.id for inner in ast.walk(value)):
                continue
            found = self.prefix_parameter(value, unit, depth + 1) if value is not None and not isinstance(value, ElementsOf) else None
            if found is None:
                return None
            anchors.add((id(found[0]), found[1]))
            anchor = found
        return anchor if len(anchors) == 1 else None

    def fixed_directory_read(self, node, unit, depth):
        """os.path.join(<前缀>, <折不出的几截>)（文件头「运行期读的数据文件」一、二、三）：前缀是开头折得出的那几个实参，折不出的几截看出处
        （unfolded_part_origins）：出处都推得出 → 那几样出处（调用方的输入、参数）；推不出 → 前缀是根里的目录交 ("directory", 相对根的目录)、
        就是根本身交 ("refuse",)、在根外交 ("skip", "outside")。不是这种写法、前缀一截都折不出 → None。"""
        if not (isinstance(node, ast.Call) and not node.keywords and len(node.args) >= 2
                and self.canonical(node.func, unit) in ("os.path.join", "posixpath.join")):
            return None
        folded = 0
        while folded < len(node.args) and self.fold(node.args[folded], unit) is not None:
            folded += 1
        if folded == 0 or folded == len(node.args):
            return None
        prefix = ast.Call(func=node.func, args=list(node.args[:folded]), keywords=[]) if folded > 1 else node.args[0]
        directories = self.path_values(prefix, unit, directory=True)
        if not directories:
            return None
        origins = set()
        for part in node.args[folded:]:
            found = self.unfolded_part_origins(part, unit, depth + 1)
            if found is None:
                origins = None
                break
            origins |= found
        if origins is not None:
            return frozenset(origins)
        out = set()
        for directory in directories:
            absolute = directory if posixpath.isabs(directory) else posixpath.join(self.index.root, directory)
            relative = self.index.relative(posixpath.normpath(absolute))
            if relative is None:
                out.add(("skip", "outside"))
            elif relative in ("", "."):
                # 根本身 + 折不出的一截：那一截是读根里的文件得来的（登记行点名的一份）才拒算；清单遍历、算出来的名单、参数给的是被判的输入，不收不拒算
                derived = "python-root-joins-refused" in self.index.breaks or any(
                    self.repository_content_derived(part, unit, depth + 1) for part in node.args[folded:])
                out.add(("refuse",) if derived else ("skip", "listing"))
            elif "python-directory-reads-refused" in self.index.breaks:
                out.add(("refuse",))
            elif os.path.isdir(absolute):
                out.add(("directory", relative))
            else:
                out.add(("skip", "not-a-repository-file"))
        return frozenset(out)

    def repository_content_derived(self, node, unit, depth=0, seen=None):
        """路径里折不出的那一截是不是读根里的文件得来的（文件头「运行期读的数据文件」三）：沿名字往回找来处（name_sources）——碰到一处读文件
        （read_target 认得出的读，拷贝不算）、它的取法里有根里的文件或目录，就是。参数、调用的返回值、认不出的都不算（照被判的输入认）。"""
        seen = set() if seen is None else seen
        if depth > 10 or node is None:
            return False
        for inner in ast.walk(node):
            if isinstance(inner, ast.Call):
                target = self.read_target(inner, unit) if self.canonical(inner.func, unit) not in COPY_CALLS else None
                if target is not None and any(origin[0] in ("file", "directory", "param-tail") for origin in self.data_read_origins(target, unit, depth + 1)):
                    return True
            elif isinstance(inner, ast.Name) and isinstance(inner.ctx, ast.Load) and inner.id not in seen:
                seen.add(inner.id)
                if any(self.repository_content_derived(source, unit, depth + 1, seen) for source in self.name_sources(inner.id, unit)):
                    return True
        return False

    def name_sources(self, name, unit):
        """名字在 unit 里的来处（repository_content_derived 用）：赋值的右边（拆包赋的取整个右边）、for 与推导式里绑它的迭代对象、
        with … as 它的那个对象；单元里一处都没有的取模块级只赋过一次的值。"""
        sources = []
        for node in walk_nodes(unit.nodes or []) if unit is not None else ():
            if isinstance(node, ast.Assign) and any(name in bound_names(target) for target in node.targets):
                sources.append(node.value)
            elif isinstance(node, (ast.AnnAssign, ast.AugAssign, ast.NamedExpr)) and node.value is not None and name in bound_names(node.target):
                sources.append(node.value)
            elif isinstance(node, (ast.For, ast.AsyncFor, ast.comprehension)) and name in bound_names(node.target):
                sources.append(node.iter)
            elif isinstance(node, (ast.With, ast.AsyncWith)):
                sources += [item.context_expr for item in node.items if item.optional_vars is not None and name in bound_names(item.optional_vars)]
        if not sources:
            found = self.single_assignment(name, None)
            if found is not None:
                sources.append(found[0])
        return sources

    def unfolded_part_origins(self, node, unit, depth=0):
        """路径或命令里折不出的那一截 → 它里头每一处折不出的东西的出处（head_origin 的取法）；有一处推不出交 None。
        os.path.join、Path(…)、a + b、a / b、f 串逐段拆开，折得出的那几段不算。"""
        if depth > 12 or node is None:
            return None
        if self.fold(node, unit) is not None:
            return set()
        parts = None
        if isinstance(node, ast.Call) and not node.keywords and node.args and (
                self.canonical(node.func, unit) in ("os.path.join", "posixpath.join") or self.canonical(node.func, unit) in PATH_CONSTRUCTORS):
            parts = list(node.args)
        elif isinstance(node, ast.BinOp) and isinstance(node.op, (ast.Add, ast.Div)):
            parts = [node.left, node.right]
        elif isinstance(node, ast.JoinedStr):
            parts = [part.value for part in node.values if isinstance(part, ast.FormattedValue)]
        if parts is None:
            if isinstance(node, ast.Name) and unit is not None and isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                arguments = unit.node.args
                if node.id in [argument.arg for argument in arguments.posonlyargs + arguments.args + arguments.kwonlyargs] \
                        and node.id not in self.local_assignments(unit):
                    # 参数：给它实参的是哪几处，收完闭包再按闭包里的调用处认（入口的参数算调用方交进来的；ScriptClosure.launch_supplied_by_caller）
                    return {("param", unit, node.id)}
            if isinstance(node, ast.Name):
                # 只赋过一次的名字（key = f".claude/gate.d/{stage}" 一类）：照它的值逐段拆
                found = self.single_assignment(node.id, unit)
                if found is not None:
                    return self.unfolded_part_origins(found[0], found[1], depth + 1)
            return self.head_origin(node, unit, depth + 1)
        out = set()
        for part in parts:
            found = self.unfolded_part_origins(part, unit, depth + 1)
            if found is None:
                return None
            out |= found
        return out

    def self_attribute_values(self, node, unit):
        """self.<属性>（方法里）→ 本类方法里给 self.<属性> 的每一处赋值 [(值节点, 那个方法的单元)]（拆包赋的逐个对上）；
        不是这种写法、一处赋值都找不到、有一处对不上 → []。弄坏开关 python-attribute-reads-unfolded 下一律交 []。"""
        if "python-attribute-reads-unfolded" in self.index.breaks or not (
                isinstance(node, ast.Attribute) and isinstance(node.value, ast.Name) and node.value.id == "self"):
            return []
        scope = unit
        while scope is not None and enclosing_unit(scope) is not None:
            scope = enclosing_unit(scope)
        owner = scope.container if scope is not None else None
        if owner is None:
            return []
        out = []
        for method in self.methods:
            if method.container is not owner or not isinstance(method.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                continue
            for statement in walk_function_statements(method.node):
                if not isinstance(statement, ast.Assign):
                    continue
                for target in statement.targets:
                    if isinstance(target, ast.Attribute) and isinstance(target.value, ast.Name) and target.value.id == "self" and target.attr == node.attr:
                        out.append((statement.value, method))
                    elif isinstance(target, (ast.Tuple, ast.List)) and any(
                            isinstance(item, ast.Attribute) and isinstance(item.value, ast.Name) and item.value.id == "self" and item.attr == node.attr
                            for item in target.elts):
                        if not isinstance(statement.value, (ast.Tuple, ast.List)) or len(statement.value.elts) != len(target.elts):
                            return []
                        out += [(value, method) for item, value in zip(target.elts, statement.value.elts)
                                if isinstance(item, ast.Attribute) and isinstance(item.value, ast.Name) and item.value.id == "self" and item.attr == node.attr]
        return out

    def name_read_origins(self, name, unit, depth):
        """被读的路径就是一个名字：本函数（连同外层）里的赋值逐个照 data_read_origins 认、with tempfile.… as 名字算临时目录、
        参数照每一处调用的实参认（没有调用处交 ("param", 函数单元)）、模块级只赋过一次的照那个值认；认不出 → None（交给尾巴与前缀那两步）。"""
        if unit is None or not isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            guarded = guard_assignments(name, unit)
            if guarded:
                return frozenset().union(*(self.data_read_origins(value, unit, depth + 1) for value in guarded))
            found = self.single_assignment(name, None)
            return self.data_read_origins(found[0], found[1], depth + 1) if found is not None else None
        if self.bound_to_temporary(name, unit):
            return frozenset({("skip", "temporary")})
        assigned = self.local_assignments(unit).get(name)
        if assigned is not None:
            out = set()
            for value in assigned:
                if value is None or isinstance(value, ElementsOf):
                    return None
                out |= self.data_read_origins(value, unit, depth + 1)
            return frozenset(out)
        arguments = unit.node.args
        if name in [argument.arg for argument in arguments.posonlyargs + arguments.args + arguments.kwonlyargs]:
            sites = self.parameter_sites(unit, name)
            if sites is None:
                return None
            if not sites:
                return frozenset({("param", unit, name)})
            out = set()
            for argument, caller in sites:
                out |= caller.module.data_read_origins(argument, caller, depth + 1) if caller is not None and caller.module is not None \
                    else self.data_read_origins(argument, caller, depth + 1)
            return frozenset(out)
        if enclosing_unit(unit) is not None:
            return self.name_read_origins(name, enclosing_unit(unit), depth + 1)
        found = self.single_assignment(name, None)
        return self.data_read_origins(found[0], found[1], depth + 1) if found is not None else None

    def parameter_sites(self, unit, name):
        """函数（方法、套着的函数）参数 name 在每一处调用里拿到的实参 [(实参节点, 调用处的作用域单元)]：没给的取缺省值；
        一处都没有交 []；有一处带 * / ** 认不出对哪个参数 → None。"""
        function = unit.node
        arguments = function.args
        positional = [argument.arg for argument in arguments.posonlyargs + arguments.args]
        method = unit.container is not None
        if method and not any(isinstance(decorator, ast.Name) and decorator.id == "staticmethod" for decorator in function.decorator_list):
            if not positional or positional[0] == name:
                return None
            positional = positional[1:]
        defaults = dict(zip(positional[len(positional) - len(arguments.defaults):], arguments.defaults))
        defaults.update({argument.arg: default for argument, default in zip(arguments.kwonlyargs, arguments.kw_defaults) if default is not None})
        position = positional.index(name) if name in positional else None
        out = []
        for call, caller in (self.method_call_sites(unit) if method else self.function_call_sites(unit)):
            if any(isinstance(item, ast.Starred) for item in call.args) or any(keyword.arg is None for keyword in call.keywords):
                return None
            argument = call.args[position] if position is not None and position < len(call.args) else None
            argument = next((keyword.value for keyword in call.keywords if keyword.arg == name), argument)
            if argument is None:
                argument, caller = defaults.get(name), unit
            if argument is None:
                return None
            out.append((argument, caller))
        return out

    def bound_to_temporary(self, name, unit):
        """name 在这个函数里绑的是新建的临时目录或临时文件：with tempfile.TemporaryDirectory() / NamedTemporaryFile() as name、
        <描述符>, name = tempfile.mkstemp()。"""
        for statement in walk_function_statements(unit.node):
            if isinstance(statement, (ast.With, ast.AsyncWith)):
                for item in statement.items:
                    if isinstance(item.optional_vars, ast.Name) and item.optional_vars.id == name and isinstance(item.context_expr, ast.Call) \
                            and self.canonical(item.context_expr.func, unit) in TEMPORARY_CALLS:
                        return True
            elif isinstance(statement, ast.Assign) and isinstance(statement.value, ast.Call) and \
                    self.canonical(statement.value.func, unit) in TEMPORARY_CALLS and \
                    any(name in bound_names(target) for target in statement.targets if isinstance(target, (ast.Tuple, ast.List))):
                return True
        return False

    def outside_prefix(self, prefix):
        """一段字面前缀（f"/proc/{pid}/stat" 的 "/proc/"）落在根外：是绝对路径，目录那一截不在根里、根也不在它底下。"""
        if not prefix.startswith("/"):
            return False
        directory = posixpath.normpath(prefix.rsplit("/", 1)[0] or "/")
        if directory == "/":
            return False
        return self.index.relative(directory) is None and not (self.index.root + "/").startswith(directory + "/")

    def head_origin(self, node, unit, depth):
        """head_origin_once 的结果按（节点, 作用域单元）记下（同 data_read_origins）。"""
        return self.memoized("_head_origins", node, unit, lambda: self.head_origin_once(node, unit, depth), None)

    def head_origin_once(self, node, unit, depth):
        """被读路径里折不出的那一截前缀从哪来 → {("skip", "temporary" | "input" | "environment" | "outside")、("param", 函数单元, 参数名)…}；认不出 → None。
        临时目录：tempfile 的 mkdtemp / gettempdir / TemporaryDirectory / NamedTemporaryFile / mkstemp 新建的（读的是这一趟自己写的）；
        输入：sys.argv、argparse 解出来的属性（调用方交进来的，判定的请求行管它们）；环境变量另记 ("skip", "environment")（读文件时同输入，
        起子进程时不算调用方交的命令）；根外：字面的绝对路径前缀不在根里（/proc/…）；self.<属性> 照本类里给它的赋值认；没有调用处的参数交 ("param", 函数单元, 参数名)。"""
        if depth > 12 or node is None:
            return None
        breaks = self.index.breaks
        if isinstance(node, ast.Call):
            canonical = self.canonical(node.func, unit)
            if canonical in TEMPORARY_CALLS:
                return {("skip", "temporary")}
            if canonical in ("os.environ.get", "os.getenv"):
                return {("skip", "environment")}
            if (canonical in ("os.path.join", "posixpath.join") or canonical in PATH_CONSTRUCTORS or canonical in PATH_TRANSFORMS) and node.args:
                return self.head_origin(node.args[0], unit, depth + 1)
            if isinstance(node.func, ast.Attribute) and node.func.attr in PATH_METHODS:
                return self.head_origin(node.func.value, unit, depth + 1)
            if isinstance(node.func, ast.Name) and self.binding(node.func.id, unit) is None:
                candidates = [candidate for candidate in self.by_name.get(node.func.id, ()) if candidate.kind == "def" and candidate.container is None]
                returns = [statement.value for statement in walk_function_statements(candidates[0].node)
                           if isinstance(statement, ast.Return) and statement.value is not None] if len(candidates) == 1 else []
                if not returns:
                    return None
                out = set()
                for value in returns:
                    found = candidates[0].module.head_origin(value, candidates[0], depth + 1)
                    if found is None:
                        return None
                    out |= found
                return out
            return None
        if isinstance(node, ast.JoinedStr):
            prefix = ""
            for part in node.values:
                if isinstance(part, ast.Constant):
                    prefix += str(part.value)
                    continue
                folded = self.fold(part.value, unit) if isinstance(part, ast.FormattedValue) else None
                if folded is not None and len(folded) == 1:
                    prefix += next(iter(folded))
                    continue
                if self.outside_prefix(prefix):
                    return {("skip", "outside")}
                return self.head_origin(part.value, unit, depth + 1) if not prefix and isinstance(part, ast.FormattedValue) else None
            return None
        if isinstance(node, ast.BinOp) and isinstance(node.op, (ast.Add, ast.Div)):
            left = self.fold(node.left, unit)
            if left is None:
                return self.head_origin(node.left, unit, depth + 1)
            return {("skip", "outside")} if left and all(self.outside_prefix(value + "/") for value in left) else None
        if isinstance(node, ast.Attribute):
            if node.attr in ("parent", "name", "stem"):
                return self.head_origin(node.value, unit, depth + 1)
            if isinstance(node.value, ast.Name):
                found = self.single_assignment(node.value.id, unit)
                if found is not None and isinstance(found[0], ast.Call) and isinstance(found[0].func, ast.Attribute) and \
                        found[0].func.attr in ("parse_args", "parse_known_args"):
                    # 弄坏开关 python-parsed-arguments-unread：argparse 解出来的属性照判法自己的数据认，拒算
                    return {("refuse",)} if "python-parsed-arguments-unread" in breaks else {("skip", "input")}
            assigned = self.self_attribute_values(node, unit)
            if assigned:
                out = set()
                for value, method in assigned:
                    found = method.module.head_origin(value, method, depth + 1)
                    if found is None:
                        return None
                    out |= found
                return out
            return None
        if isinstance(node, ast.Subscript):
            if self.canonical(node.value, unit) == "sys.argv":
                return {("skip", "input")}
            if self.canonical(node.value, unit) == "os.environ":
                return {("skip", "environment")}
            return self.head_origin(node.value, unit, depth + 1)
        if isinstance(node, ast.Name):
            if self.canonical(node, unit) == "sys.argv":
                return {("skip", "input")}
            if unit is not None and isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                if self.bound_to_temporary(node.id, unit):
                    return {("skip", "temporary")}
                assigned = self.local_assignments(unit).get(node.id)
                if assigned is not None:
                    out = set()
                    for value in assigned:
                        source = value.sequence if isinstance(value, ElementsOf) else value
                        found = self.head_origin(source, unit, depth + 1) if source is not None else None
                        if found is None:
                            return None
                        out |= found
                    return out
                arguments = unit.node.args
                if node.id in [argument.arg for argument in arguments.posonlyargs + arguments.args + arguments.kwonlyargs]:
                    sites = self.parameter_sites(unit, node.id)
                    if sites is None:
                        return None
                    if not sites:
                        return {("param", unit, node.id)}
                    out = set()
                    for argument, caller in sites:
                        found = caller.module.head_origin(argument, caller, depth + 1)
                        if found is None:
                            return None
                        out |= found
                    return out
                if enclosing_unit(unit) is not None:
                    return self.head_origin(node, enclosing_unit(unit), depth + 1)
            guarded = guard_assignments(node.id, unit)
            if guarded:
                out = set()
                for value in guarded:
                    found = self.head_origin(value, unit, depth + 1)
                    if found is None:
                        return None
                    out |= found
                return out
            found = self.single_assignment(node.id, None)
            if found is not None:
                return self.head_origin(found[0], found[1], depth + 1)
        return None

    # 单元读到的东西

    def facts(self, unit):
        if unit.facts is not None:
            return unit.facts
        facts = PythonFacts()
        unit.facts = facts
        if unit.data or self.tree is None:
            return facts
        return self.collect_facts(unit, facts, frozenset(), ())

    def view_facts(self, unit, view):
        """程序入口带参数（文件头「程序入口带参数」）：照 view（已知值表，frozenset 的 (名字, 值)）剪过的这个单元读到的东西；同一单元同一张表只算一次。"""
        cache = self.__dict__.setdefault("_view_facts", {})
        key = (id(unit), view)
        if key not in cache:
            facts = PythonFacts()
            if not unit.data and self.tree is not None:
                skipped, extra, calls, tables = self.narrowing_plan(unit, dict(view))
                self.collect_facts(unit, facts, skipped, extra)
                facts.narrowed_calls, facts.shallow_tables = calls, tables
            cache[key] = facts
        return cache[key]

    def collect_facts(self, unit, facts, skipped, extra):
        """单元读到的东西填进 facts、交回它：skipped（按 id）里的节点连同底下的不看，extra 是另外要看的节点（分派表取到的那一项）。"""
        breaks = self.index.breaks
        nodes = list(walk_nodes_skipping(unit.nodes or [], skipped)) + list(walk_nodes(extra)) if skipped or extra else list(walk_nodes(unit.nodes or []))
        if unit.kind == "import":
            for name, binding in self.local_import_bindings(unit).items():
                if name == unit.name or unit.name == "*":
                    facts.import_targets += self.binding_targets(binding, unit, facts)
        # 只查询固定的 __file__ 与直接读 __file__ 一样，不会按字符串挑到任意函数。
        # 仍遍历外层表达式；别的 globals()（动态键、传参、默认值）照旧拒算。
        file_queries = set()
        if self.binding("globals", unit) is None:
            for query in nodes:
                if (isinstance(query, ast.Call) and isinstance(query.func, ast.Attribute) and query.func.attr == "get"
                        and len(query.args) == 1 and not query.keywords
                        and isinstance(query.args[0], ast.Constant) and query.args[0].value == "__file__"):
                    receiver = query.func.value
                    if (isinstance(receiver, ast.Call) and isinstance(receiver.func, ast.Name) and receiver.func.id == "globals"
                            and not receiver.args and not receiver.keywords):
                        file_queries.add(id(receiver))
            if file_queries:
                facts.names.add("__file__")
        dropped = set()
        if "python-dispatch-tables-ignored" in breaks:
            # 弄坏开关：分派表（字典字面量）里当值写的函数名不连边
            dropped = {id(value) for node in nodes if isinstance(node, ast.Dict)
                       for value in node.values if isinstance(value, ast.Name)}
        for node in nodes:
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
                if id(node) not in file_queries:
                    # 套在单元里的函数里的调用：名字照那一层作用域折（scope_of）
                    self.call_facts(node, self.scope_of(node, unit), facts, breaks)
            elif isinstance(node, ast.Constant) and isinstance(node.value, str):
                facts.strings.add(node.value)
            elif isinstance(node, ast.Assign) and any(isinstance(target, ast.Subscript) and self.canonical(target.value, unit) == "sys.path"
                                                      for target in node.targets):
                elements = list(node.value.elts) if isinstance(node.value, (ast.List, ast.Tuple)) else [None]
                if any(element is None or self.path_values(element, unit, directory=True) is None for element in elements):
                    self.refuse(facts, "sys-path-unfolded", node)
        return facts

    # 程序入口带参数（文件头「程序入口带参数」）

    def narrowing_plan(self, unit, bindings):
        """照已知值表剪这个单元：交 (要跳过的节点 id 集合, 分派表取到的那几项的值节点, 带着已知实参往下剪的调用 [(函数单元, 已知值表)],
        只问键在不在的字典字面量的赋值单元)。只剪函数体与 main-guard；套在里面的函数不剪（那一层的名字另有绑定）。"""
        skipped, extra, calls, tables = set(), [], [], []
        if isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            self.prune_statements(unit.node.body, unit, bindings, skipped)
        elif unit.kind == "main-guard":
            for guard in unit.nodes or []:
                if isinstance(guard, ast.If):
                    self.prune_statements(guard.body, unit, bindings, skipped)
        else:
            return skipped, extra, calls, tables
        for node in walk_nodes_skipping(unit.nodes or [], skipped):
            if self.scope_of(node, unit) is not unit:
                continue
            if isinstance(node, ast.Compare) and len(node.ops) == 1 and isinstance(node.ops[0], (ast.In, ast.NotIn)) \
                    and isinstance(node.comparators[0], ast.Name) and "python-membership-tables-followed" not in self.index.breaks:
                # <词> in 表：只用到表的键，表的赋值照 SHALLOW_IMPORT_VIEW 收、值里写的函数名不连（文件头「程序入口带参数」分派表）
                found = self.module_dictionary(node.comparators[0].id, unit)
                if found is not None:
                    skipped.add(id(node.comparators[0]))
                    tables.append(found[1])
            elif isinstance(node, ast.Call):
                callee = None
                if isinstance(node.func, ast.Name):
                    callee = self.narrowable_callee(node.func.id, unit)
                elif isinstance(node.func, ast.Subscript):
                    entry = self.table_entry(node.func, unit, bindings)
                    if entry is not None and isinstance(entry[1], ast.Name):
                        callee = self.narrowable_callee(entry[1].id, None)
                view = self.call_view(callee, node, unit, bindings) if callee is not None else None
                if view is not None:
                    skipped.add(id(node.func))   # 被调的那一个按 view 另收（ScriptClosure.python_edges），名字不当普通引用
                    calls.append((callee, view))
            elif isinstance(node, ast.Subscript) and isinstance(node.ctx, ast.Load):
                entry = self.table_entry(node, unit, bindings)
                if entry is not None:
                    skipped.add(id(node.value))   # 不连整张表，只连取到的那一项
                    if entry[1] is not None:
                        extra.append(entry[1])
        return skipped, extra, calls, tables

    def prune_statements(self, statements, unit, bindings, skipped):
        """一串语句里判得出走不到的记进 skipped；交这一串是不是一定在这里收尾（return、raise、sys.exit、os._exit，或两支都收尾的 if）。"""
        reachable, ends = True, False
        for statement in statements:
            if not reachable:
                skipped.add(id(statement))
                continue
            if isinstance(statement, ast.If):
                decided = self.known_condition(statement.test, unit, bindings)
                if decided is True:
                    skipped.update(id(item) for item in statement.orelse)
                    branch_ends = self.prune_statements(statement.body, unit, bindings, skipped)
                elif decided is False:
                    skipped.update(id(item) for item in statement.body)
                    branch_ends = self.prune_statements(statement.orelse, unit, bindings, skipped)
                else:
                    body_ends = self.prune_statements(statement.body, unit, bindings, skipped)
                    branch_ends = self.prune_statements(statement.orelse, unit, bindings, skipped) and body_ends
                if branch_ends:
                    reachable, ends = False, True
            elif isinstance(statement, (ast.Return, ast.Raise)) or (
                    isinstance(statement, ast.Expr) and isinstance(statement.value, ast.Call)
                    and self.canonical(statement.value.func, unit) in ("sys.exit", "os._exit")):
                reachable, ends = False, True
            elif isinstance(statement, (ast.For, ast.AsyncFor, ast.While)):
                self.prune_statements(statement.body, unit, bindings, skipped)
                self.prune_statements(statement.orelse, unit, bindings, skipped)
            elif isinstance(statement, (ast.With, ast.AsyncWith)):
                if self.prune_statements(statement.body, unit, bindings, skipped):
                    reachable, ends = False, True
            elif isinstance(statement, ast.Try):
                self.prune_statements(statement.body, unit, bindings, skipped)
                for handler in statement.handlers:
                    self.prune_statements(handler.body, unit, bindings, skipped)
                self.prune_statements(statement.orelse, unit, bindings, skipped)
                self.prune_statements(statement.finalbody, unit, bindings, skipped)
        return ends

    def known_value(self, node, unit, bindings, depth=0):
        """程序入口带参数时一个表达式的已知值：串、KnownWords；不知道交 None。"""
        if depth > 8 or node is None:
            return None
        if isinstance(node, ast.Constant):
            return node.value if isinstance(node.value, str) else None
        if isinstance(node, (ast.List, ast.Tuple)):
            if any(isinstance(element, ast.Starred) for element in node.elts):
                return None
            values = [self.known_value(element, unit, bindings, depth + 1) for element in node.elts]
            return KnownWords(tuple(value if isinstance(value, str) else None for value in values), True)
        if isinstance(node, (ast.Name, ast.Attribute)) and self.canonical(node, unit) == "sys.argv":
            return bindings.get(ARGV_BINDING)
        if isinstance(node, ast.Name):
            if node.id in bindings:
                return bindings[node.id]
            if unit is not None and unit.kind == "main-guard":
                assigned = guard_assignments(node.id, unit)
            elif unit is not None and isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                assigned = self.local_assignments(unit).get(node.id) or []
            else:
                assigned = []
            if len(assigned) == 1 and assigned[0] is not None and not isinstance(assigned[0], ElementsOf):
                return self.known_value(assigned[0], unit, bindings, depth + 1)
            return None
        if isinstance(node, ast.Subscript):
            base = self.known_value(node.value, unit, bindings, depth + 1)
            if not isinstance(base, KnownWords):
                return None
            index = node.slice
            if isinstance(index, ast.Slice):
                lower = 0 if index.lower is None else literal_integer(index.lower)
                upper = None if index.upper is None else literal_integer(index.upper)
                if index.step is not None or lower is False or upper is False:
                    return None
                return base.sliced(lower, upper)
            if isinstance(index, ast.Constant) and type(index.value) is int:
                return base.index(index.value)
        return None

    def known_members(self, node, unit, bindings):
        """in 右边的已知成员：元素都是字面串的列表、元组、集合，键都是字面串的字典字面量（本模块只赋过一次的也算），长度知道、词都知道的一串；判不出交 None。"""
        if isinstance(node, (ast.List, ast.Tuple, ast.Set)):
            if all(isinstance(element, ast.Constant) and isinstance(element.value, str) for element in node.elts):
                return {element.value for element in node.elts}
            return None
        keys = None
        if isinstance(node, ast.Dict):
            keys = node.keys
        elif isinstance(node, ast.Name):
            found = self.module_dictionary(node.id, unit)
            keys = found[0].keys if found is not None else None
        if keys is None:
            value = self.known_value(node, unit, bindings)
            if isinstance(value, KnownWords) and value.exact and None not in value.words:
                return set(value.words)
            return None
        if all(isinstance(key, ast.Constant) and isinstance(key.value, str) for key in keys):
            return {key.value for key in keys}
        return None

    def known_condition(self, test, unit, bindings):
        """if 的条件照已知值判：True / False；判不出交 None。"""
        if isinstance(test, ast.BoolOp):
            results = [self.known_condition(value, unit, bindings) for value in test.values]
            if isinstance(test.op, ast.And):
                return False if False in results else (True if all(result is True for result in results) else None)
            return True if True in results else (False if all(result is False for result in results) else None)
        if isinstance(test, ast.UnaryOp) and isinstance(test.op, ast.Not):
            result = self.known_condition(test.operand, unit, bindings)
            return None if result is None else not result
        if isinstance(test, ast.Compare) and len(test.ops) == 1:
            operator, right = test.ops[0], test.comparators[0]
            if isinstance(operator, (ast.Eq, ast.NotEq)):
                equal = known_equal(self.known_value(test.left, unit, bindings), self.known_value(right, unit, bindings))
                return None if equal is None else (equal if isinstance(operator, ast.Eq) else not equal)
            if isinstance(operator, (ast.In, ast.NotIn)):
                left = self.known_value(test.left, unit, bindings)
                members = self.known_members(right, unit, bindings) if isinstance(left, str) else None
                return None if members is None else ((left in members) == isinstance(operator, ast.In))
            return None
        value = self.known_value(test, unit, bindings)
        if isinstance(value, str):
            return bool(value)
        if isinstance(value, KnownWords):
            return True if value.words else (False if value.exact else None)
        return None

    def table_entry(self, subscript, unit, bindings):
        """表[已知的串]，表是本模块只赋过一次、键都是字面串的字典字面量：交 (表的赋值单元, 那一项的值节点；表里没有这个键是 None)；别的交 None。"""
        if not isinstance(subscript.value, ast.Name):
            return None
        key = self.known_value(subscript.slice, unit, bindings)
        if not isinstance(key, str):
            return None
        found = self.module_dictionary(subscript.value.id, unit)
        if found is None or not all(isinstance(item, ast.Constant) and isinstance(item.value, str) for item in found[0].keys):
            return None
        dictionary, assignment = found
        return assignment, next((value for item, value in zip(dictionary.keys, dictionary.values) if item.value == key), None)

    def narrowable_callee(self, name, unit):
        """本模块里恰好一个、调用处没另绑这个名字的模块级函数 → 它的单元；别的 → None。"""
        if unit is not None and (self.local_bindings(unit).get(name) is not None or (
                isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)) and name in self.local_assignments(unit))):
            return None
        candidates = [candidate for candidate in self.by_name.get(name, ()) if candidate.kind == "def" and candidate.container is None
                      and isinstance(candidate.node, (ast.FunctionDef, ast.AsyncFunctionDef))]
        return candidates[0] if len(candidates) == 1 and name not in self.body_defines else None

    def call_view(self, callee, call, unit, bindings):
        """调 callee 的这一处带了哪些已知实参 → 已知值表（frozenset 的 (参数名, 值)）；一个都没有、带 * / ** 的交 None；callee 里给那个参数另赋过值的不带。"""
        if any(isinstance(item, ast.Starred) for item in call.args) or any(keyword.arg is None for keyword in call.keywords):
            return None
        arguments = callee.node.args
        positional = [argument.arg for argument in arguments.posonlyargs + arguments.args]
        named = set(positional) | {argument.arg for argument in arguments.kwonlyargs}
        reassigned = callee.module.local_assignments(callee)
        pairs = list(zip(positional, call.args)) + [(keyword.arg, keyword.value) for keyword in call.keywords if keyword.arg in named]
        known = [(name, value) for name, value in ((name, self.known_value(argument, unit, bindings)) for name, argument in pairs)
                 if value is not None and name not in reassigned]
        return frozenset(known) if known else None

    def table_reference_names(self, unit):
        """import 时执行的赋值（文件头「程序入口带参数」）：值里只当字典值、列表、元组、集合元素写的名字（import 时不调它们）；别的单元交空集合。"""
        if unit.cache is None:
            unit.cache = {}
        if "table_references" not in unit.cache:
            elements, others = set(), set()

            def visit(node, element):
                if isinstance(node, ast.Name):
                    (elements if element else others).add(node.id)
                elif isinstance(node, ast.Dict):
                    for key in node.keys:
                        if key is not None:
                            visit(key, False)
                    for value in node.values:
                        visit(value, True)
                elif isinstance(node, (ast.List, ast.Tuple, ast.Set)):
                    for item in node.elts:
                        visit(item, True)
                else:
                    for child in ast.iter_child_nodes(node):
                        visit(child, False)
            if unit.kind == "assign" and isinstance(unit.node, (ast.Assign, ast.AnnAssign)) and unit.node.value is not None:
                visit(unit.node.value, False)
            unit.cache["table_references"] = frozenset(elements - others)
        return unit.cache["table_references"]

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
        if "python-data-reads-ignored" not in breaks:
            # 运行期读的数据文件（文件头「运行期读的数据文件」）：取法在闭包里落成数据单元、跳过或拒算（ScriptClosure.python_data_reads）
            target = self.read_target(node, unit)
            if target is not None:
                # 拷文件（COPY_CALLS）的源另记一笔：折不出的那一截不拒算（文件头「运行期读的数据文件」二）
                copied = self.canonical(node.func, unit) in COPY_CALLS and "python-root-joins-refused" not in breaks
                facts.data_reads.append((self.data_read_origins(target, unit), node.lineno, copied))

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
                    self.refuse_launch(facts, INLINE_FORM, node, (command, self, unit))
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
            self.refuse_launch(facts, COMMAND_FORM, node, (command, self, unit))
            facts.handled.append(("command-unfolded", node.lineno))
            return
        for elements in alternatives:
            self.command_links(elements, unit, node, facts, keywords)

    def refuse_launch(self, facts, form, node, item):
        """起子进程那一处折不出（文件头「调用方交进来的命令」）：item 是折不出的那个元素 (节点, 所在模块, 所在单元)（OPAQUE、ValueElement 与 * 展开当推不出）。
        折不出的那几截出处都推得出是函数参数、sys.argv、argparse 的属性、临时目录、根外前缀的，记进 facts.caller_launches，收闭包时判是不是入口参数
        （ScriptClosure.caller_launch_refusals）；推不出、有环境变量的当场拒算。弄坏开关 python-caller-launches-refused 下一律当场拒算。"""
        origins = None
        if "python-caller-launches-refused" not in self.index.breaks and isinstance(item, tuple) and not isinstance(item[0], ast.Starred):
            element, module, owner = item
            origins = module.unfolded_part_origins(element, owner)
        if origins and all(origin[0] == "param" or (origin[0] == "skip" and origin[1] in CALLER_INPUT_ORIGINS) for origin in origins):
            facts.caller_launches.append((self.refusal(form, node.lineno), frozenset(origins)))
        else:
            self.refuse(facts, form, node)

    def registry_listed_programs(self, unit, facts, line):
        """起子进程的那个函数登在本模块的 REGISTRY_LISTED_PROGRAMS 里（文件头「登记表点名的程序」）→ 登记表点名的、根里在的脚本（相对根，排好序）；
        没登记 → None；登记表读不了、正则写错 → 拒算 subprocess-path-missing、交 []。弄坏开关 python-registry-programs-linked 下登记表另记进
        facts.registry_reads（收闭包时收成数据单元）。"""
        scope = unit
        while scope is not None and enclosing_unit(scope) is not None:
            scope = enclosing_unit(scope)
        found = self.module_dictionary(REGISTRY_LISTED_PROGRAMS_TABLE, None) if scope is not None else None
        if found is None:
            return None
        dictionary, assignment = found
        for key, value in zip(dictionary.keys, dictionary.values):
            if not (isinstance(key, ast.Constant) and key.value == scope.name):
                continue
            if not (isinstance(value, (ast.Tuple, ast.List)) and len(value.elts) == 2):
                return None
            paths = self.fold(value.elts[0], assignment)
            pattern = value.elts[1].value if isinstance(value.elts[1], ast.Constant) and isinstance(value.elts[1].value, str) else None
            if not paths or len(paths) != 1 or pattern is None:
                return None
            registry = next(iter(paths))
            relative = self.index.relative(posixpath.normpath(registry if posixpath.isabs(registry) else posixpath.join(self.index.root, registry)))
            try:
                if relative is None:
                    raise OSError(f"登记表 {registry} 不在根里")
                with open(os.path.join(self.index.root, relative), encoding="utf-8", errors="replace") as handle:
                    text = handle.read()
                expression = re.compile(pattern)
            except (OSError, re.error):
                facts.refusals.append(self.refusal("subprocess-path-missing", line))
                return []
            if "python-registry-programs-linked" in self.index.breaks:
                facts.registry_reads.append(relative)
            return sorted({match.group(1) for match in expression.finditer(text)
                           if match.groups() and match.group(1) and os.path.isfile(os.path.join(self.index.root, match.group(1)))})
        return None

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
            if "python-command-concatenation-unfolded" not in self.index.breaks:
                # 一边认不出（git + step 里循环取的 step）：那一段占一格 OPAQUE，同 *展开认不出；命令位置上的词落在它上面照旧拒算（command_links）
                left = left if left is not None else [[OPAQUE]]
                right = right if right is not None else [[OPAQUE]]
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
        if isinstance(node, ast.Subscript) and "python-command-slices-unfolded" not in self.index.breaks:
            return self.subscript_command_alternatives(node, unit, depth)
        if isinstance(node, ast.Name):
            return self.name_command_alternatives(node.id, unit, depth)
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name):
            return self.call_command_alternatives(node.func.id, unit, depth)
        return None

    def subscript_command_alternatives(self, node, unit, depth):
        """命令表达式是下标（文件头「起代码的写法逐种认」python 的命令）：f(…)[k] 取 f 各处 return 的元组里第 k 项再认；
        x[a:b]（a、b 是不小于 0 的整数字面量或不写，b 另认 x.index(<字面词>)：到头一个等于它的词之前）把 x 的每种取法照切；别的 → None。"""
        index = node.slice
        if isinstance(index, ast.Constant) and type(index.value) is int and index.value >= 0 and isinstance(node.value, ast.Call) \
                and isinstance(node.value.func, ast.Name):
            returned = self.returned_elements(node.value.func.id, unit, index.value)
            if returned is None:
                return None
            alternatives = []
            for value, module, owner in returned:
                found = module.command_alternatives(value, owner, depth + 1)
                if found is None:
                    return None
                alternatives += found
            return alternatives or None
        if not isinstance(index, ast.Slice) or index.step is not None:
            return None
        base = self.command_alternatives(node.value, unit, depth + 1)
        if base is None:
            return None
        out = []
        for alternative in base:
            cut = self.sliced_alternative(alternative, index, node.value, unit)
            if cut is None:
                return None
            if cut is not False:
                out.append(cut)
        return out or None

    def cannot_equal(self, node, unit, wanted):
        """折不出的元素一定不等于 wanted：f 串里有一段字面字不在 wanted 里（f"{X}={n}" 一类环境赋值），或路径的字面尾巴 wanted 不以它收尾。"""
        if isinstance(node, ast.JoinedStr):
            return any(isinstance(part, ast.Constant) and str(part.value) not in wanted for part in node.values)
        tail = self.literal_tail(node, unit)
        return bool(tail) and not wanted.endswith(tail)

    def environment_assignment_shape(self, node, unit):
        """折不出的元素是 env 的一条 NAME=VALUE：f"{名字常量}=…"、f"NAME=…" 一类，名字那一段折得出、后面跟着字面的 = → "NAME=…"（给 command_positions 认包装）；别的 → None。"""
        if not isinstance(node, ast.JoinedStr):
            return None
        prefix = ""
        for part in node.values:
            if isinstance(part, ast.Constant):
                text = str(part.value)
            elif isinstance(part, ast.FormattedValue) and part.format_spec is None and part.conversion in (-1, 115):
                folded = self.fold(part.value, unit)
                if folded is None or len(folded) != 1:
                    return None
                text = next(iter(folded))
            else:
                return None
            if "=" in text:
                name = prefix + text.split("=", 1)[0]
                return f"{name}=…" if re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", name) else None
            prefix += text
        return None

    def sliced_alternative(self, alternative, index, sequence, unit):
        """一种取法 [(元素节点, 所在模块, 所在单元) 或 OPAQUE…] 照切片 index 切 → 切出来的那一段；切点前有认不出个数或取值的元素 → None；
        上界是 x.index(<字面词>) 而这种取法里没有等于它的词（跑起来抛 ValueError、走不到起子进程）→ False。"""
        def bound(value):
            if value is None:
                return None, True
            if isinstance(value, ast.Constant) and type(value.value) is int and value.value >= 0:
                return value.value, True
            return None, False
        lower, lower_known = bound(index.lower)
        upper, upper_known = bound(index.upper)
        if not lower_known:
            return None
        if not upper_known:
            call = index.upper
            if not (isinstance(call, ast.Call) and isinstance(call.func, ast.Attribute) and call.func.attr == "index" and len(call.args) == 1
                    and not call.keywords and isinstance(sequence, ast.Name) and isinstance(call.func.value, ast.Name)
                    and call.func.value.id == sequence.id):
                return None
            wanted = self.fold(call.args[0], unit)
            if wanted is None or len(wanted) != 1:
                return None
            wanted = next(iter(wanted))
            upper = None
            for position, item in enumerate(alternative):
                if item is OPAQUE or isinstance(item, ValueElement) or isinstance(item[0], ast.Starred):
                    return None
                value = {"python3"} if item[1].canonical(item[0], item[2]) == "sys.executable" else item[1].fold(item[0], item[2])
                if value is None:
                    if item[1].cannot_equal(item[0], item[2], wanted):
                        continue
                    return None
                if value == {wanted}:
                    upper = position
                    break
                if wanted in value:
                    return None
            if upper is None:
                return False
        reach = len(alternative) if upper is None else upper
        if any(item is OPAQUE or (not isinstance(item, ValueElement) and isinstance(item[0], ast.Starred)) for item in alternative[:max(reach, lower or 0)]):
            return None
        return alternative[lower or 0:upper]

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

    def returned_values(self, name, unit):
        """调仓内模块级函数 name（本模块的，或 from 仓内模块 import 进来的）：(它的模块, 它的单元, 各处 return 的值)；认不出、一处 return 值都没有 → None。"""
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
        values = [statement.value for statement in walk_function_statements(candidates[0].node) if isinstance(statement, ast.Return) and statement.value is not None]
        return (module, candidates[0], values) if values else None

    def dictionary_parameter_values(self, node, key, unit, function, depth):
        """dictionary_values 里名字不在本函数里赋值时：函数参数按每一处调用给的实参折，模块级的按它那一处赋值折。"""
        values = []
        if function is not None:
            arguments = function.args
            positional = [argument.arg for argument in arguments.posonlyargs + arguments.args]
            if node.id in positional and unit.container is None:
                position = positional.index(node.id)
                defaults = dict(zip(positional[len(positional) - len(arguments.defaults):], arguments.defaults))
                sites = self.function_call_sites(unit)
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
                sites = self.function_call_sites(unit) if unit.container is None else []
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
            if enclosing_unit(unit) is not None:
                # 套着的函数里没赋、不是参数的名字：外层函数的局部变量（闭包变量）或模块级的
                return self.name_command_alternatives(name, enclosing_unit(unit), depth + 1)
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
            sites = self.function_call_sites(unit)
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
            calls = {"path_values": module.path_values, "literal_tail": module.literal_tail, "directory_wildcard": module.directory_wildcard,
                     "script_tail_group": module.script_tail_group}
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
        shape = list(literal)
        if "python-environment-assignments-unfolded" not in breaks:
            # env 后面折不出值的 NAME=VALUE（f"{X}={n}"）：名字折得出就照一条赋值认，命令位置照样往后找
            for position, item in enumerate(elements):
                if shape[position] is None and item is not OPAQUE and not isinstance(item, ValueElement) and not isinstance(item[0], ast.Starred):
                    shape[position] = item[1].environment_assignment_shape(item[0], item[2])
        positions = command_positions(shape) if "python-unfolded-heads-allowed" not in breaks else [0]
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
                if "python-toolchain-queries-refused" not in breaks and len(values) == head + 2 and literal[head + 1] in TOOLCHAIN_QUERY_WORDS:
                    # [<RUSTC 或 build.rustc 给的编译器>, "-V"] 一类版本询问：只打版本、不跑仓里的代码，输出归工具链行（文件头「版本询问」）
                    facts.toolchain_queries.append(node.lineno)
                    facts.handled.append(("toolchain-query", node.lineno))
                    return
                self.refuse_launch(facts, COMMAND_FORM, node, elements[head])
                facts.handled.append(("head-unfolded", node.lineno))
                return
            values[head] = copied
            literal[head] = single(head)
        if values[head] is not None and len(values[head]) > 1 and "python-command-heads-split-ignored" not in breaks:
            # 命令位置上的词折出几个（[name, "--version"] 里 name 取 python3 或 bash）：每个取值当一种取法各认一遍
            for value in sorted(values[head]):
                self.command_links(elements[:head] + [ValueElement(frozenset((value,)))] + elements[head + 1:], unit, node, facts, keywords)
            facts.handled.append(("head-split", node.lineno))
            return
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
                if values[argument] is None and "python-script-tail-groups-ignored" not in breaks:
                    # 目录折不出、脚本名那一截是字面的（os.path.join(<临时树>, "tool/entry.py")）：仓里以这一截收尾的脚本整组连上；
                    # 一份都没有的不是仓里的脚本（这一份自己写出来的夹具、根外的程序），不连
                    group = element_call(argument, "script_tail_group")
                    if group is not None and not group:
                        facts.handled.append(("outside-script", node.lineno))
                        position = argument + 1
                        continue
                    values[argument] = group
                listed = self.registry_listed_programs(unit, facts, node.lineno) \
                    if values[argument] is None and "python-registry-programs-ignored" not in breaks else None
                if listed is not None and "python-registry-programs-linked" in breaks:
                    # 弄坏开关 python-registry-programs-linked：登记表里点名的每一份都连（整份脚本体），登记表本身收成数据单元
                    facts.program_links.extend(listed)
                    facts.handled.append(("registry-listed", node.lineno))
                elif listed is not None:
                    # 登记表点名的程序是环境探针（文件头「登记表点名的程序」）：退出码与输出由调用方处置（前提每趟现判、版本经 environment-of: 进键），不连、不拒算
                    facts.handled.append(("environment-probe", node.lineno))
                elif values[argument] is None:
                    self.refuse_launch(facts, "subprocess-path-unfolded", node, elements[argument])
                else:
                    cwd_unfolded = "cwd" in keywords and not self.path_values(keywords["cwd"], unit, directory=True)
                    launched = launch_arguments(elements, values, argument + 1)
                    for value in sorted(values[argument]):
                        file = self.file_for_value(value, bases)
                        if file is None and cwd_unfolded and not posixpath.isabs(value) and value.strip("/") \
                                and "python-script-tail-groups-ignored" not in breaks:
                            # cwd= 折不出、脚本写的是相对路径（["bash", "setup.sh"], cwd=<拷出来的样本>）：仓里以 /<它> 收尾的整组连上，一份都没有的不是仓里的脚本
                            stripped = posixpath.normpath(value).strip("/")
                            group = [relative for relative in self.index.repository_files() if relative.endswith("/" + stripped)]
                            facts.program_links.extend(group)
                            if not group:
                                facts.handled.append(("outside-script", node.lineno))
                            continue
                        if file is None:
                            self.refuse(facts, "subprocess-path-missing", node)
                        else:
                            facts.program_links.append(file)
                            # 程序入口带参数（文件头「程序入口带参数」）：脚本之后的词带过去，被起的那份照它们剪
                            facts.program_arguments.setdefault(file, []).append(launched)
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
                self.inline_from_command(facts, node, "python" if python else "shell", texts, after, bases,
                                         source=elements[argument + 1] if argument + 1 < len(elements) else None)
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

    def inline_from_command(self, facts, node, language, texts, after, bases, stdin=False, source=None):
        """命令里 -c 的串、input= 喂的标准输入：正文折得出记成一份 InlineScript，折不出拒算 inline-script-not-literal。
        after 是串之后（标准输入时是解释器选项之后）那几个元素的取值：python 的 sys.argv[1:]；bash 的 -c 是 $0 $1 …，标准输入是 $1 …。
        source 是 -c 那一串的元素（折不出时看它的出处：调用方交进来的不当场拒算，refuse_launch）。"""
        if texts is None:
            self.refuse_launch(facts, INLINE_FORM, node, source)
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


def rows_inside_quotes(rows, mask):
    """规范文本里起头落在一个还没收尾的引号或命令替换里的那几行（多行的 '…'、$'…'、"…"、$( … )、`…`）的下标；mask 里的 here-doc 正文先抹空。
    函数头与收尾的 } 不在这几行里认：引号里的 awk 程序、内联脚本照原样留着缩进，它们的「    }」与 declare -f 打的函数收尾长得一样。"""
    text = "\n".join("" if position in mask else row for position, row in enumerate(rows))
    starts, offset = [], 0
    for position, row in enumerate(rows):
        starts.append(offset)
        offset += (0 if position in mask else len(row)) + 1
    spans, i, total = [], 0, len(text)
    while i < total:
        character = text[i]
        if character == "\\":
            i += 2
            continue
        if character == "$" and text.startswith("$'", i):
            j = i + 2
            while j < total and text[j] != "'":
                j += 2 if text[j] == "\\" else 1
            spans.append((i, j))
            i = j + 1
            continue
        if character == "'":
            end = text.find("'", i + 1)
            end = total if end < 0 else end
            spans.append((i, end))
            i = end + 1
            continue
        if character == '"':
            end = double_quote_end(text, i)
            spans.append((i, end - 1))
            i = end
            continue
        if text.startswith("$(", i):
            end = matching_close(text, i + 1, "(", ")")
            spans.append((i, end))
            i = end + 1
            continue
        if character == "`":
            end = text.find("`", i + 1)
            end = total if end < 0 else end
            spans.append((i, end))
            i = end + 1
            continue
        i += 1
    inside, cursor = set(), 0
    for opening, closing in spans:
        while cursor < len(starts) and starts[cursor] <= opening:
            cursor += 1
        probe = cursor
        while probe < len(starts) and starts[probe] <= closing:
            inside.add(probe)
            probe += 1
    return inside


SHELL_POSITIONAL_REFERENCE = re.compile(r'"\$\{?([1-9])(?::-)?\}?"|\$\{([1-9])(?::-)?\}|\$([1-9])(?![0-9])')
# 位置参数从这里起就可能变了（shift、set --）或要按轮取（循环）：从头一处起不再代入
SHELL_ARGUMENTS_MOVED = re.compile(r"(?:^|[\s;&|(])(?:shift|while|until|for|select|set\s+--|set\s+-(?=\s))(?=$|[\s;&|)])")
SHELL_DECIDABLE_IF = re.compile(r"^(\s*)if \[\[ (\S+) (==|!=) (\S+) \]\]; then$")
SHELL_DECIDABLE_CASE = re.compile(r"^(\s*)case (\S+) in\s*$")


def substituted_positionals(text, words):
    """脚本体正文里头一处 shift / set -- / 循环之前的 "$1" "${1}" "${1:-}" $1 …（单引号里的不算）代成字面参数：
    引号外的写成 shlex.quote 的词，双引号里的照双引号的转义写；参数没给的代成空串。"""
    rows = text.split("\n")
    mask = heredoc_mask(rows)
    inside = rows_inside_quotes(rows, mask)
    starts, offset = [], 0
    for row in rows:
        starts.append(offset)
        offset += len(row) + 1
    # here-doc 正文与引号里的那几行不算（python 的 here-doc 里的 for 不是这份脚本的循环）；单引号里的一截先去掉再找
    cut = next((starts[position] for position, row in enumerate(rows) if position not in mask and position not in inside
                and SHELL_ARGUMENTS_MOVED.search(re.sub(r"'[^']*'", "''", row))), len(text))
    masked_spans = {starts[position]: starts[position] + len(rows[position]) for position in mask}
    out, i, quoted = [], 0, False
    while i < cut:
        if i in masked_spans:
            out.append(text[i:masked_spans[i]])
            i = masked_spans[i]
            continue
        character = text[i]
        if character == "\\":
            out.append(text[i:i + 2])
            i += 2
            continue
        if character == "'" and not quoted:
            end = text.find("'", i + 1)
            end = cut - 1 if end < 0 or end >= cut else end
            out.append(text[i:end + 1])
            i = end + 1
            continue
        found = SHELL_POSITIONAL_REFERENCE.match(text, i) if character in "$\"" else None
        if found and not (quoted and found.group(0).startswith('"')):
            position = int(next(group for group in found.groups() if group))
            value = words[position - 1] if position <= len(words) else ""
            if quoted:
                out.append(re.sub(r'([\\"$`])', r"\\\1", value))
            else:
                out.append(shlex.quote(value))
            i = found.end()
            continue
        if character == '"':
            quoted = not quoted
        out.append(character)
        i += 1
    return "".join(out) + text[cut:]


def pruned_decided_branches(rows):
    """代过参数之后判得出真假的 if [[ <字面> == / != <字面> ]]; then … fi（不带 elif；带 else 的留判得出的那一支）与 case <字面> in …
    （分支的模式都是字面、没有 ;& ;;&：只留头一个对得上的分支，判不出的那个分支起照留）：走不到的那几行抹空（行数不变）。"""
    rows = list(rows)
    for start, row in enumerate(rows):
        found = SHELL_DECIDABLE_IF.match(row)
        if found:
            left, right = literal_word(found.group(2)), literal_word(found.group(4))
            if left is None or right is None or (found.group(4)[:1] not in "'\"" and any(mark in right for mark in "*?[")):
                continue
            indent = found.group(1)
            other, end = None, None
            for later in range(start + 1, len(rows)):
                if rows[later].startswith(indent + "elif ") and not rows[later].startswith(indent + " "):
                    other = "elif"
                    break
                if rows[later] == indent + "else":
                    other = later
                elif rows[later] in (indent + "fi", indent + "fi;"):
                    end = later
                    break
            if end is None or other == "elif":
                continue
            holds = (left == right) == (found.group(3) == "==")
            dead = range(other + 1, end) if holds and other is not None else range(start + 1, other if other is not None else end) if not holds else ()
            for position in dead:
                rows[position] = ""
            continue
        found = SHELL_DECIDABLE_CASE.match(row)
        if not found or literal_word(found.group(2)) is None:
            continue
        subject, branch_indent = literal_word(found.group(2)), found.group(1) + "    "
        branches, end, current = [], None, None
        for later in range(start + 1, len(rows)):
            text = rows[later]
            if text.rstrip(";") == found.group(1) + "esac":
                end = later
                break
            if text in (branch_indent + ";&", branch_indent + ";;&"):
                branches = None
                break
            if text == branch_indent + ";;":
                current = None
                continue
            if current is None and text.startswith(branch_indent) and not text.startswith(branch_indent + " ") and text.endswith(")"):
                current = [later, later + 1, text[len(branch_indent):-1]]
                branches.append(current)
                continue
            if current is not None:
                current[1] = later + 1
        if end is None or not branches:
            continue
        matched = False
        for head, stop, patterns in branches:
            if matched:
                for position in range(head + 1, stop):
                    rows[position] = ""
                continue
            alternatives = [literal_word(alternative.strip()) for alternative in patterns.lstrip("(").split("|")]
            if None in alternatives:
                break
            if any(fnmatch.fnmatchcase(subject, alternative) for alternative in alternatives):
                matched = True
            else:
                for position in range(head + 1, stop):
                    rows[position] = ""
    return rows


def narrowed_shell_body(text, words):
    """shell 起法带字面参数（文件头「shell 起法带字面参数」）：脚本体正文代进参数（substituted_positionals）、抹掉判得出走不到的分支
    （pruned_decided_branches）。弄坏开关 shell-program-arguments-unnarrowed 下不剪（ScriptClosure.push_program）。"""
    substituted = substituted_positionals(text, words)
    return "\n".join(pruned_decided_branches(substituted.split("\n")))


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
        if (character == "[" and word and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", "".join(word))
                and "shell-subscript-assignment-split" not in active_breaks()):
            end = matching_close(text, i, "[", "]")   # 名字[下标]=值、名字[下标]+=值：下标里有空格也是一个赋值词（bash 照赋值认）
            if text.startswith(("=", "+="), end + 1):
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


def remove_shell_pattern(value, pattern, operator):
    """${名%词} ${名%%词} ${名#词} ${名##词}：照 bash 去掉最短 / 最长的后缀或前缀（词照 shell 通配认，同 fnmatch）；对不上交原值。"""
    cuts = range(len(value), -1, -1) if operator in ("%", "##") else range(len(value) + 1)
    for cut in cuts:
        if operator.startswith("%") and fnmatch.fnmatchcase(value[cut:], pattern):
            return value[:cut]
        if operator.startswith("#") and fnmatch.fnmatchcase(value[:cut], pattern):
            return value[cut:]
    return value


def leading_command_words(command):
    """一条命令去掉开头的关键字（if、then、!…）与 NAME=VALUE 前缀之后的那几个词。"""
    words = list(command)
    while words and words[0] in SHELL_KEYWORDS:
        words = words[1:]
    while words and ASSIGNMENT_WORD.match(words[0]):
        words = words[1:]
    return words


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
    """bash -c '<串>' <$0> <$1>…：串里外层 bash 会展开的 $1…$9 换成外面那几个词（去掉外层双引号，留在串里原来的引号里；
    外面没给的、带单引号的照原样留着，照旧折不出）。外层不展开的不换：反斜杠转义的（\\$1，串里套着的里层 bash -c "…\\$1…" 留给里层，
    里层照它自己的参数再换一次）与单引号里的（双引号外的 '…$1…'）照原样留着。
    弄坏开关 shell-inline-positionals-escapes-ignored：转义的与单引号里的也换（里层的 \\$1 换成外层的 $1）。"""
    def replacement(found):
        position = int(found.group(1) or found.group(2))
        if position >= len(rest) or "'" in rest[position]:
            return found.group(0)
        return rest[position].replace('"', "")
    if "shell-inline-positionals-escapes-ignored" in active_breaks():
        return INLINE_POSITIONAL.sub(replacement, inline)
    out, i, quoted, total = [], 0, False, len(inline)
    while i < total:
        character = inline[i]
        if character == "\\" and i + 1 < total:
            out.append(inline[i:i + 2])
            i += 2
            continue
        if character == "'" and not quoted:
            end = inline.find("'", i + 1)
            end = total - 1 if end < 0 else end
            out.append(inline[i:end + 1])
            i = end + 1
            continue
        if character == '"':
            quoted = not quoted
        found = INLINE_POSITIONAL.match(inline, i) if character == "$" else None
        if found:
            out.append(replacement(found))
            i = found.end()
            continue
        out.append(character)
        i += 1
    return "".join(out)


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
        quoted = set() if "shell-function-bounds-quote-blind" in self.breaks else rows_inside_quotes(body, mask)
        functions = []
        for position, row in enumerate(body):
            if position in mask or position in quoted:
                continue
            found = FUNCTION_HEADER.match(row)
            if not found or position + 1 >= len(body) or body[position + 1] != found.group(1) + "{ ":
                continue
            indent = found.group(1)
            end = None
            for later in range(position + 2, len(body)):
                if later in mask or later in quoted:
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
        if first in (("source",) if "shell-dot-source-unrecognized" in self.breaks else ("source", ".")) and len(words) > 1:
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
        elif name == "PWD" and "shell-pwd-unfolded" not in self.breaks:
            base = self.working_directory_of(unit, progress)
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
        removal = re.match(r"^(%%|%|##|#)(.+)$", rest, re.S)
        if removal and base and "shell-pattern-removals-unfolded" not in self.breaks:
            # ${名%词} ${名%%词} ${名#词} ${名##词}：词也折得出时照 bash 去掉后缀或前缀（文件头「起代码的写法逐种认」）
            patterns = self.fold_word(removal.group(2), unit, progress)
            if patterns is None:
                return None
            return {remove_shell_pattern(value, pattern, removal.group(1)) for value in base for pattern in patterns}
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
        if "shell-relative-directories-unrooted" not in self.breaks:
            # cd 到相对目录（cd scripts/x && python3 y.py）：那个目录照根认，拼出来的才是根里的路径
            bases = [base if posixpath.isabs(base) else posixpath.join(self.root, base) for base in bases]
        for base in bases + [self.root, os.path.dirname(os.path.join(self.root, unit.file))]:
            candidate = os.path.normpath(os.path.join(base, value))
            relative = inside_root(self.root, candidate)
            if relative is not None and os.path.isfile(candidate):
                return ("ok", relative)
        return ("missing", None)

    def links(self, unit, kinds=None):
        """→ [(source|shell|python, [文件…], 拒算或 None)]。kinds 给了只走这几种写法（起头的与展开出来的都按它挑，结果不进缓存；
        shell_sourced_files 只走 source：bash -c 这类子进程里的 source 是内联脚本那一种，不在其中）。"""
        if unit.cache is None:
            unit.cache = {}
        if kinds is None and "links" in unit.cache:
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
            if kinds is not None and kind not in kinds:
                continue
            self.overlays.append(overlay or {})
            try:
                self.resolve_invocation(state, kind, word, directory, extra, generator_line, overlay)
            finally:
                self.overlays.pop()
        if kinds is None:
            unit.cache["links"] = out
            unit.cache["launch_arguments"] = state.get("arguments", {})
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
        if directory is not None and directories is None and "shell-directory-copies-ignored" not in self.breaks:
            # cd "$目录/<字面尾巴>" 而尾巴在根里是一个目录（临时树、仓副本里的同一相对路径）：照根里那个目录认
            directories = self.tree_copy_directories(directory)
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
            relative = literal_word(word)
            if directory is not None and relative and not posixpath.isabs(relative) and "shell-relative-tree-copies-ignored" not in self.breaks:
                # cd 到折不出的目录（临时树、仓副本）之后的相对路径：根里同一相对路径（至少两段）那一份
                copied = tree_copy_file(self.root, relative)
                if copied is not None:
                    out.append((kind, [copied], None))
                    return
            if kind in ("shell", "python", "source") and "shell-script-tail-groups-ignored" not in self.breaks:
                grouped = self.shell_tail_group(unquoted, unit)
                if grouped is not None:
                    if grouped:
                        out.append((kind, sorted(grouped), None))
                    return
            if kind in ("shell", "python", "source") and "shell-directory-wildcards-ignored" not in self.breaks:
                wildcard = self.directory_wildcard_targets(kind, unquoted, unit)
                if wildcard:
                    out.append((kind, sorted(wildcard), None))
                    return
            if kind in ("shell", "python", "source") and SHELL_POSITIONAL_WORD.match(word) and "shell-positional-scripts-refused" not in self.breaks:
                supplied = self.positional_scripts(word, unit, 0)
                if supplied is not None:
                    if supplied[0]:
                        out.append((kind, sorted(supplied[0]), None))
                    if not supplied[1]:
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
            if kind == "shell":
                # bash <脚本> 后面的词全是字面的：记下来，收脚本体时按它剪（shell_launch_arguments、narrowed_script_body）
                words = tuple(literal_word(item) for item in (extra[0] if extra else ()))
                recorded = state.setdefault("arguments", {})
                for file in files:
                    if None in words or recorded.get(file, set()) is None:
                        recorded[file] = None
                    else:
                        recorded.setdefault(file, set()).add(words)

    def shell_launch_arguments(self, unit, file):
        """这个单元起 bash <file> 的每一处带的字面参数（{(词…)}）；有一处带了折不出的、或没起过 → None（脚本体整份收）。"""
        self.links(unit)
        return (unit.cache or {}).get("launch_arguments", {}).get(file)

    def narrowed_script_body(self, file, words):
        """bash <file> <字面参数…> 起的那份脚本体（文件头「shell 起法带字面参数」）：边照 narrowed_shell_body 剪过的正文认，
        单元的种类、名字、指纹与原来那一份相同（键照整份脚本体算）；剪不掉什么的交原来那一份。"""
        cache = self.__dict__.setdefault("narrowed_bodies", {})
        if (file, words) not in cache:
            body = self.files[file].script_body
            text = narrowed_shell_body(body.body or "", words)
            if text == body.body:
                cache[(file, words)] = body
            else:
                view = copy.copy(body)
                view.body, view.facts, view.cache = text, None, None
                cache[(file, words)] = view
        return cache[(file, words)]

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

    def tree_copy_directories(self, word):
        """cd 过去的目录词折不出、是「$目录/<字面尾巴>」而尾巴在根里是一个目录（临时树、仓副本里的同一相对路径）→ {根里那个目录}；别的 → None。"""
        found = SHELL_TREE_PREFIX.match(word.replace('"', "").replace("'", ""))
        copied = tree_copy_file(self.root, found.group(1), directory=True) if found else None
        return {posixpath.join(self.root, copied)} if copied is not None else None

    def working_directory_of(self, unit, progress):
        """$PWD：这份文件的脚本体顶层恰好一处 cd、它折得出恰好一个目录（cd "$(dirname "$0")/../.." 这一类）→ {那个目录}；脚本体自己、别的 → None。
        函数在子 shell 里 cd 到临时树再取 $PWD 的同样照这一个目录认：拷进临时树的是同一相对路径那一份（同「临时树尾巴」那一条）。"""
        shell_file = self.files.get(unit.file) if unit is not None else None
        body = shell_file.script_body if shell_file is not None else None
        if body is None or body is unit or "PWD" in progress:
            return None
        changes = [words for words in (leading_command_words(command) for command in self.facts(body).commands) if words and literal_word(words[0]) == "cd"]
        if len(changes) != 1:
            return None
        arguments = [word for word in changes[0][1:] if not (literal_word(word) or "").startswith("-")]
        values = self.fold_word(arguments[0], body, progress | {"PWD"}) if len(arguments) == 1 else None
        if not values or len(values) != 1:
            return None
        value = next(iter(values))
        return {posixpath.normpath(value if posixpath.isabs(value) else posixpath.join(self.root, value))}

    def directory_wildcard_targets(self, kind, unquoted, unit):
        """「<折得出的目录>/<带展开的文件名>」（python3 "s/tools/$1"、bash "$dir/$name"）而那个目录在根里：目录里每一份同种脚本（python 起 .py，
        bash 与 source 起 .sh、.bash）——起的是调用方点名的其中一份，哪一份都可能，就都连（同 python 那边 os.path.join(<仓里的目录>, <折不出的文件名>)）。"""
        directory, _slash, name = unquoted.rpartition("/")
        if not directory or "$" not in name:
            return None
        values = self.fold_word(directory, unit, frozenset())
        if not values:
            return None
        suffixes = (".py",) if kind == "python" else (".sh", ".bash")
        files = set()
        for value in values:
            absolute = posixpath.normpath(value if posixpath.isabs(value) else posixpath.join(self.root, value))
            relative = inside_root(self.root, absolute)
            if relative is None or not os.path.isdir(absolute):
                return None
            files |= {posixpath.normpath(posixpath.join(relative, entry)) for entry in os.listdir(absolute)
                      if entry.endswith(suffixes) and os.path.isfile(os.path.join(absolute, entry))}
        return files or None

    def positional_scripts(self, word, unit, depth):
        """解释器后面的脚本是这个函数的位置参数（"$@" "$1" "${@:2}"）：脚本由调用处给。这份文件与 source 它的那几份里每一处以这个函数名起头的命令
        取对应的那个词，在调用处那个单元里折（折不出的试临时树尾巴；调用处又是位置参数的接着往上找，至多 3 层）
        → (相对根的文件集合, 有没有折不出的调用处)；不在函数里、一处调用都没有 → None。"""
        found = re.match(r'^"?\$(?:\{(@|\*|[1-9][0-9]*)(?::([1-9][0-9]*))?\}|(@|\*|[1-9]))"?$', word)
        if unit is None or unit.kind != "function" or depth > 3 or not found:
            return None
        which = found.group(1) or found.group(3)
        position = int(found.group(2)) if found.group(2) else (1 if which in "@*" else int(which))
        callee = unit.name.split("::")[-1]
        files, unfolded, sites = set(), False, 0
        for path in [unit.file] + sorted(self.upward(unit.file)):
            shell_file = self.files.get(path)
            for caller in shell_file.units if shell_file is not None else ():
                for command in self.facts(caller).commands:
                    words = leading_command_words(command)
                    if not words or literal_word(words[0]) != callee or position >= len(words):
                        continue
                    sites += 1
                    supplied = words[position]
                    nested = self.positional_scripts(supplied, caller, depth + 1) if SHELL_POSITIONAL_WORD.match(supplied) else None
                    if nested is not None:
                        files |= nested[0]
                        unfolded = unfolded or nested[1]
                        continue
                    values = self.fold_word(supplied, caller, frozenset())
                    if values is None:
                        copied = self.tree_copy_targets(supplied.replace('"', ""), caller)
                        files |= copied or set()
                        unfolded = unfolded or not copied
                        continue
                    for value in values:
                        status, file = self.target_of(value, caller)
                        if status == "ok":
                            files.add(file)
                        else:
                            unfolded = True
        return (files, unfolded) if sites else None

    def shell_tail_group(self, unquoted, unit):
        """「$<折不出的目录>/<折得出的尾巴>」（尾巴里的变量照赋值折："$ROOT/$SAMPLE_DIRECTORY/driver.sh" 一类）：仓里以这一截收尾的脚本整组
        （相对根的集合，同 python 一侧的 script_tail_group；一份都没有的不是仓里的脚本，交空集合，不连也不拒算）；
        尾巴折不出、不是 .sh / .py / .bash、最后一段以 . 起头、往根外走的 → None（照旧拒算）。"""
        found = re.match(r"^\$\{?[A-Za-z_][A-Za-z0-9_]*\}?(/.+)$", unquoted)
        if not found:
            return None
        tails = self.fold_word(found.group(1), unit, frozenset())
        if not tails:
            return None
        if "repository_files" not in self.__dict__:
            self.repository_files = files_under(self.root, ("",))
        files = set()
        for tail in tails:
            stripped = posixpath.normpath(tail).strip("/")
            if not stripped.endswith((".sh", ".py", ".bash")) or posixpath.basename(stripped).startswith(".") or stripped.startswith(".."):
                return None
            files |= {relative for relative in self.repository_files if relative == stripped or relative.endswith("/" + stripped)}
        return files

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


def shell_sourced_files(root, path):
    """shell 脚本 path（相对根）在它自己那个 shell 里（传递地）source / . 进来的仓内文件 → (按路径排的文件元组，不含 path 自己, 拒算列表)
    （文件头「Rust 闭包读到的 shell 脚本」）。路径照 ShellIndex.links 的折法；每份文件的每个单元都看（读到它的是整份字节，不按调没调到挑），
    新收进来的那几份接着看，到不再多出文件为止。弄坏开关 shell-data-sources-nested-unfollowed：只看 path 自己那一层；
    shell-data-source-refusals-dropped：折不出的不拒算。"""
    breaks = active_breaks()
    index = ShellIndex(root, [path])
    files = {path}
    while True:
        found, refusals = set(), set()
        scanned = {path} if "shell-data-sources-nested-unfollowed" in breaks else files
        for current in sorted(scanned):
            shell_file = index.files.get(current)
            if shell_file is None:
                continue
            refusals.update(shell_file.refusals)
            for unit in shell_file.units:
                if unit.data:
                    continue
                for _kind, targets, refusal in index.links(unit, kinds=("source",)):
                    if refusal is not None:
                        refusals.add(refusal)
                    found.update(targets)
        grown = files | found
        if grown == files:
            break
        index.add(sorted(grown - files))
        files = grown
    if "shell-data-source-refusals-dropped" in breaks:
        refusals = set()
    ordered = sorted(refusals, key=lambda refusal: (refusal.file, refusal.line, refusal.form, refusal.snippet))
    return tuple(sorted(files - {path})), ordered


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


# import 时就执行的模块级语句（文件头「Python 闭包：import 时执行的模块级语句」）：类头只当类头收进来时走的规则名（不算被点名、不连方法）
IMPORT_TIME_RULE = "import-time"
IMPORT_TIME_CLASS_RULE = "import-time-class"
# 判法单元闭包不走的子进程边（文件头「不走的子进程边」）：(起它的文件, 起它的单元的限定名, 被起的程序) → 为什么。按实测排：这一条边只打提示、不改结局，
# 自证 EXCLUDED_EDGE_PYTHON_CELL 与 EXCLUDED_EDGE_SHELL_CELL 在夹具上证这套机制（自证期间临时登记夹具里的边）。与 scripts/crash/lib_verdict_library.py 的 NOT_JUDGING_FILES
# （按角色排、整份文件）是两种排除。登记一条边之前先现查起点单元与被起的程序都在；每条登记的边配三格自证（准入边那三格是 ADMISSION_EDGE_*_CELL）。
ADMISSION_EDGE = ("scripts/experiment-apparatus/e183-capacity-cells.sh", "plan_step", "scripts/gate/admission.py")
EXCLUDED_PROGRAM_EDGES = {
    ADMISSION_EDGE: "准入是闸：退出码只定跑不跑（77 不跑、3 退 78、别的退 2，都在编译之前），标准输出只有 E7INPUT 行，"
                    "读产物的判法先 sed '/^E7INPUT /d'；不改读数",
}


class ScriptClosure:
    def __init__(self, root, under):
        self.root = os.path.abspath(root)
        self.breaks = active_breaks()
        self.python = PythonIndex(self.root)
        self.under = list(under)
        self.shell = None
        self.closure = Closure()
        self.named_classes, self.attributes_seen, self.loaded_modules, self.modules_seen = set(), set(), set(), set()
        self.import_time_classes = set()   # 只当类头收进来（IMPORT_TIME_CLASS_RULE）、还没被点名的类头：之后被点名时再连方法
        self.methods_by_name = collections.defaultdict(list)
        self.closure_sourcers, self.processed_shell = collections.defaultdict(set), collections.defaultdict(list)
        self.todo = []
        # 内联脚本切出来的单元：穿过它们连下去，自己不进闭包（正文已在起它的那个单元里）
        self.inline_seen, self.inline_units = set(), {}
        # 程序入口带参数（文件头「程序入口带参数」）：给了参数才剪；收过边的 (单元 id, 视图)（视图 None 是整份，盖住别的视图）
        self.narrowing, self.views_done = False, set()
        # 运行期读的文件里本模块没有调用处的参数 [(函数单元, 参数名, 读它的模块, 读它的单元, 行, 是不是拷文件的源)] 与调用方交进来的命令 [(拒算, 出处)]：收完闭包再判
        self.pending_parameters, self.pending_launches = [], []
        # 读 <参数>/<字面尾巴> 的每一处 [(取法, 读它的单元)] 与已经照尾巴收过的那几处（下标）：每收完一轮闭包按调用处重认一遍
        self.tail_reads, self.tail_reads_collected = [], set()
        self.calls_by_name = None

    def shell_index(self, extra=()):
        if self.shell is None:
            self.shell = ShellIndex(self.root, shell_files_under(self.root, self.under) + list(extra))
        elif extra:
            self.shell.add(list(extra))
        return self.shell

    def push(self, unit, via, rule, view=None):
        """view：None 整份收；frozenset 的已知值表是程序入口带参数剪过的那一种；SHALLOW_IMPORT_VIEW 是给了参数时 import 时执行的赋值（文件头「程序入口带参数」）。"""
        if unit is None:
            return
        if id(unit) in self.closure.members and self.view_done(unit, view):
            self.named_after_import_time(unit, rule)
        else:
            self.todo.append((unit, via, rule, view))

    def view_done(self, unit, view):
        return (id(unit), None) in self.views_done or (id(unit), view) in self.views_done

    def named_after_import_time(self, unit, rule):
        """先只当类头收进来（import 时执行）的类头，之后被点名（别的规则又推到它）：照被点名的类连方法。"""
        if unit.kind == "class" and rule != IMPORT_TIME_CLASS_RULE and id(unit) in self.import_time_classes:
            self.import_time_classes.discard(id(unit))
            self.class_entered(unit)

    def push_program(self, file, via, launched=None, shell_arguments=None):
        """launched：程序入口带参数时起它的那几处带过去的参数（KnownWords 的集合）；None 是不知道、整份收（文件头「程序入口带参数」子进程边）。
        shell_arguments：shell 起 shell 的每一处带的字面参数（{(词…)}，ShellIndex.shell_launch_arguments）；None 是有一处不知道，脚本体整份收。"""
        language = language_of_file(self.root, file)
        if language == "python":
            module = self.python.module(file)
            if module is not None:
                self.closure.refusals.update(module.refusals)
                for unit in module.program_entries():
                    if launched and unit.kind == "main-guard":
                        for words in sorted(launched, key=repr):
                            self.push(unit, via, "program", frozenset({(ARGV_BINDING, KnownWords((file, *words.words), words.exact))}))
                    else:
                        self.push(unit, via, "program")
        elif language == "shell":
            index = self.shell_index([file])
            shell_file = index.files.get(file)
            if shell_file is not None:
                self.closure.refusals.update(shell_file.refusals)
                if shell_arguments is None and launched and all(words.exact and None not in words.words for words in launched):
                    shell_arguments = {tuple(words.words) for words in launched}
                if shell_arguments and shell_file.script_body is not None and "shell-program-arguments-unnarrowed" not in self.breaks:
                    # 文件头「shell 起法带字面参数」：边照按参数剪过的脚本体认（单元与指纹照原来那一份）
                    for words in sorted(shell_arguments):
                        self.push(index.narrowed_script_body(file, words), via, "program")
                else:
                    self.push(shell_file.script_body, via, "program")

    def run(self, entries, program_arguments=None):
        """program_arguments：python 程序入口带的参数（文件头「程序入口带参数」）；None 是没给，照整份收。"""
        self.closure.entry_units.extend(entry for entry in entries if entry is not None)
        self.narrowing = program_arguments is not None and "python-program-arguments-unnarrowed" not in self.breaks
        for entry in entries:
            view = None
            if self.narrowing and entry is not None and entry.language == "python" and entry.kind == "main-guard":
                view = frozenset({(ARGV_BINDING, KnownWords((entry.file, *program_arguments), True))})
            self.push(entry, None, "entry", view)
        while True:
            self.drain_pending_units()
            # 闭包里按名字认的调用处随新收进来的单元重认；调用方交进来的命令在调用处折出的根里脚本（supplied_program_linked）推进 todo：连完再判一轮
            self.calls_by_name = None
            self.resolve_parameter_reads()
            self.caller_launch_refusals()
            if not self.todo:
                return self.closure

    def drain_pending_units(self):
        while self.todo:
            unit, via, rule, view = self.todo.pop()
            if unit.scope == "inline":
                if id(unit) in self.inline_seen:
                    self.named_after_import_time(unit, rule)
                    continue
                self.inline_seen.add(id(unit))
            else:
                self.closure.add(unit, via, rule)
                if self.view_done(unit, view):
                    self.named_after_import_time(unit, rule)
                    continue
                self.views_done.add((id(unit), view))
            if unit.data:
                continue
            if unit.language == "python":
                self.python_edges(unit, rule, view)
            elif unit.language == "shell":
                self.shell_edges(unit)

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
        if "python-import-time-statements-ignored" in self.breaks:
            return
        # 模块有单元进闭包，它就被载入过：import 时执行的模块级语句一起进来（文件头「Python 闭包：import 时执行的模块级语句」）——
        # 模块级 import、赋值、类头本身（IMPORT_TIME_CLASS_RULE：不算被点名、不连方法）与 def 头（装饰器、缺省值）；其余模块级语句在 module-body 里，本来就连
        for unit in module.units:
            if unit.data or unit.container is not None:
                continue
            if unit.kind in ("import", "assign", "def-head"):
                # 给了参数时 import 时执行的赋值只收赋值本身，值里当元素写的函数名由取这张表的那一处连（文件头「程序入口带参数」）
                self.push(unit, module.module_body, IMPORT_TIME_RULE, SHALLOW_IMPORT_VIEW if self.narrowing and unit.kind == "assign" else None)
            elif unit.kind == "class":
                self.push(unit, module.module_body, IMPORT_TIME_CLASS_RULE)

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

    def python_edges(self, unit, rule=None, view=None):
        module = unit.module
        if module.path not in self.modules_seen:
            self.module_entered(module)
        facts = module.view_facts(unit, view) if isinstance(view, frozenset) else module.facts(unit)
        self.closure.refusals.update(facts.refusals)
        self.pending_launches.extend(facts.caller_launches)
        for line in facts.toolchain_queries:
            self.closure.toolchain_queries.add((unit.file, line))
            if "python-toolchain-queries-unkeyed" not in self.breaks:
                self.closure.add(self.toolchain_unit(unit.file, line), unit, "toolchain-query")
        for relative in facts.registry_reads:
            self.closure.add(self.data_unit(relative), unit, "registry")
        for callee, callee_view in facts.narrowed_calls:
            self.push(callee, unit, f"call {callee.name}", callee_view)
        for table in facts.shallow_tables:
            self.push(table, unit, "table-keys", SHALLOW_IMPORT_VIEW)
        if unit.container is not None:
            self.push(unit.container, unit, "container")
        if unit.kind != "module-body":
            self.push(module.module_body, unit, "module-body")
        if unit.kind == "class":
            if rule == IMPORT_TIME_CLASS_RULE and "python-import-time-classes-named" not in self.breaks:
                self.import_time_classes.add(id(unit))   # 只当类头收进来：不连方法，之后被点名再连（named_after_import_time）
            else:
                self.class_entered(unit)
        for target in facts.import_targets:
            self.push(target, unit, "import")
        names = facts.names
        if view == SHALLOW_IMPORT_VIEW and "python-import-time-tables-followed" not in self.breaks:
            names = names - module.table_reference_names(unit)
        for name in sorted(names):
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
        for file in dict.fromkeys(facts.program_links):
            if (unit.file, unit.name, file) in EXCLUDED_PROGRAM_EDGES and "program-edge-exclusions-ignored" not in self.breaks:
                continue   # 登记了的那一条子进程边不走（EXCLUDED_PROGRAM_EDGES）
            # 每一处起它的都带着参数（一处不知道就整份收）才照参数剪（文件头「程序入口带参数」子进程边）
            launched = facts.program_arguments.get(file, [])
            self.push_program(file, unit, set(launched) if self.narrowing and launched and len(launched) == facts.program_links.count(file) else None)
        for script in facts.inline_scripts:
            self.push_inline(script, unit)
        for file in facts.whole_modules:
            self.push_whole(file, unit)
        self.python_data_reads(module, unit, facts)
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

    def toolchain_unit(self, file, line):
        """工具链询问那一处（文件头「版本询问」）→ 它的工具链单元；rust_toolchain_lines 一个闭包只问一次。"""
        if "toolchain_fingerprint" not in self.__dict__:
            self.toolchain_fingerprint = sha256_text("".join(text + "\n" for text in rust_toolchain_lines(self.root)))
        return Unit("data", TOOLCHAIN_UNIT_KIND, TOOLCHAIN_UNIT_KIND, file, line, line, self.toolchain_fingerprint, data=True)

    def data_unit(self, path):
        """运行期读的仓内文件 → 它的数据单元（data-file，字节 sha256；同一份只造一个）。"""
        units = self.__dict__.setdefault("data_units", {})
        if path not in units:
            units[path] = data_file_unit(self.root, path)
        return units[path]

    def data_directory(self, path):
        """运行期读的根里固定目录（文件头「运行期读的数据文件」一）→ 它的数据单元（data-directory，指纹同 directory_fingerprint；同一个只造一个）。"""
        units = self.__dict__.setdefault("data_directories", {})
        if path not in units:
            units[path] = data_directory_unit(self.root, path)
        return units[path]

    def python_data_reads(self, module, unit, facts):
        """单元里读文件的每一处（PythonFacts.data_reads）：照文件头「运行期读的数据文件」三类落（data_read_origin）。"""
        for origins, line, copied in facts.data_reads:
            for origin in sorted(origins, key=lambda item: tuple(str(part) for part in item if not isinstance(part, Unit))):
                self.data_read_origin(origin, module, unit, line, copied)

    def data_read_origin(self, origin, module, unit, line, copied=False):
        """一种取法：文件、目录收成数据单元（规则名 data-read）；拒算的记 data-path-unfolded，落在读它的那一行（copied：拷文件的源，不拒算）；
        本模块里没有调用处的参数留到收完闭包再认（resolve_parameter_reads）；别的（临时目录、调用方的输入、根外、推不出的）不收也不拒算。"""
        kind = origin[0]
        if kind == "file":
            self.closure.add(self.data_unit(origin[1]), unit, "data-read")
        elif kind == "directory":
            self.closure.add(self.data_directory(origin[1]), unit, "data-read")
        elif kind == "refuse" and not copied:
            self.closure.refusals.add(module.refusal(DATA_FORM, line))
        elif kind == "param":
            self.pending_parameters.append((origin[1], origin[2], module, unit, line, copied))
        elif kind == "param-tail":
            # 前缀是参数的字面尾巴（PythonModule.prefix_parameter）：收完闭包再按调用处认（resolve_parameter_reads）
            self.tail_reads.append((origin, unit))

    def tail_read_supplied(self, function, name, resolved, depth=0):
        """读 <参数>/<字面尾巴> 的那个参数（文件头「运行期读的数据文件」二）在闭包里每一处调用给的都是这一趟自己建的临时目录、入口的参数（被判的输入），
        或又是这样的参数（接着往上一层认）：尾巴那一组不收。闭包里一处调用都没有的算调用方的输入。给的是 sys.argv、环境变量、根里的路径、推不出的
        （仓副本、根本身都可能）→ False，照尾巴收仓里那一组。弄坏开关 python-tail-reads-unanchored 下不走这一条（读的时候就整组收）。"""
        if depth > 8:
            return False
        if id(function) in {id(entry) for entry in self.closure.entry_units}:
            return True
        key = (id(function), name)
        if key not in resolved:
            resolved[key] = False   # 正在判：绕回来的当不是
            supplied = True
            for argument, caller in self.closure_call_arguments(function, name):
                anchor = caller.module.prefix_parameter(argument, caller) if caller is not None and caller.module is not None else None
                if anchor is not None:
                    supplied = self.tail_read_supplied(anchor[0], anchor[1], resolved, depth + 1)
                else:
                    origins = caller.module.head_origin(argument, caller, 0) if caller is not None and caller.module is not None else None
                    supplied = origins is not None and all(
                        origin == ("skip", "temporary") or (origin[0] == "param" and self.tail_read_supplied(origin[1], origin[2], resolved, depth + 1))
                        for origin in origins)
                if not supplied:
                    break
            resolved[key] = supplied
        return resolved[key]

    def resolve_parameter_reads(self):
        """本模块里没有调用处的参数读的文件（文件头「运行期读的数据文件」一）：入口的参数算调用方的输入；别的在闭包里按名字找调用处（closure_call_arguments），
        逐处照实参认（实参又是这种参数的接着找）；闭包里一处都找不到的算调用方的输入。弄坏开关 python-parameter-callers-unresolved 下不是入口的拒算。"""
        entries = {id(entry) for entry in self.closure.entry_units}
        seen = set()
        while self.pending_parameters:
            function, name, module, reader, line, copied = self.pending_parameters.pop()
            if id(function) in entries or (id(function), name, line, module.path) in seen:
                continue
            seen.add((id(function), name, line, module.path))
            if "python-parameter-callers-unresolved" in self.breaks:
                self.closure.refusals.add(module.refusal(DATA_FORM, line))
                continue
            for argument, caller in self.closure_call_arguments(function, name):
                for origin in sorted(caller.module.data_read_origins(argument, caller), key=lambda item: tuple(str(part) for part in item if not isinstance(part, Unit))):
                    self.data_read_origin(origin, module, reader, line, copied)
        for position, (origin, reader) in enumerate(self.tail_reads):
            # 闭包里有一处给的不是这一趟的临时目录、也不是被判的输入：照尾巴收仓里那一组
            if position not in self.tail_reads_collected and not self.tail_read_supplied(origin[1], origin[2], {}):
                self.tail_reads_collected.add(position)
                for relative in sorted(origin[3]):
                    self.closure.add(self.data_unit(relative), reader, "data-read")

    def closure_call_arguments(self, function, name):
        """闭包里调 function 的每一处给参数 name 的实参 [(实参节点, 调用处的作用域单元)]：按名字认（f(…)、x.f(…)；__init__ 认 类名(…)），
        方法去掉 self；带 * / ** 的那一处认不出对哪个参数、跳过；没给的取缺省值（作用域是 function 自己）。"""
        definition = function.node
        if not isinstance(definition, (ast.FunctionDef, ast.AsyncFunctionDef)):
            return []
        arguments = definition.args
        positional = [argument.arg for argument in arguments.posonlyargs + arguments.args]
        method = function.container is not None and not any(
            isinstance(decorator, ast.Name) and decorator.id == "staticmethod" for decorator in definition.decorator_list)
        if method:
            positional = positional[1:]
        called = function.container.name.split(".")[-1] if method and definition.name == "__init__" else definition.name
        defaults = dict(zip(positional[len(positional) - len(arguments.defaults):], arguments.defaults))
        defaults.update({argument.arg: default for argument, default in zip(arguments.kwonlyargs, arguments.kw_defaults) if default is not None})
        position = positional.index(name) if name in positional else None
        if self.calls_by_name is None:
            self.calls_by_name = collections.defaultdict(list)
            for member in self.closure.order:
                if getattr(member, "language", None) != "python" or member.data or getattr(member, "module", None) is None:
                    continue
                for node in walk_nodes(member.nodes or []):
                    if isinstance(node, ast.Call):
                        callee = node.func.id if isinstance(node.func, ast.Name) else node.func.attr if isinstance(node.func, ast.Attribute) else None
                        if callee:
                            self.calls_by_name[callee].append((node, member))
        out = []
        for call, member in self.calls_by_name.get(called, ()):
            if any(isinstance(item, ast.Starred) for item in call.args) or any(keyword.arg is None for keyword in call.keywords):
                continue
            argument = call.args[position] if position is not None and position < len(call.args) else None
            argument = next((keyword.value for keyword in call.keywords if keyword.arg == name), argument)
            if argument is not None:
                out.append((argument, member.module.scope_of(call, member)))
            elif defaults.get(name) is not None:
                out.append((defaults[name], function))
        return out

    def caller_launch_refusals(self):
        """调用方交进来的命令（文件头「调用方交进来的命令」）：折不出的那几截出处都是这一次入口的参数，或 sys.argv、argparse 的属性、临时目录、根外前缀的
        不拒算（被起的命令由请求那一侧的单元键管）；别的函数的参数照闭包里每一处调它的实参往上认（launch_supplied_by_caller），推不出的照旧拒算。"""
        resolved = {}
        while self.pending_launches:
            refusal, origins = self.pending_launches.pop()
            if not self.launch_supplied_by_caller(origins, resolved):
                self.closure.refusals.add(refusal)

    def launch_supplied_by_caller(self, origins, resolved):
        """起子进程那一处折不出的几截都是调用方交进来的：出处是 sys.argv、argparse 的属性、临时目录、根外前缀，或入口的参数，
        或别的函数的参数而闭包里每一处调它的实参又都是这几样（接着往上找；resolved 按（函数, 参数名）记下判过的，绕回来的不算推得出）。
        闭包里没人调的别的函数的参数推不出。"""
        entries = {id(entry) for entry in self.closure.entry_units}
        for origin in origins:
            if origin[0] == "skip" and origin[1] in CALLER_INPUT_ORIGINS:
                continue
            if origin[0] != "param":
                return False
            function, name = origin[1], origin[2]
            if id(function) in entries:
                continue
            if not self.parameter_supplied(function, name, resolved):
                return False
        return True

    def parameter_supplied(self, function, name, resolved, depth=0):
        """别的函数的参数 name 给的命令：闭包里调 function 的每一处的实参都认得出（launch_argument_supplied）；入口的参数算调用方交进来的；
        闭包里没人调的推不出。resolved 按（函数, 参数名）记下判过的，绕回来的不算推得出。"""
        if id(function) in {id(entry) for entry in self.closure.entry_units}:
            return True
        key = (id(function), name)
        if key not in resolved:
            resolved[key] = False   # 正在判：绕回来的当推不出
            sites = self.closure_call_arguments(function, name)
            resolved[key] = bool(sites) and all(self.launch_argument_supplied(argument, caller, resolved, depth + 1) for argument, caller in sites)
        return resolved[key]

    def launch_argument_supplied(self, argument, caller, resolved, depth=0):
        """闭包里一处调用给命令参数的实参，在调用处折（文件头「调用方交进来的命令」：被调函数的参数按每个调用点的实参折）：
        折得出字面值的，是根里的脚本就当程序连上（解释器词不算：脚本在命令的别处，认不出）；根外的程序（outside_head）；
        名字照调用处那一层的参数（接着往上一层调用处认）、本函数里的赋值、外层函数的变量认；调仓内函数的，照它每一处 return 的值在它那边认；
        别的照调用方交进来的那几样认（unfolded_part_origins）。弄坏开关 python-launch-arguments-unfolded 下只认最后一样。"""
        if depth > 12 or caller is None or getattr(caller, "module", None) is None:
            return False
        module = caller.module
        if "python-launch-arguments-unfolded" not in self.breaks:
            folded = module.fold(argument, caller)
            if folded is not None:
                return all(self.supplied_program_linked(value, caller) for value in folded)
            if module.outside_head(argument, caller):
                return True
            if isinstance(argument, ast.Name) and isinstance(caller.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                arguments = caller.node.args
                assigned = module.local_assignments(caller).get(argument.id)
                if assigned is None and argument.id in [item.arg for item in arguments.posonlyargs + arguments.args + arguments.kwonlyargs]:
                    return self.parameter_supplied(caller, argument.id, resolved, depth + 1)
                if assigned:
                    return all(value is not None and not isinstance(value, ElementsOf) and self.launch_argument_supplied(value, caller, resolved, depth + 1)
                               for value in assigned)
                if assigned is None and enclosing_unit(caller) is not None:
                    return self.launch_argument_supplied(argument, enclosing_unit(caller), resolved, depth + 1)
            if isinstance(argument, ast.Call) and isinstance(argument.func, ast.Name):
                returned = module.returned_values(argument.func.id, caller)
                if returned is not None:
                    callee_module, callee, values = returned
                    return all(self.launch_argument_supplied(value, callee, resolved, depth + 1) for value in values)
        found = module.unfolded_part_origins(argument, caller)
        return found is not None and self.launch_supplied_by_caller(found, resolved)

    def supplied_program_linked(self, value, caller):
        """调用处折出来的命令词（launch_argument_supplied）：根里的 .py、.sh 当程序连上；python3、bash 这类解释器词不算认得出
        （脚本写在命令的别处）；别的（cargo、根外的程序）不连。"""
        if posixpath.basename(value) in PYTHON_INTERPRETERS | SHELL_INTERPRETERS:
            return False
        relative = value if not posixpath.isabs(value) else posixpath.relpath(value, self.root)
        if not relative.startswith("../") and os.path.isfile(os.path.join(self.root, relative)) and relative.endswith((".py", ".sh")):
            self.push_program(posixpath.normpath(relative), caller)
        return True

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
                    if (unit.file, unit.name, file) in EXCLUDED_PROGRAM_EDGES and "program-edge-exclusions-ignored" not in self.breaks \
                            and "program-edge-exclusions-shell-ignored" not in self.breaks:
                        continue   # 登记了的那一条子进程边不走（EXCLUDED_PROGRAM_EDGES；shell 函数起的同样认）
                    self.push_program(file, unit, shell_arguments=index.shell_launch_arguments(unit, file) if kind == "shell" else None)
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
        # 判定种类（文件头「判定种类」）；product_dump 只给库接口的调用方交 Rust 那一份 dump 行用，命令行不认
        self.judgment_kind, self.product_dump = FORMAL_VERIFICATION, None
        # python 程序入口带的参数（文件头「程序入口带参数」）；None 是没给
        self.program_arguments = None


def closure_of(root, entry, options=None):
    options = options or Options()
    if not os.path.isdir(root):
        raise FunctionUnitsError(f"根不在：{root}")
    language, path, qualified = parse_entry(root, entry)
    if language == "rust":
        workspace = RustWorkspace(root, options.judgment_kind, options.product_dump)
        unit, target_id = workspace.find_entry(path, qualified, options.target)
        return rust_closure_of(workspace, unit, target_id, options.cfg, options.no_default_cfg, options.features, options.no_default_features)
    if language == "wgsl":
        files = [wgsl_file(root, item) for item in [path] + list(options.wgsl_modules)]
        entries = [unit for unit in files[0].units if unit.name == qualified] if qualified else None
        if qualified and not entries:
            raise UsageError(f"{path} 里没有 {qualified}")
        sources, workspace, rust = None, None, None
        closure = Closure()
        closure.entry_units.extend(entries or ())
        if options.rust_strings_entry:
            rust_language, rust_path, rust_qualified = parse_entry(root, options.rust_strings_entry)
            workspace = RustWorkspace(root, options.judgment_kind, options.product_dump)
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
        return script.run(entries, None if qualified else options.program_arguments)
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


def closure_unit_lines(closure):
    """closure 与 key 命令打的单元行（dump_text）：数据目录单元照 crates/singlefs-unit-graph 按读它的那一处各一行
    （data-directory\\t<目录>\\t<读它的文件>:<行>-<行>\\t<目录指纹>），别的单元照常。弄坏开关 closure-directory-lines-span-files：
    数据目录单元打成它自己那一行（<目录>:1-<文件数>）。"""
    if "closure-directory-lines-span-files" in active_breaks():
        return [unit.dump_text() for unit in closure.units()]
    units = [unit for unit in closure.units() if unit.kind != "data-directory"]
    units += [Unit("data", "data-directory", site[2], site[0], site[1], site[1], site[3], data=True) for site in closure.data_sites if site[5] == "data-directory"]
    return [unit.dump_text() for unit in sorted(units, key=Unit.sort_key)]


# 按行进键的登记表（文件头「按行进键的登记表」）：表（相对根）→ (读整张表、交解析结果的函数名, {只按其中几行取用的函数名: 它看的那几行的正则})；
# 正则里的 {entry} 换成这一条判定的入口文件（re.escape 过）
ROW_KEYED_TABLES = {
    ".claude/hooks/selftest-registry.tsv": (frozenset(("read_selftest_registry", "selftest_registry_at")), {
        "registered_selftest": r"^{entry}\s",                                    # 按被起的那份脚本的键查它自己那一行
        "stage_samples_row": r"^\.claude/gate\.d/[^/\s]+\s",                     # 按门禁阶段查样本档那一行
        "registered_sample_tool_directories": r"^\.claude/gate\.d/[^/\s]+\s",    # 样本档行里写的假工具目录
    }),
}


def row_keyed_closure(closure, root, entry_file):
    """按行进键的登记表（文件头「按行进键的登记表」）：闭包里 ROW_KEYED_TABLES 登记的那张表的 data-file 单元，闭包里调读表函数的每个单元
    （读表函数自己除外）都是登记的取用函数时，换成只含那几个取用函数看的行的 data-rows 单元；有一个调用者不在登记里、entry_file 是 None、
    弄坏开关 row-keyed-tables-whole 下照旧整份。交回一份新的 Closure（原来那一份不动；没换的交原来那一份）。"""
    if entry_file is None or "row-keyed-tables-whole" in active_breaks():
        return closure
    replaced = {}
    for unit in closure.order:
        if not (unit.data and unit.kind == "data-file" and unit.name in ROW_KEYED_TABLES):
            continue
        readers, consumers = ROW_KEYED_TABLES[unit.name]
        patterns = set()
        for member in closure.order:
            if member.data or member.language != "python" or member.module is None or member.name.split(".")[-1] in readers:
                continue
            facts = member.module.facts(member)
            if not (readers & (set(facts.names) | {attribute for _base, attribute in facts.attributes})):
                continue
            pattern = consumers.get(member.name.split(".")[-1])
            if pattern is None:
                patterns = None
                break
            patterns.add(pattern.replace("{entry}", re.escape(entry_file)))
        if not patterns:
            continue
        expressions = [re.compile(pattern) for pattern in sorted(patterns)]
        with open(os.path.join(root, unit.file), encoding="utf-8", errors="surrogateescape") as handle:
            selected = [line for line in handle.read().split("\n") if any(expression.search(line) for expression in expressions)]
        fingerprint = sha256_text("\n".join(sorted(patterns)) + "\0" + "\n".join(selected))
        replaced[id(unit)] = Unit("data", "data-rows", unit.name, unit.file, 1, max(len(selected), 1), fingerprint, data=True)
    if not replaced:
        return closure
    projected = copy.copy(closure)
    projected.order = [replaced.get(id(unit), unit) for unit in closure.order]
    projected.members = {id(unit): unit for unit in projected.order}
    projected.via = dict(closure.via)
    for old, new in replaced.items():
        projected.via[id(new)] = closure.via.get(old)
    return projected


def toolchain_version_line(label, command, root):
    """rust_toolchain_lines 的一行：<label>: <输出>（多行用 | 连起来）；起不来、退非 0 的也记一行（起不来也是一种取值）。"""
    try:
        completed = subprocess.run(command, cwd=root, capture_output=True, timeout=TOOLCHAIN_QUERY_SECONDS)
    except (OSError, subprocess.SubprocessError) as error:
        return f"{label}: 起不来（{type(error).__name__}）"
    output = " | ".join(line for line in completed.stdout.decode("utf-8", "replace").split("\n") if line.strip())
    return f"{label}: 退 {completed.returncode}: {output}"


def configured_compilers(root):
    """cargo 配置里 build.rustc 指的编译器（文件头「版本询问」）：根往上每一层的 .cargo/config.toml 与 .cargo/config，再加 CARGO_HOME
    （没设取家目录下的 .cargo）下的 config.toml；带 / 的相对路径从 .cargo 所在那一层起，不带 / 的照 PATH 找。→ [(配置文件, 程序)]。"""
    directories, directory = [], os.path.abspath(root)
    while True:
        directories.append(os.path.join(directory, ".cargo"))
        parent = os.path.dirname(directory)
        if parent == directory:
            break
        directory = parent
    directories.append(os.environ.get("CARGO_HOME") or os.path.join(os.path.expanduser("~"), ".cargo"))
    found = []
    for configuration in directories:
        for name in ("config.toml", "config"):
            path = os.path.join(configuration, name)
            try:
                with open(path, "rb") as handle:
                    settings = tomllib.load(handle)
            except (OSError, ValueError):
                continue
            build = settings.get("build") if isinstance(settings.get("build"), dict) else {}
            program = build.get("rustc")
            if isinstance(program, str) and program:
                if "/" in program and not os.path.isabs(program):
                    program = os.path.join(os.path.dirname(configuration), program)
                found.append((path, program))
    return found


def rust_toolchain_lines(root):
    """闭包里有工具链询问时折进工具链单元的那几行（文件头「版本询问」），在 root 里起：PATH 上的 rustc -vV、cargo -V，
    环境变量 RUSTC、CARGO_BUILD_RUSTC 设了的各自 -V，cargo 配置 build.rustc 指的编译器各自 -V（configured_compilers）。"""
    lines = [toolchain_version_line("rustc -vV", ["rustc", "-vV"], root), toolchain_version_line("cargo -V", ["cargo", "-V"], root)]
    for variable in TOOLCHAIN_PROGRAM_VARIABLES:
        program = os.environ.get(variable)
        if program:
            lines.append(toolchain_version_line(f"{variable}={program} -V", [program, "-V"], root))
    for configuration, program in configured_compilers(root):
        lines.append(toolchain_version_line(f"build.rustc={program}（{configuration}）-V", [program, "-V"], root))
    return lines


def closure_key(closure, toolchain=(), dependencies=(), externals=None, allow_refused=False):
    """→ (键或 None, 拒算列表)。拒算不空时键是 None（allow_refused 只给对照用：照算出键，拒算照样交回）。
    闭包里的工具链询问（文件头「版本询问」）由闭包里的工具链单元进键，调用方不另折。
    拒算记在 closure.refusals 里，别处并起来的闭包（lib_verdict_library.py 的 FunctionUnitIndex.key）照拒算并过去，不悄悄少收。"""
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
    lines += entry_key_lines(closure)
    return sha256_text("".join(line + "\n" for line in lines)), ordered


def entry_key_lines(closure):
    """键里的 entry 行（文件头「键」）：closure.entry_units 里每个入口单元连同包它的 impl / trait 头（Unit.container）一层层往外，各取（种类，指纹），
    一个入口一行、排序去重；不带名字、路径与行号。闭包是单元的多重集，几个入口互相收进对方的闭包（一个 trait impl 里的几个方法经 R-结构、R-隐式，
    python 里互相调用的两个 def）时闭包一模一样，没有这几行就共用一把键、一份判定（scripts/gate/verus-proof-check.py 的 entry_unit_component 同一种写法）。
    文档测试取到它自己就停：它的 container 是所挂的条目，rust_closure_of 的 R-结构不连它（另编一个 crate），键里也不带它的正文。
    没记入口的闭包（别处自己拼的、并起来的）不出这几行。弄坏开关 key-entry-units-dropped：一行都不出；
    key-entry-doctest-owner-chained：文档测试照别的单元往外取所挂的条目。"""
    if "key-entry-units-dropped" in active_breaks():
        return []
    lines = set()
    for entry in getattr(closure, "entry_units", ()):
        chain, owner, visited = [], entry, set()
        while owner is not None and id(owner) not in visited:
            visited.add(id(owner))
            chain.append(f"{owner.kind}\t{owner.fingerprint}")
            if owner.kind == "doctest" and "key-entry-doctest-owner-chained" not in active_breaks():
                break
            owner = owner.container
        lines.add("entry\t" + "\t".join(chain))
    return sorted(lines)


def key_cfg_items(closure, unnarrowed):
    """键里的 cfg 那几项：闭包记了真引到的原子（cfg_atoms，「<原子>\\t<true|false>」）就只取它们；没记的（别的脚本自己拼的闭包、
    并起来的闭包没并 cfg_atoms）退回 unnarrowed（整套生效 cfg：只多不少，键照样对，只是带不带特性的两趟分开存）。
    弄坏开关 rust-cfg-atoms-unnarrowed 下一律退回 unnarrowed。"""
    narrowed = getattr(closure, "cfg_atoms", None)
    if narrowed is None or "rust-cfg-atoms-unnarrowed" in active_breaks():
        return list(unnarrowed)
    return list(narrowed)


def unit_graph_key_lines(closure, toolchain=(), lock_lines=()):
    """unit_graph_key 取 sha256 的那几行（次序就是 crates/singlefs-unit-graph 的 reuse_key 拼的次序）：entry-target-kind\\t<目标种类>、
    cfg\\t<原子>\\t<true|false>（闭包里各单元真引到的 cfg 原子与它在收那个单元的目标下的真值，排好序；key_cfg_items）、
    unit\\t<种类>\\t<指纹>（闭包里每个 Rust 与 WGSL 单元，排序）、data\\t<摘要>（每一处读数据文件，排序）、
    entry\\t<种类>\\t<指纹>[\\t<种类>\\t<指纹>…]（入口单元自己，entry_key_lines：一个入口一行、排序去重；Rust 那一份的 entry_key_lines 同一种行、同一个位置）、
    toolchain\\t<行>、lock\\t<行>（调用方给的次序）。弄坏开关 unit-graph-key-entry-lines-dropped：不出 entry 行。"""
    lines = [f"entry-target-kind\t{closure.entry_target_kind}"]
    lines += [f"cfg\t{atom}" for atom in sorted(key_cfg_items(closure, closure.entry_cfg_atoms))]
    lines += sorted(f"unit\t{unit.kind}\t{unit.fingerprint}" for unit in closure.order if not unit.data)
    lines += sorted(f"data\t{site[3]}" for site in closure.data_sites)
    if "unit-graph-key-entry-lines-dropped" not in active_breaks():
        lines += entry_key_lines(closure)
    lines += [f"toolchain\t{line}" for line in toolchain]
    lines += [f"lock\t{line}" for line in lock_lines]
    return lines


def unit_graph_key(closure, toolchain=(), lock_lines=()):
    """照 crates/singlefs-unit-graph 的 reuse_key 算 Rust 闭包的键（两份实现逐入口对拍用；python 这一份自己的复用键仍是 closure_key）→ (键或 None, 拒算)。
    unit_graph_key_lines 那几行以 \\n 连、末尾不带，取 sha256。闭包里有拒算不交键。"""
    ordered = sorted(closure.refusals, key=lambda refusal: (refusal.file, refusal.line, refusal.form, refusal.snippet))
    if ordered:
        return None, ordered
    return sha256_text("\n".join(unit_graph_key_lines(closure, toolchain, lock_lines))), []


def dump_lines(root, language=None, under=None, judgment_kind=FORMAL_VERIFICATION, product_dump=None):
    """→ (单元行, 拒算行, 各语言单元数)。判定种类是产品行为时 Rust 单元照产品行为视图（文件头「判定种类」），只给 Verus 看的不列。"""
    units, refusals, counts = [], set(), collections.Counter()
    if language in (None, "rust"):
        workspace = RustWorkspace(root, judgment_kind, product_dump)
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
                if unit.ghost_only:
                    continue
                units.append(unit)
                counts["rust"] += 1
                refusals.update(unit.refusals)
                if "rust-data-dump-unlisted" not in active_breaks():
                    # 数据单元照 crates/singlefs-unit-graph 的 dump：每一处读（include_str! 一类与运行期拼得出字面路径的读，.wgsl 不算）一行
                    # data\t<数据文件>\t<读它的文件>:<行>-<行>\t<字节 sha256>
                    # 读的是目录的（折出 crates/ 之下的目录）写 data-directory\t<目录>\t<读它的文件>:<行>-<行>\t<目录下全部文件的指纹>，指纹见 directory_fingerprint
                    for kind, value, index in unit.data_references or ():
                        if kind not in ("file", "directory"):
                            continue
                        if (kind, value) not in data_digests:
                            data_digests[(kind, value)] = (data_file_unit(root, value) if kind == "file" else data_directory_unit(root, value)).fingerprint
                        line = rust_file.lines[index]
                        site = Unit("data", "data" if kind == "file" else "data-directory", value, unit.file, line, line, data_digests[(kind, value)], data=True)
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
    ("v5_test", "crates/singlefs-verification/tests/new_pool_file_creation_publish.rs::root_slots_system_configurations_and_journal_ring_hold_the_published_state"),
    ("v11_test", "crates/singlefs-format/src/lib.rs::tests::pointer_record_and_entry_width_literals_equal_the_field_sums_they_stand_for"),
    ("cpu_judge", "crates/singlefs-verification/src/crash.rs::judge_crash_image"),
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
import runpy
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
    return runpy.run_path(os.path.join(tree, "pkg", "c.py"), run_name="__main__")


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


def run_stub():
    stub = os.path.join(os.environ["STUB_DIRECTORY"], os.environ["STUB_NAME"])
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


def run_dynamic_from_environment():
    return run_dynamic(os.environ["SNIPPET"])


def run_unknown_from_environment():
    return run_unknown(os.environ["INTERPRETER"])


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
    "s/subscript.sh": '''#!/usr/bin/env bash
declare -A item
key="$1" address="$2" value="$3"
item[$key $address]="$value"
item[$key]+=" more"
echo "${item[$key $address]}"
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
    "s/nested_inline_positionals.sh": '''#!/usr/bin/env bash
code_of() {
  printf '%s\\n' "import sys" "sys.path.insert(0, sys.argv[1])" "import helper_mod" "helper_mod.run()"
}
code="$(code_of)"
here="$(dirname "${BASH_SOURCE[0]}")"
bash -c 'bash -c "python3 -c \\"\\$1\\" \\"\\$2\\"" inner "$2" "$1"' outer "$here/py" "$code"
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
    "s/argv_slice.sh": '''#!/usr/bin/env bash
here="$(dirname "${BASH_SOURCE[0]}")"
watch_helper() {
  exec python3 -B /dev/fd/3 "$here/py" "$1" 3<<'PY'
import sys
(directory, label) = sys.argv[1:3]
sys.path.insert(0, directory)
import helper_mod
helper_mod.run()
PY
}
watch_helper sample
''',
    "s/argv_slice_mismatched.sh": '''#!/usr/bin/env bash
here="$(dirname "${BASH_SOURCE[0]}")"
watch_helper() {
  exec python3 -B /dev/fd/3 "$here/py" "$1" 3<<'PY'
import sys
(directory, label) = sys.argv[1:4]
sys.path.insert(0, directory)
import helper_mod
helper_mod.run()
PY
}
watch_helper sample
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


# 起子进程的几种折法与运行期读的数据文件的夹具（文件头「起代码的写法逐种认」python 的命令、「运行期读的数据文件」）：每个函数对一格
FOLDS_FIXTURE = {
    "pkg/folds.py": '''import concurrent.futures
import importlib.util
import os
import shutil
import subprocess
import sys
import tempfile

import rows

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
EDGE = ("pkg/c.py", "plan", "pkg/b.py")
TABLE = os.path.join(HERE, "table.tsv")


def nested_launch():
    script = os.path.join(HERE, "c.py")

    def run(extra):
        command = [sys.executable, script] + extra
        return subprocess.run(command, check=False)
    return run(["--quiet"])


def concatenated(steps):
    git = ["git", "-C", HERE]
    for step in steps:
        subprocess.run(git + step, check=False)


def concatenated_caller():
    return concatenated([["status"], ["log"]])


def launch_plan():
    return 4, [sys.executable, os.path.join(HERE, "c.py"), "--", "marker", "test"]


def sliced_launch():
    command = launch_plan()[1]
    launched = command[:command.index("marker")] + ["--flag"]
    return subprocess.run(launched, check=False)


def interpreter_lines(language):
    return {"python": ["python3"], "shell": ["bash"]}.get(language, [])


class Versions:
    def interpreter(self, name):
        return subprocess.run([name, "--version"], check=False)

    def every(self, language):
        return [self.interpreter(name) for name in interpreter_lines(language)]


def loaded_constant():
    path = os.path.join(os.path.dirname(HERE), EDGE[2])
    specification = importlib.util.spec_from_file_location("loaded_b", path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return subprocess.run(["sh", "-c", module.COMMAND], check=False)


def run_from_copy(tree):
    return subprocess.run([sys.executable, os.path.join(tree, "tools", "runner.py")], check=False)


def run_generated():
    return subprocess.run([sys.executable, os.path.join(os.environ["GENERATED_DIRECTORY"], "generated", "made.py")], check=False)


def run_in_copy(copy):
    return subprocess.run(["bash", "setup.sh"], cwd=copy, check=False)


def submitted_run(command):
    return subprocess.run(["bash", "-c", command], check=False)


def submit_all():
    with concurrent.futures.ThreadPoolExecutor() as pool:
        return pool.submit(submitted_run, "python3 pkg/c.py").result()


def environment_launch(threads):
    return subprocess.run(["env", "-u", "UNSET_ME", f"WORKERS={threads}", sys.executable, os.path.join(HERE, "c.py")], check=False)


def table_rows():
    with open(TABLE, encoding="utf-8") as handle:
        return handle.read().splitlines()


def rows_under(root):
    return open(os.path.join(root, "pkg", "table.tsv"), encoding="utf-8").read()


def rows_from_argument():
    return rows_under(sys.argv[1])


def scratch_rows():
    work = tempfile.mkdtemp()
    with open(os.path.join(work, "notes.txt"), "w") as handle:
        handle.write("x")
    return open(os.path.join(work, "notes.txt")).read()


def listed_rows():
    with open(os.path.join(HERE, "index.txt"), encoding="utf-8") as listing:
        return [open(os.path.join(ROOT, line.strip())).read() for line in listing]


def template_rows():
    chosen = sorted(os.listdir(os.path.join(HERE, "templates")))[0]
    return open(os.path.join(HERE, "templates", chosen)).read()


def rows_through_library(request):
    return rows.read_rows(TABLE) + rows.read_rows(request)


class TableReader:
    def __init__(self):
        self.path = TABLE

    def lines(self):
        with open(self.path, encoding="utf-8") as handle:
            return handle.read().splitlines()


def compiler_version():
    program = os.environ["RUSTC"]
    return subprocess.run([program, "-V"], capture_output=True, check=False).stdout


def judge_file(path):
    return open(path).read().strip()


def parsed_rows():
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--table")
    arguments = parser.parse_args()
    return open(arguments.table).read()


def listed_members(units):
    members = {line.split("\\t")[0] for line in units} | {"pkg/table.tsv"}
    return [open(os.path.join(ROOT, relative)).read() for relative in sorted(members)]


def copy_listed(work):
    with open(os.path.join(HERE, "index.txt"), encoding="utf-8") as listing:
        for line in listing:
            shutil.copyfile(os.path.join(ROOT, line.strip()), os.path.join(work, line.strip()))
''',
    "pkg/b.py": '''COMMAND = "python3 pkg/c.py"


def value():
    return 2
''',
    "pkg/c.py": '''def main():
    return 5


if __name__ == "__main__":
    main()
''',
    "pkg/tools/runner.py": '''def run():
    return 7


if __name__ == "__main__":
    run()
''',
    "pkg/sample/setup.sh": "#!/usr/bin/env bash\necho prepared\n",
    "pkg/table.tsv": "alpha\t1\n",
    "pkg/other.tsv": "gamma\t2\n",
    "pkg/index.txt": "pkg/table.tsv\n",
    "pkg/templates/first.txt": "first\n",
    "pkg/templates/second.txt": "second\n",
    "pkg/rows.py": """def read_rows(path):
    with open(path, encoding="utf-8") as handle:
        return handle.read().splitlines()
""",
}
# shell 一侧的几种折法（文件头「起代码的写法逐种认」shell 那几条）的夹具：$PWD、${名%词} / ${名##词}、cd 到临时树里的目录、cd 到折不出的目录之后的相对路径、
# 「<根里的目录>/$1」、函数里 python3 "$@"
SHELL_FOLDS_FIXTURE = {
    "top_child.sh": "#!/usr/bin/env bash\necho top\n",
    "s/child.sh": "#!/usr/bin/env bash\necho child\n",
    "s/tools/one.py": 'def main():\n    return 1\n\n\nif __name__ == "__main__":\n    main()\n',
    "s/tools/two.py": 'def main():\n    return 2\n\n\nif __name__ == "__main__":\n    main()\n',
    "s/pwd_script.sh": '#!/usr/bin/env bash\ncd "$(dirname "$0")/.." || exit 2\nrun_top() {\n  bash "$PWD/top_child.sh"\n}\nrun_top\n',
    "s/pattern_removal.sh": ('#!/usr/bin/env bash\nSELF="s/pattern_removal.sh"\nCHILD_NAME="x/child.sh"\n'
                             'CHILD="${SELF%/pattern_removal.sh}/${CHILD_NAME##*/}"\nbash "$CHILD"\n'),
    "s/directory_copy.sh": '#!/usr/bin/env bash\nrun_tool() {\n  (cd "$1/s/tools" && python3 one.py)\n}\nrun_tool "$1"\n',
    "s/relative_copy.sh": '#!/usr/bin/env bash\nrun_in_copy() {\n  (cd "$1" && bash s/child.sh)\n}\nrun_in_copy "$1"\n',
    "s/directory_wildcard.sh": '#!/usr/bin/env bash\nrun_named() {\n  python3 "s/tools/$1"\n}\nrun_named one.py\n',
    "s/positional_wrapper.sh": '#!/usr/bin/env bash\nrun_python() {\n  PYTHONDONTWRITEBYTECODE=1 python3 "$@"\n}\nrun_python s/tools/one.py --flag\n',
    "s/relative_cd.sh": '#!/usr/bin/env bash\n(cd s/tools && python3 one.py)\n',
    "s/quoted_brace.sh": ("#!/usr/bin/env bash\ncount_rows() {\n  rows=\"$(awk '\n    /x/ {\n      split($2, field, \"=\")\n    }\n"
                          "    END { print NR }\n' \"$1\")\"\n  bash s/child.sh \"$rows\"\n}\ncount_rows input.tsv\n"),
    "s/tail_group.sh": ('#!/usr/bin/env bash\nROOT="$(pwd)"\nSAMPLE_DIRECTORY=".sample"\nrun_driver() {\n  bash "$ROOT/$SAMPLE_DIRECTORY/driver.sh"\n}\n'
                        'run_driver\n'),
    "s/tail_outside.sh": '#!/usr/bin/env bash\nROOT="$(pwd)"\nbash "$ROOT/.nowhere/run.sh"\n',
    "fixtures/one/.sample/driver.sh": "#!/usr/bin/env bash\necho driver\n",
    "s/dispatch.sh": ('#!/usr/bin/env bash\nif [[ "${1:-}" == --selftest ]]; then\n  bash s/child.sh\n  exit $?\nfi\ncase "$1" in\n  run)\n'
                      '    python3 s/tools/one.py\n    ;;\n  *)\n    python3 s/tools/two.py\n    ;;\nesac\n'),
    "s/dispatch_caller.sh": "#!/usr/bin/env bash\nbash s/dispatch.sh run\n",
}
SHELL_PWD_CELL = "shell：$PWD 照脚本体顶层那一处 cd 折（cd \"$(dirname \"$0\")/..\"），函数里 bash \"$PWD/<文件>\" 连到那一份"
SHELL_PATTERN_REMOVAL_CELL = "shell：${名%词} 与 ${名##词} 的词折得出就照 bash 去掉后缀、前缀，连到折出的那一份"
SHELL_DIRECTORY_COPY_CELL = "shell：cd \"$目录/<根里的目录>\" 之后的相对路径照根里那个目录认，连到那一份"
SHELL_RELATIVE_COPY_CELL = "shell：cd 到折不出的目录之后 bash <根里的相对路径>（至少两段）连到根里那一份"
SHELL_DIRECTORY_WILDCARD_CELL = "shell：python3 \"<根里的目录>/$1\" 连到那个目录里每一份 .py"
SHELL_POSITIONAL_SCRIPT_CELL = "shell：函数里 python3 \"$@\" 的脚本照每一处调用给的那个词认，连到那一份"
SHELL_RELATIVE_CD_CELL = "shell：cd <根里的相对目录> 之后 python3 <那个目录里的文件> 连到那一份（不报不在）"
SHELL_QUOTED_BRACE_CELL = "shell：函数里多行 awk 程序带一行与函数收尾同缩进的 }：函数照引号外的 } 收尾，awk 之后的 bash s/child.sh 连上（改它键变）"
SHELL_TAIL_GROUP_CELL = "shell：bash \"$<折不出的目录>/$<折得出的变量>/driver.sh\" 连到仓里以 .sample/driver.sh 收尾的那一组（改它键变）"
SHELL_TAIL_OUTSIDE_CELL = "shell：bash \"$<折不出的目录>/.nowhere/run.sh\" 仓里没有以它收尾的脚本：不是仓里的脚本，不拒算"
SHELL_ARGUMENTS_SKIPPED_CELL = "shell：bash s/dispatch.sh run：if [[ \"${1:-}\" == --selftest ]] 那一支走不到，它起的 s/child.sh 不连（改它键不变）"
SHELL_ARGUMENTS_TAKEN_CELL = "shell：bash s/dispatch.sh run：case \"$1\" 里 run) 那一支走到，它起的 s/tools/one.py 连上（改它键变）"
SHELL_ARGUMENTS_OTHER_CASE_CELL = "shell：bash s/dispatch.sh run：case \"$1\" 里 *) 那一支走不到，它起的 s/tools/two.py 不连（改它键不变）"
FOLD_NESTED_CELL = "python：套在函数里的函数起子进程，它的局部变量与外层的变量照那一层作用域折，连到那份脚本"
FOLD_CONCATENATION_CELL = "python：git + step（step 是循环里取的、折不出）命令位置上是 git，不拒算"
FOLD_SLICE_CELL = "python：f(…)[k] 取返回的元组那一项、command[:command.index(<字面词>)] 照切，连到那份脚本"
FOLD_VERSIONS_CELL = "python：方法参数按每一处 .方法(…) 调用折，推导式变量取遍函数返回的字典里的列表，命令位置上折出几个各认一遍（python3 / bash --version 不跑代码）"
FOLD_LOADED_CELL = "python：spec_from_file_location 载入的模块的常量（module.NAME）与模块级元组[整数] 折得出，sh -c 的串连到它起的脚本"
FOLD_TAIL_GROUP_CELL = "python：os.path.join(<折不出的目录>, \"tools\", \"runner.py\") 连到仓里以 tools/runner.py 收尾的那一份"
FOLD_OUTSIDE_SCRIPT_CELL = "python：os.path.join(<折不出的目录>, \"generated\", \"made.py\") 仓里没有以它收尾的文件：不是仓里的脚本，不拒算"
FOLD_CWD_GROUP_CELL = "python：[\"bash\", \"setup.sh\"] 在折不出的 cwd= 里起：仓里以 /setup.sh 收尾的整组连上"
FOLD_SUBMITTED_CELL = "python：pool.submit(f, 实参) 算一处调用，f 的命令参数按它折，连到那份脚本"
FOLD_ENVIRONMENT_CELL = "python：env -u X NAME={折不出} 之后的命令照命令位置认，连到那份脚本"
DATA_READ_CHANGED_CELL = "python：运行期 open 读的仓内文件（路径折得出）进键：改它键变"
DATA_READ_UNRELATED_CELL = "python：运行期 open 读的仓内文件进键：改不相干的文件键不变"
DATA_READ_TAIL_CELL = "python：读 <sys.argv 给的目录>/pkg/table.tsv 照字面尾巴收仓里那一份：改它键变"
DATA_READ_SCRATCH_CELL = "python：读这一趟自己在 mkdtemp 建的目录里写的文件（仓里没有这一截尾巴）不拒算"
DATA_READ_REFUSED_CELL = "python：读 os.path.join(<根本身>, <读进来的登记行点名的一截>) 拒算 data-path-unfolded"
DATA_READ_DIRECTORY_CELL = "python：读 os.path.join(<根里的目录>, <出处推不出的一截>) 收那个目录整棵：改目录里的一份键变"
DATA_READ_LIBRARY_CELL = "python：别的模块里的函数读它参数给的文件：闭包里的调用处给仓里的常量收进键（改它键变），给入口参数的不收也不拒算"
DATA_READ_ATTRIBUTE_CELL = "python：读 self.<属性>：照本类方法里给它的赋值认，赋的是仓里的常量就收进键（改它键变）"
TOOLCHAIN_QUERY_CELL = "python：[<环境给的编译器>, \"-V\"] 是工具链询问：不拒算，工具链单元进键：换一版假 rustc 键变，闭包里没有询问的换 rustc 键不变"
CALLER_LAUNCH_CELL = "python：判法入口的参数给的命令（-c 的串、解释器）是调用方交进来的：不拒算"
REGISTRY_PROGRAMS_CELL = "python：登记表点名的程序（REGISTRY_LISTED_PROGRAMS）是环境探针：不连、不拒算，改登记表点名的一份探针，起探针的入口键不变"
ROW_KEYED_CELL = "按行进键的登记表：判法经登记的取用函数按入口查表里一行，改表里与这一条无关的行键不变、改它查的那一行键变；读整张表的不换"
PROGRAM_ARGUMENTS_CELL = "python：同一程序入口带两组参数：只改另一组参数才走到的函数，这一组键不变、改这一组走到的键变；起它的脚本把参数带过去照样剪"
DATA_READ_INPUT_CELL = "python：入口函数读它参数给的文件（调用方的输入）不拒算"
DATA_READ_PARSED_CELL = "python：读 argparse 解出来的路径（调用方的输入）不拒算"
DATA_READ_LISTING_CELL = "python：读 os.path.join(<根本身>, <算出来的名单里的一截>)（不是读根里的文件得来的）：被判的输入，不拒算"
DATA_READ_COPY_CELL = "python：拷文件的源 os.path.join(<根本身>, <读进来的登记行点名的一截>)：拷副本，不拒算"
TAIL_ANCHOR_SCRATCH_CELL = "python：读 <参数>/<字面尾巴>，闭包里的调用处给的是这一趟的临时目录：仓里这一截尾巴的那一组不收，改其中一份键不变"
TAIL_ANCHOR_JUDGE_CELL = "python：读 <判法入口的参数>/<字面尾巴>（被判的输入）：仓里这一截尾巴的那一组不收，改其中一份键不变"
TAIL_ANCHOR_ARGUMENT_CELL = "python：读 <参数>/<字面尾巴>，闭包里的调用处给的是 sys.argv：照尾巴收仓里那一组，改其中一份键变"
TAIL_ANCHOR_FIXTURE = {
    "pkg/__init__.py": "",
    "fixtures/sample/Manifest.toml": "name = sample\n",
    "pkg/manifest_reader.py": '''import os
import sys
import tempfile


def package_name(tree, relative):
    directory = os.path.dirname(os.path.join(tree, relative))
    while True:
        manifest = os.path.join(directory, "Manifest.toml")
        if os.path.isfile(manifest):
            return open(manifest, encoding="utf-8").read()
        if directory == tree:
            return None
        directory = os.path.dirname(directory)


def judge(tree):
    return package_name(tree, "src/lib.rs")


def scratch_run():
    with tempfile.TemporaryDirectory() as work:
        return package_name(os.path.join(work, "tree"), "src/lib.rs")


def from_argument():
    return package_name(sys.argv[1], "src/lib.rs")
''',
}
LAUNCH_ARGUMENT_CELL = ("python：命令是别的函数的参数（subprocess.Popen(command)、[program, *arguments]）：照闭包里每个调用点的实参折"
                        "（字面程序、根外程序、外层函数的变量、调仓内函数的返回值），不拒算")
LAUNCH_ARGUMENT_SCRIPT_CELL = "python：命令参数在调用点折出根里的脚本（run_tool(\"pkg/tool.sh\", …)）：连上那一份，改它键变"
LAUNCH_ARGUMENT_REFUSED_CELL = "python：命令参数在调用点给的是环境变量（os.environ[…]）：折不出，照旧拒算 subprocess-command-unfolded"
LAUNCH_ARGUMENT_FIXTURE = {
    "pkg/__init__.py": "",
    "pkg/tool.sh": "#!/bin/bash\necho tool\n",
    "pkg/launch.py": '''import os
import subprocess


def run_group(command, **options):
    return subprocess.Popen(command, stdout=subprocess.PIPE, start_new_session=True, **options)


def run_tool(program, arguments):
    return run_group([program, *arguments], cwd="/")


def tool_program(executable, wrapped):
    if not wrapped:
        return executable
    return os.path.join(os.path.dirname(executable), "tool-wrapper")


def installed_tool(directory):
    executable = os.path.join(directory, "tool-binary")
    return executable, "1.0"


def prepared(options):
    executable, _version = installed_tool(options["tool_root"])
    return options, executable


def check(root, options):
    _options, executable = prepared(options)
    run_tool("cargo", ["build"])

    def one(wrapped):
        return run_tool(tool_program(executable, wrapped), ["--check"])
    return [one(True), one(False)]


def check_script(root, options):
    _options, executable = prepared(options)
    run_tool("pkg/tool.sh", ["--check"])
    return run_tool(tool_program(executable, True), [])


def check_unfolded(root):
    return run_tool(os.environ["TOOL_PROGRAM"], ["--check"])


def selftest_sample():
    run_group(["bash", "-c", "echo sample"])
    return tool_program("/opt/tool/bin/tool-binary", False)
''',
}


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


def with_breaks_added(judge, *switches):
    """这一格在当时设着的弄坏开关之上再打开 switches 判（只看一条边、别的边先关掉的格用）：BREAK_CELLS 轮打开点名的开关时，两样一起开着。"""
    def judge_with_breaks(work):
        previous = os.environ.get(BREAK_VARIABLE)
        os.environ[BREAK_VARIABLE] = ",".join(sorted(active_breaks() | set(switches)))
        try:
            return judge(work)
        finally:
            if previous is None:
                os.environ.pop(BREAK_VARIABLE, None)
            else:
                os.environ[BREAK_VARIABLE] = previous
    return judge_with_breaks


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


# 闭包外的四类边（文件头 Rust 闭包「闭包外的四类边」）：每格一个小工作区，改的那一处补边前不在入口的闭包里（键不变），补边后在（键变）；
# 每格配一个弄坏开关（BREAK_CELLS 里 rust-macro-generated-types-unnamed 到 rust-excluded-path-dependencies-dropped 那六项）退回补边前；
# 工作区外的 crate 按调用关系连的两格（OUTSIDE_IMPORT_CELL、OUTSIDE_MODULE_CELL）各配 rust-outside-crate-* 一个开关，退回按同名连
MACRO_EDGE_FIXTURE = {
    "crates/fmacro/Cargo.toml": '[package]\nname = "fmacro"\n',
    "crates/fmacro/src/lib.rs": '''macro_rules! listed_enumeration {
    ($(#[$attribute:meta])* $visibility:vis enum $name:ident { $($variant:ident,)+ }) => {
        $(#[$attribute])*
        $visibility enum $name { $($variant,)+ }
    };
}
listed_enumeration! {
    pub enum Weight {
        Light,
        Heavy,
    }
}
impl Weight {
    pub fn amount(self) -> u32 {
        match self {
            Weight::Light => 1,
            Weight::Heavy => 9,
        }
    }
}
pub fn heavy_amount() -> u32 {
    Weight::Heavy.amount()
}
macro_rules! code_table {
    ($($constant:ident = $code:literal,)*) => {
        $(pub const $constant: u32 = $code;)*
        pub const EVERY_CODE: &[u32] = &[$($constant,)*];
    };
}
code_table! {
    CODE_NONE = 0,
    CODE_FULL = 7,
}
pub fn code_count() -> usize {
    EVERY_CODE.len()
}
''',
}
MACRO_EDGE_FILE = "crates/fmacro/src/lib.rs"
ALLOCATOR_EDGE_FIXTURE = {
    "crates/falloc/Cargo.toml": '[package]\nname = "falloc"\n',
    "crates/falloc/src/main.rs": '''use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

static GROWTHS: AtomicUsize = AtomicUsize::new(0);

struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        GROWTHS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(pointer, layout, size) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn growths() -> usize {
    let mut values = Vec::new();
    values.push(1_u8);
    GROWTHS.load(Ordering::Relaxed)
}

fn main() {
    println!("{}", growths());
}
''',
}
FOREIGN_IMPL_EDGE_FIXTURE = {
    "crates/fforeign/Cargo.toml": '[package]\nname = "fforeign"\n',
    "crates/fforeign/src/lib.rs": '''pub trait Measured {
    fn measure(&self) -> u64;
}

impl Measured for std::time::Duration {
    fn measure(&self) -> u64 {
        self.as_secs() + 1
    }
}

pub fn measure_of<Value>(value: &Value) -> u64
where
    Value: Measured,
{
    value.measure()
}

pub fn measured_second() -> u64 {
    measure_of(&std::time::Duration::from_secs(1))
}
''',
}
BUILD_ENVIRONMENT_EDGE_FIXTURE = {
    "crates/fbuild/Cargo.toml": '[package]\nname = "fbuild"\n',
    "crates/fbuild/build.rs": '''fn reported_features() -> String {
    String::from("plain")
}

fn main() {
    println!("cargo::rustc-env=FBUILD_REPORTED_FEATURES={}", reported_features());
}
''',
    "crates/fbuild/src/lib.rs": '''pub fn reported() -> &'static str {
    env!("FBUILD_REPORTED_FEATURES")
}
''',
}
EXCLUDED_PATH_DEPENDENCY_EDGE_FIXTURE = {
    "Cargo.toml": '[workspace]\nmembers = ["crates/fuser"]\nexclude = ["fservice"]\n',
    "crates/fuser/Cargo.toml": '[package]\nname = "fuser"\n\n[dependencies]\nfservice = { path = "../../fservice" }\n',
    "crates/fuser/src/lib.rs": "pub fn stop_count() -> u32 {\n    fservice::stopped_connections()\n}\n",
    "fservice/Cargo.toml": '[package]\nname = "fservice"\n\n[workspace]\n',
    "fservice/src/lib.rs": "pub fn stopped_connections() -> u32 {\n    3\n}\n",
}
# 工作区外的 crate 按调用关系连（文件头 Rust 闭包「工作区外的 path 依赖」后半）：fuser 只在字段名里写着 halted；fservice 里 log 与 bench 两个模块各有一个 Log，
# bench 模块里的 fn launch 与 Worker::launch 同名
OUTSIDE_CRATE_NAME_FIXTURE = {
    "Cargo.toml": '[workspace]\nmembers = ["crates/fuser"]\nexclude = ["fservice"]\n',
    "crates/fuser/Cargo.toml": '[package]\nname = "fuser"\n\n[dependencies]\nfservice = { path = "../../fservice" }\n',
    "crates/fuser/src/lib.rs": '''use fservice::control;
use fservice::restarts;

pub struct Counters {
    pub halted: u32,
}

pub fn served() -> u32 {
    let counters = Counters { halted: 1 };
    fservice::stopped_connections() + restarts() + control::paused() + counters.halted
}
''',
    "fservice/Cargo.toml": '[package]\nname = "fservice"\n\n[workspace]\n',
    "fservice/src/lib.rs": '''pub mod control {
    pub fn paused() -> u32 {
        5
    }
}

pub mod bench {
    pub struct Log {
        pub entries: u64,
    }

    pub fn launch() -> u32 {
        7
    }
}

pub mod log {
    pub struct Log {
        pub entries: u32,
    }
}

use log::Log;

pub struct Worker;

impl Worker {
    fn launch(&self) -> u32 {
        2
    }
}

pub fn stopped_connections() -> u32 {
    Worker.launch() + Log { entries: 1 }.entries
}

pub fn restarts() -> u32 {
    4
}

pub fn halted() -> u32 {
    6
}
''',
}
OUTSIDE_CRATE_NAME_ENTRY = "crates/fuser/src/lib.rs::served"
OUTSIDE_CRATE_NAME_FILE = "fservice/src/lib.rs"
OUTSIDE_IMPORT_CELL = ("工作区外的 crate 按引进的名字连：别的包里只是同名的字段 halted 不连到它的 fn halted，改它键不变；"
                       "use 引进的 restarts、use 引进的模块下的 control::paused、写成路径的 stopped_connections 照连，改它们键变")
OUTSIDE_MODULE_CELL = ("工作区外的 crate 里按模块连：use 引进 log::Log 的只连它、不连 bench 模块里同名的 Log，Worker::launch 自己的名字不连 bench::launch；"
                       "改 bench 模块里这两样键不变，改 log::Log、改 Worker::launch 键变")
CRATES_MODULE_PRIVATE_CELL = "R-裸名：crates/ 下的包同一个目标里按模块连：经 use 引进 format::SLOT_BYTES 的单元不连 gpu_checks 里私有的同名常量，改它键不变"
CRATES_MODULE_IMPORTED_CELL = "R-裸名：crates/ 下的包同一个目标里按模块连：经 use 引进的 format::SLOT_BYTES 照连，改它键变"
CRATES_MODULE_FIXTURE = {
    "crates/fmodules/Cargo.toml": '[package]\nname = "fmodules"\n',
    "crates/fmodules/src/lib.rs": "pub mod format;\nmod gpu_checks;\nmod cpu_checks;\n",
    "crates/fmodules/src/format.rs": "pub const SLOT_BYTES: usize = 64;\n",
    "crates/fmodules/src/gpu_checks.rs": "const SLOT_BYTES: usize = 32;\n\npub fn gpu_slots() -> usize {\n    SLOT_BYTES * 2\n}\n",
    "crates/fmodules/src/cpu_checks.rs": "use crate::format::SLOT_BYTES;\n\npub fn cpu_slots() -> usize {\n    SLOT_BYTES + 1\n}\n",
}


def outside_crate_edit_verdicts(work, edits):
    """OUTSIDE_CRATE_NAME_FIXTURE 上入口 served 的键：逐个改 edits 里的 (说明, 旧, 新)，每改一处跟前一把比 → ([(说明, changed / same / 拒算)], 改动前的键)。"""
    base = os.path.join(work, "tree")
    write_tree(base, OUTSIDE_CRATE_NAME_FIXTURE)
    previous, _ = fixture_key(base, OUTSIDE_CRATE_NAME_ENTRY)
    first, verdicts = previous, []
    for label, old, new in edits:
        edit_file(base, OUTSIDE_CRATE_NAME_FILE, old, new)
        current, refused = fixture_key(base, OUTSIDE_CRATE_NAME_ENTRY)
        if previous is None or current is None:
            verdicts.append((label, f"拒算 {sorted(refused)}"))
        else:
            verdicts.append((label, "changed" if current != previous else "same"))
        previous = current
    return verdicts, first


def outside_import_cell(work):
    """别的包按裸名连进工作区外的 crate 要经 use 或路径：halted 只是 fuser 的字段名，改 fservice 的 fn halted 键不变；
    use 引进的 restarts、经 use 引进的模块 control 写成 control::paused、写成 fservice::stopped_connections 的，改了键都变。"""
    verdicts, first = outside_crate_edit_verdicts(work, (("fn halted", "    6\n", "    60\n"), ("fn restarts", "    4\n", "    40\n"),
                                                         ("control::paused", "        5\n", "        50\n"),
                                                         ("fn stopped_connections", "Worker.launch() + Log", "Worker.launch() + 1 + Log")))
    wanted = [("fn halted", "same"), ("fn restarts", "changed"), ("control::paused", "changed"), ("fn stopped_connections", "changed")]
    return first is not None and verdicts == wanted, f"期望 {wanted}，实际 {verdicts}（改前键 {first}）"


def outside_module_cell(work):
    """工作区外 crate 的同一个目标里按模块连：stopped_connections 经 use log::Log 用 Log，bench 模块里同名的 Log 不连，改它键不变；
    Worker::launch 自己的名字不连到 bench 模块里的 fn launch，改它键不变；改 log::Log、改 Worker::launch 的体，键都变。"""
    verdicts, first = outside_crate_edit_verdicts(work, (("bench::Log", "pub entries: u64,", "pub entries: u16,"), ("bench::launch", "        7\n", "        70\n"),
                                                         ("log::Log", "pub entries: u32,", "pub entries: u8,"), ("Worker::launch", "        2\n", "        20\n")))
    wanted = [("bench::Log", "same"), ("bench::launch", "same"), ("log::Log", "changed"), ("Worker::launch", "changed")]
    return first is not None and verdicts == wanted, f"期望 {wanted}，实际 {verdicts}（改前键 {first}）"


MACRO_TYPE_EDGE_CELL = "R-宏：条目级宏调用里 enum 定义的类型算被点名，.方法() 连到写给它的 impl，改方法作废"
MACRO_LITERAL_EDGE_CELL = "R-宏：macro_rules! 体里按字面写出的常量名登到这个宏的每次调用名下，点名它的入口连上调用，改调用体作废"
ALLOCATOR_EDGE_CELL = "#[global_allocator]：链接集里的分配器 static 进闭包，连上它的 GlobalAlloc impl，改 realloc 作废"
FOREIGN_IMPL_EDGE_CELL = "R-方法：本仓 trait 写在外部类型上的 impl（impl Tr for std::time::Duration）经泛型约束调到，方法连上，改它作废"
BUILD_ENVIRONMENT_EDGE_CELL = "构建脚本经 cargo::rustc-env 交的值：用 env! 读它的单元连到本 crate 构建脚本的 main，改 main 里算的值作废"
EXCLUDED_PATH_DEPENDENCY_EDGE_CELL = "工作区外的 path 依赖（根清单 exclude 里）：顺 path 依赖收进来，调到它的函数连上，改它作废"


def closure_edge_cells():
    return [
        (MACRO_TYPE_EDGE_CELL, compare_cell(MACRO_EDGE_FIXTURE, f"{MACRO_EDGE_FILE}::heavy_amount", MACRO_EDGE_FILE,
                                            "Weight::Heavy => 9,", "Weight::Heavy => 8,", "changed")),
        (MACRO_LITERAL_EDGE_CELL, compare_cell(MACRO_EDGE_FIXTURE, f"{MACRO_EDGE_FILE}::code_count", MACRO_EDGE_FILE,
                                               "CODE_FULL = 7,", "CODE_FULL = 8,", "changed")),
        (ALLOCATOR_EDGE_CELL, compare_cell(ALLOCATOR_EDGE_FIXTURE, "crates/falloc/src/main.rs::growths", "crates/falloc/src/main.rs",
                                           "GROWTHS.fetch_add(1, Ordering::Relaxed);", "GROWTHS.fetch_add(2, Ordering::Relaxed);", "changed")),
        (FOREIGN_IMPL_EDGE_CELL, compare_cell(FOREIGN_IMPL_EDGE_FIXTURE, "crates/fforeign/src/lib.rs::measured_second", "crates/fforeign/src/lib.rs",
                                              "self.as_secs() + 1", "self.as_secs() + 2", "changed")),
        (BUILD_ENVIRONMENT_EDGE_CELL, compare_cell(BUILD_ENVIRONMENT_EDGE_FIXTURE, "crates/fbuild/src/lib.rs::reported", "crates/fbuild/build.rs",
                                                   'String::from("plain")', 'String::from("gpu")', "changed")),
        (EXCLUDED_PATH_DEPENDENCY_EDGE_CELL, compare_cell(EXCLUDED_PATH_DEPENDENCY_EDGE_FIXTURE, "crates/fuser/src/lib.rs::stop_count", "fservice/src/lib.rs",
                                                          "    3\n", "    4\n", "changed")),
        (OUTSIDE_IMPORT_CELL, outside_import_cell),
        (OUTSIDE_MODULE_CELL, outside_module_cell),
        (CRATES_MODULE_PRIVATE_CELL, compare_cell(CRATES_MODULE_FIXTURE, "crates/fmodules/src/cpu_checks.rs::cpu_slots", "crates/fmodules/src/gpu_checks.rs",
                                                  "usize = 32;", "usize = 33;", "same")),
        (CRATES_MODULE_IMPORTED_CELL, compare_cell(CRATES_MODULE_FIXTURE, "crates/fmodules/src/cpu_checks.rs::cpu_slots", "crates/fmodules/src/format.rs",
                                                   "usize = 64;", "usize = 65;", "changed")),
    ]


# 4-3d：键的 cfg 成分收窄、WGSL 外名的 _ 后缀与供名字的边、trait 成员里的 Self::x、文档测试不连结构、use a::{self} 的小工作区；
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
# 同一份样本再加一个测试模块：入口是里面一条测试（按测试编，测试单元都编），另一条测试的串里写着外名、函数体读运行期的清单目录（拒算 runtime-path）
WGSL_TEST_SUPPLIER_FIXTURE = dict(WGSL_SUPPLY_FIXTURE)
WGSL_TEST_SUPPLIER_FIXTURE[WGSL_SUPPLY_FILE] = WGSL_SUPPLY_FIXTURE[WGSL_SUPPLY_FILE] + '''#[cfg(test)]
mod tests {
    #[test]
    fn the_supplied_kernel_text_is_built() {
        assert!(!super::supplied_kernel_text().is_empty());
    }
    #[test]
    fn the_words_of_the_failure_points_are_listed() {
        let words = "FAILURE_POINT_UNREADABLE FAILURE_POINT_NONE";
        let _root = std::env::var_os("CARGO_MANIFEST_DIR");
        assert!(!words.is_empty());
    }
}
'''
WGSL_TEST_SUPPLIER_CELL = "WGSL 外名的着落不收测试函数：串里有外名的 #[test] fn 不并进闭包，改它的函数体键不变，改真给名字的宏调用键变"


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
    """供外名的条目级宏调用不经调用体里的名字点到（拼内核的函数只用宏体里定义的 GPU_FAILURE_POINTS）：着落做成边，宏调用并进闭包，改它里面的码键变。
    登记时套 with_breaks_added 关掉宏体字面名字那条边，只看着落。"""
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


def wgsl_test_supplier_cell(work):
    """入口是一条测试，WGSL 外名只有宏调用真给着落；同一模块里另一条测试的串里也写着这两个外名、函数体读运行期的清单目录：
    测试函数不当名字供给者收（不并进闭包、不带进它的 runtime-path 拒算），键算得出；改那条测试的函数体键不变；改宏调用里的码键变。
    登记时套 with_breaks_added 关掉宏体字面名字那条边：开着它，宏调用经 GPU_FAILURE_POINTS 进闭包、外名都有着落，供给者一个都不拉，测试函数收不收判不出。"""
    base = os.path.join(work, "tree")
    write_tree(base, WGSL_TEST_SUPPLIER_FIXTURE)
    entry = f"{WGSL_SUPPLY_FILE}::tests::the_supplied_kernel_text_is_built"
    pulled = "fn tests::the_words_of_the_failure_points_are_listed" in closure_member_names(base, entry)
    before, before_refused = fixture_key(base, entry)
    edit_file(base, WGSL_SUPPLY_FILE, 'let words = "FAILURE_POINT_UNREADABLE FAILURE_POINT_NONE";', 'let words = "FAILURE_POINT_UNREADABLE FAILURE_POINT_NONE，改了";')
    outside, _ = fixture_key(base, entry)
    edit_file(base, WGSL_SUPPLY_FILE, "GPU_FAILURE_POINT_UNREADABLE = 9,", "GPU_FAILURE_POINT_UNREADABLE = 99,")
    inside, _ = fixture_key(base, entry)
    ok = not pulled and before is not None and outside == before and inside is not None and inside != before
    return ok, (f"测试函数并进了闭包 {pulled}（要没有）；键 {before}（拒算 {sorted(before_refused)}）、改测试函数体后 {outside}（要同前）、改宏调用里的码后 {inside}（要不同）")


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
# Box、Option 与 u32 在 PRIMITIVE_TYPES 里、算被点名，这五份总是进闭包；Sealed<T> 没被点名，只有按 blanket 放行才进
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
        # 这两格只看着落那条边：GPU_FAILURE_POINTS 是宏体展开式顶层按字面写出的名字，宏体字面名字那条边（文件头 Rust 闭包「条目级宏调用生成的条目」）
        # 也让 failure_points! 调用进闭包，开着它两格的弄坏开关红不了；crates/singlefs-unit-graph 的 unit_graph_wgsl_modules_close_over_compute_entries.rs 同样关它
        ("WGSL 外名的着落做成边：只靠外名点到的条目级宏调用并进闭包，改它里面的码键变",
         with_breaks_added(wgsl_supplier_edge_cell, "rust-macro-rules-literal-names-unlinked")),
        (WGSL_TEST_SUPPLIER_CELL, with_breaks_added(wgsl_test_supplier_cell, "rust-macro-rules-literal-names-unlinked")),
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


# Rust 闭包读到的 shell 脚本（文件头「Rust 闭包读到的 shell 脚本」）：入口照 gpu_card_configuration.rs 起配置判法的写法拼出脚本路径
SHELL_DATA_FIXTURE = {
    "crates/fshell/Cargo.toml": '[package]\nname = "fshell"\n',
    "crates/fshell/src/lib.rs": '''pub fn runs_the_sourcing_script() -> bool {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/sourcing.sh");
    std::process::Command::new("bash").arg(&script).status().is_ok()
}
pub fn runs_the_plain_script() -> bool {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/plain.sh");
    std::process::Command::new("bash").arg(&script).status().is_ok()
}
pub fn runs_the_dotted_script() -> bool {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/dotted.sh");
    std::process::Command::new("bash").arg(&script).status().is_ok()
}
pub fn runs_the_unfolded_script() -> bool {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/unfolded.sh");
    std::process::Command::new("bash").arg(&script).status().is_ok()
}
''',
    "scripts/sourcing.sh": '''#!/usr/bin/env bash
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/lib/first.sh"
first_function
''',
    "scripts/lib/first.sh": '''#!/usr/bin/env bash
first_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$first_directory/second.sh"
first_function() {
  echo first
}
''',
    "scripts/lib/second.sh": '''#!/usr/bin/env bash
second_function() {
  echo second
}
''',
    "scripts/dotted.sh": '''#!/usr/bin/env bash
SCRIPT_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "$SCRIPT_DIRECTORY/lib/dot-only.sh"
dot_function
''',
    "scripts/lib/dot-only.sh": '''#!/usr/bin/env bash
dot_function() {
  echo dot
}
''',
    "scripts/plain.sh": '''#!/usr/bin/env bash
echo plain
''',
    "scripts/unfolded.sh": '''#!/usr/bin/env bash
library="$1"
source "$library"
''',
}
SHELL_DATA_FILE = "crates/fshell/src/lib.rs"
SHELL_DATA_HOST_LINE = "host: x86_64-unknown-linux-gnu"
SHELL_DATA_SOURCE_CELL = ("Rust 闭包读到的 shell 脚本：它 source 进来的文件收成 data-file 单元，改一处读这份脚本的入口键变、读另一份不 source 它的脚本的入口键不变；"
                          "被 source 的每一份按读这份脚本的那一处记一条数据位置，unit_graph_key（与 crates/singlefs-unit-graph 对拍的那一把）随它变")
SHELL_DATA_DOT_CELL = "Rust 闭包读到的 shell 脚本：. <路径> 照 source 收（路径经 $(cd \"$(dirname \"${BASH_SOURCE[0]}\")\" && pwd) 赋的变量），改被引的文件键变"
SHELL_DATA_NESTED_CELL = "Rust 闭包读到的 shell 脚本：两层（脚本 source 的库再 source 一份）逐层收，改里层那份键变"
SHELL_DATA_UNFOLDED_CELL = "Rust 闭包读到的 shell 脚本：source 的路径折不出拒算 shell-path-unfolded"


def shell_data_edit_trip(work, entry, path, old, new):
    """→ (改动前的键, 改后键, 改动前闭包里经 shell-source 收进来的文件, 改动前与改后的 unit_graph_key)。"""
    base = os.path.join(work, "tree")
    write_tree(base, SHELL_DATA_FIXTURE)
    closure = closure_of(base, f"{SHELL_DATA_FILE}::{entry}")
    sourced = sorted(unit.file for unit in closure.order if closure.via.get(id(unit), (None, ""))[1] == "shell-source")
    graph_before = unit_graph_key(closure, [SHELL_DATA_HOST_LINE])[0]
    before = closure_key(closure)[0]
    edit_file(base, path, old, new)
    changed = closure_of(base, f"{SHELL_DATA_FILE}::{entry}")
    return before, closure_key(changed)[0], sourced, (graph_before, unit_graph_key(changed, [SHELL_DATA_HOST_LINE])[0])


def shell_data_source_cell(work):
    before, after, sourced, graphs = shell_data_edit_trip(work, "runs_the_sourcing_script", "scripts/lib/first.sh", "  echo first\n", "  echo first, changed\n")
    base = os.path.join(work, "tree")
    plain_after = fixture_key(base, f"{SHELL_DATA_FILE}::runs_the_plain_script")[0]
    edit_file(base, "scripts/lib/first.sh", "  echo first, changed\n", "  echo first\n")
    plain_before = fixture_key(base, f"{SHELL_DATA_FILE}::runs_the_plain_script")[0]
    # 数据位置（data_sites：读它的文件、行、种类）：被 source 的两份都落在 lib.rs 里读 sourcing.sh 的那一处（crates/singlefs-unit-graph 的 follow_shell_sources 同）
    sites = {site[2]: (site[0], site[1], site[5]) for site in closure_of(base, f"{SHELL_DATA_FILE}::runs_the_sourcing_script").data_sites}
    reader = sites.get("scripts/sourcing.sh")
    sited = reader is not None and all(sites.get(path) == reader for path in ("scripts/lib/first.sh", "scripts/lib/second.sh"))
    ok = ("scripts/lib/first.sh" in sourced and before is not None and after is not None and before != after
          and plain_before is not None and plain_before == plain_after and sited and graphs[0] is not None and graphs[1] is not None and graphs[0] != graphs[1])
    return ok, (f"经 shell-source 收进来的 {sourced}（要有 scripts/lib/first.sh）；改它前后键 {before and before[:12]} → {after and after[:12]}（要都有、不同）；"
                f"读 plain.sh 的入口改前改后 {plain_before and plain_before[:12]} / {plain_after and plain_after[:12]}（要都有、相同）；"
                f"数据位置 {dict(sorted(sites.items()))}（被 source 的 first.sh、second.sh 要与 sourcing.sh 落在同一处）；"
                f"unit_graph_key {graphs[0] and graphs[0][:12]} → {graphs[1] and graphs[1][:12]}（要都有、不同）")


def shell_data_dot_cell(work):
    before, after, sourced, _graphs = shell_data_edit_trip(work, "runs_the_dotted_script", "scripts/lib/dot-only.sh", "  echo dot\n", "  echo dot, changed\n")
    ok = sourced == ["scripts/lib/dot-only.sh"] and before is not None and after is not None and before != after
    return ok, f"经 shell-source 收进来的 {sourced}（要 [scripts/lib/dot-only.sh]）；改它前后键 {before and before[:12]} → {after and after[:12]}（要都有、不同）"


def shell_data_nested_cell(work):
    before, after, sourced, _graphs = shell_data_edit_trip(work, "runs_the_sourcing_script", "scripts/lib/second.sh", "  echo second\n", "  echo second, changed\n")
    ok = sourced == ["scripts/lib/first.sh", "scripts/lib/second.sh"] and before is not None and after is not None and before != after
    return ok, (f"经 shell-source 收进来的 {sourced}（要 [scripts/lib/first.sh, scripts/lib/second.sh]）；改里层 second.sh 前后键 "
                f"{before and before[:12]} → {after and after[:12]}（要都有、不同）")


MANIFEST_ROOT_CELL = "Rust：清单目录往上拼出仓根（目录）拒算 runtime-path；点名清单目录里一份文件再往上取仓根的不拒算、那份文件的字节进键"
# 实验准入入口（scripts/experiment-apparatus/lib.rs::admit_experiment_run_or_exit）取仓根的两种写法
MANIFEST_ROOT_FIXTURE = {
    "crates/fbench/Cargo.toml": '[package]\nname = "fbench"\n',
    "crates/fbench/src/lib.rs": '''// 只是注释
pub fn root_by_parent_directories() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}
pub fn root_by_named_source() -> std::path::PathBuf {
    let this_source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("lib.rs");
    this_source.ancestors().nth(4).expect("lib.rs 在 <根>/crates/fbench/src/ 下").to_path_buf()
}
''',
}


def manifest_root_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, MANIFEST_ROOT_FIXTURE)
    directory_key, directory_forms = fixture_key(base, "crates/fbench/src/lib.rs::root_by_parent_directories")
    before, named_forms = fixture_key(base, "crates/fbench/src/lib.rs::root_by_named_source")
    edit_file(base, "crates/fbench/src/lib.rs", "// 只是注释\n", "// 只是注释，改了措辞\n")
    after, _ = fixture_key(base, "crates/fbench/src/lib.rs::root_by_named_source")
    ok = directory_key is None and "runtime-path" in directory_forms and before is not None and after is not None and before != after
    return ok, (f"往上拼目录的：键{'有' if directory_key else '无'}、拒算 {sorted(directory_forms)}（要拒算 runtime-path）；"
                f"点名文件的：拒算 {sorted(named_forms)}，改那份文件里一句注释前后键{'相同' if before == after else '不同'}（要算得出且不同）")


CURRENT_EXE_FIXTURE = {
    "crates/fexe/Cargo.toml": '[package]\nname = "fexe"\n',
    "crates/fexe/src/lib.rs": '''#[cfg(test)]
mod tests {
    use std::process::Command;
    const CHILD_NAME: &str = "the_child_runs_alone";
    const CHILD_TEST: &str = "tests::the_child_runs_alone";
    fn run_the_child_by_name(name: &str) {
        let output = Command::new(std::env::current_exe().expect("路径")).args(["--exact", name, "--nocapture"]).output().expect("起得来");
        assert!(output.status.success());
    }
    #[test]
    fn the_parent_starts_the_child_by_a_constant() {
        let output = std::process::Command::new(std::env::current_exe().expect("路径")).args(["--exact", CHILD_TEST]).output().expect("起得来");
        assert!(output.status.success());
    }
    #[test]
    fn the_parent_starts_the_child_through_a_parameter() {
        run_the_child_by_name(CHILD_TEST);
    }
    #[test]
    fn the_parent_names_the_child_by_a_format_of_literals_and_constants() {
        let output = Command::new(std::env::current_exe().expect("路径")).arg("--exact").arg(format!("{}::{CHILD_NAME}", "tests")).output().expect("起得来");
        assert!(output.status.success());
    }
    #[test]
    fn the_child_runs_alone() {
        let value = 1;
        assert_eq!(value, 1);
    }
    #[test]
    fn the_unrelated_test() {
        let note = 5;
        assert_eq!(note, 5);
    }
    #[test]
    fn the_parent_asks_for_a_name_nobody_defines() {
        let _ = Command::new(std::env::current_exe().expect("路径")).args(["--exact", "tests::missing"]).output();
    }
    #[test]
    fn the_parent_names_itself_before_the_exact_flag_by_a_module_path() {
        let test_name = format!("{}::x", module_path!().split_once("::").expect("包名").1);
        let _ = Command::new(std::env::current_exe().expect("路径")).args([test_name.as_str(), "--exact"]).output();
    }
}
''',
}
CURRENT_EXE_CELL = "Rust：current_exe() 交给 Command::new 起子进程、--exact 后面点名的测试（常量、参数、字面与常量拼的）连到那条 #[test]，改它的函数体键变、改别的测试键不变；点名不存在的、测试名写在 --exact 前面的照旧拒算 current-exe"
CURRENT_EXE_KEYED = ("the_parent_starts_the_child_by_a_constant", "the_parent_starts_the_child_through_a_parameter",
                     "the_parent_names_the_child_by_a_format_of_literals_and_constants")
CURRENT_EXE_REFUSED = ("the_parent_asks_for_a_name_nobody_defines", "the_parent_names_itself_before_the_exact_flag_by_a_module_path")


def current_exe_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, CURRENT_EXE_FIXTURE)
    file = "crates/fexe/src/lib.rs"
    kept = {name: fixture_key(base, f"{file}::tests::{name}") for name in CURRENT_EXE_KEYED}
    refused = {name: fixture_key(base, f"{file}::tests::{name}")[1] for name in CURRENT_EXE_REFUSED}
    reached = {}
    for name in CURRENT_EXE_KEYED:
        found = closure_of(base, f"{file}::tests::{name}")
        reached[name] = {unit.name: found.via.get(id(unit), (None, ""))[1] for unit in found.order}.get("tests::the_child_runs_alone")
    edit_file(base, file, "        let note = 5;\n        assert_eq!(note, 5);", "        let note = 6;\n        assert_eq!(note, 6);")
    outside = {name: fixture_key(base, f"{file}::tests::{name}")[0] for name in CURRENT_EXE_KEYED}
    edit_file(base, file, "        let value = 1;\n        assert_eq!(value, 1);", "        let value = 2;\n        assert_eq!(value, 2);")
    inside = {name: fixture_key(base, f"{file}::tests::{name}")[0] for name in CURRENT_EXE_KEYED}
    ok = (all(kept[name][0] is not None for name in CURRENT_EXE_KEYED) and all(rule == "current-exe-child" for rule in reached.values())
          and all(forms == {"current-exe"} for forms in refused.values())
          and all(outside[name] == kept[name][0] for name in CURRENT_EXE_KEYED)
          and all(inside[name] is not None and inside[name] != kept[name][0] for name in CURRENT_EXE_KEYED))
    return ok, (f"算得出键的入口：{ {name: kept[name][0] is not None for name in CURRENT_EXE_KEYED} }，拒算 {[forms for key, forms in kept.values() if key is None]}；"
                f"子进程测试进闭包的规则 {reached}（要都是 current-exe-child）；点名不存在的、名字在 --exact 前面的拒算 {refused}（要都是 current-exe）；"
                f"改无关的测试后键同前：{[outside[name] == kept[name][0] for name in CURRENT_EXE_KEYED]}；改子进程测试的函数体后键都变：{[inside[name] != kept[name][0] for name in CURRENT_EXE_KEYED]}")


CURRENT_EXE_CONSTANT_FIXTURE = {
    "crates/fexec/Cargo.toml": '[package]\nname = "fexec"\n',
    "crates/fexec/src/lib.rs": '''#[cfg(test)]
mod names;
#[cfg(test)]
mod other_names;
#[cfg(test)]
mod tests {
    use crate::names::{AMBIGUOUS_TEST, CHILD_TEST};
    #[test]
    fn the_parent_names_the_child_by_a_constant_from_another_file() {
        let output = std::process::Command::new(std::env::current_exe().expect("路径")).args(["--exact", CHILD_TEST]).output().expect("起得来");
        assert!(output.status.success());
    }
    #[test]
    fn the_parent_names_a_constant_defined_twice_with_different_values() {
        let _ = std::process::Command::new(std::env::current_exe().expect("路径")).args(["--exact", AMBIGUOUS_TEST]).output();
    }
    #[test]
    fn the_child_runs_alone() {
        let value = 1;
        assert_eq!(value, 1);
    }
    #[test]
    fn the_unrelated_test() {
        let note = 5;
        assert_eq!(note, 5);
    }
}
''',
    "crates/fexec/src/names.rs": 'pub const CHILD_TEST: &str = "tests::the_child_runs_alone";\npub const AMBIGUOUS_TEST: &str = "tests::the_child_runs_alone";\n',
    "crates/fexec/src/other_names.rs": 'pub const AMBIGUOUS_TEST: &str = "tests::missing";\n',
}
CURRENT_EXE_CONSTANT_CELL = ("Rust：current_exe() 的测试名里的常量定义在同包另一份文件里，--exact 后面用它：连到那条 #[test]、不拒算，改它的函数体键变、改别的测试键不变；"
                             "同名常量在同包两份文件里值不同的照旧拒算 current-exe")


def current_exe_constant_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, CURRENT_EXE_CONSTANT_FIXTURE)
    file = "crates/fexec/src/lib.rs"
    keyed, ambiguous = f"{file}::tests::the_parent_names_the_child_by_a_constant_from_another_file", f"{file}::tests::the_parent_names_a_constant_defined_twice_with_different_values"
    kept, kept_forms = fixture_key(base, keyed)
    ambiguous_key, ambiguous_forms = fixture_key(base, ambiguous)
    found = closure_of(base, keyed)
    reached = {unit.name: found.via.get(id(unit), (None, ""))[1] for unit in found.order}.get("tests::the_child_runs_alone")
    edit_file(base, file, "        let note = 5;\n        assert_eq!(note, 5);", "        let note = 6;\n        assert_eq!(note, 6);")
    outside = fixture_key(base, keyed)[0]
    edit_file(base, file, "        let value = 1;\n        assert_eq!(value, 1);", "        let value = 2;\n        assert_eq!(value, 2);")
    inside = fixture_key(base, keyed)[0]
    ok = (kept is not None and reached == "current-exe-child" and ambiguous_key is None and ambiguous_forms == {"current-exe"}
          and outside == kept and inside is not None and inside != kept)
    return ok, (f"常量在另一份文件里的入口：键{'有' if kept else '无'}（拒算 {sorted(kept_forms)}）、子进程测试进闭包的规则 {reached}（要 current-exe-child）；"
                f"同名常量两份值不同的：键{'有' if ambiguous_key else '无'}、拒算 {sorted(ambiguous_forms)}（要无键、拒算 current-exe）；"
                f"改无关的测试后键同前：{outside == kept}；改子进程测试的函数体后键变：{inside is not None and inside != kept}")


CURRENT_EXE_PATH_ONLY_FIXTURE = {
    "crates/fexp/Cargo.toml": '[package]\nname = "fexp"\n',
    "crates/fexp/src/lib.rs": '''#[cfg(test)]
mod tests {
    #[test]
    fn it_only_asks_whether_it_has_a_path() {
        assert!(std::env::current_exe().is_ok());
    }
    #[test]
    fn it_reads_the_size_of_the_file_it_was_started_from() {
        let started_from = std::env::current_exe().expect("路径");
        let size = std::fs::metadata(&started_from).expect("读得到").len();
        assert!(size > 0);
    }
    #[test]
    fn the_unrelated_test() {
        let note = 5;
        assert_eq!(note, 5);
    }
}
''',
}
CURRENT_EXE_PATH_ONLY_CELL = "Rust：current_exe() 只问路径、同一函数里不起 Command 的不拒算：键算得出，改它的函数体键变、改别的测试键不变"
CURRENT_EXE_PATH_ONLY_ENTRIES = ("it_only_asks_whether_it_has_a_path", "it_reads_the_size_of_the_file_it_was_started_from")


def current_exe_path_only_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, CURRENT_EXE_PATH_ONLY_FIXTURE)
    file = "crates/fexp/src/lib.rs"
    kept = {name: fixture_key(base, f"{file}::tests::{name}") for name in CURRENT_EXE_PATH_ONLY_ENTRIES}
    edit_file(base, file, "        let note = 5;\n        assert_eq!(note, 5);", "        let note = 6;\n        assert_eq!(note, 6);")
    outside = {name: fixture_key(base, f"{file}::tests::{name}")[0] for name in CURRENT_EXE_PATH_ONLY_ENTRIES}
    edit_file(base, file, "        assert!(std::env::current_exe().is_ok());", "        assert!(std::env::current_exe().is_err());")
    edit_file(base, file, "expect(\"读得到\").len();\n        assert!(size > 0);", "expect(\"读得到\").len();\n        assert!(size > 1);")
    inside = {name: fixture_key(base, f"{file}::tests::{name}")[0] for name in CURRENT_EXE_PATH_ONLY_ENTRIES}
    ok = (all(kept[name][0] is not None and not kept[name][1] for name in CURRENT_EXE_PATH_ONLY_ENTRIES)
          and all(outside[name] == kept[name][0] for name in CURRENT_EXE_PATH_ONLY_ENTRIES)
          and all(inside[name] is not None and inside[name] != kept[name][0] for name in CURRENT_EXE_PATH_ONLY_ENTRIES))
    return ok, (f"算得出键的入口：{ {name: kept[name][0] is not None for name in CURRENT_EXE_PATH_ONLY_ENTRIES} }，拒算 {[sorted(kept[name][1]) for name in CURRENT_EXE_PATH_ONLY_ENTRIES]}（要都不拒算）；"
                f"改无关的测试后键同前：{[outside[name] == kept[name][0] for name in CURRENT_EXE_PATH_ONLY_ENTRIES]}；"
                f"改它们自己的函数体后键都变：{[inside[name] != kept[name][0] for name in CURRENT_EXE_PATH_ONLY_ENTRIES]}")


CURRENT_EXE_SPAWN_FIXTURE = {
    "crates/fexs/Cargo.toml": '[package]\nname = "fexs"\n',
    "crates/fexs/src/lib.rs": '''#[cfg(test)]
mod tests {
    use std::process::Command;
    #[test]
    fn it_starts_itself_and_names_the_child() {
        let output = Command::new(std::env::current_exe().expect("路径")).args(["--exact", "tests::the_child_runs_alone"]).output().expect("起得来");
        assert!(output.status.success());
    }
    #[test]
    fn it_starts_itself_without_naming_a_test() {
        let output = Command::new(std::env::current_exe().expect("路径")).output().expect("起得来");
        assert!(output.status.success());
    }
    #[test]
    fn it_takes_the_path_first_and_hands_it_to_command() {
        let program = std::env::current_exe().expect("路径");
        let output = Command::new(program).args(["--exact", "tests::the_child_runs_alone"]).output().expect("起得来");
        assert!(output.status.success());
    }
    #[test]
    fn it_names_a_test_nobody_defines() {
        let _ = Command::new(std::env::current_exe().expect("路径")).args(["--exact", "tests::missing"]).output();
    }
    #[test]
    fn the_child_runs_alone() {
        let value = 1;
        assert_eq!(value, 1);
    }
}
''',
}
CURRENT_EXE_SPAWN_CELL = ("Rust：同一函数里起 Command 的 current_exe() 照旧按起自己判：折得出测试名的连边，"
                          "折不出的（没有 --exact、路径先存进变量再交 Command::new、点名不存在的）拒算 current-exe")
CURRENT_EXE_SPAWN_KEYED = "it_starts_itself_and_names_the_child"
CURRENT_EXE_SPAWN_REFUSED = ("it_starts_itself_without_naming_a_test", "it_takes_the_path_first_and_hands_it_to_command", "it_names_a_test_nobody_defines")


def current_exe_spawn_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, CURRENT_EXE_SPAWN_FIXTURE)
    file = "crates/fexs/src/lib.rs"
    kept, kept_forms = fixture_key(base, f"{file}::tests::{CURRENT_EXE_SPAWN_KEYED}")
    refused = {name: fixture_key(base, f"{file}::tests::{name}") for name in CURRENT_EXE_SPAWN_REFUSED}
    found = closure_of(base, f"{file}::tests::{CURRENT_EXE_SPAWN_KEYED}")
    reached = {unit.name: found.via.get(id(unit), (None, ""))[1] for unit in found.order}.get("tests::the_child_runs_alone")
    edit_file(base, file, "        let value = 1;\n        assert_eq!(value, 1);", "        let value = 2;\n        assert_eq!(value, 2);")
    inside = fixture_key(base, f"{file}::tests::{CURRENT_EXE_SPAWN_KEYED}")[0]
    ok = (kept is not None and reached == "current-exe-child" and inside is not None and inside != kept
          and all(key is None and forms == {"current-exe"} for key, forms in refused.values()))
    return ok, (f"点名了测试的入口：键{'有' if kept else '无'}（拒算 {sorted(kept_forms)}）、子进程测试进闭包的规则 {reached}（要 current-exe-child）、改子进程测试的函数体后键变：{inside is not None and inside != kept}；"
                f"没点名、路径先存变量、点名不存在的：{ {name: (key is not None, sorted(forms)) for name, (key, forms) in refused.items()} }（要都无键、拒算 current-exe）")


MANIFEST_DIRECTORY_FIXTURE = {
    "crates/fdir/Cargo.toml": '[package]\nname = "fdir"\n',
    "crates/fdir/rules/first.txt": "rule one\n",
    "crates/fdir/rules/nested/second.txt": "rule two\n",
    "crates/fdir/src/lib.rs": '''use std::path::Path;
const MANIFEST: &str = env!("CARGO_MANIFEST_DIR");
pub fn unrelated_value() -> u32 {
    1
}
pub fn count_rules_directly() -> usize {
    std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("rules")).expect("列得了").count()
}
pub fn count_rules_by_variable() -> usize {
    let rules = Path::new(env!("CARGO_MANIFEST_DIR")).join("rules");
    std::fs::read_dir(&rules).expect("列得了").count()
}
pub fn count_rules_by_constant() -> usize {
    let root = Path::new(MANIFEST);
    let nested = root.join("rules").join("nested");
    std::fs::read_dir(nested).expect("列得了").count()
}
pub fn count_every_crate() -> usize {
    std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("..")).expect("列得了").count()
}
pub fn count_by_climbing_from_a_variable() -> usize {
    let rules = Path::new(env!("CARGO_MANIFEST_DIR")).join("rules");
    std::fs::read_dir(rules.parent().expect("有上一级")).expect("列得了").count()
}
''',
}
MANIFEST_DIRECTORY_CELL = "Rust：清单目录折出 crates/ 之下的目录收成数据目录（直接写、绑 let 变量、经私有常量），目录里一份文件改了键变、闭包外的改了键不变；crates/ 整个与变量上接别的方法的拒算 runtime-path"
MANIFEST_DIRECTORY_ENTRIES = ("count_rules_directly", "count_rules_by_variable", "count_rules_by_constant")


def manifest_directory_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, MANIFEST_DIRECTORY_FIXTURE)
    file = "crates/fdir/src/lib.rs"
    kept, problems = {}, []
    for name in MANIFEST_DIRECTORY_ENTRIES:
        closure = closure_of(base, f"{file}::{name}")
        key, _forms = fixture_key(base, f"{file}::{name}")
        kept[name] = key
        if key is None or "data-directory" not in {unit.kind for unit in closure.order}:
            problems.append(f"{name}：键{'有' if key else '无'}、闭包里{'有' if any(unit.kind == 'data-directory' for unit in closure.order) else '没有'}数据目录")
    refused = {name: fixture_key(base, f"{file}::{name}")[1] for name in ("count_every_crate", "count_by_climbing_from_a_variable")}
    edit_file(base, file, "    1\n}", "    2\n}")
    outside = {name: fixture_key(base, f"{file}::{name}")[0] for name in MANIFEST_DIRECTORY_ENTRIES}
    edit_file(base, "crates/fdir/rules/nested/second.txt", "rule two", "rule two, changed")
    inside = {name: fixture_key(base, f"{file}::{name}")[0] for name in MANIFEST_DIRECTORY_ENTRIES}
    ok = (not problems and all(forms == {"runtime-path"} for forms in refused.values()) and outside == kept
          and all(inside[name] is not None and inside[name] != kept[name] for name in MANIFEST_DIRECTORY_ENTRIES))
    return ok, (f"没算出键或闭包里没有数据目录：{problems}；crates/ 整个与变量上接 .parent() 的拒算 {refused}（要都是 runtime-path）；"
                f"改闭包外的函数后键同前：{outside == kept}；改目录里一份文件后三个入口的键都变：{[inside[name] != kept[name] for name in MANIFEST_DIRECTORY_ENTRIES]}")


# prune-server/src/preflight.rs 的写法：清单目录绑给 let，一处 .join(参数) 折不出，一处 .join("…") 折出一份文件（crates/singlefs-unit-graph 的 fold_path_chain 逐处折）
MANIFEST_BINDING_FIXTURE = {
    "crates/fpre/Cargo.toml": '[package]\nname = "fpre"\n',
    "crates/fpre/tools/check.py": "print('check')\n",
    "crates/fpre/src/lib.rs": '''use std::path::Path;
pub fn check_from_the_manifest_directory(source_relative_to_manifest: &str) -> usize {
    let manifest_directory = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = manifest_directory.join(source_relative_to_manifest);
    let script = manifest_directory
        .join("tools")
        .join("check.py");
    source.as_os_str().len() + script.as_os_str().len()
}
pub fn check_from_the_crates_directory(source_relative_to_manifest: &str) -> usize {
    let crates_directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let source = crates_directory.join(source_relative_to_manifest);
    let script = crates_directory.join("fpre").join("tools/check.py");
    source.as_os_str().len() + script.as_os_str().len()
}
pub fn check_through_an_alias() -> usize {
    let tools: std::path::PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("tools");
    let renamed = tools;
    let script = Path::new(renamed).join("check.py");
    script.as_os_str().len()
}
''',
}
MANIFEST_BINDING_CELL = ("Rust：清单目录绑给 let 变量的，一处 .join(变量) 折不出、一处折出文件：这个单元拒算 runtime-path、那份文件照记一条 data 读"
                         "（绑的是清单目录本身与 crates/ 整个都一样）；别名再绑一层、Path::new(别名) 接的链照跟")
# 入口 → (env! 那一行, 要不要拒算)：每个入口都只读 crates/fpre/tools/check.py 一份，读记在 env! 那一行
MANIFEST_BINDING_ENTRIES = {"check_from_the_manifest_directory": (3, True), "check_from_the_crates_directory": (11, True), "check_through_an_alias": (17, False)}


def manifest_binding_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, MANIFEST_BINDING_FIXTURE)
    file = "crates/fpre/src/lib.rs"
    digest = sha256_bytes(MANIFEST_BINDING_FIXTURE["crates/fpre/tools/check.py"].encode("utf-8"))
    reads = sorted(line for line in dump_lines(base, "rust")[0] if line.startswith(("data\t", "data-directory\t")))
    want_reads = sorted(f"data\tcrates/fpre/tools/check.py\t{file}:{line}-{line}\t{digest}" for line, _refused in MANIFEST_BINDING_ENTRIES.values())
    keys = {name: fixture_key(base, f"{file}::{name}") for name in MANIFEST_BINDING_ENTRIES}
    wrong = {name: (key is not None, sorted(forms)) for name, (key, forms) in keys.items()
             if (key is None, forms) != (MANIFEST_BINDING_ENTRIES[name][1], {"runtime-path"} if MANIFEST_BINDING_ENTRIES[name][1] else set())}
    ok = reads == want_reads and not wrong
    return ok, (f"dump 的数据读 {reads}（要 {want_reads}）；键与拒算对不上的入口 (有没有键, 拒算) {wrong}"
                f"（要 check_from_the_manifest_directory、check_from_the_crates_directory 无键、拒算 runtime-path，check_through_an_alias 有键、不拒算）")


RUST_LEX_CACHE_CELL = "Rust 词法单元的磁盘缓存：同一份文本第二次取缓存、与现切的逐项相同；改一个字符（长度不变）不沿用、切出来的随文本变；缓存文件坏了照常现切"


def rust_lex_cache_cell(work):
    """缓存放在这一格自己的目录里（FUNCTION_UNITS_LEX_CACHE_DIRECTORY 临时换成它）：同一份文本头一趟现切、第二趟取缓存且与 rust_lex 现切的逐项相同；
    换一份同样长、差一个字符的文本：现切、与它的 rust_lex 逐项相同、与头一份不同（弄坏开关 rust-lex-cache-unverified 下文件名只取长度，这里沿用了头一份）；
    缓存文件写坏了：照常现切、结果同前。"""
    first_text = "pub fn answer() -> u32 {\n    // note\n    /// documented\n    41 + 1\n}\n"
    second_text = first_text.replace("41", "43")
    previous = os.environ.get(RUST_LEX_CACHE_DIRECTORY_VARIABLE)
    os.environ[RUST_LEX_CACHE_DIRECTORY_VARIABLE] = os.path.join(work, "rust-lex-cache")

    def trip(text):
        before = dict(RUST_LEX_CACHE_STATS)
        result = rust_lex_cached(text)
        return result, RUST_LEX_CACHE_STATS["lexed"] - before["lexed"], RUST_LEX_CACHE_STATS["reused"] - before["reused"]
    try:
        first, second = trip(first_text), trip(first_text)
        changed, changed_again = trip(second_text), trip(second_text)
        with open(rust_lex_cache_path(first_text), "w", encoding="utf-8") as handle:
            handle.write("{broken")   # 不是 zlib 压缩的 json
        damaged = trip(first_text)
    finally:
        if previous is None:
            os.environ.pop(RUST_LEX_CACHE_DIRECTORY_VARIABLE, None)
        else:
            os.environ[RUST_LEX_CACHE_DIRECTORY_VARIABLE] = previous
    direct_first, direct_second = rust_lex(first_text), rust_lex(second_text)
    ok = (direct_second != direct_first and first == (direct_first, 1, 0) and second == (direct_first, 0, 1) and changed == (direct_second, 1, 0)
          and changed_again == (direct_second, 0, 1) and damaged == (direct_first, 1, 0))
    return ok, (f"头一趟 (现切 {first[1]}、取缓存 {first[2]}，同 rust_lex {first[0] == direct_first})、第二趟 ({second[1]}、{second[2]}，{second[0] == direct_first})、"
                f"换文本 ({changed[1]}、{changed[2]}，同它的 rust_lex {changed[0] == direct_second})、再一趟 ({changed_again[1]}、{changed_again[2]}，{changed_again[0] == direct_second})、"
                f"缓存文件写坏 ({damaged[1]}、{damaged[2]}，{damaged[0] == direct_first})（要 头一趟 (1、0)、第二趟 (0、1)、换文本 (1、0)、再一趟 (0、1)、写坏 (1、0)，结果都同现切）")


DIRECTORY_CLOSURE_LINE_CELL = "命令行 closure：数据目录单元按读它的那一处列（data-directory 加目录、读它的文件与行、目录指纹），不打成 <目录>:1-<文件数>"


def directory_closure_line_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, MANIFEST_DIRECTORY_FIXTURE)
    file = "crates/fdir/src/lib.rs"
    closure = closure_of(base, f"{file}::count_rules_directly")
    got = [line for line in closure_unit_lines(closure) if line.startswith("data-directory\t")]
    line = next(number for number, text in enumerate(MANIFEST_DIRECTORY_FIXTURE[file].splitlines(), 1) if 'join("rules")' in text)
    want = [f"data-directory\tcrates/fdir/rules\t{file}:{line}-{line}\t{directory_fingerprint(base, 'crates/fdir/rules')[0]}"]
    return got == want, f"打出的数据目录行 {got}（要 {want}）"


def no_refusal_cell(fixture, entry):
    def judge(work):
        base = os.path.join(work, "tree")
        write_tree(base, fixture)
        key, forms = fixture_key(base, entry)
        return key is not None, f"不该拒算，实际拒算 {sorted(forms)}"
    return judge


FAKE_TOOLCHAIN_SCRIPT = '#!/bin/sh\necho "{name} $(cat "{version}")"\n'


def toolchain_query_cell(work):
    """TOOLCHAIN_QUERY_CELL（文件头「版本询问」）：PATH 最前面放假 rustc、cargo（打 work 里一份版本文件的内容），compiler_version 起
    [os.environ["RUSTC"], "-V"]：闭包记了工具链询问、不拒算、算得出键；版本文件换一版，compiler_version 的键变，闭包里没有询问的 judge_file 键不变。"""
    base = os.path.join(work, "tree")
    write_tree(base, FOLDS_FIXTURE)
    tools, version = os.path.join(work, "fake-toolchain"), os.path.join(work, "fake-toolchain-version")
    os.makedirs(tools)
    for name in ("rustc", "cargo"):
        with open(os.path.join(tools, name), "w", encoding="utf-8") as handle:
            handle.write(FAKE_TOOLCHAIN_SCRIPT.format(name=name, version=version))
        os.chmod(os.path.join(tools, name), 0o755)
    patched = {"PATH": tools + os.pathsep + os.environ.get("PATH", ""), "CARGO_HOME": os.path.join(work, "cargo-home")}
    saved = {name: os.environ.get(name) for name in (*patched, *TOOLCHAIN_PROGRAM_VARIABLES)}

    def keys(release):
        with open(version, "w", encoding="utf-8") as handle:
            handle.write(release + "\n")
        queried = closure_of(base, "pkg/folds.py::compiler_version")
        return queried, closure_key(queried), closure_key(closure_of(base, "pkg/folds.py::judge_file"))[0]
    try:
        os.environ.update(patched)
        for name in TOOLCHAIN_PROGRAM_VARIABLES:
            os.environ.pop(name, None)
        queried, (before, refusals), plain_before = keys("1.0.0-selftest")
        _queried, (after, _refusals), plain_after = keys("1.1.0-selftest")
    finally:
        for name, value in saved.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value
    toolchain_units = [unit for unit in queried.order if unit.kind == TOOLCHAIN_UNIT_KIND]
    return (bool(queried.toolchain_queries) and len(toolchain_units) == 1 and not refusals and None not in (before, after) and before != after
            and plain_before is not None and plain_before == plain_after,
            f"compiler_version：工具链单元 {len(toolchain_units)} 个、拒算 {sorted({refusal.form for refusal in refusals})}、换版本前 {before}、后 {after}；"
            f"judge_file 换版本前 {plain_before}、后 {plain_after}")


def caller_launch_cell(work):
    """CALLER_LAUNCH_CELL（文件头「调用方交进来的命令」）：run_dynamic(code)、run_unknown(interpreter) 当判法入口，-c 的串与解释器是入口的参数，不拒算
    （同一份夹具里经环境变量调它们的那两条照旧拒算，另两格判）。"""
    base = os.path.join(work, "tree")
    write_tree(base, PYTHON_FIXTURE)
    found = {entry: fixture_key(base, entry) for entry in ("pkg/m.py::run_dynamic", "pkg/m.py::run_unknown")}
    refused = [f"{entry} 拒算 {sorted(forms)}" for entry, (key, forms) in found.items() if key is None]
    return not refused, "；".join(refused)


# 登记表点名的程序（文件头「登记表点名的程序」）那一格的夹具：run_probe 起的脚本路径来自登记行，登记表写在 REGISTRY_LISTED_PROGRAMS 里
REGISTRY_FIXTURE = {
    "pkg/probes.py": '''import os
import subprocess

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REGISTRY = "pkg/registry.tsv"
REGISTRY_LISTED_PROGRAMS = {"run_probe": (REGISTRY, r"probe=(\\S+)")}


def run_probe(root, parts):
    return subprocess.run(["bash", os.path.join(root, parts["script"])], check=False).returncode


def check_all():
    with open(os.path.join(ROOT, REGISTRY), encoding="utf-8") as handle:
        rows = handle.read().splitlines()
    return [run_probe(ROOT, {"script": row.split("probe=")[1]}) for row in rows]
''',
    "pkg/registry.tsv": "alpha\tprobe=pkg/probe_alpha.sh\nbeta\tprobe=pkg/probe_beta.sh\n",
    "pkg/probe_alpha.sh": "#!/usr/bin/env bash\necho alpha\n",
    "pkg/probe_beta.sh": "#!/usr/bin/env bash\necho beta\n",
}


def registry_programs_cell(work):
    """REGISTRY_PROGRAMS_CELL：check_all 读登记表、照每一行调 run_probe（起探针的那一处登在 REGISTRY_LISTED_PROGRAMS 里）：
    键算得出（不拒算），改登记表点名的 probe_beta.sh 之后键不变（探针不连）。开关 python-registry-programs-ignored 下拒算，python-registry-programs-linked 下键变。"""
    base = os.path.join(work, "tree")
    write_tree(base, REGISTRY_FIXTURE)
    before = fixture_key(base, "pkg/probes.py::check_all")
    edit_file(base, "pkg/probe_beta.sh", "echo beta", "echo beta again")
    after = fixture_key(base, "pkg/probes.py::check_all")
    ok = before[0] is not None and after[0] == before[0]
    return ok, f"改前键 {before[0]}（拒算 {before[1]}）；改探针之后键 {after[0]}（拒算 {after[1]}）"


# 按行进键的登记表（文件头「按行进键的登记表」）那一格的夹具：guard.py 按 ROW_KEYED_TABLES 登记的取用函数 registered_selftest 查一行，audit 不在登记里、读整张
ROW_KEYED_FIXTURE = {
    ".claude/hooks/selftest-registry.tsv": "scripts/a.py\tselftest\t# 甲\nscripts/b.py\tselftest\t# 乙\n.claude/gate.d/x.sh\tsamples:tools\t# 丙\n",
    "pkg/guard.py": '''import os

REGISTRY = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), ".claude", "hooks", "selftest-registry.tsv")


def selftest_registry_at(path):
    rows = {}
    with open(path, encoding="utf-8") as handle:
        for line in handle.read().split("\\n"):
            fields = line.split("\\t")
            if len(fields) > 1:
                rows.setdefault(fields[0], fields[1])
    return rows


def read_selftest_registry(path):
    return selftest_registry_at(path)


def registered_selftest(key):
    return read_selftest_registry(REGISTRY).get(key)


def judge(key):
    return registered_selftest(key) is not None


def audit():
    return sorted(read_selftest_registry(REGISTRY))
''',
}


def row_keyed_cell(work):
    """ROW_KEYED_CELL：judge（经 registered_selftest 查 scripts/a.py 那一行）的键照 row_keyed_closure 按入口 scripts/a.py 换过：改 scripts/b.py 那一行、
    改样本档行键不变，改 scripts/a.py 那一行键变；audit（读整张表、不在登记里）改 scripts/b.py 那一行键就变。"""
    base = os.path.join(work, "tree")
    write_tree(base, ROW_KEYED_FIXTURE)

    def keys():
        out = {}
        for entry in ("judge", "audit"):
            closure = closure_of(base, f"pkg/guard.py::{entry}", Options())
            out[entry] = closure_key(row_keyed_closure(closure, base, "scripts/a.py"))[0]
        return out
    before = keys()
    edit_file(base, ".claude/hooks/selftest-registry.tsv", "# 乙", "# 乙改")
    unrelated = keys()
    edit_file(base, ".claude/hooks/selftest-registry.tsv", "samples:tools", "samples:other")
    sample_row = keys()
    edit_file(base, ".claude/hooks/selftest-registry.tsv", "# 甲", "# 甲改")
    own_row = keys()
    ok = (None not in before.values() and unrelated["judge"] == before["judge"] and sample_row["judge"] == before["judge"]
          and own_row["judge"] not in (None, before["judge"]) and unrelated["audit"] not in (None, before["audit"]))
    return ok, f"改前 {before}；改无关的一行之后 {unrelated}；改样本档行之后 {sample_row}；改它查的那一行之后 {own_row}"


# 程序入口带参数（文件头「程序入口带参数」）那一格的夹具：dispatch.py 按 argv 分派（--selftest 一支、分派表一支），launcher.py 带着 alpha 起它
ARGUMENTS_FIXTURE = {
    "tool/dispatch.py": '''import sys


def alpha_work():
    return 1


def beta_work():
    return 2


def run_selftest():
    return alpha_work() + beta_work()


COMMANDS = {"alpha": alpha_work, "beta": beta_work}


def main(arguments):
    if arguments[:1] == ["--selftest"]:
        return run_selftest()
    if not arguments or arguments[0] not in COMMANDS:
        print("用法：dispatch.py alpha|beta|--selftest")
        return 9
    return COMMANDS[arguments[0]]()


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
''',
    "tool/launcher.py": '''import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))


def main():
    return subprocess.run([sys.executable, os.path.join(HERE, "dispatch.py"), "alpha"], check=False).returncode


if __name__ == "__main__":
    sys.exit(main())
''',
}


def program_arguments_cell(work):
    """PROGRAM_ARGUMENTS_CELL：dispatch.py 带 alpha、带 --selftest 与 launcher.py（起 dispatch.py alpha）三条：改 beta_work，alpha 与 launcher 的键不变、
    --selftest 的变；再改 alpha_work，alpha 与 launcher 的键变。"""
    base = os.path.join(work, "tree")
    write_tree(base, ARGUMENTS_FIXTURE)

    def keys():
        out = {}
        for label, entry, arguments in (("alpha", "tool/dispatch.py", ["alpha"]), ("selftest", "tool/dispatch.py", ["--selftest"]),
                                        ("launcher", "tool/launcher.py", ["run"])):
            options = Options()
            options.program_arguments = arguments
            out[label] = fixture_key(base, entry, options)[0]
        return out
    before = keys()
    edit_file(base, "tool/dispatch.py", "    return 2\n", "    return 20\n")
    beta_changed = keys()
    edit_file(base, "tool/dispatch.py", "    return 1\n", "    return 10\n")
    alpha_changed = keys()
    ok = (None not in before.values() and beta_changed["alpha"] == before["alpha"] and beta_changed["launcher"] == before["launcher"]
          and beta_changed["selftest"] not in (None, before["selftest"]) and alpha_changed["alpha"] not in (None, beta_changed["alpha"])
          and alpha_changed["launcher"] not in (None, beta_changed["launcher"]))
    return ok, f"改前 {before}；改 beta_work 之后 {beta_changed}；改 alpha_work 之后 {alpha_changed}"


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


def selftest_git_steps(work, steps):
    """自证造 git 仓：逐条起 git <step>（不读系统与用户的 git 配置，HOME 指 work，提交人写死）→ 头一条退非 0 的那一句，全退 0 交 None。"""
    environment = {"PATH": os.environ.get("PATH", "/usr/bin:/bin"), "HOME": work, "LC_ALL": "C", "GIT_CONFIG_NOSYSTEM": "1"}
    git = ["git", "-c", "user.name=selftest", "-c", "user.email=selftest@example.invalid", "-c", "commit.gpgsign=false", "-c", "init.defaultBranch=main"]
    for step in steps:
        completed = subprocess.run(git + step, env=environment, capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=60)
        if completed.returncode != 0:
            return f"git {' '.join(step[:3])} 退 {completed.returncode}：{completed.stderr.strip()[:200]}"
    return None


def preflight_worktree_cell(work):
    """git worktree 里没有规范副本（.gitignore 挡着）：source / import 两份装载器都解到主仓那一份。"""
    canonical = sys.modules["project_preflight"].shared_preflight_directory()
    if canonical is None:
        return False, "本仓找不到规范副本（.claude/singlefs-ai-sop/scripts/），造不出主仓"
    main, linked = os.path.join(work, "main"), os.path.join(work, "linked")
    loader_tree(main, canonical)
    with open(os.path.join(main, ".gitignore"), "w", encoding="utf-8") as handle:
        handle.write("/.claude/singlefs-ai-sop/\n")
    problem = selftest_git_steps(work, (["init", "-q", main], ["-C", main, "add", ".gitignore", ".claude/scripts"],
                                        ["-C", main, "commit", "-q", "-m", "loaders"], ["-C", main, "worktree", "add", "-q", "--detach", linked]))
    if problem is not None:
        return False, problem
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


# ── 判法入口的登记（文件头「判法入口的登记」）──

JUDGING_ENTRY_STAGE_DIRECTORY = ".claude/gate.d"
JUDGING_ENTRY_TOKEN = re.compile(r'"judge:([^"$\s]+?|\.claude/gate\.d/\$STAGE_NAME)::(\w+)"')


def module_literal(root, path, name):
    """root 里 path 那份 python 的模块级赋值 name 的字面值：字面量，或由它前面模块级字面量赋值的名字与 + 拼成的式子（JUDGING_ENTRIES = A + (...)）；
    找不到、折不出抛 FunctionUnitsError（弄坏开关 judging-entry-literals-unfolded 下只认字面量）。"""
    with open(os.path.join(root, path), encoding="utf-8") as handle:
        tree = ast.parse(handle.read(), filename=path)
    known = {}

    def fold(node):
        if "judging-entry-literals-unfolded" in active_breaks():
            return ast.literal_eval(node)
        if isinstance(node, ast.Name) and node.id in known:
            return known[node.id]
        if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Add):
            return fold(node.left) + fold(node.right)
        return ast.literal_eval(node)

    for statement in tree.body:
        names = [target.id for target in statement.targets if isinstance(target, ast.Name)] if isinstance(statement, ast.Assign) else []
        if not names:
            continue
        try:
            value = fold(statement.value)
        except (ValueError, TypeError, SyntaxError, RecursionError) as error:
            if name in names:
                raise FunctionUnitsError(f"{path} 里模块级的 {name} 折不成字面值（只认字面量、前面模块级字面量的名字与 +）：{error}") from error
            continue
        known.update((target, value) for target in names)
        if name in names:
            return value
    raise FunctionUnitsError(f"{path} 里没有模块级的 {name}（判法入口的登记挪了：照文件头「判法入口的登记」改 registered_judging_entries）")


def registered_judging_entries(root):
    """存判定的各路判法入口，从各份脚本的登记现取（不手抄）：广谱变异 JUDGING_ENTRIES 与 GPU_JUDGING_ENTRIES、crates 变异 JUDGING_ENTRIES、
    admission.py 的 REPLAY_SCRIPT 与 REPLAY_JUDGING_FUNCTIONS、e165 的 JUDGING_ENTRY、verus-proof-check.py 的 JUDGING_ENTRIES，
    再加门禁阶段（.claude/gate.d/*.sh）里写死的 "judge:<文件>::<函数>" 令牌（$STAGE_NAME 换成那一道阶段自己）。去重、保持登记的次序。"""
    root = os.path.abspath(root)
    entries = []
    for name in ("JUDGING_ENTRIES", "GPU_JUDGING_ENTRIES"):
        entries += [f"{path}::{function}" for path, function in module_literal(root, "scripts/mutation/broad-mutation.py", name)]
    entries += list(module_literal(root, "scripts/mutation/crates-mutation-rows.py", "JUDGING_ENTRIES"))
    replay_script = module_literal(root, "scripts/gate/admission.py", "REPLAY_SCRIPT")
    entries += [f"{replay_script}::{function}" for function in module_literal(root, "scripts/gate/admission.py", "REPLAY_JUDGING_FUNCTIONS")]
    entries.append(f"scripts/experiment-apparatus/e165-pin-red-examples.py::{module_literal(root, 'scripts/experiment-apparatus/e165-pin-red-examples.py', 'JUDGING_ENTRY')}")
    entries += [f"scripts/gate/verus-proof-check.py::{function}" for function in module_literal(root, "scripts/gate/verus-proof-check.py", "JUDGING_ENTRIES").values()]
    for stage in sorted(glob.glob(os.path.join(root, JUDGING_ENTRY_STAGE_DIRECTORY, "*.sh"))):
        with open(stage, encoding="utf-8") as handle:
            text = handle.read()
        relative = os.path.relpath(stage, root).replace(os.sep, "/")
        for path, function in JUDGING_ENTRY_TOKEN.findall(text):
            entries.append(f"{relative if '$STAGE_NAME' in path else path}::{function}")
    return list(dict.fromkeys(entries))


# ── import 时执行的模块级语句与不走的子进程边（文件头「Python 闭包：import 时执行的模块级语句」「不走的子进程边」）的自证 ──

IMPORT_TIME_FIXTURE = {
    "tool/entry.py": ("import sys\n\nimport helper\n\n\ndef judge():\n    return helper.verdict()\n\n\n"
                      "if __name__ == \"__main__\":\n    sys.exit(judge())\n"),
    "tool/helper.py": ("import os\n\nUNUSED = 0\n\n\nclass Settings:\n    threshold = 0\n\n    def __init__(self):\n        self.value = 1\n\n\n"
                       "def verdict():\n    return 0\n\n\ndef unused(value=0):\n    return value\n"),
}
IMPORT_TIME_ENTRY = "tool/entry.py::judge"
# 每一种改动：(名, 文件, 原文, 改成) —— 前四种是 import 时就执行、让 python3 tool/entry.py 退非 0 的模块级语句，末一种是对照：
# 改的是程序入口（main-guard）、不在 judge 的闭包里，退出码变了而键不变，这一格的判法必须认出它
IMPORT_TIME_EDITS = (
    ("模块级 import", "tool/helper.py", "import os\n", "from os import missing_name_for_selftest\n"),
    ("模块级赋值", "tool/helper.py", "UNUSED = 0\n", "UNUSED = int(\"selftest\")\n"),
    ("类头", "tool/helper.py", "    threshold = 0\n", "    threshold = int(\"selftest\")\n"),
    ("没人调的 def 的缺省值", "tool/helper.py", "def unused(value=0):", "def unused(value=int(\"selftest\")):"),
)
IMPORT_TIME_CONTROL = ("程序入口（main-guard）", "tool/entry.py", "    sys.exit(judge())\n", "    sys.exit(judge() + 3)\n")
IMPORT_TIME_EXIT_CELL = "python：import 时执行的模块级 import、赋值、类头与 def 缺省值改了让程序退出码变的，入口的键都变（键不变而退出码变的必红，对照那一种必须认出）"
IMPORT_TIME_CLASS_CELL = "python：只因 import 时执行收进来的类头不算被点名，类里的 __init__ 改了不作废"
EXCLUDED_EDGE_PYTHON_CELL = "不走的子进程边（夹具）：登记了的 python 单元起的那一条不连、同一个单元起的另一个程序照连、不登记时那一条照连"
EXCLUDED_EDGE_SHELL_CELL = "不走的子进程边（夹具）：登记了的 shell 函数起的那一条不连、同一个函数起的另一个程序照连、不登记时那一条照连"
# 每个入口单元起两个程序：hint 是自证期间登记成不走的那一条，kept 不登记；EXCLUDED_PROGRAM_EDGES 平时的内容不进这一格
EXCLUDED_EDGE_FIXTURE = {
    "tool/entry.py": ("import os\nimport subprocess\nimport sys\n\nHERE = os.path.dirname(os.path.abspath(__file__))\n\n\n"
                      "def judge():\n    subprocess.run([sys.executable, os.path.join(HERE, \"kept.py\")], check=False)\n"
                      "    return subprocess.run([sys.executable, os.path.join(HERE, \"hint.py\")], check=False)\n"),
    "tool/kept.py": "def work():\n    return 1\n\n\nif __name__ == \"__main__\":\n    work()\n",
    "tool/hint.py": "def work():\n    return 2\n\n\nif __name__ == \"__main__\":\n    work()\n",
    "tool/run.sh": '''#!/usr/bin/env bash
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
launch() {
  bash "$here/kept.sh"
  bash "$here/hint.sh"
}
launch
''',
    "tool/kept.sh": "#!/usr/bin/env bash\necho kept\n",
    "tool/hint.sh": "#!/usr/bin/env bash\necho hint\n",
}
EXCLUDED_EDGE_PYTHON_ENTRY = "tool/entry.py::judge"
EXCLUDED_EDGE_SHELL_ENTRY = "tool/run.sh::launch"
EXCLUDED_EDGE_FIXTURE_EDGES = {
    ("tool/entry.py", "judge", "tool/hint.py"): "夹具：python 单元起的程序，只打提示",
    ("tool/run.sh", "launch", "tool/hint.sh"): "夹具：shell 函数起的程序，只打提示",
}
REAL_ENTRIES_CELL = "真仓：判法入口从登记现取（registered_judging_entries），形式化入口在里面、都认得出"
REAL_TREE_IGNORED = shutil.ignore_patterns("__pycache__", "fixtures", "kb", "target")
REAL_FORMAL_SCRIPT = "scripts/gate/verus-proof-check.py"
# 真仓上的三格，入口都是形式化入口（check_proof 经 reuse_checks.ReuseSentinel 等点到 lib_reuse_checks.py，那份模块因此被载入）：
# (格名, 文件, 原文, 改成)；原文在那份文件里要恰好一次，对不上这一格红、出路是照新原文改这里
REAL_IMPORT_TIME_EDITS = (
    ("真仓：改 scripts/gate/lib_reuse_checks.py 一个模块级 import，形式化入口的键变", "scripts/gate/lib_reuse_checks.py", "\nimport re\n", "\nimport re, json\n"),
    ("真仓：改 scripts/gate/lib_reuse_checks.py 的一个类头（ReuseSentinel），形式化入口的键变", "scripts/gate/lib_reuse_checks.py",
     "\nclass ReuseSentinel:\n", "\nclass ReuseSentinel:\n    IMPORT_TIME_SELFTEST = 1\n"),
    ("真仓：改一个没人调的 def 的缺省值（scripts/gate/lib_reuse_checks.py 的 write_target_last_run），形式化入口的键变", "scripts/gate/lib_reuse_checks.py",
     'def write_target_last_run(script_key, target, exit_code, test_filter="", target_file=""):',
     'def write_target_last_run(script_key, target, exit_code, test_filter=None, target_file=""):'),
)
LIBRARY_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
_REAL_IMPORT_TIME_RESULTS = {}


def run_fixture_program(base, path):
    """夹具里的小程序跑一遍交退出码（60 秒没退交 None）。"""
    try:
        return subprocess.run([sys.executable, "-B", os.path.join(base, path)], cwd=base, capture_output=True, text=True, timeout=60).returncode
    except subprocess.TimeoutExpired:
        return None


def import_time_exit_cell(work):
    """每一种改动各在一棵新树上量（键、python3 tool/entry.py 的退出码）与原树比：退出码变了而键不变的记下；前四种一个都不许有，对照那一种必须有。"""
    base = os.path.join(work, "base")
    write_tree(base, IMPORT_TIME_FIXTURE)
    before_key = closure_key(closure_of(base, IMPORT_TIME_ENTRY), allow_refused=True)[0]
    before_exit = run_fixture_program(base, "tool/entry.py")
    blind, lines = [], []
    for name, path, old, new in IMPORT_TIME_EDITS + (IMPORT_TIME_CONTROL,):
        tree = os.path.join(work, f"edit-{len(lines)}")
        write_tree(tree, IMPORT_TIME_FIXTURE)
        edit_file(tree, path, old, new)
        key = closure_key(closure_of(tree, IMPORT_TIME_ENTRY), allow_refused=True)[0]
        exit_code = run_fixture_program(tree, "tool/entry.py")
        lines.append(f"{name}：退出码 {before_exit}→{exit_code}，键{'变' if key != before_key else '不变'}")
        if exit_code != before_exit and key == before_key:
            blind.append(name)
    want = [IMPORT_TIME_CONTROL[0]]
    return blind == want, f"键不变而退出码变的（要恰好是对照那一种 {want}）：{blind}；{'；'.join(lines)}"


def real_tree(work, tree=None):
    """本仓的 scripts/ 与 .claude/（不带 fixtures、kb、__pycache__）拷进临时目录（tree 没给是 <work>/real；给了的拷进它底下，它可以已经在）：
    改真文件的几格在拷贝上改，不碰本仓。"""
    tree = tree or os.path.join(work, "real")
    for top in ("scripts", ".claude"):
        shutil.copytree(os.path.join(LIBRARY_ROOT, top), os.path.join(tree, top), ignore=REAL_TREE_IGNORED, symlinks=True)
    return tree


def real_entries(root):
    """形式化入口：从登记现取，取 verus-proof-check.py 登记的头一个。"""
    entries = registered_judging_entries(root)
    formal = next((entry for entry in entries if entry.startswith(REAL_FORMAL_SCRIPT + "::")), None)
    if formal is None:
        raise AssertionError(f"登记现取的 {len(entries)} 个判法入口里没有 {REAL_FORMAL_SCRIPT} 的入口：{entries}")
    return formal


def real_import_time_results():
    """真仓三格一起量（一棵拷贝、形式化入口的原键算一次）：{格名: (通过, 说明)}；同一个进程、同一组弄坏开关只量一次。"""
    breaks = os.environ.get(BREAK_VARIABLE, "")
    if breaks not in _REAL_IMPORT_TIME_RESULTS:
        results = {}
        with tempfile.TemporaryDirectory(prefix="function-units-real-tree-") as work:
            tree = real_tree(work)
            entry = real_entries(tree)
            base_key = closure_key(closure_of(tree, entry), allow_refused=True)[0]
            for name, path, old, new in REAL_IMPORT_TIME_EDITS:
                absolute = os.path.join(tree, path)
                with open(absolute, encoding="utf-8") as handle:
                    original = handle.read()
                if original.count(old) != 1:
                    results[name] = (False, f"{path} 里「{old.strip()}」命中 {original.count(old)} 次（要恰好一次）：那份文件改了，照新原文改 REAL_IMPORT_TIME_EDITS")
                    continue
                with open(absolute, "w", encoding="utf-8") as handle:
                    handle.write(original.replace(old, new))
                try:
                    key = closure_key(closure_of(tree, entry), allow_refused=True)[0]
                finally:
                    with open(absolute, "w", encoding="utf-8") as handle:
                        handle.write(original)
                results[name] = (key != base_key, f"{entry} 的键改前 {base_key}、改后 {key}")
        _REAL_IMPORT_TIME_RESULTS[breaks] = results
    return _REAL_IMPORT_TIME_RESULTS[breaks]


def real_import_time_cell(name):
    def judge(_work):
        return real_import_time_results()[name]
    return judge


def excluded_edge_cell(entry, hint, kept):
    """夹具上证 EXCLUDED_PROGRAM_EDGES 的机制：入口单元起 hint 与 kept 两个程序。不登记时两个都进闭包；自证期间临时把夹具里 hint 那一条边登记进表
    （EXCLUDED_EDGE_FIXTURE_EDGES），登记了的 hint 不进闭包、没登记的 kept 照进；跑完把表还原。弄坏开关 program-edge-exclusions-ignored 下登记了也照连，
    program-edge-exclusions-shell-ignored 下 shell 函数起的登记了也照连。"""
    def judge(work):
        base = os.path.join(work, "tree")
        write_tree(base, EXCLUDED_EDGE_FIXTURE)

        def closure_files():
            return {unit.file for unit in closure_of(base, entry).order}

        uncut = closure_files()
        registered = dict(EXCLUDED_PROGRAM_EDGES)
        EXCLUDED_PROGRAM_EDGES.update(EXCLUDED_EDGE_FIXTURE_EDGES)
        try:
            cut = closure_files()
        finally:
            EXCLUDED_PROGRAM_EDGES.clear()
            EXCLUDED_PROGRAM_EDGES.update(registered)
        ok = {hint, kept} <= uncut and kept in cut and hint not in cut
        return ok, (f"{entry}：不登记时 {sorted(uncut & {hint, kept})}、登记 {hint} 那一条边之后 {sorted(cut & {hint, kept})}"
                    f"（要：不登记时 {sorted({hint, kept})} 都在，登记之后只剩 {kept}）")
    return judge


# 「键不变而退出码变」必红那一格（夹具）：quiet.py 与 leak.py 都起 hint.py，两条边自证期间都登记成不走；改坏 hint.py 之后
# quiet.py 吞掉它的退出码、leak.py 把它的退出码当自己的。两个的键都不变，退出码只有 leak.py 变，判法必须认出 leak.py
EXCLUDED_EDGE_EXIT_CELL = "不走的子进程边（夹具）：切了边的程序改坏后包装退出码变了而键不变的必红（对照 leak.py 必须认出），吞掉退出码的 quiet.py 不红"
EXCLUDED_EDGE_EXIT_HEAD = "import os\nimport subprocess\nimport sys\n\nHERE = os.path.dirname(os.path.abspath(__file__))\n\n\n"
EXCLUDED_EDGE_EXIT_LAUNCH = "subprocess.run([sys.executable, os.path.join(HERE, \"hint.py\")], check=False)"
EXCLUDED_EDGE_EXIT_FIXTURE = {
    "tool/hint.py": "def work():\n    return 0\n\n\nif __name__ == \"__main__\":\n    work()\n",
    "tool/quiet.py": EXCLUDED_EDGE_EXIT_HEAD + f"def run():\n    {EXCLUDED_EDGE_EXIT_LAUNCH}\n    return 0\n\n\nif __name__ == \"__main__\":\n    sys.exit(run())\n",
    "tool/leak.py": EXCLUDED_EDGE_EXIT_HEAD + f"def run():\n    return {EXCLUDED_EDGE_EXIT_LAUNCH}.returncode\n\n\nif __name__ == \"__main__\":\n    sys.exit(run())\n",
}
EXCLUDED_EDGE_EXIT_WRAPPERS = ("tool/quiet.py", "tool/leak.py")
EXCLUDED_EDGE_EXIT_EDGES = {(wrapper, "run", "tool/hint.py"): "夹具：自证期间登记成不走" for wrapper in EXCLUDED_EDGE_EXIT_WRAPPERS}
EXCLUDED_EDGE_EXIT_EDIT = ("tool/hint.py", "    work()\n", "    raise RuntimeError(\"selftest\")\n")


def excluded_edge_exit_cell(work):
    """自证期间临时登记 EXCLUDED_EDGE_EXIT_EDGES；每个包装在原树与改坏 hint.py 的树上各量（键、python3 <包装> 的退出码），
    退出码变了而键不变的记下，要恰好是 leak.py。弄坏开关 excluded-edge-exit-unmeasured 下改坏之后不重跑、退出码照改坏之前记。"""
    base, edited = os.path.join(work, "base"), os.path.join(work, "edited")
    write_tree(base, EXCLUDED_EDGE_EXIT_FIXTURE)
    write_tree(edited, EXCLUDED_EDGE_EXIT_FIXTURE)
    edit_file(edited, *EXCLUDED_EDGE_EXIT_EDIT)
    registered = dict(EXCLUDED_PROGRAM_EDGES)
    EXCLUDED_PROGRAM_EDGES.update(EXCLUDED_EDGE_EXIT_EDGES)
    blind, lines = [], []
    try:
        for wrapper in EXCLUDED_EDGE_EXIT_WRAPPERS:
            entry = f"{wrapper}::run"
            before_key = closure_key(closure_of(base, entry), allow_refused=True)[0]
            after_key = closure_key(closure_of(edited, entry), allow_refused=True)[0]
            before_exit = run_fixture_program(base, wrapper)
            after_exit = before_exit if "excluded-edge-exit-unmeasured" in active_breaks() else run_fixture_program(edited, wrapper)
            lines.append(f"{wrapper}：退出码 {before_exit}→{after_exit}，键{'变' if after_key != before_key else '不变'}")
            if after_exit != before_exit and after_key == before_key:
                blind.append(wrapper)
    finally:
        EXCLUDED_PROGRAM_EDGES.clear()
        EXCLUDED_PROGRAM_EDGES.update(registered)
    want = ["tool/leak.py"]
    return blind == want, f"键不变而退出码变的（要恰好是 {want}）：{blind}；{'；'.join(lines)}"


# 登记的准入边（ADMISSION_EDGE）的三格，都在本仓上量（准入异常那一格起本仓的跑法脚本，只走到计划那一步、编译换成假的）
ADMISSION_EDGE_KEY = "E183/h2"
ADMISSION_EDGE_ANOMALY_CELL = ("不走的子进程边（真仓 E183 准入）：准入这条路上的异常（崩、被杀、退 77、退 3、退 0 不打行、打别的行）"
                               "只让计划那一步退非 0、不起编译、不写计划，读数出不来")
ADMISSION_EDGE_STRIP_CELL = "不走的子进程边（真仓 E183 准入）：准入往标准输出打的只有 E7INPUT 行（键算出、拒算两样），判法读产物前去掉这几行之后读数一行不差"
ADMISSION_EDGE_REACH_CELL = "不走的子进程边（真仓 E183 准入）：切边前后少掉的单元都在 admission.py 当程序起的闭包里（只经这条边可达），且确实少了 admission.py"
# 准入异常那一格的假准入：(名, 正文)。每个都先在 ADMISSION_EDGE_MARK 指的文件里记一笔，证明计划那一步真问到了它
ADMISSION_EDGE_ANOMALIES = (
    ("崩（抛异常）", "raise RuntimeError(\"selftest\")\n"),
    ("被 SIGKILL 杀", "import signal\nos.kill(os.getpid(), signal.SIGKILL)\n"),
    ("退 77", "import sys\nsys.exit(77)\n"),
    ("退 3", "import sys\nsys.exit(3)\n"),
    ("退 0 什么都不打", "pass\n"),
    ("退 0 打别的行", "print(\"selftest junk\")\n"),
)
# 弄坏开关 admission-edge-admitted-sample 下多跑的那一个：放行，计划往下走、起编译，这一格必须红
ADMISSION_EDGE_ADMITTED = ("放行（弄坏开关 admission-edge-admitted-sample 加的）",
                           "print(\"E7INPUT name=input_fingerprint key=E183/h2 sha256=\" + \"0123456789abcdef\" * 4 + \" units=2\")\n")
ADMISSION_EDGE_MARK_LINE = "import os\nopen(os.environ[\"ADMISSION_EDGE_MARK\"], \"w\").write(\"asked\")\n"
ADMISSION_EDGE_FAKE_CARGO = "#!/usr/bin/env bash\necho \"$*\" >> \"$ADMISSION_EDGE_CARGO_CALLS\"\nexit 1\n"
ADMISSION_EDGE_DROPPED_VARIABLES = ("CARGO_TARGET_DIR", "MULTI_HOST_RUN_PEER_SCRATCH")
ADMISSION_EDGE_READINGS = ("E7RESULT name=selftest_reading value=1", "E7RESULT name=done emitted=1")


def admission_edge_anomaly_cell(work):
    """每个假准入各起一次本仓跑法脚本的计划那一步（服务给的变量清掉、只设计划那一步要的；运行根、计划目录、产物目录都在临时目录；
    PATH 最前面放记一笔就退 1 的假 cargo）。每一个都要：问到了准入、退非 0、计划目录空、没起编译、没出产物。"""
    script = os.path.join(LIBRARY_ROOT, ADMISSION_EDGE[0])
    if not os.path.isfile(script):
        return False, f"{ADMISSION_EDGE[0]} 不在：准入边的起点没了，从 EXCLUDED_PROGRAM_EDGES 删掉这一条、连同这三格"
    fakes = os.path.join(work, "fakes")
    os.makedirs(fakes)
    cargo = os.path.join(fakes, "cargo")
    with open(cargo, "w", encoding="utf-8") as handle:
        handle.write(ADMISSION_EDGE_FAKE_CARGO)
    os.chmod(cargo, 0o755)
    samples = ADMISSION_EDGE_ANOMALIES + ((ADMISSION_EDGE_ADMITTED,) if "admission-edge-admitted-sample" in active_breaks() else ())
    clean = {name: value for name, value in os.environ.items()
             if not name.endswith("_BREAK") and not name.startswith("TASK_SERVER_") and name not in ADMISSION_EDGE_DROPPED_VARIABLES}
    unasked, leaked, lines = [], [], []
    for number, (name, body) in enumerate(samples):
        cell = os.path.join(work, f"sample-{number}")
        plan, scratch, product = (os.path.join(cell, part) for part in ("plan", "tmp", "product"))
        os.makedirs(plan)
        os.makedirs(scratch)
        admission, mark, calls = (os.path.join(cell, part) for part in ("admission.py", "asked", "cargo-calls"))
        with open(admission, "w", encoding="utf-8") as handle:
            handle.write(ADMISSION_EDGE_MARK_LINE + body)
        environment = dict(clean, PATH=fakes + os.pathsep + clean.get("PATH", ""), TASK_SERVER_SHARD_PHASE="plan",
                           TASK_SERVER_SHARD_PLAN_DIRECTORY=plan, E183_RUN_ROOT=os.path.join(cell, "run-root"), E183_PRODUCT_DIRECTORY=product,
                           E183_ADMISSION_SCRIPT=admission, ADMISSION_EDGE_MARK=mark, ADMISSION_EDGE_CARGO_CALLS=calls, TMPDIR=scratch)
        try:
            completed = subprocess.run(["bash", script, "run"], env=environment, capture_output=True, text=True, timeout=120)
            exit_code, said = completed.returncode, (completed.stdout + completed.stderr).strip().splitlines()[-1:]
        except subprocess.TimeoutExpired:
            exit_code, said = None, ["120 秒没退"]
        planned = sorted(os.listdir(plan))
        produced = os.path.isdir(product) and bool(os.listdir(product))
        compiled = os.path.exists(calls)
        lines.append(f"{name}：退 {exit_code}，计划目录 {planned or '空'}，{'起了编译' if compiled else '没起编译'}，{'出了产物' if produced else '没出产物'}，末行 {said}")
        if not os.path.exists(mark):
            unasked.append(name)
        elif exit_code in (0, None) or planned or compiled or produced:
            leaked.append(name)
    return not unasked and not leaked, (f"没问到准入的（跑法脚本在问准入之前就退了，这一格量不到）：{unasked}；往下走了的：{leaked}"
                                        f"（要都空）；{'；'.join(lines)}")


def admission_edge_strip_cell(_work):
    """载入本仓 admission.py（另起一份模块），实验键换成假的（键算出、拒算两样；不收闭包、不起 rustc）、「输入没变」换成一律变了，
    起 command_experiment 收它的标准输出：每一行都要以 E7INPUT 起头；接上假读数过判法去头的那条命令（admission.py 的 REPLAY_STRIP_HEADER_COMMAND），
    剩下的要与假读数逐行相同。弄坏开关 admission-edge-header-unstripped 下不去头。"""
    import contextlib
    import io
    import types
    path = os.path.join(LIBRARY_ROOT, ADMISSION_EDGE[2])
    specification = importlib.util.spec_from_file_location("admission_for_excluded_edge_selftest", path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    module.judge_inputs_unchanged = lambda _root, _key, _fingerprint: (False, "（自证：不比上一份产物）")
    problems, lines = [], []
    for label, key in (("键算出", "ab" * 32), ("拒算", None)):
        fake = types.SimpleNamespace(key=key, composition=[], refusals=["selftest-form\tselftest.py:1\tselftest"])
        module.experiment_key = lambda _root, _key, fake=fake: (fake, "自证假键")
        printed, messages = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(printed), contextlib.redirect_stderr(messages):
            exit_code = module.command_experiment([LIBRARY_ROOT, ADMISSION_EDGE_KEY])
        header = printed.getvalue().splitlines()
        composed = "".join(line + "\n" for line in header + list(ADMISSION_EDGE_READINGS))
        if "admission-edge-header-unstripped" in active_breaks():
            stripped = composed
        else:
            stripped = subprocess.run(["sh", "-c", module.REPLAY_STRIP_HEADER_COMMAND], input=composed, capture_output=True, text=True,
                                      timeout=60).stdout
        stray = [line for line in header if not line.startswith(module.HEADER_PREFIX)]
        lines.append(f"{label}：退 {exit_code}，打了 {header}，不以 E7INPUT 起头的 {stray}，去头之后 {stripped.splitlines()}")
        if exit_code != module.EXIT_ADMITTED or not header or stray or stripped.splitlines() != list(ADMISSION_EDGE_READINGS):
            problems.append(f"{label}（准入的说明末行 {messages.getvalue().strip().splitlines()[-1:]}）")
    return not problems, f"判错的：{problems}（要空；去头之后要恰好是 {list(ADMISSION_EDGE_READINGS)}）；{'；'.join(lines)}"


def admission_edge_reach_cell(work):
    """本仓 scripts/ 与 .claude/ 的拷贝上收跑法脚本整份的闭包：登记着（切边）与临时摘掉这一条（不切）各一次，少掉的单元都要在
    admission.py 当程序起的闭包里，少掉的里面要有 admission.py 的单元，切边之后不许多出单元。弄坏开关 admission-edge-reach-unmeasured 下
    不量 admission.py 的闭包（当它是空的）。"""
    tree = real_tree(work)
    entry, _unit, program = ADMISSION_EDGE

    def names(closure):
        return {(unit.file, unit.name) for unit in closure.order}

    cut_closure = closure_of(tree, entry)
    cut = names(cut_closure)
    registered = dict(EXCLUDED_PROGRAM_EDGES)
    EXCLUDED_PROGRAM_EDGES.pop(ADMISSION_EDGE, None)
    try:
        uncut = names(closure_of(tree, entry))
    finally:
        EXCLUDED_PROGRAM_EDGES.clear()
        EXCLUDED_PROGRAM_EDGES.update(registered)
    reach = set() if "admission-edge-reach-unmeasured" in active_breaks() else names(closure_of(tree, program))
    removed, added = uncut - cut, cut - uncut
    outside = sorted(removed - reach)
    cut_program = any(file == program for file, _name in removed)
    return cut_program and not outside and not added, (
        f"{entry}：不切 {len(uncut)} 个单元、切边 {len(cut)} 个，少掉 {len(removed)} 个（{'有' if cut_program else '没有'} {program} 的单元），"
        f"少掉的不在 {program} 当程序起的闭包里的 {len(outside)} 个 {outside[:5]}，切边后多出的 {sorted(added)[:5]}；切边后拒算 {len(cut_closure.refusal_list())} 处 "
        f"{[refusal.text() for refusal in cut_closure.refusal_list()[:3]]}（要：有 {program} 的单元、另两样都空）")


# main-guard 的两种写法与几段合一（文件头「Python 闭包：import 时执行的模块级语句」；欠账 C947）那三格
MAIN_GUARD_IMPORT_CELL = ("python：被 import 的模块里 if __name__ == \"__main__\" and …: 那一段不进闭包（自证分支不收），"
                          "块外的模块级语句与认不出的形态（带 else、__name__ 那一项不在最左）照收")
MAIN_GUARD_PROGRAM_CELL = "python：被起的程序里每一段 main-guard 都收（if __name__ == \"__main__\" and …: 那一段与后面另一段），起它的参数不是字面量也连自证分支"
MAIN_GUARD_REAL_CELL = ("真仓：以 scripts/hosts/memory_peaks_lib.py 的 command_key 为判法入口，闭包不收它的自证（run_self_test、self_test_cells），"
                        "改 self_test_cells 的函数体判法单元指纹不变")
# library.py 照 memory_peaks_lib.py 的写法：if __name__ == "__main__" and globals().get("__file__"): 里按 sys.argv 分派到自证；
# fallback.py 是认不出、要照收的两种（带 else、__name__ 那一项不在最左）；twice.py 有两段 main-guard
MAIN_GUARD_FIXTURE = {
    "tool/library.py": ("import sys\n\nHOOKS = []\nLIMIT = 3\n\n\ndef work():\n    return LIMIT\n\n\ndef registered():\n    return 1\n\n\n"
                        "def self_check():\n    return 0\n\n\nHOOKS.append(registered)\n\n"
                        "if __name__ == \"__main__\" and globals().get(\"__file__\"):\n    if sys.argv[1:] == [\"--selftest\"]:\n"
                        "        sys.exit(self_check())\n    sys.exit(work())\n"),
    "tool/fallback.py": ("def flag():\n    return False\n\n\ndef otherwise():\n    return 1\n\n\ndef leftover():\n    return 2\n\n\n"
                         "def value():\n    return 0\n\n\nif __name__ == \"__main__\" and flag():\n    value()\nelse:\n    otherwise()\n\n"
                         "if flag() and __name__ == \"__main__\":\n    leftover()\n"),
    "tool/twice.py": ("import sys\n\n\ndef early():\n    return 1\n\n\ndef late():\n    return 2\n\n\n"
                      "if __name__ == \"__main__\" and len(sys.argv) > 1:\n    early()\n\nif __name__ == \"__main__\":\n    late()\n"),
    "tool/user.py": "import fallback\nimport library\n\n\ndef judge():\n    return library.work() + fallback.value()\n",
    "tool/launcher.py": ("import os\nimport subprocess\nimport sys\n\nHERE = os.path.dirname(os.path.abspath(__file__))\n\n\n"
                         "def launch(flag):\n    subprocess.run([sys.executable, os.path.join(HERE, \"twice.py\")], check=False)\n"
                         "    return subprocess.run([sys.executable, os.path.join(HERE, \"library.py\"), flag], check=False).returncode\n"),
}
MAIN_GUARD_REAL_LIBRARY = "scripts/hosts/memory_peaks_lib.py"
MAIN_GUARD_REAL_ENTRY = f"{MAIN_GUARD_REAL_LIBRARY}::command_key"   # 峰值表的键：库里不碰自证的一个函数；做入口时库的 module-body 照收、main-guard 不收
MAIN_GUARD_REAL_SELFTEST_UNITS = ("run_self_test", "self_test_cells")
MAIN_GUARD_REAL_EDITED = "self_test_cells"   # 改它的函数体：它只经 main-guard 里的 run_self_test 到，闭包不收它，指纹就不变


def main_guard_closure_names(closure):
    return {(unit.file, unit.name) for unit in closure.order}


def main_guard_fixture_cell(entry, wanted, unwanted):
    """夹具上收 entry 的闭包：wanted 里的（文件, 限定名）都要在，unwanted 里的一个都不许在。"""
    def judge(work):
        base = os.path.join(work, "tree")
        write_tree(base, MAIN_GUARD_FIXTURE)
        names = main_guard_closure_names(closure_of(base, entry))
        missing, extra = sorted(set(wanted) - names), sorted(set(unwanted) & names)
        return not missing and not extra, f"{entry} 的闭包里缺 {missing}、多 {extra}（要：{sorted(wanted)} 都在，{sorted(unwanted)} 都不在）"
    return judge


def merged_judging_closure(root, entries):
    """crates 变异判法那一套入口（JUDGING_ENTRIES）从一个 ScriptClosure 一起收：lib_mutation_units.judging_units 逐个入口各收再合起来，
    一起收只多不少（类与属性的收窄跨入口共用），这里不含的单元逐个入口收也不含。"""
    parsed = [parse_entry(root, entry) for entry in entries]
    for entry, (language, _path, qualified) in zip(entries, parsed):
        if language != "python" or not qualified:
            raise AssertionError(f"crates 变异的判法入口 {entry} 不是 python 的 <文件>::<限定名>")
    under = [place for place in DEFAULT_SCRIPT_DIRECTORIES if os.path.isdir(os.path.join(root, place))]
    script = ScriptClosure(root, list(dict.fromkeys(under + [posixpath.dirname(path) or "." for _language, path, _qualified in parsed])))
    units = []
    for _language, path, qualified in parsed:
        module = script.python.module(path)
        found = [unit for unit in module.units if unit.name == qualified and not unit.data] if module is not None else []
        if len(found) != 1:
            raise AssertionError(f"{path} 里限定名 {qualified} 命中 {len(found)} 个单元（要恰好一个）")
        script.closure.refusals.update(module.refusals)
        units += found
    return script.run(units)


def judging_units_fingerprint(closure):
    """照 lib_mutation_units.judging_units 的口径：闭包里的单元按（文件, 种类, 限定名, 起行）去重，（种类, 指纹）的多重集排好连起来取 sha256。"""
    members = {(unit.file, unit.kind, unit.name, unit.first_line): unit for unit in closure.order}
    return sha256_text("".join(sorted(f"{unit.kind}\t{unit.fingerprint}\n" for unit in members.values())))


def via_chain(closure, unit):
    """闭包里一个单元经哪几步收进来（closure.via 一路往回到入口；内联脚本的单元不在 via 里，走到它为止）：诊断用。"""
    steps, seen = [f"{unit.file}:{unit.name}"], set()
    while id(unit) in closure.via and id(unit) not in seen:
        seen.add(id(unit))
        via, rule = closure.via[id(unit)]
        if via is None:
            break
        steps.append(f"←{rule}— {via.file}:{via.name}")
        unit = via
    return " ".join(steps)


def body_statement_inserted(text, name):
    """text 里模块级 def name 的函数体头一句之前插一句赋值（这个单元的指纹变）→ 改过的原文；找不到这个 def、函数体与 def 在同一行、插完解不开交 None。"""
    for node in ast.parse(text).body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name == name and node.body[0].lineno > node.lineno:
            rows = text.split("\n")
            row = rows[node.body[0].lineno - 1]
            rows.insert(node.body[0].lineno - 1, row[:len(row) - len(row.lstrip())] + "selftest_inserted = 1")
            edited = "\n".join(rows)
            try:
                ast.parse(edited)
            except SyntaxError:
                return None
            return edited
    return None


def main_guard_real_cell(work):
    """真仓拷贝上，以 memory_peaks_lib.py 的 command_key 为判法入口收闭包：它的 run_self_test、self_test_cells 不许在（自证只在库末尾 main-guard 里，
    入口在库里时库的 module-body 照收、main-guard 不收）；再改拷贝里 self_test_cells 的函数体，判法单元指纹不变。"""
    tree = real_tree(work)
    entries = [MAIN_GUARD_REAL_ENTRY]
    closure = merged_judging_closure(tree, entries)
    notes = [f"入口 {MAIN_GUARD_REAL_ENTRY} 收 {len(closure.order)} 个单元"]
    unwanted = [unit for unit in closure.order if unit.file == MAIN_GUARD_REAL_LIBRARY and unit.name in MAIN_GUARD_REAL_SELFTEST_UNITS]
    for unit in sorted(unwanted, key=Unit.sort_key):
        notes.append(f"不该在的 {unit.file}:{unit.name} 的来路 {via_chain(closure, unit)}")
    path = os.path.join(tree, MAIN_GUARD_REAL_LIBRARY)
    with open(path, encoding="utf-8") as handle:
        edited = body_statement_inserted(handle.read(), MAIN_GUARD_REAL_EDITED)
    if edited is None:
        return False, "；".join(notes + [f"{MAIN_GUARD_REAL_LIBRARY} 里找不到能插一句的模块级 def {MAIN_GUARD_REAL_EDITED}：照新原文改 MAIN_GUARD_REAL_EDITED"])
    before = judging_units_fingerprint(closure)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(edited)
    after = judging_units_fingerprint(merged_judging_closure(tree, entries))
    notes.append(f"改拷贝里 {MAIN_GUARD_REAL_LIBRARY} 的 {MAIN_GUARD_REAL_EDITED}：判法单元指纹 {before}→{after}")
    return not unwanted and before == after, "；".join(notes)


def real_entries_cell(work):
    tree = real_tree(work)
    entries = registered_judging_entries(tree)
    formal = real_entries(tree)
    unknown = []
    for entry in entries:
        language, path, qualified = parse_entry(tree, entry)
        if not os.path.isfile(os.path.join(tree, path)):
            unknown.append(entry)
    return not unknown and len(entries) > 2, f"登记现取 {len(entries)} 个判法入口（形式化入口 {formal}），文件不在的 {unknown}"


REAL_JUDGING_CLOSURES_CELL = "真仓：登记的每个判法入口的闭包都不拒算（拒算的那一路判定一条都写不进库），夹具里的 globals() 查找判得出来"
JUDGING_REFUSAL_FIXTURE = {"tool/judge.py": 'def verdict(row):\n    original = globals()["verdict"]\n    return original is not None and row\n\n\ndef judge(row):\n    return verdict(row)\n'}


def judging_entry_refusals_of(job):
    """(入口, 拒算行)：一个入口的闭包里算进判法单元指纹的拒算——crates 变异的入口全算，别的入口去掉 lib_verdict_library.py 的 NOT_JUDGING_FILES 那几份
    （照 judging_unit_key 与 lib_mutation_units.judging_units 的口径；弄坏开关 judging-entry-refusals-ignored 下一条都不报）。"""
    root, entry, not_judging = job
    closure = closure_of(root, entry)
    if "judging-entry-refusals-ignored" in active_breaks():
        return entry, []
    return entry, sorted(refusal.text() for refusal in closure.refusals if refusal.file not in not_judging)


def judging_entry_refusal_workers(environment, registration_file=None):
    """judging_entry_refusals 开几个进程：这份库所在仓的 scripts/hosts/heavy_threads.py 的 default_heavy_threads（SINGLEFS_THREAD_CAP 设了照它，
    没设取登记的每台默认重活线程数 .claude/heavy-thread-default；取法只在那一处，这里不另解析；按路径载入、不进 sys.modules），
    取不到照它抛 HeavyThreadDefaultError，不退回按核数算。registration_file 只给自证换登记文件。
    弄坏开关 judging-entry-workers-head-word：取登记文件的头一个词，读不出就不用登记的数（退回按核数算；这里退回 1 个进程，
    不调取核数的函数，门禁 code-tooling 的 heavy-thread-defaults 格不许）。那份文件头一个词是 #，总是读不出。"""
    if "judging-entry-workers-head-word" in active_breaks():
        try:
            with open(registration_file or os.path.join(LIBRARY_ROOT, ".claude", "heavy-thread-default"), encoding="utf-8") as handle:
                return int(handle.read().split()[0])
        except (OSError, ValueError, IndexError):
            return 1
    specification = importlib.util.spec_from_file_location("heavy_threads_for_judging_entry_refusals",
                                                           os.path.join(LIBRARY_ROOT, "scripts", "hosts", "heavy_threads.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return int(module.default_heavy_threads(environment, registration_file=registration_file))


def judging_entry_refusals(root, entries, not_judging_by_entry):
    """{入口: 拒算行}，只列有拒算的；按每台默认重活线程数并行（judging_entry_refusal_workers；一个入口要几秒到十几秒）。"""
    import concurrent.futures
    workers = judging_entry_refusal_workers(os.environ)
    jobs = [(root, entry, not_judging_by_entry.get(entry, ())) for entry in entries]
    with concurrent.futures.ProcessPoolExecutor(max_workers=max(1, min(workers, len(jobs)))) as pool:
        results = list(pool.map(judging_entry_refusals_of, jobs))
    return {entry: refusals for entry, refusals in results if refusals}


def real_judging_closures_cell(work):
    tree = real_tree(work)
    entries = registered_judging_entries(tree)
    not_judging = tuple(module_literal(tree, "scripts/crash/lib_verdict_library.py", "NOT_JUDGING_FILES"))
    mutation_entries = set(module_literal(tree, "scripts/mutation/crates-mutation-rows.py", "JUDGING_ENTRIES"))
    real = judging_entry_refusals(tree, entries, {entry: () if entry in mutation_entries else not_judging for entry in entries})
    fixture = os.path.join(work, "fixture")
    write_tree(fixture, JUDGING_REFUSAL_FIXTURE)
    probe = judging_entry_refusals(fixture, ["tool/judge.py::judge"], {})
    probe_forms = {line.split("\t")[1] for lines in probe.values() for line in lines}
    return not real and probe_forms == {"globals-lookup"}, (f"真仓 {len(entries)} 个判法入口里拒算的 {len(real)} 个：{real}（照拒算那一行改成算得出键的写法：拒算的判定不入库、下一趟照判）；"
                                      f"夹具里 globals() 查找判出 {probe}（要判出一条）")


# 跨轮缓存的登记（文件头「跨轮缓存的登记」）那两格
CROSS_RUN_CACHE_CELL = ("真仓：跨轮缓存登记表（cross_run_cache_registry）里的每一份，在 TMPDIR 指到一个临时目录、GATE_CROSS_RUN_TMPDIR 没设、在 git 仓里"
                        "（.git 是目录，或 git worktree 里 .git 是文件）时现取的位置都不落在 TMPDIR 之下；worktree 里落在 git 目录底下的几份与主仓共用 common-dir 那一处")
CROSS_RUN_CACHE_SCAN_CELL = ("真仓：提到 GATE_CROSS_RUN_TMPDIR 的脚本都登记进跨轮缓存表或在不登记表里写明为什么，不登记表里的每一份都在、都还提到它；"
                             "夹具里没登记的一份判得出来")
CROSS_RUN_CACHE_PROBE_SECONDS = 300
CROSS_RUN_CACHE_MENTION = "GATE_CROSS_RUN_TMPDIR"
CROSS_RUN_CACHE_SCANNED_DIRECTORIES = ("scripts", ".claude/gate.d", ".claude/scripts", ".claude/hooks", "task-server", "prune-server")
CROSS_RUN_CACHE_SCANNED_SUFFIXES = (".py", ".sh", ".bash")
CROSS_RUN_CACHE_SKIPPED_DIRECTORIES = frozenset(("fixtures", "__pycache__", "target", ".git", "node_modules"))
# 提到 GATE_CROSS_RUN_TMPDIR 而不往那里放跨轮缓存的脚本：{路径: 为什么不登记}；只缩不涨，指不到的、不再提到它的这一格判红
CROSS_RUN_CACHE_NOT_CACHES = {
    "scripts/hosts/test-environment-check.py": "只把它当 gate.sh 开跑前的临时目录、在那里扫残留，不往里放跨轮的东西",
    "scripts/kb/verify-citations.sh": "文本缓存每趟自己 mktemp -d、跑完连目录删；这个名字只出现在弄坏开关（改前的样子）与自证里",
}
CROSS_RUN_CACHE_SCAN_FIXTURE = {
    "scripts/sample/keeps-cache.sh": '#!/usr/bin/env bash\ncache_directory="${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/sample-cache"\nmkdir -p "$cache_directory"\n',
    "scripts/sample/plain.sh": "#!/usr/bin/env bash\necho plain\n",
}


def cross_run_cache_layout(work, tree, label):
    """另建空目录 <work>/<label> 当 TMPDIR，摘掉 GATE_CROSS_RUN_TMPDIR 与几份缓存自己的改放处变量，起子进程跑 tree 里这份库的命令行 cross-run-caches
    → (过没过, 说明, {名字: 现取的路径})：子进程退 0、它的 tempfile.gettempdir() 是那个目录、登记的每一份都算得出、没有一份落在它之下才算过。"""
    temporary = os.path.join(work, label)
    os.makedirs(temporary)
    overrides = {"GATE_CROSS_RUN_TMPDIR", RUST_LEX_CACHE_DIRECTORY_VARIABLE,
                 module_literal(tree, "scripts/mutation/crates-mutation-rows.py", "JUDGING_CACHE_DIRECTORY_VARIABLE")}
    environment = {name: value for name, value in os.environ.items() if name not in overrides}
    environment["TMPDIR"] = temporary
    work_root = work_root_library()
    # 拷贝里没有 multi-host.env：数据根没经环境给时，把本仓多机配置的位置带过去（test-environment 那一份缓存放数据根下）
    environment.setdefault(work_root.MULTI_HOST_CONFIGURATION_VARIABLE, work_root.multi_host_configuration_path(os.environ))
    try:
        completed = subprocess.run([sys.executable, "-B", os.path.join(tree, "scripts", "gate", "lib_function_units.py"), "cross-run-caches"],
                                   capture_output=True, text=True, errors="surrogateescape", env=environment, stdin=subprocess.DEVNULL,
                                   timeout=CROSS_RUN_CACHE_PROBE_SECONDS)
    except subprocess.TimeoutExpired:
        return False, f"{label}：子进程 {CROSS_RUN_CACHE_PROBE_SECONDS} 秒没退（拷贝里的这份库或它载入的 crates-mutation-rows.py 卡住）", {}
    output = completed.stdout.splitlines()
    temporary_root = os.path.realpath(temporary)
    seen_tempdirs = [line.split("\t", 1)[1] for line in output if line.startswith("tempdir\t")]
    rows = [fields for fields in (line.split("\t", 3) for line in output if line.startswith(("cache\t", "refused-cache\t"))) if len(fields) == 4]

    def under_temporary(path):
        resolved = os.path.realpath(path)
        return resolved == temporary_root or resolved.startswith(temporary_root + os.sep)

    under = [f"{name}（{script}）→ {path}" for kind, name, script, path in rows if kind == "cache" and under_temporary(path)]
    refused = [f"{name}（{script}）：{reason}" for kind, name, script, reason in rows if kind == "refused-cache"]
    registered = [name for name, _script, _locate in cross_run_cache_registry()]
    tempdir_held = len(seen_tempdirs) == 1 and os.path.realpath(seen_tempdirs[0]) == temporary_root
    ok = completed.returncode == 0 and tempdir_held and [fields[1] for fields in rows] == registered and not under and not refused
    listed = "；".join(f"{name} → {path}" for _kind, name, _script, path in rows)
    return ok, (f"{label}：子进程退 {completed.returncode}；它的 tempfile.gettempdir() {seen_tempdirs}（要恰好一行 {temporary_root}）；"
                f"现取 {len(rows)} 份（登记 {len(registered)} 份）：{listed}；落在 TMPDIR 之下的 {under}；算不出的 {refused}；"
                f"stderr 末尾：{completed.stderr.strip()[-300:]}"), {name: path for kind, name, _script, path in rows if kind == "cache"}


def cross_run_cache_cell(work):
    """两趟 cross_run_cache_layout：本仓 scripts/ 与 .claude/ 拷进临时目录（real_tree）、建空的 .git 起一趟；git 造一个主仓与它的 worktree
    （worktree 里 .git 是文件），拷进 worktree 再起一趟。两趟都要过；头一趟落在 <拷贝>/.git 底下的那几份（至少一份），第二趟要落在主仓 .git
    （git common-dir）底下同一相对路径。弄坏开关 cross-run-cache-under-tmpdir 下 cross_run_cache_directory 放回 TMPDIR，两趟都有落进去的；
    cross-run-cache-git-directory-only 下只认 .git 是目录，worktree 那一趟本库两份与 crates-mutation-rows.py 那一份落进那一趟的 TMPDIR；都判错。"""
    tree = real_tree(work)
    os.makedirs(os.path.join(tree, ".git"))
    plain_ok, plain_detail, plain_paths = cross_run_cache_layout(work, tree, "tmpdir")
    main, linked = os.path.join(work, "worktree-main"), os.path.join(work, "worktree-linked")
    problem = selftest_git_steps(work, (["init", "-q", main], ["-C", main, "commit", "-q", "--allow-empty", "-m", "base"],
                                        ["-C", main, "worktree", "add", "-q", "--detach", linked]))
    if problem is not None:
        return False, f"{plain_detail}；造 worktree：{problem}"
    if not os.path.isfile(os.path.join(linked, ".git")):
        return False, f"{plain_detail}；git worktree add 造出的 {linked}/.git 不是文件，样本没造对"
    real_tree(work, linked)
    linked_ok, linked_detail, linked_paths = cross_run_cache_layout(work, linked, "worktree-tmpdir")
    plain_git, main_git = os.path.realpath(os.path.join(tree, ".git")), os.path.realpath(os.path.join(main, ".git"))
    in_git = {name: os.path.relpath(os.path.realpath(path), plain_git) for name, path in plain_paths.items()
              if os.path.realpath(path).startswith(plain_git + os.sep)}
    unshared = sorted(f"{name} → {linked_paths.get(name)}（要 {os.path.join(main_git, relative)}）" for name, relative in in_git.items()
                      if name not in linked_paths or os.path.realpath(linked_paths[name]) != os.path.join(main_git, relative))
    ok = plain_ok and linked_ok and bool(in_git) and not unshared
    return ok, (f"{plain_detail}；{linked_detail}；.git 是目录那一趟落在 .git 底下的 {sorted(in_git)}（要至少一份）；"
                f"worktree 那一趟没落到主仓 common-dir 同一处的 {unshared}")


def cross_run_cache_mentions(root):
    """root 下 CROSS_RUN_CACHE_SCANNED_DIRECTORIES 里（跳过 CROSS_RUN_CACHE_SKIPPED_DIRECTORIES）提到 GATE_CROSS_RUN_TMPDIR 的 .py / .sh / .bash，相对 root、排好序。
    弄坏开关 cross-run-cache-mentions-unscanned：交空表。"""
    if "cross-run-cache-mentions-unscanned" in active_breaks():
        return []
    found = []
    for top in CROSS_RUN_CACHE_SCANNED_DIRECTORIES:
        for directory, subdirectories, names in os.walk(os.path.join(root, top)):
            subdirectories[:] = sorted(name for name in subdirectories if name not in CROSS_RUN_CACHE_SKIPPED_DIRECTORIES)
            for name in sorted(names):
                if not name.endswith(CROSS_RUN_CACHE_SCANNED_SUFFIXES):
                    continue
                path = os.path.join(directory, name)
                try:
                    with open(path, "rb") as handle:
                        mentioned = CROSS_RUN_CACHE_MENTION.encode("ascii") in handle.read()
                except OSError:
                    continue
                if mentioned:
                    found.append(os.path.relpath(path, root))
    return sorted(found)


def cross_run_cache_scan_cell(work):
    """本仓里提到 GATE_CROSS_RUN_TMPDIR 的脚本都是登记表里的、或在 CROSS_RUN_CACHE_NOT_CACHES 里写明为什么不登记；不登记表里的每一份都在、都还提到它；
    夹具里一份没登记的（scripts/sample/keeps-cache.sh）判得出来。弄坏开关 cross-run-cache-mentions-unscanned 下一份都扫不出，这一格判错。"""
    registered = {script for _name, script, _locate in cross_run_cache_registry()}
    mentioned = cross_run_cache_mentions(LIBRARY_ROOT)
    unaccounted = [path for path in mentioned if path not in registered and path not in CROSS_RUN_CACHE_NOT_CACHES]
    stale = [path for path in sorted(CROSS_RUN_CACHE_NOT_CACHES) if path not in mentioned]
    fixture = os.path.join(work, "fixture")
    write_tree(fixture, CROSS_RUN_CACHE_SCAN_FIXTURE)
    probe = cross_run_cache_mentions(fixture)
    ok = not unaccounted and not stale and probe == ["scripts/sample/keeps-cache.sh"]
    return ok, (f"本仓提到 {CROSS_RUN_CACHE_MENTION} 的 {len(mentioned)} 份：{mentioned}；没登记也没写明为什么的 {unaccounted}"
                f"（往 cross_run_cache_registry 加一行，或不是缓存的写进 CROSS_RUN_CACHE_NOT_CACHES）；不登记表里指不到或不再提到它的 {stale}（删掉那一行）；"
                f"夹具里扫出 {probe}（要恰好 scripts/sample/keeps-cache.sh）")


# 两个互相收进对方闭包的入口（键里的 entry 行，entry_key_lines）：一个 trait impl 里的两个方法（方法连 impl 头、impl 头点名类型、类型照 R-隐式收它的
# trait impl 全部成员，两个方法收到的单元一模一样；core 的 impl BlockDevice for FileBackedBlockDevice 的样子），与 python 里互相调用的两个 def
ENTRY_UNIT_KEY_CELL = "键：两个互相收进对方闭包的入口，键不同（一个 trait impl 里的两个方法、python 里互相调用的两个 def：闭包的单元多重集相同，键各是各的）"
ENTRY_UNIT_KEY_RUST_FILE = "crates/fsiblings/src/lib.rs"
ENTRY_UNIT_KEY_FIXTURE = {
    "crates/fsiblings/Cargo.toml": '[package]\nname = "fsiblings"\n',
    ENTRY_UNIT_KEY_RUST_FILE: '''pub trait SampleStore {
    fn fill(&self, count: u64) -> u64;
    fn width(&self) -> u64;
}

pub struct SampleDevice {
    pub width: u64,
}

impl SampleStore for SampleDevice {
    fn fill(&self, count: u64) -> u64 {
        count.min(self.width)
    }

    fn width(&self) -> u64 {
        self.width
    }
}
''',
    "tool/pair.py": "def ping(count):\n    return pong(count - 1) if count > 0 else 0\n\n\ndef pong(count):\n    return ping(count - 1) if count > 0 else 1\n",
}
ENTRY_UNIT_KEY_PAIRS = (
    ("Rust", (f"{ENTRY_UNIT_KEY_RUST_FILE}::<impl SampleStore for SampleDevice>::fill", f"{ENTRY_UNIT_KEY_RUST_FILE}::<impl SampleStore for SampleDevice>::width")),
    ("python", ("tool/pair.py::ping", "tool/pair.py::pong")),
)


def entry_unit_key_cell(work):
    """每一对入口先核夹具造对了（两个闭包的单元多重集相同），再核两把键都算得出、不相同。弄坏开关 key-entry-units-dropped 下键里不出 entry 行，两把键相同，这一格判错。"""
    base = os.path.join(work, "tree")
    write_tree(base, ENTRY_UNIT_KEY_FIXTURE)
    problems, lines = [], []
    for language, entries in ENTRY_UNIT_KEY_PAIRS:
        closures = [closure_of(base, entry) for entry in entries]
        multisets = [sorted(f"{unit.kind}\t{unit.fingerprint}" for unit in closure.order) for closure in closures]
        keys = [closure_key(closure)[0] for closure in closures]
        lines.append(f"{language}：两个闭包{'相同' if multisets[0] == multisets[1] else '不同'}（各 {len(multisets[0])}、{len(multisets[1])} 个单元），键 {keys}")
        if multisets[0] != multisets[1]:
            problems.append(f"{language} 两个入口的闭包不同：夹具没造出互相收进对方闭包的样子，照 {entries} 的闭包改夹具")
        elif None in keys:
            problems.append(f"{language} 两个入口的键算不出")
        elif keys[0] == keys[1]:
            problems.append(f"{language} 两个入口的键相同：键里没有入口单元自己（entry_key_lines）")
    return not problems, "；".join(problems + lines)


# unit_graph_key 的 entry 行（照 crates/singlefs-unit-graph 的 reuse_key 与 entry_key_lines）：一个 trait impl 里的方法入口（entry 行连 impl 头），
# 一个挂了两个文档测试的条目（文档测试入口记所挂条目的全部文档测试，照 Rust 那一份 resolve_entry_units 文档测试那一支）
ENTRY_LINES_SHAPE_CELL = ("键：unit_graph_key 照 crates/singlefs-unit-graph 的 reuse_key 带 entry 行（数据行之后、工具链行之前，排序去重；impl 方法入口连 impl 头；"
                          "文档测试入口记所挂条目的全部文档测试、各一行，闭包从它们一起收）")
ENTRY_LINES_RUST_KEYS_CELL = ("键：夹具上 unit_graph_key 与 crates/singlefs-unit-graph 的 unit-graph closure 打的键逐字相同"
                              "（一个 impl 方法入口、一个文档测试入口；Rust 那一侧是录下的键 ENTRY_LINES_RUST_KEYS）")
ENTRY_LINES_FILE = "crates/fentrylines/src/lib.rs"
ENTRY_LINES_METHOD = "<impl Shape for Square>::area"
ENTRY_LINES_HOST_LINE = "host: x86_64-unknown-linux-gnu"
ENTRY_LINES_FIXTURE = {
    "crates/fentrylines/Cargo.toml": '[package]\nname = "fentrylines"\nversion = "0.1.0"\nedition = "2021"\n',
    ENTRY_LINES_FILE: '''pub trait Shape {
    fn area(&self) -> u64;
}

pub struct Square {
    pub side: u64,
}

impl Shape for Square {
    fn area(&self) -> u64 {
        self.side * self.side
    }
}

/// Twice the value.
///
/// ```
/// assert_eq!(fentrylines::documented(2), 4);
/// ```
///
/// ```
/// assert_eq!(fentrylines::documented(3), 6);
/// ```
pub fn documented(value: u64) -> u64 {
    value * 2
}
''',
}
# Rust 那一侧录下的键（unit-graph closure 打的 key 行）：ENTRY_LINES_FIXTURE 用 write_tree 写出成一棵树（<根>），带 entry 行的那一版 crates/singlefs-unit-graph 编的
#   unit-graph closure <根> fentrylines::Square::area --toolchain-line 'host: x86_64-unknown-linux-gnu'               → "impl-method"
#   unit-graph closure <根> fentrylines::documented --target doctest --toolchain-line 'host: x86_64-unknown-linux-gnu'  → "doctest"
# 还没录的写 None（那一格判红、说明里打出 python 这一边的键）。夹具或 Rust 那一份的键的写法改了就重录。
ENTRY_LINES_RUST_KEYS = {
    "impl-method": "e74ceaa6abb0198ba10e62a909dccfc1e321557f4775cae362a3481a5a8762e7",
    "doctest": "3597d88a25cc2c3fad7d6f222e704d2ea205ccd9fe3c41ab81451c81391e6670",
}


def entry_lines_closures(work):
    """写出 ENTRY_LINES_FIXTURE → (documented 挂的文档测试单元（按 Unit.sort_key）, {"impl-method" / "doctest": (入口单元, 闭包)})；
    文档测试入口取头一个文档测试。夹具里 documented 挂的不是两个就抛 AssertionError。"""
    base = os.path.join(work, "tree")
    write_tree(base, ENTRY_LINES_FIXTURE)
    workspace = RustWorkspace(base)
    doctests = sorted((unit for unit in workspace.file(ENTRY_LINES_FILE).units if unit.kind == "doctest"), key=Unit.sort_key)
    if len(doctests) != 2:
        raise AssertionError(f"夹具里 documented 要挂两个文档测试，实际 {[unit.name for unit in doctests]}：照 ENTRY_LINES_FIXTURE 改")
    closures = {}
    for label, qualified in (("impl-method", ENTRY_LINES_METHOD), ("doctest", doctests[0].name)):
        unit, target_id = workspace.find_entry(ENTRY_LINES_FILE, qualified, None)
        closures[label] = (unit, rust_closure_of(workspace, unit, target_id))
    return doctests, closures


def entry_lines_shape_cell(work):
    """两个入口的 unit_graph_key_lines：entry 行在 entry-target-kind、cfg、unit、data 行之后、toolchain 行之前，排好序、不重；
    impl 方法入口一行（fn 连 impl 头）；文档测试入口两行（各是一个 doctest），Closure.entry_units 恰好是那两个文档测试、闭包里两个都在。"""
    doctests, closures = entry_lines_closures(work)
    problems, notes = [], []
    want_kinds = {"impl-method": [["fn", "impl"]], "doctest": [["doctest"], ["doctest"]]}
    for label, (_unit, closure) in closures.items():
        lines = unit_graph_key_lines(closure, [ENTRY_LINES_HOST_LINE])
        entry_positions = [index for index, line in enumerate(lines) if line.startswith("entry\t")]
        entry_rows = [lines[index] for index in entry_positions]
        before = [index for index, line in enumerate(lines) if line.startswith(("entry-target-kind\t", "cfg\t", "unit\t", "data\t"))]
        after = [index for index, line in enumerate(lines) if line.startswith("toolchain\t")]
        kinds = [row.split("\t")[1::2] for row in entry_rows]
        notes.append(f"{label}：entry 行的种类链 {kinds}，在第 {entry_positions} 行（共 {len(lines)} 行）")
        if kinds != want_kinds[label]:
            problems.append(f"{label} 的 entry 行种类链要 {want_kinds[label]}")
        if entry_positions and not (max(before) < min(entry_positions) and max(entry_positions) < min(after)):
            problems.append(f"{label} 的 entry 行不在数据行之后、工具链行之前")
        if entry_rows != sorted(set(entry_rows)):
            problems.append(f"{label} 的 entry 行没排序去重")
    doctest_closure = closures["doctest"][1]
    recorded = sorted(unit.name for unit in doctest_closure.entry_units)
    collected = [unit.name for unit in doctests if id(unit) not in doctest_closure.members]
    if recorded != sorted(unit.name for unit in doctests) or collected:
        problems.append(f"文档测试入口记的入口 {recorded}，要 documented 挂的全部 {[unit.name for unit in doctests]}；闭包里缺 {collected}")
    return not problems, "；".join(problems + notes)


def entry_lines_rust_keys_cell(work):
    """两个入口的 unit_graph_key 与 ENTRY_LINES_RUST_KEYS 录下的 Rust 键逐字相同；还没录的判红，说明里打出 python 这一边的键。"""
    _doctests, closures = entry_lines_closures(work)
    problems, notes = [], []
    for label, (_unit, closure) in closures.items():
        key, refusals = unit_graph_key(closure, [ENTRY_LINES_HOST_LINE])
        recorded = ENTRY_LINES_RUST_KEYS.get(label)
        notes.append(f"{label}：python {key or '拒算：' + '；'.join(refusal.text() for refusal in refusals[:2])}，Rust 录下的 {recorded}")
        if recorded is None:
            problems.append(f"{label} 的 Rust 键还没录（照 ENTRY_LINES_RUST_KEYS 上面的注释用 unit-graph closure 录）")
        elif key != recorded:
            problems.append(f"{label} 两份的键不同")
    return not problems, "；".join(problems + notes)


# 标准写法的 main-guard 带 else：if 那一支是 main-guard，else 那一支在被 import 时执行、进 module-body
MAIN_GUARD_ELSE_CELL = ("python：if __name__ == \"__main__\": … else: … 被 import 时 else 那一支照收、if 那一支不收；被当程序起时 if 那一支照收")
MAIN_GUARD_ELSE_FIXTURE = {
    "tool/either.py": ("import sys\n\n\ndef main_work():\n    return 1\n\n\ndef imported_work():\n    return 2\n\n\ndef value():\n    return 0\n\n\n"
                       "if __name__ == \"__main__\":\n    sys.exit(main_work())\nelse:\n    imported_work()\n"),
    "tool/importer.py": "import either\n\n\ndef judge():\n    return either.value()\n",
    "tool/runner.py": ("import os\nimport subprocess\nimport sys\n\nHERE = os.path.dirname(os.path.abspath(__file__))\n\n\n"
                       "def run():\n    return subprocess.run([sys.executable, os.path.join(HERE, \"either.py\")], check=False).returncode\n"),
}


def main_guard_else_cell(work):
    """import either 的 judge：imported_work（else 那一支调的）在、main_work（if 那一支调的）与 <main-guard> 不在；
    起 either.py 的 run：main_work 在（if 那一支照旧是 main-guard、当程序起时连）。"""
    base = os.path.join(work, "tree")
    write_tree(base, MAIN_GUARD_ELSE_FIXTURE)
    imported = main_guard_closure_names(closure_of(base, "tool/importer.py::judge"))
    launched = main_guard_closure_names(closure_of(base, "tool/runner.py::run"))
    missing = sorted({("tool/either.py", "imported_work"), ("tool/either.py", "value")} - imported)
    extra = sorted({("tool/either.py", "main_work"), ("tool/either.py", "<main-guard>")} & imported)
    unlaunched = sorted({("tool/either.py", "main_work"), ("tool/either.py", "<main-guard>")} - launched)
    return not missing and not extra and not unlaunched, (f"被 import 时闭包里缺 {missing}、多 {extra}；被当程序起时缺 {unlaunched}")


# 判法入口拒算的并行进程数（judging_entry_refusal_workers）
JUDGING_ENTRY_WORKERS_CELL = ("判法入口拒算的并行进程数：没设 SINGLEFS_THREAD_CAP 取登记的每台默认重活线程数（头一行是注释的登记文件照样取到），"
                              "设了照它，不退回按核数算")


def judging_entry_workers_cell(work):
    """自证造一份登记文件（头一行注释，再一行「7  # 理由」）：没设 SINGLEFS_THREAD_CAP 取到 7，设成 3 取到 3；真仓的登记照样取得到。"""
    registered = 7
    path = os.path.join(work, "heavy-thread-default")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(f"# 自证造的登记：头一个词是 #\n{registered}  # 自证样本\n")
    unset = judging_entry_refusal_workers({}, registration_file=path)
    capped = judging_entry_refusal_workers({"SINGLEFS_THREAD_CAP": "3"}, registration_file=path)
    real = judging_entry_refusal_workers({})
    return unset == registered and capped == 3 and real >= 1, (f"没设 SINGLEFS_THREAD_CAP 取到 {unset}（要登记的 {registered}）、"
                                                              f"设成 3 取到 {capped}、真仓的登记取到 {real}")


# verus! 里的 assume_specification（与 crates/singlefs-unit-graph 的
# tests/unit_graph_verus_assume_specifications_are_units_in_the_formal_view_and_blanked_in_the_product_view.rs 同一份夹具、同一套期望）
ASSUME_SPECIFICATION_FILE = "crates/fassume/src/lib.rs"
ASSUME_SPECIFICATION_FIXTURE = {
    "crates/fassume/Cargo.toml": '[package]\nname = "fassume"\nversion = "0.1.0"\nedition = "2021"\n',
    ASSUME_SPECIFICATION_FILE: '''pub const LIMIT_IN_BYTES: u64 = 32;

pub fn width_of(value: u64) -> u64 {
    value + 1
}

pub fn judge_limit() -> u64 {
    LIMIT_IN_BYTES
}

verus! {

#[allow(unsafe_code, reason = "夹具：Verus 把 assume_specification 展开成 unsafe fn 声明")]
pub assume_specification[ crate::LIMIT_IN_BYTES ] -> (limit: u64) returns 32u64;

assume_specification[ width_of ](value: u64) -> (width: u64)
    ensures width == value + 1;

pub assume_specification<Element>[ Vec::<Element>::len ](vector: &Vec<Element>) -> (length: usize);

pub open spec fn spec_limit() -> u64 {
    32
}

}
''',
    "Cargo.lock": "# 夹具不解析依赖\n",
}
ASSUME_SPECIFICATION_FORMS_CELL = ("verus!：assume_specification 带属性带 pub、不带 pub 带参数表与 ensures、带泛型参数表三种写法各切成一个单元"
                                   "（种类、名字、起止、按点名路径末段的名字找得到），dump 一条拒算都没有，后面的 spec fn 照旧自成单元")
ASSUME_SPECIFICATION_KEY_CELL = "verus!：点到常量的入口按名字连上它的 assume_specification，改它交回的值形式化键变"


def assume_specification_forms_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, ASSUME_SPECIFICATION_FIXTURE)
    unit_lines, refusal_lines, _counts = dump_lines(base, "rust")
    units = [unit for unit in RustWorkspace(base).files[ASSUME_SPECIFICATION_FILE].units]
    got = [(unit.short, unit.first_line, unit.last_line, sorted(rust_declared_names(unit)), unit.public)
           for unit in units if unit.kind == "assume_specification"]
    want = [("assume_specification@14", 13, 14, ["LIMIT_IN_BYTES"], True), ("assume_specification@16", 16, 17, ["width_of"], False),
            ("assume_specification@19", 19, 19, ["len"], True)]
    dumped = [line.split("\t")[0] for line in unit_lines if line.startswith("assume_specification\t")]
    spec_function = [(unit.first_line, unit.last_line) for unit in units if unit.kind == "fn" and unit.short == "spec_limit"]
    ok = not refusal_lines and got == want and len(dumped) == len(want) and spec_function == [(21, 23)]
    return ok, f"拒算 {refusal_lines}；assume_specification 单元 {got}（要 {want}）；dump 里 {len(dumped)} 行；spec_limit 起止 {spec_function}"


# 只在 dev 依赖里开的特性（crates/singlefs-unit-graph 的 resolve_features：入口 crate 照目标种类顺 dev 依赖，别的 crate 只顺普通依赖）：
# fbin 的普通依赖不开 counters、dev 依赖开；fhelper 同样只在 dev 依赖里开，fvia 只经普通依赖链到 fhelper
FEATURE_SECTIONS_FIXTURE = {
    "crates/fcore/Cargo.toml": '[package]\nname = "fcore"\n\n[features]\ncounters = []\n',
    "crates/fcore/src/lib.rs": '''pub fn count() -> u32 {
    counted_value()
}
#[cfg(feature = "counters")]
fn counted_value() -> u32 {
    1
}
#[cfg(not(feature = "counters"))]
fn counted_value() -> u32 {
    0
}
''',
    "crates/fbin/Cargo.toml": ('[package]\nname = "fbin"\n\n[dependencies]\nfcore = { path = "../fcore" }\n\n'
                               '[dev-dependencies]\nfcore = { path = "../fcore", features = ["counters"] }\n'),
    "crates/fbin/src/main.rs": "fn main() {\n    let _ = fcore::count();\n}\n",
    "crates/fbin/tests/counting.rs": "#[test]\nfn counting_sees_the_counters() {\n    assert_eq!(fcore::count(), 1);\n}\n",
    "crates/fhelper/Cargo.toml": ('[package]\nname = "fhelper"\n\n[dependencies]\nfcore = { path = "../fcore" }\n\n'
                                  '[dev-dependencies]\nfcore = { path = "../fcore", features = ["counters"] }\n'),
    "crates/fhelper/src/lib.rs": "pub fn helped() -> u32 {\n    fcore::count()\n}\n",
    "crates/fvia/Cargo.toml": '[package]\nname = "fvia"\n\n[dependencies]\nfhelper = { path = "../fhelper" }\n',
    "crates/fvia/tests/via.rs": "#[test]\nfn via_the_helper() {\n    let _ = fhelper::helped();\n}\n",
}
FEATURE_SECTIONS_COUNTED_LINE = 4   # crates/fcore/src/lib.rs 里 cfg(feature = "counters") 那一份 counted_value 起的行（单元从属性起）
FEATURE_SECTIONS_UNCOUNTED_LINE = 8   # cfg(not(feature = "counters")) 那一份起的行
FEATURE_SECTIONS_ENTRY_CRATE_CELL = "R-目标：入口 crate 的 dev 依赖里才开的特性，不按 cfg(test) 编的 bin 当它没开，集成测试当它开着"
FEATURE_SECTIONS_LINKED_CRATE_CELL = "R-目标：经普通依赖链进来的 crate，它的 dev 依赖里开的特性哪个目标都不顺"


def counted_value_lines(base, entry):
    """入口闭包里 crates/fcore 的 counted_value 是哪一份（起行）与闭包的 cfg 行里有没有 fcore 的 counters。"""
    closure = closure_of(base, entry)
    lines = sorted(unit.first_line for unit in closure.order if unit.file == "crates/fcore/src/lib.rs" and unit.short == "counted_value")
    return lines, 'feature="counters"@fcore' in closure.cfg_lines


def feature_sections_entry_crate_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, FEATURE_SECTIONS_FIXTURE)
    binary = counted_value_lines(base, "crates/fbin/src/main.rs::main")
    test = counted_value_lines(base, "crates/fbin/tests/counting.rs::counting_sees_the_counters")
    ok = binary == ([FEATURE_SECTIONS_UNCOUNTED_LINE], False) and test == ([FEATURE_SECTIONS_COUNTED_LINE], True)
    return ok, f"bin 入口 {binary}（要 [{FEATURE_SECTIONS_UNCOUNTED_LINE}] 不开）、集成测试入口 {test}（要 [{FEATURE_SECTIONS_COUNTED_LINE}] 开着）"


def feature_sections_linked_crate_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, FEATURE_SECTIONS_FIXTURE)
    via = counted_value_lines(base, "crates/fvia/tests/via.rs::via_the_helper")
    return via == ([FEATURE_SECTIONS_UNCOUNTED_LINE], False), f"经 fhelper 的集成测试入口 {via}（要 [{FEATURE_SECTIONS_UNCOUNTED_LINE}] 不开）"


# verus! 里 } 之后紧跟的 ;（结构体字面量当初值的 const、impl 里的关联 const）：unit-graph 的 verus_item_starts 不拿 ; 当起头，; 归前一个条目、进它的指纹
VERUS_SEMICOLON_FILE = "crates/fsemicolon/src/lib.rs"
VERUS_SEMICOLON_CONSTANT = "pub const ORIGIN: Pair = Pair { left: 0, right: 0 };"
VERUS_SEMICOLON_ASSOCIATED = "pub const UNIT: Pair = Pair { left: 1, right: 1 };"
VERUS_SEMICOLON_FIXTURE = {
    "crates/fsemicolon/Cargo.toml": '[package]\nname = "fsemicolon"\n',
    VERUS_SEMICOLON_FILE: (f"verus! {{\npub struct Pair {{\n    pub left: u64,\n    pub right: u64,\n}}\n{VERUS_SEMICOLON_CONSTANT}\n"
                           f"impl Pair {{\n    {VERUS_SEMICOLON_ASSOCIATED}\n}}\npub fn total(pair: Pair) -> u64 {{\n    pair.left + pair.right\n}}\n}}\n"),
    "Cargo.lock": "# 夹具不解析依赖\n",
}
VERUS_SEMICOLON_CELL = ("verus!：} 之后紧跟的 ; 归前一个条目（结构体字面量当初值的 const 与 impl 里的关联 const），指纹是连 ; 在内的词法单元以 0x1f 连的 sha256，"
                        "后面的条目照旧切开")


def verus_semicolon_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, VERUS_SEMICOLON_FIXTURE)
    unit_lines, refusal_lines, _counts = dump_lines(base, "rust")
    units = RustWorkspace(base).files[VERUS_SEMICOLON_FILE].units
    got = sorted((unit.kind, unit.short, unit.first_line, unit.last_line, unit.fingerprint) for unit in units if unit.kind in ("const", "fn"))
    want = sorted([("const", "ORIGIN", 6, 6, sha256_text("\x1f".join(rust_lex(VERUS_SEMICOLON_CONSTANT)[1]))),
                   ("const", "UNIT", 8, 8, sha256_text("\x1f".join(rust_lex(VERUS_SEMICOLON_ASSOCIATED)[1]))),
                   ("fn", "total", 10, 12, sha256_text("\x1f".join(rust_lex("pub fn total(pair: Pair) -> u64 {\n    pair.left + pair.right\n}")[1])))])
    return not refusal_lines and got == want, f"拒算 {refusal_lines}；const 与 fn 单元 {[item[:4] + (item[4][:12],) for item in got]}（要 {[item[:4] + (item[4][:12],) for item in want]}）"


def unit_graph_parity_cells():
    """与 crates/singlefs-unit-graph 对齐的 assume_specification 切分、dev 依赖特性与 verus! 里 } 之后的 ; 那几格（BREAK_CELLS 里各配弄坏开关）。"""
    return [
        (VERUS_SEMICOLON_CELL, verus_semicolon_cell),
        (ASSUME_SPECIFICATION_FORMS_CELL, assume_specification_forms_cell),
        (ASSUME_SPECIFICATION_KEY_CELL, compare_cell(ASSUME_SPECIFICATION_FIXTURE, f"{ASSUME_SPECIFICATION_FILE}::judge_limit", ASSUME_SPECIFICATION_FILE,
                                                     "returns 32u64;", "returns 33u64;", "changed")),
        (FEATURE_SECTIONS_ENTRY_CRATE_CELL, feature_sections_entry_crate_cell),
        (FEATURE_SECTIONS_LINKED_CRATE_CELL, feature_sections_linked_crate_cell),
    ]


def import_time_cells():
    return [
        (IMPORT_TIME_EXIT_CELL, import_time_exit_cell),
        (IMPORT_TIME_CLASS_CELL, compare_cell(IMPORT_TIME_FIXTURE, IMPORT_TIME_ENTRY, "tool/helper.py", "        self.value = 1\n", "        self.value = 2\n", "same")),
        (REAL_ENTRIES_CELL, real_entries_cell),
        (REAL_JUDGING_CLOSURES_CELL, real_judging_closures_cell),
    ] + [(name, real_import_time_cell(name)) for name, _path, _old, _new in REAL_IMPORT_TIME_EDITS] + [
        (EXCLUDED_EDGE_PYTHON_CELL, excluded_edge_cell(EXCLUDED_EDGE_PYTHON_ENTRY, "tool/hint.py", "tool/kept.py")),
        (EXCLUDED_EDGE_SHELL_CELL, excluded_edge_cell(EXCLUDED_EDGE_SHELL_ENTRY, "tool/hint.sh", "tool/kept.sh")),
        (EXCLUDED_EDGE_EXIT_CELL, excluded_edge_exit_cell),
        (ADMISSION_EDGE_ANOMALY_CELL, admission_edge_anomaly_cell),
        (ADMISSION_EDGE_STRIP_CELL, admission_edge_strip_cell),
        (ADMISSION_EDGE_REACH_CELL, admission_edge_reach_cell),
        (MAIN_GUARD_IMPORT_CELL, main_guard_fixture_cell(
            "tool/user.py::judge",
            (("tool/library.py", "work"), ("tool/library.py", "LIMIT"), ("tool/library.py", "registered"),
             ("tool/fallback.py", "otherwise"), ("tool/fallback.py", "leftover")),
            (("tool/library.py", "self_check"), ("tool/library.py", "<main-guard>")))),
        (MAIN_GUARD_PROGRAM_CELL, main_guard_fixture_cell(
            "tool/launcher.py::launch",
            (("tool/library.py", "self_check"), ("tool/library.py", "work"), ("tool/twice.py", "early"), ("tool/twice.py", "late")), ())),
        (MAIN_GUARD_REAL_CELL, main_guard_real_cell),
        (MAIN_GUARD_ELSE_CELL, main_guard_else_cell),
        (JUDGING_ENTRY_WORKERS_CELL, judging_entry_workers_cell),
    ]


# 产品行为视图（文件头「判定种类」）那几格：Rust 那一份的替身是「照 crates/singlefs-unit-graph 的剥法手工剥过的拷贝（剥掉的换成空格、换行留着）
# 照原样切出的 dump」，不起 cargo、不编；起 unit-graph 的那一条路由一个只认 dump --judgment-kind product-behavior 的假程序走一趟
PRODUCT_VIEW_FILE = "crates/fverus/src/lib.rs"
PRODUCT_VIEW_ENTRY = f"{PRODUCT_VIEW_FILE}::caller"
PRODUCT_VIEW_SOURCE = """verus! {
pub open spec fn doubled(x: u64) -> u64 {
    x * 2
}
proof fn lemma_doubled_is_even(x: u64)
    ensures
        doubled(x) % 2 == 0,
{
    assert(doubled(x) == x * 2);
}
pub fn scaled(x: u64) -> (r: u64)
    requires
        x < 1000,
    ensures
        r == doubled(x),
{
    proof {
        lemma_doubled_is_even(x);
    }
    helper(x) * 2
}
fn helper(x: u64) -> u64 {
    x + 0
}
} // verus!
pub fn caller() -> u64 {
    scaled(3)
}
"""
# 只给 Verus 看的那几段：spec 与 proof 函数整条、具名返回值的「(r:」与「)」、requires 与 ensures、函数体里的 proof 块
PRODUCT_VIEW_GHOST_PIECES = (
    "pub open spec fn doubled(x: u64) -> u64 {\n    x * 2\n}",
    "proof fn lemma_doubled_is_even(x: u64)\n    ensures\n        doubled(x) % 2 == 0,\n{\n    assert(doubled(x) == x * 2);\n}",
    "(r: ",
    ")\n    requires\n        x < 1000,\n    ensures\n        r == doubled(x),",
    "proof {\n        lemma_doubled_is_even(x);\n    }",
)
PRODUCT_VIEW_FIXTURE = {
    "crates/fverus/Cargo.toml": '[package]\nname = "fverus"\n',
    PRODUCT_VIEW_FILE: PRODUCT_VIEW_SOURCE,
    "crates/fplain/Cargo.toml": '[package]\nname = "fplain"\n',
    "crates/fplain/src/lib.rs": "pub fn plain() -> u32 {\n    7\n}\n",
}
PRODUCT_VIEW_LEMMA_CELL = "产品行为视图：引理里加一句 assert(true);，产品行为键不变、形式化键变"
PRODUCT_VIEW_ENSURES_CELL = "产品行为视图：verus! 里执行函数的 ensures 改了，产品行为键不变"
PRODUCT_VIEW_EXECUTABLE_CELL = "产品行为视图：verus! 里被调到的执行代码改了，产品行为键变"


def product_view_files(edit=None):
    """→ (原样那一份的文件表, 剥过的那一份)：edit 是 (旧串, 新串)，两份照改（落在被剥那几段里的，剥过的那一份照样剥掉）。"""
    source, pieces = PRODUCT_VIEW_SOURCE, list(PRODUCT_VIEW_GHOST_PIECES)
    if edit is not None:
        old, new = edit
        if source.count(old) != 1:
            raise AssertionError(f"产品行为视图样本里「{old.strip()}」命中 {source.count(old)} 次（要恰好一次）")
        source = source.replace(old, new)
        pieces = [piece.replace(old, new) for piece in pieces]
    blanked = source
    for piece in pieces:
        if blanked.count(piece) != 1:
            raise AssertionError(f"产品行为视图样本里要剥的「{piece[:40]}」命中 {blanked.count(piece)} 次（要恰好一次）")
        blanked = blanked.replace(piece, "".join(character if character == "\n" else " " for character in piece))
    return dict(PRODUCT_VIEW_FIXTURE, **{PRODUCT_VIEW_FILE: source}), dict(PRODUCT_VIEW_FIXTURE, **{PRODUCT_VIEW_FILE: blanked})


def product_view_dump_of(work, blanked_files):
    """Rust 那一份的替身：剥过的那一份照原样切出的 dump 行（单元行与拒算行）。"""
    base = os.path.join(work, "blanked")
    shutil.rmtree(base, ignore_errors=True)
    write_tree(base, blanked_files)
    unit_lines, refusal_lines, _counts = dump_lines(base, "rust")
    return unit_lines + refusal_lines


def product_view_key(work, files, dump, kind=PRODUCT_BEHAVIOR, entry=PRODUCT_VIEW_ENTRY):
    """→ (键或 None, 拒算形态的集合)：dump 是交给库的 Rust 那一份（None 是要用时起 unit-graph）。"""
    base = os.path.join(work, "tree")
    shutil.rmtree(base, ignore_errors=True)
    write_tree(base, files)
    options = Options()
    options.judgment_kind, options.product_dump = kind, dump
    return fixture_key(base, entry, options)


def product_view_lemma_cell(work):
    original, blanked = product_view_files()
    edited, edited_blanked = product_view_files(("    assert(doubled(x) == x * 2);\n", "    assert(true);\n    assert(doubled(x) == x * 2);\n"))
    before, before_forms = product_view_key(work, original, product_view_dump_of(work, blanked))
    after, after_forms = product_view_key(work, edited, product_view_dump_of(work, edited_blanked))
    formal_before, _ = product_view_key(work, original, None, FORMAL_VERIFICATION)
    formal_after, _ = product_view_key(work, edited, None, FORMAL_VERIFICATION)
    ok = before is not None and before == after and formal_before is not None and formal_after is not None and formal_before != formal_after
    return ok, (f"产品行为键改前 {before}、改后 {after}（拒算 {sorted(before_forms | after_forms)}）；形式化键改前 {formal_before}、改后 {formal_after}")


def product_view_compare_cell(edit, want):
    def judge(work):
        original, blanked = product_view_files()
        edited, edited_blanked = product_view_files(edit)
        before, before_forms = product_view_key(work, original, product_view_dump_of(work, blanked))
        after, after_forms = product_view_key(work, edited, product_view_dump_of(work, edited_blanked))
        if before is None or after is None:
            return False, f"拒算了：改前 {sorted(before_forms)}，改后 {sorted(after_forms)}"
        got = "changed" if before != after else "same"
        return got == want, f"期望 {want}，实际 {got}"
    return judge


def product_view_refused_cell(mangle, form):
    """Rust 那一份的 dump 经 mangle 改一处，入口的闭包要拒算 form。"""
    def judge(work):
        original, blanked = product_view_files()
        key, forms = product_view_key(work, original, mangle(product_view_dump_of(work, blanked)))
        return key is None and form in forms, f"要拒算 {form}，实际键 {'有' if key else '无'}、拒算 {sorted(forms)}"
    return judge


def product_view_extra_unit(lines):
    return lines + [f"fn\tfverus::phantom\t{PRODUCT_VIEW_FILE}:30-31\t{'0' * 64}"]


def product_view_outside_unit_missing(lines):
    return [line for line in lines if not line.startswith("fn\tfverus::caller\t")]


def product_view_block_unparsed(lines):
    return lines + [f"refused\t{VERUS_GHOST_CODE_UNPARSED}\t{PRODUCT_VIEW_FILE}:1\tverus! {{"]


def with_unit_graph_directory(directory, action):
    """FUNCTION_UNITS_UNIT_GRAPH_DIRECTORY 临时设成 directory 跑 action，跑完照原样恢复。"""
    previous = os.environ.get(UNIT_GRAPH_DIRECTORY_VARIABLE)
    os.environ[UNIT_GRAPH_DIRECTORY_VARIABLE] = directory
    try:
        return action()
    finally:
        if previous is None:
            os.environ.pop(UNIT_GRAPH_DIRECTORY_VARIABLE, None)
        else:
            os.environ[UNIT_GRAPH_DIRECTORY_VARIABLE] = previous


def product_view_unavailable_cell(work):
    original, _blanked = product_view_files()

    def keys():
        return product_view_key(work, original, None), product_view_key(work, original, None, entry="crates/fplain/src/lib.rs::plain")
    (key, forms), (plain_key, plain_forms) = with_unit_graph_directory(os.path.join(work, "nowhere"), keys)
    return (key is None and "product-view-unavailable" in forms and plain_key is not None,
            f"有 verus! 块那一份的入口：键 {'有' if key else '无'}、拒算 {sorted(forms)}（要拒算 product-view-unavailable）；没有的那一份：键 {plain_key}、拒算 {sorted(plain_forms)}")


def product_view_program_cell(work):
    original, blanked = product_view_files()
    dump = product_view_dump_of(work, blanked)
    expected, _forms = product_view_key(work, original, dump)
    program_directory = os.path.join(work, "bin")
    os.makedirs(program_directory)
    dump_file, call_log = os.path.join(work, "product-view.txt"), os.path.join(work, "calls.txt")
    with open(dump_file, "w", encoding="utf-8") as handle:
        handle.write("".join(line + "\n" for line in dump))
    program = os.path.join(program_directory, "unit-graph")
    with open(program, "w", encoding="utf-8") as handle:
        handle.write(f'#!/bin/bash\n[[ "$1 $3 $4" == "dump --judgment-kind {PRODUCT_BEHAVIOR}" && $# == 4 ]] || exit 2\n'
                     f'printf "%s\\n" "$2" >> {shlex.quote(call_log)}\ncat {shlex.quote(dump_file)}\n')
    os.chmod(program, 0o755)
    got, forms = with_unit_graph_directory(program_directory, lambda: product_view_key(work, original, None))
    formal, _formal_forms = with_unit_graph_directory(program_directory, lambda: product_view_key(work, original, None, FORMAL_VERIFICATION))
    calls = open(call_log, encoding="utf-8").read().split("\n")[:-1] if os.path.isfile(call_log) else []
    listed = dump_lines(os.path.join(work, "tree"), "rust", judgment_kind=PRODUCT_BEHAVIOR, product_dump=dump)[0]
    ghost_listed = [line for line in listed if "doubled" in line.split("\t")[1]]
    ok = got is not None and got == expected and formal is not None and calls == [os.path.join(work, "tree")] and not ghost_listed
    return ok, (f"起假程序算的键 {got}（拒算 {sorted(forms)}），交 dump 算的键 {expected}；假程序被起 {len(calls)} 次 {calls}（只该产品行为那一趟起一次、根是 tree）；"
                f"产品行为视图的 dump 里还列着只给 Verus 看的 {ghost_listed}")


# 只在规格里点到的执行侧条目（HeaderModel 只出现在 ghost 字段的类型、ensures、let ghost、proof 块与循环头里）：unit-graph 剥完再切、再收闭包，
# 闭包里没有它；手工剥过的拷贝照原样切出来的 dump 当 Rust 那一份，照原样收它的闭包当 Rust 那一份的闭包
PRODUCT_VIEW_EDGES_FILE = "crates/fghostedges/src/lib.rs"
PRODUCT_VIEW_EDGES_ENTRY = f"{PRODUCT_VIEW_EDGES_FILE}::caller"
PRODUCT_VIEW_EDGES_SOURCE = """verus! {
pub struct HeaderModel {
    pub width: nat,
}
pub open spec fn spec_header(width: u64) -> HeaderModel {
    HeaderModel { width: width as nat }
}
pub struct Recorded {
    pub width: u64,
    pub history: Ghost<Seq<HeaderModel>>,
}
pub fn parse_header(width: u64) -> (parsed: Recorded)
    ensures
        spec_header(width) == (HeaderModel { width: parsed.width as nat }),
{
    let ghost before = spec_header(width);
    proof {
        assert(before.width == width as nat);
    }
    let mut total = 0u64;
    while total < width
        invariant
            total <= width,
            spec_header(total) == if total == 0 { HeaderModel { width: 0 } } else { spec_header(total) },
        decreases width - total,
    {
        total = total + 1;
    }
    Recorded { width: total, history: Ghost(Seq::empty()) }
}
} // verus!
pub fn caller() -> u64 {
    parse_header(3).width
}
"""
# 照 crates/singlefs-unit-graph 的剥法要剥的那几截：spec 函数整条、ghost 字段连逗号、具名返回值的「(名:」与「)」、ensures、let ghost、proof 块、
# 循环头的 invariant 与 decreases、字面量里列尾的 Ghost(…) 字段连它前面那个逗号
PRODUCT_VIEW_EDGES_GHOST_PIECES = (
    "pub open spec fn spec_header(width: u64) -> HeaderModel {\n    HeaderModel { width: width as nat }\n}",
    "pub history: Ghost<Seq<HeaderModel>>,",
    "(parsed: ",
    ")\n    ensures\n        spec_header(width) == (HeaderModel { width: parsed.width as nat }),",
    "let ghost before = spec_header(width);",
    "proof {\n        assert(before.width == width as nat);\n    }",
    "invariant\n            total <= width,\n            spec_header(total) == if total == 0 { HeaderModel { width: 0 } } else { spec_header(total) },\n"
    "        decreases width - total,",
    ", history: Ghost(Seq::empty())",
)
PRODUCT_VIEW_EDGES_CELL = ("产品行为视图：只在规格里点到的执行侧条目（ghost 字段的类型、ensures、let ghost、proof 块、循环头里点到的结构体）不进闭包，"
                           "闭包与 unit-graph 剥完再收的那一份逐个相同")
PRODUCT_VIEW_UNVERIFIED_CELL = "产品行为视图：这一份剥出来的与 Rust 那一份交的核不上，照原样那一份收（闭包只多不少，规格里点到的结构体照收）"


def product_view_edges_closures(work, extra_pieces=()):
    """→ (产品行为视图下入口的闭包行, Rust 那一份的替身：手工剥过的拷贝照原样收的闭包行)；extra_pieces 是 Rust 那一份另剥的几截（这一份认不出的写法）。"""
    blanked = PRODUCT_VIEW_EDGES_SOURCE
    for piece in PRODUCT_VIEW_EDGES_GHOST_PIECES + tuple(extra_pieces):
        if blanked.count(piece) != 1:
            raise AssertionError(f"要剥的「{piece[:40]}」命中 {blanked.count(piece)} 次（要恰好一次）")
        blanked = blanked.replace(piece, "".join(character if character == "\n" else " " for character in piece))
    fixture = {"crates/fghostedges/Cargo.toml": '[package]\nname = "fghostedges"\n', "Cargo.lock": "# 夹具不解析依赖\n"}
    blanked_base, base = os.path.join(work, "blanked"), os.path.join(work, "tree")
    write_tree(blanked_base, dict(fixture, **{PRODUCT_VIEW_EDGES_FILE: blanked}))
    write_tree(base, dict(fixture, **{PRODUCT_VIEW_EDGES_FILE: PRODUCT_VIEW_EDGES_SOURCE}))
    unit_lines, refusal_lines, _counts = dump_lines(blanked_base, "rust")
    options = Options()
    options.judgment_kind, options.product_dump = PRODUCT_BEHAVIOR, unit_lines + refusal_lines
    product = closure_of(base, PRODUCT_VIEW_EDGES_ENTRY, options)
    expected = closure_of(blanked_base, PRODUCT_VIEW_EDGES_ENTRY)
    return (sorted(unit.dump_text() for unit in product.order), sorted(refusal.form for refusal in product.refusals),
            sorted(unit.dump_text() for unit in expected.order))


def closure_names(lines):
    return sorted(line.split("\t")[1].rsplit("::", 1)[-1] for line in lines)


def product_view_edges_cell(work):
    product, forms, expected = product_view_edges_closures(work)
    ok = product == expected and not forms and "HeaderModel" not in closure_names(product) and "Recorded" in closure_names(product)
    return ok, f"产品行为视图的闭包 {closure_names(product)}（拒算 {forms}）；unit-graph 剥完再收的 {closure_names(expected)}"


def product_view_unverified_cell(work):
    product, forms, expected = product_view_edges_closures(work, ("total = total + 1;",))
    names = closure_names(product)
    ok = not forms and "HeaderModel" in names and set(closure_names(expected)) <= set(names)
    return ok, f"产品行为视图的闭包 {names}（拒算 {forms}，要有 HeaderModel、不拒算）；Rust 那一份剥完再收的 {closure_names(expected)}"


# 外属性 #[verifier::…]：Rust 那一份剥掉它（verus! 里外都剥），打头的属性一剥，单元的起始行往下挪；这一份照同一套词法剥了再切，两边的起止才对得上
PRODUCT_VIEW_ATTRIBUTE_LINE = "#[verifier::external_body]"
PRODUCT_VIEW_ATTRIBUTE_INSIDE_CELL = "产品行为视图：verus! 里的执行函数只多一行 #[verifier::external_body]，产品行为视图配得上、产品行为键不变"
PRODUCT_VIEW_ATTRIBUTE_OUTSIDE_CELL = "产品行为视图：没有 verus! 块的文件里的函数只多一行 #[verifier::external_body]，产品行为视图配得上、产品行为键不变"


def product_view_attribute_cell(inside_verus):
    """函数头上只多一行 PRODUCT_VIEW_ATTRIBUTE_LINE：Rust 那一份的替身是把这一行换成空格的拷贝照原样切出的 dump；产品行为键改动前后相同且都不拒算，
    形式化键变（属性进形式化的键）。inside_verus 为真加在 verus! 里被调到的 helper 上，否则加在没有 verus! 块的 crate 的 plain 上。"""
    def judge(work):
        original, blanked = product_view_files()
        file, head, entry = ((PRODUCT_VIEW_FILE, "fn helper(x: u64) -> u64 {\n", PRODUCT_VIEW_ENTRY) if inside_verus
                             else ("crates/fplain/src/lib.rs", "pub fn plain() -> u32 {\n", "crates/fplain/src/lib.rs::plain"))
        if original[file].count(head) != 1 or blanked[file].count(head) != 1:
            raise AssertionError(f"产品行为视图样本里「{head.strip()}」命中 {original[file].count(head)} 次（要恰好一次）")
        edited = dict(original, **{file: original[file].replace(head, f"{PRODUCT_VIEW_ATTRIBUTE_LINE}\n{head}")})
        edited_blanked = dict(blanked, **{file: blanked[file].replace(head, f"{' ' * len(PRODUCT_VIEW_ATTRIBUTE_LINE)}\n{head}")})
        before, before_forms = product_view_key(work, original, product_view_dump_of(work, blanked), entry=entry)
        after, after_forms = product_view_key(work, edited, product_view_dump_of(work, edited_blanked), entry=entry)
        formal_before, _ = product_view_key(work, original, None, FORMAL_VERIFICATION, entry)
        formal_after, _ = product_view_key(work, edited, None, FORMAL_VERIFICATION, entry)
        ok = before is not None and before == after and formal_before is not None and formal_after is not None and formal_before != formal_after
        return ok, (f"产品行为键改前 {before}、改后 {after}（拒算 {sorted(before_forms | after_forms)}，要不拒算、键相同）；"
                    f"形式化键改前 {formal_before}、改后 {formal_after}（要变）")
    return judge


def product_view_cells():
    return [
        (PRODUCT_VIEW_ATTRIBUTE_INSIDE_CELL, product_view_attribute_cell(True)),
        (PRODUCT_VIEW_ATTRIBUTE_OUTSIDE_CELL, product_view_attribute_cell(False)),
        (PRODUCT_VIEW_EDGES_CELL, product_view_edges_cell),
        (PRODUCT_VIEW_UNVERIFIED_CELL, product_view_unverified_cell),
        (PRODUCT_VIEW_LEMMA_CELL, product_view_lemma_cell),
        (PRODUCT_VIEW_ENSURES_CELL, product_view_compare_cell(("        r == doubled(x),", "        r == doubled(x) + 0,"), "same")),
        (PRODUCT_VIEW_EXECUTABLE_CELL, product_view_compare_cell(("    x + 0\n", "    x + 1\n"), "changed")),
        ("产品行为视图：Rust 那一份交的单元这一份没切出，拒算 product-view-unmatched", product_view_refused_cell(product_view_extra_unit, "product-view-unmatched")),
        ("产品行为视图：verus! 块外的单元 Rust 那一份没交，拒算 product-view-unmatched", product_view_refused_cell(product_view_outside_unit_missing,
                                                                                                "product-view-unmatched")),
        ("产品行为视图：Rust 那一份解不开 verus! 那一块，链接到它的闭包拒算 verus-ghost-code-unparsed",
         product_view_refused_cell(product_view_block_unparsed, VERUS_GHOST_CODE_UNPARSED)),
        ("产品行为视图：拿不到 Rust 那一份，有 verus! 块的文件拒算 product-view-unavailable，别的文件照算", product_view_unavailable_cell),
        ("产品行为视图：没交 dump 时起 FUNCTION_UNITS_UNIT_GRAPH_DIRECTORY 里的 unit-graph（只起产品行为那一趟），dump 不列只给 Verus 看的单元",
         product_view_program_cell),
    ]


def selftest_cells():
    cells = []
    for name, entry, path, old, new, want in RUST_FIXTURE_CASES:
        cells.append((name, compare_cell(RUST_FIXTURE, f"{RUST_FIXTURE_ENTRY_FILE}::{entry}", path, old, new, want)))
    cells += [
        ("改文档注释里的代码块：文档测试入口作废", rust_doctest_cell),
        ("认不出的 cfg 谓词拒算", refused_cell(RUST_FIXTURE, f"{RUST_FIXTURE_ENTRY_FILE}::judge_speed", "cfg-unrecognized",
                                         ("crates/fextra/src/lib.rs", '#[cfg(feature = "fast")]', "#[cfg(accessible(fast))]"))),
        ("括号配不上的文件整份拒算", rust_unbalanced_cell),
        (MANIFEST_ROOT_CELL, manifest_root_cell),
        (MANIFEST_DIRECTORY_CELL, manifest_directory_cell),
        (MANIFEST_BINDING_CELL, manifest_binding_cell),
        (SHELL_DATA_SOURCE_CELL, shell_data_source_cell),
        (SHELL_DATA_DOT_CELL, shell_data_dot_cell),
        (SHELL_DATA_NESTED_CELL, shell_data_nested_cell),
        (SHELL_DATA_UNFOLDED_CELL, refused_cell(SHELL_DATA_FIXTURE, f"{SHELL_DATA_FILE}::runs_the_unfolded_script", "shell-path-unfolded")),
        (CURRENT_EXE_CELL, current_exe_cell),
        (CURRENT_EXE_CONSTANT_CELL, current_exe_constant_cell),
        (CURRENT_EXE_PATH_ONLY_CELL, current_exe_path_only_cell),
        (CURRENT_EXE_SPAWN_CELL, current_exe_spawn_cell),
        (DIRECTORY_CLOSURE_LINE_CELL, directory_closure_line_cell),
        (RUST_LEX_CACHE_CELL, rust_lex_cache_cell),
        ("WGSL 里没定义、Rust 串里也没有的名字拒算", wgsl_undefined_cell),
        ("dump 的单元行形态（带路径的宏、verus! 两个 fn、impl 头与方法）", rust_dump_cell),
        ("python：只改注释不作废", compare_cell(PYTHON_FIXTURE, "pkg/a.py::entry", "pkg/a.py", "    # 只是注释\n", "    # 只是注释，改了措辞\n", "same")),
        ("python：改文档串不作废", compare_cell(PYTHON_FIXTURE, "pkg/a.py::entry", "pkg/a.py", '"""helper 的文档串"""', '"""helper 的文档串，改了"""', "same")),
        ("python：有单元读 __doc__ 时改文档串作废", compare_cell(PYTHON_FIXTURE, "pkg/a.py::described", "pkg/a.py", '"""helper 的文档串"""',
                                                         '"""helper 的文档串，改了"""', "changed")),
        ("python：getattr 变量名拒算", refused_cell(PYTHON_FIXTURE, "pkg/a.py::dynamic", "getattr-variable")),
        ("python：globals 只查询 __file__ 不拒算", no_refusal_cell(
            {"sample.py": 'def entry():\n    return globals().get("__file__")\n'}, "sample.py::entry")),
        ("python：globals 动态查询仍拒算", refused_cell(
            {"sample.py": 'def entry(name):\n    return globals().get(name)\n'}, "sample.py::entry", "globals-lookup")),
        ("python：globals 查询其他固定名字仍拒算", refused_cell(
            {"sample.py": 'def entry():\n    return globals().get("handler")\n'}, "sample.py::entry", "globals-lookup")),
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
        (ENTRY_UNIT_KEY_CELL, entry_unit_key_cell),
        (ENTRY_LINES_SHAPE_CELL, entry_lines_shape_cell),
        (ENTRY_LINES_RUST_KEYS_CELL, entry_lines_rust_keys_cell),
        ("shell：`bash \"$tree/<仓里的相对路径>\"` 连到仓里那一份", compare_cell(SHELL_FIXTURE, "s/copies.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        (SHELL_PWD_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/pwd_script.sh", "top_child.sh", "echo top", "echo top changed", "changed")),
        (SHELL_QUOTED_BRACE_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/quoted_brace.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        (SHELL_TAIL_GROUP_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/tail_group.sh", "fixtures/one/.sample/driver.sh", "echo driver", "echo driver changed", "changed")),
        (SHELL_TAIL_OUTSIDE_CELL, no_refusal_cell(SHELL_FOLDS_FIXTURE, "s/tail_outside.sh")),
        (SHELL_ARGUMENTS_SKIPPED_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/dispatch_caller.sh", "s/child.sh", "echo child", "echo child changed", "same")),
        (SHELL_ARGUMENTS_TAKEN_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/dispatch_caller.sh", "s/tools/one.py", "return 1", "return 10", "changed")),
        (SHELL_ARGUMENTS_OTHER_CASE_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/dispatch_caller.sh", "s/tools/two.py", "return 2", "return 20", "same")),
        (SHELL_PATTERN_REMOVAL_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/pattern_removal.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        (SHELL_DIRECTORY_COPY_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/directory_copy.sh", "s/tools/one.py", "    return 1", "    return 10", "changed")),
        (SHELL_RELATIVE_COPY_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/relative_copy.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        (SHELL_DIRECTORY_WILDCARD_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/directory_wildcard.sh", "s/tools/two.py", "    return 2", "    return 20", "changed")),
        (SHELL_POSITIONAL_SCRIPT_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/positional_wrapper.sh", "s/tools/one.py", "    return 1", "    return 10", "changed")),
        (SHELL_RELATIVE_CD_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/relative_cd.sh", "s/tools/one.py", "    return 1", "    return 10", "changed")),
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
        (CROSS_RUN_CACHE_CELL, cross_run_cache_cell),
        (CROSS_RUN_CACHE_SCAN_CELL, cross_run_cache_scan_cell),
    ]
    cells += [(name, judge) for name, judge in silent_form_cells()]
    cells += alignment_cells()
    cells += closure_rule_cells()
    cells += closure_edge_cells()
    cells += unit_divergence_cells()
    cells += unit_graph_parity_cells()
    cells += import_time_cells()
    cells += product_view_cells()
    return cells


NESTED_POSITIONALS_CELL = ("shell：bash -c '…' 里套着的里层 bash -c \"…\\$1…\"：外层只换没转义的 $1…$9，转义的 \\$1 留给里层照它自己的参数折"
                           "（里层的 $1 是外层的 $2），连到它 import 的仓内模块")


def silent_form_cells():
    """不许既不连边也不拒算的几种写法：每种一格连边、一格拒算（或「认出、不装仓里的代码」），各配弄坏开关（BREAK_CELLS）。"""
    helper = ("s/py/helper_mod.py", "    return 1", "    return 10", "changed")
    child = ("s/child.sh", "echo child", "echo child changed", "changed")
    return [
        ("shell：python3 -c '<字面串>' 当内联脚本切，sys.argv 折成后面的词，连到它 import 的仓内模块", compare_cell(SHELL_FIXTURE, "s/inline_python.sh", *helper)),
        ("shell：python3 - <<'PY' 喂的 here-doc 当内联脚本切，连到它 import 的仓内模块", compare_cell(SHELL_FIXTURE, "s/heredoc_python.sh", *helper)),
        ("shell：{ python3 /dev/fd/3 …; } 3<<'PY'（重定向写在组后面）照内联脚本切", compare_cell(SHELL_FIXTURE, "s/group.sh", *helper)),
        ("shell：exec python3 -B /dev/fd/3 … 3<<'PY' 里 (目录, 标签) = sys.argv[1:3] 拆包逐个对上，sys.path.insert(0, 目录) 折得出、连到它 import 的仓内模块", compare_cell(SHELL_FIXTURE, "s/argv_slice.sh", *helper)),
        ("shell：内联 python 拆包 sys.argv[1:4] 给两个名字（上界对不上）不对上，照旧拒算 sys-path-unfolded", refused_cell(SHELL_FIXTURE, "s/argv_slice_mismatched.sh", "sys-path-unfolded")),
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
        ("shell：名字[下标]=值、名字[下标]+=值（下标里带空格与展开）照赋值认、不当命令", no_refusal_cell(SHELL_FIXTURE, "s/subscript.sh")),
        ("shell：\"${数组[@]}\" 当命令时按数组的字面赋值摊开，连到它起的脚本", compare_cell(SHELL_FIXTURE, "s/array.sh", *child)),
        ("shell：python3 -c \"$(打印正文的函数)\" 折出函数打的那几行当内联脚本", compare_cell(SHELL_FIXTURE, "s/capture.sh", *helper)),
        (NESTED_POSITIONALS_CELL, compare_cell(SHELL_FIXTURE, "s/nested_inline_positionals.sh", *helper)),
        ("shell：文件头登记的 check 判据（preflight 开跑前跑）照内联 bash 认，连到它起的脚本", compare_cell(SHELL_FIXTURE, "s/declared.sh", *child)),
        ("shell：内联 python exec sys.argv[1:] 时后面那几个词照命令认", compare_cell(SHELL_FIXTURE, "s/argv_exec.sh", *child)),
        ("python：[sys.executable, \"-c\", <字面串>, …] 当内联脚本切，连到它 import 的仓内模块", compare_cell(PYTHON_FIXTURE, "pkg/m.py::run_inline", "pkg/b.py",
                                                                                         "    return 2", "    return 20", "changed")),
        ("python：-c 的串折不出（出处是环境变量）拒算 inline-script-not-literal",
         refused_cell(PYTHON_FIXTURE, "pkg/m.py::run_dynamic_from_environment", INLINE_FORM)),
        ("python：变量解释器折不出（出处是环境变量）拒算 subprocess-command-unfolded",
         refused_cell(PYTHON_FIXTURE, "pkg/m.py::run_unknown_from_environment", COMMAND_FORM)),
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
        (FOLD_NESTED_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::nested_launch", "pkg/c.py", "    return 5", "    return 50", "changed")),
        (FOLD_CONCATENATION_CELL, no_refusal_cell(FOLDS_FIXTURE, "pkg/folds.py::concatenated")),
        (FOLD_SLICE_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::sliced_launch", "pkg/c.py", "    return 5", "    return 50", "changed")),
        (FOLD_VERSIONS_CELL, no_refusal_cell(FOLDS_FIXTURE, "pkg/folds.py::Versions.interpreter")),
        (FOLD_LOADED_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::loaded_constant", "pkg/c.py", "    return 5", "    return 50", "changed")),
        (FOLD_TAIL_GROUP_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::run_from_copy", "pkg/tools/runner.py", "    return 7", "    return 70", "changed")),
        (FOLD_OUTSIDE_SCRIPT_CELL, no_refusal_cell(FOLDS_FIXTURE, "pkg/folds.py::run_generated")),
        (FOLD_CWD_GROUP_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::run_in_copy", "pkg/sample/setup.sh", "echo prepared", "echo prepared again", "changed")),
        (FOLD_SUBMITTED_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::submitted_run", "pkg/c.py", "    return 5", "    return 50", "changed")),
        (FOLD_ENVIRONMENT_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::environment_launch", "pkg/c.py", "    return 5", "    return 50", "changed")),
        (DATA_READ_CHANGED_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::table_rows", "pkg/table.tsv", "alpha", "beta", "changed")),
        (DATA_READ_UNRELATED_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::table_rows", "pkg/other.tsv", "gamma", "delta", "same")),
        (DATA_READ_TAIL_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::rows_from_argument", "pkg/table.tsv", "alpha", "beta", "changed")),
        (DATA_READ_SCRATCH_CELL, no_refusal_cell(FOLDS_FIXTURE, "pkg/folds.py::scratch_rows")),
        (DATA_READ_REFUSED_CELL, refused_cell(FOLDS_FIXTURE, "pkg/folds.py::listed_rows", DATA_FORM)),
        (DATA_READ_INPUT_CELL, no_refusal_cell(FOLDS_FIXTURE, "pkg/folds.py::judge_file")),
        (DATA_READ_PARSED_CELL, no_refusal_cell(FOLDS_FIXTURE, "pkg/folds.py::parsed_rows")),
        (DATA_READ_LISTING_CELL, no_refusal_cell(FOLDS_FIXTURE, "pkg/folds.py::listed_members")),
        (DATA_READ_COPY_CELL, no_refusal_cell(FOLDS_FIXTURE, "pkg/folds.py::copy_listed")),
        (DATA_READ_DIRECTORY_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::template_rows", "pkg/templates/second.txt", "second", "changed", "changed")),
        (DATA_READ_LIBRARY_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::rows_through_library", "pkg/table.tsv", "alpha", "beta", "changed")),
        (DATA_READ_ATTRIBUTE_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::TableReader.lines", "pkg/table.tsv", "alpha", "beta", "changed")),
        (TOOLCHAIN_QUERY_CELL, toolchain_query_cell),
        (CALLER_LAUNCH_CELL, caller_launch_cell),
        (LAUNCH_ARGUMENT_CELL, no_refusal_cell(LAUNCH_ARGUMENT_FIXTURE, "pkg/launch.py::check")),
        (TAIL_ANCHOR_SCRATCH_CELL, compare_cell(TAIL_ANCHOR_FIXTURE, "pkg/manifest_reader.py::scratch_run", "fixtures/sample/Manifest.toml", "sample", "changed", "same")),
        (TAIL_ANCHOR_JUDGE_CELL, compare_cell(TAIL_ANCHOR_FIXTURE, "pkg/manifest_reader.py::judge", "fixtures/sample/Manifest.toml", "sample", "changed", "same")),
        (TAIL_ANCHOR_ARGUMENT_CELL, compare_cell(TAIL_ANCHOR_FIXTURE, "pkg/manifest_reader.py::from_argument", "fixtures/sample/Manifest.toml", "sample",
                                                 "changed", "changed")),
        (LAUNCH_ARGUMENT_SCRIPT_CELL, compare_cell(LAUNCH_ARGUMENT_FIXTURE, "pkg/launch.py::check_script", "pkg/tool.sh", "echo tool", "echo tool again", "changed")),
        (LAUNCH_ARGUMENT_REFUSED_CELL, refused_cell(LAUNCH_ARGUMENT_FIXTURE, "pkg/launch.py::check_unfolded", COMMAND_FORM)),
        (REGISTRY_PROGRAMS_CELL, registry_programs_cell),
        (ROW_KEYED_CELL, row_keyed_cell),
        (PROGRAM_ARGUMENTS_CELL, program_arguments_cell),
    ]


BREAK_CELLS = (
    ("rust-open-entry-traits-ignored", ("开放入口：参数是 dyn", "R-开放：dyn Trait 只带 Trait 自己的 impl，不带只在 trait 参数里写了它的 impl")),
    ("rust-dyn-traits-open-every-implementation", ("dyn 分派：闭包里的 dyn 不连没被点名的另一个 impl，改它不作废",)),
    ("rust-constants-dropped", ("关联常量（经泛型）", "函数指针表", "数组长度里的常量")),
    ("rust-macros-dropped", ("声明宏展开出的函数", "带路径的条目级宏调用定义的枚举：改了作废")),
    ("rust-trait-impl-edges-dropped", ("运算符重载", "Drop 隐式调用", "关联常量（经泛型）")),
    ("rust-verus-segments-ignored", ("verus! 里 ensures 带花括号：下一个函数照样切开，改它作废", "verus! 里够不到的函数改了不作废")),
    ("rust-cfg-ignored", ("cfg(feature) 关着的单元改了不作废", "认不出的 cfg 谓词拒算")),
    ("rust-manifest-directory-reads-unrefused", (MANIFEST_ROOT_CELL,)),
    ("rust-manifest-directories-refused", (MANIFEST_DIRECTORY_CELL,)),
    ("rust-manifest-binding-refused-whole", (MANIFEST_BINDING_CELL,)),
    ("shell-data-sources-unfollowed", (SHELL_DATA_SOURCE_CELL, SHELL_DATA_DOT_CELL, SHELL_DATA_NESTED_CELL, SHELL_DATA_UNFOLDED_CELL)),
    ("shell-data-sources-unsited", (SHELL_DATA_SOURCE_CELL,)),
    ("shell-data-sources-nested-unfollowed", (SHELL_DATA_NESTED_CELL,)),
    ("shell-dot-source-unrecognized", (SHELL_DATA_DOT_CELL,)),
    ("shell-data-source-refusals-dropped", (SHELL_DATA_UNFOLDED_CELL,)),
    ("rust-current-exe-refused", (CURRENT_EXE_CELL,)),
    ("rust-current-exe-children-unlinked", (CURRENT_EXE_CELL,)),
    ("rust-current-exe-constants-same-file-only", (CURRENT_EXE_CONSTANT_CELL,)),
    ("rust-current-exe-path-only-refused", (CURRENT_EXE_PATH_ONLY_CELL,)),
    ("rust-current-exe-spawn-unrefused", (CURRENT_EXE_SPAWN_CELL,)),
    ("closure-directory-lines-span-files", (DIRECTORY_CLOSURE_LINE_CELL,)),
    ("rust-lex-cache-unverified", (RUST_LEX_CACHE_CELL,)),
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
    ("key-entry-units-dropped", (ENTRY_UNIT_KEY_CELL,)),
    ("key-entry-doctest-owner-chained", ("R-结构：文档测试不连所挂的条目与那个模块的 use",)),
    ("unit-graph-key-entry-lines-dropped", (ENTRY_LINES_SHAPE_CELL, ENTRY_LINES_RUST_KEYS_CELL)),
    ("rust-doctest-entry-alone", (ENTRY_LINES_SHAPE_CELL, ENTRY_LINES_RUST_KEYS_CELL)),
    ("cross-run-cache-under-tmpdir", (CROSS_RUN_CACHE_CELL,)),
    ("cross-run-cache-git-directory-only", (CROSS_RUN_CACHE_CELL,)),
    ("cross-run-cache-run-times-old-spelling", (CROSS_RUN_CACHE_CELL,)),
    ("cross-run-cache-mentions-unscanned", (CROSS_RUN_CACHE_SCAN_CELL,)),
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
    ("shell-inline-positionals-escapes-ignored", (NESTED_POSITIONALS_CELL,)),
    ("shell-copied-scripts-ignored", ("shell：自己 cp 出来再起的脚本连到源那一份",)),
    ("shell-enclosing-locals-ignored", ("shell：套在外层函数里的函数看得见外层的 local",)),
    ("shell-cd-first-directory-only", ("shell：cd 过去的目录折出几个，相对路径在每一个底下都连",)),
    ("shell-pwd-unfolded", (SHELL_PWD_CELL,)),
    ("shell-function-bounds-quote-blind", (SHELL_QUOTED_BRACE_CELL,)),
    ("shell-script-tail-groups-ignored", (SHELL_TAIL_GROUP_CELL, SHELL_TAIL_OUTSIDE_CELL)),
    ("shell-program-arguments-unnarrowed", (SHELL_ARGUMENTS_SKIPPED_CELL, SHELL_ARGUMENTS_OTHER_CASE_CELL)),
    ("shell-pattern-removals-unfolded", (SHELL_PATTERN_REMOVAL_CELL,)),
    ("shell-directory-copies-ignored", (SHELL_DIRECTORY_COPY_CELL,)),
    ("shell-relative-tree-copies-ignored", (SHELL_RELATIVE_COPY_CELL,)),
    ("shell-directory-wildcards-ignored", (SHELL_DIRECTORY_WILDCARD_CELL,)),
    ("shell-positional-scripts-refused", (SHELL_POSITIONAL_SCRIPT_CELL,)),
    ("shell-relative-directories-unrooted", (SHELL_RELATIVE_CD_CELL,)),
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
                                "python：-c 的串折不出（出处是环境变量）拒算 inline-script-not-literal")),
    ("shell-later-interpreters-ignored", ("shell：一条命令里第二个解释器（bash 包装脚本 N python3 x.py）照第一个的规则认，连到 x.py",)),
    ("shell-substitutions-ignored", ("shell：起同目录脚本连边（写在命令替换里：x=\"$(bash 同目录/y.sh)\"）",
                                     "shell：< <(命令 \"$x\") 里的命令照命令认，前面那个 < 不吞掉它的第一个词")),
    ("external-program-words-unfiltered", ("shell：die 消息里出现 qemu-system-x86_64 不算外部程序",)),
    ("python-modules-ignored", ("shell：python3 -m <仓内模块> 连到那份模块的程序入口", "python：-m unittest <测试模块> 整份连上（测试方法改了作废）")),
    ("shell-unfolded-commands-allowed", ("shell：命令位置上折不出的程序拒算 shell-command-unfolded",)),
    ("shell-wrapped-commands-ignored", ("shell：包装命令（timeout 60）后面直接执行的脚本连上",)),
    ("shell-command-structure-ignored", ("shell：[[ … || … ]]、(( … ))、case 分支的模式、数组字面量里的词不当命令",
                                         "shell：{ python3 /dev/fd/3 …; } 3<<'PY'（重定向写在组后面）照内联脚本切")),
    ("shell-subscript-assignment-split", ("shell：名字[下标]=值、名字[下标]+=值（下标里带空格与展开）照赋值认、不当命令",)),
    ("shell-array-commands-unfolded", ("shell：\"${数组[@]}\" 当命令时按数组的字面赋值摊开，连到它起的脚本",)),
    ("shell-heredoc-captures-unfolded", ("shell：python3 -c \"$(打印正文的函数)\" 折出函数打的那几行当内联脚本",)),
    ("declared-checks-ignored", ("shell：文件头登记的 check 判据（preflight 开跑前跑）照内联 bash 认，连到它起的脚本",
                                 "python：上游 preflight 跑文件头登记的判据（bash -c declaration.command）不拒算，判据从声明它的脚本连")),
    ("inline-argv-commands-ignored", ("shell：内联 python exec sys.argv[1:] 时后面那几个词照命令认",)),
    ("python-unfolded-heads-allowed", ("python：变量解释器折不出（出处是环境变量）拒算 subprocess-command-unfolded",)),
    ("python-commands-literal-only", ("python：命令是 [解释器] + 参数（按调用处给的列表折）连到那份脚本", "python：认不出的命令（对象的属性）拒算 subprocess-command-unfolded")),
    ("python-path-programs-refused", ("python：shutil.which(…) 与 os.path.join(<折不出的目录>, \"bin/x\") 起的根外程序不拒算",
                                      "python：内联脚本起 sys.argv[k]（取的是起它那份脚本的 \"$1\"）不拒算：程序是调用处给的词")),
    ("python-command-dictionaries-unfolded", ("python：命令里的 *选项[\"键\"] 折回调用处给的字典字面量，连到那份脚本",)),
    ("python-dispatch-calls-ignored", ("python：经分派表 表[键](…) 调到的函数，命令参数按那一处调用折",)),
    ("python-nested-scopes-unfolded", (FOLD_NESTED_CELL,)),
    ("python-command-concatenation-unfolded", (FOLD_CONCATENATION_CELL,)),
    ("python-command-slices-unfolded", (FOLD_SLICE_CELL,)),
    ("python-command-heads-split-ignored", (FOLD_VERSIONS_CELL,)),
    ("python-method-parameters-unfolded", (FOLD_VERSIONS_CELL,)),
    ("python-comprehension-variables-unfolded", (FOLD_VERSIONS_CELL,)),
    ("python-element-calls-unfolded", (FOLD_VERSIONS_CELL,)),
    ("python-sequence-indexes-unfolded", (FOLD_LOADED_CELL,)),
    ("python-loaded-module-constants-unfolded", (FOLD_LOADED_CELL,)),
    ("python-script-tail-groups-ignored", (FOLD_TAIL_GROUP_CELL, FOLD_OUTSIDE_SCRIPT_CELL, FOLD_CWD_GROUP_CELL)),
    ("python-submitted-functions-unfolded", (FOLD_SUBMITTED_CELL,)),
    ("python-environment-assignments-unfolded", (FOLD_ENVIRONMENT_CELL,)),
    ("python-data-reads-ignored", (DATA_READ_CHANGED_CELL, DATA_READ_TAIL_CELL, DATA_READ_REFUSED_CELL, DATA_READ_DIRECTORY_CELL, DATA_READ_LIBRARY_CELL,
                                   DATA_READ_ATTRIBUTE_CELL)),
    ("python-parsed-arguments-unread", (DATA_READ_PARSED_CELL,)),
    ("python-directory-reads-refused", (DATA_READ_DIRECTORY_CELL,)),
    ("python-root-joins-refused", (DATA_READ_LISTING_CELL, DATA_READ_COPY_CELL)),
    ("python-parameter-callers-unresolved", (DATA_READ_LIBRARY_CELL,)),
    ("python-attribute-reads-unfolded", (DATA_READ_ATTRIBUTE_CELL,)),
    ("python-caller-launches-refused", (CALLER_LAUNCH_CELL,)),
    ("python-launch-arguments-unfolded", (LAUNCH_ARGUMENT_CELL, LAUNCH_ARGUMENT_SCRIPT_CELL)),
    ("python-tail-reads-unanchored", (TAIL_ANCHOR_SCRATCH_CELL, TAIL_ANCHOR_JUDGE_CELL)),
    ("python-toolchain-queries-refused", (TOOLCHAIN_QUERY_CELL,)),
    ("python-toolchain-queries-unkeyed", (TOOLCHAIN_QUERY_CELL,)),
    ("python-registry-programs-ignored", (REGISTRY_PROGRAMS_CELL,)),
    ("python-registry-programs-linked", (REGISTRY_PROGRAMS_CELL,)),
    ("python-membership-tables-followed", (PROGRAM_ARGUMENTS_CELL,)),
    ("row-keyed-tables-whole", (ROW_KEYED_CELL,)),
    ("python-program-arguments-unnarrowed", (PROGRAM_ARGUMENTS_CELL,)),
    ("python-import-time-tables-followed", (PROGRAM_ARGUMENTS_CELL,)),
    ("python-slice-unpacking-unpaired", ("shell：exec python3 -B /dev/fd/3 … 3<<'PY' 里 (目录, 标签) = sys.argv[1:3] 拆包逐个对上，sys.path.insert(0, 目录) 折得出、连到它 import 的仓内模块",)),
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
    ("rust-macro-generated-types-unnamed", (MACRO_TYPE_EDGE_CELL,)),
    ("rust-macro-rules-literal-names-unlinked", (MACRO_LITERAL_EDGE_CELL,)),
    ("rust-global-allocators-unlinked", (ALLOCATOR_EDGE_CELL,)),
    ("rust-foreign-self-trait-impls-unlinked", (FOREIGN_IMPL_EDGE_CELL,)),
    ("rust-build-environment-unlinked", (BUILD_ENVIRONMENT_EDGE_CELL,)),
    ("rust-excluded-path-dependencies-dropped", (EXCLUDED_PATH_DEPENDENCY_EDGE_CELL,)),
    ("rust-outside-crate-names-linked-unimported", (OUTSIDE_IMPORT_CELL,)),
    ("rust-outside-crate-names-linked-across-modules", (OUTSIDE_MODULE_CELL,)),
    ("rust-crates-names-linked-across-modules", (CRATES_MODULE_PRIVATE_CELL,)),
    ("rust-cfg-atoms-unnarrowed", ("cfg 成分：闭包没引到 gpu 的入口，带不带 gpu 特性两种目标下键相同",)),
    ("rust-cfg-macro-atoms-ignored", ("cfg 成分：闭包里有 cfg!、语句上的 #[cfg] 或 #[cfg_attr] 引到 gpu 的入口，带不带 gpu 特性键不同",)),
    ("wgsl-defined-name-suffixes-ignored", ("WGSL 外名：条目级宏调用定义的名字以 _ 加外名收尾就有着落",
                                            "WGSL 外名的着落做成边：只靠外名点到的条目级宏调用并进闭包，改它里面的码键变")),
    ("wgsl-prefix-remainders-unchecked", ("WGSL 外名：首段只落在无关的串里、其余那一段哪儿都没有的拒算",)),
    ("wgsl-name-suppliers-not-pulled-in", ("WGSL 外名的着落做成边：只靠外名点到的条目级宏调用并进闭包，改它里面的码键变",)),
    ("wgsl-name-suppliers-include-test-code", (WGSL_TEST_SUPPLIER_CELL,)),
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
    ("rust-assume-specifications-unrecognized", (ASSUME_SPECIFICATION_FORMS_CELL, ASSUME_SPECIFICATION_KEY_CELL)),
    ("rust-assume-specification-names-unlinked", (ASSUME_SPECIFICATION_FORMS_CELL, ASSUME_SPECIFICATION_KEY_CELL)),
    ("rust-dependency-features-from-every-section", (FEATURE_SECTIONS_ENTRY_CRATE_CELL, FEATURE_SECTIONS_LINKED_CRATE_CELL)),
    ("rust-verus-semicolon-split-off", (VERUS_SEMICOLON_CELL,)),
    ("python-import-time-statements-ignored", (IMPORT_TIME_EXIT_CELL,)),
    ("python-def-heads-ignored", (IMPORT_TIME_EXIT_CELL,)),
    ("python-import-time-classes-named", (IMPORT_TIME_CLASS_CELL,)),
    ("python-main-guard-conjunctions-unrecognized", (MAIN_GUARD_IMPORT_CELL, MAIN_GUARD_REAL_CELL)),
    ("python-main-guards-last-only", (MAIN_GUARD_PROGRAM_CELL,)),
    ("python-main-guard-else-dropped", (MAIN_GUARD_ELSE_CELL,)),
    ("program-edge-exclusions-ignored", (EXCLUDED_EDGE_PYTHON_CELL, EXCLUDED_EDGE_SHELL_CELL, ADMISSION_EDGE_REACH_CELL)),
    ("program-edge-exclusions-shell-ignored", (EXCLUDED_EDGE_SHELL_CELL, ADMISSION_EDGE_REACH_CELL)),
    ("excluded-edge-exit-unmeasured", (EXCLUDED_EDGE_EXIT_CELL,)),
    ("admission-edge-admitted-sample", (ADMISSION_EDGE_ANOMALY_CELL,)),
    ("admission-edge-header-unstripped", (ADMISSION_EDGE_STRIP_CELL,)),
    ("admission-edge-reach-unmeasured", (ADMISSION_EDGE_REACH_CELL,)),
    ("judging-entry-literals-unfolded", (REAL_ENTRIES_CELL, REAL_JUDGING_CLOSURES_CELL)),
    ("judging-entry-refusals-ignored", (REAL_JUDGING_CLOSURES_CELL,)),
    ("judging-entry-workers-head-word", (JUDGING_ENTRY_WORKERS_CELL,)),
    ("product-view-kept-formal", (PRODUCT_VIEW_LEMMA_CELL, PRODUCT_VIEW_ENSURES_CELL)),
    ("product-view-strips-executable-code", (PRODUCT_VIEW_EXECUTABLE_CELL,)),
    ("product-view-edges-from-the-written-tokens", (PRODUCT_VIEW_EDGES_CELL,)),
    ("product-view-blanking-unverified", (PRODUCT_VIEW_UNVERIFIED_CELL,)),
    ("product-view-verifier-attributes-kept", (PRODUCT_VIEW_ATTRIBUTE_INSIDE_CELL, PRODUCT_VIEW_ATTRIBUTE_OUTSIDE_CELL)),
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
    os.environ[RUST_LEX_CACHE_DIRECTORY_VARIABLE] = "off"   # 词法单元缓存只在它自己那一格用（那一格临时换一个目录）；别的格都现切，弄坏开关也只波及那一格
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
                 "--rust-strings-entry", "--judgment-kind", "--program-argument")


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
            elif argument == "--judgment-kind":
                options.judgment_kind = value
            elif argument == "--program-argument":
                options.program_arguments = (options.program_arguments or []) + [value]
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
    if options.judgment_kind not in JUDGMENT_KINDS:
        raise UsageError(f"--judgment-kind 只认 {'、'.join(JUDGMENT_KINDS)}：{options.judgment_kind}")
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
            unit_lines, refusal_lines, counts = dump_lines(root, options.language, options.under or None, options.judgment_kind)
            for line in unit_lines + refusal_lines:
                print(line)
            summary = "、".join(f"{language} {count}" for language, count in sorted(counts.items()))
            print(f"  ✓ 切出 {len(unit_lines)} 个单元（{summary}），拒算 {len(refusal_lines)} 处，{time.monotonic() - started:.1f} 秒", file=sys.stderr)
            return 0
        if command in ("closure", "key") and len(positional) == 2:
            root, entry = positional
            closure = closure_of(root, entry, options)
            if command == "closure":
                for line in closure_unit_lines(closure):
                    print(line)
                refusals = closure.refusal_list()
                for refusal in refusals:
                    print(refusal.text())
                if options.explain:
                    for line in explain_lines(closure):
                        print(line)
                return REFUSED_EXIT if refusals else 0
            # 给了 --toolchain 才算把工具链行折进了键（文件头「版本询问」）
            key, refusals = closure_key(closure, options.toolchain, options.dependencies, options.externals)
            if key is None:
                for refusal in refusals:
                    print(refusal.text())
                return REFUSED_EXIT
            print(f"key\t{key}")
            for line in closure_unit_lines(closure):
                print(line)
            return 0
        if command == "controls" and len(positional) == 1:
            return run_controls(positional[0])
        if command == "cross-run-caches" and not positional:
            return print_cross_run_cache_rows()
        if command == "entries" and len(positional) == 1:
            entries = registered_judging_entries(positional[0])
            for entry in entries:
                print(entry)
            print(f"  ✓ 从登记现取 {len(entries)} 个判法入口", file=sys.stderr)
            return 0
        raise UsageError(f"认不出的用法：{' '.join(arguments)}")
    except UsageError as error:
        print(f"  ✗ 用法错：{error}", file=sys.stderr)
        print("    → 怎么办：照文件头「用法」一段给参数（dump <根> | closure <根> <入口> | key <根> <入口> | controls <根> | entries <根> | cross-run-caches | --selftest）",
              file=sys.stderr)
        return 2
    except (FunctionUnitsError, OSError) as error:
        print(f"  ✗ 算不出：{error}", file=sys.stderr)
        print("    → 怎么办：看上一句点的是哪一样（根、bash、Cargo.toml、读不了的文件），补齐再跑；调用方按要跑处置，不当没变", file=sys.stderr)
        return 3


BREAK_VARIABLES = ("FUNCTION_UNITS_BREAK",)


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
    放行：命令行以 --selftest 起头；祖先里有自证（inside_selftest）。
    都不是而 BREAK_VARIABLES 里有设了的：点名变量、说它只在自证里认，交回退出码 2；否则交回 None。"""
    setting = [name for name in BREAK_VARIABLES if os.environ.get(name)]
    if not setting or arguments[:1] == ["--selftest"] or inside_selftest():
        return None
    print(f"  ✗ {'、'.join(setting)} 是弄坏开关，只在自证（--selftest 本身与它起的子进程）里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一项，跑 {setting[0]}=<项> python3 scripts/gate/lib_function_units.py --selftest", file=sys.stderr)
    return 2


if __name__ == '__main__':
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
