#!/usr/bin/env python3
# admission: always 连的是此刻在跑的那份 prune-server，判的是这一次的读写与这一刻的服务，上一次的结论不替这一次作保
# run-condition: command python3
"""prune-server（判定库服务，仓根 prune-server/）的 Python 客户端：Python 写的三家判定库经它远程读写同一份库。
被 import 的函数库（只用标准库，import 本身不连服务、不读文件）；直接执行只认下面「命令行」几样。

帧格式照 prune-server/src/protocol.rs 文件头（冻结，这里只照抄、不另定）：
  每帧 [u32 帧体长度][帧体]，整数小端；请求帧体 [u64 序号][u8 操作码][内容]，回应帧体 [u64 序号][u8 状态][u8 操作码][内容]。
  用到的操作码：成批点读 1、成批写 2（带客户端身份与请求号）、按前缀扫 3、统计 4、报到 6（序号 0、服务不回）、宣布停服务 8（只认 unix socket，只给自证）、删前缀 9。
  回应状态：0 做完、1 内容解不开、2 认不得的操作码、3 帧长度不对（序号 0，服务随后关连接）、4 存储层出错、5 忙、6 服务停了（序号 0）。
  表只用结论表（表号 3）。TCP 上不认证（服务那一侧没有认证）。

三家缺省走服务，开关已废：三家（scripts/crash/lib_verdict_library.py 的 VerdictLibrary、scripts/gate/admission.py 的 VerdictLibrary、
  scripts/mutation/lib_mutation_units.py 的 Library）读写只有经这份客户端这一条路，本地库目录不建、不读、不写，也没有本地文件那条代码路径；
  服务连不上退 75，调用方照 75 整道退出，不当「无记录」「不在」现判。
  广谱变异的驱动（scripts/mutation/broad-mutation.py 的 Library）同样只经这份客户端读写，族 broad-mutation；库目录只放运行目录，判定不落本地文件。

键空间（library_store 一处定，三家不各抄一份）：结论表里，键 = 族名 [@ 命名空间] / 种类 / 相对路径（UTF-8）。
  族名：verdict-library（lib_verdict_library.py，种类是「<固定目录名>/<种类>」，例 singlefs-formal-unit-verdicts/proof-function）、
        unit-verdicts（admission.py，种类是 singlefs-unit-verdicts/ 下那一层目录名）、mutation-units（lib_mutation_units.py，种类是库目录名，
        例 singlefs-crates-mutation-library、singlefs-mutate-library）、
        broad-mutation（scripts/mutation/broad-mutation.py 的 Library；种类固定 broad-mutation-units，不随库目录名；相对路径是「<表>/<条目>」，
        只追加的表一行一个条目、条目名按写入次序升序，表与条目的写法在它文件头「判定库」）。
        broad-mutation 另起一族、不并进 mutation-units：族照写它的那一份代码分，它的存法（只追加的行、同一把单元键几行取最后一个定论）
        与 lib_mutation_units.py 的判定库记录不同，并在一族里两种存法同一个前缀、分不开。
        apparatus-items（scripts/experiment-apparatus/device-code-key.py 替 E174（借据丢掉时等待与显存回还）、E178（叶容器与记录死亡的计数）这类按代码键续跑的
        实验装置转交，种类是装置的 bin 名，相对键由装置给，撤回是改写成撤回标记；那份文件头「判定库的转交」）。
  相对路径：各家给的相对种类目录的路径（keys/<键>.json、records/<键>、compositions/<号>.json.gz 这类，各家照自己的写法给）；
    mutation-units 的单元表每个单元各存一条 units/<单元号>（内容只由单元号定，几个进程同时写不丢更新，服务没有锁可借）。
  命名空间：只在临时服务底下给（TEMPORARY_SERVICE_VARIABLE 设着，文件头「命令行」--with-temporary-service）：键里带「@<库目录的 sha256 前 16 位>」，
    自证与样本各格给各自临时目录下的库目录，在同一份临时服务里各占一份、不串。不在临时服务底下一律不带：库目录只取最后一层当库名，
    哪棵仓、在不在临时目录底下都不进键，主仓与仓副本读写同一份库（namespace_of）。
    弄坏开关 namespace-by-temporary-directory 下照改前的样子按库目录在不在临时目录（/tmp、/var/tmp、/dev/shm）底下给，不看临时服务。
  大值分段（Client.write_many / read_many / scan 一处做，调用方看到的照旧是一把键一个整值）：
    这一段是大值分段格式唯一的说明：Python 这一份与 prune-server 里的 Rust 客户端（崩溃判定库经它写读）照它逐字节相同地写、读、扫，Rust 源码指到这里。
    值长过 SEGMENT_BYTES（一帧装的上限 FRAME_BATCH_BYTES），或值本身以 SEGMENT_MANIFEST_MAGIC 起头的，切成每段至多 SEGMENT_BYTES 字节分段存：
      原键 <键>                      分段清单：SEGMENT_MANIFEST_MAGIC 后接 JSON {"length": 整值字节数, "sha256": 整值的 sha256, "segments": [各段字节数…]}
      段键 <键> NUL <段号>            段号是 SEGMENT_INDEX_DIGITS 位十进制、补零（段键按段号升序，排在原键之后）
      计数键 <键> NUL count           这把键分段存过的段数高水位（十进制），只增不减
    认段键只认形状：键尾是「NUL 加 SEGMENT_INDEX_DIGITS 位十进制」的是段键的形状（尾巴前面那一截是它的原键），键尾是「NUL 加 SEGMENT_COUNT_SUFFIX」的
      是计数键的形状（segment_key_parts、has_count_key_shape）；别的键都是原键，带不带 NUL 都一样（崩溃判定库的键以表标签开头、常带 NUL）。
      write_many 只拒这两种形状的原键（reserved-key，退 3，这一批一条都不写），扫时只跳过这两种形状的键。不带 NUL 的键（Python 五族的键）
      怎么认都一样。原键带 NUL 时，NUL 后面接一个小于「0」的字节的别的原键会排在原键与它的段键之间：扫时照下面「扫」那一条办。
    写：先点读这几把键的计数键；先写各段与计数键（一段一帧），最后一帧写清单与别的条目。段数取「这一个值要几段」与计数键里的高水位两者大的那一个，
      多出来的段写成空段、照进清单：重写同一把键时以前写过的段号全被新清单点名，旧段不留孤儿（协议没有删键，只能覆盖）。
      分段存过的键之后写小值也照分段存（高水位不为 0）。写到一半断了：清单还是旧的、段已换了一部分，读时核不过、报读错，重写这把键就好。
      段数到 10 的 SEGMENT_INDEX_DIGITS 次方（段号装不下）的值写之前拒（too-many-segments，退 3），这一批一条都不写；Rust 那一侧同一条（SEGMENT_INDEX_LIMIT）。
    读：原键读回的是清单就按清单逐段点读（每帧至多 FRAME_BATCH_BYTES）、拼回，核每段长度、整值长度与 sha256；段缺了、长度或 sha256 对不上，
      照 SEGMENTED_REREAD_DELAYS_SECONDS 逐次等、重读原键：清单换过（同一把键正被重写，读到的是旧清单配新段）就丢掉已经收下的段、照新清单重拼，
      原键不再是清单就交它；等完清单还是原来那一份、或清单解不开，当读错（退 3），不当「库里没有」。
    扫：段键与计数键的形状不当条目交出去，清单那一条交拼回的整值（scan_stream 逐条交，同一时刻只攒一把键的段：紧跟着正在拼的那把原键扫到的段先收下，
      中间隔了别的原键的，那把原键当场交、缺的段逐帧点读，之后再扫到它的段照形状跳过）；清单点名之外的段不交。
    raw_scan_stream 与 read_many_raw 交服务里的原样（段键、清单都照交）：前者只给自证看存法用，后者是点读、写与扫拼回整值时底下那一层。
      一帧被截断而一行都没回（头一行的值就装不下一帧）当场退 3（scan-row-over-frame），不往下拆前缀；Rust 那一侧
      （crates/singlefs-verification/src/verdict_store.rs 的 visit_stored_rows_with_prefix）同一条。

连哪一处（library_store 第一次读写时才连，一个进程一条连接，fork 出来的子进程自己另连）：
  unix socket：SINGLEFS_PRUNE_SERVER_SOCKET 设了用它，没设 ${XDG_RUNTIME_DIR}/singlefs-prune-server/server.sock；这个路径上有 socket 文件就走它（本机）。
  TCP：没有 socket 文件时走 TCP（第二台）。地址：SINGLEFS_PRUNE_SERVER_TCP_ADDRESS 设了用它；没设读多机配置的 PRUNE_SERVER_TCP_ADDRESS——
    SINGLEFS_MULTI_HOST_CONFIG 指的那一份，没设读仓根的 multi-host.env（第二台在同一棵共享的树里跑，读的是同一份）。
  两处都没有：退 3，点名缺的是哪一样。
  SINGLEFS_PRUNE_SERVER_SOCKET、SINGLEFS_PRUNE_SERVER_TCP_ADDRESS（改连接地址）与 SINGLEFS_PRUNE_SERVER_RECONNECT_SECONDS、SINGLEFS_PRUNE_SERVER_RESPONSE_SECONDS
    （改重连总时长与等回应的时限）这四个变量只在临时服务底下认：PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET 设了
    （--with-temporary-service 起的命令，与自己起临时服务的自证：TestServer 那几份，照同一个变量设）。平常跑设了任一个就退 3（connection-override），
    一条都不连：判定库只有那一份，改连接地址等于另连一份库，平常跑的时限取常量（connection_override_problem）。只看这一个标记、不核它与 socket 相同：
    临时服务底下的自证要造「连不上」时把这两个变量改指到不在的 socket 与刚放掉的端口，标记照旧指着临时服务。
    弄坏开关 connection-variables-honored-outside-temporary-service 下不在临时服务底下也照认（改前的样子）。
  连上之后头一帧发报到：主机名、TASK_SERVER_TASK、TASK_SERVER_ATTEMPT、进程号（不在 task-server 里跑的任务号与第几次写空）。

容错（照 prune-server/src/reliable_client.rs 文件头，一问一答、不流水）：
  - 单条超限当场报：发帧之前核帧体长度，超过帧长上限 MAXIMUM_FRAME_BODY_BYTES（大值已经分段，还超的是键或前缀太长、一帧装不下一条请求）
    当场退 3（帧长超限），写清是哪一种帧、几条、最长的键与最大的值多少字节、帧体多大、上限多少；不发、不重试、不退 75。
    服务回「帧长度不对」（状态 3，序号 0）同样退 3、不重连；发帧出错（服务拒了帧长、关了连接，写到一半断了）时先读出还躺在接收缓冲里的回应，
    是「帧长度不对」就照这一条退 3。
  - 回应超过帧长上限就拆键：成批点读的请求帧小、读回的值合计可以超过帧长上限，服务这时整帧拒成「内容解不开」（状态 1，说明里写读回会有多少字节，协议不变）。
    read_many_raw 收到这一种拒而这一帧不止一把键，就把键对半拆开各自再问、递归到单键，按原次序接起来（单键回应至多一段长加帧头，不会超过：长过一段的值分段存）；
    单键还被拒照「服务拒了帧」退 3。点读原键与拼大值时读段的那一帧都走这一层。别的拒（认不得的操作码、存储层出错）与别的操作码的帧照旧当场退 3，不拆。
    回应多大事先不知道（不分段的值至多一段长，按这个上界切一帧只装得下三四把键），所以先整帧问、被拒再拆；与 prune-server/src/reliable_client.rs 的 read 同一条规则。
  - 连不上、断了、等回应超时（RESPONSE_TIMEOUT_SECONDS；临时服务底下 SINGLEFS_PRUNE_SERVER_RESPONSE_SECONDS 可改）、回应帧长度不对：关连接，指数退避加抖动（起始 RECONNECT_INITIAL_SECONDS、每次翻倍、封顶 RECONNECT_CEILING_SECONDS，再乘 0.5 到 1 的随机数）
    之后重连重发同一帧（序号换新的，写帧的身份与请求号不变，服务按它们去重，重发是幂等的）；从头一次出事算起总时长到
    RECONNECT_LIMIT_SECONDS（临时服务底下 SINGLEFS_PRUNE_SERVER_RECONNECT_SECONDS 可改）或次数到 RECONNECT_ATTEMPTS_LIMIT 还没成，退 75（连不上）。
    建连（连接与报到）只用这一轮总时长上限剩下的时间，不给满一份等回应的超时：服务那头握手迟迟不完成、连接队列满了时
    也在总时长上限里收场（照 prune-server/src/client.rs 文件头「建连有期限」；建连那一步出的事从开始连的那一刻算进总时长）。
  - 服务回忙：照它给的毫秒数再加 0 到一半的随机抖动之后重发同一帧；同一帧忙 BUSY_RETRIES_LIMIT 次退 75（忙到上限）。
  - 服务说它停了（状态 6）：不再重试，退 75（服务停了）。
  - 读失败一律退出，不当成「键不在」交回去：调用方拿到的「不在」只来自服务回的「在不在 = 0」。
  - 写在服务回确认（WAL 同步之后）才算数；没确认就退出的写这一趟丢了，下一趟那一条照现判（判定库的写都是判完才写，丢了只是少复用）。
退出码（失败时抛 PruneServerFailure，它是 SystemExit：调用方的 except Exception 拦不住，进程照它的码退；抛之前在标准错误打 ✗ 与出路）：
  75  prune-server 不可用：服务有意停了、重连到上限、同一帧忙到上限。修好、重起服务之后由提交它的会话重交。
  3   别的错：socket 与 TCP 地址都没有（配置）、服务拒了帧（内容解不开、认不得的操作码、存储层出错）、回应解不开（协议对不上）、
      帧长超限（frame-too-large）、分段存的值拼不回（segmented-read）、原键是段键或计数键的形状（reserved-key）、
      值要的段数段号装不下（too-many-segments）、按前缀扫一行就装不下一帧（scan-row-over-frame）、
      不在临时服务底下设了改连接地址或时限的变量（connection-override，「连哪一处」）。
  不用 78（task-server 已占：连不上 task-server 本身）。

命令行：
  python3 scripts/prune-server/prune_server_client.py check            连一次、取统计，打 PRUNE_SERVER_CLIENT_CHECK route=… counters=…；失败照上面的退出码
  python3 scripts/prune-server/prune_server_client.py read <族> <种类> <库目录> <相对路径>
      读一条（键照 LibraryStore(<族>, <种类>, <库目录>) 的写法拼，「键空间」）：在就把值原样写到标准输出、退 0；不在标准输出什么都不写、退 1；
      族认不得、参数个数不对退 2；读失败照上面的退出码。只读、不写；给样本与查库的人用，不 import 这份文件也能读一条。
  python3 scripts/prune-server/prune_server_client.py delete-prefix [--dry-run] <族> <种类> <库目录> <相对路径前缀>
  python3 scripts/prune-server/prune_server_client.py delete-prefix [--dry-run] --hex <非空的十六进制键前缀> [--hex <…>]…
      （--hex 可以写几次，逐个前缀做，多于一个时末行另打 PRUNE_SERVER_DELETE_PREFIX_TOTAL mode=… prefixes=<个数> keys=<总数>）
      数或删一个键前缀底下的全部键（服务端在提交线程里成批做，一帧至多 DELETE_PREFIX_KEYS_PER_FRAME 把，客户端接着来到前缀下没有更多的键；
      协议见 prune-server/src/protocol.rs 文件头「删前缀」）。不认表码、不为哪一张表写分支：第一种照 LibraryStore 拼键（临时服务底下带命名空间），
      第二种是库里的原样键字节（崩溃判定库一张表的键以表标签那一个字节起头，例 --hex 1c）。--dry-run 只数不删，数的口径和删一样。
      末行 PRUNE_SERVER_DELETE_PREFIX mode=dry-run|deleted keys=<条数> prefix=<键前缀的十六进制>，退 0；参数不对退 2；空前缀与落在服务记账那一段
      （0xF0 起头）的前缀服务拒（退 3）。删本来就幂等，不按请求号去重：断线重发只会让某一帧报的数少一批；要精确的数就先 --dry-run、再删、再 --dry-run 数到 0。
  python3 scripts/prune-server/prune_server_client.py delete-by-item-name [--dry-run] <族> <库目录> <名单文件>
      按值挑键删：判定记录的项名不在键里（键是单元键的哈希），按键前缀删不到，只能读值挑。名单文件每行「种类<TAB>项名」（# 起头与空行跳过）；
      每个种类整个扫一遍，读每条值，值是 JSON 对象且 item 字段是这个种类名单里的项名的，按完整键逐条删（连段键与计数键）；别的种类里同名的项不动。
      每条打 PRUNE_SERVER_DELETE_BY_ITEM_NAME_ROW mode=dry-run|deleted kind=<种类> keys=<条数> key=<完整键> item=<项名>，名单里没找到任何记录的（种类、项名）
      逐条打 PRUNE_SERVER_DELETE_BY_ITEM_NAME_NOT_FOUND kind=… item=…，末行 PRUNE_SERVER_DELETE_BY_ITEM_NAME mode=… kinds=… matched=<条> keys=<总键数> names=… names_not_found=…
      non_records=<值不是带 item 的 JSON 对象的条数> refused=<没删的条数>。--dry-run 只数不删、口径和删一样。退 0；参数不对、名单读不了或格式不对退 2；
      某条的键是同一种类里另一把原键的前缀（删前缀会连带删掉那一把）时那一条不删、点名，整次退 1；服务那一侧的拒绝与连不上照客户端的退出码。
      先 --dry-run 数一遍并核对逐条打出的键与项名，再不带它删，再 --dry-run 数到 matched=0。
  python3 scripts/prune-server/prune_server_client_selftest.py --selftest [--server-binary <prune-server 二进制>]
      自证：每格在临时目录起一份真 prune-server（unix socket 加回环 TCP），逐格走一遍；
      二进制从 --server-binary 或环境变量 PRUNE_SERVER_BINARY 取，都没给时：环境里有任务服务给的 CARGO_TARGET_DIR 才在这里起 cargo build --release --offline 编一份进那个编译目录（是重活，交 task-server）；
      没有 CARGO_TARGET_DIR 不现编（会编进主仓的 prune-server/target），退 2，出路是整条经 task-server 交（服务给编译目录、包装现编一遍 prune-server，不带 --server-binary）：
      task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/prune-server/prune_server_client_selftest.py --selftest；
      自己编了、两台都读得到的二进制才另加 --server-binary，写在整条之外。
      末行 PRUNE_SERVER_CLIENT_SELFTEST cells=… passed=… failed=… failed_cells=…；全绿之后逐个打开下面的弄坏开关重跑，末行另有一行
      PRUNE_SERVER_CLIENT_BREAK_SWEEP switches=… matched=…。退出码：0 全绿且每个开关都红在点名那一格；1 有红；2 用法错。
  python3 scripts/prune-server/prune_server_client_selftest.py --selftest-families [--server-binary <…>]
      起一份临时服务，逐份对它跑三家与它们旁路的原有自证（family_selftests），每份打一行 FAMILY_SELFTEST script=… exit=…，
      末行 PRUNE_SERVER_FAMILY_SELFTESTS scripts=… passed=… failed=…。
  python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service [--server-binary <…>] -- <命令> [<参数>…]
      起一份临时服务（二进制取法同 --selftest），把连哪一处（SOCKET_VARIABLE 与 TCP_ADDRESS_VARIABLE）指到它、设 TEMPORARY_SERVICE_VARIABLE，
      起那条命令（不经 shell，标准输出与标准错误照原样），等它退出再停服务；退出码是那条命令的，起不了服务退 2。
      三家与旁路的原有自证（family_selftests 那几份）只在这样起的临时服务上跑：开跑前调 selftest_service_problem()，
      不是临时服务（没有 TEMPORARY_SERVICE_VARIABLE，或它与连哪一处对不上、socket 不在）就退 2、一格都不跑，不写真服务。
      自证起的子进程照继承这几个变量，沿用同一份临时服务（各格的库目录在各自的临时目录里，命名空间不串）。
      selftest_service_problem(environment=None)：给了 environment 就只判那一份（调用方替子进程拼的环境），不看本进程的 os.environ；
      不给照本进程的 os.environ 判（开跑前核自己的那几份）。弄坏开关 service-problem-reads-process-environment 下给了也读 os.environ（改前的样子）。
  临时服务跟着起它的进程走（prune_server_client_selftest.py 的 TestServer，--with-temporary-service、--selftest-families、自证各格与别的脚本借它起的都是这一份）：
      服务经一段 python 起，先设父进程死亡信号（prctl PR_SET_PDEATHSIG 给 SIGKILL）、核起它的进程还在，再换成服务二进制；起它的进程被 KILL、finally 不跑时，
      内核当场停服务，不留孤儿。父进程死亡信号认的是起它的那一条线程：TestServer 只在活到收尾的线程里建（线程退出服务就跟着停）。
      弄坏开关 temporary-service-outlives-parent 下直接起二进制、不设（改前的样子）。

自证的格（CELLS，次序相同）：
  round-trip-unix                     经 unix socket 写三条、读回三条加一条不在的；同一请求号再发一遍回「早已生效」、值不变
  prefix-scan                         两个前缀各写几条，扫一个前缀只回它底下的、按键升序
  tcp-round-trip                      没有 socket 文件时走 TCP（地址取环境变量），读写往返
  busy-retried-after-delay            服务待处理上限 1、每组提交拖 200 毫秒，两条连接同时写：收到忙、每次等够服务给的毫秒数再重发、全部落库
  stopped-exits-75                    服务宣布停了之后子进程读一次：很快退 75、说服务停了
  unreachable-exits-75                服务被 kill -9 之后子进程读一次（重连总时长 3 秒）：限时内退 75、说连不上，不打「不在」
  families-over-service               五族（三家、广谱变异的驱动与装置的转交）各读写一遍：服务侧按各自前缀扫读得到、五族前缀互不相交、本地库目录一个文件都没建
  unreachable-family-exits-75         服务连不上，lib_verdict_library 的 lookup 子进程退 75（不答「没给判定库」「无记录」现判），库目录没建
  connect-bounded-by-reconnect-total  重连总时长 2 秒、等回应 30 秒：unix socket 连接队列满（只听不收、塞满）与 TCP 握手不完成（backlog 塞满、SYN 被丢）
                                      两种现场，子进程读一次都在总时长加 3 秒里退 75、说连不上，不堵到等回应的超时。unix 连接队列满时带超时的
                                      socket 连接当场回 EAGAIN（CPython 只在 EINPROGRESS 时等），这一种现场不分建连给不给满时长；分得出的是 TCP 那一种
  large-value-round-trip              写一个大过帧长上限的值与一条小值：逐字节读回、按前缀扫只交两条原键（大值拼回）、服务里原样有段键；
                                      同一把键重写成两段长的值：读回新值，服务里这把键的段号与新清单点名的一一对上、计数键等于清单段数（旧段不留孤儿）
  segmented-read-incomplete-refused   一把分段键清单点名的段缺一段、一把一段短了、一把一段同长换了字节：点读三把都退 3（分段拼不回），按前缀扫那把换了字节的同样退 3，
                                      没有一把读成「不在」或交回值
  segmented-read-sees-a-rewrite       一把分段键的段已换成新值、清单还是旧的，另一条线程过一会儿写新清单：点读与按前缀扫都交回新值（重读清单之后照新清单拼），不退 3
  segment-count-over-index-digits-refused  段长与段号位数调小（1 字节、1 位）之后写一个要 11 段的值：退 3（too-many-segments），原键、段键、计数键一条都没写；要 10 段的照写照读回
  scan-row-over-frame-refused         按前缀扫的一帧截断而一行都没回（假的服务）：问了一帧就退 3（scan-row-over-frame），不往下拆前缀
  segment-key-shapes-only-reserved    带 NUL 而键尾不是段键或计数键形状的几把原键（都以表标签那样的字节起头；NUL 接 NUL 的、NUL 后接五位与七位十进制的、
                                      NUL 后接「count」再接别的字节的、一把分段存的大值，与一把排在这把大值的原键和它的段键之间的原键）：写了逐字节读回，按前缀扫交的键正好是这几把原键、
                                      大值拼回相同，服务里原样有这把大值的段键与计数键；键尾是段键形状、计数键形状的原键各一把（与一条合法条目同批）：
                                      写时都退 3（reserved-key），同批那条合法条目也没写进服务
  oversize-frame-reported-at-once     子进程写一条键长超过帧长上限的条目（重连总时长 3 秒）：当场退 3、说帧长超限与上限多少，不说连不上；
                                      进程内把本地上限放宽到两倍再写同一条（照发给服务）：服务回「帧长度不对」，当场退 3（帧长超限），不重连
  temporary-service-exits-with-parent  一个子进程起一份 TestServer、打出服务的进程号之后被 SIGKILL：服务在 PARENT_DEATH_EXIT_WITHIN_SECONDS 秒内不在
                                      （进程号不在、成了僵尸、或换成了别的进程都算不在；还在就由这一格停掉它，判红）
  service-problem-judges-given-environment  本进程环境里没有临时服务、交进去的那一份指着一份活的临时服务：selftest_service_problem(交进去的) 交空串；
                                      本进程环境指着临时服务、交进去的是空的一份：交一句为什么
  connection-variables-refused-outside-temporary-service  临时服务的标记设成空，子进程只设 socket、设 TCP 地址（socket 指到不在的路径）、只设重连总时长、只设等回应时限各读一次：都退 3、点名那个变量、
                                      不打 absent 与 present；同一份环境把标记指回临时服务再读一次：照常读到 absent、退 0
  command-line-read                   经 LibraryStore 写一条之后起命令行 read：在的那一条原样打出、退 0；不在的退 1、标准输出空；认不得的族退 2
  namespace-only-under-temporary-service  标记设着时临时目录下两份与仓里一份库目录交三个不同的「@…」；标记空着时临时目录下与仓里的都交空串、前缀不带「@」（不起服务）
  binary-not-built-without-target-directory  没给 --server-binary 与 PRUNE_SERVER_BINARY、环境里也没有任务服务给的 CARGO_TARGET_DIR 时 resolve_binary 不起 cargo、
                                      交「不现编」和点名 --server-binary 的说明（自证退 2）；有 CARGO_TARGET_DIR 时才现编（cargo 在这一格里换成只记调用的替身）
  delete-prefix-by-namespace          临时服务底下两份库目录（命名空间不同）在同一个族同一个种类底下各写几条：对其中一份的相对前缀先只数不删（条数对、库不动）、再删、再数是 0，
                                      相对前缀之外的键与另一份的键一条不少；命令行 delete-prefix --dry-run 与不带它对另一份各起一次：行里 keys 对、删完那一份空
  delete-by-item-name-picks-by-value  临时服务底下两个种类各写几条判定记录（JSON，item 字段是项名；含一条大到分段存的、一条不是 JSON 的）：名单点名其中一个种类的两个项名与另一个种类的一个不存在的项名，
                                      命令行 --dry-run 数到的条数对、逐条打键与项名、库不动；不带 --dry-run 删掉后只少点名种类里项名命中的那几条（含分段的段键），
                                      另一个种类里同名的项与没点名的项一条不少；再 --dry-run 数到 matched=0；名单里没找到记录的项名逐条点名
  batched-read-answer-over-frame-limit-split  五把各正好一段长（不分段存）、合计超过帧长上限的值，中间夹一条小值与一把不在的键，一次 read_many 读全：
                                      逐字节对、次序与问的相同；那五把都没有计数键（真没分段）
弄坏开关 PRUNE_SERVER_CLIENT_BREAK=<项>（只在自证里认：自证给自己和它起的子进程设 PRUNE_SERVER_CLIENT_SELFTEST=1；直接执行别的用法时设了就退 2）。
一条一行「PRUNE_SERVER_CLIENT_BREAK=<项>  弄坏哪一处」，下一行起写打开后红的格（失败行「✗ 自证 <格名>：」的句首；自证末尾的开关轮按 BREAK_RED_CELLS 逐条核同一格）：
  PRUNE_SERVER_CLIENT_BREAK=read-failure-reported-as-absent  读失败时交回「键都不在」
      unreachable-exits-75（「✗ 自证 unreachable-exits-75：」）
  PRUNE_SERVER_CLIENT_BREAK=reconnect-unbounded  重连没有次数与总时长上限
      unreachable-exits-75（「✗ 自证 unreachable-exits-75：」）
  PRUNE_SERVER_CLIENT_BREAK=stop-notice-retried  收到「服务停了」当连接断了、照重连办
      stopped-exits-75（「✗ 自证 stopped-exits-75：」）
  PRUNE_SERVER_CLIENT_BREAK=busy-retry-without-delay  收到忙立刻重发
      busy-retried-after-delay（「✗ 自证 busy-retried-after-delay：」）
  PRUNE_SERVER_CLIENT_BREAK=family-prefixes-shared  五族用同一个族名
      families-over-service（「✗ 自证 families-over-service：」）
  PRUNE_SERVER_CLIENT_BREAK=unreachable-treated-as-no-library  服务连不上时 library_store 交 None（三家照「没有判定库」全部现判）
      unreachable-family-exits-75（「✗ 自证 unreachable-family-exits-75：」）
  PRUNE_SERVER_CLIENT_BREAK=connect-given-full-response-timeout  建连（连接与报到）给满一份等回应的超时，不看重连总时长剩多少
      connect-bounded-by-reconnect-total（「✗ 自证 connect-bounded-by-reconnect-total：」）
  PRUNE_SERVER_CLIENT_BREAK=large-value-sent-unsegmented  大值不分段、整条照一帧发（改前的样子）
      large-value-round-trip（「✗ 自证 large-value-round-trip：」）
  PRUNE_SERVER_CLIENT_BREAK=segment-count-shrinks-on-rewrite  写时不看计数键的高水位，段数只取这一个值要几段
      large-value-round-trip（「✗ 自证 large-value-round-trip：」）
  PRUNE_SERVER_CLIENT_BREAK=segments-assembled-without-sha256  拼回之后不核整值的 sha256
      segmented-read-incomplete-refused（「✗ 自证 segmented-read-incomplete-refused：」）
  PRUNE_SERVER_CLIENT_BREAK=segmented-read-not-reread  拼不回当场退 3，不重读原键（改前的样子）
      segmented-read-sees-a-rewrite（「✗ 自证 segmented-read-sees-a-rewrite：」）
  PRUNE_SERVER_CLIENT_BREAK=segment-count-unbounded  写时不核段数装不装得下段号（改前的样子）
      segment-count-over-index-digits-refused（「✗ 自证 segment-count-over-index-digits-refused：」）
  PRUNE_SERVER_CLIENT_BREAK=scan-row-over-frame-split  截断而一行都没回照样往下拆前缀（改前的样子）
      scan-row-over-frame-refused（「✗ 自证 scan-row-over-frame-refused：」）
  PRUNE_SERVER_CLIENT_BREAK=nul-keys-reserved-whole  写时拒一切带 NUL 的原键、扫时凡带 NUL 的键都不当条目（改前的样子）
      segment-key-shapes-only-reserved（「✗ 自证 segment-key-shapes-only-reserved：」）
  PRUNE_SERVER_CLIENT_BREAK=segment-key-shapes-accepted  写时不核原键的形状，段键、计数键形状的原键照写
      segment-key-shapes-only-reserved（「✗ 自证 segment-key-shapes-only-reserved：」）
  PRUNE_SERVER_CLIENT_BREAK=other-segment-keys-scanned-as-entries  扫时只收下紧跟着正在拼的那把原键的段，别的段键与计数键当条目交
      segment-key-shapes-only-reserved（「✗ 自证 segment-key-shapes-only-reserved：」）
  PRUNE_SERVER_CLIENT_BREAK=oversize-frame-sent-and-retried  发帧之前不核帧长，服务回「帧长度不对」当连接断了重连重发（改前的样子）
      oversize-frame-reported-at-once（「✗ 自证 oversize-frame-reported-at-once：」）
  PRUNE_SERVER_CLIENT_BREAK=temporary-service-outlives-parent  TestServer 直接起服务二进制、不设父进程死亡信号（改前的样子）
      temporary-service-exits-with-parent（「✗ 自证 temporary-service-exits-with-parent：」）
  PRUNE_SERVER_CLIENT_BREAK=service-problem-reads-process-environment  selftest_service_problem 给了环境也照读本进程的 os.environ（改前的样子）
      service-problem-judges-given-environment（「✗ 自证 service-problem-judges-given-environment：」）
  PRUNE_SERVER_CLIENT_BREAK=connection-variables-honored-outside-temporary-service  不在临时服务底下设了改连接地址或时限的变量也照认、照连（改前的样子）
      connection-variables-refused-outside-temporary-service（「✗ 自证 connection-variables-refused-outside-temporary-service：」）
  PRUNE_SERVER_CLIENT_BREAK=command-line-read-absent-as-empty  命令行 read 读到不在的那一条照退 0（标准输出空），分不出「不在」与「值是空的」
      command-line-read（「✗ 自证 command-line-read：」）
  PRUNE_SERVER_CLIENT_BREAK=namespace-by-temporary-directory  命名空间照改前按库目录在不在临时目录底下给，不看临时服务的标记
      namespace-only-under-temporary-service（「✗ 自证 namespace-only-under-temporary-service：」）
  PRUNE_SERVER_CLIENT_BREAK=oversized-answer-read-unsplit  成批点读的回应超过帧长上限、服务整帧拒时不拆键，照拒退 3（改前的样子）
      batched-read-answer-over-frame-limit-split（「✗ 自证 batched-read-answer-over-frame-limit-split：」）
  PRUNE_SERVER_CLIENT_BREAK=build-without-target-directory  没有任务服务给的编译目录时 resolve_binary 照旧回落现编（改前的样子）
      binary-not-built-without-target-directory（「✗ 自证 binary-not-built-without-target-directory：」）
  PRUNE_SERVER_CLIENT_BREAK=item-name-ignores-kind  delete-by-item-name 每个种类用名单里全部种类的项名（改前：不分种类按项名删）
      delete-by-item-name-picks-by-value（「✗ 自证 delete-by-item-name-picks-by-value：」）
  PRUNE_SERVER_CLIENT_BREAK=item-name-dry-run-deletes  delete-by-item-name 带 --dry-run 也删
      delete-by-item-name-picks-by-value（「✗ 自证 delete-by-item-name-picks-by-value：」）
  PRUNE_SERVER_CLIENT_BREAK=delete-prefix-without-namespace  LibraryStore.delete_prefix 拼的前缀不带命名空间（改前的键空间样子）
      delete-prefix-by-namespace（「✗ 自证 delete-prefix-by-namespace：」）
没有弄坏开关的格：round-trip-unix、prefix-scan、tcp-round-trip（帧格式由服务那一侧的自证与弄坏开关管，这里错了每一格都红）。
"""
import contextlib
import hashlib
import importlib.util
import json
import os
import random
import shutil
import signal
import socket
import struct
import subprocess
import sys
import tempfile
import threading
import time

SOCKET_VARIABLE = "SINGLEFS_PRUNE_SERVER_SOCKET"
TCP_ADDRESS_VARIABLE = "SINGLEFS_PRUNE_SERVER_TCP_ADDRESS"
RECONNECT_LIMIT_VARIABLE = "SINGLEFS_PRUNE_SERVER_RECONNECT_SECONDS"
RESPONSE_TIMEOUT_VARIABLE = "SINGLEFS_PRUNE_SERVER_RESPONSE_SECONDS"
MULTI_HOST_CONFIGURATION_VARIABLE = "SINGLEFS_MULTI_HOST_CONFIG"
MULTI_HOST_ADDRESS_KEY = "PRUNE_SERVER_TCP_ADDRESS"
BREAK_VARIABLE = "PRUNE_SERVER_CLIENT_BREAK"
SELFTEST_VARIABLE = "PRUNE_SERVER_CLIENT_SELFTEST"
BINARY_VARIABLE = "PRUNE_SERVER_BINARY"
TEMPORARY_SERVICE_VARIABLE = "PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET"   # --with-temporary-service 起的临时服务的 socket（文件头「命令行」）
# 只在临时服务底下认的四个变量（文件头「连哪一处」）
OVERRIDE_VARIABLES = (SOCKET_VARIABLE, TCP_ADDRESS_VARIABLE, RECONNECT_LIMIT_VARIABLE, RESPONSE_TIMEOUT_VARIABLE)

FAMILY_VERDICT_LIBRARY = "verdict-library"
FAMILY_UNIT_VERDICTS = "unit-verdicts"
FAMILY_MUTATION_UNITS = "mutation-units"
FAMILY_BROAD_MUTATION = "broad-mutation"
FAMILY_APPARATUS_ITEMS = "apparatus-items"
FAMILIES = (FAMILY_VERDICT_LIBRARY, FAMILY_UNIT_VERDICTS, FAMILY_MUTATION_UNITS, FAMILY_BROAD_MUTATION, FAMILY_APPARATUS_ITEMS)
SHARED_FAMILY_NAME_UNDER_BREAK = "library"
TEMPORARY_ROOTS_UNDER_BREAK = ("/tmp", "/var/tmp", "/dev/shm")   # 只给弄坏开关 namespace-by-temporary-directory 用（改前按目录给命名空间的那几处）
NAMESPACE_DIGEST_CHARACTERS = 16

EXIT_UNAVAILABLE = 75
EXIT_OTHER = 3
EXIT_USAGE = 2

RECONNECT_INITIAL_SECONDS = 0.05
RECONNECT_CEILING_SECONDS = 5.0
RECONNECT_LIMIT_SECONDS = 120.0
RECONNECT_ATTEMPTS_LIMIT = 30
RESPONSE_TIMEOUT_SECONDS = 120.0
BUSY_RETRIES_LIMIT = 1000

# protocol.rs 文件头的取值
FRAME_LENGTH_FIELD_BYTES = 4
REQUEST_HEADER_BYTES = 9
RESPONSE_HEADER_BYTES = 10
MAXIMUM_FRAME_BODY_BYTES = 64 * 1024 * 1024
CONNECTION_LEVEL_SEQUENCE = 0
MAXIMUM_CLIENT_IDENTITY_BYTES = 255
MAXIMUM_ANNOUNCEMENT_TEXT_BYTES = 255
TABLE_CONCLUSIONS = 3
OPERATION_POINT_READ = 1
OPERATION_WRITE = 2
OPERATION_PREFIX_SCAN = 3
OPERATION_STATISTICS = 4
OPERATION_DELETE_PREFIX = 9
OPERATION_ANNOUNCEMENT = 6
OPERATION_ANNOUNCE_STOP = 8
STATUS_COMPLETED = 0
STATUS_MALFORMED_PAYLOAD = 1
STATUS_UNKNOWN_OPERATION = 2
STATUS_FRAME_LENGTH_REJECTED = 3
STATUS_STORAGE_FAILED = 4
STATUS_BUSY = 5
STATUS_SERVER_STOPPING = 6
# 一帧装的上限：远低于服务的帧长上限，键多、值大时切成几帧（每帧一个写批，原子性只在一帧之内）
FRAME_BATCH_BYTES = 16 * 1024 * 1024
FRAME_BATCH_KEYS = 8192
SCAN_ENTRIES_LIMIT = 0xFFFFFFFF
# 删前缀一帧至多数到或删掉的键数（protocol.rs 的 MAXIMUM_DELETE_PREFIX_KEYS_PER_FRAME，超过服务回状态 1）
DELETE_PREFIX_KEYS_PER_FRAME = 200_000
SCAN_SPLIT_BYTE_VALUES = 256
# 大值分段（文件头「键空间」大值分段）
SEGMENT_BYTES = FRAME_BATCH_BYTES
SEGMENT_KEY_SEPARATOR = b"\x00"
SEGMENT_INDEX_DIGITS = 6
# 分段存的值拼不回时重读原键之前依次等这么久（文件头「键空间」大值分段「读」；Rust SEGMENTED_REREAD_DELAYS_MILLISECONDS 同一张表）
SEGMENTED_REREAD_DELAYS_SECONDS = (0.05, 0.1, 0.2, 0.4, 0.8, 1.6)
SEGMENT_COUNT_SUFFIX = b"count"
SEGMENT_MANIFEST_MAGIC = b"\x00singlefs-prune-server-segments\n"
SHOWN_KEY_CHARACTERS = 200
# 发帧出错之后，读还躺在接收缓冲里的「帧长度不对」至多等这么久（文件头「容错」单条超限）
REJECTION_DRAIN_SECONDS = 1.0

REPOSITORY_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))


def break_is(name):
    """弄坏开关（文件头「弄坏开关」）：只在自证与它起的子进程里认。"""
    if os.environ.get(SELFTEST_VARIABLE) != "1":
        return False
    return name in os.environ.get(BREAK_VARIABLE, "").split(",")


class PruneServerFailure(SystemExit):
    """读写没成（文件头「退出码」）：kind 是 stopped、unreachable、busy-limit（75），configuration、rejected、protocol、
    frame-too-large、segmented-read、reserved-key、too-many-segments、scan-row-over-frame（3）。
    是 SystemExit：调用方不接就照 code 退出；抛之前已在标准错误打过 ✗ 与出路。"""

    EXIT_CODES = {"stopped": EXIT_UNAVAILABLE, "unreachable": EXIT_UNAVAILABLE, "busy-limit": EXIT_UNAVAILABLE,
                  "configuration": EXIT_OTHER, "rejected": EXIT_OTHER, "protocol": EXIT_OTHER,
                  "frame-too-large": EXIT_OTHER, "segmented-read": EXIT_OTHER, "reserved-key": EXIT_OTHER, "connection-override": EXIT_OTHER,
                  "too-many-segments": EXIT_OTHER, "scan-row-over-frame": EXIT_OTHER}

    def __init__(self, kind, message):
        super().__init__(self.EXIT_CODES[kind])
        self.kind = kind
        self.message = message

    def __str__(self):
        return f"{self.kind}：{self.message}"

    def __reduce__(self):
        # 进程池的工作进程里抛的照原样 pickle 回父进程（父进程照它的码退），不按 SystemExit 的 args 重建成认不得的样子
        return PruneServerFailure, (self.kind, self.message)


REMEDIES = {
    "stopped": "prune-server 有意停了（修缺陷）：等主 agent 修好、重起服务之后重交这个任务；不要改成走本地库",
    "unreachable": "看服务在不在跑（prune-server/control check）、第二台连的地址与端口通不通；服务回来之后重交这个任务；不要改成走本地库",
    "busy-limit": "服务一直回忙：看 prune-server/control watch 与服务的待处理上限；过一会儿重交",
    "configuration": f"本机要有服务的 socket，第二台要有多机配置里的 {MULTI_HOST_ADDRESS_KEY}；判定库没有不走服务的那条路",
    "rejected": "服务拒了这一帧：这是客户端或服务的缺陷，把这一行交主 agent（prune-server 那一侧当场修）",
    "protocol": "回应解不开：客户端与服务的帧格式对不上，把这一行交主 agent（prune-server 那一侧当场修）",
    "frame-too-large": "一条请求大过帧长上限、又分不了段（键或前缀太长）：这是调用方的缺陷，把这一行交主 agent；重交照样拒，不要重交、不要改成走本地库",
    "segmented-read": "分段存的值拼不回（段缺了、长度或 sha256 对不上：写到一半断了或被改过）：重写这把键（让写它的那一道重判）；不要当成库里没有",
    "reserved-key": (f"原键的键尾是「NUL 加 {SEGMENT_INDEX_DIGITS} 位十进制」或「NUL 加 {SEGMENT_COUNT_SUFFIX.decode()}」："
                     "这两种形状留给大值分段的段键与计数键（客户端文件头「键空间」大值分段）；"
                     "换一把键尾不是这两种形状的键（别处带 NUL 照收）"),
    "too-many-segments": (f"值要的段数超过 {SEGMENT_INDEX_DIGITS} 位十进制段号装得下的：这是调用方的缺陷（一个值大到这一步不该进判定库），"
                          "把这一行交主 agent；重交照样拒"),
    "scan-row-over-frame": ("前缀底下头一行的值就装不下一帧（大值分段之前写进来的整值）：服务停着时用 prune-server drop-oversized-values 清掉，"
                            "由各入口现判写回；不要改成走本地库"),
    "connection-override": (f"平常跑 unset {'、'.join(OVERRIDE_VARIABLES)}（本机连 socket，第二台连多机配置的 {MULTI_HOST_ADDRESS_KEY}，时限取常量）；"
                            "自证、样本与压测照文件头「命令行」--with-temporary-service 起，在临时服务底下才认这四个变量"),
}


def fail(kind, message):
    print(f"  ✗ prune-server 客户端：{message}（退 {PruneServerFailure.EXIT_CODES[kind]}）", file=sys.stderr)
    print(f"     → 怎么办：{REMEDIES[kind]}", file=sys.stderr)
    raise PruneServerFailure(kind, message)


def positive_seconds(variable, default):
    """时限变量（文件头「连哪一处」）：只在临时服务底下认，平常跑取常量 default（平常跑设了已由 connection_override_problem 退 3）。"""
    if not os.environ.get(TEMPORARY_SERVICE_VARIABLE, "") and not break_is("connection-variables-honored-outside-temporary-service"):
        return default
    text = os.environ.get(variable, "").strip()
    if not text:
        return default
    try:
        value = float(text)
    except ValueError:
        value = 0.0
    if value <= 0:
        fail("configuration", f"{variable}={text!r} 要是正数（秒）")
    return value


# ── 连哪一处 ──

def multi_host_configuration_candidates():
    """多机配置文件（文件头「连哪一处」）：SINGLEFS_MULTI_HOST_CONFIG 指的那一份，没设是仓根的 multi-host.env。"""
    configured = os.environ.get(MULTI_HOST_CONFIGURATION_VARIABLE, "")
    if configured:
        return [configured]
    return [os.path.join(REPOSITORY_ROOT, "multi-host.env")]


def configuration_value(path, key):
    """KEY=值 一行一个，值不做 shell 展开、两头成对的引号去掉一层；文件读不了交 None，键不在交 None。"""
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except OSError:
        return None
    found = None
    for line in lines:
        stripped = line.strip()
        if not stripped or stripped.startswith("#") or "=" not in stripped:
            continue
        name, value = stripped.split("=", 1)
        if name.strip() != key:
            continue
        value = value.strip()
        if len(value) >= 2 and value[0] == value[-1] and value[0] in "\"'":
            value = value[1:-1]
        found = value
    return found


def connection_override_problem(environment=None):
    """改连接地址与时限的变量（OVERRIDE_VARIABLES）设了而不在临时服务底下（TEMPORARY_SERVICE_VARIABLE 没设）交一句为什么，别的交空串
    （文件头「连哪一处」）。environment 不给照本进程的 os.environ。弄坏开关 connection-variables-honored-outside-temporary-service 下一律交空串。"""
    environment = os.environ if environment is None else environment
    if break_is("connection-variables-honored-outside-temporary-service") or environment.get(TEMPORARY_SERVICE_VARIABLE, ""):
        return ""
    overriding = [name for name in OVERRIDE_VARIABLES if environment.get(name, "")]
    if not overriding:
        return ""
    return (f"{'、'.join(f'{name}={environment[name]}' for name in overriding)} 只在临时服务底下认（{TEMPORARY_SERVICE_VARIABLE} 没设）："
            "平常跑改连接地址等于另连一份判定库、时限取常量，一条都没连")


def socket_path():
    configured = os.environ.get(SOCKET_VARIABLE, "")
    if configured:
        return configured
    runtime = os.environ.get("XDG_RUNTIME_DIR", "")
    return os.path.join(runtime, "singlefs-prune-server", "server.sock") if runtime else ""


def network_address():
    configured = os.environ.get(TCP_ADDRESS_VARIABLE, "")
    if configured:
        return configured, f"环境变量 {TCP_ADDRESS_VARIABLE}"
    for path in multi_host_configuration_candidates():
        value = configuration_value(path, MULTI_HOST_ADDRESS_KEY)
        if value:
            return value, f"{path} 的 {MULTI_HOST_ADDRESS_KEY}"
    # 不读环境变量：配置里没有这一键就由调用方报错（用户定「没有配置 就报错 不允许自动补齐」）
    return "", ""


def choose_route():
    """("unix", 路径) 或 ("tcp", (主机, 端口))；两处都没有退 3，不在临时服务底下设了改连接地址或时限的变量也退 3（connection_override_problem）。"""
    problem = connection_override_problem()
    if problem:
        fail("connection-override", problem)
    path = socket_path()
    if path and os.path.exists(path):
        return "unix", path
    address, source = network_address()
    if not address:
        fail("configuration", f"没有 socket 文件（{path or 'XDG_RUNTIME_DIR 没设'}），也没有 TCP 地址（{TCP_ADDRESS_VARIABLE} 没设，"
                              f"多机配置 {'、'.join(multi_host_configuration_candidates())} 里没有 {MULTI_HOST_ADDRESS_KEY}）")
    host, separator, port_text = address.rpartition(":")
    if not separator or not host or not port_text.isdigit():
        fail("configuration", f"TCP 地址 {address!r}（{source}）要写成 <地址>:<端口>")
    return "tcp", (host.strip("[]"), int(port_text))


# ── 帧 ──

def encode_frame(body):
    return struct.pack("<I", len(body)) + body


def request_body(sequence, operation, payload):
    return struct.pack("<QB", sequence, operation) + payload


def length_prefixed(data):
    return struct.pack("<I", len(data)) + data


def short_text(text):
    data = text.encode("utf-8", "replace")[:MAXIMUM_ANNOUNCEMENT_TEXT_BYTES]
    while data:
        try:
            data.decode("utf-8")
            break
        except UnicodeDecodeError:
            data = data[:-1]
    return struct.pack("<B", len(data)) + data


class ConnectionBroken(Exception):
    """这条连接用不了了（断了、超时、回应帧校验不对、服务整条连接拒了）：关掉重连重发。"""


class AnswerOverFrameLimit(Exception):
    """成批点读这一帧被服务整帧拒成「内容解不开」（读回的值合计会超过帧长上限）：连接照用，由 read_frame_halving 对半拆键再问（文件头「容错」）。"""


class Cursor:
    def __init__(self, data):
        self.data = data
        self.position = 0

    def take(self, count, what):
        if self.position + count > len(self.data):
            fail("protocol", f"回应里读{what}要 {count} 字节，只剩 {len(self.data) - self.position} 字节")
        piece = self.data[self.position:self.position + count]
        self.position += count
        return piece

    def unpack(self, form, what):
        return struct.unpack(form, self.take(struct.calcsize(form), what))[0]

    def length_prefixed(self, what):
        return self.take(self.unpack("<I", what + "的长度"), what)

    def finished(self):
        if self.position != len(self.data):
            fail("protocol", f"回应解完之后还多 {len(self.data) - self.position} 字节")


def read_exactly(connection, count):
    chunks, remaining = [], count
    while remaining:
        try:
            chunk = connection.recv(remaining)
        except socket.timeout as error:
            raise ConnectionBroken(f"等回应超时（{error}）") from error
        except OSError as error:
            raise ConnectionBroken(f"读回应出错：{error}") from error
        if not chunk:
            raise ConnectionBroken("服务关了连接")
        chunks.append(chunk)
        remaining -= len(chunk)
    return b"".join(chunks)


def read_response(connection):
    """读一整帧回应、核长度：交 (序号, 状态, 操作码, 内容)。"""
    length = struct.unpack("<I", read_exactly(connection, FRAME_LENGTH_FIELD_BYTES))[0]
    if length < RESPONSE_HEADER_BYTES or length > MAXIMUM_FRAME_BODY_BYTES:
        raise ConnectionBroken(f"回应帧体长度 {length} 不对")
    body = read_exactly(connection, length)
    sequence, status, operation = struct.unpack("<QBB", body[:RESPONSE_HEADER_BYTES])
    return sequence, status, operation, body[RESPONSE_HEADER_BYTES:]


def announcement_payload():
    return (short_text(socket.gethostname()) + short_text(os.environ.get("TASK_SERVER_TASK", ""))
            + short_text(os.environ.get("TASK_SERVER_ATTEMPT", "")) + struct.pack("<I", os.getpid() & 0xFFFFFFFF))


# ── 客户端 ──

class Client:
    """一个进程一条连接、一问一答（文件头「容错」）。线程之间用锁排队；fork 出来的子进程第一次用时自己另连。"""

    def __init__(self):
        self.lock = threading.Lock()
        self.process = os.getpid()
        self.connection = None
        self.route = None
        self.next_sequence = 1
        self.identity = f"python:{socket.gethostname()}:{os.getpid()}:{os.urandom(4).hex()}".encode()[:MAXIMUM_CLIENT_IDENTITY_BYTES]
        self.next_request_number = 1
        self.busy_responses = 0
        self.busy_waits = []   # (服务给的毫秒数, 真等了的秒数)，自证核「等够了再重发」
        self.frame_limit = MAXIMUM_FRAME_BODY_BYTES   # 发帧之前核的帧体上限；只有自证把它放宽，照发给服务看服务那一侧的拒绝

    def close(self):
        if self.connection is not None:
            with contextlib.suppress(OSError):
                self.connection.close()
        self.connection = None

    def open_connection(self, deadline):
        """连一次、报到。连接与报到只用到 deadline（time.monotonic() 的读数：这一轮重连的总时长上限）剩下的时间，
        不给满一份等回应的超时；连上之后换回等回应的超时（照 prune-server/src/client.rs 文件头「建连有期限」）。"""
        kind, address = choose_route()
        timeout = positive_seconds(RESPONSE_TIMEOUT_VARIABLE, RESPONSE_TIMEOUT_SECONDS)
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise ConnectionBroken(f"连 {kind} {address}：到了重连总时长上限")
        if kind == "unix":
            connection = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        else:
            connection = socket.socket(socket.AF_INET6 if ":" in address[0] else socket.AF_INET, socket.SOCK_STREAM)
            connection.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
            connection.setsockopt(socket.SOL_SOCKET, socket.SO_KEEPALIVE, 1)
        given_full_timeout = break_is("connect-given-full-response-timeout")
        connection.settimeout(timeout if given_full_timeout else min(timeout, remaining))
        try:
            connection.connect(address)
            connection.settimeout(timeout if given_full_timeout else min(timeout, max(deadline - time.monotonic(), 0.001)))
            connection.sendall(encode_frame(request_body(CONNECTION_LEVEL_SEQUENCE, OPERATION_ANNOUNCEMENT, announcement_payload())))
            connection.settimeout(timeout)
        except OSError as error:
            with contextlib.suppress(OSError):
                connection.close()
            raise ConnectionBroken(f"连 {kind} {address}：{error}") from error
        self.connection, self.route = connection, f"{kind}:{address if kind == 'unix' else '%s:%d' % address}"

    def exchange(self, operation, payload, what, oversized_answer_splittable=False):
        """发一帧、等它的回应：交回应内容（状态 0 的）。容错照文件头「容错」；what 说这是哪一帧（帧长超限时报出来）。
        oversized_answer_splittable 只由 read_frame_halving 给（成批点读、键不止一把）：服务整帧拒成「内容解不开」时抛 AnswerOverFrameLimit、不退出。"""
        with self.lock:
            if self.process != os.getpid():
                self.connection, self.process = None, os.getpid()
            return self.exchange_locked(operation, payload, what, oversized_answer_splittable)

    def exchange_locked(self, operation, payload, what, oversized_answer_splittable):
        reconnect_limit = positive_seconds(RECONNECT_LIMIT_VARIABLE, RECONNECT_LIMIT_SECONDS)
        unbounded = break_is("reconnect-unbounded")
        trouble_started, trouble_count, delay = None, 0, RECONNECT_INITIAL_SECONDS
        busy_count = 0
        body_bytes = REQUEST_HEADER_BYTES + len(payload)
        oversize_unchecked = break_is("oversize-frame-sent-and-retried")
        if body_bytes > self.frame_limit and not oversize_unchecked:
            fail("frame-too-large", f"{what}：帧体 {body_bytes} 字节，超过帧长上限 {self.frame_limit} 字节；没发（服务会拒整条连接，重发照样拒）")
        while True:
            connect_started = None
            try:
                if self.connection is None:
                    connect_started = time.monotonic()
                    budget_started = connect_started if unbounded or trouble_started is None else trouble_started
                    self.open_connection(budget_started + reconnect_limit)
                    connect_started = None
                sequence = self.next_sequence
                self.next_sequence += 1
                try:
                    self.connection.sendall(encode_frame(request_body(sequence, operation, payload)))
                except OSError as error:
                    if not oversize_unchecked:
                        self.fail_on_left_frame_length_rejection(what, body_bytes)
                    raise ConnectionBroken(f"发帧出错：{error}") from error
                while True:
                    answered, status, answered_operation, content = read_response(self.connection)
                    if answered == sequence or answered == CONNECTION_LEVEL_SEQUENCE:
                        break
                    if answered > sequence:
                        fail("protocol", f"回应的序号 {answered} 比在等的 {sequence} 还大")
                    # 序号更小的是重连之前发出的帧迟到的回应：一问一答下不会有，记不上就丢
                if status == STATUS_COMPLETED and answered == sequence:
                    return content
                if status == STATUS_BUSY:
                    retry_milliseconds = Cursor(content).unpack("<I", "多少毫秒后再来")
                    busy_count += 1
                    self.busy_responses += 1
                    if busy_count >= BUSY_RETRIES_LIMIT:
                        fail("busy-limit", f"同一帧连着忙了 {busy_count} 次（{self.route}）")
                    pause = 0.0 if break_is("busy-retry-without-delay") else retry_milliseconds / 1000.0 * (1.0 + random.random() * 0.5)
                    started = time.monotonic()
                    time.sleep(pause)
                    self.busy_waits.append((retry_milliseconds, time.monotonic() - started))
                    continue
                message = self.rejection_message(content)
                if status == STATUS_SERVER_STOPPING:
                    if break_is("stop-notice-retried"):
                        raise ConnectionBroken(f"服务停了（弄坏开关 stop-notice-retried：当连接断了）：{message}")
                    self.close()
                    fail("stopped", f"服务停了（{self.route}）：{message}")
                if answered == CONNECTION_LEVEL_SEQUENCE and status == STATUS_FRAME_LENGTH_REJECTED:
                    if oversize_unchecked:
                        raise ConnectionBroken(f"服务拒了整条连接（弄坏开关 oversize-frame-sent-and-retried：当连接断了）：{message}")
                    self.close()
                    fail("frame-too-large", f"{what}：服务回「帧长度不对」并关了连接（这一帧帧体 {body_bytes} 字节，"
                                            f"上限 {MAXIMUM_FRAME_BODY_BYTES} 字节）：{message}")
                if status == STATUS_MALFORMED_PAYLOAD and oversized_answer_splittable:
                    raise AnswerOverFrameLimit(message)
                if status in (STATUS_MALFORMED_PAYLOAD, STATUS_UNKNOWN_OPERATION, STATUS_STORAGE_FAILED):
                    fail("rejected", f"服务拒了操作码 {operation} 的帧（状态 {status}）：{message}")
                fail("protocol", f"认不得的回应：序号 {answered} 状态 {status} 操作码 {answered_operation}")
            except ConnectionBroken as error:
                self.close()
                now = time.monotonic()
                # 建连那一步出的事从开始连的那一刻算起（连的时候已经花掉的时间算进总时长上限），别的从出事的那一刻算起
                trouble_started = trouble_started if trouble_started is not None else (connect_started if connect_started is not None else now)
                trouble_count += 1
                elapsed = now - trouble_started
                if not unbounded and (trouble_count >= RECONNECT_ATTEMPTS_LIMIT or elapsed >= reconnect_limit):
                    fail("unreachable", f"连不上 prune-server：试了 {trouble_count} 次、{elapsed:.1f} 秒（上限 {RECONNECT_ATTEMPTS_LIMIT} 次、"
                                        f"{reconnect_limit:g} 秒）；最后一次：{error}")
                time.sleep(max(0.0, min(delay, reconnect_limit - elapsed)) * (0.5 + random.random() * 0.5))
                delay = min(delay * 2, RECONNECT_CEILING_SECONDS)

    def fail_on_left_frame_length_rejection(self, what, body_bytes):
        """发帧出错时（服务拒了帧长、回完就关了连接，写到一半断了），读出还躺在接收缓冲里的回应：是「帧长度不对」就当场退 3（文件头「容错」单条超限）。
        读不出、或读出的是别的，交回去由调用方照连接断了办。"""
        with contextlib.suppress(OSError, ConnectionBroken):
            self.connection.settimeout(REJECTION_DRAIN_SECONDS)
            answered, status, _operation, content = read_response(self.connection)
            if answered == CONNECTION_LEVEL_SEQUENCE and status == STATUS_FRAME_LENGTH_REJECTED:
                message = self.rejection_message(content)
                self.close()
                fail("frame-too-large", f"{what}：发帧时服务回了「帧长度不对」并关了连接（这一帧帧体 {body_bytes} 字节，"
                                        f"上限 {MAXIMUM_FRAME_BODY_BYTES} 字节）：{message}")

    @staticmethod
    def rejection_message(content):
        try:
            cursor = Cursor(content)
            return cursor.length_prefixed("说明").decode("utf-8", "replace")
        except PruneServerFailure:
            return content[:200].decode("utf-8", "replace")

    # ── 操作 ──

    def read_many(self, keys):
        """成批点读：交与 keys 同序的值（bytes 或 None），分段存的拼回整值（文件头「键空间」大值分段）。读失败、拼不回都退出，不交「不在」。"""
        return [self.assembled(key, value, {}) for key, value in zip(keys, self.read_many_raw(keys))]

    def read_many_raw(self, keys):
        """成批点读服务里的原样：分段存的键交清单本身，不拼。按请求字节切帧，一帧的回应超过帧长上限时对半拆键（read_frame_halving）。"""
        values = []
        for batch in batches_of(keys, lambda key: len(key) + 5):
            try:
                values.extend(self.read_frame_halving(batch))
            except PruneServerFailure:
                if break_is("read-failure-reported-as-absent"):
                    return [None] * len(keys)
                raise
        return values

    def read_frame_halving(self, keys):
        """一帧问 keys；服务整帧拒成「内容解不开」（读回的值合计会超过帧长上限）而键不止一把，拆成前后两半各自照这样问
        （递归，深度不超过键数的二进制位数），按原次序接起来；单键还被拒照「服务拒了帧」退 3（文件头「容错」回应超过帧长上限就拆键）。
        弄坏开关 oversized-answer-read-unsplit 下不拆、照拒退 3（改前的样子）。"""
        splittable = len(keys) > 1 and not break_is("oversized-answer-read-unsplit")
        try:
            return self.read_frame(keys, splittable)
        except AnswerOverFrameLimit:
            half = len(keys) // 2
            return self.read_frame_halving(keys[:half]) + self.read_frame_halving(keys[half:])

    def read_frame(self, keys, oversized_answer_splittable):
        """一帧成批点读：交与 keys 同序的原样值（bytes 或 None）。"""
        payload = struct.pack("<I", len(keys)) + b"".join(struct.pack("<BI", TABLE_CONCLUSIONS, len(key)) + key for key in keys)
        content = self.exchange(OPERATION_POINT_READ, payload, f"成批点读 {len(keys)} 个键（最长的键 {max(len(key) for key in keys)} 字节）",
                                oversized_answer_splittable)
        cursor = Cursor(content)
        count = cursor.unpack("<I", "值的个数")
        if count != len(keys):
            fail("protocol", f"成批点读发了 {len(keys)} 个键，回了 {count} 个值")
        values = []
        for _ in range(count):
            present = cursor.unpack("<B", "在不在")
            if present == 1:
                values.append(bytes(cursor.length_prefixed("值")))
            elif present == 0:
                values.append(None)
            else:
                fail("protocol", f"「在不在」只能是 0 或 1，收到 {present}")
        cursor.finished()
        return values

    def write_frame(self, entries, request_number=None):
        """一帧成批写（一个写批，WAL 同步之后才回）：交 (写了几条, 早已生效)。request_number 只给自证造重发。"""
        if request_number is None:
            request_number = self.next_request_number
            self.next_request_number += 1
        payload = (struct.pack("<B", len(self.identity)) + self.identity + struct.pack("<QI", request_number, len(entries))
                   + b"".join(struct.pack("<BI", TABLE_CONCLUSIONS, len(key)) + key + length_prefixed(value) for key, value in entries))
        what = (f"成批写 {len(entries)} 条（最长的键 {max((len(key) for key, _value in entries), default=0)} 字节、"
                f"最大的值 {max((len(value) for _key, value in entries), default=0)} 字节）")
        cursor = Cursor(self.exchange(OPERATION_WRITE, payload, what))
        written = cursor.unpack("<I", "写了几条")
        already = cursor.unpack("<B", "早已生效")
        cursor.finished()
        return written, already == 1

    def write_batches(self, entries):
        """[(键, 值)] 照原样按帧上限切成几帧写，不分段；同一帧里的原子。"""
        for batch in batches_of(entries, lambda entry: len(entry[0]) + len(entry[1]) + 9):
            written, _already = self.write_frame(batch)
            if written != len(batch):
                fail("protocol", f"写了 {len(batch)} 条，服务说写了 {written} 条")

    def write_many(self, entries):
        """[(键, 值)] 按帧上限切成几帧写；同一帧里的原子（文件头「键空间」：一份判定与它的 items 放同一帧）。
        大值分段存（文件头「键空间」大值分段）：各段与计数键先写，清单与别的条目最后照原来的切法写。"""
        entries = list(entries)
        for key, _value in entries:
            if is_reserved_original_key(key):
                fail("reserved-key", f"键 {shown_key(key)!r} 的键尾是段键或计数键的形状，这两种形状留给大值分段；这一批 {len(entries)} 条一条都没写")
        longest = max((len(key) for key, _value in entries), default=0)
        smallest_frame = REQUEST_HEADER_BYTES + 4 + 5 + longest + len(SEGMENT_KEY_SEPARATOR + SEGMENT_COUNT_SUFFIX)
        if smallest_frame > self.frame_limit and not break_is("oversize-frame-sent-and-retried"):
            fail("frame-too-large", f"成批写 {len(entries)} 条：最长的键 {longest} 字节，连点读它的计数键的一帧都要 {smallest_frame} 字节，"
                                    f"超过帧长上限 {self.frame_limit} 字节；没发（键分不了段，重发照样拒）")
        unsegmented = break_is("large-value-sent-unsegmented")
        high_water = {} if unsegmented or break_is("segment-count-shrinks-on-rewrite") else self.segment_high_water([key for key, _value in entries])
        segment_entries, final_entries = [], []
        for key, value in entries:
            previous_count = high_water.get(key, 0)
            if unsegmented or not (previous_count or needs_segments(value)):
                final_entries.append((key, value))
                continue
            count = max(segments_needed(value), previous_count)
            if count > 10 ** SEGMENT_INDEX_DIGITS and not break_is("segment-count-unbounded"):
                fail("too-many-segments", f"{shown_key(key)} 的值 {len(value)} 字节要分 {count} 段，段号只有 {SEGMENT_INDEX_DIGITS} 位十进制"
                                          f"（至多 {10 ** SEGMENT_INDEX_DIGITS} 段）；这一批 {len(entries)} 条一条都没写")
            lengths = []
            for index in range(count):
                piece = value[index * SEGMENT_BYTES:(index + 1) * SEGMENT_BYTES]
                segment_entries.append((segment_key(key, index), piece))
                lengths.append(len(piece))
            segment_entries.append((segment_count_key(key), b"%d" % count))
            final_entries.append((key, encode_manifest(value, lengths)))
        self.write_batches(segment_entries)
        self.write_batches(final_entries)

    def segment_high_water(self, keys):
        """{键: 计数键里记的段数}，只交分段存过的键；计数键不是十进制当读错（文件头「键空间」大值分段）。"""
        found = {}
        for key, value in zip(keys, self.read_many_raw([segment_count_key(key) for key in keys])):
            if value is None:
                continue
            if not value.isdigit():
                fail("segmented-read", f"{shown_key(key)} 的计数键不是十进制段数：{value[:80]!r}")
            found[key] = int(value)
        return found

    def assembled(self, key, value, segments):
        """原键读回的 value 是分段清单就拼回整值（segments 是已经扫到的 {段号: 段}，缺的逐帧点读），核每段长度与整值 sha256；
        不是清单照原样交。拼不回照 SEGMENTED_REREAD_DELAYS_SECONDS 逐次等、重读原键：清单换过就照新清单重拼（已经扫到的段丢掉），原键不再是清单就交它；
        等完清单没换退 3（文件头「键空间」大值分段「读」）。"""
        manifest = manifest_of(key, value)
        if manifest is None:
            return value
        delays = iter(() if break_is("segmented-read-not-reread") else SEGMENTED_REREAD_DELAYS_SECONDS)
        while True:
            whole, problem = self.assembled_by_manifest(key, manifest, segments)
            if problem is None:
                return whole
            while True:
                delay = next(delays, None)
                if delay is None:
                    fail("segmented-read", problem)
                time.sleep(delay)
                current = self.read_many_raw([key])[0]
                if current is None:
                    fail("segmented-read", f"{problem}；重读时原键不在了")
                replacement = manifest_of(key, current)
                if replacement != manifest:
                    break
            if replacement is None:
                return current
            manifest, segments = replacement, {}

    def assembled_by_manifest(self, key, manifest, segments):
        """照一份清单拼回整值：交 (整值, None)，拼不回交 (None, 为什么)。"""
        lengths = manifest["segments"]
        fetched = {index: segments[index] for index in range(len(lengths)) if index in segments}
        missing = [index for index in range(len(lengths)) if index not in fetched]
        for group in batches_of(missing, lambda index: lengths[index] + len(key) + 16):
            for index, piece in zip(group, self.read_many_raw([segment_key(key, index) for index in group])):
                if piece is not None:
                    fetched[index] = piece
        pieces = []
        for index, wanted in enumerate(lengths):
            piece = fetched.get(index)
            if piece is None:
                return None, f"{shown_key(key)} 分 {len(lengths)} 段存，段号 {index} 不在服务里（拼不全）"
            if len(piece) != wanted:
                return None, f"{shown_key(key)} 的段号 {index} 长 {len(piece)} 字节，清单写的是 {wanted} 字节"
            pieces.append(piece)
        whole = b"".join(pieces)
        if not break_is("segments-assembled-without-sha256") and hashlib.sha256(whole).hexdigest() != manifest["sha256"]:
            return None, f"{shown_key(key)} 拼回的 {len(whole)} 字节 sha256 是 {hashlib.sha256(whole).hexdigest()}，清单写的是 {manifest['sha256']}"
        return whole, None

    def raw_scan(self, prefix):
        payload = struct.pack("<BI", TABLE_CONCLUSIONS, len(prefix)) + prefix + struct.pack("<I", SCAN_ENTRIES_LIMIT)
        cursor = Cursor(self.exchange(OPERATION_PREFIX_SCAN, payload, f"按前缀扫（前缀 {len(prefix)} 字节）"))
        truncated = cursor.unpack("<B", "截断标记")
        count = cursor.unpack("<I", "条数")
        entries = [(bytes(cursor.length_prefixed("键")), bytes(cursor.length_prefixed("值"))) for _ in range(count)]
        cursor.finished()
        return entries, truncated == 1

    def raw_scan_stream(self, prefix):
        """按前缀逐帧扫服务里的原样（段键、清单都照交），逐条交 (键, 值)、按键升序：一帧被帧长上限截断时按下一个字节拆成更窄的前缀接着扫
        （前缀本身那个键另点读）；任何时刻手里只有一帧。"""
        entries, truncated = self.raw_scan(prefix)
        if not truncated:
            yield from entries
            return
        if not entries and not break_is("scan-row-over-frame-split"):
            fail("scan-row-over-frame", f"前缀 {shown_key(prefix)} 下扫回来一行都没有却说被截断：头一行的值就装不下一帧")
        del entries
        exact = self.read_many_raw([prefix])[0]
        if exact is not None:
            yield prefix, exact
        for byte_value in range(SCAN_SPLIT_BYTE_VALUES):
            yield from self.raw_scan_stream(prefix + bytes([byte_value]))

    def scan_stream(self, prefix):
        """按前缀扫全部，逐条交 (键, 值)、按键升序：段键与计数键的形状不交，清单那一条交拼回的整值（文件头「键空间」大值分段）。
        同一时刻只攒一把键的段：紧跟着它扫到的段收下，中间隔了别的原键就当场交（缺的段由 assembled 点读），之后扫到它的段照形状跳过。"""
        owner, manifest_value, segments = None, None, {}
        for key, value in self.raw_scan_stream(prefix):
            if break_is("nul-keys-reserved-whole") and SEGMENT_KEY_SEPARATOR in key:
                parent, _separator, suffix = key.partition(SEGMENT_KEY_SEPARATOR)
                if parent == owner and len(suffix) == SEGMENT_INDEX_DIGITS and suffix.isdigit():
                    segments[int(suffix)] = value
                continue
            parts = segment_key_parts(key)
            if parts is not None and owner is not None and parts[0] == owner:
                segments[parts[1]] = value
                continue
            if (parts is not None or has_count_key_shape(key)) and not break_is("other-segment-keys-scanned-as-entries"):
                continue
            if owner is not None:
                yield owner, self.assembled(owner, manifest_value, segments)
                owner, manifest_value, segments = None, None, {}
            if value.startswith(SEGMENT_MANIFEST_MAGIC):
                owner, manifest_value = key, value
            else:
                yield key, value
        if owner is not None:
            yield owner, self.assembled(owner, manifest_value, segments)

    def scan(self, prefix):
        """按前缀扫全部，交按键升序的 [(键, 值)]（scan_stream 攒成一份）。"""
        return list(self.scan_stream(prefix))

    def delete_prefix_frame(self, prefix, after_key, count_only):
        """删前缀一帧（prune-server/src/protocol.rs 文件头「删前缀」）：交 (这一帧数到或删掉几把, 前缀下是不是已经没有更多的键, 最后一把键)。
        after_key 空是从前缀起，不空是从排在它之后的第一把键起；不认表码、不认键的种类。空前缀与落在服务记账那一段的前缀服务回状态 1（退 3）。"""
        payload = (length_prefixed(prefix) + length_prefixed(after_key) + struct.pack("<IB", DELETE_PREFIX_KEYS_PER_FRAME, 1 if count_only else 0))
        cursor = Cursor(self.exchange(OPERATION_DELETE_PREFIX, payload, f"{'数' if count_only else '删'}前缀（前缀 {len(prefix)} 字节）"))
        keys = cursor.unpack("<I", "这一帧碰到几把键")
        exhausted = cursor.unpack("<B", "前缀下没有更多标记")
        last_key = bytes(cursor.length_prefixed("最后一把键"))
        cursor.finished()
        if exhausted not in (0, 1):
            fail("protocol", f"前缀下没有更多标记只能是 0 或 1，收到 {exhausted}")
        return keys, exhausted == 1, last_key

    def delete_prefix(self, prefix, count_only=False):
        """数或删 prefix 下的全部键，一帧一帧接着来（每帧的续扫起点是上一帧回的最后一把键）到前缀下没有更多的键为止，交总数。
        count_only 为真只数不删（干跑），数的口径和删一样。删本来就幂等：断线重连之后重发的那一帧只会让它报的数少一批，库里的结果不变。"""
        total, after_key = 0, b""
        while True:
            keys, exhausted, last_key = self.delete_prefix_frame(prefix, after_key, count_only)
            total += keys
            if exhausted:
                return total
            if not last_key:
                fail("protocol", "删前缀说前缀下还有键，却没回最后一把键：续不下去")
            after_key = last_key

    def statistics(self):
        cursor = Cursor(self.exchange(OPERATION_STATISTICS, b"", "统计"))
        counters = {}
        for _ in range(cursor.unpack("<I", "计数的个数")):
            name = cursor.length_prefixed("计数名").decode("utf-8", "replace")
            counters[name] = cursor.unpack("<Q", "计数值")
        cursor.finished()
        return counters


def batches_of(items, size_of):
    batch, size = [], 0
    for item in items:
        item_size = size_of(item)
        if batch and (size + item_size > FRAME_BATCH_BYTES or len(batch) >= FRAME_BATCH_KEYS):
            yield batch
            batch, size = [], 0
        batch.append(item)
        size += item_size
    if batch:
        yield batch


# ── 大值分段（文件头「键空间」大值分段）──

def shown_key(key):
    return key.decode("utf-8", "replace")[:SHOWN_KEY_CHARACTERS]


def needs_segments(value):
    return len(value) > SEGMENT_BYTES or value.startswith(SEGMENT_MANIFEST_MAGIC)


def segments_needed(value):
    return max(1, -(-len(value) // SEGMENT_BYTES))


def segment_key(key, index):
    return key + SEGMENT_KEY_SEPARATOR + b"%0*d" % (SEGMENT_INDEX_DIGITS, index)


def segment_count_key(key):
    return key + SEGMENT_KEY_SEPARATOR + SEGMENT_COUNT_SUFFIX


def segment_key_parts(key):
    """键是段键的形状（键尾 NUL 加 SEGMENT_INDEX_DIGITS 位十进制）交 (原键, 段号)，不是交 None（文件头「键空间」认段键只认形状）。"""
    tail_start = len(key) - len(SEGMENT_KEY_SEPARATOR) - SEGMENT_INDEX_DIGITS
    if tail_start < 0:
        return None
    separator, digits = key[tail_start:tail_start + len(SEGMENT_KEY_SEPARATOR)], key[tail_start + len(SEGMENT_KEY_SEPARATOR):]
    if separator != SEGMENT_KEY_SEPARATOR or not digits.isdigit():
        return None
    return key[:tail_start], int(digits)


def has_count_key_shape(key):
    """键是计数键的形状（键尾 NUL 加 SEGMENT_COUNT_SUFFIX）。"""
    return key.endswith(SEGMENT_KEY_SEPARATOR + SEGMENT_COUNT_SUFFIX)


def is_reserved_original_key(key):
    """write_many 不收的原键：段键或计数键的形状；弄坏开关 segment-key-shapes-accepted 下一把都不拒，nul-keys-reserved-whole 下拒一切带 NUL 的（改前的样子）。"""
    if break_is("segment-key-shapes-accepted"):
        return False
    if break_is("nul-keys-reserved-whole"):
        return SEGMENT_KEY_SEPARATOR in key
    return segment_key_parts(key) is not None or has_count_key_shape(key)


def encode_manifest(value, lengths):
    document = {"length": len(value), "sha256": hashlib.sha256(value).hexdigest(), "segments": lengths}
    return SEGMENT_MANIFEST_MAGIC + json.dumps(document, sort_keys=True, separators=(",", ":")).encode("ascii")


def manifest_of(key, value):
    """value 是分段清单交 {"length", "sha256", "segments"}，不是（None 或不以 SEGMENT_MANIFEST_MAGIC 起头）交 None；清单解不开当读错退 3。"""
    if value is None or not value.startswith(SEGMENT_MANIFEST_MAGIC):
        return None
    try:
        manifest = json.loads(value[len(SEGMENT_MANIFEST_MAGIC):].decode("utf-8"))
        lengths, digest = manifest["segments"], manifest["sha256"]
        valid = (isinstance(lengths, list) and all(type(length) is int and 0 <= length <= SEGMENT_BYTES for length in lengths)
                 and type(manifest["length"]) is int and sum(lengths) == manifest["length"] and isinstance(digest, str) and len(digest) == 64)
    except (UnicodeDecodeError, ValueError, KeyError, TypeError):
        valid = False
    if not valid:
        fail("segmented-read", f"{shown_key(key)} 的分段清单解不开：{value[:160]!r}")
    return manifest


PROCESS_CLIENT = {}


def process_client():
    """这个进程的那一个 Client（fork 之后子进程里换一个新的）。"""
    client = PROCESS_CLIENT.get("client")
    if client is None or client.process != os.getpid():
        client = Client()
        PROCESS_CLIENT["client"] = client
    return client


def reset_process_client():
    """自证换了一份服务时丢掉旧连接。"""
    client = PROCESS_CLIENT.pop("client", None)
    if client is not None:
        client.close()


# ── 键空间 ──

def family_name(family):
    if family not in FAMILIES:
        raise ValueError(f"认不得的族 {family!r}，只认 {'、'.join(FAMILIES)}")
    return SHARED_FAMILY_NAME_UNDER_BREAK if break_is("family-prefixes-shared") else family


def namespace_of(library_directory, environment=None):
    """在临时服务底下（environment 里 TEMPORARY_SERVICE_VARIABLE 设着，不给照本进程的 os.environ）交「@<库目录的 sha256 前 16 位>」，
    不在临时服务底下交空串（文件头「键空间」命名空间）。弄坏开关 namespace-by-temporary-directory 下照改前按库目录在不在临时目录底下给。"""
    environment = os.environ if environment is None else environment
    real = os.path.realpath(library_directory)
    digest = "@" + hashlib.sha256(real.encode("utf-8", "surrogateescape")).hexdigest()[:NAMESPACE_DIGEST_CHARACTERS]
    if break_is("namespace-by-temporary-directory"):
        for root in sorted({os.path.realpath(path) for path in TEMPORARY_ROOTS_UNDER_BREAK + (tempfile.gettempdir(),)}):
            if real == root or real.startswith(root.rstrip(os.sep) + os.sep):
                return digest
        return ""
    return digest if environment.get(TEMPORARY_SERVICE_VARIABLE, "") else ""


class LibraryStore:
    """一家一个种类在服务里的那一份（文件头「键空间」）：读写都按相对路径（「/」分隔）给。"""

    def __init__(self, family, kind, library_directory):
        self.family = family
        self.kind = kind
        self.prefix = f"{family_name(family)}{namespace_of(library_directory)}/{kind}/".encode("utf-8", "surrogateescape")

    def key(self, relative):
        return self.prefix + relative.encode("utf-8", "surrogateescape")

    def location(self):
        return "prune-server:" + self.prefix.decode("utf-8", "surrogateescape")

    def read(self, relative):
        return process_client().read_many([self.key(relative)])[0]

    def read_many(self, relatives):
        return process_client().read_many([self.key(relative) for relative in relatives])

    def write(self, relative, data):
        self.write_many([(relative, data)])

    def write_many(self, pairs):
        process_client().write_many([(self.key(relative), data if isinstance(data, bytes) else data.encode("utf-8", "surrogateescape"))
                                     for relative, data in pairs])

    def delete_prefix(self, relative_prefix, count_only=False):
        """删这一家这个种类底下相对路径以 relative_prefix 起头的全部键（连它们的段键与计数键），交条数；count_only 为真只数不删。
        键照 key() 拼，临时服务底下带命名空间，只动这一份库目录自己的键。弄坏开关 delete-prefix-without-namespace 下前缀不带命名空间（改前的样子）。"""
        if break_is("delete-prefix-without-namespace"):
            prefix = f"{family_name(self.family)}/{self.kind}/".encode("utf-8", "surrogateescape") + relative_prefix.encode("utf-8", "surrogateescape")
        else:
            prefix = self.key(relative_prefix)
        return process_client().delete_prefix(prefix, count_only)

    def scan(self, relative_prefix):
        """[(相对路径, 值)]，按键升序。"""
        start = len(self.prefix)
        return [(key[start:].decode("utf-8", "surrogateescape"), value) for key, value in process_client().scan(self.key(relative_prefix))]


def library_store(family, kind, library_directory):
    """三家开库时调：交这一家这个种类的 LibraryStore（文件头「键空间」；建它不连服务，第一次读写才连）。
    弄坏开关 unreachable-treated-as-no-library 下先连一次，连不上交 None（三家照「没有判定库」全部现判，改错的样子）。"""
    store = LibraryStore(family, kind, library_directory)
    if break_is("unreachable-treated-as-no-library"):
        try:
            process_client().statistics()
        except PruneServerFailure:
            return None
    return store


def selftest_service_problem(environment=None):
    """三家与旁路的原有自证开跑前调（文件头「命令行」--with-temporary-service）：这一趟连的是那样起的临时服务交空串，不是交一句为什么。
    只看环境，不连服务、不起进程。给了 environment 只判那一份（调用方替子进程拼的环境），不给判本进程的 os.environ；
    弄坏开关 service-problem-reads-process-environment 下给了也读 os.environ（改前的样子）。"""
    if environment is None or break_is("service-problem-reads-process-environment"):
        environment = os.environ
    marked = environment.get(TEMPORARY_SERVICE_VARIABLE, "")
    if not marked:
        return f"没有临时 prune-server（{TEMPORARY_SERVICE_VARIABLE} 没设）：自证不写真服务"
    if marked != environment.get(SOCKET_VARIABLE, "") or not environment.get(TCP_ADDRESS_VARIABLE, ""):
        return f"{TEMPORARY_SERVICE_VARIABLE} 与 {SOCKET_VARIABLE}、{TCP_ADDRESS_VARIABLE} 对不上：连哪一处不全指在临时服务上"
    if not os.path.exists(marked):
        return f"临时 prune-server 的 socket {marked} 不在（服务已经停了）"
    return ""


SELFTEST_SERVICE_REMEDY = "照 scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service 那一条，在临时服务底下起这一份自证（带 --server-binary 就不编）"


# ── 命令行：check ──

def command_check():
    client = process_client()
    counters = client.statistics()
    print(f"PRUNE_SERVER_CLIENT_CHECK route={client.route} counters={len(counters)}")
    return 0


def command_read(arguments):
    """read <族> <种类> <库目录> <相对路径>（文件头「命令行」）：在就原样写到标准输出、退 0；不在退 1；用法错退 2。
    弄坏开关 command-line-read-absent-as-empty 下不在也退 0。"""
    if len(arguments) != 4 or arguments[0] not in FAMILIES:
        print(f"  ✗ read 要四个参数：<族> <种类> <库目录> <相对路径>，族只认 {'、'.join(FAMILIES)}；收到 {' '.join(arguments) or '（没给）'}", file=sys.stderr)
        print("     → 怎么办：照文件头「命令行」read 那一条写；键怎么拼看文件头「键空间」", file=sys.stderr)
        return EXIT_USAGE
    family, kind, library_directory, relative = arguments
    value = LibraryStore(family, kind, library_directory).read(relative)
    if value is None:
        return 0 if break_is("command-line-read-absent-as-empty") else 1
    sys.stdout.buffer.write(value)
    sys.stdout.flush()
    return 0


def command_delete_prefix(arguments):
    """delete-prefix [--dry-run] <族> <种类> <库目录> <相对路径前缀> 或 delete-prefix [--dry-run] --hex <十六进制的键前缀>（文件头「命令行」）：
    数或删前缀底下的全部键，末行 PRUNE_SERVER_DELETE_PREFIX mode=dry-run|deleted keys=<条数> prefix=<键前缀的十六进制>，退 0；用法错退 2；
    服务一侧的拒绝（空前缀、落在记账那一段）与连不上照客户端的退出码。--hex 的前缀是库里的原样键字节，不认表码、不认族，崩溃判定库一张表的键就是
    表标签那一个字节（例 --hex 1c）；族加种类加库目录的写法照 LibraryStore 拼键（「键空间」）。"""
    count_only = "--dry-run" in arguments
    rest = [word for word in arguments if word != "--dry-run"]
    prefixes = []
    if rest and len(rest) % 2 == 0 and all(word == "--hex" for word in rest[0::2]):
        try:
            prefixes = [bytes.fromhex(word) for word in rest[1::2]]
        except ValueError:
            prefixes = []
    elif len(rest) == 4 and rest[0] in FAMILIES:
        prefixes = [LibraryStore(rest[0], rest[1], rest[2]).key(rest[3])]
    if not prefixes or not all(prefixes):
        print(f"  ✗ delete-prefix 要 [--dry-run] <族> <种类> <库目录> <相对路径前缀>，或 [--dry-run] --hex <非空的十六进制前缀> [--hex <…>]…；族只认 {'、'.join(FAMILIES)}；"
              f"收到 {' '.join(arguments) or '（没给）'}", file=sys.stderr)
        print("     → 怎么办：照文件头「命令行」delete-prefix 那一条写；先 --dry-run 数一遍，再不带它删，再 --dry-run 数到 0", file=sys.stderr)
        return EXIT_USAGE
    total = 0
    for prefix in prefixes:
        keys = process_client().delete_prefix(prefix, count_only)
        total += keys
        print(f"PRUNE_SERVER_DELETE_PREFIX mode={'dry-run' if count_only else 'deleted'} keys={keys} prefix={prefix.hex()}", flush=True)
    if len(prefixes) > 1:
        print(f"PRUNE_SERVER_DELETE_PREFIX_TOTAL mode={'dry-run' if count_only else 'deleted'} prefixes={len(prefixes)} keys={total}")
    return 0


def read_item_name_list(path):
    """delete-by-item-name 的名单文件：制表符分隔两列「种类 项名」，空行与 # 起头的行跳过 → ({种类: {项名…}}, 问题句；没问题写空串)。"""
    names_by_kind = {}
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except OSError as error:
        return {}, f"名单文件 {path} 读不了：{error}"
    for number, line in enumerate(lines, 1):
        if not line.strip() or line.startswith("#"):
            continue
        columns = line.split("\t")
        if len(columns) != 2 or not columns[0].strip() or not columns[1].strip():
            return {}, f"名单文件 {path} 第 {number} 行不是「种类<TAB>项名」两列：{line[:80]!r}"
        names_by_kind.setdefault(columns[0], set()).add(columns[1])
    return names_by_kind, ""


def item_name_of(value):
    """一条判定记录（JSON 对象）里的 item 字段（项名）；值不是 JSON 对象或没有字符串的 item 交 None。"""
    try:
        record = json.loads(value.decode("utf-8"))
    except (UnicodeDecodeError, ValueError):
        return None
    item = record.get("item") if isinstance(record, dict) else None
    return item if isinstance(item, str) else None


def command_delete_by_item_name(arguments):
    """delete-by-item-name [--dry-run] <族> <库目录> <名单文件>（文件头「命令行」）：名单文件每行「种类<TAB>项名」；每个种类整个扫一遍（LibraryStore.scan("")）、
    读值、值是 JSON 对象且 item 字段是这个种类名单里的项名的，按完整键逐条删（delete_prefix 到这把键本身，连段键与计数键），
    每条打 PRUNE_SERVER_DELETE_BY_ITEM_NAME_ROW mode=… kind=… keys=<条数> key=<完整键> item=<项名>，末行 PRUNE_SERVER_DELETE_BY_ITEM_NAME 汇总，退 0。
    --dry-run 只数不删。同一个项名在别的种类里的记录不动。这把键的文本是同一种类里另一把原键的前缀时（删前缀会连带删掉那一把）这一条不删、点名、整次退 1。
    用法错退 2。弄坏开关 item-name-ignores-kind 下每个种类用名单里全部种类的项名；item-name-dry-run-deletes 下 --dry-run 也删。"""
    count_only = "--dry-run" in arguments
    rest = [word for word in arguments if word != "--dry-run"]
    names_by_kind, problem = read_item_name_list(rest[2]) if len(rest) == 3 else ({}, "")
    if len(rest) != 3 or rest[0] not in FAMILIES or problem or not names_by_kind:
        print(f"  ✗ delete-by-item-name 要 [--dry-run] <族> <库目录> <名单文件>，族只认 {'、'.join(FAMILIES)}，名单文件至少一行「种类<TAB>项名」；"
              f"收到 {' '.join(arguments) or '（没给）'}{'；' + problem if problem else ''}", file=sys.stderr)
        print("     → 怎么办：照文件头「命令行」delete-by-item-name 那一条写；先 --dry-run 数一遍、核对逐条打出的键与项名，再不带它删，再 --dry-run 数到 0", file=sys.stderr)
        return EXIT_USAGE
    family, library_directory, _list_path = rest
    mode = "dry-run" if count_only else "deleted"
    every_name = set().union(*names_by_kind.values())
    matched = keys_total = non_records = 0
    refused = []
    found = set()
    for kind in sorted(names_by_kind):
        store = LibraryStore(family, kind, library_directory)
        names = every_name if break_is("item-name-ignores-kind") else names_by_kind[kind]
        rows = store.scan("")
        relatives = [relative for relative, _value in rows]
        for relative, value in rows:
            item = item_name_of(value)
            if item is None:
                non_records += 1
                continue
            if item not in names:
                continue
            full_key = store.key(relative).decode("utf-8", "surrogateescape")
            shadowed = [other for other in relatives if other != relative and other.startswith(relative)]
            if shadowed:
                refused.append(full_key)
                print(f"  ✗ {full_key} 是同一种类里 {len(shadowed)} 把别的原键的前缀（例 {shadowed[0]}），删前缀会连带删掉它们，这一条没删", file=sys.stderr)
                continue
            keys = store.delete_prefix(relative, count_only and not break_is("item-name-dry-run-deletes"))
            print(f"PRUNE_SERVER_DELETE_BY_ITEM_NAME_ROW mode={mode} kind={kind} keys={keys} key={full_key} item={item}", flush=True)
            matched += 1
            keys_total += keys
            found.add((kind, item))
    missing = sorted((kind, name) for kind, names in names_by_kind.items() for name in names if (kind, name) not in found)
    for kind, name in missing:
        print(f"PRUNE_SERVER_DELETE_BY_ITEM_NAME_NOT_FOUND kind={kind} item={name}")
    print(f"PRUNE_SERVER_DELETE_BY_ITEM_NAME mode={mode} kinds={len(names_by_kind)} matched={matched} keys={keys_total} "
          f"names={sum(len(names) for names in names_by_kind.values())} names_not_found={len(missing)} non_records={non_records} refused={len(refused)}")
    return 1 if refused else 0


def main(arguments):
    if os.environ.get(BREAK_VARIABLE) and os.environ.get(SELFTEST_VARIABLE) != "1":
        print(f"  ✗ {BREAK_VARIABLE} 只在 --selftest 里认（弄坏开关只给自证证红用），这一趟不是自证", file=sys.stderr)
        print(f"     → 怎么办：unset {BREAK_VARIABLE} 再跑；要证红就跑 {BREAK_VARIABLE}=<项> python3 scripts/prune-server/prune_server_client_selftest.py --selftest", file=sys.stderr)
        return EXIT_USAGE
    if arguments[:1] in (["--selftest"], ["--selftest-families"], ["--selftest-child"], ["--with-temporary-service"]):
        print("  ✗ 自证不在这份文件里：客户端被三家判定库 import，自证另放一份", file=sys.stderr)
        print(f"     → 怎么办：python3 scripts/prune-server/prune_server_client_selftest.py {' '.join(arguments)}", file=sys.stderr)
        return EXIT_USAGE
    if arguments == ["check"]:
        return command_check()
    if arguments[:1] == ["read"]:
        return command_read(arguments[1:])
    if arguments[:1] == ["delete-prefix"]:
        return command_delete_prefix(arguments[1:])
    if arguments[:1] == ["delete-by-item-name"]:
        return command_delete_by_item_name(arguments[1:])
    print(f"  ✗ 认不出的用法：{' '.join(arguments) or '（没给参数）'}", file=sys.stderr)
    print("     → 怎么办：看文件头「命令行」：check、read、delete-prefix、delete-by-item-name；自证在 prune_server_client_selftest.py", file=sys.stderr)
    return EXIT_USAGE


if __name__ == "__main__":
    sys.dont_write_bytecode = True
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "..", ".claude", "scripts"))
    from project_preflight import preflight
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
