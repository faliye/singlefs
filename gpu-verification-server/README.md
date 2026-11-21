# gpu-verification-server

GPU 批处理验证服务：崩溃判定的后半段（卡上判）。前半段（录入、建表）在 CPU 上，由统一任务服务跑；两个服务之间只经判定库连：判段把要判的块写成待判行，本服务取走、判完在一帧里带守卫写回结论、删掉待判行。收三族：待判行（验证包表 `1D`，第 ① 段判崩溃镜像）、核对待判行（表 `1F`，第 ② 段核对事实表；「核对待判行那一族」一节）与判表待判行（表 `20`，给定状态表与观察者交回的盘的判段在任务里取不到卡时要判的整张判表）；另收判段在真服务里登记的临时库（`temporary-library` 族，照登记去连那份临时服务取这三张表的待判行、结论写回那一份；「登记的临时库」一节）。本服务管这一台的卡：起来照多机配置开全部点名的卡、一直持着，不经统一任务服务领卡。卡上判没有状态，服务只管按每张卡给服务的显存额度定每一波派多少、怎么分，不让卡 OOM。待判的块跨片、跨流进同一个池连续取；池里的态够把每张空着的卡各填满一次派活（水位线）、或最老的待判等过配置的限时，就发一波。

仓根工作区的成员，与验证包同一份 `Cargo.lock` 与 profile，path 依赖 `../crates/singlefs-verification`（`gpu` 特性常开）。clippy 的编码纪律那几条照仓根工作区走（`.claude/singlefs-ai-sop/scripts/check.sh` 的 `CODE_DISCIPLINE_LINTS`）。

| 文件 | 做什么 |
|---|---|
| `src/main.rs` | 进程本体 `gpu-verification-server serve --state-directory <目录>`：头一步换标准错误（滤逐次派活的行）、装停止信号的处理，解配置、连判定库、上一个进程没了就放掉它留下的认领、照多机配置开这一台点名的全部卡（开不成退 78）、把「在跑」那一行（进程号、进程起来的时刻、开机号、认领的持有者）写进状态目录下的 `serving`，之后跑取活循环；收到停止信号收尾、放掉自己的认领、退 0 |
| `src/library_pending_pool.rs` | 待判池的判定库那一份：不按键挑行（待判行不带期待的判器键，结论照待判行里写的结论版本键存）、在库上现算判到了哪一步（强制现判的不问库；判完而可复用的不取、带守卫删待判行）、认领与续租（认领到之后重读待判行：不在了不取、放掉刚认领到的）、放认领（上一个进程没了放它的、停的时候放自己的）、一帧写结论（判定库服务核认领、结论块那一行、删的待判行是不是读到的那一份）、守卫不过照种类重编重读再写、没写成的结论留下报红、认领与写回各打一行可数的；核对待判行那一族走同一套（「核对待判行那一族」一节）；验证包那一侧经接口 `VerificationPackageSide` |
| `src/service_process.rs` | 服务进程的身份与「在跑」那一行：下一个进程照它判上一个进程还在不在（开机号、进程号、进程起来的时刻都对得上且不是僵尸才算还活着） |
| `src/facts_verifier_family.rs` | 核对待判行那一族自己的几样：块的名字、认领键 `gpu-claim/facts-verifier/<流键>/<块号>`、上卡核的材料（事实表与每段的拆法）、这一族的行首 |
| `src/packed_tables_family.rs` | 判表待判行那一族自己的几样：一张判表的名字、认领键 `gpu-claim/packed-tables/<判表摘要>/<容器容量旋钮摘要>`、写回那一帧要的几样、写进表 `21` 的设备属性（判它的那张卡的卡型与驱动版本，不记 GPU 判器是哪一版）、这一族的行首 |
| `src/service_log.rs` | 服务进程的标准错误换成过滤管道：验证包逐次派活打的 `GPU_LIFECYCLE`、`CRASH_JUDGE_ACCOUNTING_SLICE(S)` 只计数、不进日志，别的行原样转（「日志」一节） |
| `src/verification_package_side.rs` | 验证包那一侧的真的那一份：待判行（表 `1D`）与核对待判行（表 `1F`）的扫与重读、读回材料、现算判到了哪一步与核没核过、编结论帧与核对行那一帧，一律调验证包交出的函数，服务不另写一份 |
| `src/cards_of_the_service.rs` | 服务持着的那一组卡的接口（真卡与替身卡各一份实现） |
| `src/serving_from_the_library.rs` | 取活循环：起来开卡、一直持着；扫库挑块、认领、入服务主循环、判完写回；有人借卡时不认领、手上的块写回之后借出卡键，没人借了收回（「借卡」一节）；停的时候才放掉卡；每判完一块、每隔一段每张卡各打一行汇总 |
| `src/card_lease.rs` | 借卡：请求键、卡键与等号（判定库服务的认领键）、借出与收回那两行，借卡白名单的认法，二进制的 `lease take`（先来后到排队）、`lease return <卡串>`、`lease renew`、`lease list`（「借卡」一节；用法与退出码在文件头） |
| `src/card-lease-borrowers.tsv` | 借卡白名单：`lease take` 只借给可执行文件名逐字在表里的进程；`include_str!` 编进二进制，改了要重编、`control install` 重装服务（「借卡」一节） |
| `control` | 装、起、停、查（`install`、`start`、`stop`、`check`；`unit-state` 给发起的那一台问第二台）、借卡（`lease take`、`lease return <卡串>`、`lease list`，原样转给这一台装着的二进制）与它自己的 `--selftest`；起任何一台的单元之前，发起的那一台清一次场（复用 `scripts/crash/gpu-quiesce-run.sh`；`install` 换版先停在跑的各台再清）、`check` 另判单元环境里的 `CARGO`、卡坏与判不了的块；用法、退出码与弄坏开关在文件头 |
| `singlefs-gpu-verification-server.service.in` | 用户级 systemd 单元模板（`control install` 照它写）：开机不自起、被杀由 systemd 拉起；带 `Environment=CARGO=<这一台的 cargo>` |
| `src/batch_scheduling.rs` | 批次调度（纯函数）：一波是给每张空卡排的一轮，驻留优先、新装均分、不饿死、水位线；算法写在文件头 |
| `src/card_memory_ledger.rs` | 一个判器占多少显存（与验证包 `mebibytes_a_judge_needs` 同一个算法）、一个核对器借多少显存（照它建的缓冲算）、一张卡的显存账：越界的判器进不了账，计划只经账造；核对器借之前照账核 |
| `src/pending_pool.rs` | 待判池的接口 `PendingPool` 与内存实现（服务主循环与批次调度只看它；判定库里取来的块认领之后入这一份） |
| `src/card_worker.rs` | 每张卡一条工作线程：照计划换出、装上（每装一个报一声）、按批判，结论按批交回；派来一组核对块时换出点名的判器、建核对器逐块核、整组交回；判或核的时候 panic 先问设备：显存不够而设备没丢的丢掉全部判器、在这张卡上做一次提交等它做完、报接住、照常收活；设备因显存不够丢了的重开设备、报接住、照常收活；判法自己 panic（设备没丢、没记 Out of Memory）的丢掉全部判器、报判法 panic 了、照常收活；设备为别的原因丢了（或重开没成）才报卡坏 |
| `src/gpu_judges_on_a_card.rs` | 真卡上按流的键缓存 `GpuCrashJudge`；照多机配置（`GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED`，没写取 `GPU_CARDS_BY_PRIORITY`）开这一台点名的全部卡；panic 之后读设备情况（验证包 `DeviceConditionOfACard`：丢没丢、Out of Memory 记了几次；丢了的是不是显存不够照验证包的判法认）；因显存不够丢的设备照 `nvidia-smi` 序号另开一个（每张卡至多 `DEVICE_REOPENINGS_OF_A_CARD` 次）；接住之后的那一次提交借验证包 `GpuCard::crc32c_of_ranges` 算一段空字节 |
| `src/service_loop.rs` | 服务主循环与客户端（同一进程里经通道）；核对组照卡上的显存账找空着的卡、放不下先换出驻留的判器；接住 Out of Memory（撤回这张卡上没交回结论的截、判出的结论记回收下的那一块，撤回的块别的没坏、装得下的卡照常拿，没有才等一张腾空的卡单独重判、再不够就报拒收下的那一块）；判法自己 panic 时报拒牵涉的块、卡照常用；一张卡坏了之后没有一张没坏的卡装得下的块当场拒；交出装上、接住、卡坏三件事给取活循环打行 |
| `src/service_configuration.rs`、`service-configuration.txt` | 服务的配置（凑批的限时、扫库间隔、认领租期），编译时读进来 |
| `src/registered_libraries.rs` | 登记的临时判定库：解登记键（调验证包 `TemporaryLibraryRegistration::from_claim_key`，写法归验证包）、连哪里（登记的台是这一台走 socket，否则走 TCP）、不连指到真服务与键空间前缀不合的登记；取活循环里的各份库（真库一份、登记的库若干份）：每份一个待判池、各份库的候选按入池序号合起来先来先判、登记的库读写失败只丢那一份（「登记的临时库」一节） |
| `src/break_switches.rs` | 弄坏开关 `GPU_VERIFICATION_SERVER_BREAK` |

## 起停

多机配置里点名了卡的每一台各一个进程（`GPU_CARDS_BY_PRIORITY`、`GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED` 里有它的台标签）；在这一台跑 `control`，第二台的那一份经 ssh 在同一绝对路径的仓里做。

- 装：先编（编译目录归任务服务、任务结束就删，所以编和拷到共享根在同一条提交命令里做完，整条写法在 `control` 文件头的编法与 `install` 缺二进制时的出路里），再 `python3 gpu-verification-server/control install --binary <共享根下的二进制>`。没开 GPU（`ENABLE_GPU=0`）时 `install` 退 78。`install` 换版先停、再清场、再起：发起的那一台先把单元在跑的各台停掉（这一台停与回读同 `stop`，第二台经 ssh 起 `control stop`），这几台的卡因此回到没人持着；再清一次场，罩住各台点名的全部卡；清场过了各台才装、起新版。清场没过的，新版一台都没装，停掉的那几台照旧二进制起回来（这一台照 `start`，第二台经 ssh 起 `control start`），退 78，出路写清场日志与再跑同一条 `install`；有一台停不下的，不往下停、不清场，已经停掉的同样起回来。装不成（二进制不在、`--binary` 不在共享根下）与认不出各台点名的卡都在停之前拒。换版不用先另跑 `stop`。
  单元带 `Environment=CARGO=<多机配置 CARGO_BIN_DIRECTORY_BY_HOST 里装的那一台那一项>/cargo`（写结论时验证包要从 `CARGO` 认编这个二进制的工具链，没有就在写结论时崩；两台各取各的，取不到 `install` 退 78）。
  用户级 systemd 的单元 drop-in 目录 `singlefs-gpu-verification-server.service.d/` 下手写过设 `CARGO` 的 drop-in（例 `cargo.conf`）的，装完那一份多余：`install` 点名它，删法是 `rm` 那份文件、再 `systemctl --user daemon-reload`。
- 起：`python3 gpu-verification-server/control start`。没开 GPU 时一台都不起、退 0，所以随任务服务与判定库服务一起起的那一步可以一律调它；开了 GPU 而它没起来是服务健康的毛病，不退回 CPU。
- 清场：`start` 与 `install` 只在发起的那一台跑一趟 `bash scripts/crash/gpu-quiesce-run.sh -- true`（查卡、清场、回读）；没过退 78，出路点名那一趟的清场日志（默认在 `<工作根>/gpu-quiesce-logs`）。`start` 在起任何一台的单元之前清，罩住要起的各台点名的卡，没过一台都不起；`install` 在停了在跑的各台之后清，罩住各台点名的全部卡（次序见上面「装」）。`stop` 单独起时不清场。多机配置的清场命令是在起活的那一台跑、两台都管的，两台用同一份配置、哪一台发起都照它清，所以那条命令要两台都跑得起（`scripts/crash/gpu-quiesce-run.sh` 文件头「两台对称」）；经 ssh 转去第二台的那一条不再清场。第二台在不在跑经 ssh 起 `control unit-state` 问；`start` 时单元已在跑的那一台不清它的卡（卡在它手里），都在跑就不清场。清场之后不复原。清场放在起单元之前、不套在服务进程外面的理由写在 `control` 文件头「开卡之前清场」。
- 停：`python3 gpu-verification-server/control stop`；没装的那一台照退 0。单元发 SIGTERM，进程只置标记：取活循环收尾（入了池的块判完写回、放掉卡）、以自己的持有者放掉还持着的认领，退 0；单元的停止限时到了还没收完被 SIGKILL 的，没放的认领由下一个进程起来时放（下一条）。
- 上一个进程留下的认领：认领的持有者是每个进程新取的，崩掉、被杀的进程留下的认领下一个进程认不回。新进程写自己那一行「在跑」之前读上一个进程那一行，开机号、进程号、进程起来的时刻对得上且不是僵尸就是还活着、不放；别的就是没了，以它的持有者经判定库服务的放认领帧（`prune-server/src/protocol.rs` 文件头「放认领」，只放点名的持有者名下的）放掉库里各待判块上它的认领，别的服务随即取得到，不等满租期。认不出那一行、判不了活没活的不放，打 `GPU_VERIFICATION_SERVER_PREVIOUS_PROCESS_CLAIMS_KEPT` 一行，认领到租期过了失效。
- 查：`python3 gpu-verification-server/control check`。开了 GPU 时点名了卡的每一台都要单元环境里有 `CARGO`、它指的文件在那台在且能执行，且「在跑」（`is-active` 打 `active`，且 `serving` 写的进程号等于单元此刻的主进程号），缺一样退 78、出路写全；在跑的再读单元此刻的进程打过的全部日志（`journalctl --user -b _PID=<主进程号>`），有 `GPU_VERIFICATION_SERVER_CARD_FAILED` 行（「日志」一节）就退 1，打哪一台哪张卡（服务里的号、`nvidia-smi` 的序号、卡名、PCI 总线号、设备丢没丢、原因）与出路：主 agent 当场弹窗请用户重置那张卡，重置好了 `stop`、`start` 让服务重新开卡；只有 `GPU_VERIFICATION_SERVER_OUT_OF_MEMORY_CAUGHT` 行的不算卡坏；有 `GPU_VERIFICATION_SERVER_BLOCK_NOT_JUDGEABLE` 行（「日志」一节）也退 1，打哪一族哪一块、哪张卡、为什么与出路（放给另一台，不要重置卡）；没开 GPU 退 0。
- 读卡坏：`python3 gpu-verification-server/control cards-failed`，点名了卡的每一台（第二台经 ssh 起同一条）读此刻的进程打的卡坏行与判不了的行，标准输出每台一行 `GPU_VERIFICATION_SERVER_CARDS_FAILED_READ host=<台> state=<read|not-running|unreadable|unreachable> lines=<行数>`、每一行卡坏一行 `GPU_VERIFICATION_SERVER_FAILED_CARD_OF_A_HOST host=<台> process=<进程号> <卡坏行原样>`、每一行判不了一行 `GPU_VERIFICATION_SERVER_BLOCK_NOT_JUDGEABLE_OF_A_HOST host=<台> process=<进程号> <判不了那一行原样>`，没读成（`unreadable`、`unreachable`）的那一台另打一行 `GPU_VERIFICATION_SERVER_CARDS_FAILED_UNREAD host=<台> state=<状态>`，退 0；没开 GPU 只打 `why=gpu-off`、退 0。任务服务每分钟至多读一回，读到就往名下有在排、在跑任务的会话发「服务健康 GPU 服务卡坏」、往它的告警记录写一条 `gpu-card-failed`（`task-server/task_server.py` 的 `gpu_card_failures`）。
- 开机不自起（单元没有 `[Install]`、不 `enable`）；起来之后被杀由 systemd 隔 `RestartSec` 秒拉起（这一次不清场：清过的场不复原，一直清着），`control stop` 停的不拉起。

## 借卡

判段之外要自己开卡的真卡验证（只有借卡白名单里登记的）在要开卡的那一刻向这一台的服务借这一台的全部卡，用完还；借着的那一段服务不认领新的待判块。借与还只经判定库服务的认领帧与放认领帧走，不写数据行、不另开通道、不读写服务的状态目录；任务不带选卡变量、不写卡声明。条款原文在 `records/2026-11-17-subagent账本自上而下清欠.md` 第二百六十节（借卡）。

- 三种键（判定库服务的认领键，只记认领、不当数据行写）：请求键 `gpu-claim/card-lease-request/<这一台>`，借的一方持着表示有人要借；卡键 `gpu-claim/card-lease/<这一台>`，谁持着这一台的卡这一段归谁用；等号 `gpu-claim/card-lease-waiting/<这一台>/<开始等的毫秒数>-<借的进程号>-<启动刻>`，一个在等的 `lease take` 一张（下面「先来后到」）。`<这一台>` 是多机配置主机数组里这一台那一项（多机配置判法认的，与卡串里的地址逐字相同）。三种键的租期都取 `service-configuration.txt` 的 `CLAIM_LEASE_MILLISECONDS`。
- 先来后到：`lease take` 过了借卡白名单之后、认领请求键之前先认领一张等号，持有者身份是借的一方的身份；排序段三项各补零到定宽（开始等的时刻是 UNIX 纪元以来的毫秒数，20 位；借的进程号 10 位；启动刻 20 位），按键升序列出来的次序就是开始等的次序，同一毫秒的按借的进程号、启动刻分先后（同一台的 `lease take` 都在这一台起、读同一个钟）。之后每一回要去认领请求键之前先按前缀 `gpu-claim/card-lease-waiting/<这一台>/` 列这一台的等号，自己那一张排在最前才去认领请求键，不在最前隔 2 秒再列；等的这一段每过三分之一租期续一次等号。排在前面的借方不在了（它的 `lease take` 被杀、没人续），它的等号到租期失效、自然让出。请求键与卡键都认领到、起续租进程之前放掉自己的等号；中途退出的每一支（借的进程没了、等号或请求键续不上、读写判定库失败）同样放掉。
- 服务以它认领待判块的那个持有者身份（`serving` 那一行的 `holder`，每个进程起来时新取）持卡键：开好卡之后认领卡键，别人持着就从「借出」起步（卡照开）；之后每一圈在认领候选之前，以 1 毫秒的租期探一次请求键（至多每个扫库间隔一次），有人要借就两族候选都不认领，手上入了池的块照常判完写回，都写回了放掉卡键、打 `GPU_VERIFICATION_SERVER_CARDS_LENT`；借出期间不认领、设备照持着，每分钟卡账那一行照打（块数是 0）；没人要借了认领卡键，认领到打 `GPU_VERIFICATION_SERVER_CARDS_TAKEN_BACK`、接着认领。持着卡键时每过三分之一租期续一次。起来时上一个进程没了，它持着的卡键随它的认领一起放；停的时候自己的卡键随放认领一起放。判法写在 `src/serving_from_the_library.rs` 文件头「借卡」。
- 借卡白名单 `src/card-lease-borrowers.tsv`：一行 `<可执行文件名>  # <理由，写清出处>`，`#` 起头的行与空行不算；此刻只有 E202（GPU与CPU两种模式GC对照组）的装置二进制 `e202_gpu_cpu_gc_comparison` 一项。卡只归 GPU 服务：崩溃判段写待判行由服务判，别的测试不借卡；确要借卡的真卡验证，经用户同意往表里加一行，重编、`control install` 重装服务才生效（表经 `include_str!` 编进二进制）。认借的一方只看 `/proc/<借的进程号>/exe` 指到的那个文件的文件名，不看目录、包名与命令行，与表里一项逐字相同才放行：二进制从一个包搬到另一个包、同名照认；cargo 测试二进制带 `-<十六位哈希>` 后缀，对不上。读之前、读之后各认一回借的进程（进程号、启动刻、开机号），两回都在才算读到的是它的。
- 借的一方：`python3 gpu-verification-server/control lease take` 与 `… lease return <卡串>`，查借出的 `… lease list`（原样转给这一台装着的二进制的同名子命令；二进制没装退 78、出路写装法；不转去第二台）。take 与 return 两条都从环境读 `GPU_CARD_LEASE_BORROWER=<借的进程号>:<它的启动刻，/proc/<进程号>/stat 第 22 项>`（没给或写法不对退 2），持有者身份是 `card-lease/<这一台>/<进程号>-<启动刻>`；多机配置照 `SINGLEFS_MULTI_HOST_CONFIG`，没设照编二进制的那棵仓找。
  - `lease take`：先照借卡白名单认借的一方。读不到它的可执行文件、借的进程不在或启动刻对不上、文件名不在表里，一律照拒：一把键都不认领，往标准错误先打一行 `GPU_CARD_LEASE_REFUSED borrower_process=<借的进程号> executable=<带引号的可执行文件名，认不出写 none>`，再打 `✗` 一行（借的进程号、读到的可执行文件名、白名单的路径）与出路一行，退 4。放行了才认领一张等号、排到最前之后认领请求键（别人持着每 2 秒再列一次等号、再试，一直等），再认领卡键（同样一直等，期间续着请求键与等号）；等的时候头一回、在等的那一把或自己排第几变了的那一回与之后每 30 秒往标准错误打一行 `GPU_CARD_LEASE_WAITING key=<等号|请求键|卡键> host=<这一台> holder=<在等的那一把的持有者；排在别人后面时是排在最前那一张等号的持有者> remaining_lease_milliseconds=<它的剩余租期> position=<自己排第几，从 1 起>`；两把都到手才放掉等号、起续租进程，往标准错误打 `GPU_CARD_LEASE_TAKEN`、往标准输出打这一台的卡串（多机配置 `GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED`，没写取 `GPU_CARDS_BY_PRIORITY`，里地址是这一台的那几项，照原样、空白隔开）、退 0。这一台一张卡都没点名退 78。
  - 续租进程（二进制的 `lease renew`，只由 `lease take` 起，放进它自己的进程组）：每秒一拍，借的进程不在了、或被 `lease return` 叫停（停止键 `gpu-claim/card-lease-returned/<持有者身份>` 被别人持着）就放掉两把键、退出；都不是、过了三分之一租期才续两把键。
  - `lease return <卡串>`：认领停止键叫停续租进程、等它放掉卡键（至多 3 秒），再以借的一方的身份放两把键、退 0；已经不在它名下的也退 0。卡串参数给了就行，空串也收（借的命令退 0 而卡串是空的，借的一方当场还，交的就是 `lease return ''`）：不校验，只打进标准错误那一行 `GPU_CARD_LEASE_RETURNING`；键只由这一台与借的一方那个变量定。一个参数都没给退 2。
  - `lease list`（不要 `GPU_CARD_LEASE_BORROWER`，只读、一把键都不认领）：对多机配置主机数组里点名了卡的每一台，照主机数组的次序，往标准输出打一行 `GPU_CARD_LEASE_HELD host=<台> held_by=<service|borrower|nobody> holder=<带引号的卡键持有者身份；没人持着写 none> cards=<带引号的那一台的卡串> lease_left_milliseconds=<卡键的剩余租期；没人持着写 0>`（持有者身份以 `card-lease/` 起头的是借方，别的是服务以它认领待判块的身份）；请求键有人持着时另打一行 `GPU_CARD_LEASE_REQUESTED host=<台> holder=<带引号的身份> lease_left_milliseconds=<剩余租期>`；每一张等号照次序一行 `GPU_CARD_LEASE_QUEUED host=<台> position=<第几，从 1 起> holder=<带引号的身份> lease_left_milliseconds=<剩余租期>`。读到几台都退 0；连不上判定库照判定库客户端的退出码。
  - 借的进程退出（或被杀）之后至多一两秒两把键放掉；借的进程与续租进程都被杀的，租期过了服务那一圈自己收回。

## 日志

服务的标准错误进 journald（用户级单元）。验证包的 GPU 判器每次派活往标准错误打 `GPU_LIFECYCLE`、`CRASH_JUDGE_ACCOUNTING_SLICE`、`CRASH_JUDGE_ACCOUNTING_SLICES`，满速时每秒几千行，journald 按整个用户管理器限流、成窗丢行，所以服务进程起来头一步把标准错误换成一条管道，一条线程读它：这三种行只计数、不转，别的行原样转（`src/service_log.rs` 文件头写了换法与代价：子进程照继承的标准错误是关着的；被信号打死时管道里没转出去的行会丢，panic 那一句直接写原来的标准错误；服务日志里看不到逐次派活的时刻）。

服务自己打的、可数的行（行首）：

| 行首 | 什么时候 | 带什么 |
|---|---|---|
| `GPU_VERIFICATION_SERVER_BLOCK_CLAIMED` | 认领到一块（重读过待判行、配上了流） | 块、态数 |
| `GPU_VERIFICATION_SERVER_FACTS_VERIFIER_BLOCK_CLAIMED` | 认领到一块核对待判行（重读过、读回了那一组的材料） | 块、态数 |
| `GPU_VERIFICATION_SERVER_FACTS_VERIFIER_CLAIM_HELD_BY_OTHER` | 核对待判行别的持有者持着、这一轮不取（连着几轮只打头一轮） | 块、持有者、剩余租期 |
| `GPU_VERIFICATION_SERVER_FACTS_VERIFIER_CLAIM_RELEASED_AFTER_THE_REREAD` | 认领到之后重读核对待判行不在了（或指的是另一块、重读不了），放掉刚认领到的 | 块、为什么、放掉了几把 |
| `GPU_VERIFICATION_SERVER_VERIFIED_FACTS_BLOCK_ROW_DELETED` | 核对行已在的块，核对待判行带守卫删了 | 块 |
| `GPU_VERIFICATION_SERVER_CLAIM_HELD_BY_OTHER` | 别的持有者持着、这一轮不取（同一块同一个持有者连着几轮持着只打头一轮） | 块、持有者、剩余租期 |
| `GPU_VERIFICATION_SERVER_CLAIM_RELEASED_AFTER_THE_REREAD` | 认领到之后重读待判行不在了（或指的是另一块、重读不了），放掉刚认领到的 | 块、为什么、放掉了几把 |
| `GPU_VERIFICATION_SERVER_BLOCK_JUDGED` | 每判完一块 | 流键头、块号、态数、哪几张卡、派活几次、卡上用时（毫秒） |
| `GPU_VERIFICATION_SERVER_FACTS_BLOCK_VERIFIED` | 每核完一块核对块 | 同上那一行同名的字段（`stream`、`block`、`states`、`cards`、`dispatches`、`card_milliseconds`；核对器一块一次派活，卡上用时是整组的挂钟按态数摊） |
| `GPU_VERIFICATION_SERVER_CONCLUSION_WRITTEN` | 结论那一帧写成了 | 块、态数、写没写凭据行、是不是重发早已生效 |
| `GPU_VERIFICATION_SERVER_FACTS_VERIFIER_WRITTEN` | 核对行那一帧写成了 | 块、态数、写没写核对行（库里已有逐字节相同的那一行时只删核对待判行）、是不是重发早已生效 |
| `GPU_VERIFICATION_SERVER_FACTS_VERIFIER_WRITE_REFUSED` | 核对行那一帧没写成（结论位留在服务里，核对待判行留在库里） | 块、态数、为什么 |
| `GPU_VERIFICATION_SERVER_CARD_PERIOD` | 每 60 秒（`serving_from_the_library.rs` 的 `CARD_SUMMARY_PERIOD`）每张卡一行，停的时候把没满的那一段也打 | 这一段判了几块几态、核了几块几态（`facts_blocks`、`facts_states`）、卡上用时（判与核合起来）占这一段的千分数 |
| `GPU_VERIFICATION_SERVER_DISPATCH_LINES_DROPPED` | 同上，每段一行 | 这一段滤掉了几行逐次派活的行 |
| `GPU_VERIFICATION_SERVER_CLAIMS_RELEASED` | 起来时放上一个进程的认领、停的时候放自己的 | 为什么、持有者、放掉了几把、没人持着几把、别人持着几把 |
| `GPU_VERIFICATION_SERVER_PREVIOUS_PROCESS_CLAIMS_KEPT` | 起来时上一个进程的认领不放 | 为什么（还活着、认不出、判不了） |
| `GPU_VERIFICATION_SERVER_CARDS_LENT` | 有人要借、手上的块都写回之后放掉卡键；起来时卡键被别人持着；持着的卡键续不上（「借卡」一节） | `host=<这一台>`、`blocks_finished_first=<探到有人要借起到放卡键之间写回成了的块数，两族合起来>` |
| `GPU_VERIFICATION_SERVER_CARDS_TAKEN_BACK` | 没人要借了、收回卡键 | `host=<这一台>`、`seconds_lent=<借出了多少秒>` |
| `GPU_VERIFICATION_SERVER_JUDGE_LOADED` | 每装上一个判器 | `card`、`stream`（判表摘要头）、`table_mebibytes`、`scratch_mebibytes`、`states_per_dispatch`、`judges_on_the_card`（装上之后这张卡驻留几个）、`free_mebibytes`（账上还剩的：额度减驱动那一份减驻留的；新表上卡时另有一份同样大的暂存，账不记它，批次调度分草稿区时留出来了） |
| `GPU_VERIFICATION_SERVER_OUT_OF_MEMORY_CAUGHT` | 判或核的时候显存不够（设备没丢，或因显存不够丢了、已重开设备，`reason` 里写着）：卡上的判器全丢、卡照常用（不是卡坏） | `card`、`judges_dropped`、`blocks_put_back`、`blocks_refused`、`facts_jobs_put_back`、`facts_jobs_refused`、`put_back=`/`refused=`（有截撤回的、报拒的块，各写收下的那一块、每块一次：`<流键头>/<块号>`、判表 `tables-<摘要头>`，交回过了的写 `<判表摘要头>:<序号区间>`）、`reason`（panic 的那一句） |
| `GPU_VERIFICATION_SERVER_CARD_FAILED` | 一张卡坏了、不再排（设备为别的原因丢了；因显存不够丢了而重开没成；接住 Out of Memory 之后那一次提交没等成）；判法自己 panic 不算；`control check` 读到就退 1，任务服务照它发告警 | `card`、`nvidia_smi_index`、`name`、`pci`、`device_lost=<yes\|no>`、`reason` |
| `GPU_VERIFICATION_SERVER_BLOCK_NOT_JUDGEABLE` | 这一台判不了的块（在腾空的卡上也显存不够，或判法自己 panic）：认领当场放掉、这个进程不再取，待判行留在库里；`control check` 读到这个进程打的就退 1，任务服务照它发告警 | `family=<image-judgement\|facts-verifier\|packed-tables>`、`stream`（流键头，判表写判表摘要头）、`block`（块号，判表写 `tables`）、`card`、`why=<out-of-memory-on-an-emptied-card\|judge-panicked>`、`claim=<released\|not-held\|kept>`、`reason` |
| `GPU_VERIFICATION_SERVER_STOPPED` | 收尾完了、退出之前 | 取活循环的账（含借出、收回各几回、坏了的卡 `cards_failed`、接住 Out of Memory 几回、在腾空的卡上也显存不够与判法 panic 而这个进程不再取的块数） |

派活几次与卡上用时由每张卡的工作线程量（`src/card_worker.rs` 文件头「判的账」）：派活几次取判器自己数的，卡上用时是交判器到判器交回的挂钟、按态数摊到段里各块。

接住 Out of Memory（`src/service_loop.rs` 文件头）：工作线程判或核的时候 panic，读这张卡的设备情况，不认 panic 的字句：设备没丢而 Out of Memory 比上一回多记了，丢掉卡上全部判器，在这张卡上做一次提交、等它做完（丢掉的缓冲与上一波新表的暂存挂在 wgpu 的待写列表里，下一次提交做完才还给驱动），再打 `GPU_VERIFICATION_SERVER_OUT_OF_MEMORY_CAUGHT`，这张卡照常收活，别的卡不动；设备因显存不够丢了（验证包认得出）的丢掉全部判器、重开这张卡的设备，同样打那一行、照常收活。它那一波在这张卡上没交回结论的那几截撤回、另入池（收下的那一块留着，别的卡判过的截照收，撤回的截判出的结论记回那一块，每个态只判一次），每截记下在哪几张卡上显存不够过：别的没坏、装得下这条流的卡照常拿去判（不派回它显存不够过的卡）；哪张都装不下时等一张腾空的卡单独判（撤回它的那张卡没坏就等那一张，坏了取头一张空着的没坏的卡；只装那一条流的判器，草稿区照整张卡分）；单独判又显存不够的，收下的那一块整块报拒（连同它撤回的每一截，判了一半的结论不交）。在核的那一组核对块放回排头、下一回挑一张空着的卡把它驻留的判器全换出再核（不一定是显存不够的那一张），腾空了还不够的整组报拒。
判法自己 panic（设备没丢、没记 Out of Memory，例：装表时表超过绑定上限的断言）不是卡坏：卡上的判器全丢、卡照常收活；panic 时在装哪条流的、在判哪一批的、在核哪一组的那几块报拒，这一波别的没交回结论的截撤回重排（照接住 Out of Memory 那样撤回，只是不记显存不够、不等腾空的卡）。
报拒的这两种（在腾空的卡上也显存不够、判法 panic）是这一台判不了：取活循环当场放掉那一块的认领（另一台的服务取得到）、打一行 `GPU_VERIFICATION_SERVER_BLOCK_NOT_JUDGEABLE`，这个进程不再认领它，待判行留在库里；不退回 CPU 判。设备为别的原因丢了的照旧不再排那张卡、打卡坏那一行；一张卡坏了之后，没有一张没坏的卡装得下的流的块当场报拒，再入池的也当场拒。
新装判器的那一刻表占两份（验证包 `create_buffer_init` 上卡时 wgpu 另开一份同样大的暂存，下一次提交做完才放）：批次调度挑新流与分草稿区都照两份算，登记流时判装不装得下也照两份（`card_memory_ledger::mebibytes_while_loading_a_judge`）；驻留的账照旧一份。

## 核对待判行那一族

配了 GPU 的判段第 ② 段把要核的块写成核对待判行（验证包表 `1F`），本服务取走、在卡上核、一帧写回核对行并删掉核对待判行。各步与待判行那一族走同一套，取这一族自己的那一样：

| 步 | 这一族取什么 |
|---|---|
| 扫表 | 验证包 `VerdictStore::every_facts_verifier_pending_row`（`LibraryPendingPool::facts_verifier_candidates`） |
| 核过没有 | 验证包 `facts_verifier_pending_block_is_verified_in_the_library`：核对行已在的不取、带守卫只删核对待判行 |
| 认领与租期 | 认领键 `gpu-claim/facts-verifier/<流键的十六进制>/<块号>`，以判定库服务的 GPU 认领键前缀起头（放认领帧收得下）、与待判行那一族的 `gpu-claim/<流键>/<块号>` 分得开；认领到之后重读核对待判行；在核的每过三分之一租期续一次；起来时放上一个进程的、停的时候放自己的，两族一起放 |
| 上卡 | 认领到的按（录下的流程、事实表）分组，每组读回一份材料（照组里键最小的那一块的行，验证包 `material_of_a_facts_verifier_pending_block`），交服务主循环：照卡上驻留的判器记一本显存账，借得到才派，借不到先换出驻留的判器（没有待判态的在前），哪张卡都放不下的当场拒；工作线程照材料建 `GpuFactsVerifier`、逐块 `verify_states`、核完丢掉 |
| 核对器借多少显存 | 照它建的缓冲算（`card_memory_ledger::mebibytes_of_a_facts_verifier`）：上卡一次的六张事实表，加一块一趟的头、结论与回读乘同时还活着的趟数（验证包 `gpu_memory_ledger::PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD`）；驱动与管线算在驱动那一份里；推的，没上卡量过 |
| 写回 | 验证包 `facts_verifier_frame_of_a_pending_block` 编那一帧（要写的核对行、核对行守卫、要删的核对待判行与它的守卫）；服务放进判定库服务的写结论帧，认领键与持有者由服务填，核认领、核两个守卫、写、删、放认领一帧原子地落；守卫不过照种类重编、重读，至多 4 次，没写成的结论位留在服务里报红 |

## 登记的临时库

判段的库在临时 prune-server 里（测试的一次性库、变异门禁与证红每块起的临时服务）时，判段在真服务里登记那份临时服务：一把续着租的认领，键 `gpu-claim/temporary-library/<这一台在 HOSTS 里的地址>/<进程号>-<启动刻>/<载荷的小写十六进制>`，不写数据行；载荷依次是 socket 路径、TCP 地址（一次性库是这一台 HOSTS 地址上内核挑的端口）、键空间前缀（空的是临时服务里全仓那一份的键原样，否则头一个字节是一次性库名字空间的头一个字节）、标签（族名接 `#<进程内序号>`），每段 `u16 长度（大端）` 加内容。登记的写法、解法、持有者身份与租期都在验证包 `crash_library::TemporaryLibraryRegistration`，服务调它的解法，不留第二份写法。持有者身份是 `temporary-library/<这一台>/<进程号>-<启动刻>`，租期 30 秒、每 10 秒续一次。服务每一圈列出登记（判定库服务的「列认领」），照登记去连：

| 步 | 做什么 |
|---|---|
| 开 | 每份登记开一个 `LibraryPendingPool`，用它自己的路由与两个客户端（认领与写结论的一个、验证包那一侧的一个）；追加日志放 `<状态目录>/temporary-libraries/<登记键的 sha256 头 8 个字节>/`，起来时整个目录删掉重建；头一次打开时照 `release_the_claims_left_by_the_previous_process` 放上一个进程留在那份库里的认领 |
| 连哪里 | 登记的台是这一台、socket 在本机的走 unix socket，否则走 TCP；TCP 地址是回环而登记的台不是这一台的不连；两个客户端重连只等几秒（`REGISTERED_LIBRARY_RECONNECT`），登记的库没了不拖住真库的活 |
| 不连 | 登记的 socket 解开符号链接之后等于服务自己连的真服务 socket、或 TCP 等于服务自己连的那个，登记键不合布局，键空间前缀不空而头一个字节不是 `0xE0`；点名一次（`GPU_VERIFICATION_SERVER_TEMPORARY_LIBRARY_NOT_OPENED`） |
| 键 | 待判池自己编的键（删待判行、强制现判凭据）调验证包带键空间的那一把（`key_bytes_of_the_gpu_pending_row_in`、`key_bytes_of_the_gpu_rejudged_row_in`、`key_bytes_of_the_facts_verifier_pending_row_in`、`key_bytes_of_the_packed_tables_pending_row_in`），键空间取这一份库验证包那一侧的（`VerificationPackageSide::key_space`），服务里不自己拼前缀；验证包交回的帧里的键已经接过；认领键不接（认领表是那一份临时服务自己的） |
| 先来先判 | 各份库认领到的块（三族各自）合起来按待判行的入池序号升序入池，序号相同真库在前、库内保持扫库的键序；真库一直有活时登记的库的活照样排得到 |
| 写回 | 入了池的块记着它所在的库，判完写回那一份、只经那一份的客户端；块名只在一份库里认得，几份库里的同名块各写各的；那一份库已经丢掉的不写回（`GPU_VERIFICATION_SERVER_TEMPORARY_LIBRARY_BLOCK_DISCARDED`） |
| 丢 | 登记没了（租期过了、判段放了）、读写这一份库失败：只丢那一份（`GPU_VERIFICATION_SERVER_TEMPORARY_LIBRARY_DROPPED`，读写失败那一行每份登记只打一次），追加日志目录删掉；失败的登记下一次列到再试，至多每个扫库间隔一次；真库读写失败照旧让服务退出 |
| 借卡 | 借卡只经真库那一份待判池；借出期间哪一份库都不扫、不认领，登记的库也不开 |
| 停 | 登记的库各以自己的持有者放掉还持着的认领，再丢掉；放不了的认领到租期过了自己失效 |

## 测试

全部经统一任务服务交，工作目录是带着验证包的那棵树里的 `gpu-verification-server/`（工作区成员的目录里起的 cargo 只编这一个包）。

| 测试目标 | 判什么 | 要卡 |
|---|---|---|
| `batch_scheduling_requirements` | 批次调度要满足的八条与新装那一刻给表的暂存留地方那一条，一条一个测试；核对器借的显存照表两份加几趟算 | 不要 |
| `card_memory_matches_the_verification_package` | 调度算的判器显存与验证包借卡算的逐个相同；流的键随表与源码变 | 不要 |
| `service_loop_with_judges_off_the_card` | 池 → 调度 → 工作线程 → 客户端整条路在替身判器上；卡坏了报拒；登记与入池的拒；核对组照显存账借（放得下不换出、放不下先换出、哪张卡都放不下当场拒）；接住 Out of Memory（卡照常收活、重判之前等过一次提交、撤回的块别的卡照常拿、判完一次、单独在腾空的卡上又不够的报拒、核对组腾空再核）；设备因显存不够丢了的重开、卡照常用，为别的原因丢了照旧坏卡；判法 panic 报拒那一块、卡不坏；大卡坏了之后只有它装得下的块当场拒；装上、接住、卡坏三件事与打成的行；真卡判器缓存认设备怎样那一步在假的设备情况上核 | 不要 |
| `serve_process_marks_itself_serving_and_waits_to_be_stopped` | 进程本体（连测试进程里起的临时判定库）：多机配置没点名这一台的卡退 78、不写 `serving`；参数认不出退 2、状态目录不在退 1，都不写 `serving` | 不要 |
| `library_pending_pool_on_a_temporary_library` | 判定库那一份待判池对着临时判定库服务（待判行照验证包的写法读写，验证包另外三样是替身）：别人持着不取（打一行、连着几轮只打一次）、行里写什么结论版本键都照认领并照行里的键写回、强制现判不问库且写凭据行、写结论那一帧被拒时结论留下报红、判完而可复用的待判行带守卫删、判着时待判行被升成强制现判的重读再写带凭据、结论块那一行被别人改过的重编再写（两份都绿而不同报红）、扫库与认领之间别人写回了的块不取并放掉刚认领到的、上一个进程没了起来时放它的认领、上一个进程还活着不放、停的时候放自己的；取活循环起来就开卡、池空着照持着、停的时候才放；一块判下来服务日志过滤之后没有逐次派活的行、汇总行恰好一行且态数对；核对待判行那一族（扫、核过没有、编那一帧照验证包那一份，材料与核对器是替身）：取活循环认领、核、写回核对行、删核对待判行、放认领，已核过的只删不核，两个守卫各被每一回改掉时那一帧落不下、各被改一回时重来落下，上一个进程没了与停的时候放这一族的认领，扫库与认领之间别人写回了的不再取；在腾空的卡上也显存不够的块打两行接住、一行判不了，认领当场放掉、别的持有者认领得到，这个进程不再取 | 不要 |
| `card_lease_on_a_temporary_library` | 借卡对着临时判定库服务：取活循环那一半（卡与判器是替身，借的一方由测试认领、放键来演）有人要借就停认领、手上的块写回之后才借出、借出期间不放设备、还了收回接着认领，起来时卡键被别人持着从借出起步，借的一方没了租期过后收回，借出期间每分钟卡账那一行照打、块数是 0（这一条跑一分钟多）；借的一方那一半起这个包的二进制：借的进程是拷进用例目录、改名成白名单里那个名字的 `sleep`：`lease take` 认领到卡键之后才打卡串、`lease return` 之后两把键放了、续租进程退出，`lease return ''` 退 0、放键而一个参数都没给退 2，两个借的一方先后借到，先来后到：请求键与卡键被别人持着时先后排上的两个，先等的那个停着的那一段后等的照等（等的那一行带 `position=2`），先等的先借到、还了之后后等的才借到，排在最前的那个 `lease take` 被杀之后后面那一个到它的等号租期过了才借到（这一条跑一个认领租期多，两分钟多），借的进程没了 5 秒内放键，变量没给或写法不对退 2，这一台没有卡退 78；借卡白名单：可执行文件名不在表里的（测试进程自己）、白名单里那个名字接上哈希后缀的退 4、两把键都没认领、拒绝那一行与 `✗`、出路两行都在，白名单里的借得到；`lease list`：没人借时每台一行没人持着、服务持着写 `held_by=service`、一个借着一个排着时打借方持卡那一行、请求键那一行与一行 `position=1` 的排队行 | 不要 |
| `library_pending_pool_serves_registered_temporary_libraries` | 登记的临时库对着测试进程里起的两三份临时判定库服务（一份当真服务、登记在它里面，登记照验证包 `TemporaryLibraryRegistration` 的写法与持有者身份；另几份是登记的临时服务；卡与判器、验证包那一侧的三样是替身）：登记的库里的块判完写回那一份自己的那一段名字空间、真库一行没多；两份库里的同名块各写各的；登记过期丢掉那一份、在判的块不写回；一份库读写失败只丢那一份、真库照判、点名一次；指到真服务与前缀不合的登记不开；各份库的块按入池序号先来先判；借出期间哪一份库都不扫；重起放上一个进程留在登记的库里的认领 | 不要 |
| `gpu_service_path_agrees_with_the_direct_judge` | 服务路径与直接用 `GpuCrashJudge` 判逐态相同；两条流（同一个展开上限，窄的那条少展开一段）同卡交替派与各自独占逐态相同 | 要，带 `--features gpu`（任务服务照这个特性分卡、清场）；没卡打「本次未跑」 |
| `library_service_path_agrees_with_the_direct_judge` | 同一条小流切成几块写成待判行，取活循环取走、上卡判、写回，读回的结论字与直接判逐态相同（验证包那一侧是替身，结论字原样存） | 要，带 `--features gpu`；没卡打「本次未跑」 |
| `facts_verifier_service_path_agrees_with_the_direct_verifier` | 同一条小流的事实表切成几块写成核对待判行，取活循环取走、照显存账借、上卡核、写回核对行，读回的结论位与直接用 `GpuFactsVerifier` 核的、与 CPU 参照逐态相同（核对材料是替身，交的就是读回的那一份事实表与拆法） | 要，带 `--features gpu`；没卡打「本次未跑」 |

判段写待判行、本服务判完写回、判段读回的结论与直接判逐态相同那一条端到端对拍不在这里：判段写行等判的入口（验证包 `judge_my_blocks_on_gpu`）只在验证包里可调，见「还没做的」。

`control` 的自证不起 cargo、不起真 systemctl、ssh、journalctl 与清场脚本（换成自证根里的假脚本）：`python3 gpu-verification-server/control --selftest`，弄坏开关 `GPU_VERIFICATION_SERVER_CONTROL_BREAK`（表在 `control` 文件头）。

## 弄坏开关

`GPU_VERIFICATION_SERVER_BREAK=<项>[,<项>…]`，测试读（`BreakSwitches::of_the_environment`），服务进程交的是空的；二进制的 `lease` 子命令只在临时判定库服务底下（设了 `PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET`）读它，别处设了退 2（借卡那几条测试起的就是这个二进制）。每一项打开后点名的测试目标里那一条必须红（表与 `BreakSwitch::red_test_target`、`BreakSwitch::red_test` 逐项相同）。

测试目标 `batch_scheduling_requirements`：

| 项 | 弄坏哪一处 | 红的测试 |
|---|---|---|
| `memory-ledger-unchecked` | 显存账收越界的判器 | `judges_resident_on_a_card_never_exceed_the_quota_less_the_driver_share` |
| `scratch-split-ignores-kept-judges` | 新装的判器分草稿区时不扣留着的判器 | 同上 |
| `carved-states-not-consumed` | 切过的态下一批又切 | `every_pending_state_is_dispatched_exactly_once_and_the_rest_stays_in_the_pool` |
| `pool-drops-the-rest-of-a-partly-dispatched-block` | 内存池把没派的那一截也当派了 | 同上 |
| `watermark-ignored` | 池里有态就发 | `a_wave_goes_out_when_every_card_fills_once_or_the_oldest_waited_the_limit_and_never_from_an_empty_pool` |
| `wait-limit-ignored` | 等多久都不发 | 同上 |
| `batch-not-capped-by-the-state-count` | 一次派活的态数不按流的态数压 | `streams_sharing_a_card_split_the_remaining_memory_and_a_short_stream_takes_its_own_state_count` |
| `scratch-split-by-entry-order` | 新装的几条流先到先拿、不均分 | 同上 |
| `batches-of-a-stream-back-to-back` | 一条流的批在一波里连着派完 | `batches_of_streams_sharing_a_card_alternate_from_wave_to_wave` |
| `pool-order-ignored` | 流按键排、不按入池先后 | `streams_are_taken_in_entry_order_and_a_short_stream_is_not_left_behind_a_long_one` |
| `starving-streams-not-rescued` | 等过限时也不腾卡 | 同上 |
| `residence-ignored` | 驻留的判器当新流重装 | `a_resident_stream_goes_back_to_its_card_and_only_judges_without_pending_states_are_evicted` |
| `every-resident-judge-evicted` | 腾显存时有待判态的也换出 | 同上 |
| `input-order-followed` | 照调用方给的次序走 | `the_same_inputs_give_the_same_plan_whatever_order_they_come_in` |
| `table-staging-not-reserved` | 新装判器的那一刻只给表留一份 | `a_newly_loaded_judge_leaves_room_for_a_second_copy_of_its_table_while_it_loads` |
| `facts-tables-staging-not-reserved` | 核对器借的显存里事实表只算一份 | `a_facts_verifier_borrows_two_copies_of_its_tables_and_its_passes` |

测试目标 `library_pending_pool_on_a_temporary_library`：

| 项 | 弄坏哪一处 | 红的测试 |
|---|---|---|
| `claim-held-by-others-ignored` | 认领时别的持有者持着也当认领到 | `a_block_claimed_by_another_holder_is_left_to_it` |
| `forced-block-reads-the-library` | 强制现判的也先问库，判完的不取 | `a_forced_block_is_judged_without_reading_the_library_and_leaves_a_rejudged_row` |
| `rejudged-row-not-written` | 强制现判的写回那一帧不写凭据行 | 同上 |
| `refused-conclusions-dropped` | 那一帧没写成时结论不留 | `conclusions_refused_at_the_write_are_kept_and_reported_red` |
| `cards-opened-only-with-work` | 起来不开卡，库里有了能取的块才开 | `the_cards_are_opened_at_start_and_held_while_the_pool_idles` |
| `cards-let-go-when-the-pool-idles` | 池空一圈就放掉卡、再来活再开 | 同上 |
| `reusable-block-pending-row-kept` | 判完而可复用的块只不取、待判行不删 | `a_reusable_block_is_not_taken_and_its_pending_row_is_deleted_under_the_guard` |
| `pending-row-change-not-reread` | 写结论那一帧因待判行读完之后变了被拒时不重读、当场报红 | `a_pending_row_raised_to_forced_while_judged_is_reread_and_written_with_its_credential` |
| `guard-row-change-not-reread` | 写结论那一帧因结论块那一行编帧之后变了被拒时不重编、当场报红 | `a_conclusion_changed_under_the_guard_is_compared_again_before_writing` |
| `pending-row-not-reread-after-the-claim` | 认领到之后不重读待判行：扫库与认领之间别人写回了的块照取、刚认领到的不放 | `a_block_written_back_between_the_scan_and_the_claim_is_not_taken_again` |
| `previous-process-claims-kept` | 起来时不放上一个服务进程（已不在）留下的认领 | `blocks_claimed_by_a_service_process_that_is_gone_are_released_at_the_next_start` |
| `previous-process-taken-for-gone` | 上一个服务进程还活着也当它没了，放掉它的认领 | `blocks_claimed_by_a_service_process_still_running_are_not_released` |
| `claims-kept-at-the-stop` | 停的时候不放自己还持着的认领 | `claims_still_held_when_the_service_stops_are_released` |
| `dispatch-lines-forwarded` | 服务日志不滤逐次派活的那两种行，照转 | `a_judged_block_leaves_one_summary_line_and_no_dispatch_lines_in_the_service_log` |
| `facts-verifier-rows-not-scanned` | 不扫核对待判行那一张表 | `a_facts_verifier_pending_row_is_claimed_verified_and_written_back_by_the_serving_loop` |
| `facts-verifier-frame-under-the-image-judgement-claim-key` | 核对行那一帧的认领键写成待判行那一族的形，这一族的认领没随那一帧放掉 | 同上 |
| `facts-verifier-verified-block-verified-again` | 不问核过没有，核对行已在的照取 | `an_already_verified_facts_block_only_has_its_pending_row_deleted` |
| `facts-verifier-row-guard-taken-at-the-write` | 核对行守卫取写之前此刻那一份 | `a_facts_verifier_frame_whose_verifier_row_changed_after_it_was_read_does_not_land` |
| `facts-verifier-pending-row-guard-taken-at-the-write` | 删核对待判行的期待取写之前此刻那一份 | `a_facts_verifier_frame_whose_pending_row_changed_after_it_was_read_does_not_land` |
| `facts-verifier-guard-refusal-not-retried` | 核对行那一帧守卫不过时当场报红、不重来 | `a_facts_verifier_frame_refused_once_by_each_guard_is_composed_again_and_lands` |
| `facts-verifier-claims-left-at-the-release` | 放认领只放待判行那一族的 | `facts_verifier_claims_are_released_at_the_next_start_and_at_the_stop` |
| `facts-verifier-row-not-reread-after-the-claim` | 认领到核对待判行之后不重读 | `a_facts_verifier_block_written_back_between_the_scan_and_the_claim_is_not_taken_again` |
| `cards-failed-left-out-of-the-summary` | 会话收尾时丢掉坏了的卡，取活循环的账里没有 | `a_card_failure_leaves_one_card_failed_line_and_the_summary_names_the_card` |
| `block-refused-for-memory-claimed-again` | 在腾空的卡上也显存不够而被拒的块，这个进程下一轮照样认领 | `a_block_running_out_of_memory_on_an_emptied_card_is_named_left_to_another_service_and_not_claimed_again_here` |
| `block-not-judgeable-claim-kept` | 判不了的块不当场放认领，留到租期过了 | 同上 |
| `block-not-judgeable-line-left-out` | 判不了的块不打判不了那一行 | 同上 |
| `packed-tables-rows-not-scanned` | 不扫判表待判行那一张表 | `a_packed_tables_pending_row_is_claimed_judged_and_written_back_by_the_serving_loop` |
| `packed-tables-verdict-words-judged-again` | 不问表 `21` 里结论字在不在，已有的照取、照判 | `packed_tables_whose_verdict_words_are_in_the_library_only_have_their_pending_row_deleted` |
| `packed-tables-verdict-words-guard-taken-at-the-write` | 结论字那一帧的表 `21` 守卫取写之前此刻那一份 | `a_packed_tables_frame_whose_verdict_words_row_changed_after_it_was_read_does_not_land` |
| `packed-tables-pending-row-guard-taken-at-the-write` | 删判表待判行的期待取写之前此刻那一份 | `a_packed_tables_frame_whose_pending_row_changed_after_it_was_read_does_not_land` |
| `packed-tables-guard-refusal-not-retried` | 结论字那一帧守卫不过时当场报红、不重来 | `a_packed_tables_frame_refused_once_by_each_guard_is_composed_again_and_lands` |
| `packed-tables-claims-left-at-the-release` | 放认领不放判表待判行那一族的 | `packed_tables_claims_are_released_at_the_next_start_and_at_the_stop` |
| `packed-tables-row-not-reread-after-the-claim` | 认领到判表待判行之后不重读 | `a_packed_tables_row_written_back_between_the_scan_and_the_claim_is_not_taken_again` |

测试目标 `serve_process_marks_itself_serving_and_waits_to_be_stopped`：

| 项 | 弄坏哪一处 | 红的测试 |
|---|---|---|

测试目标 `service_loop_with_judges_off_the_card`：

| 项 | 弄坏哪一处 | 红的测试 |
|---|---|---|
| `facts-verifier-memory-unchecked` | 主循环派核对器上卡时不照显存账腾地方 | `a_facts_verifier_job_borrows_card_memory_and_evicts_resident_judges_only_as_needed` |
| `out-of-memory-taken-for-a-card-failure` | 工作线程接住设备没丢的 Out of Memory 也报卡坏、线程退出 | `a_card_that_ran_out_of_memory_takes_the_next_wave_and_every_block_comes_back_judged_once` |
| `residents-kept-after-running-out-of-memory` | 接住之后账上不清那张卡驻留的判器 | 同上 |
| `blocks-of-the-out-of-memory-wave-refused` | 接住之后在判的块整块报拒、不撤回 | 同上 |
| `out-of-memory-caught-not-told` | 接住之后不交「接住」那一件 | 同上 |
| `put-back-takes-the-whole-block` | 撤回时撤整块：整块照它的序号重新入池，旧号上别的卡交回的结论不记、别的卡判过的态再判一遍 | 同上 |
| `retracted-block-shares-the-next-wave` | 撤回的块照常交调度、不等腾空的卡 | `a_block_running_out_of_memory_again_alone_on_an_emptied_card_is_refused_and_the_card_takes_the_next_block` |
| `out-of-memory-on-an-emptied-card-retried` | 单独在腾空的卡上又不够的块照样撤回、不报拒 | 同上 |
| `facts-job-of-the-out-of-memory-round-refused` | 核的时候显存不够那一组报拒、不放回去腾空再核 | `a_facts_verifier_job_that_ran_out_of_memory_is_verified_again_on_an_emptied_card` |
| `card-failure-not-told` | 卡坏了不交「卡坏」那一件 | `a_card_whose_device_is_lost_while_judging_still_fails_and_tells_one_card_failure` |
| `judge-loaded-not-told` | 装上判器不交「装上」那一件 | `every_judge_loaded_tells_one_line_with_its_table_scratch_and_what_the_card_has_left` |
| `card-not-settled-after-running-out-of-memory` | 接住之后不在卡上做那一次提交、不等它做完 | `a_card_that_ran_out_of_memory_waits_for_one_submission_to_finish_before_loading_again` |
| `device-lost-for-want-of-memory-taken-for-a-card-failure` | 因显存不够丢了、已重开设备的也报卡坏 | `a_device_lost_for_want_of_memory_is_reopened_and_the_card_takes_the_next_block` |
| `judge-panic-taken-for-a-card-failure` | 判法自己 panic 也报卡坏 | `a_judge_panicking_on_loading_refuses_its_blocks_and_every_card_keeps_judging_other_streams` |
| `put-back-blocks-wait-for-their-own-card` | 撤回的块一律只等撤回它的那张卡腾空单独判 | `blocks_put_back_after_running_out_of_memory_are_taken_by_another_card_that_fits` |
| `blocks-only-the-failed-card-could-hold-left-pending` | 卡坏了之后只有它装得下的块不报拒、挂着 | `a_block_only_the_failed_big_card_could_hold_is_refused_not_left_hanging` |

测试目标 `card_lease_on_a_temporary_library`：

| 项 | 弄坏哪一处 | 红的测试 |
|---|---|---|
| `card-lease-request-ignored` | 取活循环不探请求键：有人要借也照认领、不放卡键 | `a_lease_request_stops_new_claims_and_the_cards_are_lent_once_the_blocks_in_hand_are_written_back` |
| `cards-lent-before-the-blocks-in-hand-are-written-back` | 探到有人要借就放卡键，手上的块没写回也放 | 同上 |
| `devices-let-go-while-lent` | 借出时把设备也放掉 | 同上 |
| `card-key-not-taken-back` | 借的一方还了也不收回卡键、一直不认领 | 同上 |
| `card-key-held-by-another-at-start-ignored` | 起来时卡键被别人持着也当自己持着、照认领 | `a_service_starting_while_another_holds_the_card_key_starts_lent_with_its_cards_open` |
| `card-lease-printed-before-the-card-key` | `lease take` 认领到请求键就打卡串，不等卡键 | `lease_take_prints_the_cards_only_once_it_holds_the_card_key_and_lease_return_lets_both_keys_go` |
| `card-lease-return-leaves-the-renewer-running` | `lease return` 不认领停止键、不等续租进程，续租进程照续 | 同上 |
| `card-lease-renewer-ignores-the-borrower` | 续租进程不看借的进程在不在 | `both_keys_are_let_go_within_five_seconds_once_the_borrower_is_gone` |
| `card-lease-keys-held-by-another-taken-as-claimed` | `lease take` 认领两把键时别人持着也当认领到 | `two_borrowers_on_one_host_get_the_cards_one_after_the_other` |
| `card-lease-borrower-from-the-parent-process` | 借的一方的变量没给时取父进程当借的进程 | `lease_subcommands_refuse_a_missing_or_malformed_borrower_variable_with_the_usage_exit` |
| `card-lease-taken-without-cards-of-this-host` | 这一台没有卡也照借（卡串是空的） | `lease_take_refuses_with_78_when_the_configuration_names_no_card_of_this_host` |
| `card-lease-return-refuses-an-empty-card-string` | `lease return` 的卡串是空的（或只有空白）当用法错、退 2，不还 | `lease_return_takes_an_empty_card_string_and_refuses_a_missing_one_with_the_usage_exit` |
| `card-lease-borrower-whitelist-unchecked` | `lease take` 不照借卡白名单认借的一方，谁都借给 | `lease_take_refuses_a_borrower_whose_executable_name_is_not_on_the_whitelist_and_claims_no_key` |
| `card-lease-waiting-order-ignored` | `lease take` 不看等号的次序，每一回都直接去认领请求键（谁先撞上谁拿） | `the_borrower_that_started_waiting_first_takes_the_cards_first_and_the_later_one_waits_second_in_the_queue` |

测试目标 `library_pending_pool_serves_registered_temporary_libraries`：

| 项 | 弄坏哪一处 | 红的测试 |
|---|---|---|
| `registration-lapse-ignored` | 登记没了也不丢那一份登记的库，照扫、在判的块照写回 | `a_registration_whose_lease_lapsed_drops_its_library_and_discards_its_blocks` |
| `temporary-library-failure-ends-the-service` | 登记的库读写失败照真库那样交回失败，让服务退出 | `an_unreachable_temporary_library_is_named_once_and_the_service_keeps_serving_the_real_library` |
| `real-service-registration-opened` | 指到真服务的登记、键空间前缀头一个字节不对的登记照开 | `a_registration_pointing_at_the_real_service_or_with_a_foreign_prefix_is_not_opened` |
| `real-library-claimed-first` | 各份库认领到的块不合起来按入池序号排，真库的先 | `candidates_of_every_library_are_claimed_in_pool_entry_order` |
| `temporary-libraries-scanned-while-lent` | 卡借出期间登记的库照扫、照认领 | `no_library_is_scanned_while_the_cards_are_lent` |
| `previous-claims-kept-in-temporary-libraries` | 打开登记的库时不放上一个服务进程留在那份库里的认领 | `a_restarted_service_releases_the_claims_its_previous_process_left_in_a_temporary_library` |
| `conclusion-written-into-the-real-library` | 登记的库的认领与写结论的客户端连真服务：结论写进真库 | `a_registered_temporary_library_is_judged_and_written_back_into_that_library_only` |
| `judged-block-matched-to-its-library-by-name` | 判完的块照块名找回它的库，不照入池时记的库 | `blocks_with_the_same_name_in_two_libraries_are_each_written_back_into_their_own_library` |

## 做到哪

- 批次调度、显存账、内存待判池、每卡工作线程、服务主循环：写好，经上面几个测试目标判（上卡的那一条要卡）。
- 工作区：仓根工作区的成员。
- 进程与起停：进程本体、用户级单元、`control` 的 `install`、`start`、`stop`、`check`。
- 登记的临时库：取活循环每一圈列登记、各开一份库、各份库先来先判、结论写回那一份；登记的写法归验证包（「登记的临时库」一节）。经 `library_pending_pool_serves_registered_temporary_libraries` 判（卡与判器、验证包那一侧三样是替身）。
- 核对待判行那一族：扫表、核过没有、认领与续租与放认领、按组读回材料、照显存账借、上卡核、一帧写回（「核对待判行那一族」一节）。经 `library_pending_pool_on_a_temporary_library`、`service_loop_with_judges_off_the_card` 与 `facts_verifier_service_path_agrees_with_the_direct_verifier` 判（核对材料那一样是替身）。
- 从判定库取活（协议 `~/.local/share/singlefs/work/batch53-gpu-service-real-tasks/protocol.md`）：待判行与强制现判凭据照验证包的表 `1D`、`1E`，
  认领与续租、现算判到了哪一步、读回材料、编结论帧调验证包交出的函数；一帧写结论由判定库服务核认领、结论块那一行与删的待判行。
  经 `library_pending_pool_on_a_temporary_library` 与 `library_service_path_agrees_with_the_direct_judge` 判（编结论帧那一样是替身）。
- 卡：`control` 起单元之前对这一台点名的卡清场（不复原），起来照多机配置开这一台点名的全部卡，一直持着到进程停。
- 不按键挑行：待判行不带期待的判器键，三族都只照行的数据算，认领不看判器是哪一版；结论照待判行里写的那一把结论版本键存。
- 认领到之后重读待判行：扫库与认领之间别的持有者写回了的块不再判一遍，刚认领到的当场放掉。
- 放认领：崩掉、被杀的进程留下的认领，下一个进程起来时核它确已不在就放掉；收到停止信号的进程收尾后放掉自己的。经判定库服务的放认领帧，只放点名的持有者名下的。
- 日志：逐次派活的行不进服务日志，按块、按卡打汇总行，认领、别人持着、写回成了、放认领各打一行可数的（「日志」一节）。
- 借卡：取活循环探请求键、借出与收回卡键，二进制的 `lease take`（先照借卡白名单认借的一方，再照等号先来后到）、`lease return`、`lease renew`、`lease list`，`control lease take`、`control lease return`、`control lease list`（「借卡」一节）。经 `card_lease_on_a_temporary_library` 与 `control --selftest` 判（卡与判器是替身）。
## 还没做的

依据：`research/prompts/gpu-judge-service-r2-main-verification.md`「三、改法」与上面那份协议。

- 端到端对拍：判段写待判行、本服务判完写回、判段读回的结论与直接判逐态相同。判段写行等判的入口（验证包 `judge_my_blocks_on_gpu`）与把一条小流录进库的办法
  只在验证包里可调，这个包的测试调不到；要么验证包交出这两样，要么服务装上之后对一条小的崩溃枚举用例开 GPU 跑一趟。
- 被 systemd 拉起的那一次不清场：清过的场一直清着，用户手动复原了清场停下的服务之后，服务被杀再拉起时不再清一次。
- 登记的临时库：上一个进程留在登记的库里的认领，生产里持有者是那份库自己的客户端身份，「在跑」那一行只记真库客户端的持有者，所以起来时放的是真库持有者名下的，各份库自己客户端的认领到租期过了才失效；要立刻放，得把各份库的持有者也写进「在跑」那一行。
- 登记的临时库：验证包那一侧连登记的库用的客户端重连时限是验证包定的（真库同款），登记的库在读写中途没了时，那一次读写最长等到验证包客户端报错才丢那一份，这期间取活循环停着。
- 读回的材料与登记过的流一直留在进程里，不按库里还有没有它的待判行放掉。
- 核对待判行那一族的材料一块读两回：认领时每组照键最小的那一块读一回（建核对器），写回时每块照它自己的行再读一回（验证包编那一帧只收照同一行读回的材料，材料的块键等几样是私有的，服务造不出别的块的）；每回都读回事实表、读回流程、重排计划。
- 核对行那一帧遇到库里已有一份不同的核对行（同一把核对键下两次核出的结论位不同）照验证包编帧的规矩整份换上，服务不另报红。
- `control check` 读 `GPU_VERIFICATION_SERVER_CARD_FAILED` 与 `GPU_VERIFICATION_SERVER_BLOCK_NOT_JUDGEABLE` 两种行。
- 卡为别的原因坏了不重开：那张卡到服务重起都不排；用户重置之后要 `control stop`、`control start`。因显存不够丢的设备才重开，一张卡至多 `DEVICE_REOPENINGS_OF_A_CARD` 次，丢了的设备到服务停才放（判器缓存借着服务持着的那组卡，丢了的那一个在这期间还占着它那一份）。卡坏与判不了只经日志行报，待判行里没有提交它的会话，任务服务照 `control cards-failed` 发给此刻名下有在排、在跑任务的会话。
- 真卡那一份判器缓存（`src/gpu_judges_on_a_card.rs`）的重开设备、接住之后那一次提交、认设备怎样之后怎么做，没在卡上跑过：认设备怎样那一步在假的设备情况上核过，上卡测试归 GPU 服务代跑、接口待定，记成欠的测试。
- 接住 Out of Memory 只看这张卡的设备上记没记过 Out of Memory（验证包 `DeviceConditionOfACard`）；驱动那一份（`MEBIBYTES_LEFT_FOR_THE_DRIVER`）只量过一个进程编一套内核的情形，几十个常驻判器、几套管线时够不够没量过；卡给服务的额度是开卡那一刻的空闲读数，之后不再读。
- 判不了的块（在腾空的卡上也显存不够、判法 panic）放给另一台，另一台也判不了时两台都打判不了那一行、`control check` 都退 1，待判行留在库里，不写红判定；等它的那几片照等，直到有人处置或被统一任务服务打回。
- 核对待判行那一族端到端：判段写核对待判行、本服务核完写回、判段读回那一条只在服务装上之后对开了 GPU 的崩溃放量跑一趟看得到，这个包的测试里没有。
- 被撤的块（判它的卡坏了）、配不出流的块：认领留到租期过了（或这个进程停的时候放），当下不放。
- 一轮把扫到的候选全部认领：先扫到的那一份服务拿走整条流，另一台的卡空着；一轮认领多少没有按卡的余量设上限。
- 写回时重读到待判行已不在（别的持有者先写完了）这一支不另打行；两份结论不同而有一份红时后写的整份换上，归验证包写结论的规矩（`crates/singlefs-verification/src/verdict_store.rs`），这里没改。
- 逐次派活的现场：服务日志里没有了，卡住时看不到最后进入的阶段；要看在测试进程里跑。
- 单元模板里没写 `LogRateLimitIntervalSec`：用户级单元的限流记在整个用户管理器名下，单元级的键认不认没量过。
- 借卡时服务不放设备：借的一方开不开得起服务开着的同一张卡、显存够不够没量过（真环境那一条：借卡白名单里的 E202（GPU与CPU两种模式GC对照组）装置二进制经任务服务交，看 `GPU_CARDS_LEASED_ON_DEMAND` 与服务日志的借出、收回两行）；量到不行再加放设备、收回时重开。
- 借卡只认这一台：`control lease` 不转去第二台，任务派到哪一台就借哪一台的卡；那一台没点名卡的退 78，不挑别的台。
