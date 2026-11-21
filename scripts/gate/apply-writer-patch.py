#!/usr/bin/env python3
# admission: always 每个实现员补丁交回都要打一次，判的是这一刻的补丁目录与主工作区
# run-condition: command git rsync
"""把实现员交回的补丁目录打进主工作区：补丁、变异表按名字合并，记一笔「自上次崩溃验证全绿以来打了几个补丁」。

用法：
    apply-writer-patch.py <补丁目录> [--root 仓根] [--dry-run]
    apply-writer-patch.py --pending-status [--root 仓根]     # 上一次有没有没换完的（文件头「换上与被杀」）；退 0 没有，3 有
    apply-writer-patch.py --finish-pending [--root 仓根]     # 照记下的暂存接着换完
    apply-writer-patch.py --rollback-pending [--root 仓根]   # 照记下的补丁前原样退回
    apply-writer-patch.py --read-outcome <结局文件> [--root 仓根]   # 只读结局文件、对着主工作区现算 sha256，判打全了没有（文件头「结局文件」）
    apply-writer-patch.py --selftest          # APPLY_WRITER_PATCH_BREAK=<项> 时必须判红

补丁目录里的文件（格式由 .claude/agents/implementation-writer.md「产出」一节定）：
    crates.patch                 `git diff` 的输出，改到的路径不限（实现员的 crates 与 litmus、跟着补丁走的登记表；拒算补丁目录里只有 .claude/gate.d/refused-tests.tsv）；只有 crates/mutations.tsv 不许在里面（变异表走下面三份文件）；没有代码改动可以没有
    mutations-append.tsv         要追加的变异行，每行六段（与 crates/mutations.tsv 同），名字在表里不许已有、在这个文件里也不许出现两次
    mutations-replacements.tsv   要整行换掉的变异行，每行十二段：前六段是新行，后六段是写补丁时表里那一行的原样（前像），两半第一段的名字相同；
                                 按名字找表里恰好一行，那一行逐字等于前像才换成新行
    mutations-delete.txt         要删的变异行，每行是写补丁时表里那一行的原样（前像，六段）；按第一段的名字找表里恰好一行，逐字等于前像才删
  前像比对（防写补丁之后别的会话改过的行被照名字盖回旧值）：比的是被打那棵树（--root）此刻的 crates/mutations.tsv，整行逐字比（去掉行尾换行符，制表符算在里面）。
    任一行对不上整份不打、一个字不改、退 2，逐行写行名与两侧原文（制表符写成 ⇥），出路是以被打那棵树此刻的行为基线重出这几行（PREIMAGE_WAY_OUT）。
    段数不对的行（替换不是十二段、删除不是六段；没带前像的替换只有六段、只写名字的删除只有一段，都是这种）一份文件里有一行就整份不认、同样退 2，出路同上；
    两半名字不同的替换行拒（改名写成删一行加追加一行）。
    合补丁那一支（merge-and-verify.sh 的 MERGE_PATCH_PROGRAM）照快照里开跑之前那一份表写前像。
    result.md                    实现员的报告；补丁动了 crates/singlefs-checker/src/ 的，报告里要有「受影响的流与崩溃枚举用例」一节；补丁在产品代码里新加了走不到的断言或未定条款的错误成员的，报告里要有「走不到的理由」一节（文件头「走不到的理由」）
    baseline/<相对仓根的路径>     可以没有：补丁改的文件补丁前的原样（实现员副本里 `git show :<路径>` 取），只在补丁头 index 行左边那个 blob 不在 --root 的对象库里时读（文件头「三方合并」）
上面前四份（crates.patch 与三份变异表文件，RECOGNIZED_PATCH_FILES）是它认得的补丁，每一份单独可以没有，可是一份都没有就拒、退 2、一个字不改：目录里有别的 *.patch（例：all.patch，
  改 task-server、scripts 这类 crates 之外的文件的补丁）的点名那几份、写明它们一个字没打，出路是用 git apply <补丁文件> 直接打、回读核过；一样补丁都没有的（空目录、给错了目录、只有
  result.md）同样拒，出路是核对目录。只管「没认出」这一支：认出了的补丁照下面的次序核。
次序：先全部核一遍（补丁 git apply --check，不过的照「三方合并」重对；名字在不在、行的段数对不对、替换与删除的前像对不对得上、checker 那一节在不在、走不到的理由那一节列没列全），有一处不对就一个字不改、退 2；
都对了才算改后的全文：补丁套在临时目录里补丁前原样的那几份文件上、变异表按名字合并，每一份改后的全文算出来、sha256 记下（算不出的、套不上的一个字不改、退 2）；
再照「换上与被杀」一份份改名换上（crates/ 下的源文件与 crates/mutations.tsv 一起，不再用 git apply 在主工作区里就地写）；全部换上、回读核过之后，
记打补丁记录 <git common-dir>/singlefs-applied-writer-patches.tsv，再打出 .claude/gate.d/code-source-discipline.sh 的 mutation-tables 那一格（--check mutation-tables）要另交的整条命令，
这一格不在这个进程里跑（文件头「mutation-tables 那一格」）。
每一步在跑之前先打一行「· 进度：…」到标准输出并立刻刷出去，不只在最后打结果行（被杀时才留得下做到哪一步）。
换上与被杀（要么整份写完、要么一个字不动）：
  暂存：<git-dir>/singlefs-apply-writer-patch-pending/ 下 staged/<序号> 放每份改后的全文、before/<序号> 放每份补丁前原样的备份、journal.json 记每份的路径、动作（改、增、删）、
    补丁前与改后的 sha256 与权限位；journal.json 最后一个写（先写临时文件再改名），它在才算暂存齐、核过。暂存目录与仓在同一个文件系统（os.replace 的前提），不在就拒。
  换上：先核每份主工作区此刻的 sha256 仍等于记下的补丁前（不等就是核与换之间被别的会话改了，一个字不动、退 2），然后一份份 os.replace（删的 unlink）；
    换的过程中出错（不是被杀）就地照备份退回已换上的，退回干净了一个字没留、退 2，退不干净的留着暂存目录。全部换上后回读每份算 sha256，与记下的改后对上才写「完成」、删暂存目录；
    对不上写「异常」、留着暂存目录、退 2。每份换上与否永远按内容判（现在的 sha256 等于改后就是已换上，等于补丁前就是没换上，都不等就是被别的改动动过），不另记进度，被杀在哪一步都判得出。
  被杀：暂存目录里有 journal.json 就是上一次没换完。下一次起这个脚本（同一个补丁目录或别的补丁目录，真打与 --dry-run 一样）先认出来，退 2，列出已换上、没换上、被别的改动动过的文件，
    出路写两条：--finish-pending（照记下的暂存接着换完，回读、写完成、记打补丁记录、打出 mutation-tables 那一格要另交的命令）、--rollback-pending（照 before/ 里的备份原样退回，回读核过、写退回）；
    有被别的改动动过的文件两条都不做（会盖掉别人的改动），列出来交人。不许在半截上再打别的补丁。
    暂存目录里没有 journal.json 是暂存到一半被杀，主工作区一个字没动，下一次起自动清掉重来。
  同一个仓同一时刻只许一个在换：<git-dir>/singlefs-apply-writer-patch.lock 上 flock，不等，拿不到退 2（被杀时内核放锁）。
  占内存：这一步（算改后全文、暂存、核、换上、回读）一次只有一个一兆的读块与一个临时目录里的几份文件，文件一份份流着拷、流着算 sha256，不整份读进内存，与补丁大小无关。
    占内存的大头是 mutation-tables 那一格（code-source-discipline.sh --check mutation-tables 起的 scripts/mutation/crates-mutation-rows.py static-check
    对整个工作区做可达性判），不在这个进程里起（文件头「mutation-tables 那一格」）。
结局文件（真打时写，--dry-run 不写）：补丁目录里的 apply-outcome.tsv，每次真打重写（接着换完、退回在原文件上追加）。格式：制表符分隔，`#` 起头的行是说明，其余每行第一段是行种类：
    开始<TAB>补丁目录绝对路径<TAB>仓根绝对路径<TAB>进程号<TAB>开始秒数
    文件<TAB>相对仓根的路径<TAB>动作（改、增、删）<TAB>补丁前 sha256（增写 -）<TAB>计划的改后 sha256（删写 -）      （开始之后、动手换之前写全，每个要动的文件一行）
    回读<TAB>相对仓根的路径<TAB>回读算出的改后 sha256（删了写 -）                                            （全部换上之后每个文件一行）
    完成<TAB>文件数<TAB>结束秒数                                                                          （写入部分最后写这一行）
    退回<TAB>文件数<TAB>结束秒数                                                                          （--rollback-pending 或换的时候出错就地退回干净之后写，替代完成）
    异常<TAB>说明<TAB>点名的文件                                                                          （回读与记下的改后对不上）
    后续<TAB>门禁<TAB>没跑<TAB>要另交的整条命令                                                           （完成之后写：mutation-tables 那一格没在这里跑；被杀没有这一行，不影响打全了没有）
  被杀的那一次只有开始与文件行，没有完成。判打全了没有只读这份文件、对着主工作区现算 sha256：--read-outcome 做这件事，退 0 打全了（有完成，每个文件现在的 sha256 等于回读），
  1 已退回（有退回，每个文件现在等于补丁前），3 没打完（没有完成也没有退回：逐个文件列已换上、没换上、被别的改动动过，出路写 --pending-status），
  4 完成或退回之后文件又被改了（逐个点名），2 读不了或不是结局文件。
走不到的理由（.claude/agents/implementation-writer.md「做什么」第 6 步；核在补丁套得上之后、变异锚点静态核之前，有一处不对就拒、退 2、一个字不改）：
  对象是补丁在产品代码 crates/<包>/src/ 的 .rs 里新加的行。哪个包是产品代码，照被判那棵树（补丁打上之后的那一棵：补丁改了角色登记表就读改后那一份）里
    .claude/gate.d/crate-roles.tsv（CRATE_ROLE_TABLE，四列：包名、角色、目录、理由）按目录对上的那一行的角色：角色是「产品」的判，验证、测试支撑、实验装置不判；
    包在表里没有一行、或那棵树里读不到这张表的，照产品判，拒绝说明里点名「没登记角色」的包与出路（在角色表里登一行）。判的是：
    todo!、unreachable!、assert!（不含 debug_assert!，不含 assert_eq! 一族），与条款没定、第一版不支持的错误成员的定义行（枚举成员，名字以 Undecided、Unsupported 或 OutsideTheFirstVersion 收尾）。
    注释与字符串、字符字面量里的字不算；#[cfg(test)] 模块、#![cfg(test)] 文件与 #[test]（#[<路径>::test]）函数里的不算。行号是补丁套上之后那份文件里的行。
    整份文件也不算的：它在包里的模块声明（父模块文件里的 `mod <名>;`，父模块文件是 src/lib.rs、src/main.rs、src/bin/<名>/main.rs、src/<a>.rs 或 src/<a>/mod.rs 这几种；
      `#[path = "…"]` 声明的按 path 落的文件认）带 #[cfg(test)]（同一条目上叠了别的属性也认；外围内联模块、祖先任一层的声明或祖先文件的 #![cfg(test)] 带它也算）。
      父模块文件取补丁打上之后那棵树里的（补丁改到的取套上补丁之后那份，没改到的取 --root 里的原文）；读不到父文件、没有任何一处声明引到它的，照旧逐行判、当产品代码。
      整份文件的认法只在 scripts/gate/lib_rust_test_module_files.py（门禁 heavy-thread-defaults、checker-implementation-disjoint 两格也调它），这里只给它补丁改后的文本。
  报告 result.md 里要有一节标题恰好是「走不到的理由」（任意级别的标题），节里每一处一行「- <仓根起的文件路径>:<行> <理由>」，理由至少 8 个字；列的行号对不上、少列、理由太短，
    都拒并逐处点名「文件:行 种类」与出路。多列的（行号指不到任何一处）不拦。
  管不到的：理由写得对不对、走不走得到，靠代码轮三方照调用点重推（.claude/skills/three-way/SKILL.md「代码轮正文的检查单」）；经三方合并重对过的补丁行号以打之后的文件为准，漂了的在拒绝说明里看新行号。
变异锚点静态核（--dry-run，拆分提案第六十二节 ②：补丁改写的代码弄断别批变异行的锚点，到合入验证才看见）：核完补丁与变异表之后，把 --root 那棵树的拷贝范围
  （scripts/hosts/lib_repository_copy.py 的 copy_closure，带测试代码扫：共用清单 crates/、Cargo.toml、Cargo.lock、.cargo/ 加 path 依赖指到的目录（例：验证包
  依赖的仓根 prune-server/）、编译期与运行期读的仓根文件连同它们 source、起的；另声明 litmus/：变异行可以锚在它下面；闭包算不成的按判不了拒）
  再加补丁改到、此刻在 --root 里的每份文件（拷贝范围之外的拒算表、角色表这一类跟着补丁走的登记表也在内；补丁新建的不在 --root 里、不拷）同一刻拷进临时目录（不带 target），
  这是补丁之前那一棵；再把它整棵拷成补丁之后那一棵、套上补丁与合并后的变异表，各在那两棵树上跑这份脚本所在仓里的 scripts/mutation/crates-mutation-rows.py static-check <临时树>
  （git apply 与 static-check 都用 lib_repository_copy.py 的 environment_for_commands_in_the_copy 给的环境）。基线是补丁之前那一棵、不是 --root：
  两趟 static-check 先后跑要几百秒，跑着的时候别的会话改了 --root（改了又改回的、补丁之前就红的被修好的），基线不跟着变；判不判只看 static-check 的输出，补丁改到的文件在不在拷贝范围里不影响。输出里除 COPIED、LITERAL、CHECKED、IDENTIFIERS_CHECKED 与可达性那一道的四种提示行之外的每一行（MALFORMED、ANCHOR、NAME、ROW、STRAY、UNDEFINED、UNREACHABLE、ERROR）都算红；
  四种提示行是 REACHABILITY_CHECKED、REACHABILITY_NOT_JUDGED、REACHABILITY_EXEMPT、REACHABILITY_UNJUDGED（STATIC_CHECK_HINT_KINDS）：static-check 的退出码不受它们影响（crates-mutation-rows.py
  文件头「点名测试够不着被改坏的函数」），而它们带着随树变的东西——REACHABILITY_NOT_JUDGED 的原因里写着被判那棵树的路径（补丁前后两棵临时树的路径不同），REACHABILITY_CHECKED 是判了几行的计数
  （补丁追加一行就变）——拿基线比也去不掉，算红就把每个补丁都拒了；
  红行去掉行号（变异表追加删除会让行号漂；第二段是变异表行号的几种都去，STATIC_CHECK_ROW_NUMBER_KINDS，UNREACHABLE 与 RELEASE 也在里面：补丁删掉前面一行，
  补丁前就有的那一行 UNREACHABLE 或 RELEASE 行号少 1，连行号比就被当成补丁带来的）之后，补丁树上有而基线上没有的才算这个补丁带来的，有一处就拒、退 2、一个字没改；基线上就有的红只报一行提醒、不拦
  （门禁 code-source-discipline 的 mutation-tables 格提交时照样判）。static-check 退出码不是 0 或 1、超时、起不了、补丁树上没有 CHECKED 行，都按判不了拒。
  不加 --dry-run 的真打不跑这一步（打完照「mutation-tables 那一格」打出那一条命令另交）。
mutation-tables 那一格（.claude/gate.d/code-source-discipline.sh --check mutation-tables）：真打与 --finish-pending 都不在这个进程里起它（这一格占内存大，在这里起会撞内存上限被杀）。全部换上、回读核过、写完「完成」、记进打补丁记录之后，
  输出末尾打一行要主 agent 经服务另交的整条命令（带范围声明与 --cwd <仓根>，mutation_tables_handoff 拼），结局文件写一行「后续 门禁 没跑 <那一条命令>」。
  弄坏开关 mutation-tables-in-process 下在这个进程里起它（命令是调用方交进来的 gate33）、把它的退出码当结局。
自上次崩溃验证全绿（门禁 crash-point-replay 不指定范围判绿那一趟在它的日志目录里留的 all-green：git common-dir 里
singlefs-crash-point-replay-logs/*/all-green 最新的那一份；用例一级没有全绿标记了）以来打过的补丁到 CRASH_VERIFY_HINT_PATCHES 个（5，推的）就打一行提示：
只提示、不拦，跑不跑听用户的。
副本锁（防 scripts/mutation/prove-red.sh --copy 正在副本里就地施加变异时，往同一份副本打补丁）：
  --root 指的不是这份脚本所在的主工作区时，开头在 <副本根>/.prove-red.lock 上试拿排他锁（flock，不等），拿到了打完再放、拿不到就拒绝、退 2、一个字没改，
  出路写「副本上有证红在跑，等它跑完或换一份副本」。prove-red.sh 跑的时候拿着同一把锁（它按路径载入这里的 hold_copy_lock / release_copy_lock，锁的写法只在这一处）。
  锁随持有的进程放：正常收尾时删掉锁文件；被杀时内核放掉锁、文件留着，下一个照样拿得到。
  跨机：副本在两台共用的目录里时 flock 不保证两台互斥（两台会同时在同一份副本上跑、写同一份日志），所以拿到 flock 之后再在 <副本根>/.prove-red.owner
  上排他新建持有者记录（机器标识 进程号 随机串 任务号 尝试号；机器标识取 /proc/sys/kernel/random/boot_id，不含主机名；任务号与尝试号取统一任务服务给每次尝试设的
  TASK_SERVER_TASK、TASK_SERVER_ATTEMPT，两台都给，不经服务起的两样都写 -），持有期间每 15 秒刷新记录的修改时间。记录已在：写的是这一台的机器标识，
  flock 拿到了就说明原持有者没了，接手；是别的机器的，修改时间 300 秒内刷新过就当它还占着，拒绝（退 2、一个字不改），超过就当它被杀了，摘掉旧记录接手。
  同一任务更早的尝试（例：第二台上被 oom-kill 的那一次，服务确认清掉之后在本机重起，正落在这 300 秒里）：这一趟经服务起、
  别的机器的记录 300 秒内刷新过，而记录里的任务号与这一趟相同、尝试号比这一趟小，当场接手（摘掉旧记录、写自己的），不等过期。
  依据是服务那一侧的保证：同一任务的第 n+1 次尝试只在第 n 次收尾确认（本机单元与 scope 回读空了、第二台的 peer-cleanup.json 写成 confirmed）之后才排
  （task-server/task_server.py 的 finish_attempt 确认之前尝试留在 starting/running、任务不回队，settle_attempt 才改回 queued）。
  任务号不同的、尝试号不比这一趟小的、这一趟或那条记录不经服务起的，照 300 秒判。拒绝那一句写出记录里的任务号与尝试号（copy_lock_holder）。
  放锁时先删自己的记录。别的任务、不经服务起的持有者在别的机器上被杀时，最多等 300 秒记录过期才让出。
打进主工作区（--root 是这份脚本所在的仓，不给 --root 就是）不拿锁：照「次序」先 git apply --check、再 git apply；别的会话同时改到补丁碰的那几处、核不过的，
  先照「三方合并」重对，合不上照「核的时候就不对」拒、退 2、一个字没改；核过之后到真打之间被改了的，git apply 失败照退 2。合入后验证不在主工作区上编与验（merge-and-verify.sh 在开跑时拷的快照上打与验）。
三方合并（拆分提案第一百八十一节：补丁碰的那一段上面被别的会话加了几行，git apply --check 退 2，手工 patch -F3 重对、套错了没东西拦）：
  git apply --check 不过时，补丁里的每个文件照三份合：补丁的基线（补丁头 index 行左边那个 blob：先在 --root 的对象库里找（git rev-parse <号>^{blob}、cat-file），
  没有再读补丁目录的 baseline/<路径>，那一份的 blob 哈希要以 index 行左边那个号起头；新文件的基线是空的）、补丁改后（临时目录里把基线拼成一棵树、git apply 原补丁）、
  主工作区此刻（--root 里那份文件，不在算空），在临时目录里逐个 git merge-file。全部合得上的，在临时仓里照「主工作区此刻 → 合后」重出一份补丁（--full-index、不认改名），
  再 git apply --check 一遍，后面的变异锚点静态核与真打都用这一份；输出里逐个文件写「三方合并」那几行：补丁原来的段、主工作区此刻比基线多改的段（基线行号）、打进去的段。
  原补丁的哈希照记（打补丁记录里那一列认的是实现员交的那一份）。全程只读 --root（rev-parse、cat-file、apply --check），合与重出补丁都在临时目录里。
  合后每个文件都与主工作区此刻一样（同一份补丁已经打进去了再交一次）的，重出的补丁是空的、要打的补丁照「没有」算：不核走不到的理由（没有新加的行），
  输出写「合后与主工作区此刻一样，没有要打的」，变异表那三份照常核、照常合，补丁本身一个字不打。
  三种不打、退 2、一个字没改：
    冲突：git merge-file 报冲突的文件，逐个点名，并列出撞上的段（基线→主工作区此刻、基线→补丁改后两份 -U0 的段，基线上重叠或紧挨着的成对列出：
      基线第几行、主工作区此刻第几行、补丁改成几行）；出路是退给实现员在主工作区此刻的样子上重做那几段，或主 agent 照点名的段手工并好、重出补丁再打。
    基线取不到：index 行左边那个 blob 对象库里没有、baseline/ 里也没有，或 baseline/ 那一份的哈希对不上，或基线拼成的树上原补丁套不上（index 行点错了 blob）；
      出路是把基线原样放进补丁目录 baseline/<路径>（实现员副本还在的从副本取），副本删了的退给实现员在主工作区此刻的样子上重建副本、重出补丁。
    补丁里有三方合并不认的段：改名、复制、改权限、二进制、没有 ---/+++ 文件头的段（只认文本文件的改、增、删）；出路同上，退给实现员重出补丁。
开工快照（别的会话开的代码轮已取了快照，补丁打进主工作区时改到快照里的文件，那一轮的快照就对不上了）：
  打进主工作区（--root 是这份脚本所在的仓，或自证点名的临时仓）之前，补丁改到的文件——crates.patch 里 --- a/ 与 +++ b/ 两侧的路径，
  mutations-replacements.tsv 或 mutations-delete.txt 写了行就另算 crates/mutations.tsv——落在还没判完的三方轮的开工快照里就拒、退 2、一个字没改。
  只有 mutations-append.tsv 写了行（往 crates/mutations.tsv 只追加）不算改到它、不撞快照（与派发闸 ⑤「crates/mutations.tsv 只追加、不算撞」同义）。哪几轮开着、快照清单怎么读、拒绝说明的列表怎么写在 .claude/hooks/lib_snapshot_conflicts.py（派发闸 ⑥ 调同一份）；
  出路只有两条：等那一轮的判决写完再打（或把改动并进判决），用户已叫停、不写判决的轮在 research/prompts/abandoned-rounds.tsv 登记一行收掉（常设，写法与派发闸 ⑥ 同一份判法）。
  没有单次放行的口子：命令行给 --snapshot-ruled 照用法错退 2、一个字没改。副本上不查（副本不是那一轮取快照的工作区）。
  快照清单读不了（是目录、没有读权限、悬空的链接）的那一份不判，照「放行但必记检出」照打：输出里与别的提醒一样以 ! 起头，逐份点名那一轮与那份清单、写出路
  （lib_snapshot_conflicts.py 的 unreadable_lines），检出记录（AGENT_HOOK_DETECTIONS，没设取 .claude/hooks/lib_work_root.py 的 default_detections_path）
  追加一行、结局只记，会话号取 WORK_DIRECTORY_OWNER_SESSION、CLAUDE_CODE_SESSION_ID、CLAUDE_SESSION_ID 里先设了的那个，看门狗据此报给主 agent；退出码照打的结果，不因它变。
退出码：0 打上了（mutation-tables 那一格没在这里跑，输出末尾打出要另交的整条命令；它红不红看另交的那一趟）；不退 1（弄坏开关 mutation-tables-in-process 下在这个进程里起那一格，红了退 1）；2 核的时候就不对（含三方合并冲突、基线取不到、有不认的段、补丁目录里一样认得的补丁都没有）、补丁改到开着的三方轮开工快照里的文件、副本上有证红在跑、
  --dry-run 的变异锚点静态核红、上一次没换完（文件头「换上与被杀」）、同一个仓有别的在换、核与换之间主工作区被别的会话改了、或用法错（含给了 --snapshot-ruled），一个字没改；
  换上时出错（已就地退回干净的，一个字没留）与回读对不上（留着暂存目录，等 --rollback-pending 或 --finish-pending）也退 2。

代码分块：本文件留换上与被杀（落地、结局文件、暂存目录、回读、--pending-status 等子命令）、自证 selftest 与命令行入口；自证设的接缝（AFTER_EACH_STAGED、AFTER_EACH_REPLACE、AFTER_ALL_REPLACED、
  CRASH_VERIFY_HINT_PATCHES）与进度开关 PROGRESS_ENABLED 是模块级变量，由 globals() 与 global 在本文件里改写，读它们的换上代码也在本文件。
  别的名字写在 apply_writer_patch_plan.py（快照撞车、副本锁、补丁目录与三方合并、走不到的理由、变异锚点静态核与临时树），本文件按名字再导出，别处按路径载入本文件取名字，名字都取得到。
下面「弄坏开关」的开关表仍写在本文件。读开关的代码在哪一份（按开关名带引号搜得到；自证里设开关的写法不在这里列）：
  apply-writer-patch.py：no-hint、copy-lock-ignored、snapshot-ignored、replace-before-verify、staged-verify-skipped、pending-ignored、journal-late、rollback-skips-restore、readback-from-plan、outcome-complete-first、outcome-before-hash-omitted、outcome-reader-no-hash、mutation-tables-in-process
  apply_writer_patch_plan.py：append-duplicates、append-internal-duplicates、apply-unchecked、checker-section-ignored、unreachable-section-ignored、short-reason-accepted、debug-assert-judged、test-module-judged、cfg-test-module-file-judged、error-member-ignored、non-product-crates-judged、unreadable-role-table-as-non-product、role-table-before-patch、all-green-ignored、copy-owner-ignored、earlier-attempt-waits、append-only-mutations-conflict、snapshot-unreadable-unmarked、static-check-skipped、static-check-path-dependencies-skipped、static-check-baseline-ignored、three-way-skipped、three-way-conflict-ignored、baseline-missing-as-empty、baseline-directory-unverified、baseline-patch-failure-ignored、unchanged-merge-judged、three-way-kinds-unchecked、unrecognized-patch-accepted、empty-directory-accepted、static-check-hints-red、static-check-unreachable-row-number-kept、static-check-baseline-on-live-root、static-check-outside-files-not-copied、replacement-preimage-unchecked、replacement-preimage-against-new-row、replacement-preimage-optional、deletion-preimage-unchecked、deletion-preimage-newline-kept、deletion-preimage-optional

弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest() 里那一份补丁目录的名字，括号里是那条「✗ 自检：」的开头）：
  APPLY_WRITER_PATCH_BREAK=append-duplicates  plan() 不核追加的变异名在表里有没有
      duplicate（「✗ 自检：追加已有的名字应当拒绝」）
  APPLY_WRITER_PATCH_BREAK=append-internal-duplicates  plan() 不核追加表自己里有没有重名
      selfduplicate（「✗ 自检：追加表里两行同名」）
  APPLY_WRITER_PATCH_BREAK=apply-unchecked  plan() 里 git apply --check 不过也照样往下走
      stale（「✗ 自检：打不上的补丁应当在核的时候拒绝」：到真打时才失败，报的不是 apply --check）
      连带红的格（三方合并也一起跳过，到真打时 git apply 失败、退 2）：
      「✗ 自检：上下文被别的会话改过、改动不重叠的补丁应当三方合上、退 0、报合了哪个文件哪几段」、
      「✗ 自检：别的会话改了补丁改的同一行，应当判冲突、退 2、点名文件与段落、一个字不改」、
      「✗ 自检：补丁头的基线 blob 在对象库里没有、补丁目录里也没有 baseline/ 时应当退 2、写明基线取不到与出路、一个字不改」、
      「✗ 自检：基线 blob 不在对象库里而补丁目录 baseline/ 里有哈希对得上的那一份时应当照它三方合上、退 0」、
      「✗ 自检：baseline/ 里那一份的哈希对不上补丁头时应当当基线取不到、退 2、一个字不改」、
      「✗ 自检：补丁头点到的基线套不上补丁时应当当基线取不到、退 2、一个字不改」、
      「✗ 自检：带改名段的补丁上下文漂了时应当退 2、写明三方合并只认文本文件的改、增、删、一个字不改」、
      「✗ 自检：补丁已打进时再交同一份应当不崩、照没有改动报」（再交那一趟原补丁照打，git apply 在已打过的文件上失败、退 2）
  APPLY_WRITER_PATCH_BREAK=checker-section-ignored  plan() 不看动了 checker 的补丁报告里有没有「受影响的流与崩溃枚举用例」一节
      checker（「✗ 自检：动 checker 而报告没那一节应当拒绝」）、
      checker-code-named-section（「✗ 自检：动 checker 而报告写的是带代号的节名，应当拒绝」）
  APPLY_WRITER_PATCH_BREAK=unreachable-section-ignored  plan() 不核走不到的理由那一节
      unreachable-unlisted（「✗ 自检：新加 unreachable! 而 result.md 没有「走不到的理由」一节应当拒绝并点名 文件:行」）、
      unreachable-short-reason（「✗ 自检：走不到的理由写得不足 8 个字应当拒绝」）、
      unreachable-member-unlisted（「✗ 自检：新加条款没定的错误成员（名字以 Undecided 收尾）而报告没列应当拒绝并点名成员」）、
      unreachable-unregistered-crate（「✗ 自检：角色表里没有一行的包（singlefs-unregistered）照产品判，新加的 assert! 应当拒绝并点名没登记角色」）、
      unreachable-role-table-unreadable（「✗ 自检：被判那棵树里读不到角色表时每个包照产品判，新加的 assert! 应当拒绝并点名没登记角色」）、
      module-file-declared-without-cfg-test（「✗ 自检：父模块文件里不带 #[cfg(test)] 的 mod helpers; 引进来的文件照产品代码判，里面的 assert! 应当拒绝并点名 文件:行」）、
      module-file-undeclared（「✗ 自检：没有任何父模块文件声明它的文件照产品代码判，里面的 assert! 应当拒绝并点名 文件:行」）
  APPLY_WRITER_PATCH_BREAK=short-reason-accepted  走不到的理由那一节不核理由的字数
      unreachable-short-reason（「✗ 自检：走不到的理由写得不足 8 个字应当拒绝」）
  APPLY_WRITER_PATCH_BREAK=debug-assert-judged  debug_assert! 也当 assert! 要列
      unreachable-debug-assert（「✗ 自检：debug_assert! 不算要列的断言，应当放行」）
  APPLY_WRITER_PATCH_BREAK=test-module-judged  测试模块、测试函数里的断言也要列
      unreachable-test-module（「✗ 自检：测试模块里的 assert! 不算要列的断言，应当放行」）
  APPLY_WRITER_PATCH_BREAK=cfg-test-module-file-judged  父模块文件里 #[cfg(test)] mod <名>; 引进来的整份文件也逐行判（不认模块声明上的测试属性）
      module-file-declared-cfg-test（「✗ 自检：父模块文件里 #[cfg(test)] mod tests; 引进来的 tests.rs 整份算测试代码，里面的 assert! 应当放行」）、
      module-file-ancestor-declared-cfg-test（「✗ 自检：祖先一层 #[cfg(test)] mod a; 引进来的 a/b.rs 整份算测试代码，里面的 assert! 应当放行」）、
      module-file-directory-style（「✗ 自检：#[cfg(test)] mod m; 引进来的 m/mod.rs 与它声明的 m/n.rs 整份算测试代码，里面的 assert! 应当放行」）、
      module-file-path-attribute（「✗ 自检：#[cfg(test)] 与 #[path = …] 叠在同一条 mod 声明上，path 落的那份文件整份算测试代码，里面的 assert! 应当放行」）
  APPLY_WRITER_PATCH_BREAK=error-member-ignored  不认条款没定的错误成员
      unreachable-member-unlisted（「✗ 自检：新加条款没定的错误成员（名字以 Undecided 收尾）而报告没列应当拒绝并点名成员」）
  APPLY_WRITER_PATCH_BREAK=non-product-crates-judged  角色不是产品的包（验证、测试支撑、实验装置）里新加的断言也要列
      unreachable-non-product（「✗ 自检：角色是验证的包（singlefs-verification）里的 assert! 不算产品代码，应当放行」）、
      unreachable-test-support（「✗ 自检：角色是测试支撑的包（singlefs-unit-graph）里的 assert! 不算产品代码，应当放行」）、
      unreachable-experiment-apparatus（「✗ 自检：角色是实验装置的包（singlefs-implementation-experiments）里的 assert! 不算产品代码，应当放行」）、
      unreachable-role-from-patched-table（「✗ 自检：补丁把新包登成实验装置、同一份补丁在它里面新加 assert!，角色照补丁打上之后的角色表判，应当放行」）
  APPLY_WRITER_PATCH_BREAK=unreadable-role-table-as-non-product  被判那棵树里读不到角色表时，每个包都当非产品、一处都不判
      unreachable-role-table-unreadable（「✗ 自检：被判那棵树里读不到角色表时每个包照产品判，新加的 assert! 应当拒绝并点名没登记角色」）
  APPLY_WRITER_PATCH_BREAK=role-table-before-patch  角色表读 --root 里补丁打上之前那一份，不读补丁改后的
      unreachable-role-from-patched-table（「✗ 自检：补丁把新包登成实验装置、同一份补丁在它里面新加 assert!，角色照补丁打上之后的角色表判，应当放行」）
  APPLY_WRITER_PATCH_BREAK=no-hint  apply() 打过的补丁数到 CRASH_VERIFY_HINT_PATCHES 也不打提示
      onlytable（「✗ 自检：打过的补丁数到提示线应当打提示」）
  APPLY_WRITER_PATCH_BREAK=all-green-ignored  数「上次崩溃验证全绿以来」时不认 git common-dir 里 singlefs-crash-point-replay-logs/*/all-green（打过的补丁一律从头数）
      allgreen（「✗ 自检：不指定范围判绿那一趟留下 all-green 之后，打过的补丁数应当从它之后重数、没到提示线不打提示」）
  APPLY_WRITER_PATCH_BREAK=copy-lock-ignored  apply() 不试拿副本锁，副本上有证红在跑也照打
      locked（「✗ 自检：副本锁被占着时应当拒绝、退 2、一个字不改」）
      连带红的格：owner-same-task 里拒绝那一句的一格（「✗ 自检：副本锁的同任务一段：拒绝那一句应当写出记录里的任务号与尝试号」：apply 不拿锁就照打）、
        owner 里拒绝那一句的一格（「✗ 自检：副本锁的跨机一段：别的机器上的证红占着副本（持有者记录刚刷新过）时应当拒绝、退 2、一个字不改」：apply 不拿锁、不核持有者记录就照打）
  APPLY_WRITER_PATCH_BREAK=copy-owner-ignored  hold_copy_lock 不核、不写持有者记录（只靠 flock），别的机器上的证红占着副本也照拿
      owner（「✗ 自检：副本锁的跨机一段：别的机器上的证红占着副本（持有者记录刚刷新过）时应当拒绝」）
      连带红的格：owner-same-task 的六格（不核记录就照拿、也不写自己的记录：该拒的拿到了，该接手的记录没换成这一趟的，apply 拿到锁就照打）：
      「✗ 自检：副本锁的同任务一段：别的机器上的持有者记录刚刷新过、任务号与这一趟相同、尝试号更小时应当当场接手」、
      「✗ 自检：副本锁的同任务一段：别的机器上的持有者记录刚刷新过、任务号与这一趟不同」、
      「✗ 自检：副本锁的同任务一段：别的机器上的持有者记录刚刷新过、任务号相同而尝试号」（尝试号与这一趟相同、比这一趟大的两格）、
      「✗ 自检：副本锁的同任务一段：拒绝那一句应当写出记录里的任务号与尝试号」、
      「✗ 自检：副本锁的同任务一段：这一趟不经服务起」
  APPLY_WRITER_PATCH_BREAK=earlier-attempt-waits  copy_owner_alive_elsewhere 不认同一任务更早的尝试，别的机器上刚刷新过的记录一律等 COPY_OWNER_STALE_SECONDS 过期
      owner-same-task（「✗ 自检：副本锁的同任务一段：别的机器上的持有者记录刚刷新过、任务号与这一趟相同、尝试号更小时应当当场接手」）
  APPLY_WRITER_PATCH_BREAK=snapshot-ignored  apply() 打进主工作区之前不查开着的三方轮开工快照
      snapshot（「✗ 自检：补丁改到开着的三方轮开工快照里的文件应当拒绝、退 2、一个字不改」）；snapshot-append-only 与 snapshot-ruled-refused 照绿（分不出差别）
      连带红的格：「✗ 自检：快照清单读不了时应当照打、点名那一轮与那几份清单」
  APPLY_WRITER_PATCH_BREAK=append-only-mutations-conflict  开工快照把只追加变异行也算改到 crates/mutations.tsv
      snapshot-append-only（「✗ 自检：只追加变异行的补丁不算撞快照（crates/mutations.tsv 在开着的那一轮的清单里），应当照打、退 0」）
      连带红的格：「✗ 自检：快照清单读不了时应当照打、点名那一轮与那几份清单」（那一份补丁也只追加变异行，撞上 r-open 被拒）
  APPLY_WRITER_PATCH_BREAK=snapshot-unreadable-unmarked  转给 lib_snapshot_conflicts.py 的同名开关：读不了的快照清单不分出来、照旧只记进提醒，不打放行但必记检出那几行、不记检出
      snapshot-unreadable（「✗ 自检：快照清单读不了时应当照打、点名那一轮与那几份清单、写明放行但必记检出、检出记录多一行」）
  APPLY_WRITER_PATCH_BREAK=static-check-skipped  --dry-run 不跑变异锚点静态核
      anchorbreak（「✗ 自检：弄断变异锚点的补丁 --dry-run 应当拒绝」）
      连带红的格：「✗ 自检：不弄断锚点的补丁 --dry-run 应当放行并报 CHECKED」、「✗ 自检：补丁前就红的锚点不该拦住不相干的补丁」、
      「✗ 自检：弄断了变异锚点、补丁前也有别的红行时，--dry-run 应当只拒补丁带来的那一处」、
      「✗ 自检：补丁与变异行锚在 path 依赖目录里时 --dry-run 应当照收」（pathdep 那一格也要 --dry-run 报 CHECKED，不跑静态核就没有这一句）、
      「✗ 自检：补丁删了前面一行变异、补丁前就有的 UNREACHABLE 行号跟着漂，--dry-run 应当照补丁前就红的放行」（不跑静态核就没有「补丁前就红的 1 处」那一句提醒）、
      「✗ 自检：补丁删了前面一行变异、补丁前就有的 RELEASE 行号跟着漂，--dry-run 应当照补丁前就红的放行」（同上）、
      「✗ 自检：补丁改的是拷贝范围之外的文件（拒算表）时 --dry-run 应当照常判」（outsidecopy 那一格也要 --dry-run 报 CHECKED，不跑静态核就没有这一句）、
      「✗ 自检：没打补丁那一棵应当是同一刻从主工作区拷出的树、不是此刻的活树」（不跑静态核退 0、不点名 keep-anchor）
  APPLY_WRITER_PATCH_BREAK=static-check-path-dependencies-skipped  --dry-run 的临时树不调 copy_closure，只拷 crates/ 那几处与补丁改到的文件（锚在 prune-server/src 这类 path 依赖目录里、
      补丁没改到的文件上的变异行，两棵临时树上都找不到文件，两边都红、当成补丁前就红的放行并提醒；pathdep 那一格的 served/src/other.rs 上的变异行应当在两棵树上都判过、不带提醒）
      pathdep（「✗ 自检：补丁与变异行锚在 path 依赖目录里时 --dry-run 应当照收」）
  APPLY_WRITER_PATCH_BREAK=static-check-baseline-ignored  --dry-run 不拿没打补丁的树当基线，补丁前就红的行也算补丁带来的
      anchorpre（「✗ 自检：补丁前就红的锚点不该拦住不相干的补丁」）
      连带红的格：「✗ 自检：弄断了变异锚点、补丁前也有别的红行时，--dry-run 应当只拒补丁带来的那一处」、
      「✗ 自检：补丁删了前面一行变异、补丁前就有的 UNREACHABLE 行号跟着漂，--dry-run 应当照补丁前就红的放行」（补丁前就有的 far-unreachable 算成补丁带来的、被拒）、
      「✗ 自检：补丁删了前面一行变异、补丁前就有的 RELEASE 行号跟着漂，--dry-run 应当照补丁前就红的放行」（同上）
  APPLY_WRITER_PATCH_BREAK=static-check-baseline-on-live-root  --dry-run 的基线直接跑活的 --root、不跑同一刻拷出的补丁之前那一棵（两趟 static-check 之间主工作区被别的会话临时改了又改回，补丁弄断的行在基线上同样红，被当成补丁之前就红的放行）
      livebaseline（「✗ 自检：没打补丁那一棵应当是同一刻从主工作区拷出的树、不是此刻的活树」）
  APPLY_WRITER_PATCH_BREAK=static-check-outside-files-not-copied  --dry-run 的两棵临时树只放拷贝范围那几处，不放补丁改到的拷贝范围之外的文件（拒算表、角色表这一类），补丁在临时树上套不上、退 2 报静态核没做成
      outsidecopy（「✗ 自检：补丁改的是拷贝范围之外的文件（拒算表）时 --dry-run 应当照常判」）
  APPLY_WRITER_PATCH_BREAK=three-way-skipped  plan() 里 git apply --check 不过就拒，不做三方合并
      threeway-merged（「✗ 自检：上下文被别的会话改过、改动不重叠的补丁应当三方合上、退 0、报合了哪个文件哪几段」）
      连带红的格：「✗ 自检：别的会话改了补丁改的同一行，应当判冲突、退 2、点名文件与段落、一个字不改」、
      「✗ 自检：补丁头的基线 blob 在对象库里没有、补丁目录里也没有 baseline/ 时应当退 2、写明基线取不到与出路、一个字不改」、
      「✗ 自检：基线 blob 不在对象库里而补丁目录 baseline/ 里有哈希对得上的那一份时应当照它三方合上、退 0」、
      「✗ 自检：baseline/ 里那一份的哈希对不上补丁头时应当当基线取不到、退 2、一个字不改」、
      「✗ 自检：补丁头点到的基线套不上补丁时应当当基线取不到、退 2、一个字不改」、
      「✗ 自检：带改名段的补丁上下文漂了时应当退 2、写明三方合并只认文本文件的改、增、删、一个字不改」、
      「✗ 自检：补丁已打进时再交同一份应当不崩、照没有改动报」（再交那一趟 git apply --check 不过就拒、退 2）
  APPLY_WRITER_PATCH_BREAK=three-way-conflict-ignored  三方合并不看 git merge-file 报的冲突数，带冲突标记的合后文件照打
      threeway-conflict（「✗ 自检：别的会话改了补丁改的同一行，应当判冲突、退 2、点名文件与段落、一个字不改」）
      连带红的格：「✗ 自检：打不上的补丁应当在核的时候拒绝」（stale 那一份也是同一行三方都不同）
  APPLY_WRITER_PATCH_BREAK=baseline-missing-as-empty  基线 blob 对象库里没有、baseline/ 里也没有时不拒，当空文件去合
      threeway-baseline-missing（「✗ 自检：补丁头的基线 blob 在对象库里没有、补丁目录里也没有 baseline/ 时应当退 2、写明基线取不到与出路、一个字不改」）
  APPLY_WRITER_PATCH_BREAK=baseline-directory-unverified  baseline/ 里那一份不核哈希就当基线
      threeway-baseline-mismatch（「✗ 自检：baseline/ 里那一份的哈希对不上补丁头时应当当基线取不到、退 2、一个字不改」）
  APPLY_WRITER_PATCH_BREAK=baseline-patch-failure-ignored  基线拼成的树上原补丁套不上也往下合（补丁改后当成基线原样）
      threeway-baseline-wrong（「✗ 自检：补丁头点到的基线套不上补丁时应当当基线取不到、退 2、一个字不改」：补丁改后就是基线，合出来等于主工作区此刻，照没有要打的退 0）
  APPLY_WRITER_PATCH_BREAK=unchanged-merge-judged  三方合并合后与主工作区此刻一样（要打的补丁是 None）时 plan_with_three_way 照样核走不到的理由，None 交给 unreachable_sites 去读、抛 TypeError
      reapplied（「✗ 自检：补丁已打进时再交同一份应当不崩、照没有改动报」）
  APPLY_WRITER_PATCH_BREAK=three-way-kinds-unchecked  不拒改名、复制、改权限、二进制、没有文件头的段，照文本文件去合
      threeway-unsupported（「✗ 自检：带改名段的补丁上下文漂了时应当退 2、写明三方合并只认文本文件的改、增、删、一个字不改」）
  APPLY_WRITER_PATCH_BREAK=unrecognized-patch-accepted  补丁目录里一样认得的补丁都没有、只有别的 *.patch 时 unrecognized_refusal() 不拒（照「补丁 没有」报打上了、退 0，一个字没打）
      unrecognized（「✗ 自检：补丁目录里只有 all.patch、没有一样认得的补丁，应当拒绝、退 2、点名 all.patch、出路写 git apply、一个字不改」）
  APPLY_WRITER_PATCH_BREAK=empty-directory-accepted  一样补丁都没有（也没有别的 *.patch）的目录 unrecognized_refusal() 不拒
      emptydirectory（「✗ 自检：一样补丁都没有的目录应当拒绝、退 2、点名没有认得的补丁、一个字不改」）
  APPLY_WRITER_PATCH_BREAK=replace-before-verify  暂存时每暂存一份就把改后全文当场就地写进主工作区（边算边写），不等全部暂存、核过
      stagekill（「✗ 自检：暂存到一半被杀，主工作区应当一个字不动、下一次照常打上」）、stagecorrupt（「✗ 自检：暂存出来的某一份与算好的 sha256 对不上时应当拒绝、退 2、主工作区一个字不动」）
      连带红的格（暂存时已经全写进主工作区，换上那一步被杀时三份都已经是改后，不是只换上 emit.rs 一份）：halfkill（「✗ 自检：换上换到一半被杀，下一次起应当认出来、列出已换上与没换上的文件和两条出路、不在半截上再打别的补丁、一个字不改」）、
      outcomekilled（「✗ 自检：换到一半被杀的那一次结局文件只有开始与文件行、没有完成，--read-outcome 应当退 3、逐个文件列已换上与没换上」）
  APPLY_WRITER_PATCH_BREAK=staged-verify-skipped  暂存完不回读核暂存出来的文件、备份与主工作区此刻的 sha256
      stagecorrupt（「✗ 自检：暂存出来的某一份与算好的 sha256 对不上时应当拒绝、退 2、主工作区一个字不动」）
  APPLY_WRITER_PATCH_BREAK=pending-ignored  不认上一次没换完的暂存目录，别的补丁照打（盖掉那份暂存）
      halfkill（「✗ 自检：换上换到一半被杀，下一次起应当认出来、列出已换上与没换上的文件和两条出路、不在半截上再打别的补丁、一个字不改」）、
      readbackmismatch（「✗ 自检：换上之后回读与记下的改后 sha256 对不上时应当退 2、写异常、不写完成、留着暂存目录」，它靠拒绝说明点名被别的改动动过的文件）
  APPLY_WRITER_PATCH_BREAK=journal-late  journal.json 等全部换上之后才写（被杀在换上中间时暂存目录里没有它，下一次认成暂存到一半被杀）
      halfkill（「✗ 自检：换上换到一半被杀，下一次起应当认出来、列出已换上与没换上的文件和两条出路、不在半截上再打别的补丁、一个字不改」）
      连带红的格：halfrollback（「✗ 自检：换到一半被杀之后 --rollback-pending 应当照改前原样退回、主工作区与打之前逐字节相同、之后同一个补丁照常打上」）、 history-kept: 引的是自证失败行原文，按原文匹配
      halffinish（「✗ 自检：换到一半被杀之后 --finish-pending 应当接着换完、主工作区与整份打完逐字节相同、结局文件写完成」）、
      gatehandoff（「✗ 自检：真打与 --finish-pending 都不应当在本进程里起 mutation-tables 那一格」：gatehandofffinish 那一份同样被杀在换上中间，--finish-pending 认成暂存到一半、清掉，打不出那一条命令）
  APPLY_WRITER_PATCH_BREAK=rollback-skips-restore  --rollback-pending 不把备份放回去，只清暂存目录
      halfrollback（「✗ 自检：换到一半被杀之后 --rollback-pending 应当照改前原样退回、主工作区与打之前逐字节相同、之后同一个补丁照常打上」） history-kept: 引的是自证失败行原文，按原文匹配
  APPLY_WRITER_PATCH_BREAK=readback-from-plan  回读不读主工作区，直接抄记下的改后 sha256
      readbackmismatch（「✗ 自检：换上之后回读与记下的改后 sha256 对不上时应当退 2、写异常、不写完成、留着暂存目录」）
  APPLY_WRITER_PATCH_BREAK=outcome-complete-first  结局文件开始时就把完成行写上
      outcomenormal（「✗ 自检：真打之后补丁目录里应当有结局文件：开始、每个文件的改前 sha256、回读、完成、后续依次写全，--read-outcome 应当退 0 判打全了」）、 history-kept: 引的是自证失败行原文，按原文匹配
      outcomekilled（「✗ 自检：换到一半被杀的那一次结局文件只有开始与文件行、没有完成，--read-outcome 应当退 3、逐个文件列已换上与没换上」）、
      readbackmismatch（「✗ 自检：换上之后回读与记下的改后 sha256 对不上时应当退 2、写异常、不写完成、留着暂存目录」，它判结局文件里不该有完成行）
  APPLY_WRITER_PATCH_BREAK=outcome-before-hash-omitted  结局文件的文件行不写补丁前 sha256（写 -）
      outcomenormal（「✗ 自检：真打之后补丁目录里应当有结局文件：开始、每个文件的改前 sha256、回读、完成、后续依次写全，--read-outcome 应当退 0 判打全了」）、 history-kept: 引的是自证失败行原文，按原文匹配
      outcomekilled（「✗ 自检：换到一半被杀的那一次结局文件只有开始与文件行、没有完成，--read-outcome 应当退 3、逐个文件列已换上与没换上」）
  APPLY_WRITER_PATCH_BREAK=outcome-reader-no-hash  --read-outcome 不对着主工作区现算 sha256，信文件里写的
      outcomekilled（「✗ 自检：换到一半被杀的那一次结局文件只有开始与文件行、没有完成，--read-outcome 应当退 3、逐个文件列已换上与没换上」）、
      outcomestale（「✗ 自检：打全之后文件又被改了，--read-outcome 应当退 4 并点名那一份」）
  APPLY_WRITER_PATCH_BREAK=static-check-hints-red  --dry-run 的变异锚点静态核把可达性那一道的四种提示行也算红（REACHABILITY_NOT_JUDGED 的原因里写着被判那棵树的路径，
      临时树与 --root 不同，门禁样本那种小根上每个补丁都多一处「补丁前的树上没有」的红）
      anchorkept（「✗ 自检：不弄断锚点的补丁 --dry-run 应当放行并报 CHECKED」）、anchorpre（「✗ 自检：补丁前就红的锚点不该拦住不相干的补丁」）、
      pathdep（「✗ 自检：补丁与变异行锚在 path 依赖目录里时 --dry-run 应当照收」）、
      outsidecopy（「✗ 自检：补丁改的是拷贝范围之外的文件（拒算表）时 --dry-run 应当照常判」）、
      reachabilityhints（「✗ 自检：可达性那一道的提示行（REACHABILITY_CHECKED、REACHABILITY_NOT_JUDGED、REACHABILITY_EXEMPT、REACHABILITY_UNJUDGED）随树变也不算红」）
  APPLY_WRITER_PATCH_BREAK=static-check-unreachable-row-number-kept  --dry-run 的变异锚点静态核比基线时 UNREACHABLE 行连第二段的变异表行号一起比
      （STATIC_CHECK_ROW_NUMBER_KINDS 里没有它：补丁删掉前面一行变异，补丁前就有的 UNREACHABLE 行号少 1，被当成补丁带来的红拒掉）
      unreachablerowshift（「✗ 自检：补丁删了前面一行变异、补丁前就有的 UNREACHABLE 行号跟着漂，--dry-run 应当照补丁前就红的放行」）
  APPLY_WRITER_PATCH_BREAK=static-check-release-row-number-kept  --dry-run 的变异锚点静态核比基线时 RELEASE 行连第二段的变异表行号一起比
      （STATIC_CHECK_ROW_NUMBER_KINDS 里没有它：补丁删掉前面一行变异，补丁前就有的 RELEASE 行号少 1，被当成补丁带来的红拒掉）
      releaserowshift（「✗ 自检：补丁删了前面一行变异、补丁前就有的 RELEASE 行号跟着漂，--dry-run 应当照补丁前就红的放行」）
  APPLY_WRITER_PATCH_BREAK=mutation-tables-in-process  真打与 --finish-pending 在本进程里起 mutation-tables 那一格（调用方交进来的 gate33），把它的退出码当结局
      gatehandoff（「✗ 自检：真打与 --finish-pending 都不应当在本进程里起 mutation-tables 那一格」）
  APPLY_WRITER_PATCH_BREAK=replacement-preimage-unchecked  preimage_problems() 不比对整行替换的前像，照名字换
      preimage-replace-mismatched（「✗ 自检：整行替换有一行前像对不上表里此刻那一行时应当整份拒绝、退 2、逐行列出行名与两侧原文、出路写以此刻的行为基线重出、一个字不改」）
  APPLY_WRITER_PATCH_BREAK=replacement-preimage-against-new-row  preimage_problems() 拿替换的新行（前六段）去比表里此刻那一行，前像对得上的替换也被拒
      preimage-replace-matched（「✗ 自检：整行替换的前像与表里此刻那一行逐字相同时应当照换、退 0」）
      连带红的格：「✗ 自检：好的补丁应当打上、按名字合表」（那一份也换一行）、
      「✗ 自检：整行替换有一行前像对不上表里此刻那一行时应当整份拒绝、退 2、逐行列出行名与两侧原文、出路写以此刻的行为基线重出、一个字不改」（前像对得上的 swap 也被点名）
  APPLY_WRITER_PATCH_BREAK=replacement-preimage-optional  mutation_edits() 收只有六段（没带前像）的替换行，照名字换、不比对
      preimage-replace-without-preimage（「✗ 自检：替换表里有一行只有六段（没带前像）时应当整份拒绝、退 2、一个字不改」）
  APPLY_WRITER_PATCH_BREAK=deletion-preimage-unchecked  preimage_problems() 不比对删除的前像，照名字删
      preimage-delete-mismatched（「✗ 自检：删除有一行前像对不上表里此刻那一行时应当整份拒绝、退 2、逐行列出行名与两侧原文、一个字不改」）
  APPLY_WRITER_PATCH_BREAK=deletion-preimage-newline-kept  preimage_problems() 删除那一侧拿带行尾换行符的原行去比，前像对得上的删除也被拒
      preimage-delete-matched（「✗ 自检：删除的前像与表里此刻那一行逐字相同时应当照删、退 0」）
      连带红的格：「✗ 自检：好的补丁应当打上、按名字合表」（那一份也删一行）、
      「✗ 自检：删除有一行前像对不上表里此刻那一行时应当整份拒绝、退 2、逐行列出行名与两侧原文、一个字不改」（前像对得上的 drop 也被点名）、
      「✗ 自检：补丁删了前面一行变异、补丁前就有的 UNREACHABLE 行号跟着漂，--dry-run 应当照补丁前就红的放行」（那一份删 drop-before，被拒）、
      「✗ 自检：补丁删了前面一行变异、补丁前就有的 RELEASE 行号跟着漂，--dry-run 应当照补丁前就红的放行」（同一份补丁，同样被拒）
  APPLY_WRITER_PATCH_BREAK=deletion-preimage-optional  mutation_edits() 收只写名字（没带前像）的删除行，照名字删、不比对
      preimage-delete-names-only（「✗ 自检：删除表里有一行只写了名字（没带前像）时应当整份拒绝、退 2、一个字不改」）
"""

import collections
import fcntl
import glob
import hashlib
import importlib.util
import json
import os
import re
import secrets
import shlex
import shutil
import stat
import subprocess
import sys
import tempfile
import threading
import time

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402
sys.path.append(SCRIPT_DIRECTORY)
from lib_rust_test_module_files import CRATE_SOURCE_PATH, INNER_TEST_ATTRIBUTE, TEST_ATTRIBUTE, attribute_end, blank_out  # noqa: E402  去注释与字面量、认测试属性（整份文件的认法也在这一份）
from lib_rust_test_module_files import whole_test_files as whole_test_files_in_tree  # noqa: E402

sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
from apply_writer_patch_plan import (
     SCRIPT_DIRECTORY, REPOSITORY_ROOT, BROKEN, CHECKER_SECTION, UNREACHABLE_SECTION, UNREACHABLE_ROW, UNREACHABLE_MINIMUM_REASON_CHARACTERS,
     CRATE_ROLE_TABLE, CRATE_ROLE_TABLE_COLUMNS, PRODUCT_ROLE, PRODUCT_SOURCE_PATH, UNREACHABLE_MACRO, UNREACHABLE_MACRO_LOOSE, UNDECIDED_SUFFIXES,
     UNDECIDED_MEMBER, HUNK_HEADER, LOG_NAME, STATIC_CHECK_SCRIPT, STATIC_CHECK_TREE_PATHS, STATIC_CHECK_DECLARED, REPOSITORY_COPY_LIBRARY,
     STATIC_CHECK_ALLOWED_KINDS, STATIC_CHECK_HINT_KINDS, STATIC_CHECK_ROW_NUMBER_KINDS, STATIC_CHECK_TIMEOUT_SECONDS, CRASH_REPLAY_LOGS_NAME,
     ALL_GREEN_RUN_GLOB, COPY_LOCK_NAME, COPY_OWNER_NAME, COPY_OWNER_HEARTBEAT_SECONDS, COPY_OWNER_STALE_SECONDS, MACHINE_IDENTITY_PATH,
     SERVICE_TASK_VARIABLE, SERVICE_ATTEMPT_VARIABLE, NOT_FROM_SERVICE, COPY_OWNER_FIELDS, COPY_OWNERS, BASELINE_DIRECTORY, RECOGNIZED_PATCH_FILES,
     THREE_WAY_UNSUPPORTED_HEADERS, THREE_WAY_LABELS, SNAPSHOT_TREE_SCRIPT, OUTCOME_NAME, PENDING_DIRECTORY_NAME, LANDING_LOCK_NAME, JOURNAL_NAME,
     ABSENT, STATE_LANDED, STATE_NOT_LANDED, STATE_TOUCHED, ACTION_CHANGE, ACTION_CREATE, ACTION_DELETE, COMMAND_NAME, SNAPSHOT_CONFLICTS_LIBRARY,
     _SNAPSHOT_CONFLICTS, snapshot_conflicts_library, MUTATION_TABLE, MUTATION_FILES, MUTATION_ROW_FIELDS, REPLACEMENT_FIELDS, PREIMAGE_WAY_OUT,
     mutation_edits, tabs_shown, preimage_problems, patch_file_paths, touched_paths, SESSION_VARIABLES,
     snapshot_library_break, snapshot_touched_paths, snapshot_refusal_remedy, snapshot_refusal, hold_copy_lock, machine_identity,
     service_attempt_of_this_run, earlier_attempt_of_this_task, copy_owner_alive_elsewhere, keep_copy_owner_fresh, claim_copy_owner,
     release_copy_owner, release_copy_lock, copy_lock_holder, git, table_rows, _SNAPSHOT_TREE_MODULE, snapshot_tree_module, PatchFile, ThreeWay,
     THREE_WAY_CONFLICT_REMEDY, THREE_WAY_BASELINE_REMEDY, THREE_WAY_REBUILD_REMEDY, patch_file_sections, blob_hash, baseline_of, merge_one_file,
     zero_context_hunks, line_span, base_span, describe_base_hunk, colliding_hunks, describe_collision, rebased_patch, three_way_merge,
     added_lines_of_patch, test_code_lines, whole_test_files, patched_texts, crate_role_table_text, crate_roles_by_directory, unreachable_sites,
     unreachable_report_rows, unreachable_problems, plan, unrecognized_refusal, plan_with_three_way, patches_since_last_crash_verification,
     static_check_command, run_static_check, static_check_red, _REPOSITORY_COPY_MODULE, repository_copy_module, static_check_tree_items,
     copy_environment, static_check_copied_items, build_static_check_trees, static_check_report, unsafe_path_reason,
)  # noqa: E402,F401

CRASH_VERIFY_HINT_PATCHES = int(os.environ.get("CRASH_VERIFY_HINT_PATCHES", "5"))   # 推的，没量过


PROGRESS_ENABLED = False   # 命令行入口才开：每一步之前打一行进度并立刻刷出去（自证里不打）
# 自证的接缝（只在自证里设；平常都是 None）：在暂存、换上的某个点上插一刀，模拟被杀（os._exit）或别人动了文件
AFTER_EACH_STAGED = None   # (序号, 暂存出来的文件路径)：每暂存一份之后
AFTER_EACH_REPLACE = None   # (序号)：每换上一份之后
AFTER_ALL_REPLACED = None   # (仓根)：全部换上之后、回读之前


def progress(text):
    """一步之前打一行进度并立刻刷出去（命令行入口才开）：后面的步骤被杀时，日志里至少留着走到了哪一步。"""
    if PROGRESS_ENABLED:
        print(f"  · 进度：{text}", flush=True)


def file_digest(path):
    """路径上普通文件的 sha256（一兆一兆读，不整份读进内存）；文件不在交 ABSENT；是目录、读不了抛 OSError。"""
    digest = hashlib.sha256()
    try:
        with open(path, "rb") as handle:
            for block in iter(lambda: handle.read(1 << 20), b""):
                digest.update(block)
    except FileNotFoundError:
        return ABSENT
    return digest.hexdigest()


def absolute_git_directory(root):
    """root 的 git-dir 绝对路径（暂存目录与锁放在里面）；root 不是 git 仓交 None。"""
    done = git(root, "rev-parse", "--absolute-git-dir")
    return done.stdout.strip() if done.returncode == 0 and done.stdout.strip() else None


def sync_directory(path):
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def write_atomically(path, text):
    """同目录临时文件写完、刷盘，再改名换上；读的人看到旧的全文或新的全文，没有半截。"""
    path = os.path.abspath(path)
    temporary = f"{path}.{os.getpid()}.tmp"
    with open(temporary, "w", encoding="utf-8") as handle:
        handle.write(text)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(temporary, path)
    sync_directory(os.path.dirname(path))


def copy_synced(source, target, mode):
    """source 流着拷到不存在的 target、刷盘、设权限位。"""
    with open(source, "rb") as reader, open(target, "xb") as writer:
        shutil.copyfileobj(reader, writer, 1 << 20)
        writer.flush()
        os.fsync(writer.fileno())
    os.chmod(target, mode)


OUTCOME_COMMENTS = (
    "# 上面是这份结局文件的开始行；格式见 scripts/gate/apply-writer-patch.py 文件头「结局文件」，制表符分隔，每行第一段是行种类：",
    "#   开始 补丁目录 仓根 进程号 开始秒数 | 文件 路径 动作 改前sha256 计划改后sha256 | 回读 路径 回读算出的改后sha256 | 完成 文件数 秒数 | 退回 文件数 秒数 | 异常 说明 点名的文件 | 后续 门禁 退出码 末行",
    f"# 没有完成也没有退回就是没打完：对着主工作区现算每个文件的 sha256，等于计划改后的已换上，等于改前的没换上（{COMMAND_NAME} --read-outcome <本文件>）",
)


def outcome_text(rows):
    return "".join("\t".join(str(field) for field in row) + "\n" for row in rows)


def outcome_append(path, rows):
    with open(path, "a", encoding="utf-8") as handle:
        handle.write(outcome_text(rows))
        handle.flush()
        os.fsync(handle.fileno())


def outcome_start(path, patch_directory, root, entries):
    """结局文件整份重写，只有开始行、说明、每个要动的文件一行（补丁前与计划改后的 sha256）；弄坏开关见文件头。"""
    files = [("文件", entry["path"], entry["action"], ABSENT if BROKEN == "outcome-before-hash-omitted" else entry["before"], entry["after"])
             for entry in entries]
    if BROKEN == "outcome-complete-first":
        files.append(("完成", len(entries), int(time.time())))
    start = ("开始", os.path.abspath(patch_directory), os.path.abspath(root), os.getpid(), int(time.time()))
    write_atomically(path, outcome_text([start]) + "".join(f"{line}\n" for line in OUTCOME_COMMENTS) + outcome_text(files))


def entry_state(root, entry):
    """一份文件的换上状态，按内容判：现在的 sha256 等于改后是已换上，等于补丁前是没换上，都不等是被别的改动动过。"""
    try:
        current = file_digest(os.path.join(root, entry["path"]))
    except OSError:
        return STATE_TOUCHED
    if current == entry["after"]:
        return STATE_LANDED
    if current == entry["before"]:
        return STATE_NOT_LANDED
    return STATE_TOUCHED


def remove_empty_parents(root, target):
    """删了文件之后，往上把空了的目录一层层删掉（git apply 删文件也这样），删到仓根为止。"""
    directory = os.path.dirname(target)
    while len(directory) > len(root) and directory.startswith(root + os.sep):
        try:
            os.rmdir(directory)
        except OSError:
            return
        directory = os.path.dirname(directory)


def replace_one(root, base, entry):
    """把暂存的改后全文改名换到主工作区（同一个文件系统里 os.replace，读的人看到旧的全文或新的全文）；删的 unlink。"""
    target = os.path.join(root, entry["path"])
    if entry["action"] == ACTION_DELETE:
        os.unlink(target)
        remove_empty_parents(root, target)
    else:
        os.makedirs(os.path.dirname(target), exist_ok=True)
        os.replace(os.path.join(base, "staged", str(entry["index"])), target)


def restore_one(root, base, entry):
    """把备份的补丁前原样改名放回去；这一份是新建的就删掉。"""
    target = os.path.join(root, entry["path"])
    if entry["action"] == ACTION_CREATE:
        os.unlink(target)
        remove_empty_parents(root, target)
    else:
        os.makedirs(os.path.dirname(target), exist_ok=True)
        os.replace(os.path.join(base, "before", str(entry["index"])), target)


def replace_entries(root, base, entries):
    """一份份换上；已经是改后的跳过（接着换完时前面换过的不再动）。"""
    for position, entry in enumerate(entries):
        if entry_state(root, entry) != STATE_LANDED:
            replace_one(root, base, entry)
        if AFTER_EACH_REPLACE is not None:
            AFTER_EACH_REPLACE(position)


def restore_entries(root, base, entries):
    """已换上的照备份退回（倒着退）→ 没退成的说明清单；现在的内容既不是补丁前也不是改后的不碰（会盖掉别人的改动）。"""
    failures = []
    for entry in reversed(entries):
        state = entry_state(root, entry)
        if state == STATE_NOT_LANDED:
            continue
        if state == STATE_TOUCHED:
            failures.append(f"{entry['path']}：现在的内容既不是改前也不是计划的改后，被别的改动动过，没退")
            continue
        try:
            if BROKEN != "rollback-skips-restore":
                restore_one(root, base, entry)
        except OSError as error:
            failures.append(f"{entry['path']}：退回时出错：{error}")
            continue
        if entry_state(root, entry) != STATE_NOT_LANDED:
            failures.append(f"{entry['path']}：退回之后回读，sha256 不等于改前")
    return failures


def seal_landing(root, base, entries, outcome_path):
    """全部换上之后：回读每份算 sha256 写进结局文件，全对上记下的改后才写完成、删暂存目录 → (对上了, 说明几行)。"""
    if AFTER_ALL_REPLACED is not None:
        AFTER_ALL_REPLACED(root)
    rows, mismatched = [], []
    for entry in entries:
        actual = entry["after"] if BROKEN == "readback-from-plan" else file_digest(os.path.join(root, entry["path"]))
        rows.append(("回读", entry["path"], actual))
        if actual != entry["after"]:
            mismatched.append(entry["path"])
    outcome_append(outcome_path, rows)
    if mismatched:
        outcome_append(outcome_path, [("异常", "回读与记下的改后 sha256 对不上", "、".join(mismatched))])
        return False, [f"✗ 换上之后回读，这几份的 sha256 与记下的改后对不上：{'、'.join(mismatched)}（换的时候被别的改动动过？）",
                       f"→ 怎么办：暂存目录 {base} 留着，不在半截上再打别的补丁；{COMMAND_NAME} --pending-status 看每份此刻的状态，"
                       "别的改动该留的人处理好之后再决定接着换完还是退回"]
    outcome_append(outcome_path, [("完成", len(entries), int(time.time()))])
    shutil.rmtree(base)
    return True, []


def write_journal(base, patch_directory, root, outcome_path, entries):
    journal = {"patch_directory": os.path.abspath(patch_directory), "root": root, "outcome": outcome_path, "pid": os.getpid(),
               "started": int(time.time()), "entries": entries}
    write_atomically(os.path.join(base, JOURNAL_NAME), json.dumps(journal, ensure_ascii=False, indent=1) + "\n")


def hold_landing_lock(git_directory):
    """<git-dir> 的换上锁，flock 不等：拿到交打开的描述符（里面写进程号），被占着交 None。被杀时内核放锁。"""
    descriptor = os.open(os.path.join(git_directory, LANDING_LOCK_NAME), os.O_RDWR | os.O_CREAT | os.O_CLOEXEC, 0o644)
    try:
        fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError:
        os.close(descriptor)
        return None
    os.ftruncate(descriptor, 0)
    os.write(descriptor, f"{os.getpid()}\n".encode())
    return descriptor


def landing_lock_holder(git_directory):
    try:
        with open(os.path.join(git_directory, LANDING_LOCK_NAME), encoding="utf-8") as handle:
            return handle.read().strip() or "不明"
    except OSError:
        return "不明"


def pending_snapshot(root, git_directory):
    """上一次换上留下的暂存目录 → (种类, 暂存目录, 记录, [(一份, 状态)])。种类：none 没有；unfinished-staging 有目录没有 journal.json（暂存到一半被杀，主工作区没动）；
    unreadable journal.json 读不了；journal 有记录。"""
    if git_directory is None:
        return "none", None, None, []
    base = os.path.join(git_directory, PENDING_DIRECTORY_NAME)
    if not os.path.isdir(base):
        return "none", base, None, []
    journal_path = os.path.join(base, JOURNAL_NAME)
    if not os.path.isfile(journal_path):
        return "unfinished-staging", base, None, []
    try:
        with open(journal_path, encoding="utf-8") as handle:
            journal = json.load(handle)
        states = [(entry, entry_state(root, entry)) for entry in journal["entries"]]
    except (OSError, ValueError, KeyError, TypeError):
        return "unreadable", base, None, []
    return "journal", base, journal, states


def pending_lines(kind, base, journal, states, holder=None):
    """没换完的暂存目录打给人看的几行：已换上、没换上、被别的改动动过各列哪几份，两条出路。"""
    if kind == "unreadable":
        return [f"✗ 上一次打补丁留下的暂存目录 {base} 里的 {JOURNAL_NAME} 读不了（坏了或被改过），不在半截上再打别的补丁",
                f"→ 怎么办：一个字都没改。人看 {base} 里 staged/ 与 before/ 的文件（改后与改前），对着主工作区此刻的样子确认没有半截的改动之后，删掉 {base} 再打"]
    groups = {STATE_LANDED: [], STATE_NOT_LANDED: [], STATE_TOUCHED: []}
    for entry, state in states:
        groups[state].append(entry)
    lines = [f"✗ 上一次打补丁没换完（被杀或中途出错）：补丁目录 {journal['patch_directory']}，仓根 {journal['root']}，共 {len(states)} 份，"
             f"{len(groups[STATE_LANDED])} 份已换上、{len(groups[STATE_NOT_LANDED])} 份没换上、{len(groups[STATE_TOUCHED])} 份被别的改动动过；不在半截上再打别的补丁"]
    if holder is not None:
        lines.append(f"! 换上锁此刻被进程 {holder} 占着：那一次可能还在换，先等它跑完再看")
    for state in (STATE_LANDED, STATE_NOT_LANDED, STATE_TOUCHED):
        lines += [f"    {state}：{entry['path']}（{entry['action']}）" for entry in groups[state]]
    lines.append(f"→ 怎么办：一个字都没改。结局文件 {journal['outcome']}（只有开始与文件行，没有完成）；暂存目录 {base}")
    if groups[STATE_TOUCHED]:
        lines.append("  有被别的改动动过的文件：接着换完与退回都会盖掉别人的改动，两条都不做；人看那几份此刻的内容、暂存目录里 staged/<序号>（改后）与 before/<序号>（改前），"
                     f"处理好之后删掉暂存目录 {base}，再打")
    else:
        lines.append(f"  接着换完（照记下的暂存换上没换的，回读、写完成、打出 mutation-tables 那一格要另交的命令）：{COMMAND_NAME} --finish-pending --root {journal['root']}")
        lines.append(f"  照改前原样退回（备份放回去，回读核过、写退回）：{COMMAND_NAME} --rollback-pending --root {journal['root']}")
    return lines


def pending_refusal(root):
    """上一次没换完就拒：交拒绝说明的几行，没有交 []（真打与 --dry-run 一样先查，不在半截上再打别的补丁）。暂存到一半被杀的不算（主工作区没动，换上时自动清掉）。"""
    if BROKEN == "pending-ignored":
        return []
    git_directory = absolute_git_directory(root)
    kind, base, journal, states = pending_snapshot(root, git_directory)
    if kind in ("none", "unfinished-staging"):
        return []
    descriptor = hold_landing_lock(git_directory)
    holder = None
    if descriptor is None:
        holder = landing_lock_holder(git_directory)
    else:
        os.close(descriptor)
    return pending_lines(kind, base, journal, states, holder)


Landed = collections.namedtuple("Landed", "code lines count outcome")
Image = collections.namedtuple("Image", "path action before after source mode")   # source：暂存要拷的改后全文（临时目录里），删的是 None；mode：改后的权限位


def compute_images(root, patch, merged, work):
    """每一份要动的文件的改后全文，在临时目录 work 里算好 → (Image 清单, 问题清单)：补丁套在临时目录里补丁前原样的那几份上（git apply），变异表合并后的全文另写一份。
    只读 root；补丁前与改后一样的不进清单。问题清单非空就一个字不改。"""
    images, problems = [], []
    tree = os.path.join(work, "tree")
    os.mkdir(tree)
    paths = sorted(patch_file_paths(patch)) if patch else []
    for path in paths:
        reason = unsafe_path_reason(path)
        source = os.path.join(root, path)
        if reason is not None:
            problems.append(f"补丁里的路径 {path}：{reason}")
        elif os.path.islink(source) or (os.path.exists(source) and not os.path.isfile(source)):
            problems.append(f"补丁要动的 {path} 在主工作区里是符号链接或目录：这一步只认普通文件")
        elif os.path.isfile(source):
            copy = os.path.join(tree, path)
            os.makedirs(os.path.dirname(copy), exist_ok=True)
            shutil.copyfile(source, copy)
            shutil.copymode(source, copy)
    if problems:
        return [], problems
    if paths:
        subprocess.run(["git", "init", "-q", tree], capture_output=True, text=True, check=True)
        applied = subprocess.run(["git", "-C", tree, "-c", "core.autocrlf=false", "apply", os.path.abspath(patch)], capture_output=True, text=True)
        if applied.returncode != 0:
            return [], [f"git apply 失败（在临时目录里改前原样的文件上套补丁）：{applied.stderr.strip()[:300]}；工作区在核与打之间被改了？git status 看一眼再跑"]
    for path in paths:
        after_file = os.path.join(tree, path)
        after_exists = os.path.isfile(after_file)
        before, after = file_digest(os.path.join(root, path)), file_digest(after_file) if after_exists else ABSENT
        if before == after:
            continue
        action = ACTION_CREATE if before == ABSENT else (ACTION_CHANGE if after_exists else ACTION_DELETE)
        images.append(Image(path, action, before, after, after_file if after_exists else None,
                            stat.S_IMODE(os.stat(after_file).st_mode) if after_exists else None))
    if merged is not None:
        table = os.path.join(root, MUTATION_TABLE)
        new_table = os.path.join(work, "mutations.tsv.new")
        with open(new_table, "w", encoding="utf-8") as handle:
            handle.write(merged)
        before, after = file_digest(table), file_digest(new_table)
        if before != after:
            images.append(Image(MUTATION_TABLE, ACTION_CHANGE, before, after, new_table, stat.S_IMODE(os.stat(table).st_mode)))
    return images, []


def stage_images(root, base, images):
    """每份改后的全文拷进暂存目录 staged/<序号>、补丁前原样拷进 before/<序号> → entries（journal.json 里记的那些）。"""
    entries = []
    for index, image in enumerate(images):
        entry = {"index": index, "path": image.path, "action": image.action, "before": image.before, "after": image.after}
        staged = os.path.join(base, "staged", str(index))
        if image.source is not None:
            copy_synced(image.source, staged, image.mode)
        target = os.path.join(root, image.path)
        if image.action != ACTION_CREATE:
            copy_synced(target, os.path.join(base, "before", str(index)), stat.S_IMODE(os.stat(target).st_mode))
        entries.append(entry)
        if AFTER_EACH_STAGED is not None:
            AFTER_EACH_STAGED(index, staged)
        if BROKEN == "replace-before-verify" and image.source is not None:
            shutil.copyfile(staged, target)   # 弄坏开关：暂存一份就当场就地写进主工作区
    return entries


def staging_problems(root, base, entries):
    """暂存齐了之后回读核：暂存出来的改后全文、备份的补丁前原样的 sha256 与记下的对得上，主工作区此刻的 sha256 仍是核的时候的补丁前。"""
    problems = []
    for entry in entries:
        index = str(entry["index"])
        if entry["after"] != ABSENT and file_digest(os.path.join(base, "staged", index)) != entry["after"]:
            problems.append(f"{entry['path']}：暂存出来的改后全文的 sha256 与算好的对不上")
        if entry["before"] != ABSENT and file_digest(os.path.join(base, "before", index)) != entry["before"]:
            problems.append(f"{entry['path']}：备份的改前原样的 sha256 与记下的改前对不上（备份的时候文件被改了）")
        if BROKEN != "replace-before-verify" and file_digest(os.path.join(root, entry["path"])) != entry["before"]:
            problems.append(f"{entry['path']}：主工作区此刻的 sha256 已经不是核的时候的改前（核与换之间被别的会话改了）")
    return problems


def land(patch_directory, root, images):
    """把算好的几份文件换进主工作区（文件头「换上与被杀」）：拿锁、认上一次没换完的、暂存、核、写结局文件与 journal.json、一份份换上、回读 → Landed。"""
    root = os.path.abspath(root)
    outcome = os.path.join(os.path.abspath(patch_directory), OUTCOME_NAME)
    git_directory = absolute_git_directory(root)
    if git_directory is None:
        return Landed(2, [f"✗ {root} 不是 git 仓（git rev-parse --absolute-git-dir 失败）：暂存目录与换上锁放在 git-dir 里，没处放",
                          "→ 怎么办：一个字都没改。--root 给一个 git 仓的根"], 0, outcome)
    lock = hold_landing_lock(git_directory)
    if lock is None:
        return Landed(2, [f"✗ 同一个仓 {root} 上有别的打补丁在换上（换上锁被进程 {landing_lock_holder(git_directory)} 占着）",
                          "→ 怎么办：一个字都没改。等它跑完再打"], 0, outcome)
    try:
        return land_locked(patch_directory, root, git_directory, images, outcome)
    finally:
        os.close(lock)


def land_locked(patch_directory, root, git_directory, images, outcome):
    base = os.path.join(git_directory, PENDING_DIRECTORY_NAME)
    kind, _, journal, states = pending_snapshot(root, git_directory)
    if kind == "unfinished-staging" or (kind != "none" and BROKEN == "pending-ignored"):
        shutil.rmtree(base)   # 暂存到一半被杀留下的：主工作区没动，清掉重来
    elif kind != "none":
        return Landed(2, pending_lines(kind, base, journal, states), 0, outcome)
    try:
        for directory in (base, os.path.join(base, "staged"), os.path.join(base, "before")):
            os.mkdir(directory)
        if os.stat(base).st_dev != os.stat(root).st_dev:
            shutil.rmtree(base)
            return Landed(2, [f"✗ 暂存目录 {base} 与仓根 {root} 不在同一个文件系统：改名换上（os.replace）跨文件系统做不到",
                              "→ 怎么办：一个字都没改。--root 给 git-dir 与工作区在同一个文件系统上的仓"], 0, outcome)
        entries = stage_images(root, base, images)
        problems = [] if BROKEN == "staged-verify-skipped" else staging_problems(root, base, entries)
        if problems:
            shutil.rmtree(base, ignore_errors=True)
            return Landed(2, [f"✗ {problem}" for problem in problems] + ["→ 怎么办：主工作区一个字都没改（暂存目录已清掉）；照每处看一眼，再跑一次"], 0, outcome)
        outcome_start(outcome, patch_directory, root, entries)
        if BROKEN != "journal-late":
            write_journal(base, patch_directory, root, outcome, entries)
    except OSError as error:
        shutil.rmtree(base, ignore_errors=True)
        return Landed(2, [f"✗ 暂存或写结局文件 {outcome} 时出错：{error}", "→ 怎么办：主工作区一个字都没改（暂存目录已清掉）；看盘满没满、git-dir 与补丁目录可不可写，再跑一次"], 0, outcome)
    progress(f"核过了，要动 {len(entries)} 份文件，结局文件 {outcome} 已写好开始与文件行；开始换上")
    try:
        replace_entries(root, base, entries)
        if BROKEN == "journal-late":
            write_journal(base, patch_directory, root, outcome, entries)
    except Exception as error:   # 不是被杀，是换的时候出了错：就地照备份退回
        return undo_after_error(root, base, entries, outcome, error)
    try:
        sealed, lines = seal_landing(root, base, entries, outcome)
    except OSError as error:
        return Landed(2, [f"✗ 回读或写结局文件 {outcome} 时出错：{error}",
                          f"→ 怎么办：暂存目录 {base} 留着，{COMMAND_NAME} --pending-status 看每份此刻的状态"], 0, outcome)
    if not sealed:
        return Landed(2, lines, 0, outcome)
    kinds = collections.Counter(entry["action"] for entry in entries)
    return Landed(0, [f"换上 {len(entries)} 份文件（改 {kinds[ACTION_CHANGE]}、增 {kinds[ACTION_CREATE]}、删 {kinds[ACTION_DELETE]}），回读每份的 sha256 都等于记下的改后；结局文件 {outcome}"],
                  len(entries), outcome)


def undo_after_error(root, base, entries, outcome, error):
    """换的时候出错（不是被杀）：已换上的照备份退回。退干净了写退回、删暂存目录；退不干净留着暂存目录和异常行。"""
    failures = restore_entries(root, base, entries)
    try:
        if failures:
            outcome_append(outcome, [("异常", f"换上时出错（{error}），就地退回没退干净", "、".join(failure.split("：", 1)[0] for failure in failures))])
        else:
            outcome_append(outcome, [("退回", len(entries), int(time.time()))])
    except OSError:
        pass
    if failures:
        return Landed(2, [f"✗ 换上时出错：{error}；就地退回没退干净："] + [f"    {failure}" for failure in failures]
                      + [f"→ 怎么办：暂存目录 {base} 留着，不在半截上再打别的补丁；{COMMAND_NAME} --pending-status 看每份此刻的状态"], 0, outcome)
    shutil.rmtree(base, ignore_errors=True)
    return Landed(2, [f"✗ 换上时出错：{error}；已换上的照备份原样退回，回读核过，主工作区一个字没留", "→ 怎么办：看上面的错误（盘满、权限），修好再跑一次"], 0, outcome)


def mutation_tables_handoff(root):
    """mutation-tables 那一格要主 agent 经服务另交的整条命令（文件头「mutation-tables 那一格」）：带范围声明，--cwd 是仓根，根参数也是仓根。"""
    scope = ("种类=items; 跑什么=门禁 code-source-discipline.sh 的 mutation-tables 格; 范围=1 格; "
             "为什么=打补丁脚本换上补丁之后不在自己进程里起这一格（它要十几 GiB），另交一趟核变异表")
    quoted = shlex.quote(os.path.abspath(root))
    return (f"SINGLEFS_RUN_SCOPE={shlex.quote(scope)} task-server/task submit --cwd {quoted} -- "
            f"bash .claude/gate.d/code-source-discipline.sh --check mutation-tables {quoted}")


def after_landing(patch_directory, root, gate33, lines, outcome):
    """全部换上、回读核过之后的尾巴：记打补丁记录、打出 mutation-tables 那一格要另交的命令、结局文件记一行后续、数提示线 → (退出码, 输出行)。
    mutation-tables 那一格不在这个进程里起（文件头「mutation-tables 那一格」）；gate33 只在弄坏开关 mutation-tables-in-process 下在这里起。"""
    common = git(root, "rev-parse", "--git-common-dir").stdout.strip()
    log_path = os.path.join(common if os.path.isabs(common) else os.path.join(root, common), LOG_NAME)
    handed = os.path.join(patch_directory, "crates.patch")   # 记实现员交的那一份（三方合并重出的那一份只在临时目录里）
    digest = hashlib.sha256(open(handed, "rb").read()).hexdigest() if os.path.isfile(handed) and os.path.getsize(handed) > 0 else "-"
    with open(log_path, "a", encoding="utf-8") as handle:
        handle.write(f"{time.time()}\t{os.path.abspath(patch_directory)}\t{digest}\n")
    if BROKEN == "mutation-tables-in-process":
        progress("补丁已全部换上、回读核过、记进打补丁记录；弄坏开关 mutation-tables-in-process：照改前在这个进程里跑 code-source-discipline 的 mutation-tables 那一格")
        gate = subprocess.run(gate33, cwd=root, capture_output=True, text=True)
        gate_line = ([line for line in gate.stdout.splitlines() if line.strip()] or ["（没有输出）"])[-1]
        gate_code, gate_state, gate_text, schedule_state = gate.returncode, str(gate.returncode), gate_line.strip(), str(gate.returncode)
        lines.append(f"code-source-discipline：退出码 {gate.returncode}，{gate_text}")
    else:
        gate_code, gate_state, gate_text, schedule_state = 0, "没跑", mutation_tables_handoff(root), "待另交"
        lines += ["! mutation-tables 那一格（code-source-discipline.sh --check mutation-tables，量的占内存十几 GiB，文件头「mutation-tables 那一格」）没在这里跑：补丁已整份换上、回读核过、记进打补丁记录；"
                  "要主 agent 经服务另交这一条（它红不红看那一趟）：", f"  {gate_text}"]
    try:
        outcome_append(outcome, [("后续", "门禁", gate_state, gate_text)])
    except OSError:
        pass
    count = patches_since_last_crash_verification(root, log_path)
    if count >= CRASH_VERIFY_HINT_PATCHES and BROKEN != "no-hint":
        lines.append(f"! 自上次崩溃验证全绿以来已打 {count} 个实现员补丁（提示线 {CRASH_VERIFY_HINT_PATCHES}，推的）：该由主 agent "
                     "经服务交一趟不指定范围的 crash-point-replay 了（按条、按块都照键复用，只判输入变了的）；只提示，不拦")
    lines.append(f"调度表可贴的一行：| {os.path.basename(os.path.normpath(patch_directory))} | 已打（{time.strftime('%m-%d %H:%M', time.localtime())}） | "
                 f"code-source-discipline {schedule_state} |")
    return (0 if gate_code == 0 else 1), lines


def pending_status(root):
    """--pending-status：上一次有没有没换完的 → (退出码, 行)。0 没有（含暂存到一半被杀的，主工作区没动），3 有。"""
    root = os.path.abspath(root)
    kind, base, journal, states = pending_snapshot(root, absolute_git_directory(root))
    if kind == "none":
        return 0, ["没有没换完的：暂存目录不在"]
    if kind == "unfinished-staging":
        return 0, [f"暂存目录 {base} 在，可是没有 {JOURNAL_NAME}：上一次暂存到一半被杀，主工作区一个字没动；下一次真打自动清掉"]
    return 3, pending_lines(kind, base, journal, states)


def with_pending(root, action):
    """--finish-pending、--rollback-pending 共用：拿换上锁、认暂存目录，有 journal.json 的交给 action(root, base, journal, states) → (退出码, 行)。"""
    root = os.path.abspath(root)
    git_directory = absolute_git_directory(root)
    if git_directory is None:
        return 2, [f"✗ {root} 不是 git 仓（git rev-parse --absolute-git-dir 失败）", "→ 怎么办：--root 给一个 git 仓的根"]
    lock = hold_landing_lock(git_directory)
    if lock is None:
        return 2, [f"✗ 同一个仓 {root} 上有别的打补丁在换上（换上锁被进程 {landing_lock_holder(git_directory)} 占着）", "→ 怎么办：等它跑完再来"]
    try:
        kind, base, journal, states = pending_snapshot(root, git_directory)
        if kind == "none":
            return 0, ["没有没换完的：暂存目录不在，什么都没做"]
        if kind == "unfinished-staging":
            shutil.rmtree(base)
            return 0, [f"暂存目录 {base} 里没有 {JOURNAL_NAME}：上一次暂存到一半被杀，主工作区一个字没动；已清掉"]
        if kind == "unreadable":
            return 2, pending_lines(kind, base, journal, states)
        return action(root, base, journal, states)
    finally:
        os.close(lock)


def finish_pending(root, gate33):
    """--finish-pending：照记下的暂存把没换上的换完，回读、写完成，再走 after_landing。有被别的改动动过的文件不做。"""
    def action(root, base, journal, states):
        if any(state == STATE_TOUCHED for _, state in states):
            return 2, pending_lines("journal", base, journal, states)
        entries = journal["entries"]
        missing = [entry["path"] for entry, state in states if state == STATE_NOT_LANDED and entry["action"] != ACTION_DELETE
                   and not os.path.isfile(os.path.join(base, "staged", str(entry["index"])))]
        if missing:
            return 2, [f"✗ 没换上的这几份在暂存目录里找不到改后的全文：{'、'.join(missing)}",
                       f"→ 怎么办：一个字都没改。{COMMAND_NAME} --rollback-pending --root {root} 照改前原样退回"]
        progress(f"接着换完：{sum(1 for _, state in states if state == STATE_NOT_LANDED)} 份没换上，补丁目录 {journal['patch_directory']}")
        try:
            replace_entries(root, base, entries)
            sealed, lines = seal_landing(root, base, entries, journal["outcome"])
        except Exception as error:
            return 2, [f"✗ 接着换的时候出错：{error}", f"→ 怎么办：暂存目录 {base} 留着，{COMMAND_NAME} --pending-status 看每份此刻的状态"]
        if not sealed:
            return 2, lines
        return after_landing(journal["patch_directory"], root, gate33, [f"✓ 接着换完了：补丁目录 {journal['patch_directory']}，共 {len(entries)} 份，回读都等于记下的改后；结局文件 {journal['outcome']}"],
                             journal["outcome"])
    return with_pending(root, action)


def rollback_pending(root):
    """--rollback-pending：已换上的照备份退回，回读核过（等于补丁前），写退回、删暂存目录。有被别的改动动过的文件不做。"""
    def action(root, base, journal, states):
        if any(state == STATE_TOUCHED for _, state in states):
            return 2, pending_lines("journal", base, journal, states)
        entries = journal["entries"]
        failures = restore_entries(root, base, entries)
        if failures:
            return 2, ["✗ 没退回干净："] + [f"    {failure}" for failure in failures] + [f"→ 怎么办：暂存目录 {base} 留着，{COMMAND_NAME} --pending-status 看每份此刻的状态"]
        notes = []
        try:
            outcome_append(journal["outcome"], [("退回", len(entries), int(time.time()))])
        except OSError as error:
            notes.append(f"! 结局文件 {journal['outcome']} 写不了退回行：{error}")
        shutil.rmtree(base)
        return 0, [f"✓ 照改前原样退回了：补丁目录 {journal['patch_directory']}，{len(entries)} 份，回读都等于改前，主工作区一个字没留；结局文件 {journal['outcome']}"] + notes
    return with_pending(root, action)


def read_outcome(path, root_override=None):
    """--read-outcome：只读结局文件、对着主工作区现算每个文件的 sha256，判打全了没有（文件头「结局文件」）→ (退出码, 行)。"""
    try:
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
    except (OSError, UnicodeError) as error:
        return 2, [f"✗ 读不了结局文件 {path}：{error}"]
    rows = [line.split("\t") for line in text.splitlines() if line.strip() and not line.startswith("#")]
    starts = [row for row in rows if row[0] == "开始" and len(row) >= 5]
    files = [row for row in rows if row[0] == "文件" and len(row) >= 5]
    if not starts:
        return 2, [f"✗ {path} 不是结局文件：没有「开始」行", "→ 怎么办：核对路径；格式见 scripts/gate/apply-writer-patch.py 文件头「结局文件」"]
    root = root_override or starts[-1][2]
    current = {}
    for row in files:
        try:
            current[row[1]] = row[4] if BROKEN == "outcome-reader-no-hash" else file_digest(os.path.join(root, row[1]))
        except OSError:
            current[row[1]] = "?"
    readback = {row[1]: row[2] for row in rows if row[0] == "回读" and len(row) >= 3}
    ending = next((row for row in reversed(rows) if row[0] in ("完成", "退回", "异常")), None)
    if ending is not None and ending[0] == "完成":
        changed = [row for row in files if current[row[1]] != readback.get(row[1])]
        if not changed:
            return 0, [f"✓ 打全了：{path}（仓根 {root}）有完成行，{len(files)} 份文件现在的 sha256 都等于回读的改后"]
        return 4, [f"✗ 完成过，可是这几份现在的 sha256 不等于回读的改后（之后被改了，或被退回了）：{path}（仓根 {root}）"] \
            + [f"    {row[1]}（回读 {readback.get(row[1], '（没有）')[:12]}，现在 {current[row[1]][:12]}）" for row in changed]
    if ending is not None and ending[0] == "退回":
        changed = [row for row in files if current[row[1]] != row[3]]
        if not changed:
            return 1, [f"已退回：{path}（仓根 {root}）有退回行，{len(files)} 份文件现在的 sha256 都等于改前，补丁一个字没留"]
        return 4, [f"✗ 退回过，可是这几份现在的 sha256 不等于改前（之后被改了，或又打上了）：{path}（仓根 {root}）"] \
            + [f"    {row[1]}（改前 {row[3][:12]}，现在 {current[row[1]][:12]}）" for row in changed]
    lines = [f"✗ 没打完：{path}（仓根 {root}）有开始行，没有完成也没有退回（被杀或中途出错）；对着主工作区现算 sha256："]
    for row in files:
        state = STATE_LANDED if current[row[1]] == row[4] else STATE_NOT_LANDED if current[row[1]] == row[3] else STATE_TOUCHED
        lines.append(f"    {state}：{row[1]}（{row[2]}；改前 {row[3][:12]}，计划改后 {row[4][:12]}，现在 {current[row[1]][:12]}）")
    lines.append(f"→ 怎么办：{COMMAND_NAME} --pending-status --root {root} 看暂存目录里的记录，再 --finish-pending 接着换完或 --rollback-pending 照改前原样退回")
    return 3, lines


def apply(patch_directory, root, dry_run, gate33, main_workspace=None, static_check=None):
    """副本上先试拿副本锁（文件头「副本锁」），拿到了再核、再打，打完放锁；主工作区不拿锁（文件头「打进主工作区」），
    核之前先查开着的三方轮开工快照（文件头「开工快照」）。上一次没换完的先拒（文件头「换上与被杀」），真打与 --dry-run 一样。
    main_workspace 不给就按 root 是不是这份脚本所在的仓判（自证给 True，把临时仓当主工作区）。"""
    pending = pending_refusal(root)
    if pending:
        return 2, pending
    if main_workspace is None:
        main_workspace = os.path.realpath(root) == os.path.realpath(REPOSITORY_ROOT)
    if main_workspace:
        refusal, notes = ([], []) if BROKEN == "snapshot-ignored" else snapshot_refusal(patch_directory, root)
        if refusal:
            return 2, refusal
        code, lines = check_then_apply(patch_directory, root, dry_run, gate33, static_check)
        return code, lines + notes
    lock = None
    if BROKEN != "copy-lock-ignored":
        try:
            lock = hold_copy_lock(root)
        except OSError as error:
            return 2, [f"✗ 在 {root} 上建不了副本锁 {COPY_LOCK_NAME}：{error}", "→ 怎么办：看 --root 指的目录在不在、可不可写；一个字都没改"]
        if lock is None:
            return 2, [f"✗ 副本 {root} 上有证红在跑（{COPY_LOCK_NAME} 被进程 {copy_lock_holder(root)} 占着），这一次不打",
                       "→ 怎么办：副本上有证红在跑，等它跑完或换一份副本；一个字都没改"]
    try:
        return check_then_apply(patch_directory, root, dry_run, gate33, static_check)
    finally:
        if lock is not None:
            release_copy_lock(root, lock)


def check_then_apply(patch_directory, root, dry_run, gate33, static_check=None):
    """核（含三方合并，文件头「三方合并」）、再打；三方合并的临时目录打完就删。"""
    problems, merged, patch, three_way = plan_with_three_way(patch_directory, root)
    try:
        return apply_planned(patch_directory, root, dry_run, gate33, static_check, problems, merged, patch, three_way)
    finally:
        if three_way is not None:
            shutil.rmtree(three_way.work, ignore_errors=True)


def apply_planned(patch_directory, root, dry_run, gate33, static_check, problems, merged, patch, three_way):
    merge_lines = three_way.lines if three_way is not None else []
    if problems:
        return 2, ([f"✗ {problem}" for problem in problems] + ([three_way.remedy] if three_way is not None and three_way.remedy else [])
                   + ["→ 怎么办：一个字都没改。照每处改补丁目录（或退给实现员），再跑一次"])
    if dry_run:
        refusal, notes = static_check_report(root, merged, patch, static_check)
        if refusal:
            return 2, merge_lines + refusal
        return 0, [f"✓ 核过了（--dry-run，没改）：补丁 {'有' if patch else '没有'}，变异表合并之后 {merged.count(chr(10))} 行"] + merge_lines + notes
    work = tempfile.mkdtemp(prefix="apply-writer-patch-images-")
    try:
        progress("核过了，在临时目录里算每份改后的全文")
        images, trouble = compute_images(root, patch, merged, work)
        if trouble:
            return 2, [f"✗ {item}" for item in trouble] + ["→ 怎么办：一个字都没改。照每处看一眼，再跑一次"]
        landed = land(patch_directory, root, images)
    except (OSError, subprocess.CalledProcessError) as error:
        return 2, [f"✗ 算改后的全文没做成：{error}", "→ 怎么办：一个字都没改。看盘满没满、临时目录可不可写，再跑一次"]
    finally:
        shutil.rmtree(work, ignore_errors=True)
    if landed.code != 0:
        return landed.code, landed.lines
    lines = [f"✓ 打上了：{os.path.basename(os.path.normpath(patch_directory))}（补丁 {'有' if patch else '没有'}）"] + merge_lines + landed.lines
    return after_landing(patch_directory, root, gate33, lines, landed.outcome)


SAMPLE_FILES = (("crates/demo/src/emit.rs", "fn emit() -> u32 { 1 }\n", "fn emit() -> u32 { 2 }\n"),
                ("crates/demo/src/parse.rs", "fn parse() -> u32 { 1 }\n", "fn parse() -> u32 { 2 }\n"))   # (路径, 补丁前, 改后)：自证里换上的几份样本源文件
SAMPLE_TABLE = "# 表头\nkeep\tf\to\tr\t-p demo --lib\tt\n"
SAMPLE_APPEND = "added\tf\to\tr\t-p demo --lib\tt\n"
SAMPLE_GATE = ["true"]
SAMPLE_REGISTRATION_TABLE = ".claude/gate.d/stage-inputs.tsv"   # static-check 读被判那棵树里的这张登记表（文件头「变异锚点静态核」）：真跑 static-check 的样本树要自带，没有它打 ERROR 退 2
SAMPLE_REGISTRATION_ROW = "crates-mutation-replay.sh\t-\tcopy-range=crates/ copy-range=litmus/ copy-range=Cargo.toml copy-range=Cargo.lock copy-range=.cargo/\t# 自证样本\n"   # 拷贝范围就是 STATIC_CHECK_TREE_PATHS 那几样
SAMPLE_ORDER = [path for path, _, _ in SAMPLE_FILES] + [MUTATION_TABLE]   # 一份样本补丁动的三份文件，换上的次序（补丁里的路径排序，变异表最后）


def sample_repository(work, name):
    """自证用的一个临时仓和一个补丁目录：仓里两份源文件与变异表已提交；补丁目录里的 crates.patch 把两份源文件各改一行，mutations-append.tsv 追加一行 → (仓, 补丁目录)。"""
    repo, directory = os.path.join(work, f"{name}-repo"), os.path.join(work, f"{name}-patch")
    os.makedirs(directory)
    for path, old, _ in SAMPLE_FILES:
        os.makedirs(os.path.dirname(os.path.join(repo, path)), exist_ok=True)
        with open(os.path.join(repo, path), "w", encoding="utf-8") as handle:
            handle.write(old)
    with open(os.path.join(repo, MUTATION_TABLE), "w", encoding="utf-8") as handle:
        handle.write(SAMPLE_TABLE)
    for command in (["init", "-q"], ["add", "-A"], ["-c", "user.name=s", "-c", "user.email=s@s", "commit", "-qm", "base"]):
        git(repo, *command, check=True)
    for path, _, new in SAMPLE_FILES:
        with open(os.path.join(repo, path), "w", encoding="utf-8") as handle:
            handle.write(new)
    diff = git(repo, "diff", "--", "crates/demo").stdout
    for path, old, _ in SAMPLE_FILES:
        with open(os.path.join(repo, path), "w", encoding="utf-8") as handle:
            handle.write(old)
    with open(os.path.join(directory, "crates.patch"), "w", encoding="utf-8") as handle:
        handle.write(diff)
    with open(os.path.join(directory, "mutations-append.tsv"), "w", encoding="utf-8") as handle:
        handle.write(SAMPLE_APPEND)
    return repo, directory


def sample_after(before):
    """样本补丁整份打上之后树里每份文件的字节（before 是打之前的 tree_bytes）。"""
    after = dict(before)
    for path, _, new in SAMPLE_FILES:
        after[path] = new.encode()
    after[MUTATION_TABLE] = (SAMPLE_TABLE + SAMPLE_APPEND).encode()
    return after


def write_sample_registration_table(root):
    """真跑 static-check 的样本树放上登记表（SAMPLE_REGISTRATION_TABLE）：只有 crates-mutation-replay.sh 那一行，拷贝范围与临时树里放的那几样相同。"""
    path = os.path.join(root, *SAMPLE_REGISTRATION_TABLE.split("/"))
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(SAMPLE_REGISTRATION_ROW)


def tree_bytes(tree):
    """树里除 .git 与副本锁文件、持有者记录之外每份文件的字节：判「一个字不改」。"""
    contents = {}
    for directory, subdirectories, files in os.walk(tree):
        subdirectories[:] = [name for name in subdirectories if name != ".git"]
        for name in files:
            if name not in (COPY_LOCK_NAME, COPY_OWNER_NAME):
                with open(os.path.join(directory, name), "rb") as handle:
                    contents[os.path.relpath(os.path.join(directory, name), tree)] = handle.read()
    return contents


def outcome_rows(path):
    """结局文件里的行（不含说明行）；文件不在交 []。"""
    try:
        with open(path, encoding="utf-8") as handle:
            return [line.rstrip("\n").split("\t") for line in handle if line.strip() and not line.startswith("#")]
    except FileNotFoundError:
        return []


def pending_directory(repo):
    return os.path.join(repo, ".git", PENDING_DIRECTORY_NAME)


def killed_after(position):
    """自证的接缝：第 position 份（0 起）之后 os._exit，模拟被 SIGKILL（不跑 finally、不清暂存目录，内核放锁）。"""
    def hook(index, *_):
        if index == position:
            os._exit(137)
    return hook


def run_in_child(seam, hook, call):
    """子进程里把接缝 seam 设成 hook 再跑 call()（hook 里 os._exit 模拟被杀）→ 子进程退出码；子进程出了任何异常退 99，不往父进程的 finally 里走。"""
    pid = os.fork()
    if pid == 0:
        try:
            globals()[seam] = hook
            call()
            os._exit(0)
        except BaseException:
            os._exit(99)
    _, status = os.waitpid(pid, 0)
    return os.waitstatus_to_exitcode(status)


SAME_TASK_SELFTEST_TASK, SAME_TASK_SELFTEST_ATTEMPT = "7896", "3"   # 副本锁的同任务一段：这一趟当成服务的任务 7896 第 3 次尝试


def same_task_owner_failures(root, owner_path, patch_directory, table_path, gate33):
    """副本锁的同任务一段（文件头「副本锁」的同一任务更早的尝试）→ 失败说明的清单。持有者记录写着别的机器的标识、刚刷新过（这里自己不拿 flock，演 flock 跨机不互斥），
    这一趟当成服务的任务 7896 第 3 次尝试（设 SERVICE_TASK_VARIABLE、SERVICE_ATTEMPT_VARIABLE，跑完照原样放回）：
    记录是同一任务第 2 次写的，当场接手、记录换成这一趟的、放锁后不留；任务号不同的、同一任务第 3 次与第 4 次写的拒绝；这一趟不经服务起的，同一任务的记录也照拒；
    apply 的拒绝那一句写出记录里的任务号与尝试号、一个字不改。"""
    failures = []
    saved = {name: os.environ.get(name) for name in (SERVICE_TASK_VARIABLE, SERVICE_ATTEMPT_VARIABLE)}

    def held_with_record(record):
        """写下持有者记录 record、试拿副本锁：→ (拿到了没有, 拿着时记录切成的几段)；拿到的当场放掉。"""
        with open(owner_path, "w") as handle:
            handle.write(record)
        held = hold_copy_lock(root)
        if held is None:
            return False, []
        written = open(owner_path).read().split() if os.path.exists(owner_path) else []
        release_copy_lock(root, held)
        return True, written

    try:
        os.environ[SERVICE_TASK_VARIABLE], os.environ[SERVICE_ATTEMPT_VARIABLE] = SAME_TASK_SELFTEST_TASK, SAME_TASK_SELFTEST_ATTEMPT
        held, written = held_with_record(f"other-machine-selftest 4242 selftest-token {SAME_TASK_SELFTEST_TASK} 2\n")
        if not held or written[:1] != [machine_identity()] or written[3:] != [SAME_TASK_SELFTEST_TASK, SAME_TASK_SELFTEST_ATTEMPT] or os.path.exists(owner_path):
            failures.append(f"副本锁的同任务一段：别的机器上的持有者记录刚刷新过、任务号与这一趟相同、尝试号更小时应当当场接手（记录换成这一台的标识与任务 "
                            f"{SAME_TASK_SELFTEST_TASK} 第 {SAME_TASK_SELFTEST_ATTEMPT} 次）、放锁后不留记录，实际{'拿到了' if held else '没拿到'}锁、拿着时记录是 {written}、"
                            f"放锁后记录{'还在' if os.path.exists(owner_path) else '已删'}")
        held, _ = held_with_record("other-machine-selftest 4242 selftest-token 7000 1\n")
        if held:
            failures.append("副本锁的同任务一段：别的机器上的持有者记录刚刷新过、任务号与这一趟不同（7000）时应当拒绝，实际拿到了锁")
        for written_attempt in (SAME_TASK_SELFTEST_ATTEMPT, str(int(SAME_TASK_SELFTEST_ATTEMPT) + 1)):
            held, _ = held_with_record(f"other-machine-selftest 4242 selftest-token {SAME_TASK_SELFTEST_TASK} {written_attempt}\n")
            if held:
                failures.append(f"副本锁的同任务一段：别的机器上的持有者记录刚刷新过、任务号相同而尝试号（{written_attempt}）不比这一趟（{SAME_TASK_SELFTEST_ATTEMPT}）小时应当拒绝，"
                                "实际拿到了锁")
        before = open(table_path).read()
        with open(owner_path, "w") as handle:
            handle.write("other-machine-selftest 4242 selftest-token 7000 1\n")
        code, lines = apply(patch_directory, root, False, gate33)
        after_refusal = open(table_path).read()
        if code != 2 or not any("任务 7000 第 1 次尝试" in line for line in lines) or after_refusal != before:
            failures.append(f"副本锁的同任务一段：拒绝那一句应当写出记录里的任务号与尝试号（任务 7000 第 1 次尝试）、退 2、一个字不改，实际 {code} {lines}")
        if after_refusal != before:
            with open(table_path, "w") as handle:   # 照打了（弄坏开关 copy-owner-ignored）：表退回去，后面的格只判自己的
                handle.write(before)
        for name in (SERVICE_TASK_VARIABLE, SERVICE_ATTEMPT_VARIABLE):
            os.environ.pop(name, None)
        held, _ = held_with_record(f"other-machine-selftest 4242 selftest-token {SAME_TASK_SELFTEST_TASK} 2\n")
        if held:
            failures.append("副本锁的同任务一段：这一趟不经服务起（没有 TASK_SERVER_TASK、TASK_SERVER_ATTEMPT）时，别的机器上刚刷新过的记录照 300 秒判、应当拒绝，实际拿到了锁")
    finally:
        for name, value in saved.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value
        if os.path.exists(owner_path):
            os.unlink(owner_path)
    return failures


def landing_selftest(work):
    """换上与结局文件的自证（文件头「换上与被杀」「结局文件」）：每一格自己一个临时仓，互不连带 → 失败说明的清单。"""
    failures = []
    emit_path, parse_path = SAMPLE_FILES[0][0], SAMPLE_FILES[1][0]   # 补丁里的路径排序，换上时 emit.rs 在 parse.rs 前面

    def digest_of(data):
        return hashlib.sha256(data).hexdigest()

    # stagekill：暂存完 parse.rs 之后被杀，主工作区一个字不动、留下没有 journal.json 的暂存目录；下一次照常打上
    repo, directory = sample_repository(work, "stagekill")
    before = tree_bytes(repo)
    code = run_in_child("AFTER_EACH_STAGED", killed_after(1), lambda: apply(directory, repo, False, SAMPLE_GATE))
    untouched = tree_bytes(repo) == before
    leftover = os.path.isdir(pending_directory(repo)) and not os.path.exists(os.path.join(pending_directory(repo), JOURNAL_NAME))
    again_code, again_lines = apply(directory, repo, False, SAMPLE_GATE)
    if code != 137 or not untouched or not leftover or again_code != 0 or tree_bytes(repo) != sample_after(before) or os.path.exists(pending_directory(repo)):
        failures.append(f"暂存到一半被杀，主工作区应当一个字不动、下一次照常打上，实际子进程退 {code}、被杀时主工作区{'没动' if untouched else '动了'}、"
                        f"暂存目录{'留着没有 journal.json' if leftover else '不是预期的样子'}，再打退 {again_code} {again_lines}")
    # stagecorrupt：parse.rs 暂存出来之后被改坏，回读核应当拒绝、退 2、暂存目录清掉、主工作区一个字不动
    repo, directory = sample_repository(work, "stagecorrupt")
    before = tree_bytes(repo)

    def corrupt(index, staged):
        if index == 1:
            with open(staged, "ab") as handle:
                handle.write(b"// corrupted\n")
    globals()["AFTER_EACH_STAGED"] = corrupt
    try:
        code, lines = apply(directory, repo, False, SAMPLE_GATE)
    finally:
        globals()["AFTER_EACH_STAGED"] = None
    if code != 2 or tree_bytes(repo) != before or os.path.exists(pending_directory(repo)):
        failures.append(f"暂存出来的某一份与算好的 sha256 对不上时应当拒绝、退 2、主工作区一个字不动，实际退 {code} {lines}")
    # halfkill：换上 emit.rs 之后被杀；下一次打别的补丁应当拒、列出三类状态的文件与两条出路、主工作区不动
    repo, directory = sample_repository(work, "halfkill")
    other = os.path.join(work, "halfkill-other")
    os.makedirs(other)
    with open(os.path.join(other, "mutations-append.tsv"), "w", encoding="utf-8") as handle:
        handle.write("other\tf\to\tr\t-p demo --lib\tt\n")
    before = tree_bytes(repo)
    code = run_in_child("AFTER_EACH_REPLACE", killed_after(0), lambda: apply(directory, repo, False, SAMPLE_GATE))
    middle = tree_bytes(repo)
    changed = sorted(path for path in middle if middle[path] != before.get(path))
    other_code, other_lines = apply(other, repo, False, SAMPLE_GATE)
    text = "\n".join(other_lines)
    if code != 137 or changed != [emit_path] or not os.path.isfile(os.path.join(pending_directory(repo), JOURNAL_NAME)) or other_code != 2 or tree_bytes(repo) != middle \
            or f"{STATE_LANDED}：{emit_path}" not in text or f"{STATE_NOT_LANDED}：{parse_path}" not in text or f"{STATE_NOT_LANDED}：{MUTATION_TABLE}" not in text \
            or "--finish-pending" not in text or "--rollback-pending" not in text:
        failures.append(f"换上换到一半被杀，下一次起应当认出来、列出已换上与没换上的文件和两条出路、不在半截上再打别的补丁、一个字不改，"
                        f"实际子进程退 {code}、被杀时动了 {changed}，别的补丁退 {other_code} {other_lines}")
    # halfrollback：换上 emit.rs 之后被杀，--rollback-pending 照补丁前原样退回；结局文件写退回；之后同一个补丁照常打上
    repo, directory = sample_repository(work, "halfrollback")
    before = tree_bytes(repo)
    code = run_in_child("AFTER_EACH_REPLACE", killed_after(0), lambda: apply(directory, repo, False, SAMPLE_GATE))
    rollback_code, rollback_lines = rollback_pending(repo)
    restored = tree_bytes(repo) == before and not os.path.exists(pending_directory(repo))
    written = any(row[0] == "退回" for row in outcome_rows(os.path.join(directory, OUTCOME_NAME)))
    again_code, again_lines = apply(directory, repo, False, SAMPLE_GATE)
    if code != 137 or rollback_code != 0 or not restored or not written or again_code != 0 or tree_bytes(repo) != sample_after(before):
        failures.append(f"换到一半被杀之后 --rollback-pending 应当照改前原样退回、主工作区与打之前逐字节相同、之后同一个补丁照常打上，"
                        f"实际子进程退 {code}，退回退 {rollback_code} {rollback_lines}（逐字节相同 {restored}，结局文件有退回行 {written}），再打退 {again_code} {again_lines}")
    # halffinish：换上 emit.rs 之后被杀，--finish-pending 接着换完；结局文件写完成、--read-outcome 判打全了
    repo, directory = sample_repository(work, "halffinish")
    before = tree_bytes(repo)
    code = run_in_child("AFTER_EACH_REPLACE", killed_after(0), lambda: apply(directory, repo, False, SAMPLE_GATE))
    finish_code, finish_lines = finish_pending(repo, SAMPLE_GATE)
    verdict, verdict_lines = read_outcome(os.path.join(directory, OUTCOME_NAME))
    if code != 137 or finish_code != 0 or tree_bytes(repo) != sample_after(before) or os.path.exists(pending_directory(repo)) or verdict != 0:
        failures.append(f"换到一半被杀之后 --finish-pending 应当接着换完、主工作区与整份打完逐字节相同、结局文件写完成，"
                        f"实际子进程退 {code}，接着换退 {finish_code} {finish_lines}，--read-outcome 退 {verdict} {verdict_lines}")
    # readbackmismatch：全部换上之后 emit.rs 被别人改了，回读对不上：退 2、写异常、不写完成、暂存目录留着、下一次认成被别的改动动过
    repo, directory = sample_repository(work, "readbackmismatch")

    def tamper(root_path):
        with open(os.path.join(root_path, emit_path), "a", encoding="utf-8") as handle:
            handle.write("// edited by someone else\n")
    globals()["AFTER_ALL_REPLACED"] = tamper
    try:
        code, lines = apply(directory, repo, False, SAMPLE_GATE)
    finally:
        globals()["AFTER_ALL_REPLACED"] = None
    kinds = [row[0] for row in outcome_rows(os.path.join(directory, OUTCOME_NAME))]
    refusal = "\n".join(pending_refusal(repo))
    if code != 2 or "异常" not in kinds or "完成" in kinds or not os.path.isdir(pending_directory(repo)) or f"{STATE_TOUCHED}：{emit_path}" not in refusal:
        failures.append(f"换上之后回读与记下的改后 sha256 对不上时应当退 2、写异常、不写完成、留着暂存目录，实际退 {code} {lines}，结局文件的行种类 {kinds}")
    # outcomenormal：正常打一次，结局文件依次是开始、每个文件一行（补丁前与计划改后的 sha256）、回读、完成、后续；--read-outcome 判打全了
    repo, directory = sample_repository(work, "outcomenormal")
    before = tree_bytes(repo)
    code, lines = apply(directory, repo, False, SAMPLE_GATE)
    after = sample_after(before)
    rows = outcome_rows(os.path.join(directory, OUTCOME_NAME))
    shape = [row[0] for row in rows] == ["开始"] + ["文件"] * 3 + ["回读"] * 3 + ["完成", "后续"]
    listed = [row[1] for row in rows if row[0] == "文件"] == SAMPLE_ORDER
    before_ok = [row[3] for row in rows if row[0] == "文件"] == [digest_of(before[path]) for path in SAMPLE_ORDER]
    planned_ok = [row[4] for row in rows if row[0] == "文件"] == [digest_of(after[path]) for path in SAMPLE_ORDER]
    readback_ok = [row[2] for row in rows if row[0] == "回读"] == [digest_of(after[path]) for path in SAMPLE_ORDER]
    verdict, verdict_lines = read_outcome(os.path.join(directory, OUTCOME_NAME))
    if code != 0 or not (shape and listed and before_ok and planned_ok and readback_ok) or verdict != 0:
        failures.append(f"真打之后补丁目录里应当有结局文件：开始、每个文件的改前 sha256、回读、完成、后续依次写全，--read-outcome 应当退 0 判打全了，"
                        f"实际打退 {code} {lines}，行种类对 {shape}、文件对 {listed}、改前 sha256 对 {before_ok}、计划改后对 {planned_ok}、回读对 {readback_ok}，--read-outcome 退 {verdict} {verdict_lines}")
    # outcomekilled：换上 emit.rs 之后被杀，结局文件只有开始与文件行；--read-outcome 退 3，已换上 emit.rs、没换上其余
    repo, directory = sample_repository(work, "outcomekilled")
    code = run_in_child("AFTER_EACH_REPLACE", killed_after(0), lambda: apply(directory, repo, False, SAMPLE_GATE))
    kinds = [row[0] for row in outcome_rows(os.path.join(directory, OUTCOME_NAME))]
    verdict, verdict_lines = read_outcome(os.path.join(directory, OUTCOME_NAME))
    text = "\n".join(verdict_lines)
    if code != 137 or kinds != ["开始"] + ["文件"] * 3 or verdict != 3 or f"{STATE_LANDED}：{emit_path}" not in text \
            or f"{STATE_NOT_LANDED}：{parse_path}" not in text or f"{STATE_NOT_LANDED}：{MUTATION_TABLE}" not in text:
        failures.append(f"换到一半被杀的那一次结局文件只有开始与文件行、没有完成，--read-outcome 应当退 3、逐个文件列已换上与没换上，"
                        f"实际子进程退 {code}，行种类 {kinds}，--read-outcome 退 {verdict} {verdict_lines}")
    # outcomestale：打全之后 emit.rs 又被改了，--read-outcome 退 4 并点名它
    repo, directory = sample_repository(work, "outcomestale")
    code, lines = apply(directory, repo, False, SAMPLE_GATE)
    with open(os.path.join(repo, emit_path), "a", encoding="utf-8") as handle:
        handle.write("// later edit\n")
    verdict, verdict_lines = read_outcome(os.path.join(directory, OUTCOME_NAME))
    if code != 0 or verdict != 4 or not any(emit_path in line for line in verdict_lines):
        failures.append(f"打全之后文件又被改了，--read-outcome 应当退 4 并点名那一份，实际打退 {code}，--read-outcome 退 {verdict} {verdict_lines}")
    # gatehandoff：真打与换到一半被杀之后的 --finish-pending 都不在本进程里起 mutation-tables 那一格（交进来的命令一起就留下标记文件），
    # 打出要另交的整条命令（点名这个仓根）、结局文件的后续行写「没跑」与那一条命令、退 0
    handoff_failures = []
    for name in ("gatehandoff", "gatehandofffinish"):
        repo, directory = sample_repository(work, name)
        marker = os.path.join(work, f"{name}-gate-ran")
        gate = ["touch", marker]
        if name == "gatehandoff":
            code, lines = apply(directory, repo, False, gate)
        else:
            run_in_child("AFTER_EACH_REPLACE", killed_after(0), lambda: apply(directory, repo, False, gate))
            code, lines = finish_pending(repo, gate)
        command = mutation_tables_handoff(repo)
        follow = [row for row in outcome_rows(os.path.join(directory, OUTCOME_NAME)) if row[0] == "后续"]
        if code != 0 or os.path.exists(marker) or not any(line.strip() == command for line in lines) \
                or f"task-server/task submit --cwd {shlex.quote(repo)} -- bash .claude/gate.d/code-source-discipline.sh --check mutation-tables {shlex.quote(repo)}" not in command \
                or follow != [["后续", "门禁", "没跑", command]]:
            handoff_failures.append(f"{name}：退 {code}，那一格{'在本进程里起了' if os.path.exists(marker) else '没起'}，输出 {lines}，后续行 {follow}")
    if handoff_failures:
        failures.append(f"真打与 --finish-pending 都不应当在本进程里起 mutation-tables 那一格，应当打出要另交的整条命令、结局文件后续行写没跑，实际 {handoff_failures}")
    return failures


def selftest():
    work = tempfile.mkdtemp(prefix="apply-writer-patch-selftest-")
    try:
        root = os.path.join(work, "repo")
        os.makedirs(os.path.join(root, "crates", "demo", "src"))
        os.makedirs(os.path.join(root, "crates", "singlefs-checker", "src"))
        open(os.path.join(root, "crates", "demo", "src", "lib.rs"), "w").write("fn a() -> u32 { 1 }\n")
        open(os.path.join(root, "crates", "singlefs-checker", "src", "lib.rs"), "w").write("fn check() {}\n")
        open(os.path.join(root, "crates", "mutations.tsv"), "w").write("# 表头\nkeep\tf\to\tr\t-p demo --lib\tt\nold\tf\to\tr\t-p demo --lib\tt\ngone\tf\to\tr\t-p demo --lib\tt\n")
        for command in (["init", "-q"], ["add", "-A"], ["-c", "user.name=s", "-c", "user.email=s@s", "commit", "-qm", "base"]):
            git(root, *command, check=True)
        def patch_directory(name, source_change=None, append="", replace="", delete="", report="", repo=None):
            directory = os.path.join(work, name)
            os.makedirs(directory)
            repo = repo or root
            if source_change:
                relative, new_text = source_change
                path = os.path.join(repo, relative)
                original = open(path).read()
                open(path, "w").write(new_text)
                open(os.path.join(directory, "crates.patch"), "w").write(git(repo, "diff", "--", relative).stdout)
                open(path, "w").write(original)
            for file_name, text in (("mutations-append.tsv", append), ("mutations-replacements.tsv", replace), ("mutations-delete.txt", delete), ("result.md", report)):
                if text:
                    open(os.path.join(directory, file_name), "w").write(text)
            return directory
        gate33 = ["true"]
        failures = []
        good = patch_directory("good", ("crates/demo/src/lib.rs", "fn a() -> u32 { 2 }\n"), append="new\tf\to\tr\t-p demo --lib\tt\n",
                               replace="old\tf\to2\tr2\t-p demo --lib\tt\told\tf\to\tr\t-p demo --lib\tt\n", delete="gone\tf\to\tr\t-p demo --lib\tt\n")
        code, lines = apply(good, root, False, gate33)
        table = open(os.path.join(root, "crates", "mutations.tsv")).read()
        if code != 0 or "fn a() -> u32 { 2 }" not in open(os.path.join(root, "crates", "demo", "src", "lib.rs")).read() \
                or table != "# 表头\nkeep\tf\to\tr\t-p demo --lib\tt\nold\tf\to2\tr2\t-p demo --lib\tt\nnew\tf\to\tr\t-p demo --lib\tt\n":
            failures.append(f"好的补丁应当打上、按名字合表，实际 {code} {lines} {table!r}")
        # 前像比对（文件头「补丁目录里的文件」）：每格一个新的临时仓，变异表三行 swap、drop、fixed；fixed 的第三段此刻是 o-fixed（写补丁之后别的改动修好的），
        # 写补丁时的旧行还是 o。替换、删除各三格：前像对得上照换照删、有一行对不上整份拒、有一行没带前像整份拒（那一份里另有一行带了前像、对得上的，也不打）
        preimage_table = "# 表头\nswap\tf\to\tr\t-p demo --lib\tt\ndrop\tf\to\tr\t-p demo --lib\tt\nfixed\tf\to-fixed\tr\t-p demo --lib\tt\n"
        swap_old, swap_new, drop_row = "swap\tf\to\tr\t-p demo --lib\tt", "swap\tf\to2\tr2\t-p demo --lib\tt", "drop\tf\to\tr\t-p demo --lib\tt"
        fixed_now, fixed_stale, fixed_new = "fixed\tf\to-fixed\tr\t-p demo --lib\tt", "fixed\tf\to\tr\t-p demo --lib\tt", "fixed\tf\to\tr-moved\t-p demo --lib\tt"

        def preimage_case(name, replace="", delete=""):
            """→ (退出码, 输出的行, 打之前的树, 打之后的树, 打之后的变异表)：name 的临时仓里放 preimage_table、提交，照给的两份文件真打一次。"""
            repository = os.path.join(work, f"{name}-repo")
            os.makedirs(os.path.join(repository, "crates"))
            open(os.path.join(repository, "crates", "mutations.tsv"), "w").write(preimage_table)
            for command in (["init", "-q"], ["add", "-A"], ["-c", "user.name=s", "-c", "user.email=s@s", "commit", "-qm", "base"]):
                git(repository, *command, check=True)
            directory = patch_directory(name, replace=replace, delete=delete)
            before = tree_bytes(repository)
            code, lines = apply(directory, repository, False, gate33)
            return code, lines, before, tree_bytes(repository), open(os.path.join(repository, "crates", "mutations.tsv")).read()

        def names_both_sides(lines, name, written, now):
            """拒绝说明里有一行点名 name、写着两侧原文（制表符写成 ⇥）。"""
            return any(f"「{name}」前像" in line and tabs_shown(written) in line and tabs_shown(now) in line for line in lines)

        way_out = "以被打那棵树此刻的行为基线重出这几行"
        code, lines, before, after, table = preimage_case("preimage-replace-matched", replace=f"{swap_new}\t{swap_old}\n")
        if code != 0 or table != preimage_table.replace(swap_old, swap_new):
            failures.append(f"整行替换的前像与表里此刻那一行逐字相同时应当照换、退 0，实际退 {code} {lines} {table!r}")
        code, lines, before, after, table = preimage_case("preimage-replace-mismatched", replace=f"{swap_new}\t{swap_old}\n{fixed_new}\t{fixed_stale}\n")
        if code != 2 or after != before or not names_both_sides(lines, "fixed", fixed_stale, fixed_now) or any("「swap」前像" in line for line in lines) \
                or not any(way_out in line for line in lines):
            failures.append(f"整行替换有一行前像对不上表里此刻那一行时应当整份拒绝、退 2、逐行列出行名与两侧原文、出路写以此刻的行为基线重出、一个字不改，"
                            f"实际退 {code}、树{'没动' if after == before else '被改了'}：{lines}")
        code, lines, before, after, table = preimage_case("preimage-replace-without-preimage", replace=f"{swap_new}\n{fixed_new}\t{fixed_now}\n")
        if code != 2 or after != before or not any("mutations-replacements.tsv 有 1 行不是 12 段（swap（6 段））" in line and way_out in line for line in lines):
            failures.append(f"替换表里有一行只有六段（没带前像）时应当整份拒绝、退 2、一个字不改（同一份里带了前像、对得上的那一行也不打），"
                            f"实际退 {code}、树{'没动' if after == before else '被改了'}：{lines}")
        code, lines, before, after, table = preimage_case("preimage-delete-matched", delete=f"{drop_row}\n")
        if code != 0 or table != preimage_table.replace(drop_row + "\n", ""):
            failures.append(f"删除的前像与表里此刻那一行逐字相同时应当照删、退 0，实际退 {code} {lines} {table!r}")
        code, lines, before, after, table = preimage_case("preimage-delete-mismatched", delete=f"{drop_row}\n{fixed_stale}\n")
        if code != 2 or after != before or not names_both_sides(lines, "fixed", fixed_stale, fixed_now) or any("「drop」前像" in line for line in lines) \
                or not any(way_out in line for line in lines):
            failures.append(f"删除有一行前像对不上表里此刻那一行时应当整份拒绝、退 2、逐行列出行名与两侧原文、一个字不改，"
                            f"实际退 {code}、树{'没动' if after == before else '被改了'}：{lines}")
        code, lines, before, after, table = preimage_case("preimage-delete-names-only", delete=f"drop\n{fixed_now}\n")
        if code != 2 or after != before or not any("mutations-delete.txt 有 1 行不是 6 段（drop（1 段））" in line and way_out in line for line in lines):
            failures.append(f"删除表里有一行只写了名字（没带前像）时应当整份拒绝、退 2、一个字不改（同一份里带了前像、对得上的那一行也不删），"
                            f"实际退 {code}、树{'没动' if after == before else '被改了'}：{lines}")
        checker = patch_directory("checker", ("crates/singlefs-checker/src/lib.rs", "fn check() { assert!(true); }\n"), report="只改了判定\n")
        code, lines = apply(checker, root, False, gate33)
        if code != 2 or not any(CHECKER_SECTION in line for line in lines):
            failures.append(f"动 checker 而报告没那一节应当拒绝，实际 {code} {lines}")
        # 节名只认 CHECKER_SECTION：带代号的写法拒（样本拼着写，全仓按那个名字搜不到这里），去掉代号的写法放行（只核报告节名这一项，别的问题不管）
        checker_source = ("crates/singlefs-checker/src/lib.rs", "fn check() { let _ = 1; }\n")
        for name, section, refused_wanted in (
                ("checker-code-named-section", "## 受影响的" + "层 0 " + "流与崩溃枚举用例\n\n- 没有\n", True),
                ("checker-section", f"## {CHECKER_SECTION}\n\n- 没有\n", False)):
            found = plan(patch_directory(name, checker_source, report=section), root)[0]
            refused = any(f"「{CHECKER_SECTION}」一节" in problem for problem in found)
            if refused != refused_wanted:
                failures.append(f"动 checker 而报告写的是{'带代号的节名，应当拒绝' if refused_wanted else '「' + CHECKER_SECTION + '」，应当放行'}，实际 {found}")
        # 走不到的理由（文件头「走不到的理由」）：自己的临时仓，补丁用 plan() 核（不打）；角色表登了产品、验证、测试支撑、实验装置各一个包，另有一个包没登记，
        # 一个包（singlefs-new-apparatus）只在补丁里登角色；另一个临时仓里没有角色表
        unreachable_root = os.path.join(work, "unreachable-repo")
        roles_free_root = os.path.join(work, "unreachable-repo-without-roles")
        role_rows = (("demo", "产品"), ("singlefs-verification", "验证"), ("singlefs-unit-graph", "测试支撑"), ("singlefs-implementation-experiments", "实验装置"))
        for repository, packages, role_table in (
                (unreachable_root, [name for name, _role in role_rows] + ["singlefs-unregistered", "singlefs-new-apparatus"],
                 "# 样本里的角色表\n" + "".join(f"{name}\t{role}\tcrates/{name}\t样本里的{role}包\n" for name, role in role_rows)),
                (roles_free_root, ["singlefs-verification"], None)):
            for name in packages:
                os.makedirs(os.path.join(repository, "crates", name, "src"), exist_ok=True)
                open(os.path.join(repository, "crates", name, "src", "lib.rs"), "w").write("fn a() -> u32 { 1 }\n")
            open(os.path.join(repository, "crates", "mutations.tsv"), "w").write("# 表头\n")
            if role_table is not None:
                os.makedirs(os.path.join(repository, os.path.dirname(CRATE_ROLE_TABLE)), exist_ok=True)
                open(os.path.join(repository, CRATE_ROLE_TABLE), "w").write(role_table)
            for command in (["init", "-q"], ["add", "-A"], ["-c", "user.name=s", "-c", "user.email=s@s", "commit", "-qm", "base"]):
                git(repository, *command, check=True)

        def unreachable_check(name, new_text, report, relative="crates/demo/src/lib.rs", repository=unreachable_root):
            return plan(patch_directory(name, (relative, new_text), report=report, repo=repository), repository)[0]

        macro_source = "fn a(n: u32) -> u32 {\n    if n > 100 {\n        unreachable!()\n    }\n    n\n}\n"
        macro_section = "## 走不到的理由\n\n- crates/demo/src/lib.rs:3 唯一的调用点先判了 n 不大于 100，这一支进不来\n"
        found = unreachable_check("unreachable-unlisted", macro_source, "只改了判定\n")
        if not (len(found) == 1 and "crates/demo/src/lib.rs:3 unreachable!" in found[0] and "走不到的理由" in found[0]):
            failures.append(f"新加 unreachable! 而 result.md 没有「走不到的理由」一节应当拒绝并点名 文件:行，实际 {found}")
        found = unreachable_check("unreachable-listed", macro_source, macro_section)
        if found:
            failures.append(f"新加 unreachable! 而一节里逐处列了就该放行，实际 {found}")
        found = unreachable_check("unreachable-short-reason", macro_source, "## 走不到的理由\n\n- crates/demo/src/lib.rs:3 不会到\n")
        if not any("不足 8 个字" in problem and "crates/demo/src/lib.rs:3" in problem for problem in found):
            failures.append(f"走不到的理由写得不足 8 个字应当拒绝，实际 {found}")
        found = unreachable_check("unreachable-debug-assert", "fn a(n: u32) -> u32 {\n    debug_assert!(n < 100);\n    n\n}\n", "")
        if found:
            failures.append(f"debug_assert! 不算要列的断言，应当放行，实际 {found}")
        found = unreachable_check("unreachable-test-module", "fn a() -> u32 { 1 }\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn a_is_one() {\n        assert!(super::a() == 1);\n    }\n}\n", "")
        if found:
            failures.append(f"测试模块里的 assert! 不算要列的断言，应当放行，实际 {found}")
        # 整份测试文件（文件头「走不到的理由」）：补丁在 demo 包里改父模块文件、新增被它声明的文件，文件里的 assert! 在函数外的普通 fn 里（不靠 #[test]）
        helper_source = "fn helper(n: u32) {\n    assert!(n < 100);\n}\n"

        def module_files_check(name, changes):
            edited = os.path.join(work, f"{name}-edited")
            shutil.copytree(unreachable_root, edited)
            for relative, new_text in changes.items():
                os.makedirs(os.path.dirname(os.path.join(edited, relative)), exist_ok=True)
                open(os.path.join(edited, relative), "w").write(new_text)
            git(edited, "add", "-A", check=True)
            directory = os.path.join(work, name)
            os.makedirs(directory)
            open(os.path.join(directory, "crates.patch"), "w").write(git(edited, "diff", "--cached").stdout)
            return plan(directory, unreachable_root)[0]

        base_source = "fn a() -> u32 { 1 }\n"
        found = module_files_check("module-file-declared-cfg-test", {"crates/demo/src/lib.rs": base_source + "\n#[cfg(test)]\nmod tests;\n", "crates/demo/src/tests.rs": helper_source})
        if found:
            failures.append(f"父模块文件里 #[cfg(test)] mod tests; 引进来的 tests.rs 整份算测试代码，里面的 assert! 应当放行，实际 {found}")
        found = module_files_check("module-file-declared-without-cfg-test", {"crates/demo/src/lib.rs": base_source + "\nmod helpers;\n", "crates/demo/src/helpers.rs": helper_source})
        if not (len(found) == 1 and "crates/demo/src/helpers.rs:2 assert!" in found[0]):
            failures.append(f"父模块文件里不带 #[cfg(test)] 的 mod helpers; 引进来的文件照产品代码判，里面的 assert! 应当拒绝并点名 文件:行，实际 {found}")
        found = module_files_check("module-file-undeclared", {"crates/demo/src/orphan.rs": helper_source})
        if not (len(found) == 1 and "crates/demo/src/orphan.rs:2 assert!" in found[0]):
            failures.append(f"没有任何父模块文件声明它的文件照产品代码判，里面的 assert! 应当拒绝并点名 文件:行，实际 {found}")
        found = module_files_check("module-file-ancestor-declared-cfg-test", {"crates/demo/src/lib.rs": base_source + "\n#[cfg(test)]\nmod a;\n",
                                                                              "crates/demo/src/a.rs": "mod b;\n", "crates/demo/src/a/b.rs": helper_source})
        if found:
            failures.append(f"祖先一层 #[cfg(test)] mod a; 引进来的 a/b.rs 整份算测试代码，里面的 assert! 应当放行，实际 {found}")
        found = module_files_check("module-file-directory-style", {"crates/demo/src/lib.rs": base_source + "\n#[cfg(test)]\n#[allow(dead_code)]\nmod m;\n",
                                                                   "crates/demo/src/m/mod.rs": "mod n;\n", "crates/demo/src/m/n.rs": helper_source})
        if found:
            failures.append(f"#[cfg(test)] mod m; 引进来的 m/mod.rs 与它声明的 m/n.rs 整份算测试代码，里面的 assert! 应当放行，实际 {found}")
        found = module_files_check("module-file-path-attribute", {"crates/demo/src/lib.rs": base_source + "\n#[cfg(test)]\n#[path = \"support/checks.rs\"]\nmod checks;\n",
                                                                  "crates/demo/src/support/checks.rs": helper_source})
        if found:
            failures.append(f"#[cfg(test)] 与 #[path = …] 叠在同一条 mod 声明上，path 落的那份文件整份算测试代码，里面的 assert! 应当放行，实际 {found}")
        found = unreachable_check("unreachable-in-comments", "fn a() -> &'static str {\n    // unreachable!() 只在注释里\n    \"todo!\"\n}\n", "")
        if found:
            failures.append(f"注释与字符串里的 unreachable!、todo! 不算，应当放行，实际 {found}")
        asserting_source = "fn a() -> u32 {\n    assert!(true);\n    1\n}\n"
        for name, package, role in (("unreachable-non-product", "singlefs-verification", "验证"), ("unreachable-test-support", "singlefs-unit-graph", "测试支撑"),
                                    ("unreachable-experiment-apparatus", "singlefs-implementation-experiments", "实验装置")):
            found = unreachable_check(name, asserting_source, "", relative=f"crates/{package}/src/lib.rs")
            if found:
                failures.append(f"角色是{role}的包（{package}）里的 assert! 不算产品代码，应当放行，实际 {found}")
        found = unreachable_check("unreachable-unregistered-crate", asserting_source, "只改了判定\n", relative="crates/singlefs-unregistered/src/lib.rs")
        if not (len(found) == 1 and "crates/singlefs-unregistered/src/lib.rs:2 assert!" in found[0] and "没登记角色" in found[0]
                and "crates/singlefs-unregistered；" in found[0]):
            failures.append(f"角色表里没有一行的包（singlefs-unregistered）照产品判，新加的 assert! 应当拒绝并点名没登记角色，实际 {found}")
        found = unreachable_check("unreachable-role-table-unreadable", asserting_source, "只改了判定\n", relative="crates/singlefs-verification/src/lib.rs",
                                  repository=roles_free_root)
        if not (len(found) == 1 and "crates/singlefs-verification/src/lib.rs:2 assert!" in found[0] and "没登记角色" in found[0] and "读不到" in found[0]):
            failures.append(f"被判那棵树里读不到角色表时每个包照产品判，新加的 assert! 应当拒绝并点名没登记角色，实际 {found}")
        # 补丁自己在角色表里登一行实验装置（singlefs-new-apparatus），同一份补丁在它的 src 里新加 assert!：角色照补丁打上之后的表判
        registering = os.path.join(work, "unreachable-role-from-patched-table")
        os.makedirs(registering)
        apparatus_source = os.path.join(unreachable_root, "crates", "singlefs-new-apparatus", "src", "lib.rs")
        table_path = os.path.join(unreachable_root, CRATE_ROLE_TABLE)
        originals = {path: open(path).read() for path in (apparatus_source, table_path)}
        open(apparatus_source, "w").write(asserting_source)
        open(table_path, "a").write("singlefs-new-apparatus\t实验装置\tcrates/singlefs-new-apparatus\t补丁里新登的实验装置包\n")
        open(os.path.join(registering, "crates.patch"), "w").write(
            git(unreachable_root, "diff", "--", "crates/singlefs-new-apparatus/src/lib.rs", CRATE_ROLE_TABLE).stdout)
        for path, original in originals.items():
            open(path, "w").write(original)
        found = plan(registering, unreachable_root)[0]
        if found:
            failures.append(f"补丁把新包登成实验装置、同一份补丁在它里面新加 assert!，角色照补丁打上之后的角色表判，应当放行，实际 {found}")
        member_source = "pub enum Refusal {\n    RenameAcrossPoolsUndecided,\n}\n"
        found = unreachable_check("unreachable-member-unlisted", member_source, "只改了判定\n")
        if not (len(found) == 1 and "crates/demo/src/lib.rs:2 错误成员 RenameAcrossPoolsUndecided" in found[0]):
            failures.append(f"新加条款没定的错误成员（名字以 Undecided 收尾）而报告没列应当拒绝并点名成员，实际 {found}")
        found = unreachable_check("unreachable-member-listed", member_source,
                                  "## 走不到的理由\n\n- crates/demo/src/lib.rs:2 重命名跨池的条款没定，第一版不支持，在任何落盘动作之前返回这个成员\n")
        if found:
            failures.append(f"错误成员逐处列了就该放行，实际 {found}")
        duplicate = patch_directory("duplicate", append="keep\tf\to\tr\t-p demo --lib\tt\n")
        code, lines = apply(duplicate, root, False, gate33)
        if code != 2:
            failures.append(f"追加已有的名字应当拒绝，实际 {code} {lines}")
        selfduplicate = patch_directory("selfduplicate", append="twin\tf\to\tr\t-p demo --lib\tt\ntwin\tf\to2\tr2\t-p demo --lib\tt\n")
        code, lines = apply(selfduplicate, root, False, gate33)
        if code != 2 or not any("twin" in line and "mutations-append.tsv" in line for line in lines):
            failures.append(f"追加表里两行同名（都不在表里）应当拒绝并点名重名的变异名，实际 {code} {lines}")
        stale = patch_directory("stale", ("crates/demo/src/lib.rs", "fn a() -> u32 { 9 }\n"))
        open(os.path.join(root, "crates", "demo", "src", "lib.rs"), "w").write("fn a() -> u32 { 3 }\n")
        code, lines = apply(stale, root, False, gate33)
        if code != 2 or not any("apply --check" in line for line in lines):
            failures.append(f"打不上的补丁应当在核的时候拒绝，实际 {code} {lines}")
        os.environ["CRASH_VERIFY_HINT_PATCHES"] = "1"
        global CRASH_VERIFY_HINT_PATCHES
        CRASH_VERIFY_HINT_PATCHES = 1
        only_table = patch_directory("onlytable", append="another\tf\to\tr\t-p demo --lib\tt\n")
        code, lines = apply(only_table, root, False, gate33)
        if code != 0 or not any("崩溃验证全绿以来已打" in line for line in lines):
            failures.append(f"打过的补丁数到提示线应当打提示，实际 {code} {lines}")
        # 不指定范围判绿那一趟在 git common-dir 里留下 all-green（文件头）：留下之后的补丁数从它之后重数；提示线抬到 2，这一份是留下之后的第一份，不到线、不打提示。
        # 自己的临时仓、自己的日志：日志里先写三行早就打过的，标记的修改时间写在它们之后，两头都用写死的过去时刻，
        # 不读别的格往日志里记的行，也不靠文件系统的时间精度分先后（时间精度粗的文件系统上，同一秒里连着打的几份会和标记分不出先后）
        green_root = os.path.join(work, "all-green-repo")
        os.makedirs(os.path.join(green_root, "crates"))
        open(os.path.join(green_root, "crates", "mutations.tsv"), "w").write("# 表头\n")
        for command in (["init", "-q"], ["add", "-A"], ["-c", "user.name=s", "-c", "user.email=s@s", "commit", "-qm", "base"]):
            git(green_root, *command, check=True)
        green_common = git(green_root, "rev-parse", "--git-common-dir").stdout.strip()
        green_common = green_common if os.path.isabs(green_common) else os.path.join(green_root, green_common)
        # 标记所在的子目录用目录名常量拼，不拿带通配符的 ALL_GREEN_RUN_GLOB 去 dirname：那样建出来的是一层名叫 * 的目录，
        # 标记落在 */whole-run/all-green，比 glob 的 */all-green 深一层，apply 里怎么找都找不到（两台都红，不挑机器）
        green_directory = os.path.join(green_common, CRASH_REPLAY_LOGS_NAME, "whole-run")
        os.makedirs(green_directory)
        green_marker = os.path.join(green_directory, "all-green")
        open(green_marker, "w").write("自证\n")
        if glob.glob(os.path.join(green_common, ALL_GREEN_RUN_GLOB)) != [green_marker]:
            failures.append(f"自证自己造的 all-green 标记，数补丁用的 glob 找不到（造的 {green_marker}，找到 {glob.glob(os.path.join(green_common, ALL_GREEN_RUN_GLOB))}）")
        now = time.time()
        with open(os.path.join(green_common, LOG_NAME), "w", encoding="utf-8") as handle:
            for seconds_ago in (1000, 900, 800):
                handle.write(f"{now - seconds_ago}\t/applied-before-all-green\t-\n")
        marked_at = now - 500
        os.utime(green_marker, (marked_at, marked_at))
        CRASH_VERIFY_HINT_PATCHES = 2
        after_green = patch_directory("allgreen", append="after-green\tf\to\tr\t-p demo --lib\tt\n", repo=green_root)
        code, lines = apply(after_green, green_root, False, gate33)
        CRASH_VERIFY_HINT_PATCHES = 1
        if code != 0 or any("崩溃验证全绿以来已打" in line for line in lines):
            found = glob.glob(os.path.join(green_common, ALL_GREEN_RUN_GLOB))
            read_back = [round(os.path.getmtime(path) - marked_at, 3) for path in found]
            failures.append(f"不指定范围判绿那一趟留下 all-green 之后，打过的补丁数应当从它之后重数、没到提示线不打提示，实际 {code} {lines}"
                            f"（glob 找到的标记 {found}，修改时间读回来比写进去的多 {read_back} 秒，日志 {os.path.join(green_common, LOG_NAME)}，"
                            f"日志目录下现有 {sorted(os.path.join(directory, name) for directory, _, names in os.walk(os.path.join(green_common, CRASH_REPLAY_LOGS_NAME)) for name in names)}）")
        # 副本锁（文件头「副本锁」）：这一份当成证红在跑、拿着锁，apply 要拒绝、一个字不改；放了锁再打就照打，打完锁文件不留
        locked = patch_directory("locked", append="locked-row\tf\to\tr\t-p demo --lib\tt\n")
        table_path = os.path.join(root, "crates", "mutations.tsv")
        before = open(table_path).read()
        held = hold_copy_lock(root)
        try:
            code, lines = apply(locked, root, False, gate33)
            after_refusal = open(table_path).read()
        finally:
            if held is not None:
                release_copy_lock(root, held)
        if held is None or code != 2 or not any("副本上有证红在跑" in line for line in lines) or after_refusal != before:
            failures.append(f"副本锁被占着时应当拒绝、退 2、一个字不改，实际 {code} {lines}（锁{'拿到了' if held is not None else '没拿到'}）")
        if after_refusal != before:
            # 上一格在弄坏开关（copy-lock-ignored）下照打了：表退回去，这一格只判自己的，不因为「locked-row 表里已经有了」连带红
            with open(table_path, "w") as handle:
                handle.write(before)
        code, lines = apply(locked, root, False, gate33)
        if code != 0 or "locked-row\t" not in open(table_path).read() or os.path.exists(os.path.join(root, COPY_LOCK_NAME)):
            failures.append(f"副本锁没被占着时应当照打、打完不留锁文件，实际 {code} {lines}")
        # 副本锁的跨机一段（文件头「副本锁」）：持有者记录写着别的机器的标识。刚刷新过的（这里自己不拿 flock，演 flock 跨机不互斥）要拒绝、退 2、一个字不改；
        #   修改时间早过过期线的当它被杀了，接手、放锁后记录不留；写着这一台标识的旧记录，flock 拿得到就说明原持有者没了，接手
        owner_path = os.path.join(root, COPY_OWNER_NAME)
        owner_wrong = []
        owned = patch_directory("owned", append="owned-row\tf\to\tr\t-p demo --lib\tt\n")
        before = open(table_path).read()
        with open(owner_path, "w") as handle:
            handle.write(f"other-machine-selftest 4242 selftest-token {NOT_FROM_SERVICE} {NOT_FROM_SERVICE}\n")
        code, lines = apply(owned, root, False, gate33)
        after_refusal = open(table_path).read()
        if code != 2 or not any("副本上有证红在跑" in line for line in lines) or after_refusal != before:
            owner_wrong.append(f"别的机器上的证红占着副本（持有者记录刚刷新过）时应当拒绝、退 2、一个字不改，实际 {code} {lines}")
        if after_refusal != before:
            with open(table_path, "w") as handle:   # 弄坏开关 copy-owner-ignored 下照打了：表退回去，后面的格只判自己的
                handle.write(before)
        stale_at = time.time() - 3 * COPY_OWNER_STALE_SECONDS
        os.utime(owner_path, (stale_at, stale_at))
        held = hold_copy_lock(root)
        took_over = held is not None and open(owner_path).read().split()[:1] == [machine_identity()]
        if held is not None:
            release_copy_lock(root, held)
        if not took_over or os.path.exists(owner_path):
            owner_wrong.append(f"别的机器上的持有者记录早过过期线（{3 * COPY_OWNER_STALE_SECONDS} 秒没刷新）时应当接手、记录里换成这一台的标识、放锁后不留记录，实际{'拿到了' if held is not None else '没拿到'}锁、接手{'了' if took_over else '没有'}、记录{'还在' if os.path.exists(owner_path) else '已删'}")
        with open(owner_path, "w") as handle:
            handle.write(f"{machine_identity()} 1 old-token {NOT_FROM_SERVICE} {NOT_FROM_SERVICE}\n")
        held = hold_copy_lock(root)
        if held is not None:
            release_copy_lock(root, held)
        if held is None or os.path.exists(owner_path):
            owner_wrong.append(f"写着这一台标识的旧记录（原持有者没了、flock 拿得到）应当接手、放锁后不留记录，实际{'拿到了' if held is not None else '没拿到'}锁、记录{'还在' if os.path.exists(owner_path) else '已删'}")
        if os.path.exists(owner_path):
            os.unlink(owner_path)
        if owner_wrong:
            failures.append("副本锁的跨机一段：" + "；".join(owner_wrong))
        failures += same_task_owner_failures(root, owner_path, patch_directory("owner-same-task", append="owner-same-task-row\tf\to\tr\t-p demo --lib\tt\n"),
                                             table_path, gate33)
        # 开工快照（文件头「开工快照」）：临时仓里开一轮 r-open（快照目录、正文在，判决不在），清单列着 crates/demo/src/lib.rs 与 crates/mutations.tsv；
        # 当主工作区打（main_workspace=True）。snapshot：改 lib.rs 的补丁与整行换掉变异行的补丁都拒、退 2、一个字不改，出路不提单次放行；
        # snapshot-append-only：只追加变异行的补丁不算撞（与派发闸 ⑤ 同义），照打；snapshot-ruled-refused：命令行给 --snapshot-ruled 照用法错退 2、一个字不改
        prompts = os.path.join(root, "research", "prompts")
        os.makedirs(os.path.join(prompts, "r-open-snapshot"))
        open(os.path.join(prompts, "_r-open-body.md"), "w").write("# 样本三方轮的正文\n")
        open(os.path.join(prompts, "r-open-snapshot", "crates-sha256.txt"), "w").write(
            f"{'a' * 64}  ./crates/demo/src/lib.rs\n{'b' * 64}  tree/crates/mutations.tsv\n")
        for command in (["add", "-A"], ["-c", "user.name=s", "-c", "user.email=s@s", "commit", "-qm", "上面几格打过的样子"]):
            git(root, *command, check=True)   # 补丁照此刻的工作区造（stale 那一格把工作区改得与索引不同）
        source_before, table_before = open(os.path.join(root, "crates", "demo", "src", "lib.rs")).read(), open(table_path).read()
        snapshot_code = patch_directory("snapshot", ("crates/demo/src/lib.rs", "fn a() -> u32 { 7 }\n"))
        snapshot_replace = patch_directory("snapshot-replace", replace="keep\tf\to3\tr3\t-p demo --lib\tt\tkeep\tf\to\tr\t-p demo --lib\tt\n")
        code, lines = apply(snapshot_code, root, False, gate33, main_workspace=True)
        table_code, table_lines = apply(snapshot_replace, root, False, gate33, main_workspace=True)
        source_after, table_after = open(os.path.join(root, "crates", "demo", "src", "lib.rs")).read(), open(table_path).read()
        if code != 2 or table_code != 2 or source_after != source_before or table_after != table_before \
                or not any("r-open 的开工快照里" in line for line in lines) or not any("crates/demo/src/lib.rs（出现在补丁" in line for line in lines) \
                or not any("crates/mutations.tsv（出现在补丁" in line for line in table_lines) or any("--snapshot-ruled" in line or "快照冲突已判" in line for line in lines) \
                or not any(line.startswith("→ 怎么办：") and "abandoned-rounds.tsv" in line for line in lines):
            failures.append(f"补丁改到开着的三方轮开工快照里的文件应当拒绝、退 2、一个字不改、出路只写等判决与登记叫停表，"
                            f"实际改代码那一份退 {code} {lines}，整行换掉变异行那一份退 {table_code} {table_lines}")
        if source_after != source_before or table_after != table_before:
            # 上一格在弄坏开关（snapshot-ignored）下照打了：两份退回去，下一格只判自己的
            open(os.path.join(root, "crates", "demo", "src", "lib.rs"), "w").write(source_before)
            open(table_path, "w").write(table_before)
        # snapshot-append-only：只追加变异行（crates/mutations.tsv 在 r-open 的清单里）不算撞快照，照打、退 0（弄坏开关 append-only-mutations-conflict 下被拒）
        snapshot_append = patch_directory("snapshot-append-only", append="snapshot-row\tf\to\tr\t-p demo --lib\tt\n")
        code, lines = apply(snapshot_append, root, False, gate33, main_workspace=True)
        if code != 0 or "snapshot-row\t" not in open(table_path).read():
            failures.append(f"只追加变异行的补丁不算撞快照（crates/mutations.tsv 在开着的那一轮的清单里），应当照打、退 0，实际退 {code} {lines}")
        # snapshot-ruled-refused：命令行给 --snapshot-ruled（删了的单次放行口子）照用法错退 2、一个字不改
        ruled_run = subprocess.run([sys.executable, os.path.realpath(__file__), snapshot_code, "--root", root,
                                    "--snapshot-ruled", "快照冲突已判：r-open 只改样本函数的返回值"], capture_output=True, text=True, timeout=120)
        if ruled_run.returncode != 2 or "--snapshot-ruled" not in ruled_run.stdout + ruled_run.stderr \
                or open(os.path.join(root, "crates", "demo", "src", "lib.rs")).read() != source_before:
            failures.append(f"给 --snapshot-ruled 应当照用法错退 2、一个字不改，实际退 {ruled_run.returncode}，输出「{(ruled_run.stdout + ruled_run.stderr).strip()[-400:]}」")
        # snapshot-unreadable：再开两轮，r-folder 的清单是目录、r-locked 的清单没有读权限（列的是补丁不碰的文件）；补丁只追加变异行（crates/mutations.tsv 在 r-open
        # 的清单里，只追加不算撞）。照打、退 0，输出逐份点名那一轮与那份清单、写明放行但必记检出，检出记录多一行（结局只记）
        import json
        for round_name in ("r-folder", "r-locked"):
            open(os.path.join(prompts, f"_{round_name}-body.md"), "w").write(f"# {round_name} 正文\n")
        os.makedirs(os.path.join(prompts, "r-folder-snapshot", "crates-sha256.txt"))
        os.makedirs(os.path.join(prompts, "r-locked-snapshot"))
        locked_list = os.path.join(prompts, "r-locked-snapshot", "crates-sha256.txt")
        open(locked_list, "w").write(f"{'c' * 64}  crates/demo/src/untouched.rs\n")
        os.chmod(locked_list, 0)
        unreadable_rounds = ("r-folder",) if os.access(locked_list, os.R_OK) else ("r-folder", "r-locked")   # 以 root 跑时去掉读权限也读得了
        detections = os.path.join(work, "detections.jsonl")
        saved_detections = os.environ.get("AGENT_HOOK_DETECTIONS")
        os.environ["AGENT_HOOK_DETECTIONS"] = detections
        try:
            unreadable_patch = patch_directory("snapshot-unreadable", append="unreadable-row\tf\to\tr\t-p demo --lib\tt\n")
            code, lines = apply(unreadable_patch, root, False, gate33, main_workspace=True)
        finally:
            if saved_detections is None:
                os.environ.pop("AGENT_HOOK_DETECTIONS", None)
            else:
                os.environ["AGENT_HOOK_DETECTIONS"] = saved_detections
        recorded = []
        if os.path.exists(detections):
            for line in open(detections, encoding="utf-8"):
                try:
                    recorded.append(json.loads(line))
                except ValueError:
                    recorded.append({"坏行": line})
        named = all(any(round_name in line and "放行但必记检出" in line for line in lines) for round_name in unreadable_rounds)
        if code != 0 or "unreadable-row\t" not in open(table_path).read() or not named or len(recorded) != 1 or recorded[0].get("outcome") != "只记" \
                or len(recorded[0].get("findings") or []) != len(unreadable_rounds):
            failures.append(f"快照清单读不了时应当照打、点名那一轮与那几份清单、写明放行但必记检出、检出记录多一行，实际退 {code} {lines}，检出 {recorded}")
        # 变异锚点静态核（文件头「变异锚点静态核」）：另造一个临时仓，变异表里 keep-anchor 一行的原文在 lib.rs 里恰好一处；--dry-run 走副本那一支，真跑 crates-mutation-rows.py static-check。
        # anchorbreak：补丁把原文改没了，拒、退 2、点名变异名与 ANCHOR、仓里一个字不改；anchorkept：补丁只改别的函数，放行、报 CHECKED；
        # anchorpre：表里另有一行 old-broken 在补丁之前锚点就断了，不相干的补丁照放行、提醒一行；anchorpre-new：同一张表上弄断 keep-anchor 的补丁，只拒带来的那一处、不连带点名 old-broken
        anchor_root = os.path.join(work, "anchor-repo")
        os.makedirs(os.path.join(anchor_root, "crates", "demo", "src"))
        anchor_source = os.path.join(anchor_root, "crates", "demo", "src", "lib.rs")
        anchor_table = os.path.join(anchor_root, "crates", "mutations.tsv")
        anchor_original = "fn a() -> u32 { 1 }\nfn b() -> u32 { 5 }\n"
        open(anchor_source, "w").write(anchor_original)
        open(anchor_table, "w").write("# 表头\nkeep-anchor\tcrates/demo/src/lib.rs\tfn a() -> u32 { 1 }\tfn a() -> u32 { 2 }\t-p demo --lib\tkept_test\n")
        write_sample_registration_table(anchor_root)
        outside_relative = ".claude/gate.d/refused-tests.tsv"   # 拷贝范围之外的一份登记表（拒算表）：outsidecopy 那一格的补丁改它
        with open(os.path.join(anchor_root, *outside_relative.split("/")), "w", encoding="utf-8") as handle:
            handle.write("refused-tests-original\n")
        for command in (["init", "-q"], ["add", "-A"], ["-c", "user.name=s", "-c", "user.email=s@s", "commit", "-qm", "base"]):
            git(anchor_root, *command, check=True)
        anchor_break = patch_directory("anchorbreak", ("crates/demo/src/lib.rs", "fn a() -> u32 { 9 }\nfn b() -> u32 { 5 }\n"), repo=anchor_root)
        code, lines = apply(anchor_break, anchor_root, True, gate33)
        if code != 2 or not any("keep-anchor" in line and "ANCHOR" in line for line in lines) or open(anchor_source).read() != anchor_original:
            failures.append(f"弄断变异锚点的补丁 --dry-run 应当拒绝、退 2、点名变异名与 ANCHOR、仓里一个字不改，实际退 {code} {lines}")
        anchor_kept = patch_directory("anchorkept", ("crates/demo/src/lib.rs", "fn a() -> u32 { 1 }\nfn b() -> u32 { 6 }\n"), repo=anchor_root)
        code, lines = apply(anchor_kept, anchor_root, True, gate33)
        if code != 0 or not any("核过了（--dry-run" in line for line in lines) or not any("CHECKED" in line for line in lines):
            failures.append(f"不弄断锚点的补丁 --dry-run 应当放行并报 CHECKED，实际退 {code} {lines}")
        # outsidecopy：补丁改的是拷贝范围之外的文件（拒算表，拷贝范围里没有它）：补丁在临时树上要套得上、判不判只看 static-check 的输出，放行并报 CHECKED、不退「没做成」
        outside_patch = patch_directory("outsidecopy", (outside_relative, "refused-tests-original\nadded-line\n"), repo=anchor_root)
        code, lines = apply(outside_patch, anchor_root, True, gate33)
        if code != 0 or not any("核过了（--dry-run" in line for line in lines) or not any("CHECKED" in line for line in lines) or any("没做成" in line for line in lines):
            failures.append(f"补丁改的是拷贝范围之外的文件（拒算表）时 --dry-run 应当照常判：放行并报 CHECKED、不退没做成（补丁改到的文件也拷进两棵临时树），实际退 {code} {lines}")
        # livebaseline：假的 static-check 判补丁之后那一棵时，把主工作区里的 lib.rs 临时改了（a 的原文 1 改成 5，模拟别的会话在两趟 static-check 的几百秒之间改），判完没打补丁那一棵之后改回；
        # 补丁把 a 的原文改成 9（keep-anchor 在补丁之后那一棵里命中 0 次）：没打补丁那一棵是同一刻拷出的树、不是此刻的活树，补丁之前 keep-anchor 命中 1 次，
        # 应当算补丁新弄断的、退 2、点名 keep-anchor；拿活的主工作区当基线时它也命中 0 次、被当成补丁之前就红的放行
        live_state = os.path.join(work, "live-baseline-state.txt")
        fake_live_check = os.path.join(work, "fake-live-baseline-static-check.py")
        with open(fake_live_check, "w", encoding="utf-8") as handle:
            handle.write("import os, sys\n"
                         "tree = sys.argv[2]\n"
                         f"live = {anchor_source!r}\n"
                         f"state = {live_state!r}\n"
                         "def read(path):\n"
                         "    with open(path, encoding='utf-8') as source:\n"
                         "        return source.read()\n"
                         "def write(path, text):\n"
                         "    with open(path, 'w', encoding='utf-8') as target:\n"
                         "        target.write(text)\n"
                         "first = not os.path.exists(state)\n"
                         "if first:\n"
                         "    write(state, read(live))\n"
                         "    write(live, read(live).replace('fn a() -> u32 { 1 }', 'fn a() -> u32 { 5 }'))\n"
                         "rows = [line.split('\\t') for line in read(os.path.join(tree, 'crates', 'mutations.tsv')).split('\\n') if line.strip() and not line.startswith('#')]\n"
                         "print('COPIED\\tcrates')\n"
                         "red = 0\n"
                         "for number, columns in enumerate(rows, 1):\n"
                         "    count = read(os.path.join(tree, columns[1])).count(columns[2])\n"
                         "    if count != 1:\n"
                         "        red += 1\n"
                         "        print(f'ANCHOR\\t{number}\\t{columns[0]}\\t原文在 {columns[1]} 里命中 {count} 次，要恰好 1 次')\n"
                         "print(f'CHECKED\\t{len(rows)}\\t1\\t0')\n"
                         "if not first:\n"
                         "    write(live, read(state))\n"
                         "    os.remove(state)\n"
                         "sys.exit(1 if red else 0)\n")
        live_patch = patch_directory("livebaseline", ("crates/demo/src/lib.rs", "fn a() -> u32 { 9 }\nfn b() -> u32 { 5 }\n"), repo=anchor_root)
        code, lines = apply(live_patch, anchor_root, True, gate33, static_check=lambda tree: [sys.executable or "python3", fake_live_check, "static-check", tree])
        restored = open(anchor_source).read() == anchor_original and not os.path.exists(live_state)
        if code != 2 or not any("keep-anchor" in line and "ANCHOR" in line for line in lines) or not restored:
            failures.append(f"没打补丁那一棵应当是同一刻从主工作区拷出的树、不是此刻的活树：两趟 static-check 之间主工作区里锚着的文件被临时改了又改回，补丁弄断的 keep-anchor 仍应判新红"
                            f"（退 2、点名 keep-anchor），实际退 {code} {lines}，主工作区的文件改回了没有：{restored}")
        if open(anchor_source).read() != anchor_original:   # 假 static-check 万一没改回（开关打开、判到一半抛错），后面的格读的是原样
            open(anchor_source, "w").write(anchor_original)
            if os.path.exists(live_state):
                os.remove(live_state)
        with open(anchor_table, "a") as handle:
            handle.write("old-broken\tcrates/demo/src/lib.rs\tfn missing() {}\tfn missing() { }\t-p demo --lib\tbroken_test\n")
        for command in (["add", "-A"], ["-c", "user.name=s", "-c", "user.email=s@s", "commit", "-qm", "补丁之前锚点就断了一行"]):
            git(anchor_root, *command, check=True)
        anchor_pre = patch_directory("anchorpre", ("crates/demo/src/lib.rs", "fn a() -> u32 { 1 }\nfn b() -> u32 { 7 }\n"), repo=anchor_root)
        code, lines = apply(anchor_pre, anchor_root, True, gate33)
        if code != 0 or not any("补丁前就红的 1 处" in line for line in lines):
            failures.append(f"补丁前就红的锚点不该拦住不相干的补丁（放行、提醒补丁前就红的 1 处），实际退 {code} {lines}")
        anchor_pre_new = patch_directory("anchorprenew", ("crates/demo/src/lib.rs", "fn a() -> u32 { 9 }\nfn b() -> u32 { 5 }\n"), repo=anchor_root)
        code, lines = apply(anchor_pre_new, anchor_root, True, gate33)
        if code != 2 or not any("keep-anchor" in line for line in lines) or any("old-broken" in line for line in lines):
            failures.append(f"弄断了变异锚点、补丁前也有别的红行时，--dry-run 应当只拒补丁带来的那一处（点名 keep-anchor、不点名 old-broken），实际退 {code} {lines}")
        # pathdep：另一个临时仓，crates/demo 经 path 依赖用仓根的 served/（验证包用仓根 prune-server/ 的样子）；变异行锚在 served/src/lib.rs，
        # 补丁改的也是那份文件里别的函数：临时树要连 served/ 一起拷，补丁套得上、锚点照找得到，放行、报 CHECKED；
        # 另一行变异行锚在补丁没改到的 served/src/other.rs 上：它在两棵临时树上都要判过，不算补丁前就红的（临时树不连 served/ 一起拷时，
        # 补丁改到的那份文件另拷得到，这一行的文件两棵树里都没有、两边都红、退 0 带「补丁前就红的」提醒）
        path_root = os.path.join(work, "path-dependency-repo")
        os.makedirs(os.path.join(path_root, "crates", "demo", "src"))
        os.makedirs(os.path.join(path_root, "served", "src"))
        open(os.path.join(path_root, "Cargo.toml"), "w").write('[workspace]\nmembers = ["crates/demo"]\nexclude = ["served"]\n')
        open(os.path.join(path_root, "crates", "demo", "Cargo.toml"), "w").write('[package]\nname = "demo"\n\n[dependencies]\nserved = { path = "../../served" }\n')
        open(os.path.join(path_root, "crates", "demo", "src", "lib.rs"), "w").write("fn a() -> u32 { 1 }\n")
        open(os.path.join(path_root, "served", "Cargo.toml"), "w").write('[package]\nname = "served"\n')
        open(os.path.join(path_root, "served", "src", "lib.rs"), "w").write("fn s() -> u32 { 1 }\nfn t() -> u32 { 5 }\n")
        open(os.path.join(path_root, "served", "src", "other.rs"), "w").write("fn o() -> u32 { 1 }\n")
        open(os.path.join(path_root, "crates", "mutations.tsv"), "w").write(
            "# 表头\nserved-anchor\tserved/src/lib.rs\tfn s() -> u32 { 1 }\tfn s() -> u32 { 2 }\t-p served --lib\tserved_test\n"
            "served-other-anchor\tserved/src/other.rs\tfn o() -> u32 { 1 }\tfn o() -> u32 { 2 }\t-p served --lib\tserved_test\n")
        write_sample_registration_table(path_root)
        for command in (["init", "-q"], ["add", "-A"], ["-c", "user.name=s", "-c", "user.email=s@s", "commit", "-qm", "base"]):
            git(path_root, *command, check=True)
        path_dependency = patch_directory("pathdep", ("served/src/lib.rs", "fn s() -> u32 { 1 }\nfn t() -> u32 { 6 }\n"), repo=path_root)
        code, lines = apply(path_dependency, path_root, True, gate33)
        if code != 0 or not any("核过了（--dry-run" in line for line in lines) or not any("CHECKED" in line for line in lines):
            failures.append(f"补丁与变异行锚在 path 依赖目录里时 --dry-run 应当照收（临时树连 served/ 一起拷）、放行并报 CHECKED，实际退 {code} {lines}")
        elif any("补丁前就红的" in line for line in lines):
            failures.append(f"补丁与变异行锚在 path 依赖目录里时 --dry-run 应当照收（补丁没改到的 served/src/other.rs 上的变异行在两棵临时树上都要判过，不算补丁前就红的），实际带了补丁前就红的提醒 {lines}")
        # reachabilityhints：假的 static-check（起的是 work 里写的一份小脚本，不起 crates-mutation-rows.py），可达性那一道的四种提示行都带随树变的东西——
        # 计数与行号照那棵树变异表的行数、原因与凭据里写着那棵树的路径；补丁只追加一行，两棵树的提示行逐行不同。提示行不算红：放行、报 CHECKED
        fake_static_check = os.path.join(work, "fake-static-check.py")
        with open(fake_static_check, "w", encoding="utf-8") as handle:
            handle.write("import os, sys\n"
                         "tree = sys.argv[2]\n"
                         "rows = [line for line in open(os.path.join(tree, 'crates', 'mutations.tsv'), encoding='utf-8') if line.strip() and not line.startswith('#')]\n"
                         "print('COPIED\\tcrates')\n"
                         "print(f'REACHABILITY_EXEMPT\\t{len(rows)}\\tkeep-anchor\\tcpu-configuration\\t{tree}/crates/demo/src/lib.rs::a')\n"
                         "print(f'REACHABILITY_UNJUDGED\\t{len(rows) + 1}\\tkeep-anchor\\t闭包在 {tree} 里算不成')\n"
                         "print(f'REACHABILITY_CHECKED\\t{len(rows)}\\t1\\t1\\t0\\t{tree}/multi-host.env\\tGPU 没开')\n"
                         "print(f'REACHABILITY_NOT_JUDGED\\t选路表点名的函数在 {tree} 的源码里一个都没有')\n"
                         "print(f'CHECKED\\t{len(rows)}\\t1\\t0')\n")
        hints_patch = patch_directory("reachabilityhints", append="hint-added\tcrates/demo/src/lib.rs\tfn b() -> u32 { 5 }\tfn b() -> u32 { 6 }\t-p demo --lib\tkept_test\n", repo=anchor_root)
        code, lines = apply(hints_patch, anchor_root, True, gate33, static_check=lambda tree: [sys.executable or "python3", fake_static_check, "static-check", tree])
        if code != 0 or not any("核过了（--dry-run" in line for line in lines):
            failures.append(f"可达性那一道的提示行（REACHABILITY_CHECKED、REACHABILITY_NOT_JUDGED、REACHABILITY_EXEMPT、REACHABILITY_UNJUDGED）随树变也不算红，--dry-run 应当放行，实际退 {code} {lines}")
        # unreachablerowshift：anchor_root 的变异表再追加两行（drop-before 在前、far-unreachable 在后）提交；另一份假的 static-check 照那棵树变异表里
        # far-unreachable 所在的行号打一行 UNREACHABLE（第二段是行号，与 crates-mutation-rows.py 那一行同形）。补丁删掉 drop-before：两棵树都有这一行红、
        # 行号差 1，是补丁前就有的，应当放行、提醒补丁前就红的 1 处，不点名 far-unreachable
        with open(anchor_table, "a") as handle:
            handle.write("drop-before\tcrates/demo/src/lib.rs\tfn b() -> u32 { 5 }\tfn b() -> u32 { 8 }\t-p demo --lib\tkept_test\n"
                         "far-unreachable\tcrates/demo/src/lib.rs\tfn b() -> u32 { 5 }\tfn b() -> u32 { 9 }\t-p demo --lib\tunreached_test\n")
        for command in (["add", "-A"], ["-c", "user.name=s", "-c", "user.email=s@s", "commit", "-qm", "补丁之前就有一行点名测试够不着的变异"]):
            git(anchor_root, *command, check=True)
        fake_unreachable_check = os.path.join(work, "fake-unreachable-static-check.py")
        with open(fake_unreachable_check, "w", encoding="utf-8") as handle:
            handle.write("import os, sys\n"
                         "tree = sys.argv[2]\n"
                         "lines = open(os.path.join(tree, 'crates', 'mutations.tsv'), encoding='utf-8').read().splitlines()\n"
                         "number = next(index for index, line in enumerate(lines, 1) if line.startswith('far-unreachable\\t'))\n"
                         "print('COPIED\\tcrates')\n"
                         "print(f'UNREACHABLE\\t{number}\\tfar-unreachable\\tcpu\\tdemo::b\\tunreached_test\\t-p demo --lib')\n"
                         "print(f'CHECKED\\t{len(lines)}\\t1\\t0')\n")
        row_shift = patch_directory("unreachablerowshift", delete="drop-before\tcrates/demo/src/lib.rs\tfn b() -> u32 { 5 }\tfn b() -> u32 { 8 }\t-p demo --lib\tkept_test\n",
                                    repo=anchor_root)
        code, lines = apply(row_shift, anchor_root, True, gate33, static_check=lambda tree: [sys.executable or "python3", fake_unreachable_check, "static-check", tree])
        if code != 0 or not any("补丁前就红的 1 处" in line for line in lines) or any("far-unreachable" in line for line in lines):
            failures.append(f"补丁删了前面一行变异、补丁前就有的 UNREACHABLE 行号跟着漂，--dry-run 应当照补丁前就红的放行（提醒补丁前就红的 1 处、不点名 far-unreachable），"
                            f"实际退 {code} {lines}")
        # releaserowshift：同一棵树、同一份补丁（删掉 drop-before），另一份假的 static-check 照 far-unreachable 所在的行号打一行 RELEASE（第二段是行号，
        # 与 crates-mutation-rows.py 那一行同形）：行号差 1，是补丁前就有的，应当放行、提醒补丁前就红的 1 处，不点名 far-unreachable
        fake_release_check = os.path.join(work, "fake-release-static-check.py")
        with open(fake_release_check, "w", encoding="utf-8") as handle:
            handle.write("import os, sys\n"
                         "tree = sys.argv[2]\n"
                         "lines = open(os.path.join(tree, 'crates', 'mutations.tsv'), encoding='utf-8').read().splitlines()\n"
                         "number = next(index for index, line in enumerate(lines, 1) if line.startswith('far-unreachable\\t'))\n"
                         "print('COPIED\\tcrates')\n"
                         "print(f'RELEASE\\t{number}\\tfar-unreachable\\tcrash-case 目标 demo:case_a（--test 点名；登记 crash-case:demo）')\n"
                         "print(f'CHECKED\\t{len(lines)}\\t1\\t0')\n")
        code, lines = apply(row_shift, anchor_root, True, gate33, static_check=lambda tree: [sys.executable or "python3", fake_release_check, "static-check", tree])
        if code != 0 or not any("补丁前就红的 1 处" in line for line in lines) or any("far-unreachable" in line for line in lines):
            failures.append(f"补丁删了前面一行变异、补丁前就有的 RELEASE 行号跟着漂，--dry-run 应当照补丁前就红的放行（提醒补丁前就红的 1 处、不点名 far-unreachable），"
                            f"实际退 {code} {lines}")
        # 三方合并（文件头「三方合并」）：另一个临时仓，lib.rs 二十行、已提交；补丁改第 10 行，别的会话在工作区里改了第 8 行（落在补丁的上下文里，git apply --check 不过）。
        # threeway-merged：两处隔一行、不重叠，合上、退 0、两处都在、报合了哪个文件哪几段；threeway-conflict：别的会话改的就是第 10 行，判冲突、退 2、点名文件与段落、一个字不改。
        # 基线取不到那几格的补丁出自另一个仓（当实现员副本：第 3 行与这边不同，那份基线没进过这边的对象库）：threeway-baseline-missing 什么都不给，退 2；
        # threeway-baseline-mismatch 在补丁目录 baseline/ 里放这边那一份（哈希对不上），退 2；threeway-baseline-directory 放副本那一份，照它合上、退 0（第 3 行留这边的）。
        # threeway-baseline-wrong：index 行左边改成变异表的 blob（对象库里有、却不是这份文件的基线），套不上、退 2；threeway-unsupported：文件头改成改名段，退 2、写明只认改增删。
        # 几格都走副本那一支（临时仓不是这份脚本所在的仓）
        merge_root = os.path.join(work, "three-way-repo")
        merge_relative = "crates/demo/src/lib.rs"
        merge_source = os.path.join(merge_root, merge_relative)
        os.makedirs(os.path.dirname(merge_source))

        def numbered_source(changed):
            """二十行的样本源码，changed 里点名的行号换成给的值。"""
            return "".join(f"fn line_{number:02}() -> u32 {{ {changed.get(number, number)} }}\n" for number in range(1, 21))

        def tree_contents(tree):
            """树里除 .git 之外每份文件的字节：判「一个字不改」。"""
            contents = {}
            for directory, subdirectories, files in os.walk(tree):
                subdirectories[:] = [name for name in subdirectories if name != ".git"]
                for name in files:
                    with open(os.path.join(directory, name), "rb") as handle:
                        contents[os.path.relpath(os.path.join(directory, name), tree)] = handle.read()
            return contents

        def rewrite_patch_header(directory, rewrite):
            """补丁目录里 crates.patch 的每一行过一遍 rewrite（行 → 行），写回。"""
            patch_file = os.path.join(directory, "crates.patch")
            text = open(patch_file).read()
            open(patch_file, "w").write("".join(rewrite(line) for line in text.splitlines(keepends=True)))

        merge_base = numbered_source({})
        open(merge_source, "w").write(merge_base)
        open(os.path.join(merge_root, "crates", "mutations.tsv"), "w").write("# 表头\n")
        for command in (["init", "-q"], ["add", "-A"], ["-c", "user.name=s", "-c", "user.email=s@s", "commit", "-qm", "base"]):
            git(merge_root, *command, check=True)
        merged_patch = patch_directory("threeway-merged", (merge_relative, numbered_source({10: 100})), repo=merge_root)
        conflict_patch = patch_directory("threeway-conflict", (merge_relative, numbered_source({10: 100})), repo=merge_root)
        wrong_patch = patch_directory("threeway-baseline-wrong", (merge_relative, numbered_source({10: 100})), repo=merge_root)
        unsupported_patch = patch_directory("threeway-unsupported", (merge_relative, numbered_source({10: 100})), repo=merge_root)
        table_blob = git(merge_root, "rev-parse", "HEAD:crates/mutations.tsv").stdout.strip()
        rewrite_patch_header(wrong_patch, lambda line: f"index {table_blob[:12]}..{line.split('..', 1)[1]}" if line.startswith("index ") else line)
        moved_relative = "crates/demo/src/moved.rs"
        rewrite_patch_header(unsupported_patch, lambda line: (
            f"diff --git a/{merge_relative} b/{moved_relative}\nsimilarity index 95%\nrename from {merge_relative}\nrename to {moved_relative}\n"
            if line.startswith("diff --git ") else f"+++ b/{moved_relative}\n" if line.startswith("+++ ") else line))
        writer_copy = os.path.join(work, "three-way-writer-copy")
        os.makedirs(os.path.dirname(os.path.join(writer_copy, merge_relative)))
        copy_base = numbered_source({3: 33})
        open(os.path.join(writer_copy, merge_relative), "w").write(copy_base)
        for command in (["init", "-q"], ["add", "-A"], ["-c", "user.name=s", "-c", "user.email=s@s", "commit", "-qm", "实现员副本"]):
            git(writer_copy, *command, check=True)
        missing_patch, mismatch_patch, directory_patch = (
            patch_directory(name, (merge_relative, numbered_source({3: 33, 10: 100})), repo=writer_copy)
            for name in ("threeway-baseline-missing", "threeway-baseline-mismatch", "threeway-baseline-directory"))
        for directory, text in ((mismatch_patch, merge_base), (directory_patch, copy_base)):
            stored = os.path.join(directory, BASELINE_DIRECTORY, merge_relative)
            os.makedirs(os.path.dirname(stored))
            open(stored, "w").write(text)
        open(merge_source, "w").write(numbered_source({8: 80}))
        code, lines = apply(merged_patch, merge_root, False, gate33)
        after = open(merge_source).read()
        if code != 0 or after != numbered_source({8: 80, 10: 100}) or not any(line.startswith("三方合并：") for line in lines) \
                or not any(merge_relative in line and "合了" in line and "@@ -7,7 +7,7 @@" in line and "基线第 8 行" in line for line in lines):
            failures.append(f"上下文被别的会话改过、改动不重叠的补丁应当三方合上、退 0、报合了哪个文件哪几段，实际退 {code} {lines}，文件 {after!r}")
        open(merge_source, "w").write(numbered_source({10: 99}))
        before = tree_contents(merge_root)
        code, lines = apply(conflict_patch, merge_root, False, gate33)
        if code != 2 or tree_contents(merge_root) != before \
                or not any(line.startswith("✗ 冲突 ") and merge_relative in line and "基线第 10 行" in line and "主工作区此刻第 10 行" in line for line in lines):
            failures.append(f"别的会话改了补丁改的同一行，应当判冲突、退 2、点名文件与段落、一个字不改，实际退 {code} {lines}")
        open(merge_source, "w").write(numbered_source({8: 80}))
        for name, directory, wanted in (("missing", missing_patch, "对象库里没有"), ("mismatch", mismatch_patch, "对不上"),
                                        ("wrong", wrong_patch, "套不上"), ("unsupported", unsupported_patch, "只认文本文件的改、增、删")):
            before = tree_contents(merge_root)
            code, lines = apply(directory, merge_root, False, gate33)
            refused = code == 2 and tree_contents(merge_root) == before and any(wanted in line and line.startswith("✗ ") for line in lines)
            if name == "missing" and (not refused or not any("基线取不到" in line for line in lines)
                                      or not any(line.lstrip().startswith("→") and f"{BASELINE_DIRECTORY}/" in line for line in lines)):
                failures.append(f"补丁头的基线 blob 在对象库里没有、补丁目录里也没有 baseline/ 时应当退 2、写明基线取不到与出路、一个字不改，实际退 {code} {lines}")
            elif name == "mismatch" and not refused:
                failures.append(f"baseline/ 里那一份的哈希对不上补丁头时应当当基线取不到、退 2、一个字不改，实际退 {code} {lines}")
            elif name == "wrong" and not refused:
                failures.append(f"补丁头点到的基线套不上补丁时应当当基线取不到、退 2、一个字不改，实际退 {code} {lines}")
            elif name == "unsupported" and not refused:
                failures.append(f"带改名段的补丁上下文漂了时应当退 2、写明三方合并只认文本文件的改、增、删、一个字不改，实际退 {code} {lines}")
            if tree_contents(merge_root) != before:
                # 弄坏开关下照打了：这份文件退回去，下一格只判自己的
                open(merge_source, "w").write(numbered_source({8: 80}))
        code, lines = apply(directory_patch, merge_root, False, gate33)
        after = open(merge_source).read()
        if code != 0 or after != numbered_source({8: 80, 10: 100}) or not any(f"{BASELINE_DIRECTORY}/{merge_relative}" in line and "合了" in line for line in lines):
            failures.append(f"基线 blob 不在对象库里而补丁目录 baseline/ 里有哈希对得上的那一份时应当照它三方合上、退 0，实际退 {code} {lines}，文件 {after!r}")
        # 没认出补丁（文件头「补丁目录里的文件」）：另一个临时仓。unrecognized：目录里只有 all.patch（补丁本身是好的、套得上；改 crates 之外的文件的补丁就是这样交的，
        # 不拒就会一个字没打却报「打上了」退 0），拒、退 2、点名 all.patch、出路写 git apply、仓里一个字不改；emptydirectory：一个文件都没有，同样拒；
        # alongside：crates.patch 与别的 *.patch 同在，照打 crates.patch、退 0（只管一份都没认出的这一支）
        loose_root = os.path.join(work, "loose-patch-repo")
        loose_source = os.path.join(loose_root, "crates", "demo", "src", "lib.rs")
        os.makedirs(os.path.dirname(loose_source))
        open(loose_source, "w").write("fn a() -> u32 { 1 }\n")
        open(os.path.join(loose_root, "crates", "mutations.tsv"), "w").write("# 表头\n")
        for command in (["init", "-q"], ["add", "-A"], ["-c", "user.name=s", "-c", "user.email=s@s", "commit", "-qm", "base"]):
            git(loose_root, *command, check=True)
        unrecognized_patch = patch_directory("unrecognized", ("crates/demo/src/lib.rs", "fn a() -> u32 { 2 }\n"), repo=loose_root)
        os.rename(os.path.join(unrecognized_patch, "crates.patch"), os.path.join(unrecognized_patch, "all.patch"))
        empty_directory = patch_directory("emptydirectory")
        alongside_patch = patch_directory("alongside", ("crates/demo/src/lib.rs", "fn a() -> u32 { 2 }\n"), repo=loose_root)
        open(os.path.join(alongside_patch, "all.patch"), "w").write("")
        before = tree_contents(loose_root)
        code, lines = apply(unrecognized_patch, loose_root, False, gate33)
        if code != 2 or tree_contents(loose_root) != before or not any(line.startswith("✗ ") and "没有它认得的补丁" in line and "all.patch" in line for line in lines) \
                or not any("git apply" in line for line in lines):
            failures.append(f"补丁目录里只有 all.patch、没有一样认得的补丁，应当拒绝、退 2、点名 all.patch、出路写 git apply、一个字不改，实际退 {code} {lines}")
        code, lines = apply(empty_directory, loose_root, False, gate33)
        if code != 2 or tree_contents(loose_root) != before or not any(line.startswith("✗ ") and "没有它认得的补丁" in line for line in lines):
            failures.append(f"一样补丁都没有的目录应当拒绝、退 2、点名没有认得的补丁、一个字不改，实际退 {code} {lines}")
        code, lines = apply(alongside_patch, loose_root, False, gate33)
        if code != 0 or open(loose_source).read() != "fn a() -> u32 { 2 }\n":
            failures.append(f"crates.patch 与别的 *.patch 同在时应当照打 crates.patch、退 0，实际退 {code} {lines}")
        # reapplied：另一个临时仓，同一份补丁打进去之后再交一次。git apply --check 不过、三方合并合后与主工作区此刻一样，要打的补丁是 None（文件头「三方合并」）：
        # 应当不崩、退 0、报「没有要打的」与「补丁 没有」、仓里一个字不改。抛出来的记成这一格的失败，后面的格照跑
        reapply_root = os.path.join(work, "reapplied-repo")
        reapply_source = os.path.join(reapply_root, "crates", "demo", "src", "lib.rs")
        os.makedirs(os.path.dirname(reapply_source))
        open(reapply_source, "w").write("fn a() -> u32 { 1 }\n")
        open(os.path.join(reapply_root, "crates", "mutations.tsv"), "w").write("# 表头\n")
        for command in (["init", "-q"], ["add", "-A"], ["-c", "user.name=s", "-c", "user.email=s@s", "commit", "-qm", "base"]):
            git(reapply_root, *command, check=True)
        reapplied_patch = patch_directory("reapplied", ("crates/demo/src/lib.rs", "fn a() -> u32 { 2 }\n"), repo=reapply_root)
        landed_code, landed_lines = apply(reapplied_patch, reapply_root, False, gate33)
        before = tree_contents(reapply_root)
        try:
            code, lines = apply(reapplied_patch, reapply_root, False, gate33)
        except Exception as error:   # 不崩是这一格判的事：抛出来的照失败记，不让整份自证停在这里
            code, lines = None, [f"抛了 {type(error).__name__}：{error}"]
        if landed_code != 0 or code != 0 or tree_contents(reapply_root) != before or open(reapply_source).read() != "fn a() -> u32 { 2 }\n" \
                or not any("没有要打的" in line for line in lines) or not any(line.startswith("✓ 打上了") and "补丁 没有" in line for line in lines):
            failures.append(f"补丁已打进时再交同一份应当不崩、照没有改动报（退 0、报合后与主工作区此刻一样没有要打的、补丁 没有、仓里一个字不改），"
                            f"实际打进去那一趟退 {landed_code} {landed_lines}，再交那一趟退 {code} {lines}")
        failures += landing_selftest(work)
        for failure in failures:
            print(f"  ✗ 自检：{failure}")  # gate-lint:detail
        if failures:
            print("  → 看 plan() / apply() 的判法；APPLY_WRITER_PATCH_BREAK 设着的话这里本来就该红")
            return 1
        print("  ✓ apply-writer-patch 自检通过：好补丁打上、变异表按名字追加替换删除（替换与删除先逐字比对前像：对得上照换照删，有一行对不上、有一行没带前像都整份拒、一个字不改），动 checker 没那一节、追加重名（表里已有的、追加表自己里两行同名的）、打不上的都在核的时候拒绝，到提示线打提示，副本锁被占着时拒绝、没占时照打、别的机器上的持有者记录刷新过就拒绝、过期的与这一台的旧记录接手、"
              "同一任务更早的尝试写的当场接手（任务号不同、尝试号不比这一趟小、这一趟不经服务起的照拒，拒绝那一句写出记录里的任务号与尝试号），"
              "补丁改到开着的三方轮开工快照里的文件（代码与变异表）拒、命令行给了快照冲突已判的照打，"
              "快照清单读不了的照打、点名那一轮与那份清单并记一条检出，"
              "--dry-run 在套上补丁的临时树上跑变异锚点静态核（弄断锚点的补丁拒、不弄断的放行、补丁前就红的不算补丁带来的、临时树连 path 依赖的目录一起拷、补丁改拷贝范围之外的文件照常判、补丁前后两棵树同一刻拷出而不拿活的主工作区当基线、可达性那一道的提示行随树变也不算红），"
              "上下文被别的会话改过的补丁照基线三方合并（不重叠的合上并报段、同一行判冲突点名段落、基线 blob 对象库里没有的拒、baseline/ 里哈希对得上的照它合、"
              "对不上的拒、index 行点错 blob 套不上的拒、改名段拒，拒的都一个字不改；已打进的补丁再交一次合后与此刻一样，照没有要打的退 0、不崩），"
              "补丁目录里一样认得的补丁都没有（只有别的 *.patch、空目录）拒并点名、与 crates.patch 同在的别的 *.patch 不挡，"
              "换上：暂存到一半被杀主工作区不动、暂存出来的文件对不上 sha256 拒、换到一半被杀下一次认出来并列出已换上与没换上、退回与接着换完、回读对不上写异常，"
              "结局文件开始到完成写全、被杀的只有开始、打全之后又被改的 --read-outcome 点名，真打与接着换完都不在本进程里起 mutation-tables 那一格、打出要另交的命令，走不到的断言照补丁打上之后那份角色表只判产品与没登记角色的包（验证、测试支撑、实验装置放行，没登记的与读不到表的拒并点名没登记角色），父模块文件里 #[cfg(test)] mod 声明引进来的整份文件（祖先一层、mod.rs、#[path] 落的都算）里的断言放行、没有这种声明的文件照判（查了 62 种）")
        return 0
    finally:
        shutil.rmtree(work, ignore_errors=True)


BREAK_VARIABLES = ("APPLY_WRITER_PATCH_BREAK",)


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
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一项，跑 {setting[0]}=<项> python3 scripts/gate/apply-writer-patch.py --selftest", file=sys.stderr)
    return 2


if __name__ == "__main__":
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    arguments = sys.argv[1:]
    if arguments == ["--selftest"]:
        sys.exit(selftest())
    root, explicit_root = REPOSITORY_ROOT, None
    if "--root" in arguments:
        position = arguments.index("--root")
        root = explicit_root = os.path.abspath(arguments[position + 1])
        arguments = arguments[:position] + arguments[position + 2:]
    gate_command = ["bash", os.path.join(root, ".claude", "gate.d", "code-source-discipline.sh"), "--check", "mutation-tables"]   # 只在弄坏开关 mutation-tables-in-process 下起（文件头「mutation-tables 那一格」）
    subcommand = next((argument for argument in arguments if argument in ("--pending-status", "--finish-pending", "--rollback-pending", "--read-outcome")), None)
    if subcommand is not None:
        PROGRESS_ENABLED = True
        rest = [argument for argument in arguments if argument != subcommand]
        if subcommand == "--read-outcome" and len(rest) == 1:
            code, lines = read_outcome(rest[0], explicit_root)
        elif subcommand != "--read-outcome" and not rest:
            code, lines = {"--pending-status": lambda: pending_status(root), "--finish-pending": lambda: finish_pending(root, gate_command),
                           "--rollback-pending": lambda: rollback_pending(root)}[subcommand]()
        else:
            code, lines = 2, [f"✗ {subcommand} 的用法不对",
                              f"→ 怎么办：{COMMAND_NAME} {subcommand}{' <结局文件>' if subcommand == '--read-outcome' else ''} [--root 仓根]"]
        print("\n".join(lines))
        sys.exit(code)
    dry_run = "--dry-run" in arguments
    arguments = [argument for argument in arguments if argument != "--dry-run"]
    if any(argument == "--snapshot-ruled" or argument.startswith("--snapshot-ruled=") for argument in arguments):
        print("✗ --snapshot-ruled 删了：开工快照没有单次放过的口子，这一次什么都没做\n"
              "→ 怎么办：去掉它；补丁撞了开着的三方轮的快照，等那一轮的判决写完再打，或用户已叫停的轮在 research/prompts/abandoned-rounds.tsv 登记一行收掉（文件头「开工快照」）")
        sys.exit(2)
    if len(arguments) != 1 or not os.path.isdir(arguments[0]):
        print("✗ 要给一个补丁目录\n→ 怎么办：apply-writer-patch.py <补丁目录> [--root 仓根] [--dry-run]")
        sys.exit(2)
    PROGRESS_ENABLED = True
    code, lines = apply(arguments[0], root, dry_run, gate_command)
    print("\n".join(lines))
    sys.exit(code)
