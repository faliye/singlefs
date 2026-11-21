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
  原样片段是那一行原文，空白折成一个空格，至多 160 个字符。closure --explain 另打 via\t<文件>:<限定名>\t<从哪个单元来>\t<规则>，只当诊断看；
  按第一列选行的表脚本每被起一次另打 table-launch\t<脚本>\t<whole|rows|no-table>\t<起法的词>\t<从哪个单元起>（whole 是这一次连整张表：整份起、source、
  起法折不出或只带选项而表照定义；rows 是只连点名的实验与行项；no-table 是照词剪过之后表的赋值行不剩；起法的词空着写 -、入口写 -），同样只当诊断看。
  内联 python 里的拒算：正文出自原文里一份 here-doc 的（直接喂给解释器的在起它的那个单元里找，装进变量再交 -c 的在整份文件里找逐字相同的那一份；
  ShellIndex.inline_body_lines），落在那份正文头一行加正文里的偏移那一行，片段是正文那一行；字面 -c 串、原文里找不到那份正文的落在起它的那一行。
  弄坏开关 inline-refusal-lines-at-launch 下都落在起它的那一行。

种类与限定名：
  Rust   种类名照 crates/singlefs-unit-graph（src/units.rs 的 output_name）：fn（自由 fn 与 impl / trait 里的方法）、impl（impl 头：属性起、到 { 之前）、
         trait（trait 头）、const、static、struct、enum、union、type、macro_rules（macro_rules! 整个定义）、use、item_macro（条目级宏调用）、mod（mod x;）、
         mod_head（模块头：行内模块从外属性起、到 { 与紧跟的内属性 #![…]；文件开头的内属性 #![…] 另一个，限定名 <module attributes>；模块里的单元都连它）、
         extern_block、extern_crate、broadcast_group（verus! 里的 broadcast group）、assume_specification（verus! 里的 assume_specification[ 路径 ] …;
         整条连属性一个单元，按方括号里路径末段的名字找得到它、不按它自己的名字；产品行为视图里剥掉）、doctest（文档注释里的代码块）；
         闭包里的数据单元 data-file（include_str! / include_bytes! 与运行期拼得出字面路径读的文件：限定名是那份文件相对根的路径，起止 1-<行数>，指纹是字节 sha256；
         concat!(include_str!(…), include_str!(…), …) 一个宏调用里的每一处各收一处读，弄坏开关 rust-concatenated-includes-skipped 下 concat! 里的不看）、
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
  shell  function（规范文本里 function <名> () 起、同缩进的 } 止；嵌套的另算一个，限定名 <外层>::<名>，外层的指纹里留着它、外层的命令不含它的；here-doc 正文跳过不切）、
         script-body（规范文本去掉顶层函数，限定名 <script-body>）、shell-variable（顶层赋值语句，按变量名各一单元）、
         kept-comments（数据单元：程序读的注释「# <词>:」与 # shellcheck，限定名 <kept-comments>）。规范文本 = bash -n 过了且没有告警、
         整份包进一个函数定义 eval（不执行）、取 declare -f；指纹 = 单元规范文本的 sha256。起止行是回到原文按函数头找的，只当索引。

Python 闭包：import 时执行的模块级语句（欠账 C890（判法单元闭包少收 import 时执行的模块级语句））：
  一个模块有单元进闭包（ScriptClosure.module_entered），它就被载入过：它的模块级 import、assign、类头本身与 def-head 一起进闭包（规则名 import-time）；
  只因载入收进来的模块级 import（IMPORT_TIME_BINDING_VIEW）只连被 import 的那份模块的 module-body（那份模块载入时执行的几样随它进来）：
  from x import f 绑的 f 不在载入时执行，有单元读到 f（按名字、经属性、载入方见过的属性名）才连到 f 的单元。拆成入口加同目录模块、入口逐个再导出的，
  入口被载入不把再导出的每个函数都带进闭包。弄坏开关 python-import-time-bindings-followed 下载入时就连到绑的那个名字的单元。
  类头走 import-time-class：不算被点名、不连方法，之后被别的规则点到再照被点名的类连方法。其余模块级语句在 module-body 里，本来就连。
  内联 python（here-doc、-c）照样：这几样穿过去连边，自己不进闭包。不收的：只带注解、没装饰器也没缺省值的 def 头（注解在 import 时也求值）；main-guard 不在 import 时执行。
  main-guard 认两种写法（is_main_guard）：if __name__ == "__main__":，与不带 else 的 if __name__ == "__main__" and <别的条件>:（and 从左往右短路，import 时
  后面的条件与块都不执行；scripts/hosts/memory_peaks_lib.py 那一种）；and 写法里 __name__ 那一项不在最左、带 else 的认不出，留在 module-body，import 时照收。
  头一种带 else（或 elif）的：if 那一支是 main-guard，else 那一支在 import 时执行，照别的模块级语句进 module-body。
  被 import、被 spec_from_file_location 载入的模块只连 module-body，不连 main-guard（欠账 C947：不然被 import 的模块的自证分支连进判法闭包）；
  按路径载入的模块只连载入方调到的属性（PythonModule.loaded_value_contained、ScriptClosure.attribute_seen）：载入的模块值用 <值>.属性 调它，
    这一处认不出是哪个模块，照属性名连载入的模块里的同名单元。模块值出不了载入方那份模块的（模块名折得出、不是 "__main__"，值只在本模块里取属性、
    放进本模块的名字与容器，规矩逐条在 loaded_value_contained 的文档串里），只照载入方自己的单元上见过的属性名连：载入方只调 read_table 的，
    被载入那份的 main（连同它连到的自证）不因为闭包里别处有 x.main() 跟着进来。出得去的（交给调用方、登进 sys.modules、普通 .py 里放在模块级的）
    照整个闭包见过的属性名连。弄坏开关 loaded-attributes-closure-wide 下一律照整个闭包连。
  这份文件被当程序起（子进程边、runpy.run_path、内联脚本、只写 <文件> 的入口）时连 main-guard；块里按 sys.argv 分派的分支只在给了参数时剪（下面「程序入口带参数」），
  没给参数的照旧整份收。
  一份文件里几段 main-guard 合成一个单元（被当程序起时每一段都执行）：节点是那几段，起行取头一段、止行取末一段，只有一段时指纹照旧取那一段。
函数里读局部量的名字（local_name_reads；弄坏开关 python-local-names-resolved 下照旧按名字连）：def 与 lambda 的参数、函数体里赋值、for、with、except 绑的名字、套在里面的 def 与 class 的名字、
  推导式的循环变量，在那一层作用域与套在里面的作用域里读到的是局部量，不连模块里的同名单元（def f(main=False) 里的 main、for main in mains 里的 main 不连 def main）。
  global、nonlocal 声明过的名字不算局部（照按名字连）；函数里 import 绑的名字由 import 那一条认；默认值与装饰器在外层求值，不算；类体里赋的名字不算（宁可多连）。
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
  入口是 shell 文件的 script-body（只写 <文件>）：这组参数全是字面的已知词，脚本体照下面「shell 起法带字面参数」剪（ShellIndex.narrowed_script_body：
    头一处 shift、set --、循环之前的 $1 一类代成参数，代过之后判得出真假的 if [[ … ]]、case 只收走到的那一支）；判不出真假的分支两支都收，
    一处都折不出的整份收（只多收不少收）。按第一列选行的表照同一条记这组词（table_launched）：有不带 - 起头的词（实验号）或 --item <行项> 的只连这一趟跑得到的那几行（下面「shell 起法带字面参数」），
    只带选项的（--selftest、--list-items）照整张表连；表只在闭包里有单元读它时才连（剪过的脚本体读不到表、别处也不读的，表与表行里的驱动都不连）。
    照这组词剪过的脚本体里表的赋值行一行都不剩的（走到的那一支以 exit 收尾，replay.sh --selftest 那一种），这次起法不记（ShellIndex.launch_defines_tables）：
    这个进程不定义表，闭包里别的单元读表只照别的起法连（自证里 bash replay.sh E1 那几次）；弄坏开关 shell-table-launch-without-assignment-counted 下照旧记、连整张表。
    弄坏开关 shell-entry-arguments-unnarrowed 下 shell 入口照旧整份收；
    shell-entry-undecided-branches-dropped 下入口那一份判不出真假的 if [[ … ]] 当走不到。
  没有 main-guard 的 python 文件与写 <文件>::<限定名> 的入口照旧整份收（参数不用）。
运行期读的数据文件（python；PythonModule.read_target / data_read_origins，ScriptClosure.python_data_reads；弄坏开关 python-data-reads-ignored 下回到既不收也不拒算）：
  读文件的写法：open / io.open / codecs.open（模式里没有 w a x +）、<路径>.read_text() / .read_bytes()、Path(…).open()、shutil.copy / copy2 / copyfile 的源。
  被读的文件分两类，再加一种拒算；每一处读都落进其中一种（判据逐条写在后面）：
  一、判法自己的数据：收进这一份闭包的键（数据单元，规则名 data-read）。判据：路径推得出落在根里的固定位置——
    整个折得出（fold，连同「<折不出的目录>/<仓里的相对路径>」那一种 tree_copy_file）→ 根里的文件收成 data-file（限定名是那份文件相对根的路径，
      指纹是字节 sha256，同 Rust 的 data-file）；根外的、盘上没有的跳过；
      tree_copy_file 那一种（折不出的目录在前）登在 TAIL_READ_SCOPES 里的，同样只留登记前缀之下的（下面「按尾巴读的登记」），整个折得出的不看登记；
    只认得出字面尾巴（path_tails：os.path.join(<折不出>, "pkg", "table.tsv")、f"{目录}/notes.txt"）→ 仓里以这一截收尾的文件整组收；
      一份都没有的不是仓里的文件（这一趟自己写的、根外的），跳过；尾巴只剩后缀（".json"）的不算认得出；
      os.path.join / Path(…) / a / b 拼出、折不出的前缀一点尾巴都没给的按整段路径比（path_tails 交的尾巴带前导 /：os.path.join(<折不出>, "b.tsv")
      收 data/b.tsv、不收 data/remove_stub.tsv），a + b 与 f 串拼的照字符串后缀比；弄坏开关 python-joined-tails-unanchored 下 join 拼的也照字符串后缀比；
      那一处登在 TAIL_READ_SCOPES 里的，那一组只留登记前缀之下的（下面「按尾巴读的登记」）；
      折不出的前缀推得到本函数的一个参数的（PythonModule.prefix_parameter：os.path.join(<参数>, …)、os.path.dirname 一类、本函数里给它的赋值），
      那一组留到收完闭包再按调用处认（ScriptClosure.tail_read_supplied）：闭包里每一处给的都是这一趟自己建的临时目录、判法入口的参数（被判的输入），
      或又是这样的参数，就不收（归二）；有一处给的是 sys.argv、环境变量、根里的路径或推不出的（仓副本、根本身都可能），照尾巴整组收。
      弄坏开关 python-tail-reads-unanchored 下读的时候就整组收；
      门禁样本档（.claude/gate.d/fixtures/ 底下）的文件不按尾巴收：样本只在 run_sample 一类拷走之后读，拷的那一趟按样本自己的键管，仓里多一份样本不动别的判定的键
      （弄坏开关 python-tail-reads-samples-included 下照收）；按调用处认的时候，程序入口带参数剪掉的那几截（narrowing_plan）里的调用不算调用处（弄坏开关 python-call-sites-whole-view 下照算）；
    os.path.join(<折得出、是根里一个目录（不是根本身）>, <折不出、从那个目录的清单里挑的>…)（fixed_directory_read）→ 那个目录整棵收成 data-directory
      （指纹同 directory_fingerprint；那一截沿名字的赋值、for 与推导式的迭代对象往回找碰到一处 os.listdir / os.scandir / os.walk 列的就是这个目录，
      directory_listing_derived：读的是清单里挑出来的某一份，清单本身就读了整个目录）；迭代对象是调仓内模块级函数（本模块里的或 from … import 进来的），
      那个函数每一处 return 的值里都有一处列这个目录的 os.listdir / os.scandir / os.walk 的同样算（returned_listing_of：列的目录是它的参数的，
      照这一处调用给的实参折；for name in source_files(directory) 一类；弄坏开关 python-returned-listings-ignored 下不认）；
    os.path.join(<折得出、是根里一个目录（不是根本身）>, <折不出、是一份按名字模式列的清单里 os.path.basename 取的文件名>)（pattern_listed_files）
      → 目录里名字合模式的那几份普通文件逐份收成 data-file，不收整个目录（读的是 <目录>/<文件名>，文件名合模式；在哪个目录里列的、列了几层都不改这一点）。
      清单（listing_collection）：glob.glob / glob.iglob(<折得出的模式>)，模式取最后一段（弄坏开关 python-glob-listings-ignored 下不认）；
      内联 python 收到的参数 sys.argv[k].split("\\n") / .splitlines()，起它的那条 shell 命令交的第 k 个参数是 "$名字"，名字每一处都赋成
      $(find <起点> … -name <模式> … [| sort])（ShellIndex.listing_patterns、find_listing_pattern：-name 恰好一个、写在顶层，别的只许 -maxdepth / -mindepth、
      -type f / l、-print 与只装 -type、-o 的一对括号，都只滤掉行；顶层的 -o、! 一类、管道里接了别的命令的不认；
      弄坏开关 shell-find-listings-ignored 下一律不认，shell-find-listing-operators-unchecked 下只看 -name）；sorted / list / set 一类只改次序的内置调用、
      只有一层生成器的推导式照它们里面的清单认。那一截的名字照最近一层把它当循环变量绑的推导式或 for（for i, x in enumerate(<清单>) 的 x 同样算；
      for 的循环体里又赋值的不认）、只赋过一次的值往回找；拼接、splitext、下标改过的不认，照下面三拒算
      （弄坏开关 python-pattern-listing-transforms-accepted 下改过的也当清单里的文件名）；
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
  三、拒算 data-path-unfolded（落在读它的那一行）：os.path.join(<折得出、是根里一个目录（不是根本身）>, <折不出、出处推不出、不是从那个目录的清单里挑的>…)
    （也不是一份按名字模式列的清单里取的文件名）读的是那个目录里哪几份折不出，不按整目录收（弄坏开关 python-unlisted-directory-reads-whole 下照旧整棵收成 data-directory）；
    os.path.join(<折得出、就是根本身>, <折不出的一截>)，那一截是读根里的文件得来的
    （PythonModule.repository_content_derived：沿名字的赋值、for 与推导式的迭代对象、with … as 的对象往回找，碰到一处读根里文件或目录的读）——
    读的是根里某一份由判法自己的数据点名的文件（读进来的登记行点名的这一类），整个根当数据不成样子，哪一份又推不出。
    弄坏开关 python-root-joins-refused 下回到「根本身 + 折不出」一律拒算、拷文件的源照拒算。
  编译产物不进键：读的路径折出来、或照字面尾巴对上的文件经过 SKIPPED_DIRECTORIES（target、.git、缓存）里的目录的，不收也不拒算（归二的 skip build-output）：
    编出来的二进制随机器、编译目录变，两棵源码相同的树键要相同。弄坏开关 python-data-reads-follow-build-outputs 下照旧收成 data-file。
    编它的函数登记在 BUILT_TOOL_SOURCES：(文件, 函数名) → 它编的程序的源码目录（函数是 python 函数的限定名或 shell 函数的名字）；那个函数进了闭包，
    源码目录整棵收成 data-directory（规则名 tool-source），产生二进制的源码改了键变。
    起编出来的工具那一处折不出（程序路径取编它那一步的输出）的，登记在 BUILT_TOOL_LAUNCHES：(文件, 起它的单元的限定名, 折不出的词的 ast.unparse，
    比法同「不拒算的程序词」) → 编它的函数（BUILT_TOOL_SOURCES 的键）：这一处不拒算，编它的源码目录照样收成 data-directory（编它的函数进不进闭包都收：
    python 起的 bash -c 只看得见它 source 的那几份，叫不到同一份脚本里 export -f 交下来的编译函数）。
    弄坏开关 built-tool-sources-ignored 下两种都不收；built-tool-launches-refused 下 BUILT_TOOL_LAUNCHES 登记了也照旧拒算。
  shell 闭包读的文件（重定向 <、cat 的参数这类）这一份还不收、也不拒算；读目录（os.listdir / os.walk / glob）同样还不收。
按行进键的登记表（ROW_KEYED_TABLES，row_keyed_closure；弄坏开关 row-keyed-tables-whole 下照旧整份进键）：
  判法读一张登记表、只按其中几行取用的（按被起的那份脚本查它自己那一行、按门禁阶段查样本档那一行），那张表在这一条判定的键里只进那几行。
  ROW_KEYED_TABLES 写：表（相对根）、读整张表交解析结果的函数（读表函数）、只按几行取用的函数（取用函数）与它看的那几行的正则（{entry} 是这一条判定的入口文件）。
  调用方（admission.py 算判法键时带上请求的入口文件）照它换：闭包里调读表函数的每个单元（读表函数自己除外）都是登记的取用函数时，那张表的 data-file 单元
  换成 data-rows 单元（限定名照旧是表的路径，起止 1-<选中的行数>，指纹是那几个正则与选中的行按原次序连起来的 sha256）；有一个调用者不在登记里
  （读整张表、逐行核的）照旧整份。登记的一张：.claude/hooks/selftest-registry.tsv（门禁 code-tooling 的 research-script-selftests 那几条的判法
  scripts/gate/selftest-process-guard.py 经 .claude/hooks/lib_project_code_runs.py 读它）。
闭包的轮数（ScriptClosure.run）：收完一轮单元、认一遍读文件的参数与调用方交进来的命令，折出新的根里脚本就再收一轮；超过 CLOSURE_ROUND_LIMIT 轮抛 FunctionUnitsError 点名入口
  （弄坏开关 closure-round-limit-unchecked 下不查）；每轮收完的单元数记在 Closure.round_sizes，closure <根> <入口> --explain 打 round 行。闭包里按名字认的调用处（calls_by_name）随新收的单元增量并进来，
  不每轮重扫（弄坏开关 closure-call-index-frozen 下头一次建完不再并，后收进来的调用处看不见）。
共用一个脚本索引（PythonIndex）的闭包之间共用两样：单元自己的调用处清单（不剪的视图下存在单元的 cache["named-calls"] 里，单元随模块一起建）、运行期读的数据文件与目录的数据单元
  （PythonIndex.data_units、data_directories，同一份文件一趟只读一遍、算一遍摘要）。两样都活在索引里，索引是这一趟的文件内容建的，下一趟（新索引）重读；
  弄坏开关 data-units-process-wide 下数据单元整个进程共用（按根分），同一棵树里文件改了也认旧的。
不走的子进程边（EXCLUDED_PROGRAM_EDGES；同一份判决末节第 3、5 条）：登记了的（起它的文件, 起它的单元的限定名, 被起的程序）那一条子进程边不连
  （python 单元与 shell 单元起的都认；shell 函数的限定名同单元表，嵌套的写 <外层>::<名>）。
  按实测排：那一条边只打提示、不改结局；与 scripts/crash/lib_verdict_library.py 的 NOT_JUDGING_FILES（算键的库自己，按角色排、整份文件不进键）是两种排除。
  登记了一条：E183 跑法脚本计划那一步（plan_step）起 scripts/gate/admission.py 的准入边：准入是闸，退出码只定这一件跑不跑，标准输出只有 E7INPUT 行，
  读产物的判法先 sed 去掉这几行；它不改读数，所以不沿它把准入模块收进实验键。登记一条边之前先现查起点单元与被起的程序都在，不在的不登记。
  自证 EXCLUDED_EDGE_PYTHON_CELL 与 EXCLUDED_EDGE_SHELL_CELL 在夹具上证这套机制：自证期间临时往表里登记夹具里的一条边，核登记了的那一条不连、
  同一个单元起的另一个程序照连、不登记时那一条照连。EXCLUDED_EDGE_EXIT_CELL 在夹具上证「键不变而退出码变」必红。
  登记的准入边配三格（真仓）：ADMISSION_EDGE_ANOMALY_CELL（准入这条路上的异常只让计划那一步不跑、读数出不来）、ADMISSION_EDGE_STRIP_CELL
  （准入打的只有 E7INPUT 行，判法去掉之后读数一行不差）、ADMISSION_EDGE_REACH_CELL（切边前后少掉的单元都只经这条边可达）。
仓根被忽略的私有文件（ScriptClosure.private_root_file；弄坏开关 root-ignored-scripts-linked 下照连）：仓根这一层（相对根的路径不带 /）的文件被仓根那一份 .gitignore 挡着的，
  不当程序连（push_program：python 与 shell 起的都经它）、不当 source 读，不进任何判定的闭包，它们里面的拒算也不算进来。挡着的是多机配置读的私有文件：
  QUIESCE_* 四条指着的 multi-host-quiesce-*.sh（停、复原、回读私有服务，起仓外的脚本）、multi-host.env、qemu.env 与运行日志，清场与配置不改判定结局。
  连进来的那一处是 os.path.join(<根>, <折不出的名字>) 列仓根整层（directory_wildcard，scripts/gate/lib_reuse_checks.py 的 run_sample 那一种），哪一处这样起都会列到它们。
  排在这一层、不排别处的缘由：按文件名形态排，私有脚本换名、新加一份就漏；登记 EXCLUDED_PROGRAM_EDGES 是一份文件、一处起法各一条，换一处起法照样连上；
  按「被 git 忽略」排而不限仓根这一层，会连 .claude/singlefs-ai-sop/ 一起排掉（也被 .gitignore 挡着，下面的 lib.sh、preflight.py 照连，LAUNCHED_REPOSITORY_PROGRAMS 那两行）。
  只读仓根那一份 .gitignore（root_gitignore_rules），不起 git，不读 .git/info/exclude 与用户的 core.excludesFile：挡没挡着只随被判那棵树的内容变；
  落得到仓根这一层的模式只有不带 / 的、开头一个 / 的与 **/ 起头的（git 跟踪着而对得上模式的根级文件这里认不出，照挡着算）。
  根里没有 .gitignore 的（样本树、不带它的拷贝）一份都不挡，照旧连。
  自证 ROOT_IGNORED_SCRIPTS_CELL 在夹具上证：.gitignore 挡着的 multi-host-quiesce-*.sh 与没挡着的另一份都起仓外路径，前者不进闭包、没有落在它上面的拒算，后者进闭包、照拒算 shell-path-outside-root。
不走的调用边（EXCLUDED_CALL_EDGES；ScriptClosure.call_edge_excluded）：登记了的（调用处的文件, 调用处单元的限定名, 被调的名字）那一条按名字连的边不连
  （narrowed_calls 那一路同样认）；被调的单元只经这条边才到得了的，连同它读的数据文件、编的工具源码、起的程序都不进键，别的单元调它照连。
  只登「管怎么算键、不管判对错」的那一步，同按角色排除的文件一个角色、只排这一条边。登记了一条：unit_graph_directory 调 built_unit_graph
  （找或编产品行为视图用的 unit-graph：编它读的 Cargo.lock、.claude/heavy-thread-default、crates/singlefs-unit-graph/src 只定缓存目录名与编法，
  程序交出的单元行逐个进各条判定的键；这三样在别的读点上照那一处的读法收，不按路径排）。BUILT_TOOL_SOURCES 里 build_unit_graph 那一行照留：
  别的路调到它时编它的源码目录照收。弄坏开关 call-edge-exclusions-ignored 下登记了也照连。
  自证 EXCLUDED_CALL_EDGE_CELL 在夹具上证这套机制（自证期间临时登记夹具里的一条边：登记了的那一条不连、只经它读的数据改了键不变，同一个单元调的另一个函数照连）；
  UNIT_GRAPH_INPUTS_CELL 在真仓拷贝上证登记的那一条（unit_graph_directory 的闭包里没有 built_unit_graph、build_unit_graph 与那三样，改三样键不变）；
  UNIT_GRAPH_DIGEST_CELL 证缓存目录名罩全三样（unit_graph_inputs_digest：三样各改一处目录名都换，弄坏开关 unit-graph-digest-skips-lock-file 下改 Cargo.lock 不换）。
按角色排除的文件（Options.excluded_files、ScriptClosure 的 excluded_files；lib_verdict_library.py 的 judging_unit_key、FunctionUnitIndex.key 的 excluded_files 给；
  弄坏开关 excluded-files-followed 下只排单元、边照走）：这几份文件的单元不进闭包，也不从它们往下连：它们读的数据文件、起的程序、调到的别的文件的单元，只经它们才到得了的都不进键；
  别的路到得了的照收。NOT_JUDGING_FILES 是 scripts/crash/lib_verdict_library.py 里同名那一份的抄录（admission.py 经这份库取、不载入那份库），NOT_JUDGING_FILES_CELL 判两份相同。
  变异的两路（广谱、crates）的键里没有判法的代码，不经这里排。
  admission.py 的 KeyContext.script_closure 也给（文件头「单元键」）：入口本身在这几份里的不给（它们自己的判定、function-units-two-implementations-agree 的判法）。
  命令行 closure / key 带 --judging 照判定算键的口径给（judging_excluded_files：按路径载入同目录的 admission.py、调它的 not_judging_files_of_entry，不另抄一份；
  弄坏开关 closure-judging-exclusions-dropped 下带了也不给）：查一条判定为什么拒算、它的闭包里有什么，用这一种（via 链不穿过这几份）；
  不带的什么都不排，看的是这份入口照写法连得到的全部（这几份文件自己的闭包照它）。
测试服务代码不进键（D13（验证路线） 已定项 17「三类代码」表测试服务代码那一行：不进任何键，改它不让它自己的测试重跑）：
  登记表 scripts/gate/test-service-code-paths.tsv（TEST_SERVICE_CODE_TABLE，一行「<仓内路径><制表符><理由>」；以 / 收尾的是前缀，别的是单个文件），
  is_test_service_code(路径) 判一份文件是不是：落在前缀底下、就是登的那份，或是登的 .py、.sh、.rs 的拆分族里的部分文件
  （scripts/gate/lib_lag_split_family.py 的 in_family；.wgsl 只认登的那一份，GPU 判器的内核逐份登）。认法只这一份：别处算键要认测试服务代码的调这一个函数，不另抄；
  crates/singlefs-unit-graph 不读这张表。closure_key 里闭包中这些文件的单元（数据单元同样，按 Unit.file 认）不出 unit 行与 data 行：
  按条复用的测试函数键（admission.py 的 compute_unit_key、门禁 crash-point-replay 那一路）、judge: 的判法闭包键、lib_verdict_library.py 的 FunctionUnitIndex.key
  都经它；闭包照旧经这些单元往下收（它们引到的产品代码与测试代码照进键），entry 行照出：入口自己落在测试服务代码里的，按入口是什么分两种：
  入口是服务代码自己的（服务脚本的 script-body、服务代码里的非测试函数），entry 行里那一个单元只记种类与 <文件>::<限定名>、不记指纹
  （entry_key_lines 的 service_code_unkeyed；改入口的正文键不变，几条这样的判定照旧各是各的键）；入口是测试的照记指纹（改测试自己的正文键变）：
  Rust 的文档测试与 rust_unit_is_test 认的单元（测试目标、基准目标里的，带 #[test] 或连同外层 impl、行内模块与 mod 声明带 cfg(test) 的），连同包它的 impl / trait 头，
  由 rust_closure_of 记进 closure.test_code_entry_units（lib_function_units_rust_closure.py）；python、shell 与 WGSL 的入口都按服务代码自己的算。
  与 unit-graph 对拍的 unit_graph_key 照 Rust 那一份写法，不排。
  四格：TEST_SERVICE_CODE_TABLE_CELL（表与 D13 那一行逐项比：反引号里的每一项都登了、登的每一行都出自那一行、仓里的 crash_judge*.wgsl 逐份登了、
  登的 .wgsl 都在仓里、登的路径都在；弄坏开关 test-service-code-table-row-dropped 下读表少头一行）、TEST_SERVICE_CODE_REAL_CELL（真仓里 crates/singlefs-unit-graph/src 下一份 .rs、
  一份 GPU 判器 .wgsl、登的 .rs 拆出的子模块判是，产品包里的一份与登的 .rs 同目录没登的一份判不是；弄坏开关 test-service-code-split-families-unrecognized）、
  TEST_SERVICE_CODE_UNKEYED_CELL（夹具：改闭包里测试服务代码的一个单元键不变、改产品代码键变；弄坏开关 test-service-code-keyed）、
  TEST_SERVICE_CODE_ENTRY_UNKEYED_CELL（夹具：入口所在的那份也登成测试服务代码，改入口的正文键不变、改产品代码键变，同一份里另一个入口的键不同，
  同一份里 #[cfg(test)] mod tests 的测试函数当入口时改它的正文键变；弄坏开关 test-service-code-entry-keyed、test-service-code-test-entry-unkeyed）。
不拒算的程序词（UNKEYED_PROGRAM_WORDS：(文件, 函数, 词) → 为什么；函数是起它的单元的限定名，脚本体写 <script-body>）：命令位置、-c 的串或脚本路径折不出，
  按（文件, 函数）登记的几张表（这一张与 LAUNCHED_REPOSITORY_PROGRAMS、BUILT_TOOL_LAUNCHES、CALLER_SUPPLIED_COMMANDS、REGISTRY_COMMAND_TEXTS、TAIL_READ_SCOPES）
  函数一格同一种写法（registered_function_name）：python 写限定名（<类>.<方法>、<外层函数>.<locals>.<内层函数>，模块级的就是名字，<module-body>、<main-guard> 照写），
  shell 写函数名（套着的函数写内层的名字）；同一份文件里别的类的同名方法、别的函数里套着的同名函数各登各的，不共用一行（PROGRAM_WORDS_QUALIFIED_CELL）。
  弄坏开关 registered-functions-short-names 下 python 也只认最后一段。
  而它不是这一条判定要收的仓里代码的那几处，逐处登记：编出来的装置（Rust 入口由请求那一侧进键）、冻结根里的装置、
  并行起测试目标的脚本起的 cargo test 与预编的测试二进制（crates 的测试，由调用方那一侧的请求进键）、只给自证换的假程序、调用方交进来的一句话、自证里起的假命令、stage_cell 登记的函数（STAGE_CELL_FUNCTIONS_REAL_CELL 判第二列都是字面名字）、
  表行与样本给的命令（被起的那一份由各条判定的请求进键：入口是它、带 line:command=，样本根里的由 line:samples= 收）、只问环境的系统工具、
  按需领卡的客户端（device_card_lease.py 的 take_card、return_card 起调用方给的 <树根>/task-server/task：领卡没成装置不起，领到了只定装置用哪张卡，
  客户端的标准输出领卡那一回收成卡串、还卡那一回转到标准错误；不改读数）。领卡客户端这两行配三格（真仓，量 device_card_lease.py 这一侧）：
  CARD_LEASE_CLIENT_WORDS_ANOMALY_CELL（领卡那一回的异常抛 CardLeaseRefused、装置不起、标准输出空着，还卡那一回的异常不改装置的退出码与标准输出；
  弄坏开关 card-lease-client-words-granted-sample）、CARD_LEASE_CLIENT_WORDS_STRIP_CELL（还卡那一回客户端打的标准输出转到标准错误，读数一行不差；
  弄坏开关 card-lease-client-words-return-output-kept）、CARD_LEASE_CLIENT_WORDS_REACH_CELL（读源码判 row_driver.py 与 shared_steps.py 领卡、还卡
  都经这两个函数、不经别处起领卡客户端，device_card_lease.py 里起子进程的只有这两个函数；弄坏开关 card-lease-client-words-other-launches-unchecked）。
  shell 的词去掉引号比（ShellIndex.resolve_inline、resolve_command、resolve_invocation），python 的照 ast.unparse 比（折不出的那个元素，或起子进程那一处的头一个参数，
  PythonModule.refuse_launch）。shell 的路径词折得出、却落在根外或指到不在的文件的（{tree} 这类由别的程序换掉的模板），登了同样不拒算 shell-path-outside-root /
  shell-path-missing（ShellIndex.resolve_invocation；弄坏开关 shell-registered-missing-paths-refused 下照旧拒算）。没登记的照旧拒算；弄坏开关 program-words-refused 下登记了也拒算。PROGRAM_WORDS_CELL 在夹具上证这套机制（自证期间临时登记夹具里的词），
  REGISTERED_WORDS_REAL_CELL 判登记的文件、函数、词在真仓都还在（弄坏开关 registered-words-drifted 下多一行真仓没有的）。
  REGISTERED_FUNCTIONS_REAL_CELL 另判按（文件, 函数）登记的几张表每一行的函数定义在登记的那份文件里（只写着这个词不算：拆分把定义挪进拆出的文件、
  调用处还留在入口里的照判；登记照函数此刻定义所在的那一份写；弄坏开关 registered-functions-drifted 下多一行函数已挪走的）。
  shell 脚本里写的函数连同它起的内联 python 正文里的，与那些内联 python exec 进模块对象、正文写在这份文件里的库正文里的（文件头「exec 进模块对象」：
  这类正文算在它写在的那份文件上；tail_read_scope_modules）；弄坏开关 registered-functions-module-object-texts-unread 下不往库正文里找。
  闭包里起子进程写在库正文里的（开关复跑的 TripLauncher.run、按条复用的 admission_run），照库正文写在的那份文件与库正文里的函数名登记。
登记的被起程序（LAUNCHED_REPOSITORY_PROGRAMS：(文件, 函数, 词) → 根里被起的那几份；词的比法同「不拒算的程序词」）：命令或脚本路径折不出、
  而被起的是仓里的代码（自证起这份文件自己的别的子命令、换了假的才不走的那一份仓内脚本、{tree} 这类模板指到的这份脚本自己）的那几处，逐处登记：
  不拒算，登记的那几份照程序入口整份连上（参数不知道，不剪）；登记的文件有一份不在的照旧拒算。python 的脚本路径折得出、根里没有的
  （subprocess-path-missing）同样先照这两张登记表认，没登记的照旧拒算。弄坏开关 launched-programs-refused 下登记了也拒算。
  LAUNCHED_PROGRAMS_CELL 在夹具上证（自证期间临时登记），REGISTERED_WORDS_REAL_CELL 一并判这张表的文件、函数、词与被起的那几份在真仓都还在。
  整条登记过的 bash -c（python 起的，头一个参数登在这两张表里）：正文是字面的也不拆开认，照登记办（inline_from_command；$1… 是带引号的多行串、换不进正文时
  拆开认会拒算）。弄坏开关 python-registered-inline-launches-analyzed 下照旧拆开认。
import 进来的模块加的 sys.path（PythonModule.search_directories）：这份模块在模块级 import 的仓内模块、照 SPEC_PATH_ARGUMENTS 载入的那几份，
  它们往 sys.path 加的目录（传下去的也算）对这份模块的 import 同样生效。弄坏开关 python-imported-sys-path-ignored 下只认这份模块自己加的。
import 它的模块加的 sys.path（PythonModule.importer_directories）：拆成入口加同目录模块的，入口先往 sys.path 插目录、再 import 拆出的模块，
  拆出的模块自己不插：它 import 的仓里有这个名字的模块、自己的搜索目录里又找不到时，照同一个拆分族里 import 它的那几份（模块级与函数里的 import 都算）插的目录解，
  传递地往上找（import 它们的同族模块插的也算）。弄坏开关 python-importer-sys-path-ignored 下不看，照旧拒算 import-unresolved。
  拆分族（PythonModule.same_split_family、split_family_root）：同一个目录里，两份归到同一个入口、或一份是另一份归到的入口；归到哪个入口照
  scripts/gate/lib_lag_split_family.py 的 family_owner（基准取同目录里别的 .py、.sh：入口 <名>.py / <名>.sh 加 <名，「-」换「_」>_<部分>.py，最长前缀取最近一层）。
  拆分前是同一份文件的几份才在一个族里；同目录里 import 一份共用库的别的脚本（lib_process_group.py 的调用方这类）不在：它们交的实参不折进库里函数的参数，
  库里起的程序照「调用方交进来的命令」那一条在闭包里调它的那一处往上认。弄坏开关 python-importer-sites-outside-family 下同目录里 import 它的都算（各调用方交的命令连进每一个经这份库的闭包）。
拆出的模块继承入口给内联 python 设的环境变量（PythonModule.inherited_environment、ScriptClosure.entry_inline_scripts）：拆成 shell 入口加同目录模块的
  （入口 <名>.sh 里 export 了变量、或在命令前设了它，再起内联 python；内联 python import 拆出的 <名，「-」换「_」>_<部分>.py），拆出的模块与内联 python 在同一个进程里，
  它读的 os.environ[…]、os.environ.get(…)、os.getenv(…) 照入口起那几份内联 python 时设的值折（只认正文 import 了这一族里某一份模块的内联 python；
  同名的几份取值合起来，有一份折不出的那个名字照旧折不出）。入口取 split_family_root，不是 .sh 的、不在拆分族里的不认。
  弄坏开关 python-split-part-environment-unread 下不认（拆出的模块里按它取的目录与起的脚本照旧拒算）。
shell 里被 source 的库读别的文件里赋的变量（ShellIndex.assignment_sites、started_scripts）：拆成入口加被 source 的库的，库里的函数读入口赋的变量
  （STAGE_REPOSITORY、ROOT 这类）时，那个赋值词在入口赋它的那个单元里折：词里的 BASH_SOURCE、它再读的变量照入口那一份认（不照读它的库）。
  库里的 $0 是 source 它的那条链最上头的脚本（自己不被别人 source 的那几份；bash 里被 source 的文件看到的 $0 是起头那份脚本），没人 source 的是它自己。
  和它一起被 source 进同一份脚本的几份库共用一个全局命名空间：变量的赋值、$(函数) 的输出与位置参数的调用处，在自己、自己 source 的、source 它的
  这几份里都找不到时，到这几份里找（ShellIndex.siblings）。
  赋成它自己的词（命令前的 NAME="$NAME" cmd 把变量原样交给那条命令的环境、local NAME="$NAME" 拷一份外层的值）不带新值，找赋值时不算这一层赋过，
  接着往外找（self_assigning_word）：库里另一个函数写了 STAGE_REPOSITORY="$STAGE_REPOSITORY" python3 …，读它的函数照旧到入口那一处折。
  source 边与反向边（sourcers_of）每算一趟一起换，下一趟的折叠看得到这一趟的边。
  shell 命令前的赋值只管那一次调用（ShellFacts.prefixed、ShellIndex.reader_scoped）：X=v <函数> … 的 X 只在那一次调用里是 v，
  头一个词是这一族文件里的函数、从它照函数正文里命令位置上的函数名往下走走不到读 X 的那个函数（function_reaches；读它的是脚本体也算走不到）的，这一处不算读它的赋值；
  走得到而 v 是这一趟自己的临时目录底下的路径（temporary_word：开头的 $名字 每一处赋值都是 "$(mktemp …)" 或又是这样的词）的也不算
  （自证里换成临时目录下的假脚本那一类，起的是这一趟自己写出来的东西，同 python 一侧临时目录算调用方交进来的）。头一个词不是函数的照旧算。
  弄坏开关 shell-prefix-assignments-unscoped 下前一种照算，shell-temporary-sites-folded 下后一种照算。
  弄坏开关 shell-sourced-values-folded-in-reader 下别的文件里的赋值也在读它的单元里折；shell-sourced-zero-own-file 下库里的 $0 是库自己；
  shell-sourced-siblings-unread 下不到一起被 source 的那几份里找；shell-self-assignments-counted 下赋成它自己的词照赋值算（盖住入口里那一处，折成空、不连）。
  闭包里一份库的函数按名字叫别的函数、读变量时看得见的文件（ScriptClosure.visible_roots）：它那份文件、这个闭包里 source 了它的那几份，与这个闭包里按名字
  叫到它那份文件里的函数的单元所在的那几份（叫它的那一份又照它自己的这几样往上找），连同这些 source 进来的；入口是脚本里的一个函数、source 库的脚本体
  不在闭包里时，库里的函数调同一个脚本 source 的另一份库里的函数照样连上。弄坏开关 shell-name-reach-context-dropped 下不认按名字叫到的那几份。
内联正文出自别的文件（ShellIndex.inline_body_lines、InlineScript 的 body_files、PythonModule.source_file）：-c 交的、here-string 喂的程序正文是变量折出来的
  （入口里 $(cat <<词 …) 装进变量，被 source 的库里的函数起它），正文在起它的那份文件里找不到时，到它 source 的、source 了它的、和它一起被 source 的那几份里找
  逐字相同的那一份 here-doc：正文里的拒算落在那份文件里正文那一行，这份内联模块也算在那份文件上，按文件查的登记表（SPEC_PATH_ARGUMENTS、TAIL_READ_SCOPES、
  UNKEYED_PROGRAM_WORDS 这几张）照正文写在哪份文件里查。弄坏开关 inline-bodies-launch-file-only 下只在起它的那份文件里找。
拆出的模块里的函数，参数照 import 它的模块里的调用处折（PythonModule.importer_call_sites、importer_method_call_sites）：模块级函数的参数照本模块里调它的
  每一处折，连同同一个拆分族里 import 这份模块的模块里调它的那几处（按 from <这份> import <名> [as 别名]、from <这份> import *、import <这份> 之后 <别名>.<名>(…) 认，
  那份模块又把这个名字交出去的接着往 import 它的那几份找）；类里方法的参数另照拆分族（import 这份模块的同族模块，连同那几份 import 的同族模块）里每一处
  <任何东西>.<方法名>(…) 折。实参照调用处所在的那份模块折。拆分族的认法见「import 它的模块加的 sys.path」那一条。
  弄坏开关 python-importer-call-sites-ignored 下只看本模块里的调用处。
globals 的下标与 update（collect_facts）：globals()[键] 读，键是字面量或折得出的连到那几个名字，折不出的连这份模块的每一个模块级名字（只会多收、不会少收）；
  globals()[键] = 值、del globals()[键]、globals().update(…) 只写不读，不连（值是这个单元里的表达式或调用方交的，各在各的单元里）。都不拒算。
  globals().get(…) 与别的用法（传参、当默认值）照旧拒算 globals-lookup。弄坏开关 python-globals-subscripts-refused 下照旧一律拒算。
按部分文件 exec（PythonModule.compiled_part_exec，call_facts 里认）：模块级（module-body、main-guard 两个单元，连同它们里的 for、if、try、with）调内建的 exec(<码>)，
  实参不是字面串，同一个单元里又有一处内建的 compile(<源码>, <文件名>, "exec")、文件名那一格不是字面串（读出一份部分文件、按它的路径编，再 exec 进这个命名空间）：
  exec 进来的是哪份文件、绑了哪些名字，入口与部分文件之间按裸名字互调连不连，算键库没有一条规则折得出，一律拒算 compiled-part-exec（落在 exec 那一行），不连也不静默跳过。
  不算这一种的：exec 进模块对象的（下面「exec 进模块对象」；那一处的 compile 也不算同一个单元里的那一处）、文件名是字面串的、只 compile 不 exec 的（核语法）、函数里的。
  弄坏开关 python-compiled-part-exec-allowed 下不认（照旧不连也不拒算）。
exec 进模块对象（PythonModule.exec_facts、module_object_exec，ScriptClosure.connect_loaded）：exec(compile(<源码>, <文件名>, "exec"), <名>.__dict__)，exec、compile 是内建的、
  只两个位置实参，<名> 只赋过一次 types.ModuleType(<字面串，不是 "__main__">)（或只赋过一次 <名> = <别的这样的名字>），之后按 <名>.<属性> 用：
  照「载入那一份、绑到这个模块名」收，与按路径载入同一套（上面「按路径载入的模块只连载入方调到的属性」）：模块体与 import 时执行的单元进闭包，按属性调到的单元连上；
  模块对象出不了载入方那份模块的（内联脚本里，绑它的名字只在 <名>.属性、getattr / hasattr、条件、又一次 exec 进它与 <别名> = <名> 里用；
  PythonModule.module_object_contained）只照载入方自己的单元上见过的属性名连，别的照整个闭包见过的属性名连。源码两种：
  open(<路径>…).read() 的，路径折得出、每一份都是根里的文件：载入那份文件（进 loads，单元照常进闭包；读它那一处另照「运行期读的数据文件」收成数据单元）；
  别的照 fold 折（内联脚本的 sys.argv[k] 取起它的那条命令里第 k 个词、os.environ[…] 取那条命令给的环境，词是 $(cat <<'词' … 词) 赋的 shell 变量的取正文），
  折出的每段正文当一份内联脚本载入（sys.argv、环境与工作目录照载入方）：单元只穿过去、自己不进闭包，正文已在交它的那个单元里
  （shell 变量单元、字面串）。源码折不出（调用方交的 "$1"、路径折不出或不在根里）拒算 compiled-part-exec（落在 exec 那一行）。
  正文里的拒算落在哪：载入方是 shell 起的内联 python、正文是那条命令的参数或环境变量交进去的、在原文里找得到逐字相同的一份 here-doc 的
  （找法同「内联正文出自别的文件」，ShellIndex.resolve_inline、heredoc_texts_located），落在那份 here-doc 里正文的那一行，这份正文也算在那份文件上
  （按文件查的登记表照它写在哪份文件里查）；几段拼起来的（printf '%s\n' 打几个变量拼成的）与别的落在这个 exec 那一行。
  弄坏开关 python-module-object-refusal-lines-at-exec 下一律落在 exec 那一行（算在载入方那份文件上），按文件查的登记表也就照载入方那份查
  （REGISTERED_FUNCTIONS_REAL_CELL 在登记的库正文那份文件里找不到 admission_run、TripLauncher.run）。
  正文里模块级函数的参数（PythonModule.module_object_loader_call_sites，fold_parameter 经 function_call_sites）：照正文自己里调它的那几处，
  连同 exec 它的那一份载入方里绑这个模块对象的名字（连别名）上的 <名>.<函数>(…) 那几处折，实参照载入方折；别的载入方载入同一段正文的调用处不算。
  类里方法的参数只照正文自己里的调用处折。弄坏开关 python-module-object-loader-call-sites-ignored 下只看正文自己里的调用处。
  弄坏开关 python-module-object-exec-refused 下不认这一种（照「按部分文件 exec」认：同一个单元里有文件名不是字面串的 compile 就拒算，别的不连也不拒算：
  源码折不出、文件名是字面串的那一处也不拒算），python-module-object-exec-unfolded-allowed 下源码折不出的不拒算（不连也不拒算）。
文档串的读法（PythonModule.collect_facts、reads_docstring、call_facts）：文档串不进单元指纹，被读了才收成 docstring 数据单元（规则名 docstring）。认作读的四种：
  <对象>.__doc__ 与模块里的 __doc__ 取值（ast.Load）、<对象>.__doc__ += …（先读后写）、getattr(<对象>, "__doc__")、inspect.getdoc(<对象>)；
  <对象> 是这份模块里的 def / class 收它的那一段，是 import 进来的模块收那份模块的模块文档串，认不出主人的收这份模块的全部文档串。
  <对象>.__doc__ = … 与 del 只写、不读，不收（模块级给 namedtuple 一类写文档串，不把整份模块的文档串带进闭包）。
  弄坏开关 python-docstring-writes-counted 下写也算读；python-docstring-augmented-assignment-not-read 下只认 ast.Load；
  python-docstring-call-readers-ignored 下 getattr 与 inspect.getdoc 两种不认。
登记表里的命令（REGISTRY_COMMAND_TEXTS：(文件, 函数) → (登记表, 命令那一列的下标)）：bash -c 的串是这个函数从登记表读进来的：登记表收成数据单元，
  那一列的每一条命令当内联 bash 认（PythonModule.registry_command_texts）；表不在交空集合（没有要起的）。弄坏开关 python-registry-command-texts-refused 下照旧拒算 inline-script-not-literal。
载入路径的登记（SPEC_PATH_ARGUMENTS：(文件, 路径表达式的 ast.unparse) → 根里的文件）：spec_from_file_location 的路径折不出（调用方当参数交的、从候选里现挑的），
  而实际载入的就是仓里这一份（PythonModule.registered_spec_path）；登记值可以是几份文件的元组（载入的是其中哪一份折不出：每一份都连，有一份不在整条不认、照旧拒算）。
  弄坏开关 spec-path-arguments-refused 下照旧拒算 spec-path-unfolded；spec-path-tuple-first-only 下元组只连头一份。
按尾巴读的登记（TAIL_READ_SCOPES：(文件, 函数的限定名, 被读路径表达式的 ast.unparse) → (前缀的元组, 为什么)；PythonModule.registered_tail_scope）：
  只认得出字面尾巴的那一处读（「运行期读的数据文件」一）照尾巴收的是全仓同名的文件，前缀推不出（os.walk 的目录变量、往上找清单的那一串）；
  「<折不出的目录>/<至少两段的尾巴>」照 tree_copy_file 收的是根里同一相对路径的那一份（os.path.join(<请求给的根>, "crates", "mutations.tsv")），读的却可能是请求给的那棵树；
  登记了的，那一组只留落在登记前缀之下的（前缀相对根，按整段目录比：crates 不认 crates-old/）；前缀是空元组的，读的是调用方交进来的输入（请求的 cwd 一类），一份都不收、不拒算。
  登记的前缀写那一处真走得到的地方，不写「看着相关」的：前缀外的文件那一处读不到，前缀里的改了键照变；读得到哪一份要等请求才定的才写空元组。
  前缀一格写 "."（TAIL_READ_ROOT_COPY）的是被判那棵树根上的那一份：只留相对根的路径就是那截字面尾巴本身的那一份（尾巴只有一段的就是根级文件，
  os.path.join(<调用方给的根>, "Cargo.toml") 只留根上的 Cargo.toml），别处同名的照旧不收；tree_copy_file 那一种收的本来就是根里同一相对路径的那一份，照留；
  可以与目录前缀并写（("crates", TAIL_READ_ROOT_COPY)），比法在 in_tail_read_scope。前缀推得到本函数参数的，留下的那一份同目录前缀下的一样按调用处认（「运行期读的数据文件」一）。
  读调用方给的根上的根级文件（Cargo.toml、Cargo.lock）登 "."；只有读到的值已由调用方解析之后写进键的（依赖行、工具链行、cfg、crate 起法）才登空元组，
  为什么一格写进键的那一处。弄坏开关 tail-read-scopes-root-ignored 下 "." 当目录前缀比（根上那一份也不留）；
  TAIL_READ_ROOT_SCOPE_CELL 在夹具上证：登 "." 之后根上那一份改了键变、research/ 下多一份与改别处同名的一份键不变，登空元组时改根上那一份键不变。
  弄坏开关 tail-read-scopes-ignored 下登记了也照全仓那一组收（两种都是）；tail-read-scopes-copies-unscoped 下只有 tree_copy_file 那一种不看登记。
  TAIL_READ_SCOPES_CELL 在夹具上证两种（自证期间临时登记），REGISTERED_WORDS_REAL_CELL 判登记的那一处与前缀在真仓都在；
  TAIL_READ_SCOPES_REAL_CELL 在真仓逐行核：登记的函数里有一处表达式的 ast.unparse 与登记的逐字相同，从那一处起 data_read_origins 真问到了这一行的登记、
  拿到的前缀与登记的相同，收进来的文件都在前缀下（"." 只认根上那一份，空元组一份都不收）；弄坏开关 tail-read-scopes-drifted 下多登一行真仓没有的表达式。
  那一处在 shell 脚本起的内联 python 正文里的，登记的文件写那份 shell 脚本（PythonModule.source_file），函数一格写正文里的函数名，模块级的写赋值的名字、
  <module-body> 或 <main-guard>：TAIL_READ_SCOPES_REAL_CELL 到那份脚本各单元起的内联正文里找（ShellIndex.links 的 python-inline，同 dump），
  REGISTERED_WORDS_REAL_CELL 不在源码里找 <module-body>、<main-guard> 这两个名字。弄坏开关 tail-read-scopes-inline-unread 下只认 .py 文件里的函数（那几行判红）。
  TAIL_READS_RECORDS_REAL_CELL 在真仓收两批入口的闭包：登记的判法入口（registered_judging_entries）与 .claude/hooks/selftest-registry.tsv 登记成 selftest 的脚本
  （带 --selftest 当程序入口的参数，selftest_registry_scripts；只收 .py、.sh、.bash）。按尾巴读而没登记的那几处，收的时候把落在 records/、research/ 下的文件记进
  UNSCOPED_TAIL_READS（record_unscoped_tail_read）；读它的函数在某个闭包里、记下的文件也收进了那个闭包的，判红，逐处点名 文件:行、函数、表达式与那几份文件，
  出路是照那一处真走到的地方登记 TAIL_READ_SCOPES。收不成闭包的入口同样判红。先在夹具上证：没登记的一处判得出、临时登记之后不判。
  弄坏开关 tail-reads-under-records-unrecorded 下不记（夹具那一处判不出）；tail-read-scopes-ignored 下登记了照记（登记之后那一趟照判出）。
判法入口的登记（registered_judging_entries；命令行 entries <根>）：存判定的各路判法入口从各份脚本的登记现取，不手抄——admission.py 的 REPLAY_SCRIPT 与
  REPLAY_JUDGING_FUNCTIONS、e165 的 JUDGING_ENTRY、E165 第二段判法模块的 JUDGING_ENTRIES、verus-proof-check.py 的 JUDGING_ENTRIES，
  加门禁阶段（.claude/gate.d/*.sh）里写死的 "judge:<文件>::<函数>" 令牌；自证真仓那几格的入口从它取。变异的两路（广谱、crates）键里没有判法的代码，不登判法入口。
  登记值只认字面量；入口里没有这一句赋值、是从同目录一份模块 from <模块> import … <名字> 再导出的
  （拆成入口加同目录模块），到那一份里接着找（module_literal_location）。形式化判法按 __file__ 取入口文件，它的入口写 JUDGING_ENTRIES 赋值所在的那一份。
  弄坏开关 judging-entry-literals-split-unfollowed 下不到再导出的那一份里找（拆过的几份登记都取不到）。自证 REAL_JUDGING_CLOSURES_CELL 逐个收登记的入口的闭包，有一个拒算就红
  （拒算的那一路判定写不进库、每趟照判）；口径照 judging_unit_key：每个入口都去掉 lib_verdict_library.py 的 NOT_JUDGING_FILES。

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
  工作区成员的收法（discover_rust_crates；crates/singlefs-unit-graph 的 workspace.rs 同一条规则，门禁 code-tooling 的 function-units-two-implementations-agree 格对拍 dump）：
          根清单 [workspace] members 逐条按 glob 展开，仓根路径当字面（里面的 [ ] * ? 不是通配；弄坏开关 rust-workspace-members-glob-unescaped 下当通配）；
          成员写 "." 与根目录是同一个（弄坏开关 rust-workspace-dot-member-unnormalized 下当另一个目录收）：根清单有 [package] 时与它是同一个 crate，
          没有 [package]（只有 [workspace] members = ["."]）时收成叫 root 的 crate、有库目标；绝对路径的成员照收；内联空表 workspace = {} 与没有 [workspace] 一样、退回收 crates/ 下各目录；
          两个成员目录同名（crates/common 与 tools/common）时目标号的标签用目录全路径，同名的测试目标不撞（弄坏开关 rust-same-named-member-directories-merged 下只取目录名）。
          嵌套的成员（crates/outer 与 crates/outer/inner）各收各的，文件只归最深的那一个 crate（crate_of_path）；
          members 里写成多行基础串的成员去掉开头那个换行照收，串里有反斜杠的报读不出（FunctionUnitsError；弄坏开关 rust-workspace-multiline-backslash-read 下照 tomllib 读）。
  R-裸名同一个目标里按模块筛  crates/ 下的包与工作区外的 crate 一样，同一个目标里的裸名只连单元所在模块看得见的那几份
          （RustWorkspace.candidates_seen_from_module：同一模块的、所在模块的 use 引进了它且路径（花括号展开之后）恰好点到候选所在模块的、写成 模块::名 的，super 照个数往上数层；
          use 路径或 …::名 的根名是 std、core、alloc 或看得见的别的 crate 的库名的指别的 crate，跳过那一处）；点不准的退回全连，
          字段名、局部变量与 impl 成员自己的名字一份都不连；名字是 use 改名的原名、单元是 macro_rules!（体里的名字在调用处解）的不筛；跨目标的只认 pub、不按模块筛。
          use 的一支路径点准之后再加几道退回全连：点到的候选带条件编译（自己的条件链非空，或它所在文件在这个目标里的连法里有一条带条件；is_compiled_conditionally）、
          或候选所在模块另有 use 按名字引进同名（cfg 互斥的另一支可能是 pub use 转来的）；相对路径（首段不是 crate、self、super）的首段在本作用域被 use 按名字引进
          （含 as 改名、含同一条 use 里别的一支 use {a as b, b::c}、含同一条 use 花括号里的 self：use a::{self, c} 的首段 a；glob 不算），
          或本作用域有条目级宏调用（宏展开出的 use 与 as 改名看不见；任一条目级宏调用都算，不看它展开出什么）。
          use 原名 as 名 那一支：叶子是别名、as 前面不是 段::，路径只到外层花括号，退回全连，不顺着原名点准。
          弄坏开关 rust-use-path-conditional-candidates-narrowed、rust-use-path-re-exporting-modules-narrowed、rust-use-path-first-segment-imported-narrowed、
          rust-use-alias-leaf-followed-to-the-original 各去掉其中一道（各红 USE_PATH_CONDITIONAL_CELL、USE_PATH_RE_EXPORT_CELL、USE_PATH_FIRST_SEGMENT_CELL、USE_PATH_ALIAS_LEAF_CELL）；
          rust-use-path-first-segment-same-use-excluded（首段检查把路径所在的那条 use 自己排掉）红 USE_PATH_SAME_USE_CELL、USE_PATH_SELF_IMPORT_CELL，
          rust-use-path-first-segment-item-macros-ignored（不看本作用域的条目级宏调用）红 USE_PATH_ITEM_MACRO_CELL
          验证包里 CPU 判器的单元经 use 从格式库引进 SLOT_BYTES，不连到 gpu_unit_checks 里私有的同名常量。
          弄坏开关 rust-crates-names-linked-across-modules：crates/ 下的包照旧按同名连
  局部 let 与参数绑定遮住的名字（bare_name_only_a_local_binding；与 crates/singlefs-unit-graph 的 is_bare_name_only_a_local_binding_of_the_function 同一个判法）：
          函数单元里，名字每一处都是局部绑定或落在绑定的作用域里的（语句开头的 let [mut] 名从它的 ; 起到外层花括号收尾，参数从参数表的 ) 起到单元收尾；if let、match、for、闭包参数、解构不认），
          不往外连同名的函数（let main = …; 不连 fn main）；同一目标、跨目标都这样。只遮函数：let 写成常量、静态量、单元结构体的名字是拿它当模式，照连。
          绑定之前或作用域之外的同名、带路径的、宏名、作用域外的格式串捕获、宏调用括号里的 let、函数体里有嵌套条目的，都照连，宁可多连。use 改名的原名不看这一条。
          落在宏调用括号里的名字（宏可能把它接在路径后面或放进嵌套条目，运行时解到同名函数）照连，只有 STANDARD_MACROS_EVALUATING_THEIR_ARGUMENTS_IN_PLACE
          那 16 个标准库宏（格式化、打印、写、panic!、断言一族、vec!）的括号里、在作用域里的照算局部；外层的括号落在名单之外的宏里的，里层也照连。
          弄坏开关 rust-let-shadowed-names-resolved：照旧连到同名函数；rust-macro-arguments-all-local：每个宏的括号里的名字都照局部算

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

程序起自己（RustFile.own_program_path_is_handed_to_command_unchanged；closure.rs 的 entry_is_the_main_of_a_program；crates/singlefs-unit-graph 同写，两道都过才不拒算）：
  子进程起的是在跑的这个程序文件、从同一个 main 进，换参数只换走哪条路，走的代码都已在闭包里，不连边。
  路径形状：current_exe() 后接 ?、.unwrap()、.expect("字面串")、闭包一定不交值（以 panic!(…) 或 …process::exit(…) 收尾）的 .unwrap_or_else(…)，
  或 match current_exe() { Ok(名字) => 同一个名字, Err(…) => <以 return 起头，或一定不交值> }；这个表达式就地写在 Command::new(…) 的括号里，
  或绑给 let 名字（单元里这个名字不再 let），名字的每一处用法最内的那一层花括号就是这条 let 所在的那一层（不进更深的块：if、循环、match 的分支块、闭包体都算更深；也不出这个块；
  圆括号、方括号不算块），只是 Command::new(名字)、Command::new(&名字)、名字.display()，至少起一次。
  「一定不交值」碰到以 } 收尾的表达式：先找这个 } 配的 {，{ 前面一个词是 |（闭包参数）或前两个词是 =、>（=>）才往块里最后一个表达式接着判；别的（else、)、if 的条件、裸块）一律当会交值。
  入口：目标是 bin，这一趟不按测试编，一组入口恰好是目标根文件里排头一个的顶层 fn main（rust_entry_is_the_main_of_a_program；crates/singlefs-unit-graph 的 main_function_of）。
  收闭包时判入口，不判「闭包里有 main」：单测调 main 的进程仍是 libtest。目标根文件里有两个顶层 fn main（cfg 互斥的两份）时，Rust 那一份把两份当一组入口、拒算 current-exe；
  这一份的入口按 <文件>::<限定名> 找、限定名命中两个单元就当场报 UsageError，走不到这个判断（入口解析是两边的差别，不在这一条里改）。
  --exact 折得出名字的先走「测试二进制起自己」，解不到恰好一个 #[test] fn 照旧拒算。形状不认的，单元里有 SPAWNING_WORDS 就就地拒算，没有的只是问路径。
  弄坏开关 rust-own-program-restart-refused、rust-own-program-entry-unchecked、rust-own-program-path-unchecked 各红 OWN_PROGRAM_KEYED_CELL、OWN_PROGRAM_ENTRY_CELL、OWN_PROGRAM_PATH_CELL；
  rust-own-program-uses-any-block、rust-own-program-divergence-any-block 各红 OWN_PROGRAM_BLOCK_CELL、OWN_PROGRAM_DIVERGENCE_CELL。
extern 块（RustFile.extern_block_declares_only_listed_c_library_calls；crates/singlefs-unit-graph 同写，那边的形态名是 extern-function-interface）：
  ABI 恰好是 "C"、条目前的属性里任何位置都没有标识符 link（#[link(…)]、#[cfg_attr(条件, link(…))]；link_name 不算，extern_block_attributes_name_link）、
  块里只有 [#[link_name = "…"]] [pub] fn 名字(…) …;、至少一条、每条的符号（link_name，没有就取函数名）都在
  C_LIBRARY_CALLS_ALLOWED_IN_EXTERN_BLOCKS 里的，切分时不拒算：标准库的文件读写走同一层 C 库，键本来就不跟 C 库变，这两个符号没有多出一类依赖。别的写法拒算 extern-block。
  不拒算的块进闭包时再看链接的最终产物（RustWorkspace.target_may_link_libraries_beyond_the_system_library，参数是入口的目标与块所在的目标）：从入口所在的包与块所在的包出发，
  顺依赖（普通、开发、构建三种，crates_reached_from）够到的全部仓内包里有一个包满足 package_adds_libraries 的一条，这个块照拒算 extern-block（行与片段用这个块的）：
  块带 link：它的任一目标（库、测试、示例、构建脚本各目标）里有 extern 块的外属性带 link（属性里任何位置；不管进不进闭包）；
  构建脚本打指令：它的构建脚本目标里任一份文件有字面串写出 cargo:rustc-link-lib、cargo:rustc-link-arg 或 cargo:rustc-flags（含 cargo:: 写法；rustc-flags 不分 -l 与 -L；
  只认带 cargo: 或 cargo:: 前缀的字面串，注释里提到与裸词不算：单元图自己的源码里有 rustc-link-lib 裸词，验证包把它当构建依赖）；
  库代码打指令：它被够到的某个包当构建依赖时，它的库目标里有同样的字面串（构建依赖的库代码替构建脚本打指令）；Python 一侧认字面串里含这样的写法，比「以它起头」宽；
  外部依赖：它有工作区外的依赖（普通、开发、构建任一种：registry、git 或仓外的 path 来的；cc、pkg-config 这一类的指令字面量在它们里面，看不到，宁可拒算）。
  没有依赖关系的包里写了这些，不影响。registry、git 来的依赖里的 #[link] 看不到，由外部依赖那一条拒。
  不判：工程里自己定义同名导出符号盖掉 C 库（#[no_mangle] fn pwrite）。弄坏开关 rust-extern-listed-calls-refused、rust-extern-any-block-keyed 各红 EXTERN_LISTED_CELL、EXTERN_REFUSED_CELL；
  rust-extern-link-first-attribute-only 红 EXTERN_LINK_ATTRIBUTE_CELL，rust-extern-other-block-link-unchecked（不看块带 link）红 EXTERN_OTHER_BLOCK_LINK_CELL、EXTERN_OTHER_TARGET_LINK_CELL、EXTERN_DEPENDENCY_LINK_CELL，
  rust-extern-build-script-link-unchecked（不看构建脚本打指令）红 EXTERN_BUILD_SCRIPT_LINK_CELL、EXTERN_DEPENDENCY_LINK_CELL、EXTERN_LINK_ARGUMENT_CELL；
  rust-extern-link-of-other-targets-unchecked（块带 link 只看入口的目标与别的包的库目标）红 EXTERN_OTHER_TARGET_LINK_CELL，
  rust-extern-link-of-dependencies-unchecked（只看入口所在的包）红 EXTERN_DEPENDENCY_LINK_CELL、EXTERN_LINK_ARGUMENT_CELL，
  rust-extern-dependency-dev-dependencies-unfollowed（顺依赖往下时不跟别的包的开发依赖）红 EXTERN_DEPENDENCY_LINK_CELL，
  rust-extern-link-arguments-unchecked（只认 rustc-link-lib）红 EXTERN_LINK_ARGUMENT_CELL，rust-extern-two-colon-instructions-ignored（只认 cargo: 一个冒号）红 EXTERN_LINK_ARGUMENT_CELL，
  rust-extern-instruction-words-bare（指令名裸词也算）、rust-extern-library-literals-of-any-package（别的包的库代码都看）各红 EXTERN_INSTRUCTION_LITERAL_CELL，
  rust-extern-external-dependencies-unchecked（不看外部依赖）红 EXTERN_EXTERNAL_DEPENDENCY_CELL。
构建档与线程数不进键（常变的输入不进键）：这份库与 crates/singlefs-unit-graph 都不出编译档的行
  （[profile.<名>] 的各项设置、生效的 incremental、依赖一侧的 codegen-units），也不读 jobs、CARGO_BUILD_JOBS 这一类线程数；工具链与依赖版本同样不进
  （下面「版本不进键」）。
  自证 PROFILE_UNKEYED_CELL 读 crates/singlefs-unit-graph/src 与这份库的几份源码，判两边都没有编译档的行；弄坏开关 profile-unkeyed-scan-sample 下
  多扫一段带 PROFILE_SETTING_KEYS 的样本文字，这一格判错。
测试二进制起自己（RustFile.current_exe_child；规则名 current-exe-child；crates/singlefs-unit-graph 同写）：
  Command::new(…current_exe()…) 后接 .arg(…) / .args([…]) 链，参数里恰好有一个字面 "--exact"，紧跟在它后面的那个参数折得出测试名的，
  从起子进程的那个单元连一条边到同一编译目标里 libtest 全名（文件在目标里的模块路径加限定名）恰好等于它的那一个 #[test] fn，不拒算 current-exe。
  测试名折法：字面串；常量名（名字记下来，值在收闭包时解：先找起子进程那个单元自己里面的 const <名>: &str = "…";，再找同一个包、在这一次链接集里叫这个名字的
  常量单元，几个都得写成 = "…";、值相同；单元里另有 let 绑定的名字不认，是参数的按参数）；全由这两样组成的 concat!(…) 与 format!("…{}…{名}…", …)；
  起子进程那个 fn 自己的参数名：闭包里每一处调用它（不是 .方法 调用）的那个实参各折一遍，各自再对 #[test] fn，每趟 drain 之后、WGSL 供给之前跑，
  一处都没有、折不出、实参又是参数的拒算。测试名写在 --exact 前面、用 module_path!() 拼的、套着 as_str() 的、单元里另有 let 绑定的，折不出。
  同一个函数里起了子进程的（函数里出现 SPAWNING_WORDS 里的词：Command、spawn、posix_spawn、fork、execv、execve、execvp），对不上恰好一个 #[test] fn、折不出名字、
  不是 Command::new(current_exe()) 的写法的（路径改过、流向别处），照旧拒算 current-exe；路径原样交给 Command::new 的看入口（「程序起自己」）。
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
  toolchain\t<调用方给的一行>          排序（这份库与 admission.py 不给，下面「版本不进键」）
  dependency\t<调用方给的一行>         排序（同上）
  external\t<程序名>\t<sha256>\t<--version 那一行>   调用方给了才出；排序（同上）
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
版本不进键（D13（验证路线） 已定项 17：键只由代码函数、屏障与数据流组成）：工具链（rustc -vV、cargo -V、目标、RUSTFLAGS 这一类构建环境）、解释器、
  外部程序（cargo、herd7、busybox、qemu-system-*：内容与版本）与 Cargo.lock 里的依赖版本，这份库自己一样都不出：命令行 key 不收工具链、依赖与外部程序的行，
  闭包里的工具链询问不出工具链单元（「版本询问」），闭包点名了外部程序（Closure.programs：一个词（python 里一个串）去掉目录后等于 cargo / herd7 / busybox，
  或以 qemu-system- 起头、后面还有字；带空白的串、消息命令 echo、printf、die、bad、howto 后面的词不算）只记、不拒算；admission.py 的 compute_unit_key 同样不给。
  unit_graph_key 只给对拍、不存判定：照 crates/singlefs-unit-graph 的 reuse_key 把调用方给的工具链行与依赖行接在 entry 行之后（Rust 那一份 ExternalKeyInputs 同一口径；
  门禁 code-tooling 的 function-units-two-implementations-agree 两边同给一行 host:，只为定目标平台）。closure_key 的 toolchain、dependencies、externals
  三个参数只在调用方给了时出行（scripts/mutation/mutation-prune.py、broad_mutation_judging.py 与 scripts/crash/lib_verdict_library.py 的 FunctionUnitIndex.key 给，
  归它们那一处）。Verus、herd7、QEMU 这类判定工具升级由人登记、整族删重判，不靠键作废。
  弄坏开关 external-programs-refused：闭包点名了外部程序而调用方没给 externals 的拒算 external-program-unkeyed；python-toolchain-queries-keyed 见「版本询问」。
目标写法：<包目录名>:<lib|bin|test|bench|example|build|doctest>:<名>。

入口：<文件>::<限定名>（按后缀认语言：.rs .wgsl .py .sh / .bash）；只写 <文件> 的，Python 是程序入口（main-guard，没有就 module-body），
  shell 是 script-body；Rust 另认 fn 路径 <crate 名>::<模块…>::<名>（不带文件，按 lib 目标的模块树找）。

拆成的文件（同目录；这份入口 import 它们、再导出每个顶层名字，别处按原名取。文件头里写「这份库」「这份模块」的，代码在这些文件里）：
  lib_function_units_lexer.py                Rust 与 WGSL 的词法切分（rust_lex）、词法缓存、跨轮缓存目录、弄坏开关变量与取法（active_breaks）
  lib_function_units_base.py                 常量、错误种类、Refusal、Unit、数据文件单元、外部程序与命令包装的认法、库所在仓的根 LIBRARY_ROOT
  lib_function_units_registry.py             按尾巴读的登记 TAIL_READ_SCOPES、不拒算的程序词、起仓内程序的登记、不走的调用边与子进程边、按角色排除的文件、
                                             测试服务代码（is_test_service_code，读 scripts/gate/test-service-code-paths.tsv）
  lib_function_units_python_facts.py         Python 切单元与折叠用的小函数、KnownWords、PythonFacts
  lib_function_units_python_module.py        PythonIndex 与 PythonModule（切单元、名字绑定与解析、值的折叠）；PythonModule 继承下面两份混入类
  lib_function_units_python_data_reads.py    混入类 PythonModuleDataReads：运行期读的数据文件的出处、单元的事实收集、程序入口带参数的剪枝
  lib_function_units_python_launches.py      混入类 PythonModuleLaunches：调用事实、子进程命令的折叠与候选取值、起脚本与内联脚本的边
  lib_function_units_shell_text.py           shell 规范文本、here-doc 与引号里的行、参数代入、分支判定、按行键选行的表、命令切分
  lib_function_units_shell_index.py          shell 单元与调用边：ShellFile、ShellIndex、仓内 shell 文件的枚举、Rust 闭包读到的 shell 脚本
  lib_function_units_rust_file.py            Rust 源文件切单元（RustFile）与词法单元上的小解析
  lib_function_units_rust_verus.py           verus! 里规格与证明的识别与剥除
  lib_function_units_unit_graph.py           产品行为视图的 unit-graph 程序（缓存目录、输入摘要、编译、取 dump）
  lib_function_units_rust_workspace.py       Rust 工作区：清单与目标、cfg 求值、RustWorkspace、Closure
  lib_function_units_wgsl.py                 WGSL 切单元与闭包
  lib_function_units_rust_closure.py         Rust 闭包规则（rust_closure_of）与 WGSL 外名的着落
  lib_function_units_script_closure.py       Python 与 shell 的调用图收闭包（ScriptClosure）、入口写法、Options、命令行选项、closure_of
  lib_function_units_closure_keys.py         键与摘要：closure_key、unit_graph_key、dump 与 explain 行、变异控制组、跨轮缓存的登记
  lib_function_units_selftest_*.py           自证的格函数与并行编排（lib_function_units_selftest_run.py 是 selftest 本身，成功句报「并行 {jobs} 个进程」）
  lib_function_units_selftest_fixtures*.py   自证夹具常量三份（按次序载入，格式见「自证夹具」）
  这份入口只留命令行的 main、弄坏开关只在自证里认的放行（settle_break_switches）与 preflight。

用法：
  lib_function_units.py dump <根> [--language rust|python|shell|wgsl] [--under <目录>]… [--judgment-kind product-behavior|formal-verification]
  lib_function_units.py closure <根> <入口> [选项] [--judging] [--explain]
  lib_function_units.py key <根> <入口> [选项] [--judging]
  lib_function_units.py controls <根>     V5 / V11 两处变异（crates/mutations.tsv 里原文是那两行的）在临时拷贝上施加，
                                           三个入口的键变不变与 design 报告 2.2 节那张表比；只读根，不写根
  lib_function_units.py entries <根>      判法入口的登记现取（registered_judging_entries），一行一个；只读根
  lib_function_units.py cross-run-caches  跨轮缓存的登记（cross_run_cache_registry，下面「跨轮缓存的登记」）照此刻的环境现取每一份的位置：先打 tempdir\t<tempfile.gettempdir()>，
                                           再一份一行 cache\t<名字>\t<脚本>\t<位置>（算不出的打 refused-cache\t<名字>\t<脚本>\t<原因>、退 3）；只读，不建目录
  lib_function_units.py --selftest        临时目录里的夹具，另把本仓的 scripts/ 与 .claude/（不带 fixtures、kb）拷进临时目录量真仓那几格；不写本仓。
                                           各格与各开关下的每一次重判各在自己 fork 出来的子进程里跑（gather_forked），同时至多每台默认重活线程数个
                                           （scripts/hosts/heavy_threads.py 的 default_heavy_threads，SINGLEFS_THREAD_CAP 设了照它；取不到退 2）；
                                           结论照登记次序打（先各格、再各开关），每条子进程写下的输出打在它那一行之前；没收回结果的逐条点名、整份红
  选项：--target <目标>  --cfg <名或 名="值">（加进全局 cfg）  --no-default-cfg  --feature <包名>/<特性>  --no-default-features
        --wgsl-module <文件>（WGSL 入口拼进同一个模块的别的 .wgsl）  --rust-strings-entry <Rust 入口>（WGSL 里没定义的名字到这个入口闭包的串字面量里找）
        --under <目录>（python / shell 扫哪几处，默认 scripts、.claude/gate.d、.claude/hooks、.claude/scripts 与入口所在目录）
        --judgment-kind product-behavior|formal-verification（Rust 单元在哪个视图上切、收、算，不给是 formal-verification；下面「判定种类」）
        --program-argument <词>（可给几次，照次序是程序入口的参数；上面「程序入口带参数」，只对只写 <文件> 的 python 与 shell 入口起作用）
        --judging（照判定算键的口径按角色排除文件：下面「按角色排除的文件」；不带的什么都不排）
退出码：0 判完；2 用法错；3 算不出（根不在、bash 起不来、清单读不出）；4 拒算（closure / key 的闭包里有拒算：打拒算行，key 不打键行）。

库接口（门禁里 python 写的判法直接 import 它算键，不为算键先编 Rust；产品行为视图例外，下面「判定种类」）：
  RustWorkspace(根, 判定种类=FORMAL_VERIFICATION, product_dump=None)：判定种类给 PRODUCT_BEHAVIOR 的是产品行为视图；
  rust_units(根) → [单元]；rust_closure(根, 入口, 目标=None, cfg=None) → 闭包；wgsl_units(根)、python_units(根, under)、shell_units(根, under)；
  closure_of(根, 入口, 选项) → 闭包（.members 成员与数据单元、.refusals、.cfg_lines、.programs）；
  closure_key(闭包[, toolchain, dependencies, externals]) → (键或 None, 拒算)；row_keyed_closure(闭包, 根, 入口文件) → 按行进键的登记表换过的闭包；unit.text() 是进 UnitKey 组成清单的单元行（嵌套路径），
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
    没设照这份库所在仓的 crates/singlefs-unit-graph、.claude/heavy-thread-default 与 Cargo.lock 的内容摘要（unit_graph_inputs_digest；这一步经
    「不走的调用边」EXCLUDED_CALL_EDGES 登记的那一条调，这三样不进各条判法的闭包）找
    <跨轮缓存目录>/singlefs-unit-graph-product-view/<摘要>/unit-graph（跨轮缓存目录：设了 GATE_CROSS_RUN_TMPDIR 取它，没设取这份库所在仓的 git common-dir（git worktree 里是主仓的 .git），不在 git 仓里取 TMPDIR），没有就在那里的临时工作区经 scripts/hosts/capped.sh default
    起 cargo build --release --offline 编一份（几个进程同时要时排他锁，编完删工作区与编译目录，只留最近 3 份程序）。
  RustWorkspace 构造时取一次 Rust 那一份（fork 出来的子进程共用父进程那一份），之后载入的有 verus! 块或写着外属性 #[verifier…] 的文件照同一份改。

Rust 词法单元的缓存（rust_lex_cached）：RustFile 载入一份 .rs 先切词法单元（rust_lex 按字符逐个走，是载入整个工作区最费的一步，全仓约几秒到几十秒），
  结果只随这份文本变，而点名几行的 row-plan 每趟都是新进程、要把全仓的 .rs 重新切一遍。切出来的 (kinds, texts, lines, documents) 按文本的 sha256 存成一个 zlib 压缩的 json 文件
  （去重的词表加 base64 的序号表，不用 pickle：缓存目录在可共写的临时目录里；全仓约 20 MB），放 <跨轮缓存目录>/singlefs-rust-lex-cache/<目录名>/（同产品行为视图那一段的跨轮缓存目录）；
  FUNCTION_UNITS_LEX_CACHE_DIRECTORY 设了放那里，写 off 不用缓存（自证除缓存那一格外都设 off）。目录名（rust_lex_cache_namespace）取 rust_lex_cached 整个闭包的现场指纹
  （live_closure_fingerprint：从 rust_lex_cached 起照字节码按全局名往下收，rust_lex 与它的帮手、rust_lex_cache_path、读写缓存的 load_rust_lex 与 store_rust_lex、
  格式号 RUST_LEX_CACHE_FORM、词种名常量 IDENT / LITERAL / LIFETIME / PUNCT 都在里面；进程的计数与已算好的目录名 RUST_LEX_CACHE_PROCESS_STATE 只记名字；
  模块放在哪、怎么载入的 MODULE_PLACEMENT_NAMES（cross_run_cache_directory 读的 __file__ 这类）也只记名字：放处已经定了缓存目录在哪，同一份代码在别的路径、
  别的工作树里目录名照旧相同）加此刻的弄坏开关，
  不读这份库的原文：闭包里有一样变了旧目录整个不再取，改这份库别处的注释、文档串与不在闭包里的函数，目录不换；经它算键的判法闭包里因此没有这份库整份的
  source-text 与 data-file（自证 RUST_LEX_NAMESPACE_CLOSURE_CELL 在真仓拷贝上判）。解释器换了版本，字节码跟着变、目录跟着换。
  只留最近 3 个；文件只看文本内容，不看路径、修改时刻、不设时限；读不了、写坏了、各表对不上的照常现切再存；写不成不影响结果。

跨轮缓存的登记（cross_run_cache_registry；命令行 cross-run-caches）：仓里有意跨轮复用的缓存都登记在这一处（.claude/singlefs-ai-sop/rules/command-safety.md
  「测试镜像一律放临时目录」末段：放 GATE_CROSS_RUN_TMPDIR 底下或项目定的位置，不放这一趟的 TMPDIR）。一行（名字、放它的脚本、现取它的位置的函数）；
  现取 = 调那份脚本自己算位置的那一段：本库的两份调本库算位置的函数，shell 读那份脚本里恰好一行的双引号赋值、
  照此刻的环境展开（shell_assignment_row；expand_shell_parameters 只认 $名、${名}、${名:-词}、${名-词}），不在这里另抄一份位置。
  test-environment 那一行位置分两段算（脚本里先取 GATE_CROSS_RUN_TMPDIR，没设再调 .claude/hooks/lib_work_root.py 的 shared <名>，再接文件名），不是一行赋值，
  现取 test_environment_stage_run_times_file 逐段从那份脚本里读：「local run_times_file cross_run_cache=」那一行读 GATE_CROSS_RUN_TMPDIR，没设时读脚本里恰好一处
  「python3 "$STAGE_DIR/../hooks/lib_work_root.py" shared <名>)」取名字、调同一个库（载入这份库所在仓里的 .claude/hooks/lib_work_root.py）的 shared_data_path，
  「run_times_file=」那一行照这个目录展开成文件；名字、目录、文件名都不在这里抄，哪一段脚本改了写法，对应那一段抛 ValueError、点名那份脚本，自证那一格红。登记的是：本库的 Rust 词法单元缓存
  与产品行为视图编出的 unit-graph 程序、.claude/gate.d/test-environment.sh 记的上一趟各阶段用时、
  .claude/scripts/lkmm.sh 与 .claude/scripts/fetch-deps.sh 的下载缓存。Rust 一侧 crates/singlefs-verification/src/gpu_unit_checks.rs 的 driver_shader_disk_cache_directory
  （没设 GATE_CROSS_RUN_TMPDIR 时放家目录的 .cache/singlefs 底下）取位置要编 验证包，不登记。新加一份跨轮缓存，在 cross_run_cache_registry 加一行。
  自证两格：CROSS_RUN_CACHE_CELL 把本仓 scripts/ 与 .claude/ 拷进临时目录（real_tree）、建空的 .git（在 git 仓里），另建一个空目录当 TMPDIR，摘掉 GATE_CROSS_RUN_TMPDIR
  与缓存自己的改放处变量（FUNCTION_UNITS_LEX_CACHE_DIRECTORY），起子进程跑拷贝里
  这份库的命令行 cross-run-caches：子进程的 tempfile.gettempdir() 要是那个目录、登记的每一份都算得出、没有一份落在它之下。
  同一格再用 git 造一个主仓与它的 worktree（git worktree add --detach，worktree 里 .git 是文件），scripts/ 与 .claude/ 拷进 worktree 照样起一趟：
  照样不落在那一趟的 TMPDIR 之下，头一趟落在 <拷贝>/.git 底下的那几份，这一趟要落在主仓 .git（git common-dir）底下同一相对路径（与主仓共用同一份）。
  登记表写成函数（cross_run_cache_registry），不写成模块级的表：表里现取位置的那几个函数（载入 lib_work_root.py、读别的脚本）不随 import 时执行的模块级语句进别的闭包。
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
         compiled-part-exec（模块级把读出的部分文件编了 exec 进自己的命名空间，上面「按部分文件 exec」；exec 进模块对象而源码折不出，上面「exec 进模块对象」）
  shell  bash-syntax shell-path-unfolded shell-path-missing shell-path-outside-root shell-command-unfolded（命令位置上的程序折不出）
  公共   inline-script-not-literal（-c 的串、喂给标准输入的脚本折不出，或标准输入从哪来认不出）
         fold-limit-exceeded（FOLD_LIMIT_FORM：折出的取值超过 FOLD_LIMIT 种，那一处又没有收全的认法；PythonIndex / ShellIndex 的 fold_limit_hits 计超了几次，
           用的地方比前后两次的数：python 读文件落到「推不出」那几样（fold_limit_refused）、python 起子进程折不出（refuse_launch、element_over_fold_limit）、
           shell 起脚本与命令折不出（不照根里同一相对路径那一份猜，整组、整个目录照走）、交进内联 python 的参数、环境与工作目录超了（resolve_inline）、
           python -m 的模块名与目录超了（resolve_module_run）；起 shell 脚本的字面参数超了照旧整份收脚本体，不拒）。
           python 按尾巴读的 os.path.join(<折不出的前缀>, <折得出的几段>) 超了而折得出的那几截（每一种取值都以「/<那一截>」收尾）在仓里一份都对不上的，
           整组都对不上，照那几截认、不拒（PythonModuleDataReads.unmatched_tail_ends；有一截对得上照拒；弄坏开关 fold-limit-unmatched-tails-refused 下照拒）
  起代码的写法逐种认（shell 里每一个解释器词，python 里每一次起子进程），每一种要么连边、要么拒算：
    解释器 + 脚本：连到那份脚本；脚本之后接着往后认（bash capped.sh N python3 x.py、run-with-memory-cap.sh 上限 bash y.sh 里的第二个解释器）。
    -c 的串、- / /dev/fd/N / bash -s 喂的 here-doc、here-string（含 { … } 3<<'词' 写在组后面的）：正文字面的、或折得出的
      （$变量、$(cat <<'词' …)、$(只打字面行或函数体是一条 cat <<'词' … 词 的函数)）当内联脚本：python 的切成单元（不进闭包，穿过它连下去；sys.argv、命令前设的环境变量、同一个单元里 export 过的变量
      折成那条命令里的词），bash 的切开再认；折不出拒算 inline-script-not-literal。内联 python exec sys.argv[k:] 时第 k 个词起照命令认。
      内联脚本里 printf %q 引过的词（ShellIndex.quoted_word_inline_texts）：bash -c 的串是一个词、整个折不出，而词里 $(printf '%q' <词>) / $(printf %q <词>)
      那几处之外都折得出的：printf 照 bash 的引法把一个值引成恰好一个词，正文的词法不变，只有那一个词的取值不定；那一处换成 <词> 的取值逐个 shlex.quote
      （折得出的）或一个取值折不出的词（QUOTED_WORD_UNFOLDED），正文照切、照认：它落在命令或脚本的位置上照折不出拒算，当参数、重定向的目标照不收。
      只认 shell 的内联正文。弄坏开关 shell-inline-quoted-words-unfolded 下不认（照旧拒算 inline-script-not-literal）。
      $(函数 [<字面词…>])，函数体只有一条 printf '%s\n' <词…>（ShellIndex.function_printed_word）：每个词里的 $1… 换成给的词、在那个函数里折，
      几个词的取值逐行连起来、去掉末尾的换行（printf 每个词打一行，$( … ) 去掉末尾的换行）；有一个词折不出的照旧折不出。
      弄坏开关 shell-function-printed-words-single 下只认给了字面词、printf 只打一个词的那一种。
      IFS= read -r -d '' <变量> <<词 … 词（ShellIndex.command_facts 里认，read_heredoc_assignment）：变量的取值是 here-doc 的内容（正文加末尾一个换行；
      带引号的词原样，不带引号的照 bash 展开），照一处赋值记；IFS 不是命令前写的空值、没带 -r、-d 后面不是空串、别的选项、几个变量名、
      输入不是这一条命令自己的 here-doc（< 文件、管道、while 循环）的都不认：read 改写的每个变量（选项之后的名字、-a 的数组名，没写名字的是 REPLY；
      read_target_names）记一处折不出的赋值（READ_UNFOLDED_WORD），同一变量别处赋的字面值不替它，折这个变量时整个折不出（先赋字面空串、再从 "$1" 读的照拒算）。
      弄坏开关 shell-read-heredoc-assignments-ignored 下 here-doc 那一种也不认；shell-read-assignments-unrecorded 下不认的那几种不记（只剩别处的字面赋值）。
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
      脚本体单元与指纹照原来那一份（键照整份脚本体算），只剪边。
      脚本路径照根里同一相对路径认出来的（"$<折不出的目录>/<根里的相对路径>"、cd 到折不出的目录之后的相对路径：仓副本、临时树里的那一份）同样照这组词记、照它剪
      （ShellIndex.record_launch_words）；弄坏开关 shell-tree-copy-launch-words-dropped 下这两种不记：这个单元起那份脚本只经它们时脚本体整份收、按第一列选行的表连整张，
      与字面路径的起法混在一起时只照字面那几处剪。
      起法里折不出的词之前有字面词的（bash <脚本> -- "$@"、[…, <脚本>, "--", *命令]）：字面词照代，折不出的那个词起是开口的一串（SHELL_OPEN_TAIL），那几个位置不代、也不代成空串；
      开口的一串里有没有 --force 动不了字面词的位置。头一个词就折不出的那一处，那份脚本体整份收。弄坏开关 shell-open-tail-unnarrowed 下有一个词折不出就整份收。
      python 起 shell 不看入口带没带参数：[…, <脚本>, 词…] 的每个词折得出（折出几个取值的逐个展开，*<参数> 照调用处给的列表摊开），同样照它们剪
      （弄坏开关 python-shell-launch-arguments-unfolded 下整份收）。项目脚本开头 preflight "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+…} 那一句不算动了位置参数（它只摘 --force；
      起法里带 --force 的照算动了；弄坏开关 shell-preflight-arguments-moved 下照算）。脚本体顶层的 if 走到的那一支以 exit 收尾，fi 之后的整段走不到（弄坏开关 shell-exit-branches-followed 下照收）。
      按第一列选行的表（SHELL_ROW_SELECTED_TABLES：scripts/experiment-apparatus/replay.sh 的复跑表与按行的准入键）：起法里有不带 - 起头的词（实验号）或 --item <行项> 时，
      这一趟跑不到的行不收边（table_rows_selected，照 replay.sh 的 want_row：没被 --item 点到的实验号整个跑；行项写成 <实验号>.<行序号>，行序号是这个实验号的行在复跑表里
      从 1 数的次序，被点到的实验号只跑点名的那几行，准入键表的行按第二列抄的复跑表前三列认是哪一行）（表里写的 @驱动函数按名字连，别的实验与别的行的驱动走不到；
      弄坏开关 shell-table-rows-unselected 下整张收，shell-table-row-items-unmatched 下行项照实验号拿去比第一列、点到的那一行也不收）。
      表本身是个变量单元（TABLE=$(cat <<'TSV' …），别的变量与函数读 $TABLE 时连的是它：闭包里这份脚本只按实验号起过的，读表连到只留这些实验那几行的那一份（ScriptClosure.push_variable，ShellIndex.selected_table_variable；
      指纹照整张表）；这份脚本在闭包里整份起过（入口就是它而没带参数、source 了它、有一处起法折不出）的，连整张表；入口就是它而带了参数的照这组词算一次起法（文件头「程序入口带参数」）（先按实验号连过的也补上，table_launched）；
      带字面参数起（入口或 bash <脚本> <词…>）而照这组词剪过的脚本体里表的赋值行一行都不剩的，不算一次起法（launch_defines_tables，文件头「程序入口带参数」）。弄坏开关 shell-table-rows-unselected 下一律连整张表。
      表行之外写到的驱动函数（ScriptClosure.table_unselected_drivers、written_only_as_table_row）：这份脚本在闭包里只按实验号起过时，只写在别的实验那几行的 @驱动函数，
      单元里每一处都只写成 @<名>（行的身份：declare -A 的键、[[ "$bin" == @<名> ]]）或 <名>()（定义：载入时换掉驱动的 eval '<名>() { …; }'）的，不按词连：
      这一趟跑的行里没有它，驱动只由表行起。不带 @ 的词（命令位置上的、放进变量再起的）照连；整份起过、一处都没起过（入口是 <文件>::<函数>）的不走这一条。
      这一条不看起的是根里那一份还是临时目录里的副本：副本照根里同一相对路径那一份认（tree_copy_file），副本里留下的框架函数照连，表换成桩的副本带实验号起时，真表里的驱动函数不连。
      不带实验号起的副本照整张表连（认不出表换没换，只多收不少收）。弄坏开关 shell-row-driver-mentions-linked 下照按词连，shell-row-driver-plain-words-dropped 下不带 @ 的词也不连，
      shell-row-driver-whole-launch-dropped 下整份起也不连，shell-row-driver-selected-rows-dropped 下点名的实验那几行也写到的驱动不连。
      弄坏开关 shell-program-arguments-unnarrowed 下不剪。
    shell 起 python：python3 <脚本> <词…> 的词照「程序入口带参数」子进程边带过去（ShellIndex.python_launch_words，KnownWords）：字面的是已知的词，一个双引号括着、里面不是 $@ 与 [@] 的是不知道的一个词，
      别的（不带引号的展开、"$@"、拼着展开的）从那里起是开口的一串。闭包的入口带没带参数都这样；python 起 python 同样不看入口（ScriptClosure.python_edges）；被这样起的模块的 import 时赋值照「给了参数」收（narrowed_programs）。
      同一个单元里有一处起法是经尾巴组、目录通配认出来的，那份脚本整份收。弄坏开关 shell-python-launch-arguments-unfolded 下整份收；python-program-arguments-unnarrowed 下 python 起 python 也整份收。
    shell 里套在函数里的函数（ShellFile.parse 记的 container）：只有外层函数跑过才定义，外层函数（连同它里面）之外的单元只在命令位置上写到它的名字才连（ScriptClosure.shell_nested_function_named）：
      驱动函数里一个叫 device 的变量不连到某个自证函数里套着的 device()，也就不沿 container 把那个自证函数整个连进来。弄坏开关 shell-nested-functions-by-any-word 下照旧按词连。
      外层函数只认自己那一层：指纹照含套着的函数的全文算，切命令、认词用的正文（Unit.body）把套在里面的函数那几行抹空，内层的命令只归内层，
      内层起的折不出的程序词拒算落在内层名下、照内层的短名查 UNKEYED_PROGRAM_WORDS。外层调到内层照命令位置上的那个词连。
      赋值、local、export 与写出的脚本照全文认（Unit.whole_body；套着的函数经 container 找外层的变量与兄弟函数写出的脚本，照旧看得见）。
      弄坏开关 shell-nested-commands-in-outer 下外层照全文认命令。
    python 的命令：字面列表、a + b、x if c else y、*展开、本函数里的赋值与 .append / .extend / +=、函数参数（本模块每一处调用给的实参）、
      调仓内函数的返回值都折；shell=True 与 os.system 的串当内联 bash；认不出拒算 subprocess-command-unfolded。另认：
      套在函数里的函数自成一层作用域（PythonModule.nested_unit / scope_of）：它的局部赋值与参数照它自己的，参数按外层函数里调它的那几处折，别的名字到外层找；
      a + b 有一边认不出那一段占一格（同 *展开认不出），命令位置落在它上面照旧拒算；f(…)[k] 取 f 各处 return 的元组第 k 项，x[a:b] 照切
      （b 可以是 x.index(<字面词>)：到头一个等于它的词之前，切点前折不出的词能断定不等于它的照过）；env 后面 f"{名字}={折不出}" 名字折得出照一条赋值认；
      类里方法的参数按本模块每一处 <对象>.<方法>(…) 折；推导式的循环变量取遍它迭代的元素（序列可以是调仓内函数的返回值、{…}.get(键, 缺省)、
      import 进来的仓内模块的模块级元组 / 列表 module.NAMES，弄坏开关 python-module-sequence-elements-unfolded 下不认）；
      只赋过一次的元组 / 列表字面量[整数]；spec_from_file_location 载入的仓内模块的 module.NAME；pool.submit(f, 实参) 算对 f 的一处调用；
      命令位置上折出几个取值的，每个取值当一种取法各认一遍；模块体里的语句照函数体同一条认（循环、分支里赋的名字、for 取的字面列表，模块顶层赋的也算一种取法）。
      解释器后面的脚本折不出、只认得出字面尾巴（os.path.join(<折不出的目录>, "tools", "runner.py")）：仓里以这一截收尾的文件整组连上
      （script_tail_group）；一份都没有的不是仓里的脚本（这一份自己写出来的夹具、根外的程序），不连也不拒算；连尾巴都认不出的照旧拒算 subprocess-path-unfolded。
      cwd= 折不出、脚本写的是相对路径（["bash", "setup.sh"], cwd=<拷出来的样本>）同样按 /<它> 收尾整组连上。
      版本询问（[<折不出的程序>, "-V"]、"--version"、"-vV"，命令里只有这一个词）：程序只打版本、不跑仓里的代码，不连、不拒算，记成工具链询问
      （Closure.toolchain_queries）；它打的版本不进键（「版本不进键」）：闭包里不收工具链单元，换 rustc 键不变。
      弄坏开关 python-toolchain-queries-refused 下照旧拒算命令折不出；python-toolchain-queries-keyed 下每一处询问收一个工具链单元（语言 data、种类 rust-toolchain，
      文件与行是询问那一处，指纹是根里起的 PATH 上 rustc -vV 那一行的 sha256），换 rustc 键变。
      调用方交进来的命令（refuse_launch）：命令、-c 的串、脚本路径折不出，折不出的那几截（unfolded_part_origins）出处都推得出是入口函数的参数或 sys.argv、
      argparse 解出来的属性、临时目录、根外前缀（调用方交的那一条命令，由请求那一侧的单元键与 line:command= 管）：不拒算；别的函数的参数照闭包里每一处
      调它的实参接着往上认（闭包里没人调它的推不出）；出处是环境变量、推不出的照旧拒算。是不是入口在收闭包时判（ScriptClosure.caller_launch_refusals）。弄坏开关 python-caller-launches-refused 下一律照旧拒算。
      闭包里调它的那几处（closure_call_arguments，运行期读的数据文件按调用处认参数也用它）按名字找，再只留真解析得到它的（ScriptClosure.call_reaches）：
      f(…) 被调用处往外头一层把 f 当局部名字的函数作用域截住（套着的同名函数遮住模块级的），x.f(…) 的 x 是 import 进来的别的模块（标准库、外部包、别的仓内模块）的不算；
      判不了的（self.f、载入的模块、局部变量上的属性）照旧算。弄坏开关 python-call-sites-matched-by-name 下一律照名字算。
      functools.partial(f, a, k=v) / partial(f, a, k=v)（头一个实参是名字或属性）同样算 f 的一处调用：a 对上 f 的头一个参数，照 f(a, k=v) 认（partial_bound_call）；
      弄坏开关 python-partial-call-sites-ignored 下不算（经 partial 交的实参折不到被调函数的参数上，照旧拒算）。
      调用方登记（CALLER_SUPPLIED_COMMANDS：(调用处的文件, 调用处单元的限定名, 交命令的实参的 ast.unparse) → 为什么；ScriptClosure.caller_supplied_registered）：
      共用的起法（lib_process_group.run_in_own_session 这类）拿参数起子进程，被起的那一份由调用方那一侧的请求进键时，在调用处逐处登记：
      往上认到这一处、照下一条认不出时，只这一处的那一条边算调用方交进来的、不拒算；同一个函数别的调用处照旧往上认，没登记的照拒算。
      不登在被调函数自己那一处（UNKEYED_PROGRAM_WORDS）：那一行对每一个经它起程序的闭包都生效。弄坏开关 caller-supplied-commands-refused 下登记了也照旧拒算。
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

自证夹具（load_selftest_fixtures）：只给自证与弄坏开关读的模块级常量（夹具文本、格名、开关到格的对照表 BREAK_CELLS）放在同目录的
  lib_function_units_selftest_fixtures.py、lib_function_units_selftest_fixtures_closure_rules.py、lib_function_units_selftest_fixtures_real_repo.py 三份里，
  --selftest 开跑前按这个次序按路径载入：先把自证各模块（lib_function_units_selftest_*.py）此刻的名字放进当前这一份、再执行它，它新定义的名字放进之后载入的几份与自证各模块，格函数照名字读。
  平常 import 这份库不载入它们：那几百个常量不在这份库 import 时执行的语句里，经这份库算键的判法闭包（D13（验证路线） 已定项 17）不收它们，改一份夹具不动判法的键。
  按「谁读」分：运行期函数也读的（别的文件点名的、写在串里按名字读的、import 时执行的语句读的、经别的文件调的函数读的）留在这份库各模块里；
  正文里写着 --selftest 这个词的也留在这份库里（放进夹具文件，门禁 code-tooling 的 research-script-selftest-coverage 格会把那一份认成实现了自证）。
  往这份库加只给自证读的常量，写进那几份夹具之一（每份读排在它之前载入的几份定义的名字）。
自证模块（lib_function_units_selftest_*.py 里不是夹具的那几份：格函数与跑法）：这份入口不在 import 时载入它们，main 走到 --selftest 那一支、entries 子命令才载入
  （按路径载入这份库的闭包，连同不去掉 NOT_JUDGING_FILES 的那种，因此不收自证模块的单元与它们里面的拒算）。
  别的脚本的自证与单测经这份库调的自证辅助（fixture_key、product_view_dump_of、with_unit_graph_directory）在入口里各包一层，用到时才载入定义它的那一份；
  往这份库加别处要经它调的自证辅助，照这三个包一层，不在入口模块级 import 自证模块。
  真仓格 SELFTEST_MODULES_OUTSIDE_CELL 判：按路径载入这份库的 lib_mutation_units.py 那几个入口（SELFTEST_MODULES_OUTSIDE_ENTRIES）的闭包里没有自证模块的单元、没有拒算
  （弄坏开关 selftest-modules-loaded-at-import）。

弄坏开关（FUNCTION_UNITS_BREAK=<项>[,<项>…]，只给自证证红用：--selftest 本身与它起的子进程认，平常跑设了退 2、点名只在自证里认；
自证逐个打开、核点名的那几格转红，点名的格就是 BREAK_CELLS（在 lib_function_units_selftest_fixtures_real_repo.py）里这一项的那几句，自证逐条打「✓ 弄坏开关 <项> 下「<格>」红」）：
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
  FUNCTION_UNITS_BREAK=rust-let-shadowed-names-resolved  局部 let 与参数绑定遮住的名字照旧往外解析
      「R-裸名：局部 let 绑的名字不连同名函数（let sample = …; sample + 1），改那个函数键不变」
      「R-裸名：参数名不连同名函数（fn f(sample: u32)），改那个函数键不变」
      「R-裸名：串字面量里的 {名} 是局部 let 绑的，不连同名函数，改那个函数键不变」
  FUNCTION_UNITS_BREAK=rust-macro-arguments-all-local  名单之外的宏的括号里的名字也照局部算（STANDARD_MACROS_EVALUATING_THEIR_ARGUMENTS_IN_PLACE 不再分内外）
      「R-裸名：局部 let 绑的名字只写在名单之外的宏（dbg!）的括号里，照连同名函数，改那个函数键变」
  FUNCTION_UNITS_BREAK=rust-macro-list-short  名单少最后一项（vec），standard_macros_evaluating_their_arguments_in_place 交回的与 Rust 源码里那个常量不同
      「R-裸名：宏名单与 crates/singlefs-unit-graph/src/closure.rs 的 STANDARD_MACROS_EVALUATING_THEIR_ARGUMENTS_IN_PLACE 逐项相同（读那份源码比）」
  FUNCTION_UNITS_BREAK=rust-use-paths-flattened  use 路径压平成一串比，「末段在串里」或「起头是 crate 且候选在根」就算点到（经 pub use 转出的那一份不连）
      「R-裸名：use 路径点到的模块里没有这个名字（经 pub use 转出来的），退回全连，转出的那一份改了键变」
  FUNCTION_UNITS_BREAK=rust-use-path-conditional-candidates-narrowed  use 路径点到的候选带条件编译也照点准
      「R-裸名：use 路径点到的候选带条件编译（另一支 cfg 下名字由 pub use 转来），退回全连，转出的那一份改了键变」
  FUNCTION_UNITS_BREAK=rust-use-path-re-exporting-modules-narrowed  候选所在模块另有 use 引进同名也照点准
      「R-裸名：use 路径点到的候选所在模块另有 use 按名字引进同名，退回全连，别的模块里同名的那一份改了键变」
  FUNCTION_UNITS_BREAK=rust-use-path-first-segment-imported-narrowed  相对路径的首段是别的 use 引进的名字（含 as 改名）也照点准
      「R-裸名：use 的相对路径首段是别的 use 按名字（as 改名）引进的模块，退回全连，那个模块里的那一份改了键变」
      「R-裸名：同一条 use 里先 as 改名、再用改名后的模块走相对路径（use {crate::deep::real as short, short::helper_total}），退回全连，改名前那个模块里的那一份改了键变」
      「R-裸名：use foo::{self, helper_total}（foo 是本模块的子模块）的相对路径首段 foo 被这条 use 自己引进，退回全连，别的模块里同名的那一份改了键变」
      「R-裸名：本作用域有条目级宏调用（宏体里 use … as 改名），相对路径的首段指的模块看不出，退回全连，改名前那个模块里的那一份改了键变」
  FUNCTION_UNITS_BREAK=rust-workspace-members-glob-unescaped  成员通配连仓根路径一起交给 glob（根路径里的 [ ] 当通配）
      「工作区成员：仓根路径带 [ ] 时成员照样展开」
  FUNCTION_UNITS_BREAK=rust-workspace-dot-member-unnormalized  成员写 "." 时当成目录 "."，不当根目录
      「工作区成员：[workspace] members = ["."]：没有 [package] 的根收成叫 root 的 crate 且有库目标，有 [package] 的根只收一个 crate」
  FUNCTION_UNITS_BREAK=rust-same-named-member-directories-merged  目标号的标签只取目录名
      「工作区成员：两个成员目录同名（crates/common、tools/common）各有 tests/smoke.rs，两个目标都算得出键且键不同」
  FUNCTION_UNITS_BREAK=rust-workspace-multiline-backslash-read  members 里含反斜杠的多行基础串照 tomllib 读，不报读不出
      「工作区成员：members 里写成多行基础串（三个双引号）的：没有反斜杠的去掉开头的换行照收，含反斜杠的报读不出（FunctionUnitsError）」
  FUNCTION_UNITS_BREAK=rust-use-path-first-segment-same-use-excluded  相对路径的首段检查不看路径所在的那条 use 自己（同一条 use 里先改名再走相对路径、花括号里 self 引进首段的也照点准）
      「R-裸名：同一条 use 里先 as 改名、再用改名后的模块走相对路径（use {crate::deep::real as short, short::helper_total}），退回全连，改名前那个模块里的那一份改了键变」
      「R-裸名：use foo::{self, helper_total}（foo 是本模块的子模块）的相对路径首段 foo 被这条 use 自己引进，退回全连，别的模块里同名的那一份改了键变」
  FUNCTION_UNITS_BREAK=rust-use-path-first-segment-item-macros-ignored  相对路径的首段检查不看本作用域有没有条目级宏调用
      「R-裸名：本作用域有条目级宏调用（宏体里 use … as 改名），相对路径的首段指的模块看不出，退回全连，改名前那个模块里的那一份改了键变」
  FUNCTION_UNITS_BREAK=rust-use-alias-leaf-followed-to-the-original  use 原名 as 名 那一支顺着原名点准
      「R-裸名：use 原名 as 名 那一支退回全连，别的模块里同名的那一份改了键变」
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
  FUNCTION_UNITS_BREAK=python-docstring-writes-counted  X.__doc__ = … 这一类写也当读文档串
      「python：模块级给 namedtuple 写 X.__doc__ = … 只写不读，别的文档串不跟着进闭包（改它键不变）」
  FUNCTION_UNITS_BREAK=python-docstring-augmented-assignment-not-read  读文档串只认 ast.Load，X.__doc__ += … 不算读
      「python：X.__doc__ += … 先读后写，照算读（改那段文档串键变）」
  FUNCTION_UNITS_BREAK=python-docstring-call-readers-ignored  getattr(<对象>, "__doc__") 与 inspect.getdoc(<对象>) 不当读文档串
      「python：getattr(<函数>, "__doc__") 读另一个函数的文档串（改那段文档串键变）」
      「python：inspect.getdoc(<函数>) 读另一个函数的文档串（改那段文档串键变）」
  FUNCTION_UNITS_BREAK=python-getattr-allowed  getattr(x, <变量>) 不拒算
      「python：getattr 变量名拒算」
  FUNCTION_UNITS_BREAK=python-globals-subscripts-refused  globals()[键] 与 globals().update(…) 照旧拒算 globals-lookup
      「python：globals()[固定名字] 读连到那个名字的单元，不拒算（改它键变）」
      「python：globals()[折不出的名字] 读连本模块每个名字，不拒算（改其中一个键变）」
      「python：globals()[键] = 值、del 与 globals().update(…) 只写不读：不连、不拒算」
  FUNCTION_UNITS_BREAK=python-imported-sys-path-ignored  import 进来的模块加的 sys.path 不算在引它的模块上
      「python：import 进来的仓内模块在载入时往 sys.path 加的目录，对引它的模块同样生效（那个目录里的模块连得上，改它键变）」
  FUNCTION_UNITS_BREAK=python-registered-inline-launches-analyzed  整条登记过的 bash -c 照旧拆开正文认
      「python：整条登记过的 bash -c，正文字面而 $2 换不进去（带引号的多行串）：不登记拒算，登记了不拆开认、不拒算」
  FUNCTION_UNITS_BREAK=python-spec-paths-ignored  spec_from_file_location 不折叠、不拒算
      「python：spec_from_file_location 字面路径折叠出跨模块边」
  FUNCTION_UNITS_BREAK=python-attributes-unnarrowed  obj.m 连到全部同名方法（不按闭包里出现过的类收窄）
      「python：obj.方法 按闭包里出现过的类收窄」
  FUNCTION_UNITS_BREAK=loaded-attributes-closure-wide  按路径载入、模块值出不了载入方的模块也照整个闭包见过的属性名连（载入方没调的 main 跟着别处的 x.main() 进来）
      「python：按路径以非 __main__ 名载入、值不出载入方（函数里的局部名字、内联脚本模块级的容器）的模块只连载入方调到的 wanted 与 import 时执行的，别处有 x.main() 也不连它的 main；交给调用方的、以 __main__ 名载入的照整个闭包连」
  FUNCTION_UNITS_BREAK=shell-heredoc-split  here-doc 正文照样按缩进切
      「shell：here-doc 正文里有一行 `};` 不许被切开」
  FUNCTION_UNITS_BREAK=shell-names-command-position-only  函数名只在命令位置上才连
      「shell：函数名当参数传照样连上」
  FUNCTION_UNITS_BREAK=shell-unfolded-paths-ignored  折不出的路径不拒算
      「shell：`bash "$x"` 折不出拒算」
  FUNCTION_UNITS_BREAK=shell-raw-text  脚本体的指纹取原文
      「shell：脚本体只改注释不作废」
  FUNCTION_UNITS_BREAK=external-programs-refused  闭包点名了外部程序而调用方没给 externals 的拒算 external-program-unkeyed
      「键：闭包点名了 cargo 不拒算，外部程序的版本不进键：不给 externals 也算得出键」
      「python：命令是别的函数的参数（subprocess.Popen(command)、[program, *arguments]）：照闭包里每个调用点的实参折（字面程序、根外程序、外层函数的变量、调仓内函数的返回值），不拒算」
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
  FUNCTION_UNITS_BREAK=cross-run-cache-git-directory-only  git_common_directory_of 只认 <仓>/.git 是目录（worktree 里 .git 是文件，本库两份退回这一趟的 TMPDIR、每趟重算）
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
      「shell：`bash "$tree/<仓里的相对路径>"` 只连根里同一相对路径那一份，别处以这一截收尾的不连（改它键不变）」「python：os.path.join(<折不出的目录>, <仓里的相对路径>) 连到仓里那一份」
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
  FUNCTION_UNITS_BREAK=python-module-sequence-elements-unfolded  for 名字 in module.NAMES（import 进来的仓内模块的模块级元组）的循环变量不折
      「python：读 os.path.join(<参数缺省的目录>, <for 名字 in 别的模块.NAMES 取的一项>)（读的函数在另一份模块里）收那几份文件：改其中一份键变，不拒算」
      「真仓：E171 重跑的两份入口脚本（名单核对 list_check.py、变异驱动 mutation_driver.py）的闭包都不拒算；没被单测点名的自证格（reuse_library 的 cell_rp1_layers_end_to_end、」
  FUNCTION_UNITS_BREAK=fold-limit-reads-skipped  python 读文件的路径取值超过 FOLD_LIMIT 种、落到「推不出」那几样的，照原样交（既不收也不拒算）
      「python：读 os.path.join(<根本身>, <取遍 65 项字面元组的循环变量>)（取值超过 FOLD_LIMIT 种）拒算 fold-limit-exceeded，不当调用方的输入不收」
  FUNCTION_UNITS_BREAK=fold-limit-launches-skipped  python 起子进程那一处取值超过 FOLD_LIMIT 种才折不出的，照折不出的那一路认（refuse_launch 不判 over_limit）
      「python：起 ["python3", os.path.join(<根本身>, <取遍 65 项字面元组的循环变量>)]（取值超过 FOLD_LIMIT 种）拒算 fold-limit-exceeded」
  FUNCTION_UNITS_BREAK=fold-limit-unmatched-tails-refused  python 按尾巴读的 os.path.join 超过 FOLD_LIMIT 种时，折得出的那几截在仓里对不上也照拒算（unmatched_tail_ends 一律交 None）
      「python：读 os.path.join(<参数>, "dump", <3 种流>, <5 种文件名>)，参数照调用处取 5 种尾巴（合起来 75 种、超过 FOLD_LIMIT）：折得出的那一截（dump/<流>/<名>）在仓里一份都对不上，整组都不是仓里的文件，不拒算」
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
  FUNCTION_UNITS_BREAK=shell-heredoc-captures-unfolded  $(函数) 不照函数体折（ShellIndex.function_output 不认）：函数体是一条 cat <<'词' … 词 的折不出；
      只打 printf '%s\n' <词…> 的那一种归 function_printed_word、照折，$(cat <<'词' …) 直接写的照 substitution_outputs 折
      「shell：python3 -c "$(函数体是一条 cat <<'词' … 词 的函数)" 折出那份 here-doc 的正文当内联脚本」
  FUNCTION_UNITS_BREAK=declared-checks-ignored  文件头登记的 check 判据不连、上游 preflight 跑它们的那一处照样拒算
      「shell：文件头登记的 check 判据（preflight 开跑前跑）照内联 bash 认，连到它起的脚本」「python：上游 preflight 跑文件头登记的判据（bash -c declaration.command）不拒算，判据从声明它的脚本连」
  FUNCTION_UNITS_BREAK=inline-argv-commands-ignored  内联 python exec sys.argv[k:] 时不把后面的词当命令
      「shell：内联 python exec sys.argv[1:] 时后面那几个词照命令认」
  FUNCTION_UNITS_BREAK=python-unfolded-heads-allowed  python 命令位置上折不出的程序不拒算
      「python：变量解释器折不出（出处是环境变量）拒算 subprocess-command-unfolded」
  FUNCTION_UNITS_BREAK=python-commands-literal-only  python 的命令只认字面列表与只赋过一次列表的名字（认不出的不拒算）
      「python：命令是 [解释器] + 参数（按调用处给的列表折）连到那份脚本」「python：认不出的命令（对象的属性）拒算 subprocess-command-unfolded」
  FUNCTION_UNITS_BREAK=python-path-programs-refused  shutil.which(…) 与 os.path.join(<折不出的目录>, <根外的字面尾巴>) 起的程序照样拒算
      「python：shutil.which(…) 与 os.path.join(<折不出的目录>, "bin/x") 起的根外程序不拒算、不当工具链询问」「python：内联脚本起 sys.argv[k]（取的是起它那份脚本的 "$1"）不拒算、不当工具链询问：程序是调用处给的词」
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
      「python：方法参数按每一处 .方法(…) 调用折，推导式变量取遍函数返回的字典里的列表，命令位置上折出几个各认一遍（python3 / bash --version 不跑代码，不当工具链询问）」
  FUNCTION_UNITS_BREAK=python-method-parameters-unfolded  类里方法的参数不按调用处折
      「python：方法参数按每一处 .方法(…) 调用折，推导式变量取遍函数返回的字典里的列表，命令位置上折出几个各认一遍（python3 / bash --version 不跑代码，不当工具链询问）」
  FUNCTION_UNITS_BREAK=python-comprehension-variables-unfolded  推导式的循环变量不取遍它迭代的元素
      「python：方法参数按每一处 .方法(…) 调用折，推导式变量取遍函数返回的字典里的列表，命令位置上折出几个各认一遍（python3 / bash --version 不跑代码，不当工具链询问）」
  FUNCTION_UNITS_BREAK=python-element-calls-unfolded  序列是调仓内函数的返回值、{…}.get(键, 缺省) 的不取元素
      「python：方法参数按每一处 .方法(…) 调用折，推导式变量取遍函数返回的字典里的列表，命令位置上折出几个各认一遍（python3 / bash --version 不跑代码，不当工具链询问）」
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
      「python：读 os.path.join(<根里的目录>, <从那个目录的清单里挑的一截>) 收那个目录整棵：改目录里的一份键变」
      「python：别的模块里的函数读它参数给的文件：闭包里的调用处给仓里的常量收进键（改它键变），给入口参数的不收也不拒算」
      「python：读 self.<属性>：照本类方法里给它的赋值认，赋的是仓里的常量就收进键（改它键变）」
  FUNCTION_UNITS_BREAK=python-parsed-arguments-unread  argparse 解出来的路径不当调用方的输入（照拒算）
      「python：读 argparse 解出来的路径（调用方的输入）不拒算」
  FUNCTION_UNITS_BREAK=python-directory-reads-refused  os.path.join(<根里的目录>, <从那个目录的清单里挑的一截>) 不收目录、照拒算
      「python：读 os.path.join(<根里的目录>, <从那个目录的清单里挑的一截>) 收那个目录整棵：改目录里的一份键变」
  FUNCTION_UNITS_BREAK=python-unlisted-directory-reads-whole  os.path.join(<根里的目录>, <出处推不出、不是从清单里挑的一截>) 照旧收那个目录整棵、不拒算
      「python：读 os.path.join(<根里的目录>, <出处推不出、不是从清单里挑的一截>) 拒算 data-path-unfolded，不收那个目录整棵」
  FUNCTION_UNITS_BREAK=python-returned-listings-ignored  经仓内模块级函数的返回值交出来的目录清单不当清单（从里面挑的一截照拒算 data-path-unfolded）
      「python：读 os.path.join(<根里的目录>, <本模块函数返回的那个目录的清单里挑的一截>)（for 名字 in 列清单的函数(目录)）收那个目录整棵：改目录里的一份键变」
  FUNCTION_UNITS_BREAK=python-glob-listings-ignored  glob.glob / glob.iglob 列的不当按名字模式列的清单（从里面取的文件名照拒算）
      「python：读 os.path.join(<根里的目录>, <glob 按 *.sh 列出、os.path.basename 取的文件名>) 只收目录里合 *.sh 的几份：改其中一份键变，改目录里别的文件键不变，不拒算」
  FUNCTION_UNITS_BREAK=python-pattern-listing-transforms-accepted  清单里的文件名经拼接、splitext、下标改过的也当清单里的文件名，只收合模式的几份、不拒算
      「python：读 os.path.join(<根里的目录>, <glob 列出的文件名换了后缀>)（读的那一份不一定合模式）拒算 data-path-unfolded」
  FUNCTION_UNITS_BREAK=shell-find-listings-ignored  交给内联 python 的 "$名字"（名字赋成 $(find …)）不当按名字模式列的清单
      「shell：$(find <目录> -maxdepth 1 -name '*.sh' \\( -type f -o -type l \\) | sort) 交给内联 python，读 os.path.join(<根里的目录>, <清单一行的 os.path.basename>) 只收目录里合 *.sh 的几份：改其中一份键变，改目录里别的文件键不变，不拒算」
  FUNCTION_UNITS_BREAK=shell-find-listing-operators-unchecked  $(find …) 只看 -name，顶层的 -o 与别的测试一概不看（不合模式的行也进清单时照收）
      「shell：$(find <目录> -maxdepth 1 -name '*.sh' -o -type f) 交给内联 python（顶层的 -o 让不合模式的行也进清单），读 os.path.join(<根里的目录>, <清单一行的 os.path.basename>) 拒算 data-path-unfolded」
  FUNCTION_UNITS_BREAK=python-root-joins-refused  os.path.join(<根本身>, <折不出的一截>) 一律拒算（不看那一截是不是读根里的文件得来的），拷文件的源照拒算
      「python：读 os.path.join(<根本身>, <算出来的名单里的一截>)（不是读根里的文件得来的）：被判的输入，不拒算」
      「python：拷文件的源 os.path.join(<根本身>, <读进来的登记行点名的一截>)：拷副本，不拒算」
  FUNCTION_UNITS_BREAK=python-parameter-callers-unresolved  本模块里没有调用处的函数参数不在闭包里按名字找调用处：不是入口的拒算
      「python：别的模块里的函数读它参数给的文件：闭包里的调用处给仓里的常量收进键（改它键变），给入口参数的不收也不拒算」
  FUNCTION_UNITS_BREAK=python-attribute-reads-unfolded  读 self.<属性> 不照本类里给它的赋值认（落到调用方交进来的输入，不收）
      「python：读 self.<属性>：照本类方法里给它的赋值认，赋的是仓里的常量就收进键（改它键变）」
  FUNCTION_UNITS_BREAK=python-joined-tails-unanchored  os.path.join / Path(…) / a / b 拼出的字面尾巴不带前导 /，照字符串后缀收（b.tsv 收进 remove_stub.tsv）
      「python：读 os.path.join(<折不出的目录>, "b.tsv") 按整段路径比：同目录的 remove_stub.tsv 不收，改它键不变」
  FUNCTION_UNITS_BREAK=python-tail-reads-unanchored  读 <参数>/<字面尾巴> 不按调用处认，读的时候就照尾巴收仓里那一组
      「python：读 <参数>/<字面尾巴>，闭包里的调用处给的是这一趟的临时目录：仓里这一截尾巴的那一组不收，改其中一份键不变」
      「python：读 <判法入口的参数>/<字面尾巴>（被判的输入）：仓里这一截尾巴的那一组不收，改其中一份键不变」
  FUNCTION_UNITS_BREAK=python-tail-reads-samples-included  按尾巴读的那一组连门禁样本档底下的文件也收
      「python：按字面尾巴读 Manifest.toml：门禁样本档底下多一份同名文件、改它都不动键」
  FUNCTION_UNITS_BREAK=python-call-sites-whole-view  程序入口带参数剪掉的那几截里的调用照算调用处
      「python：程序入口带 --selftest：main 里走不到的那一处调用不算调用处，读 <参数>/<字面尾巴> 的那一组不收（改它键不变）」
  FUNCTION_UNITS_BREAK=python-data-reads-follow-build-outputs  读到 target 一类目录下编出来的文件照旧收成数据单元
      「python：拷 <临时目录>/target/release/<程序>，仓里 target 下有一份编出来的二进制：不收进键，改它键不变」
  FUNCTION_UNITS_BREAK=built-tool-sources-ignored  编工具程序的函数（BUILT_TOOL_SOURCES）进了闭包、起编出来的工具那一处登在 BUILT_TOOL_LAUNCHES 里，
      它编的程序的源码目录都不收
      「python：编工具程序的函数（BUILT_TOOL_SOURCES）进了闭包：它编的程序的源码目录收成数据目录，改里面一份键变、改编出来的二进制键不变」
      「shell 函数编工具、内联 python 起编出来的那份（BUILT_TOOL_SOURCES 与 BUILT_TOOL_LAUNCHES）：不登记时拒算，登记了不拒算、改工具源码里一个函数键变」
  FUNCTION_UNITS_BREAK=built-tool-launches-refused  BUILT_TOOL_LAUNCHES 登记了也照旧拒算（起编出来的工具那一处折不出）
      「shell 函数编工具、内联 python 起编出来的那份（BUILT_TOOL_SOURCES 与 BUILT_TOOL_LAUNCHES）：不登记时拒算，登记了不拒算、改工具源码里一个函数键变」
  FUNCTION_UNITS_BREAK=caller-supplied-commands-refused  CALLER_SUPPLIED_COMMANDS 登记了的调用处也照旧拒算
      「调用方登记：共用起法拿参数起子进程，登记了的调用处交的命令不拒算」
  FUNCTION_UNITS_BREAK=caller-supplied-commands-callee-wide  被调函数有一处调用登记了，它所有的调用处都当调用方交进来的（等于登在被调函数自己那一处）
      「调用方登记：同一个共用起法，没登记的调用处交的命令照拒算」
  FUNCTION_UNITS_BREAK=python-call-sites-matched-by-name  闭包里调一个函数的那几处只按名字认（ScriptClosure.call_reaches 不判）：套着的同名函数、别的模块上的同名属性调用都算
      「python：调用方交进来的命令按调用处认，套在别的函数里的同名函数的调用 run(1, 5) 不算模块级 run 的调用处：不拒算，模块级 run 真正的调用处给的 tool/child.sh 连上（改它键变）」
      「python：调用方交进来的命令按调用处认，import 进来的别的模块上的 other_tools.run(7) 不算 runner_lib.run 的调用处：不拒算，runner_lib.run 真正的调用处给的 tool/child.sh 连上（改它键变）」
  FUNCTION_UNITS_BREAK=python-partial-call-sites-ignored  functools.partial(f, …) 不算 f 的调用处：经它交的实参折不到 f 的参数上
      「python：调用方交进来的命令按调用处认，functools.partial(runner_lib.run, "tool/child.sh") 算 runner_lib.run 的一处调用：不拒算，它交的 tool/child.sh 连上（改它键变）」
  FUNCTION_UNITS_BREAK=unit-graph-text-busy-not-waited  起 unit-graph 撞上 ETXTBSY（程序文件还有人开着写）当场抛，不隔一会儿再起
      「产品行为视图：起 unit-graph 的那一刻程序文件还有人开着写（ETXTBSY），写的那一方关掉之后再起得成，不记成起不了」
  FUNCTION_UNITS_BREAK=shell-nested-commands-in-outer  外层 shell 函数照全文认命令与词：套在里面的函数的命令也算外层的，拒算落在外层名下
      「shell：外层函数里套着的函数起折不出的程序、按内层的短名登了 UNKEYED_PROGRAM_WORDS：外层只认自己那一层，外层当入口不拒算，内层起的 tool/child.sh 连上（改它键变）」
  FUNCTION_UNITS_BREAK=inline-refusal-lines-at-launch  正文出自 here-doc 的内联 python 里的拒算落在起它的那一行（word_line 找到的头一处像起 python 的行），不落在正文那一行
      「内联 python 的正文出自 here-doc（直接喂的、装进变量再交 -c 的）：拒算落在原文那一行（正文头一行加偏移），不落在起它的那一行或头一处像起 python 的行」
  FUNCTION_UNITS_BREAK=closure-round-limit-unchecked  闭包 run() 的轮数不查上限
      「闭包轮数超过上限：抛 FunctionUnitsError，点名入口与轮数」
  FUNCTION_UNITS_BREAK=closure-call-index-frozen  闭包里按名字认的调用处头一次建完就不再并
      「闭包的调用处索引增量：头一轮之后折出的程序里的调用处照样认得（它读的数据文件改了键变）」
  FUNCTION_UNITS_BREAK=data-units-process-wide  运行期读的数据文件与目录的数据单元整个进程共用（按根分），不随脚本索引；同一棵树里文件改了也认旧的
      「python：运行期 open 读的仓内文件（路径折得出）进键：改它键变」「python：读 os.path.join(<根里的目录>, <从那个目录的清单里挑的一截>) 收那个目录整棵：改目录里的一份键变」
  FUNCTION_UNITS_BREAK=judging-entry-index-not-shared  closure_of 不用并行之前父进程建好的脚本索引（每个子进程各解析一遍）
      「判法入口拒算并行：closure_of 用并行之前父进程建好的脚本索引（SHARED_SCRIPT_INDICES）」
  FUNCTION_UNITS_BREAK=python-launch-arguments-unfolded  命令参数不在调用点照实参折，只认调用方交进来的那几样（字面的不连、根外程序与返回值不认）
      「python：命令是别的函数的参数（subprocess.Popen(command)、[program, *arguments]）：照闭包里每个调用点的实参折（字面程序、根外程序、外层函数的变量、调仓内函数的返回值），不拒算」
      「python：命令参数在调用点折出根里的脚本（run_tool("pkg/tool.sh", …)）：连上那一份，改它键变」
  FUNCTION_UNITS_BREAK=python-caller-launches-refused  入口参数给的命令、-c 的串、解释器照旧拒算
      「python：判法入口的参数给的命令（-c 的串、解释器）是调用方交进来的：不拒算」
  FUNCTION_UNITS_BREAK=python-toolchain-queries-refused  [<折不出的程序>, "-V"] 照旧拒算 subprocess-command-unfolded
      「python：[<环境给的编译器>, "-V"] 是工具链询问：不拒算、算得出键，询问打的版本不进键：闭包里没有工具链单元，换一版假 rustc 键不变」
  FUNCTION_UNITS_BREAK=python-toolchain-queries-keyed  每一处工具链询问收一个工具链单元（询问打的版本进键）
      「python：[<环境给的编译器>, "-V"] 是工具链询问：不拒算、算得出键，询问打的版本不进键：闭包里没有工具链单元，换一版假 rustc 键不变」
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
  FUNCTION_UNITS_BREAK=shell-entry-arguments-unnarrowed  shell 入口（script-body）给了参数也照整份收：脚本体不剪，按第一列选行的表照整份起过连整张
      「shell 入口带参数：s/dispatch.sh 带 run，--selftest 那一支与 *) 那一支走不到（改 s/child.sh、s/tools/two.py 键不变），run 那一支走到（改 s/tools/one.py 键变）」
      「shell 入口是 <复跑表脚本> 带参数：带 E1 时别的实验的驱动函数走不到（改 s/two.sh 键不变），只带 --selftest 照整张表连（改 s/two.sh 键变）」
      「shell 入口是 <复跑表脚本>、--selftest 那一支以 exit 收尾、表与读表的变量在 fi 之后：带 --selftest 时表里的驱动函数都不连（改 s/one.sh、s/two.sh 键不变），不带参数时都连（改它们键变）」
  FUNCTION_UNITS_BREAK=shell-entry-undecided-branches-dropped  shell 入口带参数时，入口那一份里判不出真假的 if [[ … ]] 当走不到（then 那一支不收）
      「shell 入口带参数：if [[ "${1:-}" == "$SELFTEST_WORD" ]] 判不出真假，两支都收（改 then 那一支起的 s/child.sh 键变）」
  FUNCTION_UNITS_BREAK=shell-preflight-arguments-moved  preflight 放回参数那一句算动了位置参数（后面的 $1 不代）
      「shell：bash s/dispatch_preflight.sh run：preflight 放回参数那一句（set -- ${PREFLIGHT_ARGUMENTS[@]+…}）不算动了位置参数，--selftest 那一支照样剪掉（改 s/child.sh 键不变）」
  FUNCTION_UNITS_BREAK=shell-exit-branches-followed  走到的那一支以 exit 收尾，fi 之后的照收边
      「shell：bash s/dispatch_exit.sh --only：走到的那一支以 exit 收尾，fi 之后的 s/tools/two.py 不连（改它键不变）」
      「shell 入口是 <复跑表脚本>、--selftest 那一支以 exit 收尾、表与读表的变量在 fi 之后：带 --selftest 时表里的驱动函数都不连（改 s/one.sh、s/two.sh 键不变），不带参数时都连（改它们键变）」
  FUNCTION_UNITS_BREAK=shell-table-rows-unselected  按第一列选行的表整张收边，带实验号起也连别的实验的驱动函数
      「shell：bash <复跑表脚本> E1：表里别的实验的驱动函数走不到，它起的 s/two.sh 不连（改它键不变）」
      「shell 入口是 <复跑表脚本> 带参数：带 E1 时别的实验的驱动函数走不到（改 s/two.sh 键不变），只带 --selftest 照整张表连（改 s/two.sh 键变）」
  FUNCTION_UNITS_BREAK=shell-table-row-items-unmatched  --item 的行项（<实验号>.<行序号>）照实验号拿整个词去比第一列，对不上任何一行，点到的那一行的驱动也不连
      「shell 入口是 <复跑表脚本> 带 --item E1.2：E1.2 那一行的驱动连上（改 s/one_sweep.sh 键变），E1.1 那一行与 E2 的驱动走不到（改 s/one_grid.sh、s/two.sh 键不变）」
  FUNCTION_UNITS_BREAK=shell-table-launch-without-assignment-counted  照这组词剪过的脚本体里表的赋值行一行都不剩的起法照旧记成一次起法（只带选项的连整张表）
      「shell 入口是 <复跑表脚本> 只带 --selftest、走到的那一支以 exit 收尾、表在 fi 之后才赋值：这次起法不算整张表，读表只照自证里 bash <复跑表脚本> E1 那一次连（改 s/one.sh 键变、改 s/two.sh 键不变）」
  FUNCTION_UNITS_BREAK=shell-tree-copy-launch-words-dropped  脚本路径照根里同一相对路径认出来的起法（"$<目录>/<相对路径>"、cd 到折不出的目录之后的相对路径）不记参数
      「shell：cd 到折不出的目录（仓副本）之后 bash <复跑表脚本> E1：参数照记、照 E1 剪，表里别的实验的驱动函数走不到（改 s/two.sh 键不变），E1 那一行的连上（改 s/one.sh 键变）」
  FUNCTION_UNITS_BREAK=shell-row-driver-mentions-linked  只按实验号起的复跑表脚本，表行之外写成 @<名>、<名>() 的驱动函数照按词连
      「python：带实验号起拷进临时目录、表换成桩的副本：只被真表点名的驱动函数不进闭包（表行之外写成 @<名>、<名>() 的也不连），副本里留下的框架函数与它读的变量进闭包」
  FUNCTION_UNITS_BREAK=shell-row-driver-plain-words-dropped  表行之外写成不带 @ 的词的驱动函数也不连（命令位置上的、放进变量再起的都走不到）
      「python：带实验号起表换成桩的副本：脚本体在命令位置上调的驱动函数照连（改它起的 s/called.sh 键变）」
      「python：带实验号起表换成桩的副本：写成不带 @ 的词的驱动函数照连（改它起的 s/plain_word.sh 键变）」
  FUNCTION_UNITS_BREAK=shell-row-driver-whole-launch-dropped  脚本整份起过时表行里的驱动函数也不连
      「python：不带实验号起表换成桩的副本：认不出表换没换，真表里的驱动函数照连（改它起的 s/row_only.sh 键变）」
      「shell：同一个闭包里 <复跑表脚本> 既按实验号起、又有一处起法折不出：读表连整张，别的实验的驱动照连（改 s/two.sh 键变）」
      「shell：入口就是 <复跑表脚本>：读表连整张（改 s/two.sh 键变）」
      「shell 入口是 <复跑表脚本> 带参数：带 E1 时别的实验的驱动函数走不到（改 s/two.sh 键不变），只带 --selftest 照整张表连（改 s/two.sh 键变）」
  FUNCTION_UNITS_BREAK=shell-row-driver-selected-rows-dropped  点名的实验那几行也写到的驱动函数，只要别的实验那几行写到就不连
      「python：带实验号起表换成桩的副本，那个实验号在真表里有行：那几行的驱动函数照连，别的实验那几行也写到它也算（改它起的 s/shared.sh 键变）」
  FUNCTION_UNITS_BREAK=python-shell-launch-arguments-unfolded  python 起 shell 的字面参数不带过去（shell_launch_words 与程序入口带参数两条路都不带，脚本体整份收）
      「python：["bash", <脚本>, *<调用处给的字面列表>] 起 shell：脚本体照字面参数剪（改走不到的 s/child.sh 键不变）」
  FUNCTION_UNITS_BREAK=excluded-files-followed  按角色排除的文件只排单元，边照走（它们读的数据、起的程序、调到的别的文件的单元照收）
      「排除文件：只经它读到的数据不进键（只改判法文件一行注释键不变）」「排除文件：只经它起的程序不进键（改那个程序键不变）」
      「命令行 closure --judging 照 admission.py 的 not_judging_files_of_entry 排除：只经 scripts/gate/lib_function_units.py 起到的 pkg/only_via_role.py 与它自己都不进闭包、别的路起的 pkg/direct.py 照进；不带 --judging 三份都进；入口就是排除文件的不排」
  FUNCTION_UNITS_BREAK=not-judging-files-drifted  抄录的 NOT_JUDGING_FILES 少一份，与 lib_verdict_library.py 那一份不同
      「真仓：lib_function_units.py 抄录的 NOT_JUDGING_FILES 与 lib_verdict_library.py 那一份相同」
  FUNCTION_UNITS_BREAK=closure-judging-exclusions-dropped  命令行 closure / key 带 --judging 也什么都不排（照不带的口径收）
      「命令行 closure --judging 照 admission.py 的 not_judging_files_of_entry 排除：只经 scripts/gate/lib_function_units.py 起到的 pkg/only_via_role.py 与它自己都不进闭包、别的路起的 pkg/direct.py 照进；不带 --judging 三份都进；入口就是排除文件的不排」
  FUNCTION_UNITS_BREAK=shell-deferred-heredocs-unmoved  写在 if 条件上的 here-doc 不挪回去：declare -f 把正文打在 then 那一串头一条命令之后，那条命令被当正文、正文解不开
      「shell：if python3 - <<'PY' 的 here-doc 照内联脚本切（declare -f 把正文打在 then 那一串头一条命令之后），连到它 import 的仓内模块」
      「shell：if python3 - <<'PY' 之后 then 那一串的头一条命令不算 here-doc 正文，它起的 s/child.sh 连上（改它键变）」
  FUNCTION_UNITS_BREAK=python-text-pieces-unfolded  f 串里的 {sys.executable}、{整数常量} 与 "<分隔>".join(<字面元组>) 折不出
      「python：bash -c f'"{sys.executable}" <脚本>' 折得出，连到那份脚本（改它键变）」
      「python：bash -c 的串里 f 串的 {<整数常量>} 与 " ".join(<字面元组>) 折得出，串里起的脚本连上（改它键变）」
  FUNCTION_UNITS_BREAK=shell-open-tail-unnarrowed  起法里有一个词折不出，那份脚本体就整份收（字面词也不代）
      「shell：bash s/dispatch.sh run "$@"：头一个词字面，--selftest 那一支走不到（改 s/child.sh 键不变）」
      「python：["bash", <脚本>, "run", *<折不出的列表>]：头一个词字面，--selftest 那一支走不到（改 s/child.sh 键不变）」
  FUNCTION_UNITS_BREAK=launched-programs-refused  LAUNCHED_REPOSITORY_PROGRAMS 登记了也拒算
      「登记的被起程序（夹具）：登记了的两处（路径折不出的、路径折得出而根里没有的）不拒算、被起的那一份整份连上（改它键变），不登记时都拒算」
      「登记的被起程序（夹具，shell）：source 的路径折不出，登记了连到那一份（改它键变），不登记拒算」
  FUNCTION_UNITS_BREAK=shell-process-substitution-splits-command  进程替换的左括号照旧把外面那条命令切断
      「shell：python3 - <(命令) "$y" <<'PY'：进程替换不切断外面那条命令，here-doc 照旧是 python3 的脚本，不拒算」
  FUNCTION_UNITS_BREAK=shell-refusal-lines-by-first-mention  拒算行照旧只照解释器名、照头一处提到那个词的行找
      「shell：拒算落在真起它的那一行（bash -c "$词" 那一行、命令位置上折不出的词跟在 && 之后的那一行），不落在头一处提到 bash 或那个词的行」
  FUNCTION_UNITS_BREAK=fold-limit-shell-paths-skipped  shell 起脚本、起命令、内联脚本那几处取值超过 FOLD_LIMIT 种的，照折不出的那一路认（照根里同一相对路径那一份猜，拒算照旧的形态）
      「shell：bash "$dir/tools/run.sh"，dir 取遍 for 里的 65 个目录（取值超过 FOLD_LIMIT 种）：不照根里同一相对路径那一份猜，照尾巴整组收，改 s/a07/tools/run.sh 键变」
  FUNCTION_UNITS_BREAK=shell-inline-quoted-words-unfolded  bash -c 的串里 $(printf '%q' <词>) 不认（quoted_word_inline_texts 一律交 None），整个折不出照旧拒算 inline-script-not-literal
      「shell：bash -c "touch $(printf '%q' "$work/ran"); bash s/child.sh"（work 折不出）：printf %q 那一处在内联正文里是一个取值折不出的词，正文照切，不拒算，连到 s/child.sh（改它键变）」
  FUNCTION_UNITS_BREAK=shell-case-subjects-launched  case <词> in 照命令认（剪过的正文里 case bash in 的 in 当成 bash 起的脚本）
      「shell：bash s/dispatch.sh bash：剪过的正文里 case "$1" in 代成 case bash in，in 不当成 bash 起的脚本：不拒算」
  FUNCTION_UNITS_BREAK=shell-refusal-lines-by-substring  拒算落行（word_line）照子串找那个词（in 认得出 rejoin 里那一截）
      「shell：拒算落在真起它的那一行：词按整个词找（bash join 不落在前面 bash s/tools/joined.sh 那一行），剪过的正文里解释器是代进来的 "$1" 的落在原文 "$1" join 那一行（不落在前面 echo join 那一行）」
  FUNCTION_UNITS_BREAK=shell-refusal-lines-positional-unmatched  拒算落行不认「解释器位置上是位置参数、后面紧跟这个词」那一层
      「shell：拒算落在真起它的那一行：词按整个词找（bash join 不落在前面 bash s/tools/joined.sh 那一行），剪过的正文里解释器是代进来的 "$1" 的落在原文 "$1" join 那一行（不落在前面 echo join 那一行）」
  FUNCTION_UNITS_BREAK=shell-prefix-assignments-unscoped  命令前的 X=v <函数> 照普通赋值算，哪个函数读 X 都算（reader_scoped 不去掉走不到的）
      「shell：命令前的 tool=<折不出> plan 只管 plan 那一次调用：plan 走不到的 merge 里 bash "$tool" 照别处的赋值折，连到 s/child.sh（改它键变）」
  FUNCTION_UNITS_BREAK=shell-temporary-sites-folded  命令前赋成这一趟临时目录底下路径的那一处照折（reader_scoped 不去掉 temporary_word 认出的）
      「shell：命令前的 tool="$work/fake.sh" plan（work 是 mktemp -d）管得到 plan 里的 bash "$tool"，但起的是这一趟自己写的临时文件：这一处赋值不折，照别处的赋值连到 s/child.sh（改它键变）」
  FUNCTION_UNITS_BREAK=shell-env-array-assignments-kept  env 起头的数组里 NAME=… 照旧当命令词
      「shell：env 起头的数组里 NAME="$值" 是 env 的赋值，"${数组[@]}" "$@" 不拒算」
  FUNCTION_UNITS_BREAK=shell-process-substitution-redirects-dropped  进程替换括号里的命令不带它们自己的重定向记录
      「shell：< <(python3 - "$x" <<'PY' …) 进程替换里的 here-doc 照旧是 python3 的脚本，不拒算」
  FUNCTION_UNITS_BREAK=program-words-refused  UNKEYED_PROGRAM_WORDS 登记了也拒算
      「不拒算的程序词（夹具）：登记了的 shell 词与 python 词不拒算，同一份文件里没登记的一处照拒算，不登记时都拒算」
  FUNCTION_UNITS_BREAK=registered-functions-short-names  按（文件, 函数）登记的几张表里 python 单元只认限定名的最后一段（同一份文件里别的类的同名方法共用一行登记）
      「不拒算的程序词按限定名认（夹具）：登了 TripLauncher.run 的词不拒算，同一份文件里 OtherLauncher.run 起折不出的命令照拒算」
  FUNCTION_UNITS_BREAK=shell-registered-missing-paths-refused  登了 UNKEYED_PROGRAM_WORDS 的 shell 路径词折得出、落在根外或指到不在的文件时照旧拒算
      「shell：登了 UNKEYED_PROGRAM_WORDS 的路径词 "{tree}/…" 折得出、指到不在的文件：不拒算，同一个函数里别的起法照连（改 tool/child.sh 键变）；不登记照拒算 shell-path-missing」
  FUNCTION_UNITS_BREAK=registered-words-drifted  登记表多一行真仓没有的词
      「真仓：UNKEYED_PROGRAM_WORDS、REGISTRY_COMMAND_TEXTS、SPEC_PATH_ARGUMENTS、TAIL_READ_SCOPES 登记的文件、函数、词与点名的文件都还在」
  FUNCTION_UNITS_BREAK=python-registry-command-texts-refused  bash -c 的串来自登记表也照旧拒算
      「python：bash -c 的串来自登记表：登记表那一列的命令照内联 bash 认，改它起的脚本键变」
      「python：bash -c 的串来自登记表：登记表收成数据单元，改表里一行键变」
  FUNCTION_UNITS_BREAK=spec-path-arguments-refused  登记了实际载入的文件也照旧拒算 spec-path-unfolded
      「python：spec_from_file_location 的路径折不出而登记了实际载入的文件：不拒算、连到那一份（改它键变）」
  FUNCTION_UNITS_BREAK=spec-path-tuple-first-only  登记值是几份文件的元组时只连头一份
      「python：spec_from_file_location 的路径折不出而登记了实际载入的文件：不拒算、连到那一份（改它键变）」
  FUNCTION_UNITS_BREAK=tail-read-scopes-ignored  TAIL_READ_SCOPES 登记了也照全仓同名的那一组收
      「python：按字面尾巴读而登记了前缀（TAIL_READ_SCOPES）：前缀外多一份同名文件键不变、前缀里改一份键变；登记成调用方输入的那一处一份都不收」
      「真仓：TAIL_READ_SCOPES 每一行那一处读还在、算键时问到的就是这一行，收进来的文件都在登记的前缀下」
      「真仓：按尾巴整组收、没登记 TAIL_READ_SCOPES 的读法，收进来的文件里没有落在 records/ 或 research/ 下的（夹具里没登记的一处判得出来、登记之后不判）」
  FUNCTION_UNITS_BREAK=tail-reads-under-records-unrecorded  按尾巴读而没登记的那几处收进 records/、research/ 下的文件不记进 UNSCOPED_TAIL_READS
      「真仓：按尾巴整组收、没登记 TAIL_READ_SCOPES 的读法，收进来的文件里没有落在 records/ 或 research/ 下的（夹具里没登记的一处判得出来、登记之后不判）」
  FUNCTION_UNITS_BREAK=tail-read-scopes-copies-unscoped  「<折不出的目录>/<仓里的相对路径>」那一种（tree_copy_file）不看 TAIL_READ_SCOPES，照根里那一份收
      「python：按字面尾巴读而登记了前缀（TAIL_READ_SCOPES）：前缀外多一份同名文件键不变、前缀里改一份键变；登记成调用方输入的那一处一份都不收」
  FUNCTION_UNITS_BREAK=tail-read-scopes-drifted  TAIL_READ_SCOPES 多一行真仓那个函数里没有的表达式
      「真仓：TAIL_READ_SCOPES 每一行那一处读还在、算键时问到的就是这一行，收进来的文件都在登记的前缀下」
  FUNCTION_UNITS_BREAK=tail-read-scopes-inline-unread  核 TAIL_READ_SCOPES 时只认 .py 文件里的函数：shell 脚本起的内联 python 正文、模块级语句里登记的那几行找不到
      「真仓：TAIL_READ_SCOPES 每一行那一处读还在、算键时问到的就是这一行，收进来的文件都在登记的前缀下」
  FUNCTION_UNITS_BREAK=tail-read-scopes-root-ignored  TAIL_READ_SCOPES 前缀一格的 "."（被判那棵树根上的那一份）当目录前缀比，根上那一份也不收
      「python：按字面尾巴读而前缀登了 .（TAIL_READ_SCOPES 里被判那棵树根上的那一份）：根上那一份改了键变，research/ 下多一份、改别处同名的一份键不变；登空元组时改根上那一份键不变」
  FUNCTION_UNITS_BREAK=python-getattr-names-unfolded  getattr 的名字是变量就拒算，取遍字面值的也拒算
      「python：getattr(<对象>, <推导式变量>)，变量取遍模块级字典字面量的值里的字面名：不拒算」
  FUNCTION_UNITS_BREAK=shell-function-outputs-unfolded  $(函数 <字面词>) 折不出
      「shell：函数里 python3 "$@"，调用处的脚本是 "$(只打一个词的函数 <字面词>)/<文件>"：照函数打的词折，连到那一份（改它键变）」
  FUNCTION_UNITS_BREAK=shell-quoted-assignments-unread  env 后面带引号的 "X=…" 与数组里的 "X=…" 当命令词
      「shell：env "X=$y" nice -n 19 bash <脚本>：带引号的赋值不算命令词，连到那份脚本（改它键变）」
      「shell：env ${e[@]+"${e[@]}"} bash <脚本>，e=("X=$(f)")：数组里的赋值不算命令词，连到那份脚本（改它键变）」
  FUNCTION_UNITS_BREAK=python-local-names-resolved  函数里读局部量的名字照按名字连到模块里的同名单元
      「python：参数、局部赋值、循环变量、套着的函数里的局部量与模块里的函数同名：不连（改那个函数键不变）」
  FUNCTION_UNITS_BREAK=python-module-body-commands-unfolded  模块体里循环、分支赋的名字当命令时不认（照旧拒算 subprocess-command-unfolded）
      「python：模块体里循环中赋的列表与 for 取的字面列表拼成 [*a, *b]：照函数体同一条折，连到那份脚本（改它键变）」
  FUNCTION_UNITS_BREAK=shell-exported-variables-unread  同一个单元里 export 过的变量不到内联 python 的环境里
      「shell：同一个单元里 export 过的变量到内联 python 的环境里：os.environ[…] 折得出，sys.path 与它起的脚本都认（改它键变）」
      「python：拆成 shell 入口加同目录模块，拆出的模块读入口给内联 python export 的变量：os.environ[…] 照入口折，它起的脚本连上（改它键变）」
  FUNCTION_UNITS_BREAK=python-guard-names-unfolded  main-guard 里赋的名字折不出（sys.path.insert 拒算 sys-path-unfolded）
      「python：main-guard 里赋的名字给 sys.path.insert：折得出，from 那份模块连上（改它键变）」
  FUNCTION_UNITS_BREAK=stage-cell-functions-drifted  登记行里多一行拿变量当函数的 stage_cell
      「真仓：各道门禁 stage_cell 那一行第二列都是字面函数名（stage-cells.sh 起的 "$function_name" 靠写着名字的那一行进键）」
  FUNCTION_UNITS_BREAK=shell-heredoc-empty-rows-spanned  代位置参数时 here-doc 里的空行也记成要跨过去的一段（长度 0，走到那一处抛，不真转死）
      「shell：bash <脚本> run，脚本里的 here-doc 带空行：代位置参数走得出来，--selftest 那一支照样剪掉（改 s/child.sh 键不变）」
  FUNCTION_UNITS_BREAK=shell-narrowed-views-unshared  剪出同一份正文的几种字面参数各有一个视图（解析结果留几份）
      「shell：几种字面参数剪出同一份正文的共用一个视图，剪出别的正文的是另一个」
  FUNCTION_UNITS_BREAK=rust-own-program-restart-refused  路径原样交给 Command::new 也不认，折不出测试名的照旧拒算 current-exe
      「Rust：bin 的 main 起自己（路径原样交给 Command::new）：不拒算、不连边，改子进程那个角色跑的函数键变；库函数从 bin 的 main 连到的同样不拒算」
  FUNCTION_UNITS_BREAK=rust-own-program-entry-unchecked  路径原样交给 Command::new 的不看入口，一律不拒算
      「Rust：路径原样交给 Command::new 而入口不是 bin 的 main（库单测、bin 里的单测、单测调 main、按测试编的 main、bin 的辅助函数、模块里的 main）：拒算 current-exe」
  FUNCTION_UNITS_BREAK=rust-own-program-path-unchecked  路径怎么走都当原样交给 Command::new
      「Rust：入口是 bin 的 main 而路径改过或流向别处（同目录的程序、父目录、失败时交回别的路径、当参数交给别的程序、交给辅助函数、名字绑成别的路径、--exact 点名不存在的）：拒算 current-exe」
  FUNCTION_UNITS_BREAK=rust-own-program-uses-any-block  名字的用法落在哪一层花括号里都不看（进更深的块、出了 let 所在的块的也算起自己）
      「Rust：路径绑给 let 名字，用法进了更深的块或出了 let 所在的块（起子进程失败的闭包块里显示、if 块里起、内层块的 let 在块外用）：拒算 current-exe」
  FUNCTION_UNITS_BREAK=rust-own-program-divergence-any-block  「一定不交值」碰到以 } 收尾的表达式一律往块里看，不看块前面是不是闭包参数或 =>
      「Rust：取路径失败时的闭包以 if … else { panic!() } 收尾（块前面是 else）：拒算 current-exe」
  FUNCTION_UNITS_BREAK=rust-extern-listed-calls-refused  extern 块只声明名单里的符号也拒算 extern-block
      「Rust：extern "C" 块只声明名单里的符号（#[link_name] 或函数名）：不拒算，改调用方键变」
  FUNCTION_UNITS_BREAK=rust-extern-any-block-keyed  extern 块一律不拒算
      「Rust：extern 块有名单外的符号、#[link]、别的 ABI、static、别的属性：拒算 extern-block」
  FUNCTION_UNITS_BREAK=rust-extern-link-first-attribute-only  属性里的 link 只认紧挨 #[ 的（#[cfg_attr(条件, link(…))] 里的不认）
      「Rust：extern 块外属性里任何位置有 link（#[cfg_attr(条件, link(…))]）：拒算 extern-block」
  FUNCTION_UNITS_BREAK=rust-extern-other-block-link-unchecked  不看同一个目标里别的 extern 块的 link
      「Rust：同一个目标里另一个 extern 块带 #[link]（那个块不在闭包里）：名单里的块也拒算 extern-block」
      「Rust：同一个包或依赖的测试目标里有 #[link] 的 extern 块（那个目标不是入口的目标）：名单里的块也拒算 extern-block」
      「Rust：顺依赖（含开发依赖、构建依赖）够到的包带 #[link]、构建脚本打 rustc-link-lib、构建依赖的库代码打 cargo:rustc-link-lib 指令：名单里的块也拒算 extern-block」
  FUNCTION_UNITS_BREAK=rust-extern-build-script-link-unchecked  不看包的构建脚本里有没有 cargo:rustc-link-lib 一类指令
      「Rust：包的构建脚本写了 rustc-link-lib：名单里的块也拒算 extern-block；构建脚本没写的不拒算」
      「Rust：顺依赖（含开发依赖、构建依赖）够到的包带 #[link]、构建脚本打 rustc-link-lib、构建依赖的库代码打 cargo:rustc-link-lib 指令：名单里的块也拒算 extern-block」
      「Rust：构建脚本与构建依赖的库代码打 cargo:rustc-link-arg=-l…、cargo:rustc-flags=-l …、cargo::rustc-link-lib=…（两个冒号）：名单里的块也拒算 extern-block」
  FUNCTION_UNITS_BREAK=rust-extern-link-of-other-targets-unchecked  extern 块的 link 只看入口所在的目标（别的包只看库目标），不看同一个包与依赖的测试、示例等别的目标
      「Rust：同一个包或依赖的测试目标里有 #[link] 的 extern 块（那个目标不是入口的目标）：名单里的块也拒算 extern-block」
  FUNCTION_UNITS_BREAK=rust-extern-link-of-dependencies-unchecked  只看入口所在的包，不看顺依赖够到的包
      「Rust：顺依赖（含开发依赖、构建依赖）够到的包带 #[link]、构建脚本打 rustc-link-lib、构建依赖的库代码打 cargo:rustc-link-lib 指令：名单里的块也拒算 extern-block」
      「Rust：构建脚本与构建依赖的库代码打 cargo:rustc-link-arg=-l…、cargo:rustc-flags=-l …、cargo::rustc-link-lib=…（两个冒号）：名单里的块也拒算 extern-block」
  FUNCTION_UNITS_BREAK=rust-extern-link-arguments-unchecked  构建脚本与构建依赖只认 rustc-link-lib，不认 rustc-link-arg、rustc-flags
      「Rust：构建脚本与构建依赖的库代码打 cargo:rustc-link-arg=-l…、cargo:rustc-flags=-l …、cargo::rustc-link-lib=…（两个冒号）：名单里的块也拒算 extern-block」
  FUNCTION_UNITS_BREAK=rust-extern-two-colon-instructions-ignored  构建脚本与构建依赖的库代码只认 cargo: 一个冒号的指令，不认 cargo::
      「Rust：构建脚本与构建依赖的库代码打 cargo:rustc-link-arg=-l…、cargo:rustc-flags=-l …、cargo::rustc-link-lib=…（两个冒号）：名单里的块也拒算 extern-block」
  FUNCTION_UNITS_BREAK=rust-extern-dependency-dev-dependencies-unfollowed  顺依赖往下时不跟别的包（不是入口所在的包与块所在的包）的开发依赖
      「Rust：顺依赖（含开发依赖、构建依赖）够到的包带 #[link]、构建脚本打 rustc-link-lib、构建依赖的库代码打 cargo:rustc-link-lib 指令：名单里的块也拒算 extern-block」
  FUNCTION_UNITS_BREAK=rust-extern-instruction-words-bare  构建脚本与库代码里指令名的裸词（注释里的、没有 cargo: 前缀的）也算打了指令
      「Rust：只有带 cargo: 或 cargo:: 前缀的字面串算打指令：构建脚本里的注释与裸词、不是构建依赖的库里的字面串都不算，名单里的块不拒算」
  FUNCTION_UNITS_BREAK=rust-extern-library-literals-of-any-package  别的包（不是入口所在的包）的库代码里有指令字面串都算，不只看被当构建依赖的
      「Rust：只有带 cargo: 或 cargo:: 前缀的字面串算打指令：构建脚本里的注释与裸词、不是构建依赖的库里的字面串都不算，名单里的块不拒算」
  FUNCTION_UNITS_BREAK=rust-extern-external-dependencies-unchecked  有工作区外的依赖（cc、serde 这一类，普通、开发、构建任一种）时不拒算
      「Rust：块所在的包、入口所在的包与它们顺依赖够到的包有工作区外的依赖（普通、开发、构建任一种，cc 这一类）：名单里的块也拒算 extern-block；依赖都在仓里的、没有依赖关系的包里的不拒算」
  FUNCTION_UNITS_BREAK=rust-extern-system-library-list-short  放行的 C 库调用名单少最后一项（fdatasync），与 Rust 源码里那个常量不同
      「Rust：extern 块放行的 C 库调用名单与 crates/singlefs-unit-graph 的 SYSTEM_LIBRARY_CALLS_ALLOWED_IN_EXTERN_BLOCKS 逐项相同（读那份源码比）」
  FUNCTION_UNITS_BREAK=shell-nested-functions-by-any-word  套在函数里的函数按词连，外面哪个单元写到同一个词都算
      「shell：外面的单元里一个词与套在别的函数里的函数同名：不连（改它起的 s/child.sh 键不变）」
  FUNCTION_UNITS_BREAK=shell-python-launch-arguments-unfolded  shell 起 python 的词不带过去，那份 python 整份收
      「shell：python3 <脚本> alpha "$1"：--selftest 那一支走不到（改它调的函数键不变）」
  FUNCTION_UNITS_BREAK=shell-pwd-unfolded  shell 的 $PWD 不折（落到「$<折不出的目录>/<尾巴>」那一条，仓里同名的整组连上）
      「shell：$PWD 折出来只连折出的那一份，别处同名的不连（改它键不变）」
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
      「python：同一程序入口带两组参数：只改另一组参数才走到的函数，这一组键不变、改这一组走到的键变；起它的脚本把参数带过去照样剪」「python：入口没带参数，它起的 python 带字面参数：被起的那份照它们剪（改走不到的函数键不变）」
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
  FUNCTION_UNITS_BREAK=rust-concatenated-includes-skipped  concat!(…) 整段跳过，里面的 include_str! / include_bytes! 不收读
      「concat!(include_str!(…), include_str!(…)) 里每个 include_str! 各切一个数据单元：dump 按读它的那一行各列一行 data，改其中 body.txt 那一份入口的键变」
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
      「Rust：同一函数里起 Command 的 current_exe() 照旧按起自己判：折得出测试名的连边，折不出的（没有 --exact、路径先存进变量再交 Command::new、路径改过、点名不存在的）拒算 current-exe」
  FUNCTION_UNITS_BREAK=rust-current-exe-constants-same-file-only  常量名只在起子进程那个单元所在的那一份文件里找（常量定义在同包别的文件里的拒算 current-exe）
      「Rust：current_exe() 的测试名里的常量定义在同包另一份文件里，--exact 后面用它：连到那条 #[test]、不拒算，改它的函数体键变、改别的测试键不变；同名常量在同包两份文件里值不同的照旧拒算 current-exe」
  FUNCTION_UNITS_BREAK=closure-directory-lines-span-files  命令行 closure 与 key 把数据目录单元打成它自己那一行（<目录>:1-<文件数>）
      「命令行 closure：数据目录单元按读它的那一处列（data-directory 加目录、读它的文件与行、目录指纹），不打成 <目录>:1-<文件数>」
  FUNCTION_UNITS_BREAK=rust-lex-cache-unverified  Rust 词法单元的磁盘缓存文件名只取文本长度（缓存不核内容）
      「Rust 词法单元的磁盘缓存：同一份文本第二次取缓存、与现切的逐项相同；改一个字符（长度不变）不沿用、切出来的随文本变；缓存文件坏了照常现切」
  FUNCTION_UNITS_BREAK=rust-lex-cache-namespace-lexer-only  词法缓存的目录名只从 rust_lex 起收（读写缓存的两个函数与格式号不在里面）
      「Rust 词法单元缓存的目录名：改格式号 RUST_LEX_CACHE_FORM、改一个词种名常量（IDENT 的值）、改 load_rust_lex 的读法，目录名各换」
  FUNCTION_UNITS_BREAK=rust-lex-cache-namespace-whole-module  词法缓存的目录名把这份模块的每个函数都当起点收
      「Rust 词法单元缓存的目录名：改 rust_lex 里一行注释、这份库别处一行注释、一个不在 rust_lex_cached 闭包里的函数（match_brackets），目录名都不换」
  FUNCTION_UNITS_BREAK=rust-lex-cache-namespace-module-placement-read  词法缓存的目录名把 MODULE_PLACEMENT_NAMES（cross_run_cache_directory 读的 __file__）照取值
      「Rust 词法单元缓存的目录名：改 rust_lex 里一行注释、这份库别处一行注释、一个不在 rust_lex_cached 闭包里的函数（match_brackets），目录名都不换」
  FUNCTION_UNITS_BREAK=selftest-fixtures-loaded-at-import  在真仓拷贝上给这份库加一句模块级赋值点名 load_selftest_fixtures（import 时就载入自证夹具那一份）
      「真仓：lib_function_units_selftest_fixtures.py 的单元不进 rust_lex_cached 的闭包（算键库 import 时执行的语句都在里面），那一份末尾加一句常量键不变」
  FUNCTION_UNITS_BREAK=selftest-fixtures-runtime-name-moved  在读进来的原文上把运行期读的 ROW_KEYED_TABLES 从这份库挪进自证夹具那一份
      「真仓：lib_function_units_selftest_fixtures.py 定义的名字，算键库运行期那一边（import 时执行的语句、selftest 够不到的函数与类、别的脚本经属性调的函数）一个都不读，算键库自己也不再定义」
  FUNCTION_UNITS_BREAK=rust-lex-cache-namespace-reads-own-file  在真仓拷贝上给 rust_lex_cache_namespace 加回一句读这份库自己原文的
      「真仓：rust_lex_cached 的闭包里没有算键库整份的 source-text 与 data-file，读写缓存的两个函数、格式号与词种名常量在里面；只改算键库一行注释它的键不变，改 load_rust_lex 键变」
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
  E171 重跑入口的闭包（自证 E171_CLOSURE_CELL，真仓）：
  FUNCTION_UNITS_BREAK=e171-selftest-table-at-module-level  在拷贝上给 reuse_library.py 加一张模块级的自证格表（import 时执行的赋值把格函数拖进闭包）
      「真仓：E171 重跑的两份入口脚本（名单核对 list_check.py、变异驱动 mutation_driver.py）的闭包都不拒算；没被单测点名的自证格（reuse_library 的 cell_rp1_layers_end_to_end、」
  FUNCTION_UNITS_BREAK=e171-admission-selftest-at-module-level  在拷贝上给 admission.py 加一张模块级的表点名 run_selftest（import 时执行的赋值把自证拖进闭包）
      「真仓：E171 重跑的两份入口脚本（名单核对 list_check.py、变异驱动 mutation_driver.py）的闭包都不拒算；没被单测点名的自证格（reuse_library 的 cell_rp1_layers_end_to_end、」
  FUNCTION_UNITS_BREAK=e171-list-check-imports-reuse-library  在拷贝上给 list_check.py 加一条模块级 import reuse_library（名单核对的闭包连上复用库）
      「真仓：E171 重跑的两份入口脚本（名单核对 list_check.py、变异驱动 mutation_driver.py）的闭包都不拒算；没被单测点名的自证格（reuse_library 的 cell_rp1_layers_end_to_end、」
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
  FUNCTION_UNITS_BREAK=call-edge-exclusions-ignored  EXCLUDED_CALL_EDGES 登记的调用边照连
      「python：EXCLUDED_CALL_EDGES 登记的调用边不走：只经它读的数据文件改了键不变，同一个单元调的另一个函数读的改了键变」
      「真仓：unit_graph_directory 的闭包里没有 built_unit_graph、build_unit_graph、unit_graph_inputs_digest；Cargo.lock 里头一个包的版本、.claude/heavy-thread-default、crates/singlefs-unit-graph/src 里一份文件各改一处，键都不变」
  FUNCTION_UNITS_BREAK=unit-graph-inputs-lock-undeclared  自证 UNIT_GRAPH_INPUTS_CELL 经 scripts/hosts/lib_repository_copy.py 拷真仓时不声明 Cargo.lock（拷出来的树里没有它）
      「真仓：unit_graph_directory 的闭包里没有 built_unit_graph、build_unit_graph、unit_graph_inputs_digest；Cargo.lock 里头一个包的版本、.claude/heavy-thread-default、crates/singlefs-unit-graph/src 里一份文件各改一处，键都不变」
  FUNCTION_UNITS_BREAK=unit-graph-failure-step-unnamed  编 unit-graph 的一步抛 OSError 照原样抛（unit_graph_step 不带步与路径，外层只报缓存目录与 errno）
      「产品行为视图：编 unit-graph 的一步抛 OSError（缓存目录的位置上是一份普通文件）：报错带上是哪一步（os.makedirs）与那个路径」
  FUNCTION_UNITS_BREAK=unit-graph-present-program-rebuilt  缓存目录里的程序只认执行位（unit_graph_program_present 不认文件在而看不到执行位的）
      「产品行为视图：缓存目录里的 unit-graph 文件在、本机看不到执行位（共享盘上别的机器刚换上）：照用，不重编」
  FUNCTION_UNITS_BREAK=unit-graph-directories-not-shared  编 unit-graph 的几处建目录不带 exist_ok（shared_directory 撞上已在的抛 EEXIST）
      「产品行为视图：缓存目录与这一份程序的目录已由别的进程建好：照用、换上编出来的程序，不报 EEXIST（编译换成假的）」
  FUNCTION_UNITS_BREAK=unit-graph-digest-skips-lock-file  unit-graph 程序的缓存目录名（unit_graph_inputs_digest）不算 Cargo.lock
      「unit-graph 程序的缓存目录名：Cargo.lock、.claude/heavy-thread-default、crates/singlefs-unit-graph/src 里一份文件各改一处，目录名各换」
  FUNCTION_UNITS_BREAK=root-ignored-scripts-linked  仓根被 .gitignore 挡着的私有文件照连（private_root_file 一律交 False）
      「仓根被 .gitignore 挡着的私有脚本：列仓根整层起的 multi-host-quiesce-*.sh 不进闭包、不拒算；同一层没挡着、同样起仓外路径的那一份照连、照拒算 shell-path-outside-root」
  FUNCTION_UNITS_BREAK=excluded-edge-exit-unmeasured  改坏被切的程序之后不重跑包装，退出码照改坏之前记
      「不走的子进程边（夹具）：切了边的程序改坏后包装退出码变了而键不变的必红（对照 leak.py 必须认出），吞掉退出码的 quiet.py 不红」
  FUNCTION_UNITS_BREAK=admission-edge-admitted-sample  准入异常那几个假准入里多跑一个放行的（退 0、打带 sha256 的 E7INPUT 行），计划往下走、起编译
      「不走的子进程边（真仓 E183 准入）：准入这条路上的异常（崩、被杀、退 77、退 3、退 0 不打行、打别的行）只让计划那一步退非 0、不起编译、不写计划，读数出不来」
  FUNCTION_UNITS_BREAK=admission-edge-header-unstripped  判法读产物前不去掉 E7INPUT 行
      「不走的子进程边（真仓 E183 准入）：准入往标准输出打的只有 E7INPUT 行（键算出、拒算两样），判法读产物前去掉这几行之后读数一行不差」
  FUNCTION_UNITS_BREAK=admission-edge-reach-unmeasured  不量 admission.py 当程序起的闭包（当它是空的）
      「不走的子进程边（真仓 E183 准入）：切边前后少掉的单元都在 admission.py 当程序起的闭包里（只经这条边可达），且确实少了 admission.py」
  FUNCTION_UNITS_BREAK=card-lease-client-words-granted-sample  领卡那一回的异常样本里多跑一个领到了的（退 0、打卡串），装置照起
      「不拒算的程序词（真仓领卡客户端）：device_card_lease.py 的 take_card 那一回的异常（抛异常、被 SIGKILL 杀、退 3、退 0 不打卡串、退 0 只打空白）抛 CardLeaseRefused、装置不起、标准输出空着；return_card 那一回的异常（抛异常、被 SIGKILL 杀、退 1、退 3）装置照起一回、退出码与标准输出就是装置的」
  FUNCTION_UNITS_BREAK=card-lease-client-words-return-output-kept  device_card_lease.py 的 return_card 不接管客户端的标准输出（照旧接在调用方的标准输出上）
      「不拒算的程序词（真仓领卡客户端）：device_card_lease.py 的 return_card 那一回客户端往标准输出打的行转到标准错误，调用方的标准输出与装置打的读数一行不差」
  FUNCTION_UNITS_BREAK=card-lease-client-words-other-launches-unchecked  不核别处起领卡客户端（调用方另起的、device_card_lease.py 里这两个函数之外起子进程的），只看登记、这两个函数起的头一个词与调用点
      「不拒算的程序词（真仓领卡客户端）：row_driver.py 与 shared_steps.py 领卡、还卡都经 device_card_lease.py 的 take_card、return_card，不经别处起领卡客户端；device_card_lease.py 里起子进程的只有这两个函数，起的头一个词是登记的 client」
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
  自证自己的并行（gather_forked）：
  FUNCTION_UNITS_BREAK=selftest-lost-results-unchecked  收回的条数不与派出去的比：没收回结果的那几条不点名、不算红
      「自证并行：每条在自己 fork 出来的进程里跑，收回的按派出去的次序排、各带自己的输出；没收回结果的（退出码 0 却没写结果、退出码 3）逐条点名，收回的条数与派出去的对不上不吃掉」
  拆分族（入口加被 source 的库、入口加同目录模块）那几条：
  FUNCTION_UNITS_BREAK=shell-sourced-values-folded-in-reader  被 source 的库读别的文件里赋的变量时，赋值词在读它的库的单元里折（BASH_SOURCE 折成库自己）
      「shell：被 source 的库里的函数读入口赋的 STAGE_REPOSITORY（值里有 ${BASH_SOURCE[0]}），照入口那一份折，连上它起的脚本，改那份脚本键变」
  FUNCTION_UNITS_BREAK=shell-sourced-zero-own-file  被 source 的库里的 $0 折成库自己，不是 source 它的入口
      「shell：被 source 的库里的 $0 是 source 它的入口，$(dirname "$0")/.. 折到入口往上一层，连上它起的脚本，改那份脚本键变」
  FUNCTION_UNITS_BREAK=shell-sourced-siblings-unread  一份库读的变量不到同一个入口 source 的别的库里找（SIBLING_SCRIPT 折不出、拒算 shell-path-unfolded）
      「shell：一份库读同一个入口 source 的另一份库赋的变量（SIBLING_SCRIPT），照那一份折，连上它起的脚本，改那份脚本键变」
  FUNCTION_UNITS_BREAK=shell-self-assignments-counted  NAME="$NAME" 这类赋成它自己的词照赋值算（库里一处命令前的这种写法盖住入口里真赋的值）
      「shell：被 source 的库里另一个函数在命令前写 NAME="$NAME"（原样交给那条命令），库里的函数读 NAME 照旧到入口里赋它的那一处折，起的脚本连上（改它键变）」
  FUNCTION_UNITS_BREAK=python-importer-sys-path-ignored  拆出的模块 import 的名字不照 import 它的入口插的 sys.path 解（拒算 import-unresolved）
      「python：拆出的模块自己不插 sys.path，它 import 的模块照同目录里 import 它的入口插的目录解，不拒算 import-unresolved，改那份模块键变」
  FUNCTION_UNITS_BREAK=python-import-time-bindings-followed  只因载入收进来的模块级 from x import f 也连到 f 的单元（入口再导出的函数随载入全进闭包）
      「python：模块级 from helpers import used, unused 只因载入收进来时不连 unused，没人读它，改它不作废」
  FUNCTION_UNITS_BREAK=selftest-modules-loaded-at-import  真仓拷贝上给算键库入口末尾加一句模块级 from lib_function_units_selftest_run import selftest
      「真仓：按路径载入算键库的入口（lib_mutation_units.py 的 UnitKeys 那几个方法）的闭包里没有 lib_function_units_selftest*.py 的单元、没有拒算（算键库入口 import 时不载入自证模块）」
  FUNCTION_UNITS_BREAK=judging-entry-literals-split-unfollowed  判法入口的登记不到入口再导出的同目录模块里找（拆过的几份取不到，registered_judging_entries 抛错）
      「真仓：登记的每个判法入口的闭包都不拒算（拒算的那一路判定一条都写不进库），夹具里的 globals() 查找判得出来」
  FUNCTION_UNITS_BREAK=registered-functions-drifted  登记表多一行函数已挪进拆出文件的登记（task-server/task_server.py 的 read_gpu_cards_failed）
      「真仓：按（文件, 函数）登记的几张表（UNKEYED_PROGRAM_WORDS、LAUNCHED_REPOSITORY_PROGRAMS、BUILT_TOOL_SOURCES、BUILT_TOOL_LAUNCHES、CALLER_SUPPLIED_COMMANDS、REGISTRY_COMMAND_TEXTS、TAIL_READ_SCOPES、EXCLUDED_PROGRAM_EDGES、EXCLUDED_CALL_EDGES）每一行的函数都定义在登记的文件里」
  FUNCTION_UNITS_BREAK=unit-graph-sources-top-level-only  读 crates/singlefs-unit-graph 的常量只看 src 这一层（拆进 <模块>/ 底下的找不到）
      「Rust：extern 块放行的 C 库调用名单与 crates/singlefs-unit-graph 的 SYSTEM_LIBRARY_CALLS_ALLOWED_IN_EXTERN_BLOCKS 逐项相同（读那份源码比）」
      「R-裸名：宏名单与 crates/singlefs-unit-graph 的 STANDARD_MACROS_EVALUATING_THEIR_ARGUMENTS_IN_PLACE 逐项相同（读那份源码比）」
  FUNCTION_UNITS_BREAK=selftest-fixture-names-receivers-ignored  「别的脚本经属性调的函数」不看 . 前面是不是这份库（别的模块上同名的属性、注释里的 import 行也算）
      「真仓：lib_function_units_selftest_fixtures.py 定义的名字，算键库运行期那一边（import 时执行的语句、selftest 够不到的函数与类、别的脚本经属性调的函数）一个都不读，算键库自己也不再定义」
  FUNCTION_UNITS_BREAK=shell-name-reach-context-dropped  库里按名字连进来的函数照它自己那份文件看得见的文件找函数（不认叫到它的那一份 source 的别的库）
      「shell：入口是脚本里的一个函数（source 它那几份库的脚本体不在闭包里），一份库里的函数调同一个脚本 source 的另一份库里的函数，照叫到它的那一份看得见的文件找，连上，改那个函数键变」
  FUNCTION_UNITS_BREAK=python-importer-call-sites-ignored  拆出的模块里的函数（方法）的参数只照本模块里的调用处折，import 它的同目录模块里的调用处不看
      「python：拆出的模块里按参数载入的路径（spec_from_file_location(…, path)）照 import 它的模块里调它的那几处折，连上载入的那份，改它键变」
      「python：拆成混入类的方法的参数照拆分族里调它的那几处折（几个混入类由同一份模块 import 进来拼成一个类）：按参数载入的路径（spec_from_file_location(…, path)）连上载入的那份，改它键变」
  FUNCTION_UNITS_BREAK=inline-bodies-launch-file-only  内联 python 的正文只在起它的那份文件里找出处（入口装进变量、库里起的程序：拒算落在起它的那一行、登记表按库查）
      「内联 python 的正文写在入口里（装进变量）、由被 source 的库起：拒算落在入口里正文那一行，按文件查的登记表照入口查」
  FUNCTION_UNITS_BREAK=python-importer-sites-outside-family  同目录里 import 这份模块的都算进拆分族（共用库的每个调用方交的实参都折进库里函数的参数）
      「python：库里函数的参数不照拆分族外 import 它的模块里的调用处折（同目录别的脚本交的那份程序不连进经这份库的闭包），改那份程序键不变」
  FUNCTION_UNITS_BREAK=python-split-part-environment-unread  拆出的模块不继承 shell 入口给内联 python 设的环境变量（os.environ[…] 照旧折不出、拒算）
      「python：拆成 shell 入口加同目录模块，拆出的模块读入口给内联 python export 的变量：os.environ[…] 照入口折，它起的脚本连上（改它键变）」
  FUNCTION_UNITS_BREAK=python-compiled-part-exec-allowed  模块级按部分文件 exec 照旧不认（不连也不拒算 compiled-part-exec）
      「python：模块级把读出的部分文件 compile(<源码>, <路径>, "exec") 再 exec 进自己的命名空间：exec 进来的是哪份文件的哪些名字折不出，拒算 compiled-part-exec」
  FUNCTION_UNITS_BREAK=python-module-object-exec-refused  exec 进模块对象不认（照按部分文件 exec 认：同一个单元里有文件名不是字面串的 compile 就拒算，别的不连也不拒算；
      源码折不出而文件名是字面串的那一处也就不拒算）
      「python：exec(compile(<shell 变量交进来的库正文>, <标签>, "exec"), <模块对象>.__dict__) 照载入那段正文连：按属性调到的函数起的脚本连上，改它键变」
      「python：exec(compile(open(<折得出的路径>).read(), <路径>, "exec"), <模块对象>.__dict__) 照载入那份文件连：按属性调到的函数起的脚本连上，改它键变」
      「python：exec 进模块对象的那份文件里没按属性调到的函数不连：它起的脚本改了键不变」
      「python：exec 进模块对象的源码折不出（调用方交的 "$1"）拒算 compiled-part-exec」
  FUNCTION_UNITS_BREAK=python-module-object-exec-unfolded-allowed  exec 进模块对象而源码折不出的不拒算（不连也不拒算）
      「python：exec 进模块对象的源码折不出（调用方交的 "$1"）拒算 compiled-part-exec」
  FUNCTION_UNITS_BREAK=shell-function-printed-words-single  $(函数 [<字面词…>]) 只认给了字面词、printf 只打一个词的那一种（不带参数、打几个变量的折不出，库正文拒算 compiled-part-exec）
      「shell：内联 python 的参数是 "$(不带参数的函数)"、函数体只有一条 printf '%s\n' 打两个变量：两个变量的正文逐行连起来当库正文 exec 进模块对象，按属性调到的函数起的脚本连上，改它键变」
      「shell：printf 打出的库正文里没按属性调到的函数不连：它起的脚本改了键不变」
  FUNCTION_UNITS_BREAK=shell-read-heredoc-assignments-ignored  IFS= read -r -d '' <变量> <<词 不当赋值（变量折不出，库正文拒算 compiled-part-exec）
      「shell：IFS= read -r -d '' <变量> <<词 从 here-doc 读进变量、经环境变量交进内联 python exec 进模块对象：照 here-doc 的内容折，按属性调到的函数起的脚本连上，改它键变」
      「shell：read -d '' 读进的库正文里没按属性调到的函数不连：它起的脚本改了键不变」
      「python：exec 进模块对象的库正文写在被 source 的库里（read -d '' 读进变量、经参数交进来）：正文里的拒算落在那份库里正文那一行，算在那份文件上」
  FUNCTION_UNITS_BREAK=shell-read-assignments-unrecorded  read 改写的变量（不认作 here-doc 赋值的那几种）不记折不出的赋值，别处赋的字面值照折
      「shell：先给变量赋字面空串、再 IFS= read -r -d '' 从 "$1" 那份文件读进它、经环境变量交进内联 python exec 进模块对象：read 改写的值折不出，拒算 compiled-part-exec」
  FUNCTION_UNITS_BREAK=python-module-object-loader-call-sites-ignored  exec 进模块对象的库正文里模块级函数的参数不照载入方的调用处折（按参数路径载入的拒算 spec-path-unfolded）
      「python：exec 进模块对象的库正文里模块级函数 load(path) 按参数路径 spec_from_file_location 载入，路径照载入方 <模块对象>.load(…) 那一处折：载入的那一份连上，改它键变」
      「python：载入方交给库正文里另一个函数（describe，不载入）的路径不当 load 的参数：那份文件改了键不变」
  FUNCTION_UNITS_BREAK=python-module-object-refusal-lines-at-exec  参数、环境变量交进去的库正文不找出自哪一份 here-doc（正文里的拒算一律落在 exec 那一行、算在载入方那份文件上；
      按文件查的登记表照载入方那份查，写在库正文里的 admission_run、TripLauncher.run 在登记的那份文件里找不到）
      「python：exec 进模块对象的库正文写在被 source 的库里（read -d '' 读进变量、经参数交进来）：正文里的拒算落在那份库里正文那一行，算在那份文件上」
      「真仓：按（文件, 函数）登记的几张表（UNKEYED_PROGRAM_WORDS、LAUNCHED_REPOSITORY_PROGRAMS、BUILT_TOOL_SOURCES、BUILT_TOOL_LAUNCHES、CALLER_SUPPLIED_COMMANDS、REGISTRY_COMMAND_TEXTS、TAIL_READ_SCOPES、EXCLUDED_PROGRAM_EDGES、EXCLUDED_CALL_EDGES）每一行的函数都定义在登记的文件里」
  FUNCTION_UNITS_BREAK=registered-functions-module-object-texts-unread  核登记的函数时不往 exec 进模块对象的库正文里找（写在库正文里的 admission_run、TripLauncher.run 找不到）
      「真仓：按（文件, 函数）登记的几张表（UNKEYED_PROGRAM_WORDS、LAUNCHED_REPOSITORY_PROGRAMS、BUILT_TOOL_SOURCES、BUILT_TOOL_LAUNCHES、CALLER_SUPPLIED_COMMANDS、REGISTRY_COMMAND_TEXTS、TAIL_READ_SCOPES、EXCLUDED_PROGRAM_EDGES、EXCLUDED_CALL_EDGES）每一行的函数都定义在登记的文件里」
  FUNCTION_UNITS_BREAK=test-service-code-table-row-dropped  读测试服务代码登记表时少头一行（task-server/ 没登）
      「真仓：测试服务代码登记表 scripts/gate/test-service-code-paths.tsv 与 D13（验证路线） 已定项 17 三类代码表测试服务代码那一行逐项对上（反引号里的每一项都登了、登的每一行都出自那一行、仓里的 crash_judge*.wgsl 逐份登了、登的 .wgsl 都在仓里、登的路径都在）」
  FUNCTION_UNITS_BREAK=test-service-code-split-families-unrecognized  is_test_service_code 不认拆分族（登的 .rs 拆出的子模块判成不是）
      「真仓：is_test_service_code 判 crates/singlefs-unit-graph/src 下一份 .rs、一份 GPU 判器 .wgsl、登的 .rs 拆出的子模块是测试服务代码，产品包里的一份与登的 .rs 同目录没登的一份不是」
  FUNCTION_UNITS_BREAK=test-service-code-keyed  closure_key 照出测试服务代码的 unit 行与 data 行
      「键：闭包里测试服务代码的单元不进 closure_key（按条复用的测试函数键）：改测试服务代码里的一个单元键不变，改闭包里的产品代码键变」
  FUNCTION_UNITS_BREAK=test-service-code-entry-keyed  closure_key 的 entry 行里落在测试服务代码里的入口单元照取指纹
      「键：入口自己落在测试服务代码里：服务代码自己的入口 entry 行只记是哪一个，改它的正文键不变、改产品代码键变，同一份里另一个入口的键不同；同一份里的测试函数当入口照记指纹，改它的正文键变」
  FUNCTION_UNITS_BREAK=test-service-code-test-entry-unkeyed  closure_key 的 entry 行里落在测试服务代码里、是测试的入口单元也只记是哪一个（不认 closure.test_code_entry_units）
      「键：入口自己落在测试服务代码里：服务代码自己的入口 entry 行只记是哪一个，改它的正文键不变、改产品代码键变，同一份里另一个入口的键不同；同一份里的测试函数当入口照记指纹，改它的正文键变」
  FUNCTION_UNITS_BREAK=runtime-data-files-short  RUNTIME_DATA_FILES（lib_function_units_registry.py：算键库运行期从仓根底下读、算键必须有的数据文件，拷样本仓的一侧按 ast 读）少最后一项
      「真仓：算键库各份（scripts/gate/lib_function_units*.py，不含自证模块与夹具）里以 LIBRARY_ROOT 拼出的读法，每一处要么是 RUNTIME_DATA_FILES 登的数据文件、要么写在 RUNTIME_LIBRARY_ROOT_READS_UNLISTED（理由）里；登的每一项仓里有文件、也被读到；不登的每一行也被读到」
  FUNCTION_UNITS_BREAK=profile-unkeyed-scan-sample  构建档不进键那一格多扫一段带 PROFILE_SETTING_KEYS 的样本文字
      「构建档与线程数不进键：crates/singlefs-unit-graph/src 与这份库的源码里都没有编译档的行（PROFILE_SETTING_KEYS、profile.incremental、依赖一侧的 codegen-units）」

弄坏开关的代码在哪份文件（开关名照上面各条；名字只在夹具里写着的是只给自证夹具的对照表读的）：
  lib_function_units_lexer.py：
    cross-run-cache-under-tmpdir、cross-run-cache-git-directory-only、rust-lex-cache-unverified、rust-lex-cache-namespace-lexer-only
    rust-lex-cache-namespace-whole-module、rust-lex-cache-namespace-module-placement-read
  lib_function_units_base.py：
    external-program-words-unfiltered、rust-extern-link-arguments-unchecked、rust-extern-two-colon-instructions-ignored
    rust-extern-system-library-list-short、rust-kind-names-hyphenated、rust-qualified-names-file-local
  lib_function_units_registry.py：
    built-tool-launches-refused、launched-programs-refused、program-words-refused、tail-reads-under-records-unrecorded、tail-read-scopes-root-ignored
    test-service-code-table-row-dropped、test-service-code-split-families-unrecognized、registered-functions-short-names
  lib_function_units_python_facts.py：
    shell-open-tail-unnarrowed、python-slice-unpacking-unpaired、python-main-guard-conjunctions-unrecognized
  lib_function_units_shell_text.py：
    git-repository-paths-unfolded、shell-inline-positionals-escapes-ignored、shell-substitutions-ignored、shell-command-structure-ignored
    shell-subscript-assignment-split、shell-preflight-arguments-moved、shell-exit-branches-followed、shell-table-row-items-unmatched
    shell-row-driver-plain-words-dropped、shell-row-driver-selected-rows-dropped、shell-open-tail-unnarrowed、shell-process-substitution-splits-command
    shell-process-substitution-redirects-dropped、shell-quoted-assignments-unread、shell-heredoc-empty-rows-spanned、shell-find-listing-operators-unchecked
  lib_function_units_shell_index.py：
    shell-heredoc-split、shell-unfolded-paths-ignored、shell-raw-text、shell-variable-first-line-only、tree-copy-tails-ignored
    shell-redirect-targets-loaded、shell-path-arguments-loaded、shell-generated-scripts-ignored、shell-inline-positionals-ignored
    shell-copied-scripts-ignored、shell-enclosing-locals-ignored、shell-cd-first-directory-only、shell-data-sources-nested-unfollowed
    shell-dot-source-unrecognized、shell-data-source-refusals-dropped、inline-scripts-ignored、shell-later-interpreters-ignored
    shell-substitutions-ignored、external-program-words-unfiltered、python-modules-ignored、shell-unfolded-commands-allowed
    shell-wrapped-commands-ignored、shell-array-commands-unfolded、shell-heredoc-captures-unfolded、inline-argv-commands-ignored
    shell-nested-commands-in-outer、inline-refusal-lines-at-launch、shell-function-bounds-quote-blind、shell-script-tail-groups-ignored
    shell-table-rows-unselected、shell-tree-copy-launch-words-dropped、shell-deferred-heredocs-unmoved、shell-refusal-lines-by-first-mention
    shell-env-array-assignments-kept、shell-registered-missing-paths-refused、shell-function-outputs-unfolded、shell-quoted-assignments-unread
    shell-exported-variables-unread、shell-narrowed-views-unshared、shell-python-launch-arguments-unfolded、shell-pwd-unfolded
    shell-pattern-removals-unfolded、shell-directory-copies-ignored、shell-relative-tree-copies-ignored、shell-directory-wildcards-ignored
    shell-positional-scripts-refused、shell-relative-directories-unrooted、shell-sourced-values-folded-in-reader、shell-sourced-zero-own-file
    shell-sourced-siblings-unread、inline-bodies-launch-file-only、shell-self-assignments-counted、shell-function-printed-words-single
    shell-read-heredoc-assignments-ignored、python-module-object-refusal-lines-at-exec、shell-read-assignments-unrecorded、shell-find-listings-ignored
    fold-limit-shell-paths-skipped、shell-case-subjects-launched、shell-refusal-lines-by-substring、shell-refusal-lines-positional-unmatched
    shell-prefix-assignments-unscoped、shell-temporary-sites-folded、shell-inline-quoted-words-unfolded
  lib_function_units_python_data_reads.py：
    python-docstring-writes-counted、python-docstring-augmented-assignment-not-read、python-globals-subscripts-refused、tree-copy-tails-ignored
    python-dispatch-tables-ignored、python-getattr-stdlib-refused、python-written-scripts-ignored、python-directory-wildcards-ignored
    python-unfolded-heads-allowed、python-parsed-arguments-unread、python-directory-reads-refused、python-unlisted-directory-reads-whole、python-root-joins-refused
    python-attribute-reads-unfolded、python-tail-reads-unanchored、python-tail-reads-samples-included、python-data-reads-follow-build-outputs
    python-membership-tables-followed、spec-path-arguments-refused、spec-path-tuple-first-only、tail-read-scopes-ignored
    tail-read-scopes-copies-unscoped、python-local-names-resolved、python-joined-tails-unanchored、python-glob-listings-ignored
    python-pattern-listing-transforms-accepted、python-returned-listings-ignored、fold-limit-reads-skipped、fold-limit-unmatched-tails-refused
  lib_function_units_python_launches.py：
    python-docstring-call-readers-ignored、python-getattr-allowed、python-registered-inline-launches-analyzed、python-spec-paths-ignored
    python-command-arguments-loaded、python-written-scripts-ignored、python-import-names-unfolded、inline-scripts-ignored、python-modules-ignored
    declared-checks-ignored、inline-argv-commands-ignored、python-unfolded-heads-allowed、python-commands-literal-only、python-path-programs-refused
    python-command-dictionaries-unfolded、python-command-concatenation-unfolded、python-command-slices-unfolded、python-command-heads-split-ignored
    python-script-tail-groups-ignored、python-environment-assignments-unfolded、python-data-reads-ignored、python-root-joins-refused
    python-caller-launches-refused、python-toolchain-queries-refused、python-registry-programs-ignored、python-registry-programs-linked
    python-registry-command-texts-refused、python-getattr-names-unfolded、python-module-body-commands-unfolded、python-compiled-part-exec-allowed
    python-module-object-exec-refused、python-module-object-exec-unfolded-allowed、python-module-object-loader-call-sites-ignored
    fold-limit-launches-skipped
  lib_function_units_python_module.py：
    python-comments-kept、python-docstrings-kept、python-imported-sys-path-ignored、python-call-returns-unfolded、python-module-constants-unfolded
    python-subscript-targets-rebind、python-environment-defaults-unfolded、python-loop-variables-unfolded、python-imports-first-directory-only
    python-path-tables-unfolded、python-comprehension-scope-leaks、python-dispatch-calls-ignored、python-nested-scopes-unfolded
    python-method-parameters-unfolded、python-comprehension-variables-unfolded、python-element-calls-unfolded、python-sequence-indexes-unfolded
    python-loaded-module-constants-unfolded、python-submitted-functions-unfolded、python-text-pieces-unfolded、python-getattr-names-unfolded
    python-guard-names-unfolded、python-def-heads-ignored、python-main-guards-last-only、python-main-guard-else-dropped、python-importer-sys-path-ignored
    python-importer-call-sites-ignored、python-importer-sites-outside-family、python-split-part-environment-unread
    python-module-sequence-elements-unfolded
  lib_function_units_rust_file.py：
    rust-macro-arguments-all-local、rust-macro-list-short、rust-verus-segments-ignored、rust-doc-comments-kept、rust-format-captures-ignored
    rust-own-program-restart-refused、rust-own-program-path-unchecked、rust-own-program-uses-any-block、rust-own-program-divergence-any-block
    rust-extern-listed-calls-refused、rust-extern-any-block-keyed、rust-extern-link-first-attribute-only、rust-trait-impls-by-trait-arguments
    rust-doctest-fingerprint-from-rows、rust-module-heads-dropped、rust-escaped-brace-captures-skipped、rust-union-as-keyword
    rust-assume-specifications-unrecognized、rust-assume-specification-names-unlinked、rust-verus-semicolon-split-off、rust-use-self-imports-ignored
    rust-open-by-trait-arguments、rust-includes-outside-root-refused、rust-manifest-directory-reads-unrefused、rust-manifest-directories-refused
    rust-manifest-binding-refused-whole、rust-current-exe-refused、rust-current-exe-path-only-refused、rust-current-exe-spawn-unrefused
    rust-blanket-self-parameter-only、rust-concatenated-includes-skipped
  lib_function_units_rust_verus.py：
    product-view-verifier-attributes-kept
  lib_function_units_unit_graph.py：
    unit-graph-text-busy-not-waited、unit-graph-digest-skips-lock-file、unit-graph-failure-step-unnamed、unit-graph-present-program-rebuilt
    unit-graph-directories-not-shared
  lib_function_units_rust_workspace.py：
    rust-macro-rules-literal-names-unlinked、rust-excluded-path-dependencies-dropped、rust-use-paths-flattened
    rust-use-path-conditional-candidates-narrowed、rust-use-path-re-exporting-modules-narrowed、rust-use-path-first-segment-imported-narrowed
    rust-workspace-members-glob-unescaped、rust-workspace-dot-member-unnormalized、rust-same-named-member-directories-merged
    rust-workspace-multiline-backslash-read、rust-use-path-first-segment-same-use-excluded、rust-use-path-first-segment-item-macros-ignored
    rust-use-alias-leaf-followed-to-the-original、rust-extern-other-block-link-unchecked、rust-extern-build-script-link-unchecked
    rust-extern-link-of-other-targets-unchecked、rust-extern-link-of-dependencies-unchecked、rust-extern-link-arguments-unchecked
    rust-extern-dependency-dev-dependencies-unfollowed、rust-extern-instruction-words-bare、rust-extern-library-literals-of-any-package
    rust-extern-external-dependencies-unchecked、rust-dependency-features-from-every-section、rust-underscore-items-named、product-view-kept-formal
    product-view-strips-executable-code、product-view-edges-from-the-written-tokens、product-view-blanking-unverified
  lib_function_units_rust_closure.py：
    rust-open-entry-traits-ignored、rust-dyn-traits-open-every-implementation、rust-constants-dropped、rust-macros-dropped、rust-trait-impl-edges-dropped
    rust-macro-generated-types-unnamed、rust-global-allocators-unlinked、rust-foreign-self-trait-impls-unlinked、rust-build-environment-unlinked
    rust-outside-crate-names-linked-unimported、rust-outside-crate-names-linked-across-modules、rust-crates-names-linked-across-modules
    rust-let-shadowed-names-resolved、rust-cfg-ignored、wgsl-raw-text、rust-doctest-entry-alone、shell-data-sources-unfollowed、shell-data-sources-unsited
    rust-own-program-entry-unchecked、rust-module-heads-unlinked、rust-module-declarations-unlinked、rust-use-names-resolved、rust-use-aliases-unresolved
    wgsl-external-names-by-long-identifiers、rust-cfg-macro-atoms-ignored、wgsl-defined-name-suffixes-ignored、wgsl-prefix-remainders-unchecked
    wgsl-name-suppliers-not-pulled-in、wgsl-name-suppliers-include-test-code、wgsl-modules-per-naming-unit、rust-trait-self-paths-ignored
    rust-doctest-structure-linked、rust-blanket-impls-ignored、rust-current-exe-children-unlinked、rust-current-exe-constants-same-file-only
    wgsl-other-file-definitions-unlinked
  lib_function_units_script_closure.py：
    python-docstring-readers-ignored、python-attributes-unnarrowed、loaded-attributes-closure-wide、shell-names-command-position-only
    declared-checks-ignored、python-parameter-callers-unresolved、python-call-sites-whole-view、built-tool-sources-ignored
    caller-supplied-commands-refused、caller-supplied-commands-callee-wide、python-call-sites-matched-by-name、python-partial-call-sites-ignored
    closure-round-limit-unchecked、closure-call-index-frozen、data-units-process-wide、judging-entry-index-not-shared、python-launch-arguments-unfolded
    python-toolchain-queries-keyed、shell-program-arguments-unnarrowed、shell-entry-arguments-unnarrowed、shell-entry-undecided-branches-dropped
    shell-table-rows-unselected、shell-table-launch-without-assignment-counted、shell-row-driver-mentions-linked、shell-row-driver-whole-launch-dropped
    python-shell-launch-arguments-unfolded、excluded-files-followed、closure-judging-exclusions-dropped、shell-nested-functions-by-any-word
    python-program-arguments-unnarrowed、python-import-time-tables-followed、wgsl-external-name-sources-unlinked、closure-directory-lines-span-files
    python-import-time-statements-ignored、python-import-time-classes-named、program-edge-exclusions-ignored、program-edge-exclusions-shell-ignored
    call-edge-exclusions-ignored、root-ignored-scripts-linked、python-import-time-bindings-followed、shell-name-reach-context-dropped
  lib_function_units_closure_keys.py：
    rust-constants-dropped、external-programs-refused、key-entry-units-dropped
    key-entry-doctest-owner-chained、unit-graph-key-entry-lines-dropped、cross-run-cache-run-times-old-spelling、row-keyed-tables-whole
    rust-data-dump-unlisted、rust-cfg-atoms-unnarrowed、test-service-code-keyed、test-service-code-entry-keyed、test-service-code-test-entry-unkeyed
  lib_function_units_selftest_fixture_trees.py：
    profile-unkeyed-scan-sample、judging-entry-workers-head-word、judging-entry-literals-split-unfollowed
    unit-graph-sources-top-level-only
  lib_function_units_selftest_rust_cells.py：
    rust-macro-rules-literal-names-unlinked、selftest-fixtures-loaded-at-import、selftest-fixtures-runtime-name-moved
    rust-lex-cache-namespace-reads-own-file、selftest-fixture-names-receivers-ignored、unit-graph-inputs-lock-undeclared
  lib_function_units_selftest_program_registries.py：
    registered-words-drifted、tail-read-scopes-drifted、tail-read-scopes-inline-unread、stage-cell-functions-drifted、registered-functions-drifted
    runtime-data-files-short、registered-functions-module-object-texts-unread
  lib_function_units_selftest_closure_rules.py：
    not-judging-files-drifted
  lib_function_units_selftest_real_repo.py：
    cross-run-cache-mentions-unscanned、e171-selftest-table-at-module-level、e171-admission-selftest-at-module-level
    e171-list-check-imports-reuse-library、excluded-edge-exit-unmeasured、admission-edge-admitted-sample、admission-edge-header-unstripped
    admission-edge-reach-unmeasured、card-lease-client-words-granted-sample、card-lease-client-words-return-output-kept
    card-lease-client-words-other-launches-unchecked、judging-entry-refusals-ignored、selftest-modules-loaded-at-import
  lib_function_units_selftest_run.py：
    selftest-lost-results-unchecked

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
  Python：exec / eval 的字符串（exec 进模块对象的除外，上面「exec 进模块对象」）、按字符串名字派活（{"x": "函数名"} 再 globals()[…] 以外的写法）、类属性之外的动态属性赋值；
    exec 进模块对象的库正文里类的方法与构造的参数不照载入方的调用处折（载入方 <对象>.<方法>(…) 的实参到不了那个参数上），
    收闭包时按调用处认参数（closure_call_arguments）也看不见内联正文里的调用处（内联单元不进 calls_by_name）；
    模块级语句只在那个模块有单元进闭包时连进来（import 时的副作用照算），模块被别的进程起时连的是 main-guard 与 module-body。
  起代码的写法（推的，有夹具钉住的是上面「起代码的写法逐种认」那几条）：内联 bash 串里自己赋的变量只在这个串里看得见（串之外的赋值照本单元的折）；
    跨脚本的参数不传（被起的脚本里 "$1" 起的程序，按 "$1" 一类认）；根外程序按「不带脚本后缀、根里没有这一截」认，同名的替身不分。
  shell：eval 的字符串、按变量名间接调用（"${handler}"）只在那个变量的值是本闭包可见的函数名、而这个名字作为一个词出现在闭包里时才连上；
    here-doc 的起止按规范文本里的 <<词 与单独一行的词认，引号里长得像 <<词 而后面又恰好有单独一行同一个词的会多跳一段。
"""
import os
import sys
import time
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402
# 同目录的各份模块按名字 import；入口按路径载入时调用方的 sys.path 里不一定有这个目录，放在末尾、不盖别处的同名模块
_LIBRARY_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
if _LIBRARY_DIRECTORY not in sys.path:
    sys.path.append(_LIBRARY_DIRECTORY)
from lib_function_units_lexer import (
    BREAK_VARIABLE, active_breaks, IDENT, LITERAL, LIFETIME, PUNCT, OPENING_OF, block_document_rows, is_identifier_start, is_identifier_continue,
    rust_lex, RUST_LEX_CACHE_FORM, RUST_LEX_CACHE_DIRECTORY_NAME, RUST_LEX_CACHE_DIRECTORY_VARIABLE, RUST_LEX_CACHE_KEPT_NAMESPACES,
    RUST_LEX_CACHE_STATS, _RUST_LEX_CACHE_NAMESPACES, RUST_LEX_CACHE_PROCESS_STATE, GLOBAL_NAME_OPERATIONS, CLASS_PLACEMENT_ATTRIBUTES,
    MODULE_PLACEMENT_NAMES, git_common_directory_of, cross_run_cache_directory, rust_lex_cache_directory, live_closure_fingerprint,
    rust_lex_cache_namespace, rust_lex_cache_path, load_rust_lex, store_rust_lex, rust_lex_cached, match_brackets,
)  # noqa: E402,F401
from lib_function_units_base import (
    KEY_FORM, FORMAL_VERIFICATION, PRODUCT_BEHAVIOR, JUDGMENT_KINDS, UNIT_GRAPH_DIRECTORY_VARIABLE, UNIT_GRAPH_KEPT_PROGRAMS,
    VERUS_GHOST_CODE_UNPARSED, SPAWNING_WORDS, C_LIBRARY_CALLS_ALLOWED_IN_EXTERN_BLOCKS, c_library_calls_allowed_in_extern_blocks,
    RUSTC_LINK_LIBRARY_INSTRUCTION, LINK_INSTRUCTION_NAMES, LINK_INSTRUCTION_IN_A_STRING, LINK_LIBRARY_INSTRUCTION_IN_A_STRING,
    SINGLE_COLON_LINK_INSTRUCTION_IN_A_STRING, SINGLE_COLON_LINK_LIBRARY_INSTRUCTION_IN_A_STRING, link_instruction_pattern, REFUSED_EXIT,
    SHELL_TIMEOUT_SECONDS, SNIPPET_CHARACTERS, FOLD_LIMIT, FOLD_LIMIT_FORM, DEEPEST_MANIFEST_BINDING_CHAIN, MANIFEST_FOLD_REFUSED, SKIPPED_DIRECTORIES,
    DEFAULT_SCRIPT_DIRECTORIES, EXTERNAL_PROGRAMS, EXTERNAL_PROGRAM_PREFIXES, PYTHON_INTERPRETERS, SHELL_INTERPRETERS, INLINE_FORM, COMMAND_FORM,
    MESSAGE_HEADS, COMMAND_WRAPPERS, PYTHON_VALUE_OPTIONS, SHELL_VALUE_OPTIONS, TEST_RUNNER_MODULES, PREFLIGHT_MODULE, DECLARED_CHECK,
    declared_checks, SHELL_POSITIONAL_WORD, external_program_name, command_positions, interpreter_mode, ARGV_SPAWNS, argv_command_offsets,
    descriptor_of, InlineScript, OPAQUE, MEMO_IN_PROGRESS, COMPREHENSION_UNPACKED, POSITIONAL, ValueElement, FunctionUnitsError, UsageError,
    sha256_text, sha256_bytes, snippet_of, Refusal, UNIT_SLOTS, HYPHENATED_KIND_NAMES, Unit, data_file_unit, directory_fingerprint,
    data_directory_unit, files_under, inside_root, ASSIGNMENT_WORD, SHELL_OPEN_TAIL, LIBRARY_ROOT,
)  # noqa: E402,F401
from lib_function_units_registry import (
    IMPORT_TIME_RULE, IMPORT_TIME_CLASS_RULE, NOT_JUDGING_FILES, UNKEYED_PROGRAM_WORDS, LAUNCHED_REPOSITORY_PROGRAMS, BUILT_TOOL_SOURCES,
    BUILT_TOOL_LAUNCHES, CALLER_SUPPLIED_COMMANDS, REGISTRY_COMMAND_TEXTS, SPEC_PATH_ARGUMENTS, TAIL_READ_ROOT_COPY, in_tail_read_scope,
    TAIL_READ_SCOPES, UNSCOPED_TAIL_READS, TAIL_READ_RECORD_PREFIXES, record_unscoped_tail_read, shell_unquoted_word, program_word_registered,
    launched_programs_registered, built_tool_launch_source, root_gitignore_rules, root_file_ignored, ADMISSION_EDGE, EXCLUDED_PROGRAM_EDGES,
    EXCLUDED_CALL_EDGES, TEST_SERVICE_CODE_TABLE, TEST_SERVICE_CODE_FAMILY_SUFFIXES, TEST_SERVICE_CODE_ROWS, test_service_code_rows, is_test_service_code,
)  # noqa: E402,F401
from lib_function_units_python_facts import (
    SUBPROCESS_CALLS, EXEC_CALLS, SHELL_STRING_CALLS, SOURCE_READERS, PATH_TRANSFORMS, PATH_CONSTRUCTORS, TEMPORARY_CALLS, PATH_METHODS, DATA_FORM,
    COPY_CALLS, STAGE_SAMPLE_DIRECTORY, TOOLCHAIN_QUERY_WORDS, TOOLCHAIN_UNIT_KIND, TOOLCHAIN_QUERY_SECONDS,
    REGISTRY_LISTED_PROGRAMS_TABLE, ARGV_BINDING, SHALLOW_IMPORT_VIEW, CALLER_INPUT_ORIGINS, PYTHON_BUILTIN_CALLS, is_docstring, without_docstrings,
    dump_fingerprint, assignment_names, import_bound_names, is_main_name_test, is_main_guard, stored_names, dotted_name, ElementsOf, bound_names,
    walk_nodes, function_local_names, local_name_reads, named_calls_in, partial_bound_call, walk_nodes_skipping, KnownWords, launch_arguments,
    shell_launch_choices, known_equal, literal_integer, unique_alternatives, paired_elements, sliced_elements, OWN_SCOPES, walk_function_statements,
    guard_assignments, enclosing_unit, walk_targets, PythonFacts,
)  # noqa: E402,F401
from lib_function_units_shell_text import (
    SHELL_PARAMETER_NAME, expand_shell_parameters, shell_assignment_value, shell_assignment_row, SHELL_WRAPPER, SHELL_NORMALIZE_SCRIPT,
    KEPT_SHELL_COMMENT, FUNCTION_HEADER, HEREDOC_START, HEREDOC_BODY_PLACED, TOP_ASSIGNMENT, SHELL_KEYWORDS, SHELL_DECLARATIONS, SHELL_WORDS,
    VARIABLE_REFERENCE, kept_shell_comments, shell_normal_forms, SHELL_DEFERRED_HEREDOC, raw_heredoc_bodies, raw_heredoc_rows,
    deferred_heredocs_moved_back, heredoc_mask, rows_inside_quotes, SHELL_POSITIONAL_REFERENCE, SHELL_ARGUMENTS_MOVED,
    SHELL_PREFLIGHT_ARGUMENTS_RESTORED, SHELL_EXIT_ROW, SHELL_DECIDABLE_IF, SHELL_DECIDABLE_CASE, substituted_positionals, pruned_decided_branches,
    narrowed_shell_body, SHELL_ROW_SELECTED_TABLES, SHELL_TABLE_ASSIGNMENT, SHELL_TABLE_ROW_DRIVER, table_data_rows, table_row_key,
    table_row_identity, table_row_selection, selected_row_identities, table_rows_selected, rows_of_selected_keys, table_drivers_of_unselected_rows,
    written_only_as_table_row, shell_launch_prefix, COMMAND_POSITION_BEFORE_WORD, SHELL_ENV_ARRAY_ASSIGNMENT, SHELL_QUOTED_ASSIGNMENT,
    shell_position_values, shell_known_words, heredoc_bodies, command_inputs, substitutions, matching_close, double_quote_end,
    PROCESS_SUBSTITUTION_WORD, split_commands, case_pattern_end, test_command_end, git_repository_location, TREE_COPY_TAIL, remove_shell_pattern,
    leading_command_words, in_skipped_directory, tree_copy_file, INLINE_POSITIONAL, inline_positionals, SHELL_TREE_PREFIX, literal_word, ShellFacts,
    GENERATOR_HEADS, COPYING_HEADS, generated_scripts,
)  # noqa: E402,F401
from lib_function_units_shell_index import (
    ShellFile, shell_block_end, ShellIndex, REPOSITORY_MODULE_NAMES, repository_module_names, repository_module, shell_files_under, shell_units,
    language_of_file, shell_sourced_files, inline_shell_units, inline_refusals,
)  # noqa: E402,F401
from lib_function_units_python_data_reads import PythonModuleDataReads  # noqa: E402,F401
from lib_function_units_python_launches import PythonModuleLaunches  # noqa: E402,F401
from lib_function_units_python_module import PythonIndex, PythonModule, python_units  # noqa: E402,F401
from lib_function_units_rust_file import (
    rust_plain_string, rust_string_value, FORMAT_CAPTURE, format_captures, WORDS_STARTING_A_NESTED_ITEM,
    STANDARD_MACROS_EVALUATING_THEIR_ARGUMENTS_IN_PLACE, standard_macros_evaluating_their_arguments_in_place, function_signature_positions,
    parameter_binding_position, identifier_name, extern_block_attributes_name_link, bare_name_only_a_local_binding, use_path_leading_to,
    token_references, RUST_QUALIFIERS, RUST_ITEM_STARTS, RUST_NAMED_KINDS, RUST_KINDS_DEFINING_NAMES, RUST_TYPE_KINDS, use_path_root,
    STANDARD_CRATE_PATH_ROOTS, RUST_TYPE_OR_TRAIT_KINDS, RUST_STRUCTURE_ONLY_KINDS, RUST_ITEM_MACROS_HOLDING_ITEMS, PRIMITIVE_TYPES,
    SELF_TYPE_SKIPPED, RUST_KEYWORDS, UNION_KEYWORD, TEST_ATTRIBUTE_RAW, cfg_tree_leaves, cfg_atom_text, cfg_predicates_in_tokens, parse_cfg_tokens,
    RustFile, RUST_CODE_BLOCK_WORDS, is_rust_code_block_information, code_fence_of, without_hidden_line_marker, rust_code_blocks,
)  # noqa: E402,F401
from lib_function_units_rust_verus import (
    VERUS_GHOST_FUNCTION_MODES, VERUS_SIGNATURE_CLAUSES, VERUS_LOOP_CLAUSES, VERUS_GHOST_CALLS, VERUS_GHOST_WRAPPERS, VERUS_ITEM_START_WORDS,
    VERUS_NOT_CALLEES, VERUS_CLAUSE_CONTINUATIONS, VerusGhostCode, VERIFIER_ATTRIBUTE_HEAD, OPENING_PUNCTUATION, CLOSING_PUNCTUATION,
    verifier_attribute_token_ranges, verus_ghost_code_blanked, product_view_agrees,
)  # noqa: E402,F401
from lib_function_units_unit_graph import (
    UNIT_GRAPH_PRODUCT_VIEW_CACHE_DIRECTORY_NAME, unit_graph_product_view_cache_directory, UNIT_GRAPH_TEXT_BUSY_ATTEMPTS,
    UNIT_GRAPH_TEXT_BUSY_PAUSE_SECONDS, run_program_waiting_out_text_busy, ProductViewUnavailable, product_view_dump, unit_graph_directory,
    built_unit_graph, unit_graph_inputs_digest, build_unit_graph,
)  # noqa: E402,F401
from lib_function_units_rust_workspace import (
    KNOWN_CFG_NAMES, DEFAULT_RUST_CFG, TARGET_PREFERENCE, RUST_TARGET_ORDER, load_manifest, RustDependency, RustCrate, RustTarget,
    multiline_basic_strings, members_in_multiline_basic_strings_with_a_backslash, discover_rust_crates, crate_targets, CfgEvaluator, cfg_entries,
    condition_mentions_test, RustWorkspace, doctest_entry_paths, doctest_entry_group, rust_unit_is_test, rust_unit_is_test_code, Closure,
)  # noqa: E402,F401
from lib_function_units_wgsl import (
    WGSL_PREDECLARED, WgslFile, WGSL_FILES, WGSL_TOP_LEVEL_DEFINING_KINDS, wgsl_top_level_names_in, wgsl_file, CRATE_IDENTIFIERS,
    crate_identifier_of, literal_mentions, literal_prefixes, wgsl_closure, wgsl_units,
)  # noqa: E402,F401
from lib_function_units_rust_closure import (
    rust_literal_texts, SHORTEST_WGSL_EXTERNAL_NAME_PREFIX_CHARACTERS, WGSL_NAME_DEFINING_KINDS, rust_declared_names, name_and_underscore_suffixes,
    ascii_upper, rust_string_literal_words, WgslNameSources, rust_entry_dependency_sections, entry_names_only_defined,
    rust_entry_is_the_main_of_a_program, rust_closure_of,
)  # noqa: E402,F401
from lib_function_units_script_closure import (
    CLOSURE_ROUND_LIMIT, SHARED_SCRIPT_INDICES, PROCESS_WIDE_DATA_UNITS, default_script_directories, ScriptClosure, ENTRY_PATTERN, parse_entry,
    Options, judging_excluded_files, command_closure, closure_of, closure_unit_lines, toolchain_version_line, VALUE_OPTIONS, parse_options,
)  # noqa: E402,F401
from lib_function_units_closure_keys import (
    TEST_ENVIRONMENT_STAGE_SCRIPT, TEST_ENVIRONMENT_STAGE_SHARED_ENTRY, work_root_library,
    test_environment_stage_run_times_file, cross_run_cache_registry, cross_run_cache_rows, print_cross_run_cache_rows, RustClosureResult, rust_units,
    rust_closure, ROW_KEYED_TABLES, row_keyed_closure, closure_key, entry_key_lines, key_cfg_items, unit_graph_key_lines, unit_graph_key, dump_lines,
    dump_sort_key, cfg_tree_known, explain_lines, CONTROL_ENTRIES, CONTROL_VARIANTS, mutation_row, control_keys, run_controls, write_tree, edit_file,
)  # noqa: E402,F401
# 自证的几份模块（lib_function_units_selftest_*.py）不在 import 时载入：import 这份库的判法闭包不收它们（文件头「自证模块」）。
# 别的脚本的自证与单测经这份库调的三个自证辅助在下面各包一层，用到时才载入定义它的那一份。


def fixture_key(root, entry, options=None):
    """lib_function_units_selftest_fixture_trees.fixture_key：→ (键或 None, 拒算形态的集合)。"""
    from lib_function_units_selftest_fixture_trees import fixture_key as implementation
    return implementation(root, entry, options)


def product_view_dump_of(work, blanked_files):
    """lib_function_units_selftest_real_repo.product_view_dump_of：Rust 那一份的替身（剥过的那一份照原样切出的 dump 行）。"""
    from lib_function_units_selftest_real_repo import product_view_dump_of as implementation
    return implementation(work, blanked_files)


def with_unit_graph_directory(directory, action):
    """lib_function_units_selftest_real_repo.with_unit_graph_directory：FUNCTION_UNITS_UNIT_GRAPH_DIRECTORY 临时设成 directory 跑 action。"""
    from lib_function_units_selftest_real_repo import with_unit_graph_directory as implementation
    return implementation(directory, action)


def main(arguments):
    if arguments == ["--selftest"]:
        # 自证模块走到这一支才载入（文件头「自证模块」）
        from lib_function_units_selftest_run import selftest
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
            closure = command_closure(root, entry, options)
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
            # 版本不进键（文件头「版本不进键」）：命令行不收工具链、依赖与外部程序的行
            key, refusals = closure_key(closure)
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
            from lib_function_units_selftest_fixture_trees import registered_judging_entries
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
