#!/usr/bin/env python3
# admission: always 每一次换钩子都要现判这一刻的规格、草稿与主工作区那几份，上一次换过不替这一次作保
# run-condition: command bash python3
"""先验后换：.claude/hooks/ 下的钩子与它读的共用模块、登记表，或脚本规格目录（见 <规格.json>）下的脚本，先在草稿里套上定点替换验过，再在主工作区定点换上；
换上之后再验一遍，不对就按反向定点替换回滚（只把这一次换上的几处改回去，不整份覆盖，别的会话这段时间里改的照留）。
两种规格各走一遍，不混在一份里：钩子规格（文件在 .claude/hooks/ 这一层）要 --hook、拿钩子逐条喂；脚本规格（文件在脚本规格目录下，不限层数）不喂钩子，
换上之后改跑 --verify 给的命令（一般是被换脚本的 --selftest）。

用法：
  hook-swap.py <规格.json> --hook <钩子> --scratch <草稿目录> [--feeds <喂法.json>] [--root <仓根>] [--dry-run]
  hook-swap.py <规格.json> --scratch <草稿目录> [--verify <命令>] [--root <仓根>] [--dry-run]   # 脚本规格：文件都在脚本规格目录下，不带 --hook 与 --feeds
  hook-swap.py --feed-only --hook <钩子> [--feeds <喂法.json>] [--root <仓根>]   # 只喂不换：拿此刻的钩子逐条喂，一条一行写退出码，不写任何文件
  hook-swap.py --check-callers [<库.py>…] [--root <仓根>]   # 只核不换：调用方引到的名字库里都有没有（不写库就核 .claude/hooks/ 下全部 .py），不写任何文件
  hook-swap.py --read-outcome [--root <仓根>] [--scratch <草稿目录>]   # 只看结局文件：上一趟换没换完（文件头「结局文件」），不写任何文件
  hook-swap.py --seal-outcome [--root <仓根>] [--scratch <草稿目录>]   # 上一趟几份文件都已是换后的内容、只缺完成行：验过之后补写完成
  hook-swap.py --selftest        # HOOK_SWAP_BREAK=<项> 时必须判红

<规格.json> 同 scripts/kb/replace-batch.py：JSON 数组，每项 {"file": 路径, "old": 旧串, "new": 新串, "count": 可省，默认 1}，
  同一个文件的多处按数组次序依次施加；file 写相对仓根的路径或绝对路径，只收 <仓根>/.claude/hooks/ 这一层里已有的普通文件（钩子规格），
  或脚本规格目录下（任意层）已有的普通文件（脚本规格）；钩子规格与脚本规格的文件不能写进同一份规格；旧串、新串都不许空。
  脚本规格目录（SCRIPT_SPEC_DIRECTORIES，相对仓根）：scripts/、.claude/gate.d/、.claude/scripts/、task-server/、prune-server/、gpu-verification-server/，
  这几处的文件可以写进同一份脚本规格；其中不收的子目录（SCRIPT_SPEC_EXCLUDED）：.claude/gate.d/fixtures/（门禁样本）、prune-server/ 与
  gpu-verification-server/ 下的 target/（编译产物）与 src/（Rust 源码，在仓副本里改、存成补丁由主 agent 一次打上）。
  .claude/hooks/write-guard.sh 第十一道认同一组目录：工具员不在主工作区里用 Write / Edit 就地改这几处已存在的脚本，新版写在草稿目录、经这条脚本一次换上；
  两个服务的 src/ 下已存在的文件它同样拒，出路是仓副本与补丁。
  规格用 scripts/gate/make-swap-spec.py 从基线与新版生成（每处正着倒着都恰好命中一次），不手写、不另写生成规格的小脚本。
<钩子> 是要喂 JSON 的那一份（.claude/hooks/ 下的 .sh），不在规格里也照样做语法检查与喂法。
<喂法.json>：JSON 数组，每项 {"label": 说明, "command": Bash 命令, "exit": 0 或 2, "agent_type": 可省（省了是主 agent）, "cwd": 可省（默认 {root}），
  "neutral": 可省（true 是新旧两版判得一样的，回滚之后拿它们再核一遍；套规格之前也先拿换上之前那一版在草稿树里喂一遍，对不上的是喂法失效，退 8，--feed-only 不走这一步）}；
  也可以写 "input": 一整份钩子 JSON 代替 command、cwd 与 agent_type。
  command、cwd 与 input 里的 {root} 换成这一次判的仓根（草稿里是草稿树，换上之后是主工作区）。
  喂法里 tool_input.file_path（按落点判的钩子，例如 write-guard.sh）解开符号链接之后要落在 {root} 里：草稿树里只有 .claude/hooks/ 是实拷的，
  仓根与 .claude/ 下别的项都是指回主工作区的符号链接，落在树外的目标被按落点判的钩子一律当仓外放行，期望放行的喂法也「过」、等于没判；
  落在树外的那一条不喂，点名它，目标写成 {root}/.claude/hooks/<名字>（相对路径照 {root} 算）。
  {root} 跟着树走：同一条喂法在草稿里验（目标在草稿目录里）与换上之后验、回滚之后核（目标在主工作区里）是两个落点；钩子按落点分工作根内外时，
  期望随落点变的喂法在这两处不会都成立，写不进会换上的规格。
  不给 --feeds 时用内置的那一份（BUILTIN_FEEDS，按钩子相对仓根的路径取；没有内置的就要给）。
<命令>（--verify，只给脚本规格）：换上之后在 <仓根> 里用 bash -c 跑一遍，退 0 才算验过；写被换脚本的 --selftest（例：python3 scripts/kb/replace-once.py --selftest），
  跑得过 VERIFY_TIMEOUT_SECONDS 秒的算不过。命令只在换上之后跑，草稿里（没有仓的样子）只做语法检查；不给 --verify 就只做换上之后的语法检查与回读。
  起编译或真虚机的命令不要写进来（自证都用假驱动、秒级；要跑别的重活的命令经 task-server 交）。
在哪台跑：两台都跑得起，经统一任务服务交、派到哪台由服务定；第二台在共享的活树里、同一个绝对路径上跑，读写的就是本机主工作区那一份，
  规格、草稿目录与 --root 都写落在多机配置 NFS_SHARED_ROOTS 共享根下的绝对路径。同一个 --scratch 同一时刻只许一趟在跑（--dry-run 同样）：
  草稿树 <草稿目录>/hook-swap/tree 只有一棵，并行起会互相删掉、盖掉，所以一开跑就占住草稿目录（下面「草稿目录的占用」），占不到退 10、一个字节不写；
  几条 --dry-run 要并行就各给一个 --scratch。真换（含 --seal-outcome）在同一个仓里一次只许一趟：结局文件按仓只有一份，另一趟真换在跑时换草稿目录也占不到，
  在读结局文件之前退 10、一个字节不写（下面「按仓的占用」）。
撤回一次换上：把规格倒过来（old 与 new 对调、次序倒排；每次换上都写一份在 <草稿目录>/hook-swap/reverse-<时刻>.json）再跑一遍，
  --feeds 给一份不含这一次新判法的喂法。

结局文件（真换时写，--dry-run 不写）：几份文件是逐份改名换上的，两份之间被杀（内存上限、断电），回滚来不及跑，几份文件就停在半截。
  所以换上之前先把这一趟要换哪几份、各自换前与换后的 sha256、反向规格在哪写进结局文件（同目录临时文件写完、刷盘再改名），全部换完、回读过、换上之后再验过，才写「完成」。
  位置：<仓根>/.git 的 git-dir 下的 singlefs-hook-swap-outcome.tsv（每次真换整份重写，只留最近一趟）；仓根不是 git 工作树的根时放 <草稿目录>/hook-swap/outcome.tsv（下一趟要用同一个 --scratch）。
  格式：制表符分隔，`#` 起头的行是说明，其余每行第一段是行种类：
    开始<TAB>仓根<TAB>进程号<TAB>开始秒数<TAB>反向规格路径<TAB>反向规格 sha256<TAB>机器<TAB>开机<TAB>进程启动刻
                                    （机器是 /etc/machine-id 的 sha256 前 16 位，开机是 /proc/sys/kernel/random/boot_id，进程启动刻是 /proc/<进程号>/stat 第 22 列；
                                      读不到的写 -。三段合起来认出起这一趟的是哪一台、哪一次开机里的哪一个进程，进程号被别的进程占了也认得出；
                                      认出是哪一台靠两台的 /etc/machine-id 不同，两台相同时（克隆出来的镜像）判错：另一台起的、活着的持有者在这一台上读成「机器相同、开机不同」，
                                      判成不在、立刻抢；判法不改，两台的 machine-id 要不同）
    文件<TAB>相对仓根的路径<TAB>换前 sha256<TAB>换后 sha256            （开始之后、动手换之前写全，每份一行）
    回读<TAB>相对仓根的路径<TAB>回读算出的 sha256                        （全部换上之后每份一行）
    完成<TAB>份数<TAB>结束秒数                                          （换上、回读、换上之后再验都过了才写；--seal-outcome 补写的末尾另有一段「补记」）
    退回<TAB>份数<TAB>结束秒数                                          （换上之后验不过、或换的时候出错，已按反向定点替换回滚干净之后写，替代完成）
  没有完成也没有退回就是没换完。判法只看内容：对着主工作区现算每份文件的 sha256，等于换后的是「已换上」，等于换前的是「没换上」，都不等的是「被别的改动动过」。
  下一趟（含 --dry-run）起手先读这份文件：没换完、几份文件不是都在换前，就拒绝再换、一个字节不写，拒之前先看开始行记的那个进程：
    还在（机器与开机都是这一台此刻的，/proc/<进程号> 在、不是僵尸、启动刻对得上）：那一趟还在换或在验，退 9，说明写进程号与起了多少秒、叫等它写完成再来，不给退回与补写的出路；
    不在（同一台而开机变了、/proc/<进程号> 没有、是僵尸、启动刻对不上）：退 7，拒绝说明逐份点名状态，写出怎么退回（已换上的几份按反向规格里属于它的几处
      用 Edit 改回）或补完（没换上的几份按规格用 Edit 换上，全到换后；先 --feed-only 与 --check-callers 验过，再 --seal-outcome 补写完成）；
    在另一台起的（机器对不上，或有一边读不到机器标识）：这一台看不到那个进程，退 7，说明先叫看起它的那一趟还在不在跑，再给同样的退回与补完。
  几份文件全回到换前（手工退回之后，或被杀在第一份之前），按内容判就不再拦，不用另外清；全到换后、没写完成的，要 --seal-outcome 补写。
  开始行不是 9 段的不是这一版写的结局文件，照读不了办（拒，出路是核过几份文件之后删掉它）。
  --read-outcome 只看不写：退 0 换完了（有完成，每份文件现在都等于回读），1 已退回（有退回，每份都等于换前），2 读不了、没有结局文件或不是结局文件，
  3 没换完（逐份列已换上、没换上、被动过），4 完成或退回之后文件又被改了（逐份点名）。

草稿目录的占用（防同一个 --scratch 上并发；--read-outcome、--seal-outcome、--feed-only、--check-callers 不占这一份，--seal-outcome 占下面「按仓的占用」那一份）：
  一开跑，草稿目录核过不在主工作区里之后、读结局文件与建树之前，在 <草稿目录>/hook-swap/swap.lock 上放一份占用记录，这一趟退出（含 --dry-run）时删掉。
  记录一行，制表符分隔：机器（/etc/machine-id 的 sha256 前 16 位）、开机（boot_id）、进程号、进程启动刻、开始秒数、随机串、主机名。
  放法：先在同目录写好临时文件（刷盘），再 os.link 成 swap.lock——link 在文件系统那一侧原子、已有就报 EEXIST，所以谁放成了没有二义，记录出现时内容已写全。
  占不到（记录在、写它的那一趟还占着）：不建树、不写任何文件，退 10，拒绝说明写草稿目录、持有者的进程号、主机名、这一台还是另一台、开跑时刻、已跑多少秒与记录多少秒前刷新过，
    出路是等那一趟结束再交、或换一个 --scratch（--dry-run 照常并行；真换还要过下面按仓的那一道）。
  放成：link 报已有、而读回来的就是这一趟自己写的那份记录（共享盘上 link 在服务端做成、回复丢了、重发时报已有），当放成；读回来是别人的才算被占。
  写记录的那一趟还占不占着：
    这一台起的（机器、开机、进程号、启动刻都对得上，认法同「结局文件」的 swap_process_state）：进程在就占着，进程没了（被杀、断电重启、进程号被别的进程占了）立刻不挡，不管记录多新；
    另一台起的（或有一边读不到机器标识；记录写不全的同样）：这一台看不到进程，看记录的修改时间：持有期间每 SCRATCH_LOCK_HEARTBEAT_SECONDS 秒刷新一次，
      SCRATCH_LOCK_STALE_SECONDS 秒内刷新过的算占着，更久没刷新的当它被杀了。另一台上被杀的持有者最多让后来的等这么久，拒绝说明里写了「确认它死了就删记录」的出路。
  不再挡的旧记录：先在 <记录>.taking 上排他放一份这一趟自己的身份（内容与记录同一套字段：写同目录临时文件、刷盘，再 os.link 成 .taking，已有就报已有；记录从头到尾不挪开，
    别的趟不会在中间看到一个空位放进去），放成了再读记录：不是刚才判过不挡的那一份（别的趟刚放了新的，或已经没了）就只放掉 .taking、记录不动，重判；是才删记录、再放掉 .taking，然后放自己的。
    .taking 在、不是自己的：用判记录的同一个判法（下面「写记录的那一趟还占不占着」）判写 .taking 的那一趟，还占着就退 10，拒绝说明点名那一趟（进程号、主机、这一台还是另一台）；
    不再占着（被 KILL、被 TERM 来不及放、进程没了，另一台起的 SCRATCH_LOCK_STALE_SECONDS 秒没刷新）的当场摘掉、重来，不留着挡后来的。
    摘已死的 .taking 只许一趟动手：先在 <.taking>.reap-<读到的 .taking 内容的 sha256 前 16 位> 上排他放一份这一趟自己的身份（摘取记录，放法同记录；名字跟着那一份已死的内容走，
    同一份只有一趟放得成），放成的那一趟再读 .taking，还是判过的那一份才删，再放掉摘取记录、重来；放不成的判写摘取记录的那一趟（同一个判法），还占着就退 10、拒绝说明点名它；
    不再占着（放成之后、删之前被 KILL）的照同一个办法摘掉那份摘取记录再来，一层层往下最多 SCRATCH_LOCK_ATTEMPTS 层，再深就当被占、退 10。
    摘的那一趟放着 .taking 的期间每 SCRATCH_LOCK_HEARTBEAT_SECONDS 秒刷新一次它的修改时间（同占用记录的心跳），另一台上还活着的摘的那一趟不会被后来的当成过期。
    删记录之前与之后各核一次 .taking 上还是不是这一趟放的：不是了，就是这一趟停住时被当成已死、别的趟摘掉 .taking 接手了，这一趟不占、退 10，拒绝说明写被接手。
    link 报已有而读回来就是这一趟自己写的（共享盘上 link 在服务端做成、回复丢了、重发报已有），当放成。
  放锁：正常返回、抛异常（含 BadInput）、被 Ctrl-C 都在 finally 里删记录；TERM 与 HUP 由装上的处理先放掉这一趟放着的 .taking 与摘取记录、再删记录、再还原默认处理并重发同一个信号（退出状态仍是被这个信号杀的）；
    处理在占住记录时装上，摘旧记录的那一段（记录还没放成）也装着；只删内容还是自己写的那一份。被 KILL 的留下记录、.taking 与摘取记录，靠上面的「进程没了就不挡」与心跳过期让路。
  为什么不用 flock：共享根在一台上是本地盘、另一台经 NFS 挂上去，flock 在 NFS 上被转成字节范围锁，本地进程的 flock 与它是两种锁、互不冲突，
    所以 flock 在两台之间不保证互斥（scripts/gate/apply-writer-patch.py 文件头「副本锁」的跨机一段写的是同一件事）；这里只靠 link 的原子创建（NFS 上由服务端执行，不看客户端缓存）、
    记录里的机器标识与心跳，两台都起作用。
  留下的缝：都要另一台上的一趟整个进程停住超过 SCRATCH_LOCK_STALE_SECONDS 秒（心跳跟着停），被后来的一趟当成死了，之后又恢复：
    占着记录的那一趟停住：后来的一趟摘掉它的记录、放上自己的，它恢复后不核记录还是不是自己的、接着跑，两趟同时占着；摘它记录的那一趟核对过内容之后、删之前，
      它恰好恢复、放锁、第三趟又放了新记录，删掉的是第三趟的新记录，两趟同时占着。
    摘旧记录的那一趟停住：后来的一趟摘掉它的 .taking 接手摘下去；它恢复后删记录之前那一次核对认出被接手、不占；停在那一次核对与删之间的，恢复后删掉的是接手那一趟刚放的记录，
      删之后那一次核对认出被接手、不占，接手的那一趟照跑而盘上没有记录，第三趟来了照样占得到，两趟同时占着。
    摘取记录没有心跳（放成到放掉只有几次系统调用）：另一台上放了摘取记录的那一趟整个进程停住超过 SCRATCH_LOCK_STALE_SECONDS 秒，后来的一趟摘掉它的摘取记录、照摘 .taking 并放上自己的，
      它恢复后删掉的可能是接手那一趟刚放的 .taking，两趟同时当自己在摘旧记录；被删了 .taking 的那一趟删记录前后的核对认出被接手、不占，
      停在核对与删之间的同上一条。
    两趟同时占着草稿目录：草稿树互相删掉、盖掉（两趟真换还要各自占按仓的记录）。两趟同时占着按仓的记录：两趟真换都往结局文件写开始行，后写的整份盖掉先写的（「按仓的占用」说的那种重叠），
      剩下的只有基线核对（第 4 步：换上之前主工作区那几份要与第 1 步记的逐字节相同）拦着两趟改到同一份文件；改不同文件的两趟都换得上，结局文件里先写的开始行被盖掉、两趟往后写的行交错在一起。
    这几种不拦：都要整个进程停住超过 SCRATCH_LOCK_STALE_SECONDS 秒又恢复，后果写在这里。

按仓的占用（防同一个仓上两趟真换重叠；--dry-run、--read-outcome、--feed-only、--check-callers 不占，--seal-outcome 占）：
  结局文件按仓只有一份，两趟真换（各给一个草稿目录也一样）会在读结局文件与写开始行之间重叠：后写开始行的整份盖掉先写的，先写的那一趟换到一半被杀，没人知道几份文件停在半截。
  所以真换占完草稿目录之后、读结局文件之前，在结局文件所在的目录放一份占用记录 singlefs-hook-swap.lock（git-dir 下；仓根不是 git 工作树的根时是 <草稿目录>/hook-swap/），
  写完成或退回、或这一趟退出（含出错）才删。记录的放法、持有者还占不占着、摘旧记录、放锁、心跳都同「草稿目录的占用」，记录末尾多一段「草稿目录」写这一趟用的草稿目录（--seal-outcome 写 -）。
  占不到：在读结局文件之前拒绝，退 10，拒绝说明写那一趟的进程号、主机、用的草稿目录与开跑多久，出路是等它结束再交；一个字节不写。
  --seal-outcome 占同一份，占到之后再看开始行记的那一趟还在不在（判法同「结局文件」里下一趟起手的那一条），还在就退 9、不补写。

做法（一开跑先占草稿目录、真换再占这个仓，占不到退 10）：
  1. 读规格里每份文件此刻的内容、记下字节；在内存里依次施加，每处要恰好命中 count 次；再从改后的文本倒着施加（新旧对调、次序倒排），
     要回得到原文——回不到（新串在改后的文件里不止 count 处）就停：反向定点替换回滚不了的规格不收。
  2. 草稿树 <草稿目录>/hook-swap/tree：.claude/hooks/ 整份拷一份（不带 __pycache__），套上改后的文本；仓根与 .claude/ 下别的项一律符号链接到主工作区
     （钩子按自己所在的位置找仓根、登记表与 preflight，链接过去判的就是主工作区的那几份；喂法里写文件路径的目标不许落在这些链接上，见 <喂法.json>）。
     脚本规格的草稿树只放改后的那几份文件（按它们在仓里的相对路径），不链接仓里别的东西：脚本规格不在草稿里跑脚本，写进去的路径也就不会穿过链接落到主工作区。
     钩子规格有标了 neutral 的喂法时，草稿树拷好、套规格之前先拿换上之前那一版把它们逐条喂一遍（目标落在草稿树外的不喂）；有一条退出码对不上就不套规格，
     逐条点名这条喂法对换上之前那一版在草稿树里就不成立（退几、应当退几），说这是喂法失效、不是规格的毛病，退 8、主工作区一个字节不写，--dry-run 同样；
     最常见的因是喂法的目标落在草稿树里，而钩子按落点分仓内仓外、工作根内外。没有 neutral 喂法时这一步不做。
  3. 草稿里验：规格里的文件与 <钩子> 是 .sh 的跑 `bash -n`，再把里面喂给 python 的 heredoc（`python3 … <<'PY'`；起头那一行以反斜杠收尾的，正文从接上的最后一行的下一行起）逐段编译；.py 整份编译；
     逐条喂 <喂法>（file_path 落在草稿树外的那一条先点名、不喂；CLAUDE_PROJECT_DIR 设成草稿树、检出记录写草稿目录；草稿树不是 git 工作树，多机配置经 scripts/hosts/lib_repository_copy.py 的
     environment_for_commands_in_the_copy 显式交给 SINGLEFS_MULTI_HOST_CONFIG），退出码要等于写的那一个，
     stderr 里不许有「没判成」「读不到共用模块」（钩子坏了放行时说的话）。有一条不对就停，主工作区一个字节不写。--dry-run 停在这里。
     脚本规格只做语法检查（.sh 的 bash -n 与内嵌 python、.py 整份编译，别的后缀不查），不喂钩子。
     规格里有 .py 库的（钩子规格与脚本规格都算），另核调用方引到的名字：本仓里载入这些库的钩子、共用库、task-server/ 与 scripts/ 下的文件（.py 整份；.sh 里喂给 python 的 heredoc），
     凡是 <载入它的变量>.<名字>（含 from lib import 名字、getattr(库, "名字")）用到被换库的名字，换上之后那一版的模块层里都要有；缺了就当草稿验不过，点名「谁在哪一行引了哪个库的哪个名字」。
     怎么认「载入了哪个库、绑在哪个变量上」写在 CALLER_SCAN_DIRECTORIES 前的注释；认不出去向的写法逐处列成「本次未判」，不当成查过。只核不换用 --check-callers。
     按字符串键存取的容器（sys.modules 这类）按键分开认：取的键折得出、是这份库存进去或载入时用的名字，才算这份库；是另一份库载入时用的名字，归那另一份库
     （它也在这一次要核的库里时，名字照它核）；键折不出、或折出的名字哪份库都对不上，列成「本次未判」。本文件没按键存过的 sys.modules 按键取同样认：键折得出而对得上某份库的名字，
     取出的就是那份库，名字照核；折不出的列成「本次未判」。取法认 sys.modules[键] 与 sys.modules.get(键)，容器名照 import 的别名认（from sys import modules 也算）。
     只用 from <库> import 名字引库的文件（没有别的载入或绑定）照样核引到的名字。仓里叫这个库名的 .py 不止一份时，按调用方运行时真解到的那一份核（路径载入的写死路径、
     往 sys.path 最前放的目录、调用方自己所在的目录、往 sys.path 后面放的目录，依次取；调用方在模块层 import 进来的仓内模块往 sys.path 放的目录同样算），
     解到的不是被核的那一份就不核；认不出是哪一份的列成「本次未判」。文本里点名了库、载入与绑定都没认出的写法（不带 as 的带点 import、SourceFileLoader、runpy.run_path）也列成「本次未判」；
     判不到的写法（不读、不列）：库名拼出来的（importlib.import_module("fake_" + "target")，整份文本里没有库名）；往上两层以上的相对导入（from .. import 库）
     解到调用方自己所在目录里的同名文件；别的路径折不出的载入。
  4. 再读主工作区那几份，要与第 1 步记的逐字节相同（这段时间里别的会话改了就停，一个字节不写）；整批先预演一遍（同目录临时文件写完、fsync，
     改名之前停下），全过了逐份改名换上（scripts/kb/lib_atomic_replace.py），回读。
  5. 主工作区里照第 3 步再验一遍（CLAUDE_PROJECT_DIR 设成仓根，调用方引到的名字按盘上此刻的文件再核）；脚本规格再跑 --verify 的命令（输出写 <草稿目录>/hook-swap/verify-<时刻>.log）。
     不对就回滚：逐份在此刻的内容上做反向定点替换（每处要恰好命中 count 次），改名换上；再对回滚后的文件做语法检查、喂标了 neutral 的几条
     （脚本规格回滚后只做语法检查，不再跑 --verify 的命令）。
  草稿树跑完就删；喂法的输出与检出记录留在 <草稿目录>/hook-swap/ 下。

钩子在 bash 这一层坏了时，每一条 Bash 调用都会被它拒（退 2），连改回去的那一条也起不来：这条脚本在同一个进程里回滚，不靠下一条 Bash；
退出码 6 时只剩 Edit / Write 能改，提示行逐处写出要改回的地方，反向规格在草稿目录里。
脚本规格为什么要有：脚本规格目录下的脚本被别的会话当工具用（scripts/kb/replace-once.py 这几份）、被任务拷进快照去跑、或被直接起成服务（task-server/control 这类），
换到一半或换坏了，别的会话每一次调用、每一个拷走它的任务都受影响；先验后换、基线核对、整批预演、逐份改名换上、换上之后验证、反向定点回滚都落在这一个进程里，
中间态不在主工作区出现。

退出码：0 换上了、换上之后验过（--dry-run：草稿里验过；--check-callers：调用方引到的名字都在）；1 草稿里验不过（一个字节没写；--check-callers：有缺的名字）；2 参数、路径或规格不对（一个字节没写）；
3 草稿验过之后主工作区那几份被改了（一个字节没写）；4 换上时失败（已换上的几份回滚了）；
5 换上之后验不过（脚本规格含 --verify 的命令退非 0 或超时）、已按反向定点替换回滚、回滚后验过；6 回滚不成或回滚后还验不过（看提示行）；
7 上一趟换上没写完成（被杀或回滚不成，几份文件停在半截，见「结局文件」），拒绝再换，一个字节没写（--read-outcome 与 --seal-outcome 的退出码另见「结局文件」）；
8 标了 neutral 的喂法对换上之前那一版在草稿树里就不成立（喂法失效，不是规格的毛病；规格没套，一个字节没写，--dry-run 同样）；
9 上一趟换上还没完：起它的进程还在这一台上跑（还在换或在验），拒绝再换（--seal-outcome 同样拒补写），一个字节没写，等它写完成再来；
10 草稿目录正被另一趟 hook-swap.py 占着（--dry-run 同样；见「草稿目录的占用」），或这个仓的真换（含 --seal-outcome）正被另一趟占着（见「按仓的占用」），
  没建树、没写文件（主工作区一个字节没写），等那一趟结束再交；前一种也可以换一个 --scratch。

弄坏开关（只给自证判红用，生效时往 stderr 打一行；平常跑设了退 2、点名只在自证里认：--selftest 本身与它起的子进程认）。
代码分块：本文件留 main（参数解析与分派）、bootstrap 与 --selftest 的守卫；别的名字写在同目录的几份模块里，本文件按名字再导出，别处按路径载入本文件取名字，名字都取得到：
  hook_swap_base.py                      常量与弄坏开关集合、内置喂法、定点替换的解析与套用、钩子与脚本的静态检查、喂法的验证与只喂不换、草稿树的检查
  hook_swap_callers.py                   引到的名字：调用方的流分析（CallerFlow、SearchPaths）、调用方文件的收集、--check-callers
  hook_swap_swap.py                      草稿树的搭建与演练、回滚、结局文件、草稿目录与仓的占用记录、换上（swap）
  hook_swap_selftest_fixtures.py         自检的样本钩子、样本调用方与喂法
  hook_swap_selftest.py                  自检 selftest（main 的 --selftest 分支里才 import，模块级不 import）
下面「弄坏开关」的开关表仍写在本文件。读开关的代码在哪一份（按开关名带引号搜得到；自检里设开关的写法不在这里列）：
  hook_swap_base.py：swap-on-draft-red、skip-embedded-python、skip-base-check、skip-post-swap-check、rollback-whole-file、stale-builtin-feed、brace-feeds-dropped、feed-target-unchecked、script-verify-skipped、script-paths-unrestricted、script-gate-fixtures-accepted、script-server-target-accepted、script-server-src-accepted、skip-caller-names、subscript-key-ignored、registry-read-unseen、registry-get-ignored、registry-alias-ignored、library-by-stem-only、from-import-only-skipped、heredoc-continuation-ignored、imported-sys-path-ignored、environment-default-unfolded、imported-constants-unfolded、unrecognized-loads-unlisted、module-attribute-key-unfolded、function-local-names-ignored、several-choices-unfolded、known-tail-unfolded、vague-directories-ignored、outcome-start-omitted、outcome-complete-first、pending-not-checked、running-swap-not-checked、swap-process-pid-only、neutral-base-skipped、outcome-reader-trusts-file、scratch-lock-skipped、scratch-lock-keyed-by-parent、dead-holder-blocks、scratch-lock-kept-on-exit、scratch-lock-term-unhandled、scratch-lock-no-heartbeat、elsewhere-holder-ignored、elsewhere-holder-never-expires、scratch-record-overwrites、repository-lock-keyed-by-scratch、seal-outcome-lock-skipped、seal-outcome-running-unchecked、stale-record-renamed-away、link-reply-lost-refused、taking-claim-unjudged、taking-claim-kept-on-term、taking-reply-lost-refused、taking-holder-unnamed、taking-reap-unclaimed、taking-no-heartbeat、taken-over-unchecked
  hook_swap_callers.py：skip-caller-names、subscript-key-ignored、registry-read-unseen、registry-get-ignored、registry-alias-ignored、library-by-stem-only、from-import-only-skipped、imported-sys-path-ignored、environment-default-unfolded、imported-constants-unfolded、unrecognized-loads-unlisted、module-attribute-key-unfolded、function-local-names-ignored、several-choices-unfolded、known-tail-unfolded、vague-directories-ignored
  hook_swap_swap.py：swap-on-draft-red、skip-base-check、skip-post-swap-check、rollback-whole-file、script-verify-skipped、outcome-start-omitted、outcome-complete-first、pending-not-checked、running-swap-not-checked、swap-process-pid-only、outcome-reader-trusts-file、scratch-lock-skipped、scratch-lock-keyed-by-parent、dead-holder-blocks、scratch-lock-kept-on-exit、scratch-lock-term-unhandled、scratch-lock-no-heartbeat、elsewhere-holder-ignored、elsewhere-holder-never-expires、scratch-record-overwrites、repository-lock-keyed-by-scratch、seal-outcome-lock-skipped、seal-outcome-running-unchecked、stale-record-renamed-away、link-reply-lost-refused、taking-claim-unjudged、taking-claim-kept-on-term、taking-reply-lost-refused、taking-holder-unnamed、taking-reap-unclaimed、taking-no-heartbeat、taken-over-unchecked

一条一行「HOOK_SWAP_BREAK=<项>  弄坏哪一处」，下一行起写打开后自证红的项（失败行 `✗ hook-swap 自检：` 之后的句首）：
  HOOK_SWAP_BREAK=swap-on-draft-red  草稿里验不过也换上
      「草稿红不换」「花括号写法上放行的钩子，先验停下」「脚本规格：草稿红不换」「引到的名字：sys.modules 按键存取，换上的库自己的键取出的名字真缺了仍被点名，别的库的名字不点名」
      「引到的名字：库里去掉一个名字，各种载入写法的调用方都被点名，草稿验不过、一个字节不写」「neutral 的喂法对旧版成立、新版不对：照旧报草稿里验不过」「没有 neutral 喂法的规格照旧：旧版上就不成立的喂法照旧报草稿里验不过」
      「引到的名字：sys.modules 按库名取而本文件没存过的，库里缺的名字点名，草稿验不过、一个字节不写」「引到的名字：只用 from 库 import 名字引库的调用方，缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：sys.modules.get(键, 缺省) 取而本文件没存过的，库里缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：from sys import modules 之后 modules[键] 取而本文件没存过的，库里缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：同名的库不止一份时按调用方运行时解到的那一份核，只点名解到要核的那一份的调用方，草稿验不过、一个字节不写」
      「引到的名字：仓里只有一份的库，离它再远的调用方也照核，缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：起头那一行以反斜杠收尾的 heredoc 里的调用方，缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：调用方 import 的模块往 sys.path 最前放了被换的库所在的目录，解到被换的那一份，缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：调用方 import 的模块放的目录取自 os.environ.get(…) or 缺省，按缺省折，解到被换的那一份，缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：调用方放的目录取自 import 进来的模块的常量，解到被换的那一份，缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：sys.modules[X.__module__]，X 从被换的库 import 来，取出的算那份库，缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：调用方 import 的模块在函数里往 sys.path 放目录，名字只在那个函数里赋值，别处另有同名的，照函数里的赋值折，解到被换的那一份，缺的名字点名，草稿验不过、一个字节不写」
  HOOK_SWAP_BREAK=skip-embedded-python  不编译 .sh 里喂给 python 的 heredoc（喂法也红，但说明里点不出是哪一段）
      「草稿红不换」「内嵌 python 静态检查：起头那一行以反斜杠收尾的 heredoc，正文编译过、正文里的语法错点名」
  HOOK_SWAP_BREAK=skip-base-check  换上之前不核主工作区那几份在验的时候改没改
      「验的时候主工作区那份被改了」「脚本规格：验的时候主工作区那份被改了」
  HOOK_SWAP_BREAK=skip-post-swap-check  换上之后不再验
      「换上之后红回滚」「脚本规格：过」「脚本规格：验证红回滚」「结局文件：换上之后红回滚，写退回，--read-outcome 退 1」
  HOOK_SWAP_BREAK=rollback-whole-file  回滚时整份写回换上之前的内容（盖掉别的会话这段时间里的改动）
      「换上之后红回滚」「脚本规格：验证红回滚」
  HOOK_SWAP_BREAK=stale-builtin-feed  内置喂法里混进一条脚本搬家之前的旧路径
      「内置喂法「旧路径」里的 scripts/mutate.s?」
  HOOK_SWAP_BREAK=brace-feeds-dropped  内置喂法里不带花括号与参数通配那六条
      「内置喂法带花括号与参数通配的六条」「花括号写法上放行的钩子，先验停下」
  HOOK_SWAP_BREAK=feed-target-unchecked  喂之前不核喂法的 file_path 解开符号链接之后在不在草稿树里（落在符号链接上的照喂，按落点判的钩子当仓外放行、喂法空过）
      「喂法目标落在符号链接上：草稿红、点名那一条与出路、不喂」「只喂不换：喂法目标落在仓根之外：点名那一条与出路」
  HOOK_SWAP_BREAK=script-verify-skipped  脚本规格换上之后不跑 --verify 的命令（输出里没有「验证命令退 0」）
      「脚本规格：过」「脚本规格：验证红回滚」
  HOOK_SWAP_BREAK=script-paths-unrestricted  规格里的文件不限在 .claude/hooks/ 与脚本规格目录下（仓里已有的普通文件都收）
      「脚本规格：脚本规格目录之外的文件不收」「脚本规格：.claude/gate.d/fixtures/ 下的样本不收」「脚本规格：prune-server/target/ 下的文件不收」
      「脚本规格：gpu-verification-server/target/ 下的文件不收」「脚本规格：prune-server/src/ 下的文件不收」「脚本规格：gpu-verification-server/src/ 下的文件不收」
  HOOK_SWAP_BREAK=script-gate-fixtures-accepted  脚本规格收 .claude/gate.d/fixtures/ 下的样本
      「脚本规格：.claude/gate.d/fixtures/ 下的样本不收」
  HOOK_SWAP_BREAK=script-server-target-accepted  脚本规格收 prune-server/、gpu-verification-server/ 下 target/ 里的文件
      「脚本规格：prune-server/target/ 下的文件不收」「脚本规格：gpu-verification-server/target/ 下的文件不收」
  HOOK_SWAP_BREAK=script-server-src-accepted  脚本规格收 prune-server/、gpu-verification-server/ 下 src/ 里的文件
      「脚本规格：prune-server/src/ 下的文件不收」「脚本规格：gpu-verification-server/src/ 下的文件不收」
  HOOK_SWAP_BREAK=skip-caller-names  调用方引到的名字缺了也不报（换上的库去掉了调用方还在引的名字，照样验过、换上）
      「引到的名字：库里去掉一个名字，各种载入写法的调用方都被点名，草稿验不过、一个字节不写」
      「引到的名字：sys.modules 按键存取，换上的库自己的键取出的名字真缺了仍被点名，别的库的名字不点名」「引到的名字：sys.modules 按键存取，另一份库也在要核的库里时，它的键取出的名字照它核、缺了点名它」
      「引到的名字：sys.modules 按库名取而本文件没存过的，库里缺的名字点名，草稿验不过、一个字节不写」「引到的名字：只用 from 库 import 名字引库的调用方，缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：sys.modules.get(键, 缺省) 取而本文件没存过的，库里缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：from sys import modules 之后 modules[键] 取而本文件没存过的，库里缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：同名的库不止一份时按调用方运行时解到的那一份核，只点名解到要核的那一份的调用方，草稿验不过、一个字节不写」
      「引到的名字：仓里只有一份的库，离它再远的调用方也照核，缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：起头那一行以反斜杠收尾的 heredoc 里的调用方，缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：调用方 import 的模块往 sys.path 最前放了被换的库所在的目录，解到被换的那一份，缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：调用方 import 的模块放的目录取自 os.environ.get(…) or 缺省，按缺省折，解到被换的那一份，缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：调用方放的目录取自 import 进来的模块的常量，解到被换的那一份，缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：sys.modules[X.__module__]，X 从被换的库 import 来，取出的算那份库，缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：调用方 import 的模块在函数里往 sys.path 放目录，名字只在那个函数里赋值，别处另有同名的，照函数里的赋值折，解到被换的那一份，缺的名字点名，草稿验不过、一个字节不写」
  HOOK_SWAP_BREAK=subscript-key-ignored  按键取容器里的库时不看键（装过库的 sys.modules 这类容器，按什么键取出来都当先存进去的那份库）
      「引到的名字：sys.modules 按键存取，另一份库的键取出的名字不算到换上的库头上，照换、键折不出的列成本次未判」
      「引到的名字：sys.modules 按键存取，换上的库自己的键取出的名字真缺了仍被点名，别的库的名字不点名」「引到的名字：sys.modules 按键存取，另一份库也在要核的库里时，它的键取出的名字照它核、缺了点名它」
  HOOK_SWAP_BREAK=registry-read-unseen  本文件没按键存过的 sys.modules 按键取时一概不认（按库名取出的库不核、折不出的键不列本次未判，也不当作文件提到了目标库）
      「引到的名字：sys.modules 按库名取而本文件没存过的，库里缺的名字点名，草稿验不过、一个字节不写」「引到的名字：sys.modules 按折不出的键取而本文件没存过的，列成本次未判、仍照常换上」
      「引到的名字：sys.modules.get(键, 缺省) 取而本文件没存过的，库里缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：from sys import modules 之后 modules[键] 取而本文件没存过的，库里缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：sys.modules.get(折不出的键) 取而本文件没存过的，列成本次未判、仍照常换上」
      「引到的名字：sys.modules[X.__module__]，X 从被换的库 import 来，取出的算那份库，缺的名字点名，草稿验不过、一个字节不写」
  HOOK_SWAP_BREAK=registry-get-ignored  sys.modules 这类容器的 .get(键) 取法不认（只认下标）
      「引到的名字：sys.modules.get(键, 缺省) 取而本文件没存过的，库里缺的名字点名，草稿验不过、一个字节不写」「引到的名字：sys.modules.get(折不出的键) 取而本文件没存过的，列成本次未判、仍照常换上」
  HOOK_SWAP_BREAK=registry-alias-ignored  容器名不照 import 的别名还原（from sys import modules 之后的 modules[键] 认不出是 sys.modules）
      「引到的名字：from sys import modules 之后 modules[键] 取而本文件没存过的，库里缺的名字点名，草稿验不过、一个字节不写」
  HOOK_SWAP_BREAK=library-by-stem-only  库只按文件名认（仓里叫这个名字的几份 .py 都当成要核的那一份，不看调用方运行时解到哪一份）
      「引到的名字：自己目录里有同名的库的调用方，解到那一份、不核，汇总里写缺 0 处、照常换上」
      「引到的名字：同名的库不止一份时按调用方运行时解到的那一份核，只点名解到要核的那一份的调用方，草稿验不过、一个字节不写」
      「引到的名字：同名的库不止一份而认不出解到哪一份的调用方，列成本次未判、仍照常换上」
      「引到的名字：调用方 import 的模块往 sys.path 放的目录折不出，列成本次未判、仍照常换上」
      「引到的名字：调用方 import 的模块放的目录前半截折不出、末尾几段与同名库所在的目录对不上，不挡同目录里的同名库，解到那一份、不核，照常换上」
      「引到的名字：调用方 import 的模块放的目录前半截折不出、末尾几段正是被换的库所在的目录，列成本次未判、仍照常换上」
      「引到的名字：调用方 import 的模块放的目录取自函数的几处返回，都不是同名库所在的目录，不挡同目录里的同名库，解到那一份、不核，照常换上」
      「引到的名字：调用方 import 的模块放的目录取自函数的几处返回，其中一种是被换的库所在的目录，列成本次未判、仍照常换上」
  HOOK_SWAP_BREAK=from-import-only-skipped  没有别的载入或绑定的文件里，from <目标库> import 名字不当作文件提到了目标库（整份不核）
      「引到的名字：只用 from 库 import 名字引库的调用方，缺的名字点名，草稿验不过、一个字节不写」
  HOOK_SWAP_BREAK=heredoc-continuation-ignored  起头那一行以反斜杠收尾的 heredoc，续行也当 python 正文（续行里的 || fail … 被当成 python）
      「内嵌 python 切段：起头那一行以反斜杠收尾，正文从续行结束的下一行起」「内嵌 python 静态检查：起头那一行以反斜杠收尾的 heredoc，正文编译过、正文里的语法错点名」
      「引到的名字：起头那一行以反斜杠收尾的 heredoc 里的调用方，缺的名字点名，草稿验不过、一个字节不写」
  HOOK_SWAP_BREAK=imported-sys-path-ignored  调用方在模块层 import 进来的仓内模块往 sys.path 放的目录不算（只认调用方自己写的）
      「引到的名字：调用方 import 的模块往 sys.path 最前放了被换的库所在的目录，解到被换的那一份，缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：调用方 import 的模块往 sys.path 放的目录折不出，列成本次未判、仍照常换上」
      「引到的名字：调用方 import 的模块放的目录取自 os.environ.get(…) or 缺省，按缺省折，解到被换的那一份，缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：调用方 import 的模块在函数里往 sys.path 放目录，名字只在那个函数里赋值，别处另有同名的，照函数里的赋值折，解到被换的那一份，缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：调用方 import 的模块放的目录前半截折不出、末尾几段正是被换的库所在的目录，列成本次未判、仍照常换上」
      「引到的名字：调用方 import 的模块放的目录取自函数的几处返回，其中一种是被换的库所在的目录，列成本次未判、仍照常换上」
  HOOK_SWAP_BREAK=environment-default-unfolded  <读环境变量> or <缺省> 的路径折不出（环境变量顶替的那一边不按缺省折）
      「引到的名字：调用方 import 的模块放的目录取自 os.environ.get(…) or 缺省，按缺省折，解到被换的那一份，缺的名字点名，草稿验不过、一个字节不写」
  HOOK_SWAP_BREAK=imported-constants-unfolded  X.NAME（X 是 import 进来的仓内模块）折不出，不按那份模块里 NAME 的赋值折
      「引到的名字：调用方放的目录取自 import 进来的模块的常量，解到被换的那一份，缺的名字点名，草稿验不过、一个字节不写」
  HOOK_SWAP_BREAK=unrecognized-loads-unlisted  文本里点名了库而载入与绑定都没认出的写法（不带 as 的带点 import、SourceFileLoader、runpy）不列本次未判
      「引到的名字：文本里点名了库而载入与绑定都没认出的写法（不带 as 的带点 import、SourceFileLoader、runpy.run_path），列成本次未判、仍照常换上」
  HOOK_SWAP_BREAK=module-attribute-key-unfolded  X.__module__ 当 sys.modules 的键时折不出（X 从目标库 import 来也不折成那份库）
      「引到的名字：sys.modules[X.__module__]，X 从被换的库 import 来，取出的算那份库，缺的名字点名，草稿验不过、一个字节不写」
  HOOK_SWAP_BREAK=function-local-names-ignored  往 sys.path 放的目录里的名字只在模块层找（函数里赋的名字折不出）
      「引到的名字：调用方 import 的模块在函数里往 sys.path 放目录，名字只在那个函数里赋值，别处另有同名的，照函数里的赋值折，解到被换的那一份，缺的名字点名，草稿验不过、一个字节不写」
      「引到的名字：调用方 import 的模块放的目录前半截折不出、末尾几段与同名库所在的目录对不上，不挡同目录里的同名库，解到那一份、不核，照常换上」
      「引到的名字：调用方 import 的模块放的目录前半截折不出、末尾几段正是被换的库所在的目录，列成本次未判、仍照常换上」
      「引到的名字：调用方 import 的模块放的目录取自函数的几处返回，都不是同名库所在的目录，不挡同目录里的同名库，解到那一份、不核，照常换上」
      「引到的名字：调用方 import 的模块放的目录取自函数的几处返回，其中一种是被换的库所在的目录，列成本次未判、仍照常换上」
  HOOK_SWAP_BREAK=several-choices-unfolded  一个名字在函数里赋了几处、本文件里不带参数的函数有几处返回时，折不出（只认赋一处的名字）
      「引到的名字：调用方 import 的模块放的目录取自函数的几处返回，都不是同名库所在的目录，不挡同目录里的同名库，解到那一份、不核，照常换上」
      「引到的名字：调用方 import 的模块放的目录取自函数的几处返回，其中一种是被换的库所在的目录，列成本次未判、仍照常换上」
  HOOK_SWAP_BREAK=known-tail-unfolded  前半截折不出的目录不再折成「只知道末尾几段」（一概折不出）
      「引到的名字：调用方 import 的模块放的目录前半截折不出、末尾几段与同名库所在的目录对不上，不挡同目录里的同名库，解到那一份、不核，照常换上」
      「引到的名字：调用方 import 的模块放的目录前半截折不出、末尾几段正是被换的库所在的目录，列成本次未判、仍照常换上」
      「引到的名字：调用方 import 的模块放的目录取自函数的几处返回，都不是同名库所在的目录，不挡同目录里的同名库，解到那一份、不核，照常换上」
      「引到的名字：调用方 import 的模块放的目录取自函数的几处返回，其中一种是被换的库所在的目录，列成本次未判、仍照常换上」
  HOOK_SWAP_BREAK=vague-directories-ignored  半折的目录（几种可能、只知道末尾几段）里有同名库也不挡解析（当成没放过）
      「引到的名字：调用方 import 的模块放的目录前半截折不出、末尾几段正是被换的库所在的目录，列成本次未判、仍照常换上」
      「引到的名字：调用方 import 的模块放的目录取自函数的几处返回，其中一种是被换的库所在的目录，列成本次未判、仍照常换上」
  HOOK_SWAP_BREAK=outcome-start-omitted  换上之前不写开始与文件行，拖到回读之后才一起写（被杀的那一趟什么都没留下）
      「结局文件：被杀在两份之间，只有开始与文件行，--read-outcome 退 3、点名已换上与没换上」「结局文件：被杀在两份之间之后，下一趟拒（退 7），点名反向规格，一个字节不写」
  HOOK_SWAP_BREAK=outcome-complete-first  开始行写完马上写完成行（没换完也像换完了）
      「结局文件：换完的一趟依次写开始、文件、回读、完成，--read-outcome 退 0」「结局文件：被杀在两份之间，只有开始与文件行，--read-outcome 退 3、点名已换上与没换上」
      「结局文件：被杀在两份之间之后，下一趟拒（退 7），点名反向规格，一个字节不写」「结局文件：全换上了还没写完成就被杀，下一趟拒（退 7）、--seal-outcome 补写完成之后照换」
      「结局文件：上一趟的进程还在，下一趟拒（退 9），只叫等它写完成，不给退回与补写的出路，一个字节不写」「结局文件：上一趟的进程不在了，下一趟拒（退 7），说明同被杀的那一趟」
      「结局文件：上一趟的进程号被别的进程占了（启动刻对不上），下一趟拒（退 7）」「结局文件：上一趟在另一台上起的，下一趟拒（退 7），先叫看那一趟还在不在跑」
      「补写完成：开始行那一趟还在（没占着记录），--seal-outcome 退 9、不补写」
  HOOK_SWAP_BREAK=pending-not-checked  下一趟起手不读上一趟的结局文件（半截照换）
      「结局文件：被杀在两份之间之后，下一趟拒（退 7），点名反向规格，一个字节不写」「结局文件：全换上了还没写完成就被杀，下一趟拒（退 7）、--seal-outcome 补写完成之后照换」
      「结局文件：上一趟的进程还在，下一趟拒（退 9），只叫等它写完成，不给退回与补写的出路，一个字节不写」「结局文件：上一趟的进程不在了，下一趟拒（退 7），说明同被杀的那一趟」
      「结局文件：上一趟的进程号被别的进程占了（启动刻对不上），下一趟拒（退 7）」「结局文件：上一趟在另一台上起的，下一趟拒（退 7），先叫看那一趟还在不在跑」
  HOOK_SWAP_BREAK=running-swap-not-checked  拒之前不看上一趟的进程还在不在（还在验的那一趟也照「被杀或回滚不成」拒、给退回与补写的出路）
      「结局文件：上一趟的进程还在，下一趟拒（退 9），只叫等它写完成，不给退回与补写的出路，一个字节不写」
  HOOK_SWAP_BREAK=swap-process-pid-only  判上一趟的进程还在只看 /proc/<进程号> 在不在，不核机器、开机与启动刻
      「结局文件：上一趟的进程号被别的进程占了（启动刻对不上），下一趟拒（退 7）」「结局文件：上一趟在另一台上起的，下一趟拒（退 7），先叫看那一趟还在不在跑」
  HOOK_SWAP_BREAK=neutral-base-skipped  套规格之前不拿换上之前那一版喂标了 neutral 的喂法（喂法失效照样往下套规格，当成规格的毛病报「草稿里验不过」退 1）
      「喂法失效：标了 neutral 的喂法对换上之前那一版在草稿树里就不成立，退 8、点名那一条、不套规格、一个字节不写」「喂法失效：--dry-run 同样退 8、点名那一条」
  HOOK_SWAP_BREAK=outcome-reader-trusts-file  判每份文件的状态时信结局文件里写的换后，不对着主工作区现算 sha256
      「结局文件：被杀在两份之间，只有开始与文件行，--read-outcome 退 3、点名已换上与没换上」「结局文件：完成之后文件又被改，--read-outcome 退 4 并点名」
      「结局文件：被杀之后手工退回到换前，下一趟照换」「结局文件：换上之后红回滚，写退回，--read-outcome 退 1」
  HOOK_SWAP_BREAK=scratch-lock-skipped  一开跑不占草稿目录（不放占用记录，同一个 --scratch 上并发的几趟照旧互相删树）
      「草稿目录的占用：同一个 --scratch 上两趟并发，后一趟退 10、没有回溯、没碰前一趟的草稿树，前一趟照常」「草稿目录的占用：持有者被杀（留下记录）之后，后来的一趟不被挡」
      「草稿目录的占用：这一台起的、进程还在的记录挡住后来的一趟，说明写进程号与主机名，不动那份记录、不建树」「草稿目录的占用：另一台起的、刚刷新过的记录（含读不全的）挡住后来的一趟」
      「草稿目录的占用：持有期间每隔心跳间隔刷新记录的修改时间」「草稿目录的占用：放不了记录时退 2 并给出路，不抛回溯」
      「草稿目录的占用：三趟同时摘同一份旧记录，只有一趟占着，占着那一趟的记录还在盘上，没留下临时文件」
      「草稿目录的占用：摘旧记录的那一趟被 KILL（留下 .taking）之后，按仓的真换、--seal-outcome 与草稿目录的占用都不被挡，.taking 当场摘掉」（草稿目录那一处）
      「草稿目录的占用：.taking 在而摘的那一趟还占着，挡住后来的一趟并被点名；另一台起的、过了期的不挡」
  HOOK_SWAP_BREAK=scratch-lock-keyed-by-parent  占用记录放在草稿目录的上一层（同一层下的几个不同草稿目录互相挡）
      「草稿目录的占用：不同的 --scratch 上两趟并发，都照常」「草稿目录的占用：放不了记录时退 2 并给出路，不抛回溯」
      「按仓的占用：两个草稿目录的真换，后一趟在读结局文件之前退 10、点名前一趟的草稿目录、没写任何东西，前一趟照常换完；另一个草稿目录的 --dry-run 照常」
  HOOK_SWAP_BREAK=dead-holder-blocks  这一台起的、进程已经没了的持有者，记录还在就照旧挡（不看进程在不在）
      「草稿目录的占用：持有者被杀（留下记录）之后，后来的一趟不被挡」「草稿目录的占用：记录里的进程号被别的进程占了（启动刻对不上），不挡后来的一趟」
      「草稿目录的占用：三趟同时摘同一份旧记录，只有一趟占着，占着那一趟的记录还在盘上，没留下临时文件」
      「草稿目录的占用：摘旧记录的那一趟被 KILL（留下 .taking）之后，按仓的真换、--seal-outcome 与草稿目录的占用都不被挡，.taking 当场摘掉」
      「草稿目录的占用：摘旧记录的当中被 TERM 杀掉（没占着草稿目录与已占着各一趟），放掉 .taking 与草稿目录的记录，退出状态仍是被 TERM 杀的」
      「草稿目录的占用：摘旧记录那一次建 .taking 的 link 在服务端做成而回复丢了（报已有、读回来是自己写的），当放成，摘掉旧记录，不留 .taking」
      「草稿目录的占用：.taking 在而摘的那一趟还占着，挡住后来的一趟并被点名；另一台起的、过了期的不挡」
      「草稿目录的占用：两趟同时摘同一份已死的 .taking（草稿目录的记录），只有一趟删得了 .taking、占到，它的记录在盘上，不留 .taking 与摘取记录」
      「草稿目录的占用：摘的那一趟放着 .taking 期间刷新它的修改时间，另一台上还活着的摘的那一趟放了超过过期秒数也照挡后来的一趟」
      「草稿目录的占用：摘的那一趟停住、.taking 被当成已死接手之后，它恢复了不占、拒绝说明写被接手，只有接手的一趟占到」
  HOOK_SWAP_BREAK=scratch-lock-kept-on-exit  退出时不删占用记录
      「按仓的占用：两个草稿目录的真换，后一趟在读结局文件之前退 10、点名前一趟的草稿目录、没写任何东西，前一趟照常换完；另一个草稿目录的 --dry-run 照常」（后一趟的草稿目录里留着没放掉的记录）
      「草稿目录的占用：退出时放锁（正常、规格不对、抛异常、Ctrl-C 各一趟），记录与临时文件都不留」「草稿目录的占用：被 TERM 杀掉时先放锁，退出状态仍是被 TERM 杀的」
      「草稿目录的占用：摘旧记录的当中被 TERM 杀掉（没占着草稿目录与已占着各一趟），放掉 .taking 与草稿目录的记录，退出状态仍是被 TERM 杀的」（已占着草稿目录那一趟，它的记录没放掉）
      另有结局文件那七格：同一个草稿目录上接着起下一趟，被没放掉的记录挡成退 10——
      「结局文件：被杀在两份之间之后，下一趟拒（退 7），点名反向规格，一个字节不写」「结局文件：被杀之后手工退回到换前，下一趟照换」
      「结局文件：全换上了还没写完成就被杀，下一趟拒（退 7）、--seal-outcome 补写完成之后照换」「结局文件：上一趟的进程还在，下一趟拒（退 9），只叫等它写完成，不给退回与补写的出路，一个字节不写」
      「结局文件：上一趟的进程不在了，下一趟拒（退 7），说明同被杀的那一趟」「结局文件：上一趟的进程号被别的进程占了（启动刻对不上），下一趟拒（退 7）」
      「结局文件：上一趟在另一台上起的，下一趟拒（退 7），先叫看那一趟还在不在跑」
  HOOK_SWAP_BREAK=scratch-lock-term-unhandled  不装 TERM 与 HUP 的处理（被 TERM 杀掉的一趟来不及删记录）
      「草稿目录的占用：被 TERM 杀掉时先放锁，退出状态仍是被 TERM 杀的」
      「草稿目录的占用：摘旧记录的当中被 TERM 杀掉（没占着草稿目录与已占着各一趟），放掉 .taking 与草稿目录的记录，退出状态仍是被 TERM 杀的」
  HOOK_SWAP_BREAK=scratch-lock-no-heartbeat  持有期间不刷新占用记录的修改时间
      「草稿目录的占用：持有期间每隔心跳间隔刷新记录的修改时间」
  HOOK_SWAP_BREAK=elsewhere-holder-ignored  另一台起的（或读不全的）记录一律当持有者已放
      「草稿目录的占用：另一台起的、刚刷新过的记录（含读不全的）挡住后来的一趟」
      「草稿目录的占用：.taking 在而摘的那一趟还占着，挡住后来的一趟并被点名；另一台起的、过了期的不挡」（另一台起的、刚放下那一处）
      「草稿目录的占用：摘的那一趟放着 .taking 期间刷新它的修改时间，另一台上还活着的摘的那一趟放了超过过期秒数也照挡后来的一趟」
  HOOK_SWAP_BREAK=elsewhere-holder-never-expires  另一台起的（或读不全的）记录一律当还占着，不看多久没刷新
      「草稿目录的占用：另一台起的、过了期还没刷新的记录（含读不全的）不挡后来的一趟」
      「草稿目录的占用：.taking 在而摘的那一趟还占着，挡住后来的一趟并被点名；另一台起的、过了期的不挡」（另一台起的、过了期那一处）
      「草稿目录的占用：摘的那一趟停住、.taking 被当成已死接手之后，它恢复了不占、拒绝说明写被接手，只有接手的一趟占到」
  HOOK_SWAP_BREAK=scratch-record-overwrites  放记录时改名盖上去，不用硬链接排他创建（几趟同时放都当自己放成）
      「草稿目录的占用：几趟同时放记录只有一趟放成，放成的内容写全」「草稿目录的占用：link 在服务端做成而回复丢了（报已有、读回来是自己写的记录），当放成；报已有而读回来是别人的记录，不算放成」
  HOOK_SWAP_BREAK=repository-lock-keyed-by-scratch  按仓的占用记录放在这一趟草稿目录的 hook-swap/ 里（不同草稿目录的真换互相不挡，--seal-outcome 仍占结局文件目录里的那一份）
      「按仓的占用：两个草稿目录的真换，后一趟在读结局文件之前退 10、点名前一趟的草稿目录、没写任何东西，前一趟照常换完；另一个草稿目录的 --dry-run 照常」
      「补写完成：另一趟真换正在换上时，--seal-outcome 占不到按仓的占用，退 10、不补写」
  HOOK_SWAP_BREAK=seal-outcome-lock-skipped  --seal-outcome 不占按仓的占用记录（只剩看开始行那一趟还在不在）
      「补写完成：另一趟真换正在换上时，--seal-outcome 占不到按仓的占用，退 10、不补写」
  HOOK_SWAP_BREAK=seal-outcome-running-unchecked  --seal-outcome 占到之后不看开始行那一趟还在不在
      「补写完成：开始行那一趟还在（没占着记录），--seal-outcome 退 9、不补写」
  HOOK_SWAP_BREAK=stale-record-renamed-away  摘旧记录时把记录改名挪开、核内容、不是刚才判过的就放回去，放不回去（别的趟已放了新的）就删掉摘到的那一份
      「草稿目录的占用：三趟同时摘同一份旧记录，只有一趟占着，占着那一趟的记录还在盘上，没留下临时文件」
      「草稿目录的占用：摘旧记录的那一趟被 KILL（留下 .taking）之后，按仓的真换、--seal-outcome 与草稿目录的占用都不被挡，.taking 当场摘掉」
      「草稿目录的占用：摘旧记录的当中被 TERM 杀掉（没占着草稿目录与已占着各一趟），放掉 .taking 与草稿目录的记录，退出状态仍是被 TERM 杀的」
      「草稿目录的占用：摘旧记录那一次建 .taking 的 link 在服务端做成而回复丢了（报已有、读回来是自己写的），当放成，摘掉旧记录，不留 .taking」
      「草稿目录的占用：.taking 在而摘的那一趟还占着，挡住后来的一趟并被点名；另一台起的、过了期的不挡」（这一台起的与另一台起的刚放下两处）
      「草稿目录的占用：两趟同时摘同一份已死的 .taking（草稿目录的记录），只有一趟删得了 .taking、占到，它的记录在盘上，不留 .taking 与摘取记录」
      「草稿目录的占用：摘的那一趟放着 .taking 期间刷新它的修改时间，另一台上还活着的摘的那一趟放了超过过期秒数也照挡后来的一趟」
      「草稿目录的占用：摘的那一趟停住、.taking 被当成已死接手之后，它恢复了不占、拒绝说明写被接手，只有接手的一趟占到」
  HOOK_SWAP_BREAK=link-reply-lost-refused  link 报已有就一概算被占，不读回来认自己的记录
      「草稿目录的占用：link 在服务端做成而回复丢了（报已有、读回来是自己写的记录），当放成；报已有而读回来是别人的记录，不算放成」
  HOOK_SWAP_BREAK=taking-claim-unjudged  摘旧记录时 .taking 在就一概当别的趟还占着，不判写 .taking 的那一趟还占不占着（被 KILL 的摘的那一趟留下的 .taking 永远挡后来的）
      「草稿目录的占用：摘旧记录的那一趟被 KILL（留下 .taking）之后，按仓的真换、--seal-outcome 与草稿目录的占用都不被挡，.taking 当场摘掉」
      「草稿目录的占用：.taking 在而摘的那一趟还占着，挡住后来的一趟并被点名；另一台起的、过了期的不挡」（另一台起的、过了期那一处）
      「草稿目录的占用：两趟同时摘同一份已死的 .taking（草稿目录的记录），只有一趟删得了 .taking、占到，它的记录在盘上，不留 .taking 与摘取记录」
      「按仓的占用：两趟同时摘同一份已死的 .taking（按仓的记录），只有一趟删得了 .taking、占到，它的记录在盘上，不留 .taking 与摘取记录」
      「草稿目录的占用：摘的那一趟停住、.taking 被当成已死接手之后，它恢复了不占、拒绝说明写被接手，只有接手的一趟占到」
  HOOK_SWAP_BREAK=taking-claim-kept-on-term  TERM 与 HUP 的处理不放这一趟放着的 .taking（被 TERM 杀掉的摘的那一趟留下 .taking）
      「草稿目录的占用：摘旧记录的当中被 TERM 杀掉（没占着草稿目录与已占着各一趟），放掉 .taking 与草稿目录的记录，退出状态仍是被 TERM 杀的」
  HOOK_SWAP_BREAK=taking-reply-lost-refused  建 .taking 的 link 报已有就一概当别的趟正在摘，不读回来认自己写的（回复丢了的那一趟自己留下 .taking）
      「草稿目录的占用：摘旧记录那一次建 .taking 的 link 在服务端做成而回复丢了（报已有、读回来是自己写的），当放成，摘掉旧记录，不留 .taking」
  HOOK_SWAP_BREAK=taking-holder-unnamed  别的趟正在摘时拒绝说明不点名写 .taking 的那一趟（只剩「读不全」那一句）
      「草稿目录的占用：.taking 在而摘的那一趟还占着，挡住后来的一趟并被点名；另一台起的、过了期的不挡」（这一台起的与另一台起的刚放下两处）
  HOOK_SWAP_BREAK=taking-reap-unclaimed  摘已死的 .taking 不放摘取记录，读到还是判过的那一份就删（几趟同时摘都删得下手，后删的删掉先删那一趟刚放的 .taking）
      「草稿目录的占用：两趟同时摘同一份已死的 .taking（草稿目录的记录），只有一趟删得了 .taking、占到，它的记录在盘上，不留 .taking 与摘取记录」
      「按仓的占用：两趟同时摘同一份已死的 .taking（按仓的记录），只有一趟删得了 .taking、占到，它的记录在盘上，不留 .taking 与摘取记录」
  HOOK_SWAP_BREAK=taking-no-heartbeat  摘的那一趟放着 .taking 的期间不刷新它的修改时间（另一台上还活着的摘的那一趟放了超过过期秒数就被后来的当成死了）
      「草稿目录的占用：摘的那一趟放着 .taking 期间刷新它的修改时间，另一台上还活着的摘的那一趟放了超过过期秒数也照挡后来的一趟」
  HOOK_SWAP_BREAK=taken-over-unchecked  删记录之前与之后不核 .taking 还是不是这一趟放的（停住被接手的那一趟恢复后照占）
      「草稿目录的占用：摘的那一趟停住、.taking 被当成已死接手之后，它恢复了不占、拒绝说明写被接手，只有接手的一趟占到」
不认识的值直接拒绝，免得拼错的开关被当成「没开」。
"""

import ast
import hashlib
import json
import os
import re
import secrets
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import threading
import time
from typing import NamedTuple

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path[:0] = [os.path.join(REPOSITORY_ROOT, ".claude", "scripts"), SCRIPT_DIRECTORY]
from project_preflight import preflight  # noqa: E402
sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "kb"))
from lib_atomic_replace import ReplaceRefused, replace_file_contents_by_rename  # noqa: E402
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "hosts"))
import lib_repository_copy as repository_copy  # noqa: E402

sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
from hook_swap_base import (
     SCRIPT_DIRECTORY, REPOSITORY_ROOT, BREAK_VARIABLE, KNOWN_BREAK_MODES, REPOSITORY_LOCK_MODES, HOOKS_DIRECTORY, SCRIPT_SPEC_DIRECTORIES,
     SCRIPT_SPEC_EXCLUDED, VERIFY_TIMEOUT_SECONDS, WORK_DIRECTORY_NAME, FEED_TIMEOUT_SECONDS, FEED_EXIT_CODES, BROKEN_HOOK_MARKERS,
     EMBEDDED_PYTHON_START, EXIT_SWAPPED, EXIT_DRAFT_RED, EXIT_BAD_INPUT, EXIT_CHANGED, EXIT_SWAP_FAILED, EXIT_ROLLED_BACK, EXIT_ROLLBACK_FAILED,
     EXIT_PENDING_OUTCOME, EXIT_NEUTRAL_FEED_STALE, EXIT_PREVIOUS_SWAP_RUNNING, EXIT_SCRATCH_BUSY, OUTCOME_NAME, START_ROW_COLUMNS, OUTCOME_COMMENTS,
     WRITER, BRACE_AND_PARAMETER_GLOB_FEEDS, BUILTIN_FEEDS, builtin_feeds_for, stale_feed_paths, BadInput, Edit, RehearsalStoppedBeforeRename,
     break_mode, is_inside, read_bytes, read_text, sha256_of, script_spec_problem, resolve_edits, is_script_edit, apply_edits, continues_on_next_line,
     embedded_python_segments, static_problems, substitute, hook_input_of, validate_feeds, feed_target_problem, feed_environment, feed_only,
     feed_problems, neutral_feeds_failing_on_base, neutral_feed_stale_lines, check_tree, CALLER_SCAN_DIRECTORIES, CALLER_SKIPPED_DIRECTORIES,
     LOADER_CALLEE, HARMLESS_CALLEES, LOADER_BODY_CALLEES, CONTAINER_METHODS, MODULE_REGISTRIES, DYNAMIC_NAME_CALLEES, FLOW_ROUNDS, CROSS_FILE_ROUNDS,
     PATH_FOLD_DEPTH, PATH_PASS_THROUGH_CALLEES, PATH_JOIN_CALLEES, PATH_IDENTITY_METHODS, ENVIRONMENT_READ_CALLEES, UNRECOGNIZED_LOADER_CALLEES,
)  # noqa: E402,F401
from hook_swap_callers import (
     callee_name, dotted_path, annotate_parents, enclosing_scope, import_aliases, canonical_path, assignment_scan, target_names, module_level_names,
     CallerFlow, module_level_imports, SearchPaths, CallerUnit, ModuleSources, CallerNames, caller_source_files, caller_units, caller_names_report,
     caller_names_check, unjudged_names_lines, check_callers,
)  # noqa: E402,F401
from hook_swap_swap import (
     build_scripts_draft_tree, verify_problem, build_draft_tree, write_planned_texts, rehearse, manual_rollback_lines, rollback, OutcomeUnreadable,
     Outcome, UNKNOWN_IDENTITY, MACHINE_ID_PATH, BOOT_ID_PATH, PROC_STAT_STATE_FIELD_AFTER_COMM, PROC_STAT_START_FIELD_AFTER_COMM,
     EXITED_PROCESS_STATES, SWAP_PROCESS_RUNNING, SWAP_PROCESS_GONE, SWAP_PROCESS_ELSEWHERE, identity_file_value, this_machine,
     process_state_and_start, swap_process_state, git_work_tree_directory, outcome_path, write_outcome_atomically, outcome_text, append_outcome,
     write_outcome_start, parse_outcome, current_digest, STATE_LANDED, STATE_NOT_LANDED, STATE_TOUCHED, file_states, state_lines, recovery_lines,
     pending_refusal, read_outcome, seal_outcome, seal_holding_repository, SCRATCH_LOCK_NAME, SCRATCH_LOCK_HEARTBEAT_SECONDS,
     SCRATCH_LOCK_STALE_SECONDS, SCRATCH_LOCK_ATTEMPTS, SCRATCH_LOCK_COLUMNS, REPOSITORY_LOCK_NAME, REPOSITORY_LOCK_COLUMNS, REMOVAL_SUFFIX,
     REAP_INFIX, RECORD_REMOVED, RECORD_CHANGED, RECORD_BEING_REMOVED, RECORD_TAKEN_OVER, REMOVAL_TAKEN_OVER, HANDLED_SIGNALS, HELD_SCRATCH_LOCKS,
     HELD_REMOVAL_CLAIMS, REMOVAL_CLAIM_HEARTBEATS, ScratchHolder, ScratchLock, scratch_lock_path, scratch_lock_content, parse_scratch_holder,
     scratch_holder_occupies, scratch_busy_lines, stale_holder_hint, holder_description, repository_busy_lines, read_scratch_record, occupying_holder,
     remove_stale_record, removal_claim_lost, claim_record_removal, reap_dead_file, forget_removal_claim, release_removal_claim, link_scratch_record,
     keep_scratch_lock_fresh, keep_file_fresh, release_scratch_lock, release_then_die, install_release_handlers, restore_signal_handlers,
     start_holding, hold_scratch, acquire_record, repository_lock_path, hold_repository, swap, swap_holding_scratch, swap_holding_repository,
)  # noqa: E402,F401

def main(arguments):
    if arguments == ["--selftest"]:
        from hook_swap_selftest import selftest
        return selftest()
    usage = ("hook-swap.py <规格.json> --hook <钩子> --scratch <草稿目录> [--feeds <喂法.json>] [--root <仓根>] [--dry-run]，"
             "或 hook-swap.py <规格.json> --scratch <草稿目录> [--verify <命令>] [--root <仓根>] [--dry-run]（规格里的文件都在脚本规格目录下），"
             "或 hook-swap.py --feed-only --hook <钩子>，或 hook-swap.py --check-callers [<库.py>…]，或 hook-swap.py --read-outcome / --seal-outcome，或 hook-swap.py --selftest")
    options = {"--hook": None, "--scratch": None, "--feeds": None, "--root": REPOSITORY_ROOT, "--verify": None}
    dry_run, feed_only_mode, check_callers_mode, positional, position = False, False, False, [], 0
    read_outcome_mode, seal_outcome_mode = False, False
    while position < len(arguments):
        argument = arguments[position]
        if argument in options and position + 1 < len(arguments):
            options[argument] = arguments[position + 1]
            position += 2
            continue
        if argument == "--dry-run":
            dry_run = True
        elif argument == "--feed-only":
            feed_only_mode = True
        elif argument == "--check-callers":
            check_callers_mode = True
        elif argument == "--read-outcome":
            read_outcome_mode = True
        elif argument == "--seal-outcome":
            seal_outcome_mode = True
        else:
            positional.append(argument)
        position += 1
    if read_outcome_mode or seal_outcome_mode:
        if (read_outcome_mode and seal_outcome_mode) or positional or feed_only_mode or check_callers_mode or dry_run or any(options[name] is not None for name in ("--hook", "--feeds", "--verify")):
            print("✗ 参数不对：--read-outcome、--seal-outcome 二选一，只收 --root 和 --scratch（仓根不是 git 工作树的根时点名上一趟用的草稿目录）")
            print("→ 怎么办：hook-swap.py --read-outcome [--root <仓根>] [--scratch <草稿目录>]")
            return EXIT_BAD_INPUT
        try:
            outcome_root = os.path.realpath(options["--root"])
            code, lines = (read_outcome if read_outcome_mode else seal_outcome)(outcome_path(outcome_root, options["--scratch"]), outcome_root, break_mode())
        except BadInput as error:
            problem, _, next_step = str(error).partition("\n")
            print(f"✗ {problem}")
            print(f"→ {next_step.lstrip('→ ') or usage}")
            return EXIT_BAD_INPUT
        print("\n".join(lines))
        return code
    if check_callers_mode:
        if feed_only_mode or dry_run or any(options[name] is not None for name in ("--hook", "--scratch", "--feeds", "--verify")):
            print("✗ 参数不对：--check-callers 只收库路径（可省）和 --root，不收 --hook、--scratch、--feeds、--verify、--feed-only、--dry-run")
            print("→ 怎么办：hook-swap.py --check-callers [<库.py>…] [--root <仓根>]")
            return EXIT_BAD_INPUT
        try:
            code, lines = check_callers(options["--root"], positional)
        except BadInput as error:
            problem, _, next_step = str(error).partition("\n")
            print(f"✗ {problem}")
            print(f"→ {next_step.lstrip('→ ') or usage}")
            return EXIT_BAD_INPUT
        print("\n".join(lines))
        return code
    if feed_only_mode:
        if positional or options["--hook"] is None:
            print("✗ 参数不对：--feed-only 只要 --hook（可带 --feeds、--root），不收规格与 --scratch")
            print("→ 怎么办：hook-swap.py --feed-only --hook <钩子> [--feeds <喂法.json>] [--root <仓根>]")
            return EXIT_BAD_INPUT
        try:
            feeds = None
            if options["--feeds"] is not None:
                with open(options["--feeds"], encoding="utf-8") as handle:
                    feeds = json.load(handle)
            code, lines = feed_only(options["--hook"], options["--root"], feeds=feeds)
        except (OSError, ValueError) as error:
            print(f"✗ 读不了喂法（{error}）")
            print("→ 怎么办：喂法是 UTF-8 的 JSON 数组，写法见文件头")
            return EXIT_BAD_INPUT
        except BadInput as error:
            problem, _, next_step = str(error).partition("\n")
            print(f"✗ {problem}")
            print(f"→ {next_step.lstrip('→ ') or usage}")
            return EXIT_BAD_INPUT
        print("\n".join(lines))
        return code
    if len(positional) != 1 or options["--scratch"] is None:
        print("✗ 参数不对：要一份规格与 --scratch（钩子规格另要 --hook）")
        print(f"→ 怎么办：{usage}")
        return EXIT_BAD_INPUT
    try:
        with open(positional[0], encoding="utf-8") as handle:
            spec = json.load(handle)
        feeds = None
        if options["--feeds"] is not None:
            with open(options["--feeds"], encoding="utf-8") as handle:
                feeds = json.load(handle)
    except (OSError, ValueError) as error:
        print(f"✗ 读不了规格或喂法（{error}）")
        print("→ 怎么办：两份都是 UTF-8 的 JSON 数组，写法见文件头")
        return EXIT_BAD_INPUT
    try:
        code, lines = swap(spec, options["--hook"], options["--scratch"], options["--root"], feeds=feeds, dry_run=dry_run, verify=options["--verify"])
    except BadInput as error:
        problem, _, next_step = str(error).partition("\n")
        print(f"✗ {problem}")
        print(f"→ {next_step.lstrip('→ ') or usage}")
        return EXIT_BAD_INPUT
    print("\n".join(lines))
    return code


BREAK_VARIABLES = ("HOOK_SWAP_BREAK",)


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
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一项，跑 {setting[0]}=<项> python3 scripts/gate/hook-swap.py --selftest", file=sys.stderr)
    return 2


if __name__ == "__main__":
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
