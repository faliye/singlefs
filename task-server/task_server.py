#!/usr/bin/env python3
"""task-server 的服务：唯一读写库的进程。FastAPI 跑在 uvicorn 上，只开 unix socket（${运行时目录}/server.sock，目录 700、socket 600），不开 TCP。
用户级 systemd 服务 singlefs-task-server.service 起它（python3 task-server/control install 装）；人与 agent 不直接起它，也不直接读写库。
路径、错误类型、重试、资源记录、编译目录、分片、环境变量名单与身份写在 task_server_lib.py 文件头。

  task_server.py serve      常驻；收到 TERM / INT 停下（在跑的任务在各自的单元里接着跑，不杀；服务自己起的探测子进程停掉再退，下面 ③「停服务」）、退 0。
                            同一个 socket 上已有一份在回话退 3；必须依赖、多机配置、签名密钥、每台默认重活线程数不在退 78；socket 路径太长或绑不上退 78
                            （点名那个路径；太长的另报它有多少字节、上限多少，task_server_lib.py 文件头「路径」）；只在自证里认的变量设在了别处退 2；
                            NFS 共享判不过（task_server_lib.py 文件头「NFS 共享」：多机配置没写 NFS_SHARED_ROOTS、哪个根的目录、导出、挂载或两向探针不过）退 78，
                            一句话写明哪个根、哪一项，出路指 task-server/README.md「## NFS 共享」；这一判在 Scheduler 建起之前。
                            NFS 判过之后取共享数据根与工作根（task_server_lib.shared_data_root、work_root：照 .claude/hooks/lib_work_root.py，环境里的 SINGLEFS_DATA_ROOT，
                            没设取多机配置的 SHARED_DATA_ROOT，要落在 NFS_SHARED_ROOTS 某一段下）：取不到退 78，出路点名往 multi-host.env 加 SHARED_DATA_ROOT 那一行；不补缺省。

接口（HTTP over unix socket，JSON；调用方的进程号与 uid 由 SO_PEERCRED 拿，uid 不是服务自己的拒 403；会话号照 task_server_lib.session_of_peer 从进程树上读）：
  GET  /health                    {ok, problem, database_writable, scheduler_seconds_since_tick, tag, queued, running}；problem 只报三样：库写不进；派活判排头能起却连着三轮没起成
                                  （用户：「查清楚为什么总是误报」「根改」：/health 不另判放不放得下，排队线程每一轮排完记下派活给排头的结论（HeadVerdict：判能起而起了、
                                  判能起却没起成及原因、判不能起及原因），只有判能起却没起成的才算，判不能起的（等卡、等负载、交替封组、第二台一轮起满、起之前复核没过……）一律不报，
                                  不认排队原因里的字样；starved_problem）；看进展：某台照公式（room_by_the_formula）放得下排着的最小那条、却连着 PROGRESS_STALL_SECONDS（10 分钟）
                                  一条都没起（不读派活的结论，兜派活自己判错；等卡、等负载、等占盘、封着组等腾空的不算；progress_problem）。
                                  两样都由排队线程每一轮排完算好、存下（starved_text、progress_text），/health 只读存的那一句，不现读余量、不起子进程；queued、running 只数行
                                  （弄坏开关 health-judges-head 下照改前现判排头，health-head-judged-apart 下照改前另判一套，progress-unwatched 下不看进展）；
                                  另带 nfs（每个共享根的四项结论：起来时判的，前三项每 NFS_RECHECK_SECONDS 由复核线程更新）与 peer（第二台用不用得了、原因、从哪一刻起、
                                  最近一次那一趟没跑成；note_peer 记的）：两栏都只读存的，不让 ok 变 false（第二台用不了由 control watch 报）；
                                  再带 paused（暂停着没有）与 pause（{reason, since, allow_session, other_running, text}，text 是「暂停中（原因：…，起于…）、在跑 N 个」，放行了一个会话的接
                                  「、放行会话 <号>」与「、别的会话在跑 M 个」，没暂停是 null）：暂停不让 ok 变 false，暂停中不判「判能起却没起成」，也不看进展（下面 ⑤「暂停」）
  POST /tasks                     {command: [词…], cwd: 绝对路径, environment: {名: 值}} → {id, session, gpu, sharded}；环境变量照名单再核、命令里套了包装或派发器的拒、
                                  命令外层与 bash / sh -c 里写了退役变量「名=值」的拒（task_server_lib.py 文件头「环境变量」）、
                                  调用方给的编译目录指到仓里的拒、派不到第二台的三种与临时 prune-server 包装里会编或要卡的拒（400；这四种的 detail 是
                                  {placement: {problem, next_step}}，下面 ②「提交时拒」）
  GET  /tasks/{号}                任务与它每一次尝试的记录；排着的带 estimated_seconds（预计用时）、estimate_basis（这个数照哪种算法得的：command 按命令、items 按项、none 没有）
                                  与 waiting_for（上一轮判的在等什么）；分片的父任务带 children（各子任务与它们的尝试）；
                                  按项估预计用时的带 item_estimate（估算子任务与它的尝试，不进 children），还在估算的 waiting_for 写在等哪一条估算子任务
  GET  /tasks/{号}/wait?timeout=秒  至多等这么久（不超过 60）：结局有了回 {finished: true, …}，没有回 {finished: false}；结局交给提交它的会话时记成已取；
                                  在跑的那一次正疑似卡住、这一段还没报给提交它的会话的回 {finished: false, suspect: {…}}（下面 ①「疑似卡住」）；
                                  这条任务（分片的连同子任务）有还没报给提交它的会话的要处置的提醒（或档又跨过一档）的回 {finished: false, attention: [{kind, tier, task, basis, why, next}…]}，
                                  结局有了的同样在 {finished: true, …} 里带 attention（下面 ①「要处置的提醒」）
  POST /tasks/{号}/cancel         只许提交它的会话：在队里的当场记 cancelled；在跑的 systemctl --user stop 它的单元、回读 ActiveState 与 cgroup.procs 停了，
                                  再等对账写回结局（至多 cancel_settle_seconds）：结局是 cancelled 才回 {stopped: true}；取消到的时候命令已自己结束、写回的是别的结局
                                  （succeeded 这类）回 409 {stopped: false, outcome}，等不到回 500，都不报停了（cancel_outcome_reply；弄坏开关 cancel-outcome-unchecked）。
                                  拿这条任务的锁判（下面 ③「起」）：判放得下之后、记尝试之前到的不起，记了尝试之后到的等单元起完再停、回读（TS-01）。
                                  分片的一片（子任务号）单独取消：只撤那一片，别的片照跑、合并照起并点名缺它（task_server_lib.py 文件头「分片」的「只取消一片」）。
                                  名下有校验不过的尝试或子任务记录（关联列改过，TS-05）、或任务已因校验不过记了 record-tampered 的，一个单元都不停、回 409，不报停了。
                                  分片的父任务连同它在排、在跑的子任务一起取消：父任务先记 cancel_requested、阶段改 cancelling（不先记结局），在排的子任务当场收尾、在跑的逐个停、
                                  回读，都停了才给整件记 cancelled、回 stopped；有没停的回 500（取消已记下，accepted），再取消照样重查、重停活动的子任务；父任务已有结局而子任务
                                  还在跑的同样停它们（TS-04）
  GET  /sessions/self             这个会话在排、在跑与完成了没人取的任务（子任务不单列，列在父任务名下）
  POST /history/import            照服务自己的环境找峰值表与用时表、导进 history 表（task_server_lib.py 文件头「资源记录」）→ {peaks: {read, added, skipped}, durations: {read, added}, paths}
  POST /pins                      {command: [词…], pinned: true|false} → {command_key, pinned, changed, source, bytes_by_host, problem}：给这条命令（键照 task_server_lib.py 文件头「资源记录」的命令键换算）
                                  钉上或撤掉内存上限的钉值（下面 ②「内存」的钉值；control pin-cap / unpin-cap 调它，服务在跑时也能用，不绕过服务写库）；
                                  哪一台那一项减那台此刻的余量不大于 0 的回 422、detail 写哪一台与出路，不钉
  POST /pins/show                 {command: [词…]} → 同上（只读：钉没钉、钉值此刻多少）
  POST /pause                     {reason: 一句话（可空）, allow_session: 会话号（可空）} → {paused, changed, reason, since, running, other_running, allow_session, allow_before, allow_changed}：
                                  暂停排队（下面 ⑤「暂停」）；给了 allow_session，暂停时这个会话交的任务照常排、照常起；已经暂停着的不改原因与起点，changed 为 false，
                                  给了的 allow_session 与现在放行的不同就换（allow_changed 为 true、allow_before 是换之前的）；不给 allow_session 的不动放行；
                                  只认服务自己的 uid（peer_of，同别的写操作）；control pause 调它
  POST /resume                    {} → {paused, changed, reason, since}：放开暂停（reason、since 是放开之前的）；同样只认服务自己的 uid；control resume 调它

排队（一个后台线程，每 TICK_SECONDS 秒一轮，提交与取消时当场叫醒）：
  ① 对账：每一次在跑的尝试读它的结果（attempts/<标签>/result.json，attempt_runner.py 写）。有结果：照 classify_attempt 定类型、照 retry_decision 定重排还是收尾；
     卡坏另写告警，停用的与告警点名的是清场那一行坏卡清单里的卡，不是分到的那张（task_server_lib.py 文件头「错误类型」card-fault 那一行；
     弄坏开关 card-fault-assigned-card-quarantined 下照改前停用分到的那张）。没有结果：每 UNIT_CHECK_SECONDS 秒读一次单元的 LoadState 与 ActiveState（照输出判）：在跑的照等；
     状态还是「起单元之前」、起跑标记不在、单元不在（LoadState=not-found）的，是服务在写了单元名与起单元之间没了的那一次：起它一次（同名单元在跑时 systemd-run 自己拒，不会起第二份）；
     别的单元不在了又没留结果的，过 LOST_GRACE_SECONDS 秒记 launcher-lost。
     续起「起单元之前没了」的那一次拿这条任务的锁、现读任务：
     取消了的照取消收尾、不起（resume_starting）。起单元回执超时过的（start_uncertain_at，下面 ③「起」）单元不在、起跑标记也没有的，过 START_UNCERTAIN_GRACE_SECONDS
     还是这样才确认没起、记 unit-start-failed；单元在跑的照常 running。
     每一轮另把还没结局的分片父任务与还在估算的父任务各现推一遍（drive_sharded_parents，下面 ④；弄坏开关 sharded-parents-unreconciled 下不推）。
     第二台的结局（task_server_lib.py 文件头「错误类型」）：日志里有 multi-host-run.sh 打的「MULTI_HOST_RUN_STEP step=peer-command event=end … exit=<码>」的，
     照那个码（第二台那条命令自己的退出码）定结局，命令自己退 255 记 command-failed、不重排；没有这一行才是 multi-host-run.sh 那一层没跑成（peer-run-failed）。
     弄坏开关 peer-step-exit-ignored 下不看这一行。第二台那一层的内存峰值与编译目录占盘峰值照日志里的 MULTI_HOST_RUN_PEER_MEMORY_PEAK、MULTI_HOST_RUN_PEER_BUILD_PEAK
     记进那一次尝试（不打的记「没量到」），同本机一样进定额度（弄坏开关 peer-memory-peak-ignored 下不读内存峰值那一行）。
     留下的进程（stop_leftovers）：尝试正常结束（有结果、没被停）之后，单元里的由 attempt_runner 退出之前自己停掉、回读，记进 result.json 的 unit_leftovers
     （它一退 systemd 就收单元、当场杀掉剩下的，服务那时已回读不到）；外层 scope（结果里记的 scope，或按尝试标签在 slice 里找到的）由服务回读 cgroup.procs，
     还有进程就对这一次自己的 scope 名发一次 systemctl --user kill --signal=SIGKILL，至多等 LEFTOVER_STOP_SECONDS 秒回读 cgroup 空了没有；
     结局（与重排）的说明前面写留下了几个、哪几个进程号、回读空了没有。弄坏开关 leftover-processes-kept 下两处都不停（runner 照 spec 的 leftovers_kept）。
     收尾回读还有进程或读不了时，尝试保持 starting/running 并写 cleanup_pending，资源和显卡占用继续保留，后续对账重查。
     新的远端尝试另有 peer-cleanup.json：起远端命令之前先落 pending，确认进程停下并核目录没有持有者后才写 confirmed；
     失联保留远端目录和 PID/启动时间/退出码，包装已退出时对账经 multi-host-run.sh --recover-cleanup 重查。升级前无此回执的尝试按旧协议处理。
     跑得久而失败（finished_preface）：结局不是 succeeded、这一次跑超过 task_server_lib.LONG_RUN_SECONDS、命令又没声明续跑的，说明前面写「这条跑了 N 分钟、没有续跑：
     重交会从头跑」（task_server_lib.py 文件头「续跑」）。
     命令退 75（prune-server 不可用）：记 prune-server-unavailable，另起一个线程在仓根跑一次 python3 prune-server/control check（PRUNE_SERVER_CHECK_SECONDS），
     不通写告警 prune-server-unavailable；只报，不起、不停 prune-server（check_prune_server）。
     卡住（用户：「就是距离上一次输出日志10分钟没有日志输出」，又选「日志与 CPU 都不涨才停」，又说「至少要延迟十分钟后在判定吧」）：尝试起来后先宽限
     STALL_GRACE_SECONDS（600）秒不判；宽限过了，日志距上一次增长满 STALL_SECONDS（600）秒、
     这一个窗口里 scope 的 CPU 平均不到 STALL_CPU_CORES 个核，服务停掉它、记 stalled（CPU 在涨的是在算，照跑，再看下一个窗口）、
     照常经 wait 叫醒提交它的会话；CPU 本机读 attempt_runner 写的 cpu_usec（各 scope 合计），第二台读日志里 multi-host-run.sh 每轮打的 MULTI_HOST_RUN_PEER_CPU 一行，
     这一行与 MULTI_HOST_RUN_STEP 行不算日志在涨。已跑多久、记录用时、日志多久没涨、到此刻平均在用核数
     只写进 task status（running_progress），核数过少不停；头一回量一次尝试（服务刚起来对账在跑的、或宽限刚过）时「日志上一次涨」取日志文件的修改时刻
     （读不到取此刻；服务重启不把日志多久没涨清零），CPU 的窗口照旧从此刻起量；远超记录用时只照下一句停：
     已跑超过记录用时的 OVERRUN_STOP_FACTOR（5）倍、日志又满 STALL_SECONDS 秒没涨的，不看 CPU 也停掉、记 stalled（用户：「工具里不限时」，占着 CPU 的死循环靠这一条停，整条重来）；没有记录用时的不判这一条。服务重启不杀在跑的任务（它们在各自的单元里），重启之后照这一步对回来。
     顶在上限上卡住（judge_stall_stop；任务 1866 第 4 次：上限 13824M、峰值 13.50 GiB、日志 1288 秒没涨）：照上面两条判卡住的那一刻，外层 scope（尝试目录的 scope 文件记着）
     的 memory.peak 已到这一次上限的 STALL_AT_CAP_FRACTION（98%）的，照样停掉，但不记 stalled：记 memory-cap-hit、照重试规矩乘 1.5 重排（钉值的打回 memory-cap-at-maximum），
     结局与重排的说明前面写「顶在上限上卡住：…」；峰值读不到（第二台、还没采到 scope）的照记 stalled。
     疑似卡住（用户：「加 gpu任务看显卡。但是不自己终止，告诉负责的对话 自己检查」；check_suspect）：同一宽限过后，整个单元的 CPU 满 STALL_SECONDS 秒平均不到
     STALL_CPU_CORES 个核、要卡的这段时间它那张卡上又没有它的计算负载（本机读 nvidia-smi -i <序号> 的计算进程里有没有这一次尝试的子孙进程；第二台与量不到的当有负载，不报），
     不管日志涨不涨：不停它，尝试记 suspect（依据）、写告警 suspected-stall、叫醒在等的 wait（它打 TASK_SERVER_SUSPECT 一行、退 task_server_lib.SUSPECT_EXIT，任务照跑，
     再起 wait 接着等）、task status 写「疑似卡住」与依据。一段只报给提交它的会话一次（服务内存里记报过的段号）；之后 CPU 又涨了（相邻两次读之间平均不少于 STALL_CPU_CORES 个核）
     或卡上又有了它的负载，这一段结束、从此刻再量下一段；读不到 CPU 窗口作废重量。这一次照上面两条停了的不再报疑似。
     要处置的提醒（用户在弹窗里选「task wait 自己报」；三方判决 research/prompts/watchdog-task-half-r1-main-verification.md「三、怎么办」岔路表甲；attention_items）：
     删掉看门狗盯任务那一半之后没人报的五种情形，加改主工作区的命令没正常结束那一种，服务不停、不改任务，只在 wait 那一侧交给提交它的会话。分片的任务连同它的子任务一起判（提醒点名的 task 是子任务号）：
       queued-too-long   任务在 queued：从提交（重排的从上一次尝试结束）起排满头一档 QUEUE_ATTENTION_MINUTES 分钟，依据写已排多久与排队原因（waiting_reasons），出路写看 status、等不起就取消改小重交；
       overrun           在跑、有记录用时（estimated_seconds）：这一次尝试已跑的秒数到了 max(OVERRUN_ATTENTION_FACTOR × 记录用时, STALL_GRACE_SECONDS)；日志在涨时服务永远不停它，靠这一条叫人；
       card-replaced     尝试的结局是 card-fault 且清场点名了坏卡（faulty_cards），后面还有尝试、或任务正排着等换卡（结局可能是 succeeded）；出路是请用户重置那张卡；
       cleanup-pending   在跑的尝试 cleanup_pending（命令已退出、清场回读没收干净，资源占用保留；finish_attempt）；
       shard-advance-failed  drive_sharded_parents 推父任务抛错（note_advance_failure 记在内存里，推通了删）；
       workspace-write-interrupted  改主工作区的命令（modifies_workspace）有了结局而没正常结束：结局是 WORKSPACE_WRITER_INTERRUPTED_OUTCOMES 之一（撞上限、被挤掉、显存不够、
         第二台没跑成、撞顶、取消、卡住被停、对不回来）或命令被信号杀（command-failed 而退出码 128 + 信号号，SIGNAL_EXIT_CODES），可能写了一半
         （用户：「这个问题很严重 因为可能会导致补丁没有正常写入」）；依据写哪一次尝试、什么结局，出路点名要核的文件或补丁目录与 git status。
         apply-writer-patch.py 有结局文件（补丁目录里的 PATCH_WRITER_OUTCOME_NAME）的照它判：服务起一次它的 --read-outcome，打全了（退 0）、已退回（退 1）的不报，
         别的报、依据带它打的那几行；结局文件不在的不报（它在动手换之前写开始与文件行）；带 --dry-run 这类只读参数的不报（workspace_write_item，结了局的只核一次）。
         只报给有结局的任务，所以这一行打在 TASK_SERVER_REASON 之前、退码仍是任务的（下面「wait 读到还没报的提醒」）。
         弄坏开关 workspace-write-interrupted-unreported 下不报，apply-outcome-ignored 下不读结局文件、一律照可能写了一半报。
     档（attention_tier）：到头一档是 1，到头一档的 2 倍、4 倍……每翻一倍加一档（cleanup-pending 的档是尝试的次数，card-replaced 的是换过卡的尝试数，shard-advance-failed 的是第几回开始推不动）。
     同一条任务同一种提醒、同一档只报一次：报给提交它的会话就记下报过的档（服务内存里，同疑似卡住；别的会话来等照样看得到、不记），档跨过下一档才再叫；服务重启后再报一次。
     wait 读到还没报的提醒就返回：任务没结局的回 {finished: false, attention}，客户端打 TASK_SERVER_ATTENTION 一行、退 ATTENTION_EXIT；结局已有的（换卡重排跑成）
     {finished: true, attention} 打在理由行之前，退码仍是任务的。阈值取值的理由写在常量上（QUEUE_ATTENTION_MINUTES、OVERRUN_ATTENTION_FACTOR）。
     服务健康（服务不通、库写不进、单元不在、NFS、第二台用不了）不归 wait，由 task-server/control watch 盯（主 agent 用 run_in_background 起，退出即处置）。
     弄坏开关 attention-queue-unreported、attention-overrun-unreported、attention-card-replaced-unreported、attention-cleanup-unreported、attention-shard-advance-unreported 各关一种，
     attention-finished-unreported 下结局有了的 wait 不带提醒，attention-reported-every-wait 下报过的照报。
  ② 排队：在排的任务按任务号从小到大逐个看（同一个队，轻重不分，不另设延迟；重排的回到原来的位置；分片的子任务与普通任务同一个队）。行核不过的不起、记 record-tampered、写告警。
     两台同一段逻辑（用户 2026-11-02：「两台型号都是一样的 根本没有必要写 if 第一台 else 第二台这种代码」）：记账、判放不放得下、判卡住与 CPU 用量、记峰值、
       上限收法、编译目录、挑台与盘余量对每台走同一段，台是参数（多机配置 HOSTS 里的地址、那台的总上限与余量、编译根）；按台不同的只在「这一台怎么读、怎么起」的接口
       Machine 里（LocalMachine：服务所在这一台，直接读 cgroup、起内存包装；PeerMachine：HOSTS 里另一台，经 multi-host-run.sh --peer 起、经 ssh 探测），
       台的先后是 MACHINE_ORDER。只给自证证红的弄坏开关里照改前的样子分台的那几处（host-lists-restored、workspace-writers-local-only、peer-overflow-blocked、
       margin-local-for-both、pinned-cap-this-host-only、card-labels-all-local）不算。
     机器：照 task_server_lib.py 文件头「挑机器」：放得下的几台里挑放下之后剩余占比大的（按余量均衡）；撞上限重排不钉台，到了这台最大才换到总上限更大的那台；
       没有只派一台的名单（用户：「【只能在本机跑】 是不能接受的。必须尽快解决 让错误暴露错来」）：计划、合并与各片也不钉台，哪台由余量定；派到第二台跑红的照红，
       由脚本那一侧修，服务不替它挡（弄坏开关 host-lists-restored 下照改前放回 RESTORED_HOST_LISTS 那几张名单与计划的 pin）。第二台在同一棵共享的树里、同一个绝对路径上跑
       （仓、工作根与会话记录目录经 NFS 两台挂在同一绝对路径）；派不派哪一台只看那一台此刻用不用得了与余量（host_eligible），派不过去的三种在提交时就拒了（下面「提交时拒」）。
       改主工作区的工具（hook-swap.py、apply-writer-patch.py、stage-mine.py、scripts/kb/ 下的、不带 --skip-apply 的 merge-and-verify）派得到第二台（改的就是同一棵树）；
       同一件的兄弟片不避开彼此（task_server_lib.py 文件头「挑机器」）：放不放得下只看内存额度与空卡，同一台能同时跑几片。
       GPU 任务照下面「GPU」挑卡。
     提交时拒（用户：「改成提交时当场拒（推荐）：不再悄悄改派本机：碰到这三种就在提交时报错、点名是哪个路径或哪一族、出路写怎么改，任务不跑。」；placement_refusal）：
       POST /tasks 收到时判，命中头一种就回 400、不进库、不起尝试，task submit 退 3、打那一句与出路；分片的子任务不另判（父任务提交时判过）。
       ① 共享根外的路径（find_outside_shared_path）：工作目录与命令里的绝对路径（整个词、「名字=绝对路径」的值、bash -c 词里的每一段 QUOTED_WORD_PATH；
          两台相同的系统路径 HOST_INDEPENDENT_PATHS 不判）有不在多机配置 NFS_SHARED_ROOTS 任何一段下的：点名那个路径，出路是改放某一段下。
          落在 VOLATILE_ROOT（/tmp）下的同样拒，那一句另说重启会清 /tmp（问题 93，用户：「防止重启后 内容丢失，但是被认为做完了」）。弄坏开关 outside-path-accepted 下不判。
       ② 共享根（shared_roots_problem）：这一次要交给命令的 SINGLEFS_DATA_ROOT、SINGLEFS_WORK_ROOT（服务自己的，或任务带的白名单变量）有不在 NFS 共享根下的：
          点名变量与值，出路是配多机配置。弄坏开关 shared-roots-outside-accepted 下不判。
       ③ 判定库（verdict_library_refusal）：带不带判定库只照登记表 .claude/gate.d/verdict-library-scripts.tsv 判（registry_families，不另问别的脚本）：
          命令里有词点到登记表里带判定库的一行、而那一行第三列不以 prune-server 起头的：点名那一族（登记的路径），出路是那一族接进 prune-server、把第三列翻成 prune-server；
          登记表读不了、写坏了的同样拒（判不了）。临时 prune-server 的包装剥开、照原命令认。弄坏开关 verdict-library-family-accepted 下不判。
       ④ 临时 prune-server 包装里会编或要卡的（temporary_service_heavy_refusal；用户：「旧脚本不兼容 我再说一次 如果错了让 对方重新提交」）：这层包装
          （python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service [--server-binary <路径>] -- <原命令>）只给脚本自证用；
          照 task_server_lib.unwrapped_temporary_service 剥出原命令（只用来判，收下的照旧整条执行），原命令照 compiles 的认法有会编的 cargo、或照 gpu_need 的认法要卡的，
          点名是哪一种与原命令，出路是去掉包装重交（验证包的 cargo test 对真服务判）。服务不剥包装去迁就：编译目录、分卡与清场只照外层那一条给，
          包装里的认不进来（任务 4229 包装里套 cargo test --release -p singlefs-verification --features gpu，记成不编、不要卡，编进了仓根 target/）。
          包装自己要现编的另算（量到的：任务 6778 派在本机、退 0，跑的是改包装之前的旧版，里层命令与 6834、6835 不同，不能当本机的样本；
          任务 6834、6835 同一条命令派到第二台、退 0，第二台的编译目录由 multi-host-run.sh 照例给了。推的、没有任务实测过：compiles 为 false 的包装派本机，
          服务不给编译目录，包装的 resolve_binary 没有 CARGO_TARGET_DIR 就退 2）：
          不带 --server-binary 的包装在服务给的编译目录里现编 prune-server，compiles 把外层这一条认成会编的（lib.temporary_service_builds_server），两台都由服务给编译目录；
          带了 --server-binary 的照旧不编。弄坏开关 temporary-service-build-unrecognized 下不认。
          自带临时服务的两份脚本单独的 --selftest（prune_server_client_selftest.py 还有 --selftest-families、prune-server-load-simulation.py）不带 --server-binary 与 PRUNE_SERVER_BINARY 时
          同样自己现编，同样按会编的算（lib.own_server_selftest_split），两台都给编译目录；带了二进制的照旧不编。弄坏开关 temporary-service-own-server-unrecognized 下这两份不认。
          弄坏开关 temporary-service-heavy-accepted 下不判（改前的样子：照外层收下）。
       第二台用不了要告警（note_peer）：余量那一轮判出它用不了（连不上、余量查不成、多机配置判不过、NFS 复核判出没挂上共享根、盘余量不到 DISK_FLOOR_BYTES）、
       或那一趟没跑成（peer-run-failed：ssh 被重置、第二台那边的准备没过），写告警 peer-unavailable（用得了变用不了、原因变了才再写一条；盘不够这一种按 DISK_FLOOR_KIND 只写一条），
       /health 的 peer 一栏写明原因，control watch 报出来；这时只在本机排着的任务排队原因带「第二台此刻用不了：<那一句>」。
       盘余量（Resources.disk_problem；task_server_lib.py 文件头「挑机器」的「盘余量」）：两台同一个下限。读每台余量的那一次（同一个缓存周期）另判那台编译目录所在盘
       （本机是 TASK_SERVER_BUILD_DIRECTORY 所在盘；第二台是多机配置 BUILD_ROOT_BY_HOST 里它那一项，multi-host-run.sh 在它下面建每一趟的本地目录）的余量：
       报告里有 DISK_REPORT_KEY 照它，没有就照那一台的接口现量（Machine.disk_free：本机读盘，第二台起 PEER_DISK_PROBE_TEXT）；不到 DISK_FLOOR_BYTES（50G）或查不成，
       这一轮不给那一台的报告、不派它，原因写「<台>盘只剩 X（下限 Y）」；够了下一轮自己认回。弄坏开关 disk-floor-unchecked 下不判。
     内存：上限与记账只照服务自己库里的尝试记录（用户定不再读内存包装的峰值表与用时表；task_server_lib.py 文件头「资源记录」）：同一命令键最近 RECORD_WINDOW 次
       量到的内存峰值（量不到的那几次不算）取最大，撞了上限的那几次按那一次的上限算；上限 = 预测 × 1.5（按 MiB 向上取整）。只用跑完的（退 0 的与撞了这一条上限的），
       别的结局的峰值不进、不占名额。一次跑完的都没有的，预测、上限与记账都取 task_server_lib.UNRECORDED_MEMORY_DEFAULT_BYTES（4G）；只有没跑完的尝试、
       最近一次命令自己退非 0（command-failed）而量到了峰值的，上限不低于那个峰值 × 1.5（failed_peak_floor；任务 2045 第 1 次峰值 3.34 GiB 退 1，
       同一命令键的任务 2159 从 4096M 起、跑 8007 秒撞顶）。撞了 250 由 retry_decision 乘 1.5 重排；重排的照 retry_decision 定的上限（computed_prediction）。
       每条任务的上限不低于 MEMORY_FLOOR_BYTES（4G；问题 92，用户：「按照我们的计划 默认是要给4gb的」）：重排、判红峰值定的上限低于 4G 的，上限与记账都取 4G，预测照记录；
       钉值与服务自己的峰值记录定的除外（memory_floor_applied；用户：「如果有真实的使用数据 按照真实的使用数据来」：有记录的上限照峰值 × 1.5，撞了照重排乘 1.5、
       重排那一次兜到 4G）。弄坏开关 memory-floor-ignored 下重排定到低于 4G 的也照用，recorded-cap-floored 下照改前记录定的也兜到 4G。
       同一命令键最近一次结束的尝试撞了上限（lib.RECORD_CAP_HIT_OUTCOMES）的，新提交的那一条第一次尝试照「重排那一次」办（latest_attempt_hit_cap；用户：「这个问题很严重 交给 task-server去处理」）：
       服务不自动重排的命令（改主工作区的）撞了上限，调用方自己再交一遍在服务看来是新任务，不照这一条就只有峰值 × 1.5 的小上限、一撞再撞；这时上限与记账兜到 4G，
       比 4G 大的照撞过那次的上限 × 1.5，prediction 记 CAP_HIT_RERUN_HISTORY。没记录的从 4G 起、有真实峰值记录而最近一次没撞上限的不兜，两条照旧。
       弄坏开关 cap-hit-rerun-unfloored 下不兜。
       编译族（task_server_lib.py 文件头「资源记录」的编译族；records/2026-10-31-task-server巡检.md 批 60：会编的 cargo 同键没记录只从 4G 起，一天里撞上限 78 次）：同键一次跑完的记录都没有的，
       照这一条的编译族（提交时记的 build_family：会编的 cargo 取包、档、特性那几样，入口脚本照文件头声明）最近 RECORD_WINDOW 次跑成而量到峰值的尝试
       （别的命令键的也算）取最大 × 1.5 起步，比原来的上限大才换（raised_prediction：只往上抬），prediction 记 BUILD_FAMILY_HISTORY（build_family_peak）。
       有自己记录的照自己的，不看族。弄坏开关 build-family-ignored 下不看族。
       同一计划共用峰值（同一批：同一计划 11 片各一个键、各自从 4G 爬，一天里撞上限 48 次）：分片的一片（role=shard）不是重排的那一次时，同一父任务名下计划那一步与别的片
       已结束的尝试里跑成的峰值、撞了上限的那一次的上限，按结束时刻取近 RECORD_WINDOW 个、取最大 × 1.5，比这一片照自己记录（或没记录的 4G）定的上限大才换
       （只往上抬，不压到 4G 以下），prediction 记 SHARD_FAMILY_HISTORY（shard_family_peak）；排着的片起之前现算（fresh_prediction），兄弟片跑完之后起的看得到它的数。
       重排的那一次照 retry_decision 定的上限，再照下一段往上抬。弄坏开关 shard-siblings-unshared 下各片照自己的键。
       按近几次最高起跑（task_server_lib.py 文件头「资源记录」的同名一段；history_raised_prediction）：上面定出、兜过下限之后，新提交的头一次与撞上限之后的每次重排
       都照这把键近 RECORD_WINDOW 次里最高的一次（key_recent_peak：撞过的上限、跑成的峰值，按结束时刻从新到旧取，与上面定额度同一个窗口）× 1.5 往上抬，
       重排的那一次另照编译族与兄弟片往上抬，prediction 记 KEY_HISTORY_PEAK_HISTORY；没记录的照旧 4G。能升能降（用户：「资源的使用记录 服务要有升降的能力 不能只升不降」）：
       窗口里刚撞上去的那一档新提交与重排都看得到，出了窗口的不再算，后面几次都跑得小，起点跟着降。弄坏开关 history-peak-ignored 下不看，
       history-peak-unwindowed 下照改前不截近几次、全部记录里取最大（一次大峰值永远压着后面），兄弟片那一段同样不截。
       显存预测取 host-dispatch.py 的 DEFAULT_GPU_PREDICTION_MEBIBYTES。
     余量（用户：「这部分最大上限看起来要动态计算」「2gb 要动态计算 … 向上取整」；host_margin、host_capacity、host_limit）：每台单条能给到的最大上限 =
       那台 slice 的总上限（多机配置 SLICE_MEMORY_CAP_BY_HOST 里那台那一项，配多少就是多少；第二台另取它那一项的小者）− 那台此刻的余量。
       余量是那台此刻 singlefs-heavy.slice 里回收不掉、又不归任何在跑任务的那一截，按 1 GiB 向上取整（scripts/hosts/memory_peaks_lib.py 文件头「余量」：怎么量、
       读哪几项）；它随那台的余量报告一起读（Resources.snapshot，第二台照 host-dispatch.py 的 probe_peer 在第二台上量），两台各用各的，不存、不跨轮缓存：
       派活、记账、整片的界、钉值、撞上限重排与「上限到了总上限」、起之前复核都照判的那一刻那一份报告里的那一台的数。配置里没有余量的键（写了照多机配置判法 crash-point-replay-shard-configuration-check.sh 认不得的键判不过，不另认）。
       哪一台量不到余量（没报告、cgroup 读不出、第二台探测失败）、或总上限不比余量大、或钉了的命令在那台的钉值不大于 0：那一台这一轮不派，排队原因写哪一台、为什么；
       能派的每一台都这样：不起，结局记 run-condition-unmet（退 78），说明写每一台的原因与出路。不退回固定的数或旧值，也没有让它退回的配置键与运行开关（下面几个弄坏开关只给自证证红）。
       弄坏开关 margin-fallback-fixed 下量不到的退回 2G；margin-local-for-both 下第二台也用本机的余量；margin-unmeasured-waits 下每一台都量不到时照排着等；
       booked-up-to-slice-total 下单条的上限不减余量（照改前收到总上限；钉值照减）。
       准入按上限记账（用户：「当然， 谁超了杀谁 不要连累别人」；booked_at_cap）：交给 judge_host 的记账量 = 这一次在那台上的上限（收到那台总上限减那台此刻的余量为止；
       峰值本来就比总上限大的照原样、照旧判永远放不下），在跑的那几次同样按各自的上限算，每台在跑任务的上限之和不超过那台 slice 的总上限减那台此刻的余量
       （judge_host 判记账时 slice 放得下的另减余量）。
       判放不放得下照 host-dispatch.py 的 judge_host，减去的「还没涨到量」那一截 = 服务自己账上这台在跑的每一次尝试 max(0, 上限 − 它的 scope 此刻回收不掉的用量)
       加派发器账上刚派出去的那几笔（server_pending）。两台同一段：用量照这一轮读那一台余量的那一份报告（probe_local / probe_peer 量的
       memory_peaks_lib.SCOPES_REPORT_KEY，第二台是同一次 ssh；scope 照里面进程环境的 TASK_SERVER_ATTEMPT_LABEL——每次尝试两台都给，task_server_lib.ATTEMPT_LABEL_VARIABLE——
       对到尝试的 label 上，memory_peaks_lib.py 文件头「在跑的 scope」；几个 scope 带同一个标签（命令里再套内存包装另开的平级 scope）取用量最小的那一份，不往少里记待涨量）。
       报告里没有这一条的 scope（还没起来、读不了、这一轮那台没报告）记整份上限，不记 0。第二台那一层内存包装不排队、不记账（run-with-memory-cap.sh 文件头「排队、记账、挑机器、
       定额度都归仓根 task-server/」那一句、multi-host-run.sh 文件头 ⑤），记账只在这一处，不重复扣。准入只在这里判一处：
       放得下才起；放不下的留在库里排着，不起包装进程去空等；排头放不下时后面放得下的照先起（回填，用户定「保守回填」），另照下面两段交替与满时限；
       没满时限的排着的一律不留账（用户：「现在都在数据库里面 记什么账？？？？  每一条能不能启动 是启动时候才查的 记账做什么？」）；GPU 任务在等空卡时不挡后面不要卡的任务，后面要卡的只在它等的那几台（卡够它张数的台）上让它、别台有空卡照起
       （任务 7615 要第二台 3 张卡在等，7618 要 1 张也被挡住；弄坏开关 card-wait-blocks-later-cards 下照改前前面一条等卡就挡住后面全部要卡的）。
       哪台都等多久也放不下记 never-fits；点名第二台而它用不了同样记 never-fits。
     整片与碎片按台交替（用户：「最好是 碎片50gb任务组 整片50gb 碎片50gb任务组 整片50gb 这样交替」「拼装碎片任务的时候也按照 85%一组算」；Alternation）：
       每台的界 = 可用上限（那台总上限减那台此刻的余量，booked_at_cap 收到的那个数）× WHOLE_FRACTION（85%，50G、余量 1G 的台是 41.65G）；这一条在那台上的记账不小于界的是整片，别的是碎片。
       没满时限的照交替排：碎片按队列先后装进那台的这一组（一组的账 = 这一组里起过的碎片各自记账之和），再放下一条就超过界、而那台有没满时限的整片在等时封组，
       封了组的那台不再放碎片；没有整片在等就开下一组。一条整片只在一台上等腾空、只封那一台（Alternation.waiting_hosts；用户：「一个 46.4G 的整片任务，
       把大量几 GB 到 1 GB 级的小任务整体挡在后面。这是调度策略造成的，不是资源真不够。」任务 7585 两台都封组）：上一轮已在一台上封了组等它、那台它还算整片的，
       接着等那台；别的在它算整片的几台里挑此刻离放得下最近的那台（它在那台的记账减那台照公式 room_by_the_formula 还剩的最小，一样小取 candidate_hosts 里排在前面的）；
       别的台不为它封组、照常起碎片，它在那几台放得下照样起。要卡的整片只算空卡凑得够它要的张数的台（卡被占着时腾空内存也起不了它），哪台都凑不够的不封组。
       弄坏开关 whole-seals-every-host 下照改前它算整片的每一台都封，card-waiting-whole-seals 下要卡的不看空卡。整片在这一组已封、或起过碎片、或没有碎片在等那台时就起，放不放得下只看内存（booked_at_cap），不等那台腾空（在跑的小任务可能跑几个钟头：
       任务 2461 留成孤儿的两小时里本机只用 2G、10 条排着等「腾空」）；弄坏开关 whole-waits-for-empty-host 下照改前等腾空；
       起了整片那台的组清零，整片跑完回到下一组碎片。碎片能不能挨着在跑的整片起只看内存。组的账记在服务的内存里（重启从零起）。
       交替挡住的排队原因都带「在等这台腾空」；/health 不另判、不认排队原因里的字样，只读派活给排头的结论（文件头「接口」）。弄坏开关 alternation-disabled 下不交替。
       插空（用户：「我们应该允许长任务时候 可以插入短任务。 这个需要按照历史时间来对比计算。 如果短任务的历史最新执行时间 在长任务剩余执行时间内 那么可以不被锁定。」；
       Alternation.gap_fill）：封了组的那台，一条碎片先看它占不占排头整片将来要的那部分内存（用户：「多个短任务队列的时候 一个短任务进入 会导致其他时间比他长的短任务都进不去」）：
       排头整片（在等那台的整片里任务号最小的那条）起来之后，照用户定的公式（room_by_the_formula：配置的总上限 − 余量 − 在跑的上限之和）那台减去排头在那台的记账
       还放得下这一条的，不看时长，交替不挡它；会占排头那部分的，预计用时（estimated_seconds；task_server_lib.py 文件头「资源记录」的预计用时，点了 --item 的按项估）
       不是 None、又不超过挡路时长的，交替才不挡它。交替不挡的内存照常判（放得下才起，verdict_within_the_formula 照用户定的公式），起了照常记它的上限、组照封着
       （同一轮里先插进去的那几条已记进在跑的上限之和，后面的照减过的判）。挡路时长 = 那台此刻在跑的尝试各自还要跑多久里最大的一个（blocking_seconds）：
       预计用时减已跑；没有预计用时、或已跑过预计用时的取已跑（用户定：至少还要跑它已经跑了的那么久）。会占排头那部分而没有预计用时的照旧封着。
       插空起的那一次尝试记 gap_fill（插空的那一句：排头起来之后还剩多少，或挡路时长与这一条的预计用时）；没插进去的排队原因在封组那一句后面写为什么
       （没有预计用时，或预计用时超过挡路时长，点名两个数；会占排头那部分的另写排头起来之后照公式只剩多少），插得进而内存放不下的写插空那一句与内存那一句。
       弄坏开关 sealed-gap-not-backfilled 下封着一律不放，sealed-gap-length-ignored 下有预计用时就放、不比挡路时长，
       blocker-remaining-unbounded 下没有预计用时、或已跑过的挡路尝试当成没有尽头，gap-fill-room-ignored 下照改前一律看时长、不看排头起来之后还剩多少。
     满时限（Scheduler.aged、queue_order）：提交满 AGED_WAIT_SECONDS（两小时）还在排的排到最前、先判，不归交替管；放不下时不挡后面、不留份，后面放得下的照队列先后起
       （用户：「现在不存在这个问题。 现在小任务插入 内存和时间两个小件。 因此不存在大任务被卡死不能进的。 因此 保底都可以删了。」，records/2026-10-31-task-server巡检.md 批 47）。
       弄坏开关 aged-task-not-prioritized 下不往前排。
     钉值（用户：「变异的内存上限直接按照 根目录设置的50gb走」「50 这个是读取的配置 不是随便给的」；pinned_prediction）：库的 history 表里 state=pin 的一行按命令键
       （task_server_lib.py 文件头「资源记录」的命令键）记「这条命令的上限取多机配置 SLICE_MEMORY_CAP_BY_HOST 里派到的那一台那一项」，不记字面的数；定额度先认它：
       预测、上限与记账都取此刻现读的那一项（configured_slice_caps：按台的键照 task_server_lib.py 的 configured_by_host 切、判，地址照 Resources.addresses 映到本机与第二台；
       每一项正整数加 K / M / G / T，写法同 run-with-memory-cap.sh 读它的那一句），判哪一台放不放得下就换成哪一台那一项减那一台此刻的余量
       （booked_at_cap、pinned_on_host，Prediction.memory_by_host 存的是配置里那一项，余量每一轮现量；上面「余量」），
       学到的峰值、撞上限重排的 next_cap 都不顶掉它；撞了上限不放大，照 memory-cap-at-maximum 打回；
       哪一台那一项不比那台此刻的余量大（减完不大于 0）：control pin-cap 当场拒（POST /pins 回 422、说哪一台、出路是改大多机配置里那一项），不钉；
       已钉的命令派活时那一台照「余量」那一段不派、写原因，不兜底、不取下限（弄坏开关 pinned-nonpositive-accepted 下照钉、照派）；
       配置读不到（没有这个键、写法不对、少一台、多了不在 HOSTS 里的地址、认不出 HOSTS 里哪一项是这一台、文件读不了）的这一条不起，结局记 run-condition-unmet（退 78），
       说明写读不到的原因与出路（改对多机配置的 SLICE_MEMORY_CAP_BY_HOST，或 python3 task-server/control unpin-cap <命令…> 撤掉钉值，再重交）；不退回学到的记录。
       弄坏开关 pin-unreadable-predicted 下照改前这一回照常预测、说一句；pinned-cap-this-host-only 下两台都取服务所在这一台那一项；
       pinned-cap-at-slice-total 下照那一项记满、不减余量。
       python3 task-server/control pin-cap / unpin-cap <命令…> 经 POST /pins 写（服务在跑时也能用）；status 写明这一条是钉值。
     预测每条只算一次（用户：「每个任务在运行前读一次 来确认可用的资源量才是对的吧」）：一条在排任务的内存预测（prediction_of 的结果，上面「内存」那一套）
       第一次算出后存下，之后每一轮直接用，不再读峰值表、不再扫记录；next_cap 变了（撞上限放大重排）作废重算；收尾、不在队里了就删掉存的那一份。
     余量一轮只读一次（PassView）：每台的余量（Resources.snapshot）、服务账上还没涨到量的那一截（server_pending）在这一轮第一次用到时读，
       这一轮里各条共用；这一轮起了一条，它的记账与显存记账加进这一轮那台的待涨量，后面的照加过的判，同一份余量不判给两条。/health 判排头另读一份。
     编译目录占盘（records/2026-10-31-task-server巡检.md 问题 76；build_disk_problem）：会编的、或这条命令量过编译目录占盘的，那一台盘余量（本机是
       TASK_SERVER_BUILD_DIRECTORY 所在盘，第二台是多机配置 BUILD_ROOT_BY_HOST 里它那一项所在盘，报告里都是 DISK_REPORT_KEY）减去服务账上
       那台在跑的尝试编译目录还要涨的（起的时候记的占盘 − 此刻量到的，量不到的照记的算；server_disk_pending、Machine.build_bytes）要放得下「这条命令的占盘预测 + BUILD_DISK_FLOOR_BYTES」，
       放不下就排着、排队原因点名「编译目录占盘」；预测是同一命令键最近 RECORD_WINDOW 次跑成而量到了占盘峰值的最大值（取法同内存，build_disk_prediction），
       起了的尝试记 disk_booked。弄坏开关 build-disk-unpredicted 下不判（改前只看第二台的下限）。
     起之前复核：判出要起的那一条，起之前现算一次它的预测、现读一次它要去的那台的余量与待涨量（缓存作废），按现读的数再判一次那台放不放得下；
       放得下才起，上限与记账照现算的；放不下的留在队里，排队原因写「起之前复核没过」与现读的那一句；现读的那份换进这一轮，后面的照它判。
     GPU：认命令（gpu_need：带上卡特性（GPU_FEATURE_NAMES 里的 gpu、tests-on-a-real-gpu，<包>/<名字>，--all-features）而跑编出来的代码的 cargo——子命令是 test、run、bench（连同别名 t、r）、`--` 之前不带 --no-run；
       clippy、check、build、doc 与 test / bench --no-run 这类只编不跑的带 gpu 特性也不要卡、不清场——、gpu-quiesce-run.sh、crash-amplification*.sh；
       走 python 入口、按参数认的（PYTHON_GPU_ENTRIES：python3 scripts/experiment-apparatus/device-code-key.py run …，`--` 之前的 --feature 值里有一段是 gpu 的，例 e174-gpu；
       入口自己不清场，服务给它套）；脚本文件头声明了要卡的参数、第一个非选项参数对得上的（declared_gpu_run；写法在 task_server_lib.py 文件头「GPU」）；
       判法是服务自己的 gpu_need 与下面两份名单；只照认法，不看「这条命令上一次用过卡」的记录）。卡照多机配置 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 的先后（没写取 GPU_CARDS_BY_PRIORITY），
       变异门禁 crates-mutation-replay.sh（mutation_gate_uses_gpu）：被判那棵树的 crates/mutations.tsv 里有一行 cargo test 参数开了上卡的特性（gpu、<包>/gpu、tests-on-a-real-gpu、--all-features）就要卡，由服务套一次清场、分一张卡；门禁里的格在服务的清场之下不再各自套
       （用户：「目前已经是清场状态 你清场什么」；任务 2170 七个工作进程各自清场，回读把同任务另一个工作进程刚起的测试二进制当外人）；带 --list-items 的不算。
       挑优先级最高、此刻没有服务起的任务占着、所在那台又放得下的一张，经环境 GPU_CARDS_BY_PRIORITY=<台标签>:<序号>:<额度>（原样那一项）交给命令；一张都没点名的记 run-condition-unmet。
       要几张（task_server_lib.py 文件头「GPU」的张数；用户：「一张卡1gb 2张卡 三张卡 每张卡都能用来分片gc」）：源文件头声明了张数的（gpu_cards_wanted，提交时记 gpu_cards），
       照卡的优先级挑头一台空卡凑得出这么多张、内存又放得下的，那台空卡里照优先级的前几张整组分给它（place_on_card）：GPU_CARDS_BY_PRIORITY 交这几项用空格接的一串，
       清场照这一串整组清，尝试记 card 这一串，几张在跑时都算占着（busy_cards、card_keys）；疑似卡住看这几张上有没有它的负载。多机配置点名的卡哪台都凑不出这么多张的
       记 never-fits（有卡停用着时只排着）；张数声明写坏的（不是一个正整数）记 run-condition-unmet、说明点名哪一份。分片的片继承父任务的张数。
       弄坏开关 gpu-card-count-ignored 下一律一张（改前的样子：任务 7227 的多卡测试只罩到一张）。
       卡的台标签是多机配置 HOSTS 里那一台的地址（例 192.0.2.1:0:32G；从右边切两刀，地址可以是 IPv6；切法与认法是 scripts/hosts/gpu-card-health.py 的 card_labels，不另写一份）：
       服务照 HOSTS 认出自己这一台（scripts/hosts/multi_host_addresses.py 的 resolve，task_server_lib.py 的 host_addresses），台标签是它的那几张是 local、是另一台的是 peer
       （local / peer 只是服务自己记尝试的视角）；台标签不在 HOSTS 里、认不出自己这一台的，那一轮当一张卡都没点名、排队原因写那一句。
       弄坏开关 card-labels-all-local 下照改前不认地址，台标签一律当服务所在这一台的卡。
     不认合入锁（用户 2026-11-02：「可以 按这个做 不要拿锁了」；验证在快照上跑）：主工作树正在合入时，会编的与派第二台的照常排、照常起。
  ③ 起：每次尝试一个脱离服务的用户级单元 singlefs-task-<库标签>-<任务号>-<第几次>.service（systemd-run --user --collect，KillMode=mixed），先把单元名写进库再起，只起一次。
     照现读的任务（不拿调用方手里的旧字典）备好目录、环境与说明之后拿这条任务的锁（Store.task_lock），claim_attempt 一个事务里判 queued 且没取消、父任务没结局、
     分尝试号、插尝试、改任务（TS-01、TS-03；库里 (task_id, number) 唯一），之后起单元，起完才放锁；判不过的不起、备好的目录删掉，取消了的记 cancelled。
     弄坏开关 cancel-race-unguarded 下照改前不拿锁、先插尝试再拿旧字典整行写回任务；attempt-claim-split 下照改前插尝试与改任务分两次提交。
     systemd-run 超时（START_TIMEOUT_SECONDS）只算「起没起不确定」（TS-06：它是发给用户级 systemd 的请求，调用端超时不说明管理器没收下）：照预先定好的单元名
     读回状态与起跑标记，起了的照常 running；判不了的尝试留在 starting、账照记、写告警 start-uncertain，交给对账（上面 ①）。弄坏开关 start-timeout-as-failure 下照改前当没起。
     单元里跑 attempt_runner.py，它照说明起这一条：
       本机：bash scripts/hosts/run-with-memory-cap.sh <上限> bash <仓根>/scripts/hosts/capped.sh <线程数> <命令…>，环境里设 RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER=<任务号>.<第几次>
             （内存包装不再排第二遍，① 每条上限与 ② slice 总上限照旧做内核层兜底）；会编的 cargo 另设这一次的 CARGO_TARGET_DIR（task_server_lib.py 文件头「编译目录」）
       第二台：bash scripts/hosts/multi-host-run.sh --peer --memory-cap <上限> --label <标签> -- env <白名单变量>=<值>… bash <仓根>/scripts/hosts/capped.sh <线程数> <命令…>，
               在任务的工作目录里起：multi-host-run.sh 在第二台同一个绝对路径里 env -C 起（仓与工作根在 NFS 共享根下，两台同一棵树；multi-host-run.sh 文件头 ③ ⑤），
               不送、不取回、不改写路径、不留第二台的树；写出的文件就在共享的树与共享根下，两台都读得到。编译目录由 multi-host-run.sh 在第二台本地盘上这一趟新建、
               跑完删（服务不设、不留，task clean 不管它）。分片那几片同样：计划目录与产物目录在共享工作根下（task_server_lib.py 文件头「分片」），交绝对路径。
               env 那一段带任务的白名单变量与下面「环境」点名的那几个；本机的路径与编译目录、按台取值的（PEER_HOST_SPECIFIC_VARIABLES）、值是本机绝对路径而不在共享根下的不带。
               这一层的环境里另设 MULTI_HOST_RUN_ADMITTED_BY_TASK_SERVER=<任务号>.<第几次>（task_server_lib.py 的 PEER_ADMITTED_VARIABLE）：multi-host-run.sh --peer
               只收服务直接起的这一次，任务里面再带 --peer（两台都一样）它拒、退 255（那份文件头 ①）；env 那一段不带它，本机的尝试不设。弄坏开关 peer-admission-unmarked 下照改前不设。
               弄坏开关 peer-tree-copied 下照改前拷树：加 --send <工作目录> <树里的落点>、命令换到落点里起（multi-host-run.sh 已不认 --send，退 2）。
       要卡的（下面 ②「GPU」认出的）一律在最外层再套 bash scripts/crash/gpu-quiesce-run.sh --：卡与清场只在这里定一次，任务里面不再套清场。
       只认服务直接起的那一层：内存包装、capped.sh、清场包装与 multi-host-run.sh --peer 各认一个准入标记（task_server_lib.py 的 ADMITTED_VARIABLE、
       THREAD_ADMITTED_VARIABLE、QUIESCE_ADMITTED_VARIABLE、PEER_ADMITTED_VARIABLE，值都是 <任务号>.<第几次>），各自读掉、不交给命令，
       于是任务里面的脚本再套这几样当场被拒（退 2 或那一份的用法错退出码），出路是删掉那一层。capped.sh 的那个两台都设（第二台的经 env 那一段带过去）；
       清场那个只在套了清场时设；内存包装那个本机设在环境里，第二台的由 multi-host-run.sh 照它拿到的那一份设给第二台那一层内存包装。
       弄坏开关 entry-quiesce-skipped 下按名字认出要卡的那几样（NAMED_GPU_REASON）照改前当成自己清场、不套清场。
     环境：服务自己的环境，加上任务带的白名单变量、TASK_SERVER_TASK、TASK_SERVER_ATTEMPT、TASK_SERVER_ATTEMPT_LABEL（这一次尝试的标签，两台都给：scope 照它对到尝试上）、TASK_SERVER_SOURCE_REPOSITORY（提交方那一份仓的根，本机绝对路径，第二台也给、
       拼两台一致的路径用）、SINGLEFS_DATA_ROOT 与 SINGLEFS_WORK_ROOT（服务这一侧的共享数据根与工作根的真路径，任务自己带了的照任务的；第二台原样带本机的值，
       脚本不经 ~ 与 $HOME 取：第二台的家目录是另一个）、TASK_SERVER_MEMORY_CAP_BYTES（这一次的内存上限，字节、十进制；
       第二台的经 multi-host-run.sh 那一段 env 带过去）、CLAUDE_CODE_SESSION_ID（提交它的会话，读得到才设）、
       分片那几步的 TASK_SERVER_SHARD_* 与 RUN_WITH_MEMORY_CAP_KEY（task_server_lib.py 文件头「分片」）、给内存包装的 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT（退出时峰值，
       task_server_lib.py 文件头「资源记录」）与 RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY（现场目录，同一份文件头「现场」；两样都只给包装那一层，包装读了就 unset、
       第二台的由 multi-host-run.sh 设给那边那一层，env 那一段不带）。systemd-run 退非 0 记 unit-start-failed。
     停服务（收到 TERM / INT）：排队线程与 NFS 复核线程停下（各至多 STOP_JOIN_SECONDS）之后，服务自己起的子进程还在的（去第二台探测余量、复核挂载的 bash、ssh、python3 这类，
       KillMode=process 下它们留在服务单元的 cgroup 里，control stop 回读不空）逐个按进程号 TERM、至多等 STOP_CHILD_GRACE_SECONDS 再 KILL，停完才退
       （stop_own_children；弄坏开关 no-detach 下起成子进程的尝试照旧只发 TERM、不算在内）。弄坏开关 probe-children-left 下照改前不等、不停。
  ④ 分片：父任务不进队，子任务（计划、各片、合并）进队；子任务收尾时（advance_parent）与每一轮对账 drive_parent 照库里父任务的阶段与子任务集合现推下一步：计划成了照 plan.json 建各片，
     片都成了（人手单独取消的那几片不算缺，合并时点名，记在父任务的 missing_shards）建合并，合并成了整件成；
     有一步没成整件照那一步的类型收尾、点名是哪一步；一片没成时在排的兄弟片当场撤、在跑的跑完收齐了才收尾（task_server_lib.py 文件头「分片」）。
     子任务记录经 children_of 读，有一行校验不过的抛 StoreRefused("tampered")：推父任务的记 record-tampered，status、clean、取消整件写告警、不认那几行（children_refused）。父任务的阶段（phase：plan、shards、merge、cancelling）与片的计划
     （shard_plan，计划读通之后写进父任务）都在库里，推法幂等：缺的子任务照阶段与计划补建（库里 (parent_id, role, shard_index) 唯一，同一步同一片只一个），
     子任务终态提交之后、建了一片之后、写了 outputs.tsv 之后、建了合并之后服务没了的，重启之后与每一轮对账照库里的样子接着推（TS-02）。
     按项估预计用时的估算（task_server_lib.py 文件头「资源记录」的预计用时）借同一套：父任务记 state=estimating、不进队，估算子任务（role=estimate）进队；
     子任务收尾时与每一轮对账照 drive_item_estimate 推：子任务还没建的补建，有了结局父任务改记 queued 进队（item_plan 记 ITEM_PLAN 那几行，没成的、没打的记 null）；
     还在估算时取消照 cancel_estimating：停估算子任务、回读，父任务记 cancelled。估算子任务不算分片的子任务：估算完的父任务照不分片的任务取消、status 的 children 不列它。
  ⑤ 暂停（control pause / resume；换 prune-server 的二进制、清判定库旧行这类要一段没有任务在跑的空档的活用）：暂停着，服务照收提交、照答 status / wait / log / cancel / health，
     排队线程不起任何新尝试（place_queued_into 开头与每一次 start_attempt 之前都判；插空、分片的子任务与估算子任务、重试重排回队的都是新尝试，一样不起），
     对账照做（在跑的照跑到结局，写回结局、重排回队、分片父任务照推，只是回到队里的不起）；续起「起单元之前没了」的那一次（resume_starting）是在跑的尝试的延续，不算新尝试，照起。
     放行一个会话（control pause --allow-session <会话号>；管 prune-server 的会话要在空档里经 task submit 交清判定库的命令，暂停着起不来，放开之后再起又会和别的会话新起的任务撞）：
     暂停着，任务记的会话（POST /tasks 时 session_of 取的，task status 里「会话 …」那一栏，tasks 表 session 列）等于放行的会话的，照常排、照常起——排队线程只排这些、
     start_attempt_unless_paused 只放这些（Scheduler.allowed_during_pause）；分片的子任务、估算子任务继承父任务的会话，重试重排回队的还是同一条任务，所以都随提交它的会话。
     别的会话（含没有会话的人手任务）照旧排着不起。放行的会话是空串就是不放行谁；人手任务的会话也是空串，空串从不匹配。已暂停着再 pause 带不同的 --allow-session 就换放行的会话
     （原因与起点不改，写一条操作日志 pause，备注「换放行会话 旧 → 新」）；不带 --allow-session 的 pause 不动放行；resume 连放行一起清掉（放行记在暂停态里，暂停态没了放行就没了）。
     暂停态记在库的 meta 表（名 pause，正文 {reason, since, allow_session} 连同签名，放行的会话在签名里；Store.pause_record），服务重启后照旧暂停、照旧放行，直到 resume；
     签名对不上的（绕过服务改过，改放行的会话也算）按暂停算、不放行谁、说明写校验不过。
     pause 与 start_attempt 共用一把锁（Scheduler.start_gate）：pause 返回时，已经开始的起尝试都已记下（running 数得到它们），之后一条新的都不起——
     「等 control check 报在跑 0 个」才不会被一条正在起的挤进来。pause / resume 各写一条操作日志（操作 pause、resume，备注原因）。
     暂停中：/health 与 control check 写「暂停中（原因：…，起于…）、在跑 N 个」，ok 不变；放行了会话的写「暂停中（原因：…，起于…）、放行会话 <号>、在跑 N 个、别的会话在跑 M 个」
     （M 是 state 为 running、会话不是放行的那个的任务数；等 M 为 0 才做空档的活）；排着的任务 status 写「在等：服务暂停中（原因：…）」（Scheduler.waiting_text，
     wait 的排队过久提醒的依据同它），放行会话排着的任务照常写它真在等什么；不判「判能起却没起成」（starved_problem），也不看进展（progress_problem）。
     弄坏开关 pause-queue-ignored（排队线程当没暂停）、pause-start-ungated（pause 与起尝试不共锁）、pause-halts-reconcile（暂停时对账也停）、pause-state-memory-only（暂停态不写库）、
     resume-leaves-cache-paused（resume 只清库不清排队线程读的那份）、pause-health-unreported（/health 不带暂停）、pause-waiting-unreported（排着的任务不写暂停的原因）、
     pause-judges-starved（暂停中照判放得下没起）、pause-allow-ignored（放行的会话也不起）、pause-allow-matches-any-session（放行了谁都起）、
     pause-allow-waiting-paused（放行会话排着的任务也写暂停）、pause-allow-change-ignored（已暂停再带不同的放行会话不换）、resume-keeps-allowed-session（resume 之后下一次 pause 沿用旧放行）、
     pause-allow-memory-only（放行的会话不写库）、pause-allow-unsigned（放行的会话不进签名）、pause-allow-unreported（/health 与 check 不写放行的会话）、
     pause-other-running-miscounted（别的会话在跑数把放行会话的也数进去）各关一种。
弄坏开关读在 task_server_lib.py 的 broken（开关表在那份文件头）。
"""
import os
import sys

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
sys.path.insert(0, SCRIPT_DIRECTORY)
import task_server_lib as lib  # noqa: E402

import asyncio  # noqa: E402
import contextlib  # noqa: E402
import dataclasses  # noqa: E402
import fnmatch  # noqa: E402
import hmac  # noqa: E402
import itertools  # noqa: E402
import json  # noqa: E402
import math  # noqa: E402
import re  # noqa: E402
import resource  # noqa: E402
import secrets  # noqa: E402
import shlex  # noqa: E402
import shutil  # noqa: E402
import signal  # noqa: E402
import socket  # noqa: E402
import sqlite3  # noqa: E402
import struct  # noqa: E402
import subprocess  # noqa: E402
import threading  # noqa: E402
import time  # noqa: E402
import traceback  # noqa: E402
import types  # noqa: E402
from dataclasses import dataclass, field  # noqa: E402

TICK_SECONDS = 30.0             # 用户（records/2026-10-30-编译绕过内存总上限.md）：「每秒循环太浪费资源了 我30s一次就好了」；提交、取消、尝试结束（runner 调 /wake）都当场叫醒，这个间隔只兜底
PEER_STARTS_PER_PASS = 4         # 一轮至多在第二台起这么多条：一齐起几十条时第二台的 sshd 会重置新连接（实测 100 条一齐交时两条 ssh 被重置）
PEER_REPASS_SECONDS = 2.0        # 第二台这一轮起满了：隔这么久再排一轮，不等满 TICK_SECONDS
UNIT_CHECK_SECONDS = 5.0
LOST_GRACE_SECONDS = 10.0
PEER_RECHECK_SECONDS = 60.0
RESOURCE_CACHE_SECONDS = 2.0
START_TIMEOUT_SECONDS = 60
LEFTOVER_STOP_SECONDS = 10.0     # 尝试正常结束之后停掉单元与 scope 里留下的进程，至多等这么久回读 cgroup 空了没有（stop_leftovers；SIGKILL 当场生效，推的）
UNIT_STOP_SECONDS = 150          # 单元被停时给命令收尾的时限：multi-host-run.sh 停第二台那条命令至多等 30 秒再删第二台这一趟的本地目录，内存包装停自己的 scope，留足（推的）
CANCEL_SETTLE_MARGIN_SECONDS = 20.0   # 取消停了单元之后等对账写回结局：没留结果的过 lost_grace_seconds 才记，再加两次读单元的间隔与这一截（推的；cancel_outcome_reply）
CANCEL_SETTLE_POLL_SECONDS = 0.1      # 等结局写回时隔这么久现读一次任务、叫醒排队线程
STARVED_ROUNDS = 3               # 派活判排头能起而连着这么多轮（秒数按一轮的长短乘）没起成，/health 回不健康；判不能起的、队空不动不算
PROGRESS_STALL_SECONDS = 600.0   # 看进展（Scheduler.progress_problem）：一台照公式放得下排着的最小那条、却这么久一条都没起，/health 回不健康。取 10 分钟的依据：与 STALL_SECONDS
                                 # 同一个数（用户定一条任务日志满 10 分钟不涨才算没进展，服务自己没进展用同一把尺）；派活故意让排头等的几种都短得多（第二台一轮起满等 PEER_REPASS_SECONDS
                                 # 2 秒、CPU 闸记刚起的 CPU_SETTLE_SECONDS 5 秒），等卡、等负载、等占盘与封着组等腾空这道另外不算；队列卡死 4 个多小时那一次 10 分钟就报得出来
STALL_SECONDS = 600.0            # 用户定：一条任务的日志距上一次增长满 10 分钟、这段时间 CPU 也几乎没涨，服务停掉它、记 stalled
PEER_CPU_LINE = re.compile(r"^MULTI_HOST_RUN_PEER_CPU poll=([0-9]+) usage_usec=([0-9]+) read=(yes|no)$")   # multi-host-run.sh 文件头 ⑤ 每轮一行
OVERRUN_STOP_FACTOR = 5.0        # 用户：跑得超过记录用时这么多倍、日志又满 STALL_SECONDS 不涨，就不看 CPU、停掉打回（工具里不限时，死循环靠这一条停）
QUEUE_ATTENTION_MINUTES = 30     # 排队过久头一档（分钟；文件头 ①「要处置的提醒」）：排满这么久 wait 叫醒一次，之后每翻一倍（60、120 分钟…）再叫一次。取 30 的理由（推的）：常见排队几分钟到十几分钟，
                                 # 满 30 分钟多半是第二台用不了、余量被大任务占着或预测过大；比 AGED_WAIT_SECONDS（两小时升高优先）早，叫醒时人还来得及改命令重交
OVERRUN_ATTENTION_FACTOR = 3.0   # 超过记录用时头一档（倍；同一节）：跑到记录用时的这么多倍 wait 叫醒一次，之后每翻一倍（6、12、24 倍…）再叫一次。取 3 的理由（推的）：低于 OVERRUN_STOP_FACTOR（5 倍且日志满
                                 # STALL_SECONDS 没涨才停），日志在涨时服务永远不停它，要靠这一档先叫人；status 的「已超过记录用时的两倍」只提示、不叫醒。不足 STALL_GRACE_SECONDS 的不叫（短任务抖动）
STALL_GRACE_SECONDS = 600.0      # 用户：「至少要延迟十分钟后在判定吧」「如果准备时间或者编译时间长 这不是很容易被杀掉」：尝试起来后先宽限这么久不判，之后才量窗口
CARD_QUARANTINE_SECONDS = 3600   # 清场判出坏的卡停用这么久（Scheduler.quarantined_cards），到点再进候选；还坏清场会再判出来、再停用，任务照样换卡
CPU_OVERSUBSCRIPTION = 3         # CPU 闸：一台此刻的负载（1 分钟负载与可跑进程数的大者）加刚起的与这一条的线程，不超过核数的这个倍数（cpu_problem）
CPU_SETTLE_SECONDS = 5           # CPU 闸：起了不到这么久的尝试，负载还没涨上来，按它的线程数整份记上（编译几秒内就把可跑进程数顶上去；记久了接连起的短任务会把整台挡死）
STALL_CPU_CORES = 0.01           # 「CPU 几乎没涨」：这一个窗口里平均不到一个核的 1%（用户选「日志与 CPU 都不涨才停」）
STALL_AT_CAP_FRACTION = 0.98     # 判卡住那一刻 scope 的内存峰值到了上限的这么多：顶在上限上卡在回收里，按撞上限办（任务 1866 第 4 次：上限 13824M、峰值 13.50 GiB）
AGED_WAIT_SECONDS = 7200.0       # 提交满这么久还在排的升成高优先：排到最前、先判（用户在 records/2026-10-30-编译绕过内存总上限.md 里定）；放不下时不挡后面、不留份（文件头 ②「满时限」）
WHOLE_FRACTION = 0.85            # 整片的界：记账不小于那台可用上限（总上限减那台此刻的余量，host_capacity）的这么多（用户：「超过设定量85% 的算」「拼装碎片任务的时候也按照 85%一组算」）
WAIT_LIMIT_SECONDS = 60.0
BREAK_FIXED_MARGIN_BYTES = 2 * 1024 ** 3   # 只给弄坏开关 margin-fallback-fixed：哪台量不到余量时照改前退回的固定数（文件头 ②「余量」）
RECORD_WINDOW = 5
LOG_TAIL_BYTES = 65536
LOG_SCAN_BYTES = 4 * 1024 * 1024   # log_content 接着读日志时一次至多读这么多
FAST_TICK_SECONDS = 0.1          # 自证的定时（TASK_SERVER_SELFTEST_FAST=1）
FAST_UNIT_CHECK_SECONDS = 0.3
FAST_LOST_GRACE_SECONDS = 1.0
FAST_STARVED_SECONDS = 1.0
FAST_PROGRESS_STALL_SECONDS = 3.0   # 自证里看进展的时限：比 FAST_STARVED_SECONDS 长，短于各格盯 /health 的 4 秒
NFS_RECHECK_SECONDS = 60.0       # 服务起来之后隔这么久复核一次 NFS 共享的前三项（task_server_lib.py 文件头「NFS 共享」；与 PEER_RECHECK_SECONDS 重读第二台同一个间隔）
FAST_NFS_RECHECK_SECONDS = 0.5
ACTIVE_ATTEMPT_STATES = ("starting", "running")
PAUSE_META_NAME = "pause"        # 暂停态在库 meta 表里的名字（文件头 ⑤「暂停」）
PAUSE_REASON_CHARACTERS = 200    # 暂停原因收的字数（一句话；多的截掉）
CARD_ENTRY = re.compile(r"^(?P<address>.+):(?P<index>[0-9]{1,9}):(?P<quota>[0-9]{1,15}[GM])$")   # 尝试记下的那张卡（原样那一项）：从右边切两刀，同 gpu-card-health.py 的 CARD_ENTRY
# GPU 的认法：cargo 带 gpu 特性而跑编出来的代码、经 gpu-quiesce-run.sh 起、名单上的入口脚本（判法是 cargo_uses_gpu、script_uses_gpu 与 recognize_gpu）
GPU_RUNNING_CARGO_SUBCOMMANDS = {"test", "t", "run", "r", "bench"}   # 跑编出来的代码的 cargo 子命令与 cargo 内建的别名；别的子命令只编不跑
NO_RUN_ARGUMENT = "--no-run"   # cargo test / bench 只编不跑
GPU_SCRIPT_NAMES = {"gpu-quiesce-run.sh"}
NAMED_GPU_REASON = "按名字认的 GPU 跑法"   # 按名字认出要卡的那几样，gpu_reason 以它起头（弄坏开关 entry-quiesce-skipped 只对它们不套清场）
# 脚本里面才用卡的入口（records/2026-10-30-编译绕过内存总上限.md #13：灰度会话交的 crash-point-replay.sh --item <项>、
# merge-and-verify.sh 回显 gpu=0）：按名字认，服务给它套清场；带 --list-items、--help、-h、单独 --selftest 的不算
GPU_ENTRY_SCRIPT_NAMES = {"crash-point-replay.sh", "merge-and-verify.sh", "gate-staged.sh", "gate.sh"}   # crash-point-replay.sh 是入口与门禁两份的文件名，两份都要卡
GPU_NOT_RUN_ARGUMENTS = ("--list-items", "--help", "-h")
# 变异门禁：被判那棵树的变异表里有上卡的行就要卡，整条由服务套一次清场、分一张卡（mutation_gate_uses_gpu；用户：「目前已经是清场状态 你清场什么」）
MUTATION_GATE_SCRIPT_NAME = "crates-mutation-replay.sh"
MUTATION_TABLE_IN_TREE = os.path.join("crates", "mutations.tsv")
GPU_FEATURE_NAMES = ("gpu", "tests-on-a-real-gpu")   # 开了就上卡的 cargo 特性（与 scripts/mutation/crates-mutation-rows.py 的 GPU_TEST_FEATURES 同一组）
# 判定库读写接完 prune-server 的族：登记表第三列（判定库放哪）写「prune-server」的行（registry_families）照收；别的带判定库的提交时拒（文件头 ②「提交时拒」③）
VERDICT_LIBRARY_REGISTRY = os.path.join(".claude", "gate.d", "verdict-library-scripts.tsv")
REGISTRY_SERVICE_MARK = "prune-server"   # 第三列以它起头的行：那一族的判定库读写全在服务里
REGISTRY_NO_LIBRARY_PREFIXES = ("不算：", "无判定库：")   # 第三列以这两种起头的没有判定库（登记表文件头）
# 只派一台的名单全删了（用户：「我再重申一次 【只能在本机跑】 是不能接受的。必须尽快解决 让错误暴露错来」）：任何命令都派得到两台，哪台由余量定，
# 派到第二台跑红的由脚本那一侧修。下面这几张是删之前的样子，只给弄坏开关 host-lists-restored 照改前放回用（文件头 ②「机器」）
RESTORED_HOST_LISTS = {
    "local_directories": ("scripts/hosts", ".claude/gate.d"),   # 点到这几个目录的只在本机
    "local_entries": (".claude/gate.d/doc-registries.sh", "scripts/crash/crash-point-replay-shard-configuration-check.sh", "prune-server/control",
                      "scripts/experiment-apparatus/e180-reading-shards.sh", "scripts/experiment-apparatus/e183-sweep-history-shards.sh"),   # 读本机才有的东西的入口
    "peer_entries": ("scripts/experiment-apparatus/e185-r2-segment-run.sh",),   # 只在第二台跑的入口
}
VOLATILE_ROOT = "/tmp"   # 重启会清掉的目录：提交时拒的共享根外路径落在它下面的，拒绝说明另说一句重启会清（文件头 ②「提交时拒」①）
# bash -c 这类整条写在一个词里的：词里的绝对路径（find_outside_shared_path）；前面紧挨着 \w、/（$VAR/…、a/b 的后半）、}（${X:?}/…）、)（$(cmd)/…）的是变量展开、命令替换
# 之后的尾巴，真路径运行时才由展开给，不是命令写死的，不算；前面是空白、引号、=、: 一类的照判
QUOTED_WORD_PATH = re.compile(r"(?<![\w/}).])/[^\s'\"`;|&()<>]+")
QUOTED_WORD_PATH_BEFORE_EXPANSION_TAILS = re.compile(r"(?<![\w/])/[^\s'\"`;|&()<>]+")   # 弄坏开关 expansion-tail-as-path 下用的（改前的样子）
# 两台相同的系统路径（find_outside_shared_path 不判它们在不在共享根下；与 scripts/hosts/multi-host-run.sh 的 MULTI_HOST_HOST_INDEPENDENT_PATHS 同一组）：以 / 收尾的是整个目录
HOST_INDEPENDENT_PATHS = ("/dev/null", "/dev/stdin", "/dev/stdout", "/dev/stderr", "/dev/fd/", "/usr/", "/bin/")
# 走 python 入口、按参数认要不要卡的（入口文件名 → (子命令, 特性参数)）：子命令对得上、`--` 之前的特性参数里有一个带 gpu 这个词的就要卡（文件头 ②「GPU」）
PYTHON_GPU_ENTRIES = {"device-code-key.py": ("run", "--feature")}
PYTHON_INTERPRETERS = ("python3", "python")
# 疑似卡住（文件头 ①「疑似卡住」）：只通知、不停；wait 打 lib.SUSPECT_MARK 那一行、退 lib.SUSPECT_EXIT
NVIDIA_SMI_SECONDS = 30
# 多机配置 SLICE_MEMORY_CAP_BY_HOST 里一台那一项的值（写法同 scripts/hosts/run-with-memory-cap.sh 读它的那一句：正整数加 K / M / G / T）与单位
SLICE_CAP_VALUE = re.compile(r"^([1-9][0-9]*)([KMGT])$")
SIZE_UNITS = {"K": 1024, "M": 1024 ** 2, "G": 1024 ** 3, "T": 1024 ** 4}
# 改主工作区的工具（modifies_workspace：跑第二遍结果就变，撞上限这类本会重排的结局一律打回、不重排；派哪台不看它，第二台改的就是同一棵共享的树）
HOOK_SWAP_ENTRY = "scripts/gate/hook-swap.py"   # 带 --feed-only 的只喂不换（modifies_workspace 不算它改工作区）
WRITER_PATCH_ENTRY = "scripts/gate/apply-writer-patch.py"   # 带 --dry-run 的只在临时树上核（modifies_workspace 不算它改工作区）
WORKSPACE_READ_ONLY_ARGUMENTS = ("--selftest", "--list-items", "--dry-run", "--read-outcome", "--pending-status")   # 改工作区的工具带了这几样之一只读不写（自证、列项、干跑、读结局），modifies_workspace 不算（任务 7543、7549）
WORKSPACE_WRITERS = {"scripts/gate/hook-swap.py", "scripts/gate/apply-writer-patch.py", "scripts/gate/stage-mine.py"}
PATCH_APPLYING_ENTRY = "scripts/gate/merge-and-verify.sh"   # 不带 --skip-apply 的往工作区打补丁
WORKSPACE_WRITER_NO_RETRY_OUTCOMES = ("memory-cap-hit", "squeezed-out", "gpu-memory-short", "peer-run-failed")   # 改主工作区的命令遇到这几种本会重排的结局一律打回、不重排
# 改主工作区的命令以这几种结局收尾（连同被信号杀的 command-failed）：可能写了一半，wait 报 workspace-write-interrupted（文件头 ①「要处置的提醒」）
WORKSPACE_WRITER_INTERRUPTED_OUTCOMES = WORKSPACE_WRITER_NO_RETRY_OUTCOMES + ("memory-cap-at-maximum", "cancelled", "stalled", "launcher-lost")
SIGNAL_EXIT_CODES = range(129, 129 + 64)   # 128 + 信号号：attempt_runner 记被信号杀的命令就是这个码（attempt_runner.py 文件头 ⑥），bash 起的命令被信号杀同样退它
PATCH_WRITER_OUTCOME_NAME = "apply-outcome.tsv"   # WRITER_PATCH_ENTRY 的结局文件：补丁目录里这一份，--read-outcome 判打全了没有
PATCH_WRITER_READ_ONLY_ARGUMENTS = ("--dry-run", "--pending-status", "--read-outcome", "--selftest")   # 带这几样的不写主工作区
PATCH_WRITER_CHECK_SECONDS = 120   # 起一次 --read-outcome 至多等这么久（它只读结局文件、对着主工作区算几份文件的 sha256）
PATCH_WRITER_VERDICTS = {0: "打全了", 1: "已退回", 2: "结局文件读不了", 3: "没打完", 4: "打完之后文件又被改了"}   # --read-outcome 的退出码（apply-writer-patch.py 文件头「结局文件」）
PEER_HOST_SPECIFIC_VARIABLES = ("ROCKSDB_LIB_DIR", "ROCKSDB_STATIC", "SNAPPY_LIB_DIR", "SNAPPY_STATIC")   # 按台取值（灰度会话报 359：第二台的预编库核对红）
GPU_SCRIPT_PATTERNS = ("crash-amplification*.sh",)
MAXIMUM_LAUNCHER_DEPTH = 4
# 盘余量（task_server_lib.py 文件头「挑机器」的「盘余量」）：一台编译目录所在盘不到这么多就这一轮不派那一台，两台同一个下限。一趟的编译目录量过 200G 级
#（records/2026-10-31-task-server巡检.md 问题 72：任务 2665 的合入后验证在第二台留了一份 213G 的编译目录），余量取不到一次合入后验证的四分之一，
# 那一趟编译会在中途写满那台的盘（本机是 TASK_SERVER_BUILD_DIRECTORY 所在盘，第二台是多机配置 BUILD_ROOT_BY_HOST 里它那一项所在盘，multi-host-run.sh 文件头 ⑤）
DISK_FLOOR_BYTES = 50 * 1024 ** 3
DISK_REPORT_KEY = "disk_available"   # 每台余量报告里它编译目录所在盘的余量（字节；Machine.probe 带上，报告里没有时 Machine.disk_free 现量）
PEER_DISK_PROBE_TEXT = "df -B1 --output=avail -- {directory}"   # 在第二台的编译根（BUILD_ROOT_BY_HOST 里它那一项）上起：末行是余量的字节数
PEER_BUILD_REMOVE_SECONDS = 120   # multi-host-run.sh --remove-build-directory（第二台留着的编译目录）至多等这么久：核持有者、rm、回读各一次 ssh（推的，同 --recover-cleanup 的量级）
DISK_FLOOR_KIND = "disk-floor"   # note_peer 去重的原因种类：盘不够这一种只告警一次，余量的数变了不另写
# 编译目录占盘（task_server_lib.py 文件头「资源记录」的「编译目录占盘」；records/2026-10-31-task-server巡检.md 问题 76）：派到一台之前那台的盘余量要放得下
# 这条命令量过的编译目录峰值加 BUILD_DISK_FLOOR_BYTES（盘余量照上面的 DISK_REPORT_KEY）
BUILD_DISK_FLOOR_BYTES = DISK_FLOOR_BYTES
# 启动回执超时（TS-06，records/2026-11-01-task-server深入审核.md）：systemd-run 超时只算「起没起不确定」，照预先定好的单元名读回；
# 读回单元不在、起跑标记也没有，过了这么久才确认没起、记 unit-start-failed
START_UNCERTAIN_GRACE_SECONDS = 120.0
FAST_START_UNCERTAIN_GRACE_SECONDS = 2.0
SELFTEST_PAUSE_SECONDS = 60.0   # 自证的停顿点至多等这么久放行（TASK_SERVER_SELFTEST_PAUSE_POINT）
PRUNE_SERVER_CHECK_SECONDS = 60   # 命令退 75（prune-server 不可用）时起一次 prune-server/control check 的时限
SCENE_KEEP_AFTER_COLLECT_SECONDS = 3600.0   # 现场在 task wait 取走结局之后再留这么久（task_server_lib.py 文件头「现场」：提交方读日志、照日志去读现场，一小时够，推的）
FAST_SCENE_KEEP_AFTER_COLLECT_SECONDS = 1.0
STOP_JOIN_SECONDS = 10.0          # 停服务时等排队线程与 NFS 复核线程停下的时限（两个合计；单元的 TimeoutStopSec 是 30 秒，文件头 ③「停服务」）
STOP_CHILD_GRACE_SECONDS = 3.0    # 停服务时给自己起的子进程发 TERM 之后至多等这么久再 KILL


def say(text):
    print(text, file=sys.stderr, flush=True)


def append_private_line(path, line):
    lib.ensure_private_directory(os.path.dirname(path))
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
    with os.fdopen(descriptor, "a", encoding="utf-8") as handle:
        handle.write(line.rstrip("\n") + "\n")


def nofile_property():
    """尝试单元的打开文件数上限：软、硬都取服务自己的硬上限（用户级 systemd 给单元的软上限是 1024，崩溃判定库开不了；
    records/2026-10-30-编译绕过内存总上限.md #16）。"""
    hard = resource.getrlimit(resource.RLIMIT_NOFILE)[1]
    value = "infinity" if hard == resource.RLIM_INFINITY else str(hard)
    return f"LimitNOFILE={value}:{value}"


def directory_bytes(path):
    """一个目录底下文件的字节数之和（task clean 报大小用；读不到的跳过）。"""
    total = 0
    for root, _, names in os.walk(path):
        for name in names:
            with contextlib.suppress(OSError):
                total += os.lstat(os.path.join(root, name)).st_size
    return total


def read_json_file(path):
    try:
        with open(path, encoding="utf-8") as handle:
            return json.load(handle)
    except (OSError, ValueError):
        return None


def write_json_atomically(path, value):
    temporary = f"{path}.{os.getpid()}.partial"
    descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    with os.fdopen(descriptor, "w", encoding="utf-8") as handle:
        json.dump(value, handle, ensure_ascii=False)
    os.replace(temporary, path)


def log_tail(path):
    try:
        with open(path, "rb") as handle:
            handle.seek(0, os.SEEK_END)
            handle.seek(max(0, handle.tell() - LOG_TAIL_BYTES))
            return handle.read().decode("utf-8", "replace")
    except OSError:
        return ""


# ── 库 ──
class StoreRefused(Exception):
    """Store.change / claim_attempt 判出这一次不能写（行校验不过、状态不对、取消了）：str 是为什么，kind 是哪一种。"""

    def __init__(self, kind, why):
        super().__init__(why)
        self.kind = kind


class Store:
    """库（task_server_lib.py 文件头「库」）：每个线程一条连接，写都在 write_lock 里、BEGIN IMMEDIATE。
    任务的状态转换（task_server_lib.py 文件头「库」的「状态转换」）：change 在一个事务里读当前那一行、核校验、判 guard、只并进给的几栏再写，
    不拿调用方手里的旧字典整行覆盖；结了局的任务不回到别的状态。claim_attempt 把「判 queued 且没取消、分尝试号、插尝试、改任务」放进同一个事务。
    task_lock 是每条任务一把锁：起单元与取消都拿它，取消要么在起之前（没起），要么等起完再停、回读。
    锁表 task_locks 只留此刻有人拿着或在等的那几把（{任务号: [锁, 拿着与在等的个数]}）：个数回到 0 就删，任务有了结局、没人再拿它的锁之后表里没有它；
    表的大小不随跑过的任务数涨。弄坏开关 task-locks-kept 下照改前一把都不删。"""
    SIGNED_TABLES = ("tasks", "attempts", "history", "item_rates")
    LOG_TABLES = ("operations", "alerts")
    # (task_id, number) 与 (parent_id, role, shard_index) 两条业务唯一（TS-02、TS-03）：照正文里的那几栏建索引；task_id 是 0 的尝试（自证记进库的那几次）不归任务、不判
    UNIQUE_INDEXES = (
        "CREATE UNIQUE INDEX IF NOT EXISTS attempts_task_number ON attempts(task_id, json_extract(record, '$.number')) WHERE task_id > 0",
        "CREATE UNIQUE INDEX IF NOT EXISTS tasks_child_identity ON tasks(json_extract(record, '$.parent_id'), json_extract(record, '$.role'), "
        "json_extract(record, '$.shard_index')) WHERE json_extract(record, '$.parent_id') > 0",
        "CREATE UNIQUE INDEX IF NOT EXISTS item_rates_key ON item_rates(command_key) WHERE command_key != ''",   # 每（入口脚本、项名）一行（task_server_lib.py 文件头「库」）
    )

    def __init__(self, path, key, environment):
        self.path = path
        self.key = key
        self.environment = environment
        self.local = threading.local()
        self.write_lock = threading.Lock()
        self.task_locks = {}
        self.task_locks_guard = threading.Lock()
        self.tag = ""
        with self.write_lock:
            self.initialize()

    def connection(self):
        connection = getattr(self.local, "connection", None)
        if connection is None:
            connection = sqlite3.connect(self.path, timeout=30, isolation_level=None, check_same_thread=True)
            connection.execute("PRAGMA journal_mode=WAL")
            connection.execute("PRAGMA busy_timeout=30000")
            self.local.connection = connection
        return connection

    def initialize(self):
        connection = self.connection()
        connection.execute("BEGIN IMMEDIATE")
        connection.execute("CREATE TABLE IF NOT EXISTS meta(name TEXT PRIMARY KEY, value TEXT NOT NULL)")
        for table in self.SIGNED_TABLES:
            connection.execute(f"CREATE TABLE IF NOT EXISTS {table}(id INTEGER PRIMARY KEY AUTOINCREMENT, task_id INTEGER NOT NULL DEFAULT 0, state TEXT NOT NULL, "
                               "session TEXT NOT NULL, command_key TEXT NOT NULL, record TEXT NOT NULL, signature TEXT NOT NULL)")
            connection.execute(f"CREATE INDEX IF NOT EXISTS {table}_state ON {table}(state)")
        for table in self.LOG_TABLES:
            connection.execute(f"CREATE TABLE IF NOT EXISTS {table}(id INTEGER PRIMARY KEY AUTOINCREMENT, record TEXT NOT NULL, signature TEXT NOT NULL)")
        for statement in self.UNIQUE_INDEXES:
            try:
                connection.execute(statement)
            except sqlite3.IntegrityError as error:
                connection.execute("ROLLBACK")
                raise lib.Unavailable(f"库 {self.path} 里已有违反业务唯一的行（同一任务同一次号的两次尝试，或同一父任务同一步同一片的两个子任务）：{error}",
                                      "停着服务时 python3 task-server/control clear-queue（先备份库）清掉任务与尝试，再起；这一版不认这种库") from None
        row = connection.execute("SELECT value FROM meta WHERE name = 'tag'").fetchone()
        if row is None:
            self.tag = secrets.token_hex(4)
            connection.execute("INSERT INTO meta(name, value) VALUES('tag', ?)", (self.tag,))
        else:
            self.tag = row[0]
        connection.execute("INSERT OR REPLACE INTO meta(name, value) VALUES('health', '0')")
        connection.execute("COMMIT")
        for suffix in ("", "-wal", "-shm"):
            with contextlib.suppress(FileNotFoundError):
                os.chmod(self.path + suffix, 0o600)

    def signature(self, table, row_id, state, session, command_key, record_text):
        return lib.row_signature(self.key, table, row_id, state, session, command_key, record_text)

    def verified(self, table, row):
        """row 是 (id, state, session, command_key, record, signature, task_id)：校验列对得上、正文与几列对得上才交 True。
        决定关联的那一列 task_id（attempts_of、children_of 照它挑）不在签名里，读的时候与正文的 task_id 比，对不上就是改过（TS-05）；
        子任务的 parent_id 另要与 task_id 相同。弄坏开关 association-column-unchecked 下不比（改前的样子）。"""
        if lib.broken("signature-unchecked", self.environment):
            return True
        row_id, state, session, command_key, record_text, given, column_task_id = row
        if not hmac.compare_digest(given or "", self.signature(table, row_id, state, session, command_key, record_text)):
            return False
        try:
            record = json.loads(record_text)
        except ValueError:
            return False
        if not (isinstance(record, dict) and record.get("id") == row_id and record.get("state", "") == state and record.get("session", "") == session
                and record.get("command_key", "") == command_key):
            return False
        if table in self.SIGNED_TABLES and not lib.broken("association-column-unchecked", self.environment):
            if (record.get("task_id") or 0) != (column_task_id or 0):
                return False
            if table == "tasks" and record.get("parent_id") and record.get("parent_id") != record.get("task_id"):
                return False
        return True

    def decoded(self, table, row):
        try:
            record = json.loads(row[4])
        except ValueError:
            record = {}
        if not isinstance(record, dict):
            record = {}
        record.setdefault("id", row[0])
        return record, self.verified(table, row)

    def insert(self, table, record):
        with self.write_lock:
            connection = self.connection()
            connection.execute("BEGIN IMMEDIATE")
            try:
                if table in self.SIGNED_TABLES:
                    cursor = connection.execute(f"INSERT INTO {table}(task_id, state, session, command_key, record, signature) VALUES(?, '', '', '', '{{}}', '')",
                                                (record.get("task_id", 0),))
                else:
                    cursor = connection.execute(f"INSERT INTO {table}(record, signature) VALUES('{{}}', '')")
                record["id"] = cursor.lastrowid
                self.write_row(connection, table, record)
                connection.execute("COMMIT")
            except BaseException:
                connection.execute("ROLLBACK")
                raise
        return record

    def update(self, table, record, force=False):
        """整行写（尝试、钉值这类只有排队线程写的行）。任务行改用 change；这里另守一条：库里已结了局的任务不让写回别的状态
        （force 只给弄坏开关 cancel-race-unguarded 照改前整行覆盖的那一支）。"""
        with self.write_lock:
            connection = self.connection()
            connection.execute("BEGIN IMMEDIATE")
            try:
                if table == "tasks" and record.get("state") != "finished" and not force:
                    current = connection.execute("SELECT state FROM tasks WHERE id = ?", (record["id"],)).fetchone()
                    if current is not None and current[0] == "finished":
                        raise StoreRefused("finished", f"任务 {record['id']} 已结了局，不写回 {record.get('state')}")
                self.write_row(connection, table, record)
                connection.execute("COMMIT")
            except BaseException:
                connection.execute("ROLLBACK")
                raise

    def read_row(self, connection, table, row_id):
        """事务里读一行：→ (正文, 校验过没有)；不在交 (None, False)。"""
        row = connection.execute(f"SELECT id, state, session, command_key, record, signature, task_id FROM {table} WHERE id = ?", (row_id,)).fetchone()
        return self.decoded(table, row) if row is not None else (None, False)

    def change(self, table, row_id, changes, guard=None, reopen=False):
        """在一个事务里读当前那一行、核校验、判 guard（交空串放行，交一句就不写、抛 StoreRefused("guard", 那一句)）、只并进 changes 那几栏、写回；→ 写好的正文。
        行不在抛 StoreRefused("missing")，校验不过抛 StoreRefused("tampered")；结了局的任务要写回别的状态（reopen 没给）抛 StoreRefused("finished")。
        changes 可以是函数（拿当前正文、交要并进的几栏）。"""
        with self.write_lock:
            connection = self.connection()
            connection.execute("BEGIN IMMEDIATE")
            try:
                record, valid = self.read_row(connection, table, row_id)
                if record is None:
                    raise StoreRefused("missing", f"{table} 里没有 {row_id}")
                if not valid:
                    raise StoreRefused("tampered", f"{table} 的 {row_id} 校验不过")
                why = guard(record) if guard is not None else ""
                if why:
                    raise StoreRefused("guard", why)
                fields = changes(record) if callable(changes) else changes
                was_finished = record.get("state") == "finished"
                record.update(fields)
                if table == "tasks" and was_finished and record.get("state") != "finished" and not reopen:
                    raise StoreRefused("finished", f"任务 {row_id} 已结了局，不回到 {record.get('state')}")
                self.write_row(connection, table, record)
                connection.execute("COMMIT")
            except BaseException:
                connection.execute("ROLLBACK")
                raise
        return record

    def claim_attempt(self, task_id, number, attempt, task_changes, between=None, split=False):
        """起一次尝试的那一个事务（TS-01、TS-03）：读任务、判 queued 且没取消、号是 attempt_count + 1、父任务（有的话）还没结局，插尝试、改任务，一起提交。
        判不过抛 StoreRefused（kind 是 missing、tampered、cancelled、state、number、parent-finished）；between 在插了尝试、改任务之前调（自证的崩溃点）。
        split（弄坏开关 attempt-claim-split）照改前分两次提交：先插尝试、提交，再改任务。→ (尝试正文, 改过的任务正文)。"""
        with self.write_lock:
            connection = self.connection()
            connection.execute("BEGIN IMMEDIATE")
            try:
                task, valid = self.read_row(connection, "tasks", task_id)
                if task is None:
                    raise StoreRefused("missing", f"没有任务 {task_id}")
                if not valid:
                    raise StoreRefused("tampered", f"任务 {task_id} 的记录校验不过")
                if task.get("cancel_requested"):
                    raise StoreRefused("cancelled", f"任务 {task_id} 判放得下之后、起之前被取消了")
                if task.get("state") != "queued":
                    raise StoreRefused("state", f"任务 {task_id} 此刻是 {task.get('state')}，不是 queued")
                if int(task.get("attempt_count") or 0) + 1 != number:
                    raise StoreRefused("number", f"任务 {task_id} 已起过 {task.get('attempt_count')} 次，这一次的号 {number} 对不上")
                parent_id = task.get("parent_id")
                if parent_id:
                    parent, parent_valid = self.read_row(connection, "tasks", parent_id)
                    if parent is None or not parent_valid or parent.get("state") == "finished":
                        raise StoreRefused("parent-finished", f"子任务 {task_id} 的父任务 {parent_id} 已结了局（或记录校验不过），不起")
                cursor = connection.execute("INSERT INTO attempts(task_id, state, session, command_key, record, signature) VALUES(?, '', '', '', '{}', '')", (task_id,))
                attempt = dict(attempt, id=cursor.lastrowid, task_id=task_id, number=number)
                self.write_row(connection, "attempts", attempt)
                if split:
                    connection.execute("COMMIT")
                    connection.execute("BEGIN IMMEDIATE")
                if between is not None:
                    between()
                task.update(task_changes, attempt_count=number)
                self.write_row(connection, "tasks", task)
                connection.execute("COMMIT")
            except BaseException:
                with contextlib.suppress(sqlite3.Error):
                    connection.execute("ROLLBACK")
                raise
        return attempt, task

    @contextlib.contextmanager
    def task_lock(self, task_id):
        """拿着这条任务的那把锁跑 with 里那一段（起单元、取消、恢复时续起都拿它）。拿着与在等的都算一个；最后一个放下时把它从 task_locks 删掉
        （在等的那几个手里是同一把，删之前没有人能另建一把顶替它）。"""
        with self.task_locks_guard:
            entry = self.task_locks.setdefault(task_id, [threading.Lock(), 0])
            entry[1] += 1
        try:
            with entry[0]:
                yield
        finally:
            with self.task_locks_guard:
                entry[1] -= 1
                if entry[1] == 0 and not lib.broken("task-locks-kept", self.environment):
                    del self.task_locks[task_id]

    def write_row(self, connection, table, record):
        text = lib.canonical_json(record)
        if table in self.SIGNED_TABLES:
            state, session, command_key = record.get("state", ""), record.get("session", ""), record.get("command_key", "")
            connection.execute(f"UPDATE {table} SET state = ?, session = ?, command_key = ?, record = ?, signature = ? WHERE id = ?",
                               (state, session, command_key, text, self.signature(table, record["id"], state, session, command_key, text), record["id"]))
        else:
            connection.execute(f"UPDATE {table} SET record = ?, signature = ? WHERE id = ?",
                               (text, self.signature(table, record["id"], "", "", "", text), record["id"]))

    def delete_row(self, table, row_id):
        with self.write_lock:
            connection = self.connection()
            connection.execute("BEGIN IMMEDIATE")
            connection.execute(f"DELETE FROM {table} WHERE id = ?", (row_id,))
            connection.execute("COMMIT")

    def count(self, table, state):
        """这张表里这个 state 的行数（/health 用：只数、不解、不核校验列）。"""
        return self.connection().execute(f"SELECT COUNT(*) FROM {table} WHERE state = ?", (state,)).fetchone()[0]

    def count_outside_session(self, table, state, session):
        """这张表里这个 state、会话列不是 session 的行数（/health 的「别的会话在跑 M 个」：没有会话的人手任务也是别的会话）。"""
        return self.connection().execute(f"SELECT COUNT(*) FROM {table} WHERE state = ? AND session != ?", (state, session)).fetchone()[0]

    def delete_attempt(self, attempt_id):
        with self.write_lock:
            connection = self.connection()
            connection.execute("BEGIN IMMEDIATE")
            connection.execute("DELETE FROM attempts WHERE id = ?", (attempt_id,))
            connection.execute("COMMIT")

    def select(self, table, where="1", parameters=(), order="id"):
        rows = self.connection().execute(f"SELECT id, state, session, command_key, record, signature, task_id FROM {table} WHERE {where} ORDER BY {order}",
                                         parameters).fetchall()
        return [self.decoded(table, row) for row in rows]

    def task(self, task_id):
        found = self.select("tasks", "id = ?", (task_id,))
        return found[0] if found else None

    def attempts_of(self, task_id):
        return self.select("attempts", "task_id = ?", (task_id,))

    def health_write(self):
        """库写不写得进：试写一行 meta（自证的 TASK_SERVER_SELFTEST_FAIL_HEALTH_WRITE 指的文件在时当失败）。交 (写进了没有, 原因)。"""
        failing = lib.selftest_override("TASK_SERVER_SELFTEST_FAIL_HEALTH_WRITE", self.environment)
        if failing and os.path.exists(failing):
            return False, "自证造的写不进"
        try:
            with self.write_lock:
                connection = self.connection()
                connection.execute("BEGIN IMMEDIATE")
                connection.execute("UPDATE meta SET value = ? WHERE name = 'health'", (str(time.time()),))
                connection.execute("COMMIT")
        except sqlite3.Error as error:
            return False, f"{type(error).__name__}: {error}"
        return True, ""

    # ── 暂停态（文件头 ⑤「暂停」）：meta 表里名为 pause 的一行，正文是 {reason, since, allow_session, signature}；没有这一行就是没暂停 ──
    def pause_signature(self, reason, since, allow_session):
        """签名盖住原因、起点和放行的会话（弄坏开关 pause-allow-unsigned 下放行的会话不进签名，改它改不出校验不过）。"""
        body = {"reason": reason, "since": since}
        if not lib.broken("pause-allow-unsigned", self.environment):
            body["allow_session"] = allow_session
        return self.signature("meta", 0, PAUSE_META_NAME, "", PAUSE_META_NAME, lib.canonical_json(body))

    def pause_record(self):
        """库里记的暂停态：没暂停交 None，暂停着交 {reason, since, allow_session, valid}；签名对不上（绕过服务改过）或读不出的按暂停算、不放行谁，valid 为 False。"""
        row = self.connection().execute("SELECT value FROM meta WHERE name = ?", (PAUSE_META_NAME,)).fetchone()
        if row is None:
            return None
        try:
            stored = json.loads(row[0])
            reason, since, allow_session, given = stored["reason"], stored["since"], stored["allow_session"], stored["signature"]
            valid = (isinstance(reason, str) and isinstance(since, (int, float)) and isinstance(allow_session, str) and isinstance(given, str)
                     and hmac.compare_digest(given, self.pause_signature(reason, since, allow_session)))
        except (ValueError, KeyError, TypeError):
            valid = False
        if not valid:
            return {"reason": "暂停记录校验不过（库被绕过服务改过），按暂停算", "since": 0.0, "allow_session": "", "valid": False}
        return {"reason": reason, "since": since, "allow_session": allow_session, "valid": True}

    def write_pause(self, reason, since, allow_session):
        """记暂停态（放行的会话连同原因、起点一起签名）；换放行的会话也是整行重写。弄坏开关 pause-allow-memory-only 下放行的会话不写进库。"""
        if lib.broken("pause-allow-memory-only", self.environment):
            allow_session = ""
        value = lib.canonical_json({"reason": reason, "since": since, "allow_session": allow_session, "signature": self.pause_signature(reason, since, allow_session)})
        with self.write_lock:
            connection = self.connection()
            connection.execute("BEGIN IMMEDIATE")
            connection.execute("INSERT OR REPLACE INTO meta(name, value) VALUES(?, ?)", (PAUSE_META_NAME, value))
            connection.execute("COMMIT")

    def clear_pause(self):
        with self.write_lock:
            connection = self.connection()
            connection.execute("BEGIN IMMEDIATE")
            connection.execute("DELETE FROM meta WHERE name = ?", (PAUSE_META_NAME,))
            connection.execute("COMMIT")


# ── 两台的余量与卡 ──
LOCAL, PEER = "local", "peer"   # 服务记尝试的视角（task_server_lib.HOST_NAMES）：本机是服务所在这一台，第二台是多机配置 HOSTS 里另一台
MACHINE_ORDER = (LOCAL, PEER)   # 挑台时放下之后剩余占比一样大取排在前面的；分片的各片倒过来（candidate_hosts）


@dataclass
class Snapshot:
    reports: dict                                 # {台: 余量报告}：这一轮量得到、能派的台
    memory_caps: dict                             # {台: 那台总上限另收到的数（字节）}：Machine.configured_cap 交得出的台才有
    reachable: dict                               # {台: 此刻连不连得上}（服务所在这一台恒为真；另一台照多机配置判法与 NFS 复核）
    down: dict                                    # {台: 这一轮没给报告的原因}
    cards: list = field(default_factory=list)     # [gpu-card-health.py 的 CardLabel(side 台, address 台标签, index 序号, quota 额度文字, entry 原样那一项)]，按优先级
    cards_problem: str = ""
    hold: bool = False                            # 只在自证里：假余量里写了 hold，一条都不起
    expected: dict = field(default_factory=dict)  # {台: 用不了要不要告警}（note_peer）：只有自证里没给那一台的格是 False
    down_kind: dict = field(default_factory=dict)  # {台: down 那一句是哪一种}（note_peer 按它去重）：盘余量不到下限是 DISK_FLOOR_KIND，别的没有（按原因的字面去重）

    def total(self, host):
        """这台的总上限（字节）：那台 slice 的总上限，另有收到的数（memory_caps）取小者。减那台此刻的余量在 Scheduler.host_capacity。"""
        total = int(self.reports[host]["memory"].get("slice_total") or 0)
        cap = self.memory_caps.get(host)
        return min(total, cap) if cap else total

    def maxima(self):
        """每台的总上限（total）：连得上而这一轮没报告的是 None（查不到），连不上的不在里面。减那台此刻的余量在 Scheduler.usable_maxima（retry_decision 照它比）。"""
        result = {}
        for host in MACHINE_ORDER:
            if host in self.reports:
                result[host] = self.total(host)
            elif self.reachable.get(host):
                result[host] = None
        return result
def parse_cards(text, card_health, addresses, environment):
    """多机配置的显卡先后 → ([CardLabel…], 问题)（文件头 ②「GPU」）：切法与认法照 gpu-card-health.py 的 card_labels（台标签是 HOSTS 里的地址，从右边切两刀），
    addresses 是 host_addresses 交的 Addresses(this, peer)；写坏的那几项不收，问题写头一处。弄坏开关 card-labels-all-local 下台标签一律当这一台的。"""
    if lib.broken("card-labels-all-local", environment):
        found = [card_health.CARD_ENTRY.match(entry) for entry in (text or "").split()]
        return [card_health.CardLabel(LOCAL, match.group("address"), int(match.group("index")), match.group("quota"), match.group(0))
                for match in found if match], ""
    cards, problem = card_health.card_labels(text, addresses)
    return cards, (f"多机配置的显卡先后里{problem}" if problem else "")


def card_key(entry):
    """一张卡在账上的键（busy_cards）：原样那一项切出来的（台标签, 序号）；切不开交 None。"""
    found = CARD_ENTRY.match(entry or "")
    return (found.group("address"), int(found.group("index"))) if found else None


def card_keys(text):
    """尝试记下的卡（一张或同一台的几张，原样那几项用空格接；文件头 ②「GPU」的张数）在账上的键：逐项 card_key，切不开的那项不算。"""
    return [key for key in (card_key(entry) for entry in (text or "").split()) if key is not None]


def peer_build_root(declarations):
    """第二台的编译根：多机配置 BUILD_ROOT_BY_HOST 里第二台那一项（host-dispatch.py 的 Peer 读出的 declare -p 那几行里的 PEER_HOST_BUILD_ROOT，
    peer-host-lib.sh 按地址取），去掉末尾的 /；读不到交空串。"""
    found = re.search(r"PEER_HOST_BUILD_ROOT=\"?([^\"\n]+?)\"?$", declarations or "", re.MULTILINE)
    return found.group(1).rstrip("/") if found else ""


def df_available_bytes(output):
    """df -B1 --output=avail 的输出（表头一行、数一行）：末行的字节数；认不出交 None。"""
    lines = [line.strip() for line in (output or "").splitlines() if line.strip()]
    return int(lines[-1]) if lines and lines[-1].isdigit() else None


def gibibytes_text(byte_count):
    return f"{byte_count / lib.GIBIBYTE:.1f}G"


def attention_tier(value, first):
    """要处置的提醒的档（文件头 ①「要处置的提醒」）：value 不到 first 是 0，到了 first 是 1，到了 2 × first 是 2，到了 4 × first 是 3……（每档翻倍）。"""
    if first <= 0 or value < first:
        return 0
    tier = 1
    while value >= first * 2 ** tier:
        tier += 1
    return tier


# ── 两台：这一台怎么读、怎么起（文件头 ②「两台同一段逻辑」）──
@dataclasses.dataclass
class BuildDirectory:
    """交给那一台起法的这一次编译目录（Machine.command 的 build）。"""
    directory: str   # target 的上一层
    reused: bool     # 接着用上一次尝试留下的那一份（文件头「编译目录」的重试复用）


class Machine:
    """一台机器在服务这一侧的接口：记账、判放不放得下、判卡住、记峰值、上限、编译目录、挑台与盘余量对两台走同一段（Scheduler 里），
    按台不同的只在这里——怎么连上它、怎么量它的余量与盘、怎么读一次尝试此刻的 CPU 与编译目录占盘、怎么起那一次、它的编译目录由谁建。
    两台型号相同（用户 2026-11-02：「两台型号都是一样的 根本没有必要写 if 第一台 else 第二台这种代码」）：这里只有摸到那台的路不同，判法不分台。"""
    name = ""
    always_reachable = False     # 服务所在这一台总连得上；另一台要照多机配置判法与 NFS 复核
    starts_per_pass = None       # 一轮至多在这台起几条（None 不限）
    admission_variable = ""      # 这一台最外层那一层只认服务起的准入标记（task_server_lib.py 的 ADMITTED_VARIABLE / PEER_ADMITTED_VARIABLE）
    admission_break = None       # 只给自证证红：照改前不设这一台准入标记的弄坏开关

    def __init__(self, scheduler):
        self.scheduler = scheduler

    @property
    def label(self):
        return lib.HOST_NAMES[self.name]

    def address(self, addresses):
        """这一台在多机配置 HOSTS 里的地址（Addresses 是 task_server_lib.host_addresses 交的）。"""
        raise NotImplementedError

    def reachable(self, resources):
        """→ (此刻连不连得上, 连不上的原因)。"""
        raise NotImplementedError

    def probe(self, resources, gpu):
        """→ (这台此刻的余量报告或 None, 查不成的原因)；报告里另带 DISK_REPORT_KEY（这台编译目录所在盘的余量，量得到才带）。"""
        raise NotImplementedError

    def disk_free(self, resources):
        """报告里没带盘余量时现量：→ (退出码, 输出, 错误)，输出末行是字节数（同 df -B1 --output=avail）。"""
        raise NotImplementedError

    def disk_advice(self):
        raise NotImplementedError

    def configured_cap(self, resources):
        """这台的总上限另收到的数（字节）：多机配置 SLICE_MEMORY_CAP_BY_HOST 里这台那一项照它现读得到的；没有另收的交 None。"""
        return None

    def cpu_usec(self, attempt):
        """这一次尝试此刻累计用了多少 CPU（微秒）；读不到交 None。"""
        raise NotImplementedError

    def build_bytes(self, attempt):
        """这一次尝试的编译目录此刻的占盘（字节）；量不到交 None（记账照起的时候记的算）。"""
        return None

    def card_busy(self, attempt):
        """这一次尝试此刻在它那张卡上有没有计算负载；量不到当有（不报疑似卡住）。"""
        return True

    def command_exit(self, exit_code, tail):
        """→ (定结局用的退出码, classify_attempt 的 host, 起法那一层自己退的码或 None)。"""
        return exit_code, self.name, None

    def ended_peaks(self, tail):
        """尝试结束时起法那一层打进日志的峰值：{memory_peak, build_peak}（没打的不带）。"""
        return {}

    def kept_build_label(self, tail):
        """尝试结束时起法那一层报的「这一次的编译目录留着」（第二台是 multi-host-run.sh 打的 PEER_BUILD_KEPT_MARK 那一行）：→ 留着的那一份的标签或 None。"""
        return None

    def command(self, task, placement, label, environment, capped, build):
        """build 是这一次的编译目录（BuildDirectory；服务不给编译目录的是 None）。"""
        raise NotImplementedError

    def build_directory(self, label):
        """这一次尝试的编译目录：→ (目录, 是不是服务在本机建、由服务删)。目录是 target 的上一层；给不出交空串。"""
        raise NotImplementedError

    def build_directory_exists(self, directory):
        """一次尝试记着、没记成已删的编译目录此刻还在不在（文件头「编译目录」的重试复用与收尾删）。"""
        raise NotImplementedError

    def remove_build_directory(self, directory):
        """删一次尝试留下的编译目录：→ (删了、回读不在了, 没删成时的说明行)。有进程开着里面的文件就不删。"""
        raise NotImplementedError

    def removal_howto(self, directory):
        """没删成时告警的出路。"""
        raise NotImplementedError


class LocalMachine(Machine):
    """服务所在这一台：余量照 host-dispatch.py 的 probe_local 现读，CPU 读 attempt_runner 写的 cpu_usec，起法是内存包装。"""
    name = LOCAL
    always_reachable = True
    admission_variable = lib.ADMITTED_VARIABLE

    def address(self, addresses):
        return addresses.this

    def reachable(self, resources):
        return True, ""

    def probe(self, resources, gpu):
        report, why = resources.host_dispatch.probe_local(resources.settings, gpu)
        if report is not None:
            with contextlib.suppress(OSError):   # 本机编译目录所在盘（TASK_SERVER_BUILD_DIRECTORY）的余量
                report[DISK_REPORT_KEY] = shutil.disk_usage(resources.build_directory).free
        return report, why

    def disk_free(self, resources):
        try:
            return 0, f"Avail\n{shutil.disk_usage(resources.build_directory).free}\n", ""
        except OSError as error:
            return 1, "", f"{type(error).__name__}: {error}"

    def disk_advice(self):
        return "先 task-server/task clean 删掉跑完留下的编译目录（多机配置 TASK_SERVER_BUILD_DIRECTORY 所在盘）与别的大目录"

    def cpu_usec(self, attempt):
        return self.scheduler.scope_cpu_usec(attempt)

    def build_bytes(self, attempt):
        with contextlib.suppress(OSError, ValueError):
            with open(os.path.join(attempt.get("directory", ""), "build_bytes"), encoding="utf-8") as handle:
                return int(handle.read().strip())
        return None

    def card_busy(self, attempt):
        """nvidia-smi -i <序号> 的计算进程里有这一次尝试的子孙进程（从尝试目录的 started 记的 runner 进程号往下）就算有；没记卡、读不到进程号、
        nvidia-smi 起不来或退非 0 当有。"""
        keys = card_keys(attempt.get("card"))   # 分到同一台几张的：几张一起问（nvidia-smi -i 收逗号连的几个序号）
        if not keys:
            return True
        scheduler = self.scheduler
        try:
            with open(scheduler.attempt_paths(attempt)[1], encoding="utf-8") as handle:
                runner = int(handle.read().strip())
        except (OSError, ValueError):
            return True
        family = descendant_processes(runner) | {runner}
        try:
            completed = subprocess.run([scheduler.nvidia_smi, "-i", ",".join(str(key[1]) for key in keys), "--query-compute-apps=pid,used_memory", "--format=csv,noheader,nounits"],
                                       capture_output=True, text=True, env=scheduler.environment, stdin=subprocess.DEVNULL, timeout=NVIDIA_SMI_SECONDS)
        except (OSError, subprocess.TimeoutExpired):
            return True
        if completed.returncode != 0:
            return True
        listed = {int(line.split(",")[0]) for line in completed.stdout.splitlines() if line.split(",")[0].strip().isdigit()}
        return bool(listed & family)

    def command(self, task, placement, label, environment, capped, build):
        # 编译目录（连同重排接着用的那一份）经 environment 的 CARGO_TARGET_DIR 交给命令（start_attempt），这里不用 build
        return ["bash", self.scheduler.memory_wrapper, lib.mebibytes_text(placement.cap)] + capped + task["command"]

    def build_directory(self, label):
        return os.path.join(self.scheduler.configuration.build_directory, label), True

    def build_directory_exists(self, directory):
        return os.path.isdir(directory)

    def remove_build_directory(self, directory):
        removed, lines = self.scheduler.build_directories.remove_run_build_directory(directory)
        return removed and not os.path.lexists(directory), lines

    def removal_howto(self, directory):
        return "照上面那句停掉开着它的进程，再 python3 scripts/hosts/run_build_directory.py remove " + directory


class PeerMachine(Machine):
    """多机配置 HOSTS 里的另一台：经 host-dispatch.py 的 Peer（多机配置判法）连、probe_peer 在那台上量余量；CPU 读日志里 multi-host-run.sh 每轮打的
    MULTI_HOST_RUN_PEER_CPU；起法是 multi-host-run.sh --peer，编译目录由它在那台本地盘上这一趟新建、跑完删（multi-host-run.sh 文件头 ⑤）。"""
    name = PEER
    starts_per_pass = PEER_STARTS_PER_PASS
    admission_variable = lib.PEER_ADMITTED_VARIABLE
    admission_break = "peer-admission-unmarked"

    def address(self, addresses):
        return addresses.peer

    def reachable(self, resources):
        return resources.peer.available, resources.peer.why

    def probe(self, resources, gpu):
        return resources.host_dispatch.probe_peer(resources.peer, gpu)   # 查不成的原因留给 note_peer 告警

    def disk_free(self, resources):
        directory = peer_build_root(resources.peer.declarations)
        if not directory:
            return 1, "", "多机配置 BUILD_ROOT_BY_HOST 里读不到第二台那一项"
        return resources.peer.run(PEER_DISK_PROBE_TEXT.format(directory=shlex.quote(directory)))

    def disk_advice(self):
        return ("先删第二台的编译根（多机配置 BUILD_ROOT_BY_HOST 里它那一项）下 builds/ 里留下的本地目录（multi-host-run.sh 没删成的那几个，它当时报了原因）"
                "与别的大目录")

    def configured_cap(self, resources):
        return resources.peer.memory_cap_bytes if resources.peer is not None else None

    def cpu_usec(self, attempt):
        return self.scheduler.log_offsets.get(attempt.get("id"), (0, 0, None))[2]

    def command_exit(self, exit_code, tail):
        # 第二台那条命令自己的退出码（multi-host-run.sh 打的 MULTI_HOST_RUN_STEP step=peer-command event=end … exit=<码>）：有这一行照它定结局，
        # 命令自己退 255 记 command-failed；没有这一行才是 multi-host-run.sh 那一层没跑成（task_server_lib.py 文件头「错误类型」）
        command_exit = lib.peer_command_exit(tail, self.scheduler.environment) if exit_code is not None else None
        if command_exit is None:
            return exit_code, self.name, None
        return command_exit, lib.PEER_COMMAND_HOST, exit_code

    def ended_peaks(self, tail):
        # multi-host-run.sh 在第二台那条命令结束时打的 MULTI_HOST_RUN_PEER_MEMORY_PEAK 与 MULTI_HOST_RUN_PEER_BUILD_PEAK；读不到不打
        environment = self.scheduler.environment
        peaks = {"memory_peak": None if lib.broken("peer-memory-peak-ignored", environment) else lib.peer_marked_bytes(tail, lib.PEER_MEMORY_PEAK_MARK),
                 "build_peak": lib.peer_marked_bytes(tail, lib.PEER_BUILD_PEAK_MARK)}
        return {name: value for name, value in peaks.items() if value is not None}

    def kept_build_label(self, tail):
        return lib.peer_kept_build_label(tail)

    def build_directory_exists(self, directory):
        # 第二台本地盘上的那一份服务这一侧看不到：记着留下了就当在（multi-host-run.sh 接着用之前在第二台上核它在不在，不在退 255）
        return True

    def remove_build_directory(self, directory):
        # multi-host-run.sh --remove-build-directory 在第二台上核没有进程开着里面的文件、删、回读不在了才退 0（它文件头「留着的编译目录」）
        try:
            completed = subprocess.run(["bash", self.scheduler.peer_runner, "--remove-build-directory", os.path.basename(directory)],
                                       capture_output=True, text=True, env=self.scheduler.environment, stdin=subprocess.DEVNULL, timeout=PEER_BUILD_REMOVE_SECONDS)
        except (OSError, subprocess.TimeoutExpired) as error:
            return False, [f"multi-host-run.sh --remove-build-directory 起不了或超过 {PEER_BUILD_REMOVE_SECONDS} 秒：{error}"]
        return completed.returncode == 0, (completed.stdout + completed.stderr).splitlines()[-5:]

    def removal_howto(self, directory):
        return ("照上面那句到第二台停掉开着它的进程（python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号>），再 bash scripts/hosts/multi-host-run.sh --remove-build-directory "
                + os.path.basename(directory))

    def command(self, task, placement, label, environment, capped, build):
        scheduler = self.scheduler
        # 两个共享根照来源仓根那样原样带过去（task_server_lib.py 文件头「共享根」）；弄坏开关 shared-roots-not-passed 下照改前不带
        shared_roots = () if lib.broken("shared-roots-not-passed", scheduler.environment) else lib.SHARED_ROOT_VARIABLES
        carried_names = ("GPU_CARDS_BY_PRIORITY", "TASK_SERVER_TASK", "TASK_SERVER_ATTEMPT", lib.ATTEMPT_LABEL_VARIABLE, lib.THREAD_ADMITTED_VARIABLE, lib.MEMORY_CAP_VARIABLE,
                         lib.SOURCE_REPOSITORY_VARIABLE, lib.SHARD_PHASE_VARIABLE, lib.SHARD_VARIABLE, lib.SHARD_PLAN_VARIABLE, lib.SHARD_OUTPUT_VARIABLE, lib.SHARD_HOST_VARIABLE,
                         lib.SHARD_OUTPUTS_VARIABLE, lib.SHARD_MISSING_VARIABLE, *shared_roots)   # 合并那一步派到第二台也读得到各片的清单（那份 TSV 在共享根下的分片根里）与缺了哪几片
        if lib.broken("shard-outputs-not-carried", scheduler.environment):   # 弄坏开关：照改前不带各片的清单（task_server_lib.py 文件头「分片」③）
            carried_names = tuple(name for name in carried_names if name != lib.SHARD_OUTPUTS_VARIABLE)
        pairs = [f"{name}={value}" for name, value in sorted(environment.items()) if name in task["environment"] or name in carried_names]
        # 本机的路径与编译目录，第二台用它自己的（编译目录由 multi-host-run.sh 在第二台本地盘上建）
        pairs = [pair for pair in pairs if not pair.startswith(("PATH=", "CARGO_HOME=", "RUSTUP_HOME=", "TMPDIR=", *(name + "=" for name in lib.TARGET_DIRECTORY_VARIABLES)))]
        if not lib.broken("host-specific-variables-sent-to-peer", scheduler.environment):
            # 按台取值的（预编库这类）不带；值是本机绝对路径的只带落在共享根下的（第二台照同一个绝对路径读得到：来源仓根、两个共享根、分片的计划与产物目录），别的第二台用它自己的
            pairs = [pair for pair in pairs if pair.split("=", 1)[0] not in PEER_HOST_SPECIFIC_VARIABLES
                     and (not pair.split("=", 1)[1].startswith("/") or pair.split("=", 1)[0] in (lib.SOURCE_REPOSITORY_VARIABLE, *shared_roots)
                          or scheduler.containing_shared_root(os.path.realpath(pair.split("=", 1)[1])))]
        # 判定库服务的地址不带过去：第二台照 multi-host-run.sh 带过去的 SINGLEFS_MULTI_HOST_CONFIG 读同一份多机配置里的 PRUNE_SERVER_TCP_ADDRESS；
        # 配置里没写，第二台上的判定库客户端退 3、点名缺的是哪一样（scripts/prune-server/prune_server_client.py 文件头「连哪一处」），服务不从别处补
        command = list(task["command"])
        copied = []
        if lib.broken("peer-tree-copied", scheduler.environment):   # 弄坏开关：照改前把工作目录拷进第二台的一棵树、在那份拷贝里起
            copied = ["--send", task["cwd"], ".task-server-tree-copy"]
            command = ["env", "-C", ".task-server-tree-copy", *command]
        # 服务给编译目录的（会编的）叫 multi-host-run.sh 跑完留着这一份、重排到这一台的那一次接着用上一次留下的（文件头「编译目录」的重试复用，两台同一段）；
        # 弄坏开关 peer-retry-build-directory-fresh 下照改前不留、不接着用
        kept = []
        if build is not None and not lib.broken("peer-retry-build-directory-fresh", scheduler.environment):
            kept = ["--keep-build-directory"] + (["--reuse-build-directory", os.path.basename(build.directory)] if build.reused else [])
        return (["bash", scheduler.peer_runner, "--peer", "--memory-cap", lib.mebibytes_text(placement.cap), "--label", label, *kept, *copied, "--"]
                + (["env", *pairs] if pairs else []) + capped + command)

    def build_directory(self, label):
        # multi-host-run.sh 在这台的编译根（多机配置 BUILD_ROOT_BY_HOST 里它那一项）下 builds/<标签>/ 新建、跑完删（带 --keep-build-directory 的留着，服务删）；服务只记下它在哪
        resources = self.scheduler.resources
        root = resources.build_root_of_peer()
        return (os.path.join(root, "builds", label) if root else ""), False


class Resources:
    """两台的余量与卡：平常经各台的 Machine.probe 现查（缓存 RESOURCE_CACHE_SECONDS 秒，尝试结束时与起之前复核时作废）；自证里读 TASK_SERVER_FAKE_RESOURCES 那份 JSON（每次现读）。
    两台同一套判法：每台照它自己的接口（Machine）连、量，盘余量照同一个下限判（disk_problem）。"""

    def __init__(self, environment, host_dispatch, build_directory, configuration_path):
        self.environment = environment
        self.host_dispatch = host_dispatch
        self.build_directory = build_directory   # 本机编译目录所在盘（TASK_SERVER_BUILD_DIRECTORY）：余量记进本机报告的 DISK_REPORT_KEY
        self.configuration_path = configuration_path   # 多机配置：卡的台标签照它的 HOSTS 认（addresses）
        self.addresses_read = None   # (配置文件的修改时刻与大小, 读的时刻, Addresses 或 None, 问题)
        self.fake_path = lib.selftest_override("TASK_SERVER_FAKE_RESOURCES", environment)
        self.slow_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_SLOW_RESOURCES_SECONDS", environment) or 0)
        self.settings = host_dispatch.settings_from_environment()
        self.settings.say = lambda line: None
        self.card_health = lib.load_module("gpu_card_health", os.path.join(lib.HOSTS_DIRECTORY, "gpu-card-health.py"))
        self.cached = {}
        self.peer = None
        self.peer_read_at = 0.0
        self.machines = {}   # {台: Machine}：Scheduler 建好之后放进来（两台各一份接口）

    def invalidate(self):
        self.cached = {}

    def build_root_of_peer(self):
        """第二台的编译根：真服务读多机配置里它那一项（peer_build_root）；自证读假余量 JSON 的 peer_build_root（自证不连真的第二台、self.peer 永远是 None，
        假 multi-host-run.sh 在那个目录下 builds/<标签>/ 建这一趟的编译目录）。读不到交空串。"""
        if self.fake_path:
            return (read_json_file(self.fake_path) or {}).get("peer_build_root") or ""
        return peer_build_root(self.peer.declarations) if self.peer is not None else ""

    def addresses(self):
        """(Addresses(this, peer) 或 None, 问题)：服务所在这一台与另一台在 HOSTS 里的地址（文件头 ②「GPU」）。配置文件没改过、又没过 PEER_RECHECK_SECONDS 的照上一次的，不每轮都起判法。"""
        try:
            status = os.stat(self.configuration_path)
            stamp = (status.st_mtime_ns, status.st_size)
        except OSError:
            stamp = None
        cached = self.addresses_read
        if cached is not None and cached[0] == stamp and time.monotonic() - cached[1] < PEER_RECHECK_SECONDS:
            return cached[2], cached[3]
        addresses, problem = lib.host_addresses(self.configuration_path, self.environment)
        self.addresses_read = (stamp, time.monotonic(), addresses, problem)
        return addresses, problem

    def cards_of(self, text):
        """显卡先后的值 → ([CardLabel…], 问题)：先认 HOSTS 里哪一项是这一台，认不出的当一张卡都没点名、问题写那一句。"""
        if not (text or "").split():
            return [], ""
        addresses, problem = self.addresses()
        if addresses is None:
            return [], f"认不出多机配置 HOSTS 里哪一项是服务所在这一台，卡的台标签认不了（{problem}）"
        return parse_cards(text, self.card_health, addresses, self.environment)

    def refresh_peer(self):
        """多机配置里的另一台（host-dispatch.py 的 Peer）至多每 PEER_RECHECK_SECONDS 重读一次。"""
        if self.peer is None or time.monotonic() - self.peer_read_at > PEER_RECHECK_SECONDS:
            self.peer = self.host_dispatch.Peer(self.settings)
            self.peer_read_at = time.monotonic()

    def fake_report(self, fake, machine):
        """自证：假余量 JSON 里这一台的报告（{台: 报告}；down 里写了这一台的演它此刻用不了）→ (报告或 None, 原因)。"""
        why = (fake.get("down") or {}).get(machine.name)
        if fake.get(machine.name) and not why:
            return fake[machine.name], ""
        return None, why or f"自证里没给{machine.label}"

    def snapshot(self, gpu):
        fake = None
        if self.fake_path:
            if self.slow_seconds:   # 自证：演余量读得慢（/health 不该跟着慢）
                time.sleep(self.slow_seconds)
            fake = read_json_file(self.fake_path) or {}
        else:
            cached = self.cached.get(gpu)
            if cached is not None and time.monotonic() - cached[0] < RESOURCE_CACHE_SECONDS:
                return cached[1]
            self.refresh_peer()
        reports, reachable, down, down_kind, expected, caps = {}, {}, {}, {}, {}, {}
        for host, machine in self.machines.items():
            if fake is not None:   # 假余量里写了 down（{台: 一句原因}）的演那台用不了：照真的那样要告警；没给那一台的格不当它用不了
                report, why = self.fake_report(fake, machine)
                reachable[host] = machine.always_reachable or report is not None
                expected[host] = host in fake or bool((fake.get("down") or {}).get(host))
                cap = (fake.get("memory_caps") or {}).get(host)
            else:
                reachable[host], why = machine.reachable(self)
                report = None
                if reachable[host]:
                    report, why = machine.probe(self, gpu)
                expected[host] = True
                cap = machine.configured_cap(self)
            if cap:
                caps[host] = int(cap)
            if report is not None:
                disk_problem = self.disk_problem(machine, report, fake)
                if disk_problem:   # 盘余量不到下限或查不成：这一轮不给这台的报告、不派它（同查不成余量的那一种，够了下一轮自己认回）
                    report, why, down_kind[host] = None, disk_problem, DISK_FLOOR_KIND
            if report is None:
                down[host] = why
            else:
                reports[host] = report
        cards, problem = [], ""
        if fake is not None:
            cards, problem = self.cards_of(fake.get("cards", ""))
        elif gpu:
            holders = self.host_dispatch.live_quiesce_holders(self.settings)
            for report in reports.values():
                report["quiesce_holders"] = holders
            _, values, problem = self.card_health.read_configuration(self.settings.environment)
            if not problem:
                text = values.get("GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED", "") or values.get("GPU_CARDS_BY_PRIORITY", "")
                cards, problem = self.cards_of(text)
        snapshot = Snapshot(reports, caps, reachable, down, cards, problem, bool((fake or {}).get("hold")), expected=expected, down_kind=down_kind)
        if fake is None:
            self.cached[gpu] = (time.monotonic(), snapshot)
        return snapshot

    def disk_problem(self, machine, report, fake=None):
        """这台编译目录所在盘的余量够不够（task_server_lib.py 文件头「挑机器」的「盘余量」；两台同一个下限 DISK_FLOOR_BYTES）：→ 不到下限或查不成的那一句，够了交空串。
        report 里有 DISK_REPORT_KEY 照它；没有就照这台的接口现量（Machine.disk_free：本机读编译目录所在盘，第二台在它的编译根上起 df）。
        自证里（fake 是假余量那份 JSON）没有这一项读 fake 的 df 里这一台那一项（演 df 的输出），都没写的当盘够。弄坏开关 disk-floor-unchecked 下一律交空串。"""
        if lib.broken("disk-floor-unchecked", self.environment):
            return ""
        available = report.get(DISK_REPORT_KEY)
        if not isinstance(available, int) or isinstance(available, bool):
            if fake is not None:
                shown = (fake.get("df") or {}).get(machine.name)
                if shown is None:
                    return ""
                status, output, error = 0, str(shown), ""
            else:
                status, output, error = machine.disk_free(self)
            available = df_available_bytes(output) if status == 0 else None
            if available is None:
                return f"{machine.label}盘余量查不成（退 {status}：{(error or output).strip()[-200:]}）"
            report[DISK_REPORT_KEY] = available   # 记进报告：派活时照它判这一条的编译目录占盘放不放得下（文件头 ②「编译目录占盘」）
        if available < DISK_FLOOR_BYTES:
            return f"{machine.label}盘只剩 {gibibytes_text(available)}（下限 {gibibytes_text(DISK_FLOOR_BYTES)}）：编译会在中途写满，{machine.disk_advice()}"
        return ""

    def recheck(self, gpu, host, base):
        """起之前现读一次那一台（文件头 ②「起之前复核」）：缓存作废，交一份新的 Snapshot，这台的余量是现读的，别的照 base；这台读不到的，交的那份里没有它。
        自证里读假余量 JSON，里面写了 recheck 的（{台: 余量}）这一次照它。"""
        self.invalidate()
        machine = self.machines[host]
        if self.fake_path:
            fake = read_json_file(self.fake_path) or {}
            report = dict(fake, **(fake.get("recheck") or {})).get(host)
        else:
            self.refresh_peer()
            report = machine.probe(self, gpu)[0] if machine.reachable(self)[0] else None
            if report is not None and gpu:
                report["quiesce_holders"] = self.host_dispatch.live_quiesce_holders(self.settings)
        reports = {name: value for name, value in base.reports.items() if name != host}
        if report is not None:
            reports[host] = report
        return dataclasses.replace(base, reports=reports)


class PassView:
    """一轮排队读的余量（文件头 ②「余量一轮只读一次」）：每台的余量（Resources.snapshot）、服务账上还没涨到量的那一截（server_pending）
    在这一轮第一次用到时读，这一轮里各条共用；这一轮起了一条，它的记账加进那台的待涨量（book），后面的照加过的判。
    弄坏开关 pending-read-per-task 下每次用到都现读，pass-booking-not-carried 下起了的不加。"""

    def __init__(self, scheduler, source):
        self.scheduler = scheduler
        self.source = source              # pass（排队那一轮）、health（/health 判排头）
        self.number = next(scheduler.view_numbers)
        self.snapshots = {}               # {要不要卡: Snapshot}
        self.latest_snapshot = None       # 这一轮最近读进来的那一份 Snapshot：待涨量照它里面第二台的报告对 scope（server_pending）
        self.pending = {}                 # {台: (内存字节, 显存 MiB)}：读到的待涨量
        self.disk_pending = {}            # {台: 字节}：服务账上这台在跑的尝试编译目录还要涨的占盘，连同这一轮起了的那几条的记账
        self.carried = {}                 # {台: (内存字节, 显存 MiB)}：这一轮起了的那几条的记账

    def read_each_time(self):
        return lib.broken("pending-read-per-task", self.scheduler.environment)

    def snapshot(self, gpu):
        if gpu not in self.snapshots or self.read_each_time():
            self.scheduler.selftest_read(f"read view={self.number} source={self.source} what=snapshot gpu={int(gpu)}")
            self.snapshots[gpu] = self.scheduler.peer_view(self.scheduler.resources.snapshot(gpu))
            self.latest_snapshot = self.snapshots[gpu]
        return self.snapshots[gpu]

    def read_pending(self, host, source):
        """现读这台的待涨量；读进来的已含库里此刻在跑的每一次尝试，这一轮起了的那几条不再另加。第二台的用量照这一轮最近读进来的那一份余量报告
        （latest_snapshot；这一轮还没读余量时没有报告，第二台在跑的记整份上限），不为它另读一次余量。"""
        self.scheduler.selftest_read(f"read view={self.number} source={source} what=pending host={host}")
        report = self.latest_snapshot.reports.get(host) if self.latest_snapshot is not None else None
        self.pending[host] = self.scheduler.server_pending(host, report)
        self.carried.pop(host, None)

    def pending_of(self, host):
        """(内存字节, 显存 MiB)：这台还没涨到量的那一截，连同这一轮起了的那几条的记账。"""
        if host not in self.pending or self.read_each_time():
            self.read_pending(host, self.source)
        memory, gpu = self.pending[host]
        carried_memory, carried_gpu = self.carried.get(host, (0, 0))
        return memory + carried_memory, gpu + carried_gpu

    def book(self, host, memory, gpu, disk=0):
        """这一轮在这台起了一条：把它的记账加进这一轮那台的待涨量。"""
        if lib.broken("pass-booking-not-carried", self.scheduler.environment):
            return
        carried_memory, carried_gpu = self.carried.get(host, (0, 0))
        self.carried[host] = (carried_memory + int(memory or 0), carried_gpu + int(gpu or 0))
        self.disk_pending[host] = self.disk_pending_of(host) + int(disk or 0)

    def disk_pending_of(self, host):
        """这台在跑的尝试的编译目录还要涨的占盘（server_disk_pending，这一轮第一次用到时读），连同这一轮起了的。"""
        if host not in self.disk_pending:
            self.disk_pending[host] = self.scheduler.server_disk_pending(host)
        return self.disk_pending[host]

    def refresh(self, host, gpu, snapshot):
        """起之前复核现读过的：现读的余量换进这一轮，这台的待涨量也现读（复核那一读记 source=recheck）。"""
        self.snapshots[gpu] = snapshot
        self.latest_snapshot = snapshot
        self.read_pending(host, "recheck")


class Alternation:
    """一轮排队里的整片与碎片交替（文件头 ②「整片与碎片按台交替」）：开轮时数每台服务起的在跑几条、在排的没满时限的里哪几条在等那台是整片（一条整片只算一台，waiting_hosts）、哪几条是碎片；
    组的账（{台: {"booked": 字节, "sealed": 封了没有}}）在 Scheduler.fragment_groups 里，跨轮留着。"""

    def __init__(self, scheduler, view, queued, now):
        self.scheduler = scheduler
        self.groups = scheduler.fragment_groups
        self.now = now
        self.estimates = {}    # {尝试的记录号: 它那条任务的预计用时或 None}：这一轮算挡路时长用（blocking_seconds），每条在跑的只算一次
        self.gap_fills = {}    # {(任务号, 台): 插空那一句}：这一轮封着组而插得进空的（gap_fill），起了记进尝试，内存放不下的写进排队原因
        self.whole_bookings = {}   # {(任务号, 台): 记账}：在等那台的整片在那台上的记账（gap_fill 算排头整片起来之后那台还剩多少）
        self.occupied = {}
        for attempt, _ in scheduler.store.select("attempts", "state IN ('starting', 'running')"):
            self.occupied[attempt.get("host")] = self.occupied.get(attempt.get("host"), 0) + 1
        self.wholes, self.fragments = {}, {}   # {台: {任务号}}
        busy = scheduler.busy_cards()
        kept_hosts = {}
        for task, valid in queued:
            if not valid or scheduler.aged(task, now):
                continue
            try:
                snapshot = view.snapshot(bool(task.get("gpu")))
                prediction = scheduler.prediction_of(task)
                hosts = [host for host in scheduler.candidate_hosts(task, snapshot) if host in snapshot.reports
                         and scheduler.host_limit(prediction, host, snapshot)[0] is not None]   # 余量量不到的台这一轮不派（文件头 ②「余量」），不算它在等那台
                bookings = {host: int(scheduler.booked_at_cap(prediction, host, snapshot).memory_booked or 0) for host in hosts}
                whole_on = [host for host in hosts if scheduler.is_whole(bookings[host], host, snapshot)]
                chosen = self.waiting_hosts(task, whole_on, bookings, snapshot, busy)
            except (OSError, ValueError, KeyError, subprocess.SubprocessError):
                continue   # 判不了的这一条：place 那一步照样跳过它，这里不算它在等
            for host in hosts:
                if host not in whole_on:
                    self.fragments.setdefault(host, set()).add(task["id"])
            for host in chosen:   # 整片只在等腾空的那一台上算（waiting_hosts），别的台不为它封组
                self.wholes.setdefault(host, set()).add(task["id"])
                self.whole_bookings[(task["id"], host)] = bookings[host]
            if len(chosen) == 1:
                kept_hosts[task["id"]] = chosen[0]
        scheduler.whole_hosts = kept_hosts   # 不在队里了的、这一轮不算整片的不留
        for host, group in self.groups.items():
            if group["sealed"] and not self.wholes.get(host):
                self.groups[host] = {"booked": 0, "sealed": False}   # 等着的整片不在了（起在别台、取消了）：开下一组

    def group(self, host):
        return self.groups.setdefault(host, {"booked": 0, "sealed": False})

    def waiting_hosts(self, task, whole_on, bookings, snapshot, busy):
        """这条整片在哪一台上等腾空（文件头 ②「整片与碎片按台交替」）：交至多一台的列表。whole_on 是它在上面算整片的那几台，bookings 是它在各台的记账。
        要卡的只算空卡凑得够它要的张数的台（卡被占着时腾空内存也起不了它）；上一轮已在一台上封了组等它、那台还在 whole_on 里的接着等那台；
        别的挑此刻离放得下最近的那台：它的记账减那台照公式（room_by_the_formula）还剩的最小，一样小取排在前面的（candidate_hosts 的先后）。
        弄坏开关 whole-seals-every-host 下照改前 whole_on 里每一台都算，card-waiting-whole-seals 下要卡的不看空卡。"""
        scheduler = self.scheduler
        if task.get("gpu") and not lib.broken("card-waiting-whole-seals", scheduler.environment):
            wanted = int(task.get("gpu_cards") or 1)
            whole_on = [host for host in whole_on
                        if sum(1 for card in snapshot.cards if card.side == host and (card.address, card.index) not in busy) >= wanted]
        if lib.broken("whole-seals-every-host", scheduler.environment) or not whole_on:
            return list(whole_on)
        previous = scheduler.whole_hosts.get(task["id"])
        if previous in whole_on and self.group(previous)["sealed"]:
            return [previous]

        def shortfall(host):
            room = scheduler.room_by_the_formula(host, snapshot)
            return math.inf if room is None else bookings[host] - room

        return [min(whole_on, key=lambda host: (shortfall(host), whole_on.index(host)))]

    def blocked(self, task, host, booked, snapshot):
        """这一条（没满时限）此刻在这台上照交替起不起：起交空串，不起交排队原因（都带「在等这台腾空」）。判到下一条碎片放不下这一组、又有整片在等时封组；
        封着组而插得进空的（gap_fill）交空串、插空那一句记进 gap_fills，插不进的在封组那一句后面接上为什么。
        整片：这一组已封、或起过碎片、或没有碎片在等就起，放不放得下只看内存、不等那台腾空（弄坏开关 whole-waits-for-empty-host 下照改前等服务起的都跑完）。"""
        scheduler = self.scheduler
        threshold = scheduler.whole_threshold(host, snapshot)
        group = self.group(host)
        human = scheduler.host_dispatch.human
        waiting = sorted(self.wholes.get(host, set()) - {task["id"]})
        if booked >= threshold:
            if self.occupied.get(host, 0) and lib.broken("whole-waits-for-empty-host", scheduler.environment):
                return (f"整片任务（这台上记 {human(booked)}，不小于可用上限的 {WHOLE_FRACTION:.0%}：{human(threshold)}）在等这台腾空："
                        f"服务在这台上还有 {self.occupied[host]} 条在跑（弄坏开关 whole-waits-for-empty-host：改前的样子）")
            if not group["sealed"] and not group["booked"] and self.fragments.get(host, set()) - {task["id"]}:
                return f"整片任务在等这台腾空之前先起一组碎片：有 {len(self.fragments[host] - {task['id']})} 条碎片在等这台、这一组还没起过"
            return ""
        if group["sealed"] and not self.occupied.get(host, 0) and not lib.broken("sealed-group-held-when-drained", scheduler.environment):
            # 封了组、这台已腾空（服务在这台一条都没在跑）而整片仍没起（放不下）：等下去这台也只是空着，开下一组让碎片接着起
            self.groups[host] = {"booked": 0, "sealed": False}
            group = self.groups[host]
        if not group["sealed"] and group["booked"] + booked > threshold and waiting:
            group["sealed"] = True   # 按队列先后装到再放这一条就超过界：封组，腾空之后先起整片
        if group["sealed"]:
            wholes = "、".join(map(str, waiting)) or "-"
            fits_gap, note = self.gap_fill(task, host, wholes, waiting[0] if waiting else None, booked, snapshot)
            if fits_gap:   # 插空：不占排头整片要的那部分内存、或预计用时不超过挡路时长，交替不挡它，内存照常判（judge_placement）
                self.gap_fills[(task["id"], host)] = note
                return ""
            return (f"这台这一组碎片已装到 {human(group['booked'])}（界 {human(threshold)}）、封组，在等这台腾空起整片任务 {wholes}"
                    + (f"；{note}" if note else ""))
        return ""

    def gap_fill(self, task, host, wholes, head, booked, snapshot):
        """封了组的那台这一条插不插得进空（文件头 ②「整片与碎片按台交替」的插空）：交 (插得进, 那一句)。head 是在等这台的排头整片（任务号最小的那条），booked 是这一条在这台上的记账。
        排头整片起来之后，照用户定的公式（room_by_the_formula：配置的总上限 − 余量 − 在跑的上限之和）这台减去排头的记账还放得下这一条的，不看时长就插得进；
        会占排头那部分的，预计用时不是 None、不超过挡路时长（blocking_seconds）的才插得进。
        弄坏开关 sealed-gap-not-backfilled 下一律插不进（不写那一句），sealed-gap-length-ignored 下有预计用时就插得进，gap-fill-room-ignored 下照改前一律看时长。"""
        scheduler = self.scheduler
        environment = scheduler.environment
        human = scheduler.host_dispatch.human
        if lib.broken("sealed-gap-not-backfilled", environment):
            return False, ""
        room_note = ""
        head_booked = self.whole_bookings.get((head, host)) if head is not None else None
        room = scheduler.room_by_the_formula(host, snapshot) if head_booked is not None else None
        if room is not None and not lib.broken("gap-fill-room-ignored", environment):
            left = room - head_booked
            if int(booked or 0) <= left:
                return True, (f"插空：这台封着组、为整片任务 {wholes} 腾空；排头整片任务 {head}（记 {human(head_booked)}）起来之后，照配置的总上限减余量减在跑的上限之和"
                              f"这台还剩 {human(left)}，放得下这一条的 {human(int(booked or 0))}，不看预计用时")
            room_note = f"；排头整片任务 {head}（记 {human(head_booked)}）起来之后这台照公式只剩 {human(max(left, 0))}，放不下这一条的 {human(int(booked or 0))}，要看预计用时"
        estimate = scheduler.estimated_seconds(task)
        if estimate is None:
            return False, f"没插空：这一条没有预计用时（同一条命令在服务里没跑成过，或按项估缺速率、估算没成）{room_note}"
        blocking, blocker = self.blocking_seconds(host)
        if blocking is None:
            return False, f"没插空：服务在这台上此刻没有在跑的尝试{room_note}"
        gap = f"挡路时长 {blocking:.1f} 秒（这台在跑的任务 {blocker} 还要跑这么久）"
        if estimate <= blocking or lib.broken("sealed-gap-length-ignored", environment):
            return True, f"插空：这台封着组、为整片任务 {wholes} 腾空，这一条预计用时 {estimate:.1f} 秒，不超过{gap}{room_note}"
        return False, f"没插空：这一条预计用时 {estimate:.1f} 秒，超过{gap}{room_note}"

    def blocking_seconds(self, host):
        """挡路时长（文件头 ②「整片与碎片按台交替」的插空）：服务在这台此刻在跑的尝试各自还要跑多久里最大的一个，交 (秒, 那一条的任务号)；一条都没有交 (None, None)。
        还要跑多久 = 它那条任务的预计用时 − 已跑（这一轮的此刻 − started_at）；没有预计用时、或已跑过预计用时的取已跑。
        弄坏开关 blocker-remaining-unbounded 下这两种当成没有尽头。"""
        scheduler = self.scheduler
        longest, blocker = None, None
        for attempt, _ in scheduler.store.select("attempts", "state IN ('starting', 'running')"):
            if attempt.get("host") != host:
                continue
            if attempt["id"] not in self.estimates:
                found = scheduler.store.task(attempt["task_id"])
                self.estimates[attempt["id"]] = scheduler.estimated_seconds(found[0]) if found is not None and found[1] else None
            estimate = self.estimates[attempt["id"]]
            elapsed = max(0.0, self.now - float(attempt.get("started_at") or self.now))
            if estimate is not None and estimate > elapsed:
                remaining = estimate - elapsed
            elif lib.broken("blocker-remaining-unbounded", scheduler.environment):
                remaining = math.inf
            else:
                remaining = elapsed   # 用户定：没有历史用时的（连同已跑过预计用时的）至少还要跑它已经跑了的那么久
            if longest is None or remaining > longest:
                longest, blocker = remaining, attempt["task_id"]
        return longest, blocker

    def started(self, host, task, booked, snapshot):
        """这一轮在这台起了一条：数进在跑的；整片清零那台的组，碎片记进这一组（这一组再放它就超过界、又没整片在等的，开下一组从它算起）。"""
        self.occupied[host] = self.occupied.get(host, 0) + 1
        for kind in (self.wholes, self.fragments):
            for ids in kind.values():
                ids.discard(task["id"])
        threshold = self.scheduler.whole_threshold(host, snapshot)
        group = self.group(host)
        if booked >= threshold:
            self.groups[host] = {"booked": 0, "sealed": False}
        elif group["sealed"]:
            group["booked"] += int(booked)   # 封着组时起的（满时限的不归交替管）：照记，组照封着
        elif group["booked"] + booked > threshold:
            self.groups[host] = {"booked": int(booked), "sealed": False}
        else:
            group["booked"] += int(booked)


@dataclass
class Placement:
    kind: str                  # start、blocked、never、unmet
    host: str = ""
    cap: int = 0
    booked: int = 0
    gpu_booked: int = 0
    disk_booked: int = 0       # 编译目录占盘的记账（字节；文件头 ②「编译目录占盘」）
    card: str = ""
    history: str = ""
    waiting_for_card: bool = False
    why: str = ""
    gap_fill: str = ""         # 起的这一条是封着组插空进去的：插空那一句（Alternation.gap_fill），起的时候记进尝试


HEAD_STARTED = "started"                      # 派活这一轮判它能起，起了
HEAD_NOT_STARTED = "startable-not-started"    # 派活这一轮判它能起，却没起成（why 写为什么）：/health 只报连着几轮这样的
HEAD_BLOCKED = "not-startable"                # 派活这一轮判它不能起（why 写在等什么，与排队原因同一句）或判不了


@dataclass
class HeadVerdict:
    """派活这一轮给一条排着的任务的结论（Scheduler.place_queued 记，排头那一条换进 head_verdict；文件头「接口」的 /health）。"""
    task_id: int
    kind: str                  # HEAD_STARTED、HEAD_NOT_STARTED、HEAD_BLOCKED
    why: str = ""


@dataclass
class SuspectWindow:
    """判疑似卡住的一个 CPU 窗口（Scheduler.check_suspect）：时刻是 time.monotonic() 的钟。"""
    started_at: float
    started_cpu: int
    last_at: float
    last_cpu: int
    card_busy_seen: bool      # 这个窗口里有没有读到过它那张卡上有它的计算负载（量不到也算有）
    suspected: bool           # 这一段已报疑似卡住、还没结束


def descendant_processes(root_process_id):
    """root 的子孙进程号（照 /proc/<号>/stat 的父进程号一层层往下，不按名字找；读不到的那几个跳过）。"""
    children = {}
    for name in os.listdir("/proc"):
        if not name.isdigit():
            continue
        try:
            with open(f"/proc/{name}/stat", encoding="utf-8", errors="replace") as handle:
                parent = int(handle.read().rsplit(")", 1)[1].split()[1])
        except (OSError, IndexError, ValueError):
            continue
        children.setdefault(parent, []).append(int(name))
    found, frontier = set(), [root_process_id]
    while frontier:
        for child in children.get(frontier.pop(), []):
            if child not in found:
                found.add(child)
                frontier.append(child)
    return found


def process_is_zombie(process_id):
    """这个进程号已退出、只剩等父进程收的僵尸（/proc/<号>/stat 的状态是 Z）；读不到交 False（调用方另看 /proc/<号> 在不在）。"""
    try:
        with open(f"/proc/{process_id}/stat", encoding="utf-8", errors="replace") as handle:
            return handle.read().rsplit(")", 1)[1].split()[0] == "Z"
    except (OSError, IndexError):
        return False


# ── GPU 的认法 ──
def cargo_feature_gpu(words):
    """命令词是 cargo、`--` 之前的特性参数里有开了就上卡的特性（GPU_FEATURE_NAMES、<包>/<名字>，--all-features 也算；与 mutation_gate_uses_gpu 同一个判法）；不看子命令。
    弄坏开关 gpu-feature-names-unread 下照改前只认 gpu 与 <包>/gpu。"""
    if os.path.basename(words[0]) != "cargo":
        return False
    arguments = words[1:words.index("--")] if "--" in words else words[1:]
    values = []
    for index, word in enumerate(arguments):
        if word == "--all-features":
            return True
        if word in ("--features", "-F") and index + 1 < len(arguments):
            values.append(arguments[index + 1])
        elif word.startswith("--features="):
            values.append(word.split("=", 1)[1])
        elif word.startswith("-F") and len(word) > 2:
            values.append(word[2:])
    names = ("gpu",) if lib.broken("gpu-feature-names-unread") else GPU_FEATURE_NAMES
    return any(feature.split("/")[-1] in names for value in values for feature in re.split(r"[\s,]+", value) if feature)


def cargo_uses_gpu(words, project_code_runs, directory):
    """带 gpu 特性的 cargo 要不要卡（文件头 ②「GPU」）：子命令跑编出来的代码（GPU_RUNNING_CARGO_SUBCOMMANDS）、`--` 之前不带 --no-run 才要；
    clippy、check、build、doc 这类只编不跑的与 test / bench --no-run 不要卡、不清场。弄坏开关 gpu-feature-any-subcommand 下照改前不看子命令，
    no-run-needs-card 下带 --no-run 的照要卡。"""
    if not cargo_feature_gpu(words):
        return False
    if lib.broken("gpu-feature-any-subcommand"):
        return True
    cargo = project_code_runs.cargo_subcommand(words[1:], directory or lib.REPOSITORY_ROOT)
    if cargo is None or cargo.subcommand not in GPU_RUNNING_CARGO_SUBCOMMANDS:
        return False
    options = cargo.rest[:cargo.rest.index("--")] if "--" in cargo.rest else cargo.rest
    return NO_RUN_ARGUMENT not in options or lib.broken("no-run-needs-card")


def prove_red_uses_gpu(command):
    """prove-red.sh [--copy <副本>] [选项…] <crate> <变异名…>：点名的变异行里 cargo test 的参数（变异表第 5 段）带 gpu 特性就要卡。
    变异表取副本里的 crates/mutations.tsv，没给副本取起它的目录往上那棵仓的。"""
    if lib.broken("prove-red-gpu-unread"):
        return False
    words = command.words[1:]
    root = words[words.index("--copy") + 1] if "--copy" in words[:-1] else lib.REPOSITORY_ROOT
    root = shell_path(command.directory, root)
    names = {word for word in words if not word.startswith("-")}
    try:
        with open(os.path.join(root, "crates", "mutations.tsv"), encoding="utf-8") as handle:
            rows = [line.rstrip("\n").split("\t") for line in handle if line.strip() and not line.startswith("#")]
    except OSError:
        return False
    for row in rows:
        if len(row) >= 5 and row[0] in names:
            arguments = row[4].split()
            features = [arguments[index + 1] for index, word in enumerate(arguments[:-1]) if word == "--features"]
            features += [word.split("=", 1)[1] for word in arguments if word.startswith("--features=")]
            if any("gpu" in feature.split(",") for feature in features):
                return True
    return False


def mutation_gate_uses_gpu(command):
    """crates-mutation-replay.sh [--item <行>…] <根>：被判那棵树的变异表 crates/mutations.tsv 里有一行 cargo test 的参数（第 5 段）
    开了上卡的特性（GPU_FEATURE_NAMES、<包>/<名字>、--all-features）就要卡：整条门禁由服务套一次清场、分一张卡，门禁里的格在它的清场之下不再各自套
    （文件头 ②「GPU」）。根取参数里第一个是目录的词，没有取命令的目录。弄坏开关 mutation-gate-gpu-unread：不认（改前的样子：每格各自清场）。"""
    if lib.broken("mutation-gate-gpu-unread"):
        return False
    words = command.words[1:]
    root = next((shell_path(command.directory, word) for word in words if not word.startswith("-") and os.path.isdir(shell_path(command.directory, word))),
                command.directory or lib.REPOSITORY_ROOT)
    try:
        with open(os.path.join(root, MUTATION_TABLE_IN_TREE), encoding="utf-8") as handle:
            rows = [line.rstrip("\n").split("\t") for line in handle if line.strip() and not line.startswith("#")]
    except OSError:
        return False
    for row in rows:
        if len(row) < 5:
            continue
        arguments = row[4].split()
        if "--all-features" in arguments:
            return True
        values = [arguments[index + 1] for index, word in enumerate(arguments[:-1]) if word in ("--features", "-F")]
        values += [word.split("=", 1)[1] for word in arguments if word.startswith("--features=")]
        values += [word[2:] for word in arguments if word.startswith("-F") and len(word) > 2]
        if any(feature.split("/")[-1] in GPU_FEATURE_NAMES for value in values for feature in re.split(r"[\s,]+", value) if feature):
            return True
    return False


def shell_path(directory, path):
    return path if os.path.isabs(path) else os.path.normpath(os.path.join(directory or lib.REPOSITORY_ROOT, path))


def script_uses_gpu(command):
    name = os.path.basename(command.words[0])
    if name == "prove-red.sh":
        return prove_red_uses_gpu(command)
    if command.launcher == "source" or command.words[1:] == ["--selftest"] or any(word in GPU_NOT_RUN_ARGUMENTS for word in command.words[1:]):
        return False
    if name == MUTATION_GATE_SCRIPT_NAME:
        return mutation_gate_uses_gpu(command)
    if name in GPU_ENTRY_SCRIPT_NAMES and not lib.broken("gpu-entry-scripts-unrecognized"):
        return True
    return name in GPU_SCRIPT_NAMES or any(fnmatch.fnmatchcase(name, pattern) for pattern in GPU_SCRIPT_PATTERNS)


def declared_gpu_run(command):
    """命令位置上起的脚本（bash / sh 起的、python3 起的、直接执行的）文件头前 task_server_lib.DECLARED_HEADER_LINES 行里写了
    「# task-server-gpu-arguments: <词>…」、这一条的第一个非选项参数是其中一个（* 是任何参数）就要卡（task_server_lib.py 文件头「GPU」）；
    source 读进来的、单独 --selftest 的、带 --list-items / --help / -h 的不算；文件不在、没写那一行照不要卡。"""
    if command.launcher == "source" or command.words[1:] == ["--selftest"] or any(word in GPU_NOT_RUN_ARGUMENTS for word in command.words[1:]):
        return False
    path = shell_path(command.directory, command.words[0])
    return lib.declared_gpu_arguments(path, command.words[1:]) or lib.declared_gpu_binary(path, command.words[1:])


def python_entry_uses_gpu(words):
    """走 python 入口、按参数认的 GPU 跑法（PYTHON_GPU_ENTRIES；文件头 ②「GPU」）：python3 <入口> <子命令> … 或直接起的 <入口> <子命令> …，
    子命令对得上、`--` 之前的特性参数（--feature <值> 或 --feature=<值>）里有一个值带 gpu 这个词（按非字母数字切开之后有一段是 gpu，例 e174-gpu）。
    交入口的文件名或空串。弄坏开关 python-entry-gpu-unrecognized 下一律空串。"""
    if lib.broken("python-entry-gpu-unrecognized"):
        return ""
    rest = words[1:] if words and os.path.basename(words[0]) in PYTHON_INTERPRETERS else words
    while rest and rest[0].startswith("-"):   # python3 -B、-u 这类解释器自己的选项
        rest = rest[1:]
    if not rest or os.path.basename(rest[0]) not in PYTHON_GPU_ENTRIES:
        return ""
    name = os.path.basename(rest[0])
    subcommand, feature_argument = PYTHON_GPU_ENTRIES[name]
    arguments = rest[1:rest.index("--")] if "--" in rest else rest[1:]
    if not arguments or arguments[0] != subcommand:
        return ""
    values = [arguments[index + 1] for index, word in enumerate(arguments[:-1]) if word == feature_argument]
    values += [word.split("=", 1)[1] for word in arguments if word.startswith(feature_argument + "=")]
    return name if any("gpu" in re.split(r"[^0-9A-Za-z]+", value) for value in values) else ""


def recognize_gpu(project_code_runs, text, directory, environment=None, depth=0):
    """→ 命令文本里第一条要卡的为什么（一句话），没有交 None；认出要卡的一律由服务在最外层套清场（文件头 ③「起」）。"""
    shell_words = project_code_runs.shell_words
    for command in shell_words.commands_at_command_position(text, directory, environment).commands:
        entry = python_entry_uses_gpu(command.words)
        if entry:
            return f"带 gpu 特性的 python 入口 {entry}（{shlex.join(command.words)[:120]}）"
        launched = project_code_runs.command_under_launcher(command.words, command.directory)
        if launched is not None:
            if depth < MAXIMUM_LAUNCHER_DEPTH:
                inner_words, inner_directory = launched
                inner_start = shell_words.resolve_path(command.directory, inner_directory) if inner_directory else command.directory
                found = recognize_gpu(project_code_runs, shlex.join(inner_words), inner_start, command.environment, depth + 1)
                if found:
                    return found
            continue
        if cargo_uses_gpu(command.words, project_code_runs, command.directory):
            return f"带 gpu 特性、跑编出来的代码的 cargo（{shlex.join(command.words)[:120]}）"
        if declared_gpu_run(command):
            return f"文件头声明了要卡的参数的脚本 {os.path.basename(command.words[0])}（{shlex.join(command.words)[:120]}）"
        if script_uses_gpu(command):
            return f"{NAMED_GPU_REASON} {os.path.basename(command.words[0])}"
    return None


# ── 单元 ──
def unit_states(unit, environment):
    """systemctl --user show 读的 (LoadState, ActiveState)（照输出判，不看退出码）；查不了交 None。"""
    try:
        completed = subprocess.run(["systemctl", "--user", "show", "-p", "LoadState", "-p", "ActiveState", unit], capture_output=True, text=True,
                                   env=environment, stdin=subprocess.DEVNULL, timeout=30)
    except (OSError, subprocess.TimeoutExpired):
        return None
    values = dict(line.split("=", 1) for line in completed.stdout.splitlines() if "=" in line)
    if "ActiveState" not in values:
        return None
    return values.get("LoadState", ""), values["ActiveState"]


def unit_processes(unit, environment):
    """单元 cgroup 里此刻的进程号（读 cgroup.procs 的内容）；单元已经没了交空表。"""
    try:
        completed = subprocess.run(["systemctl", "--user", "show", "-p", "ControlGroup", "--value", unit], capture_output=True, text=True,
                                   env=environment, stdin=subprocess.DEVNULL, timeout=30)
    except (OSError, subprocess.TimeoutExpired):
        return None
    if completed.returncode:
        states = unit_states(unit, environment)
        return [] if states is not None and states[0] == "not-found" else None
    group = completed.stdout.strip()
    if not group:
        return []
    return cgroup_processes("/sys/fs/cgroup" + group)


def cgroup_processes(group_path):
    """一个 cgroup 目录此刻的进程号（读 cgroup.procs）：目录已经没了交空表，读不了交 None。"""
    try:
        with open(os.path.join(group_path, "cgroup.procs"), encoding="utf-8") as handle:
            return handle.read().split()
    except FileNotFoundError:
        return None if os.path.isdir(group_path) else []
    except OSError:
        return None


def registered_threads(environment):
    """每台默认重活线程数（task_server_lib.py 文件头「线程」）；取不到抛 Unavailable（不退回按核数算）。"""
    heavy_threads = lib.load_module("heavy_threads", os.path.join(lib.HOSTS_DIRECTORY, "heavy_threads.py"))
    try:
        return int(heavy_threads.default_heavy_threads(environment))
    except (heavy_threads.HeavyThreadDefaultError, ValueError) as error:
        raise lib.Unavailable(f"取不到每台默认重活线程数（{error}）", "修好 .claude/heavy-thread-default（或 unset 写坏的 SINGLEFS_THREAD_CAP）再起服务") from None


class Scheduler:
    def __init__(self, configuration, store, environment, threads, nfs_report):
        self.configuration = configuration
        self.store = store
        self.environment = environment
        self.threads = threads
        self.host_dispatch = lib.load_module("host_dispatch", os.path.join(lib.HOSTS_DIRECTORY, "host-dispatch.py"))
        self.peaks = lib.load_module("memory_peaks_lib", os.path.join(lib.HOSTS_DIRECTORY, "memory_peaks_lib.py"))
        self.project_code_runs = lib.load_module("lib_project_code_runs", os.path.join(lib.HOOKS_DIRECTORY, "lib_project_code_runs.py"))
        self.cargo_lock = lib.load_module("lib_workspace_cargo_lock", os.path.join(lib.HOOKS_DIRECTORY, "lib_workspace_cargo_lock.py"))
        self.build_directories = lib.load_module("run_build_directory", os.path.join(lib.HOSTS_DIRECTORY, "run_build_directory.py"))
        self.resources = Resources(environment, self.host_dispatch, configuration.build_directory, configuration.path)
        self.machines = {LOCAL: LocalMachine(self), PEER: PeerMachine(self)}   # 两台各一份「这一台怎么读、怎么起」（文件头 ②「两台同一段逻辑」）
        self.resources.machines = self.machines
        speedy = lib.fast(environment)
        self.tick_seconds = FAST_TICK_SECONDS if speedy else TICK_SECONDS
        override = lib.selftest_override("TASK_SERVER_TICK_SECONDS", environment)   # 自证量「30 秒一轮也不慢」那一格用
        if override:
            self.tick_seconds = float(override)
        self.finished_listeners = []   # 记下结局时叫醒在等的 wait（文件头「通知」）
        self.unit_check_seconds = FAST_UNIT_CHECK_SECONDS if speedy else UNIT_CHECK_SECONDS
        self.lost_grace_seconds = FAST_LOST_GRACE_SECONDS if speedy else LOST_GRACE_SECONDS
        self.cancel_settle_seconds = self.lost_grace_seconds + 2 * self.unit_check_seconds + CANCEL_SETTLE_MARGIN_SECONDS
        self.alerted_child_refusals = set()   # children_refused 已告警过的 (父任务号, 那一句)
        self.start_uncertain_grace_seconds = FAST_START_UNCERTAIN_GRACE_SECONDS if speedy else START_UNCERTAIN_GRACE_SECONDS
        self.scene_keep_seconds = FAST_SCENE_KEEP_AFTER_COLLECT_SECONDS if speedy else SCENE_KEEP_AFTER_COLLECT_SECONDS
        self.build_disk_predictions = {}   # {任务号: 编译目录占盘的预测（字节）}：与内存预测一样每条在排任务只算一次（文件头 ②「编译目录占盘」）
        self.starved_seconds = FAST_STARVED_SECONDS if speedy else STARVED_ROUNDS * self.tick_seconds
        self.aged_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_AGED_SECONDS", environment) or AGED_WAIT_SECONDS)
        self.placement_paused = lib.selftest_override("TASK_SERVER_SELFTEST_PLACEMENT_PAUSED", environment)
        self.placement_refused = lib.selftest_override("TASK_SERVER_SELFTEST_PLACEMENT_REFUSED", environment)
        self.progress_seconds = FAST_PROGRESS_STALL_SECONDS if speedy else PROGRESS_STALL_SECONDS
        self.stall_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_STALL_SECONDS", environment) or STALL_SECONDS)
        self.stall_grace_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS", environment) or STALL_GRACE_SECONDS)
        self.log_growth = {}   # {尝试号: (日志字节数, 上一次看到它变的时刻, 这一个 CPU 窗口开头的时刻, 那时的 CPU, 窗口是不是因 CPU 在涨或读不到而接着开的)}（判卡住）
        self.log_offsets = {}   # 第二台的尝试：{尝试号: (日志读到哪, 算在涨的字节数, 最近的 usage_usec)}
        self.cpu_windows = {}   # 判疑似卡住：{尝试号: SuspectWindow}（文件头 ①「疑似卡住」）
        self.suspect_reported = {}   # {尝试号: 报给提交它的会话的那一段的段号}：一段只报一次
        self.suspect_lock = threading.Lock()
        # 要处置的提醒（文件头 ①「要处置的提醒」）：阈值的头一档；报过的档在内存里（同 suspect_reported，服务重启后再报一次）；分片父任务推不动的现状由 drive_sharded_parents 记
        self.queue_attention_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS", environment) or QUEUE_ATTENTION_MINUTES * 60)
        self.overrun_attention_factor = float(lib.selftest_override("TASK_SERVER_SELFTEST_OVERRUN_ATTENTION_FACTOR", environment) or OVERRUN_ATTENTION_FACTOR)
        self.attention_reported = {}   # {(任务号, 种类): 报给提交它的会话的最高档}：同一种同一档只报一次
        self.workspace_write_checks = {}   # {任务号: workspace-write-interrupted 那一项或 None}：结了局的改工作区命令只核一次（workspace_write_item）
        self.attention_lock = threading.Lock()
        self.advance_failures = {}   # {分片父任务号: {"episode": 第几回开始推不动, "why": 最近一次的错, "count": 这一回连着推不动几轮}}：推通了就删
        self.advance_episodes = {}   # {分片父任务号: 推不动开始过几回}：推通之后再推不动是新的一回（提醒的档加一）
        self.nvidia_smi = lib.selftest_override("TASK_SERVER_NVIDIA_SMI", environment) or "nvidia-smi"
        self.pass_state = None   # 这一轮排队留下的（看进展读它的余量；弄坏开关下另判排头也复用：在排的、起了或收尾了的号、这一轮的余量）
        self.head_verdict = None   # 派活这一轮给排头的结论（HeadVerdict；队空、这一轮没排是 None）：/health 判「判能起却没起成」只读它
        self.starved_text = ""   # 排队线程每一轮算好的「派活判排头能起却一直没起成」（/health 只读它，文件头「接口」）
        self.progress_text = ""   # 排队线程每一轮算好的看进展那一句（progress_problem；/health 只读它）
        self.progress_since = {}   # 看进展：{台: 这台从哪一刻起照公式放得下排着的最小那条、又一条都没起}
        self.progress_attempt_seen = None   # 看进展：上一轮数到的最大尝试号（新起的尝试照号认，起了又收尾的也数得到）
        self.pause = store.pause_record()   # 暂停态（文件头 ⑤「暂停」）：库里记的那一份，起来时读；排队线程与接口读这一份（None 是没暂停），pause_service、resume_service 改库同时改它
        self.start_gate = threading.Lock()  # 起尝试与 pause_service 共用：持锁判没暂停才起，pause 返回之后一条新的都不起
        self.kept_allow_session = ""   # 只在弄坏开关 resume-keeps-allowed-session 下用：上一次 resume 时放行的会话
        self.predictions = {}   # {任务号: (算它时的 next_cap, 预测)}：每条在排任务的内存预测只算一次（文件头 ②「预测每条只算一次」）
        self.pin_unreadable_reasons = {}   # {任务号: 钉值读不到的那一句}：pinned_prediction 记、place 照它记 run-condition-unmet（文件头 ②「内存」的钉值）
        self.fragment_groups = {}   # {台: {"booked": 字节, "sealed": 封了没有}}：每台这一组碎片的账（文件头 ②「整片与碎片按台交替」；Alternation）
        self.whole_hosts = {}   # {任务号: 台}：没满时限的整片在哪一台上等腾空（一条只一台；Alternation.waiting_hosts），跨轮留着、不在队里了就删
        self.prediction_lock = threading.Lock()   # 排队线程与弄坏开关 health-judges-head 下 /health 的另判排头都取预测：同一条不算两遍
        self.view_numbers = itertools.count(1)   # PassView 的编号（自证的读数记录里分各轮）
        world = lib.selftest_override("TASK_SERVER_SELFTEST_WORLD", environment)
        self.reads_log = os.path.join(world, "scheduler-reads") if world else ""   # 只在自证里：排队读预测与余量的每一次记一行（selftest_read）
        self.reads_lock = threading.Lock()
        self.repass_soon = False   # 第二台这一轮起满了 PEER_STARTS_PER_PASS 条：主循环隔 PEER_REPASS_SECONDS 再排
        self.waiting_reasons = {}   # 上一轮判的每条排着的任务在等什么：{任务号: 一句}（status 打出来）
        self.fits_since = {}   # 派活从哪一刻起一直判排头能起却没起成：{任务号: 时刻}（/health 判「判能起却没起成」）
        self.memory_wrapper = lib.selftest_override("TASK_SERVER_MEMORY_WRAPPER", environment) or lib.MEMORY_WRAPPER
        self.peer_runner = lib.selftest_override("TASK_SERVER_PEER_RUNNER", environment) or lib.PEER_RUNNER
        self.gpu_quiesce = lib.selftest_override("TASK_SERVER_GPU_QUIESCE", environment) or lib.GPU_QUIESCE
        self.systemd_run = lib.selftest_override("TASK_SERVER_SYSTEMD_RUN", environment) or "systemd-run"
        self.peer_root = lib.selftest_override("TASK_SERVER_FAKE_REPOSITORY_ROOT", environment) or lib.REPOSITORY_ROOT
        # 工作根与共享数据根（task_server_lib.py 文件头「资源记录」后的「共享根」）：照 .claude/hooks/lib_work_root.py 取（不另算、不补缺省），交给每次尝试；
        # serve 在建 Scheduler 之前已判过取得到（取不到退 78）
        self.work_root = lib.work_root()
        self.scenes_root = os.path.join(self.work_root, lib.SCENES_ROOT_NAME)   # 判红那一趟的现场（task_server_lib.py 文件头「现场」；serve 起来时建好）
        self.data_root = lib.shared_data_root(environment)
        self.boundary = int(lib.selftest_override("TASK_SERVER_SESSION_BOUNDARY_PID", environment) or 1)
        self.wake_event = threading.Event()
        self.stopping = threading.Event()
        self.last_tick = time.time()
        self.unit_checked_at = {}
        self.unit_gone_since = {}
        self.children = []
        self.thread = threading.Thread(target=self.loop, name="task-server-scheduler", daemon=True)
        # NFS 共享（task_server_lib.py 文件头「NFS 共享」）：起来时判过的那一份，前三项由 nfs_loop 每 nfs_recheck_seconds 复核；/health 的 nfs 一栏读它
        self.nfs = nfs_report
        self.nfs_lock = threading.Lock()
        self.shared_roots = [os.path.realpath(entry["root"]) for entry in nfs_report.get("roots", [])]   # 第二台照同一绝对路径读写的目录（提交时拒的 ① ②）
        self.mount_problem = ""   # 复核判出第二台没挂上共享根的那一句：这时当第二台此刻用不了（peer_view）
        self.nfs_recheck_seconds = FAST_NFS_RECHECK_SECONDS if speedy else NFS_RECHECK_SECONDS
        self.nfs_thread = threading.Thread(target=self.nfs_loop, name="task-server-nfs-recheck", daemon=True)
        # 第二台用不用得了（note_peer）：/health 的 peer 一栏读它；peer_alerted 是上一条 peer-unavailable 告警写的原因（同一个原因不再写）
        self.peer_status = {"available": True, "why": "", "since": None, "last_run_failure": None}
        self.peer_alerted = ""
        self.peer_status_lock = threading.Lock()
        self.quarantined_cards = {}   # {"<台>:<序号>"（清场那一行坏卡清单里的一项，attempt 的 faulty_cards）: 停用起点}：清场判出坏的卡，CARD_QUARANTINE_SECONDS 秒里不派（cards_in_service）

    # ── 记录 ──
    def operation(self, action, task_id, session, process_id=0, note=""):
        if lib.broken("operations-unlogged", self.environment):
            return
        record = {"time": round(time.time(), 3), "action": action, "task": task_id, "session": session, "pid": process_id, "note": note}
        self.store.insert("operations", dict(record))
        with contextlib.suppress(OSError):
            append_private_line(self.configuration.operations_log, lib.canonical_json(record))

    def alert(self, kind, task_id, line, next_step):
        record = {"time": round(time.time(), 3), "kind": kind, "task": task_id, "line": line, "next_step": next_step}
        self.store.insert("alerts", dict(record))
        with contextlib.suppress(OSError):
            append_private_line(self.configuration.alerts_log, lib.canonical_json(record))
        say(f"  ✗ 告警（{kind}）任务 {task_id}：{line}")
        say(f"     → 怎么办：{next_step}")

    def tampered(self, task_id, record, what):
        """校验不过（文件头「库」）：不起、不认，任务记 record-tampered 收尾，写告警。"""
        if not lib.broken("tamper-unalerted", self.environment):
            self.alert("record-tampered", task_id, f"库里任务 {task_id} 的{what}校验不过（绕过服务直接写进库，或写进之后被改过），不起、不认",
                       "查是谁直接写了库（只有 task-server 读写它）；任务照 task-server/task submit 重新提交")
        record = dict(record, id=task_id, state="finished", outcome="record-tampered", exit_code=lib.final_exit_code("record-tampered", None),
                      finished_at=round(time.time(), 3), note=f"{what}校验不过，一次都不起")
        for name in ("session", "command_key"):
            record[name] = record.get(name) if isinstance(record.get(name), str) else ""
        self.store.update("tasks", record)
        self.forget_prediction(task_id)
        if isinstance(record.get("parent_id"), int) and record["parent_id"] != task_id:
            self.advance_parent(record)

    def selftest_read(self, line):
        """只在自证里：排队读预测与余量的每一次往 TASK_SERVER_SELFTEST_WORLD/scheduler-reads 记一行（task_server_lib.py 文件头「只在自证里认的变量」）。"""
        if not self.reads_log:
            return
        with self.reads_lock, open(self.reads_log, "a", encoding="utf-8") as handle:
            handle.write(line + "\n")

    # ── 暂停（文件头 ⑤「暂停」）──
    def pause_in_force(self):
        """排队线程此刻要照着不起新尝试的暂停态（None 是不用）；弄坏开关 pause-queue-ignored 下排队线程当没暂停。"""
        return None if lib.broken("pause-queue-ignored", self.environment) else self.pause

    def allowed_during_pause(self, task, pause):
        """暂停着，这条任务是不是放行的会话交的（文件头 ⑤「暂停」的放行一段）：放行的会话是空串谁都不是，会话是空串的（人手任务）也不匹配。
        弄坏开关 pause-allow-ignored 下谁都不放行，pause-allow-matches-any-session 下放行了会话就谁都放行。"""
        allowed = pause.get("allow_session", "")
        if lib.broken("pause-allow-ignored", self.environment):
            return False
        if allowed and lib.broken("pause-allow-matches-any-session", self.environment):
            return True
        return bool(allowed) and task.get("session", "") == allowed

    def pause_holds(self, task):
        """这条任务此刻要不要留在队里不起：排队线程要照着暂停、而它又不是放行的会话交的 → 那份暂停态，不然 None。"""
        pause = self.pause_in_force()
        if pause is None or self.allowed_during_pause(task, pause):
            return None
        return pause

    def running_outside_allowed(self, pause, running):
        """暂停着在跑的任务里不是放行会话的有几个（等它为 0 才做空档的活）；没放行谁，在跑的都是别的会话的。弄坏开关 pause-other-running-miscounted 下把放行会话的也数进去。"""
        allowed = pause.get("allow_session", "")
        if not allowed or lib.broken("pause-other-running-miscounted", self.environment):
            return running
        return self.store.count_outside_session("tasks", "running", allowed)

    def pause_text(self, pause, running, other_running):
        """暂停态的一句话（/health 的 pause.text，control pause / check 照打）：暂停中（原因：…，起于…）、在跑 N 个；放行了会话的另写放行会话与别的会话在跑几个。
        弄坏开关 pause-allow-unreported 下不写放行的会话。"""
        started = time.strftime("%Y-%m-%d %H:%M:%S", time.localtime(pause["since"])) if pause["since"] else "不知道什么时候"
        text = f"暂停中（原因：{pause['reason'] or '没写'}，起于 {started}）"
        allowed = pause.get("allow_session", "")
        if allowed and not lib.broken("pause-allow-unreported", self.environment):
            return f"{text}、放行会话 {allowed}、在跑 {running} 个、别的会话在跑 {other_running} 个"
        return f"{text}、在跑 {running} 个"

    def pause_reply(self, changed, pause, allow_before="", allow_changed=False):
        """接口 POST /pause、/resume 的回话：pause 是这一回涉及的那一份暂停态（没有交 None）；running 是此刻 state 为 running 的任务数，
        other_running 是其中不是放行会话的；allow_session 是这一份里放行的会话，allow_before 是换之前的，allow_changed 是这一回换了没有。"""
        running = self.store.count("tasks", "running")
        current = self.pause
        return {"paused": current is not None, "changed": changed, "reason": pause["reason"] if pause else "", "since": pause["since"] if pause else None,
                "running": running, "other_running": self.running_outside_allowed(current, running) if current is not None else running,
                "allow_session": pause.get("allow_session", "") if pause else "", "allow_before": allow_before, "allow_changed": allow_changed}

    def pause_service(self, reason, allow_session, session, process_id):
        """暂停（接口 POST /pause）：拿 start_gate 改库里的暂停态、再改排队线程读的那一份，之后一条新尝试都不起（放行的会话交的除外）；
        已经暂停着的不改原因与起点，changed 为 false，给了不同的 allow_session 就换放行的会话（allow_changed 为 true），不给的不动放行。
        暂停态校验不过的（valid 为 False）不换放行。弄坏开关 pause-start-ungated 下不拿锁，pause-state-memory-only 下不写库，
        pause-allow-change-ignored 下已暂停着不换放行的会话，resume-keeps-allowed-session 下没给 allow_session 的沿用上一次 resume 时的放行。"""
        gate = contextlib.nullcontext() if lib.broken("pause-start-ungated", self.environment) else self.start_gate
        allow_before, allow_changed = "", False
        with gate:
            if self.pause is not None:
                allow_before = self.pause["allow_session"]
                if (not allow_session or allow_session == allow_before or not self.pause["valid"]
                        or lib.broken("pause-allow-change-ignored", self.environment)):
                    return self.pause_reply(False, self.pause, allow_before)
                self.store.write_pause(self.pause["reason"], self.pause["since"], allow_session)
                self.pause = dict(self.pause, allow_session=allow_session)
                allow_changed = True
            else:
                if not allow_session and lib.broken("resume-keeps-allowed-session", self.environment):
                    allow_session = self.kept_allow_session
                since = round(time.time(), 3)
                if not lib.broken("pause-state-memory-only", self.environment):
                    self.store.write_pause(reason, since, allow_session)
                self.pause = {"reason": reason, "since": since, "allow_session": allow_session, "valid": True}
        if allow_changed:
            self.operation("pause", 0, session, process_id, f"换放行会话 {allow_before or '没有'} → {allow_session}")
        else:
            self.operation("pause", 0, session, process_id, reason + (f"（放行会话 {allow_session}）" if allow_session else ""))
        self.wake()
        return self.pause_reply(not allow_changed, self.pause, allow_before, allow_changed)

    def resume_service(self, session, process_id):
        """放开暂停（接口 POST /resume）：清库里的暂停态（放行的会话在里面，一并没了）、再清排队线程读的那一份，叫醒排队线程；本来没暂停的 changed 为 false。
        弄坏开关 resume-leaves-cache-paused 下只清库，resume-keeps-allowed-session 下记下放行的会话、下一次不带 --allow-session 的 pause 照它放行。"""
        with self.start_gate:
            was = self.pause
            if was is None:
                return self.pause_reply(False, None)
            self.store.clear_pause()
            if not lib.broken("resume-leaves-cache-paused", self.environment):
                self.pause = None
            if lib.broken("resume-keeps-allowed-session", self.environment):
                self.kept_allow_session = was["allow_session"]
        self.operation("resume", 0, session, process_id, was["reason"] + (f"（放行会话 {was['allow_session']}）" if was["allow_session"] else ""))
        self.wake()
        return self.pause_reply(True, was)

    def waiting_text(self, task):
        """这条任务在等什么：排着而服务暂停着，是暂停的原因（放行会话交的除外，它写自己真在等什么）；不然是上一轮判的排队原因（status、列表与 wait 的排队过久提醒读它）。
        弄坏开关 pause-waiting-unreported 下不写暂停，pause-allow-waiting-paused 下放行会话排着的也写暂停。"""
        pause = self.pause
        if (pause is not None and task.get("state") == "queued" and not lib.broken("pause-waiting-unreported", self.environment)
                and (lib.broken("pause-allow-waiting-paused", self.environment) or not self.allowed_during_pause(task, pause))):
            return f"服务暂停中（原因：{pause['reason'] or '没写'}）"
        return self.waiting_reasons.get(task["id"], "")

    def start_attempt_unless_paused(self, task, placement):
        """起这一次尝试，除非此刻暂停着而它不是放行的会话交的 → (尝试, 因暂停没起)。拿 start_gate 判过没暂停才起，pause_service 同一把锁：pause 返回之后一条新的都不起，
        已经开始的在它返回之前都记下了。弄坏开关 pause-start-ungated 下判了不拿锁。"""
        if lib.broken("pause-start-ungated", self.environment):
            if self.pause_holds(task) is not None:
                return None, True
            return self.start_attempt(task, placement), False
        with self.start_gate:
            if self.pause_holds(task) is not None:
                return None, True
            return self.start_attempt(task, placement), False

    # ── 主循环 ──
    def wake(self):
        self.wake_event.set()

    def notify_finished(self, task_id):
        """记下结局之后叫醒在等这条任务的 wait（文件头「通知」）；弄坏开关 wait-notify-skipped 时不叫，wait 只能等兜底那一轮。"""
        if lib.broken("wait-notify-skipped", self.environment):
            return
        for listener in list(self.finished_listeners):
            with contextlib.suppress(Exception):
                listener(task_id)

    def loop(self):
        while not self.stopping.is_set():
            self.last_tick = time.time()
            try:
                if not (self.pause is not None and lib.broken("pause-halts-reconcile", self.environment)):   # 暂停着对账照做（在跑的照跑到结局）；弄坏开关下对账也停
                    self.reconcile()
                self.place_queued()
                self.sweep_scenes()   # 取走结局满一段时间的现场删掉（task_server_lib.py 文件头「现场」）
                if not lib.broken("health-judges-head", self.environment):
                    self.starved_text = self.starved_problem(time.time())   # /health 只读这一份（文件头「接口」）
                self.progress_text = self.progress_problem(time.time())   # 看进展：/health 同样只读这一份
            except Exception:   # 一轮出事不停服务：打出来、下一轮再判
                say("  ! 这一轮排队出了异常（下一轮再判）：\n" + traceback.format_exc())
            self.wake_event.wait(PEER_REPASS_SECONDS if self.repass_soon else self.tick_seconds)
            self.wake_event.clear()

    def stop(self):
        self.stopping.set()
        self.wake()
        if lib.broken("probe-children-left", self.environment):   # 弄坏开关：照改前只等排队线程、不管自己起的子进程
            self.thread.join(timeout=30)
        else:   # 文件头 ③「停服务」：两个线程合计至多等 STOP_JOIN_SECONDS，再停还在的子进程
            deadline = time.monotonic() + STOP_JOIN_SECONDS
            for thread in (self.thread, self.nfs_thread):
                if thread.is_alive():
                    thread.join(timeout=max(0.0, deadline - time.monotonic()))
        if lib.broken("no-detach", self.environment):
            for child in self.children:   # 弄坏开关 no-detach：起成服务的子进程的尝试随服务一起停（单元的 cgroup 被停时就是这样）
                with contextlib.suppress(ProcessLookupError):
                    child.send_signal(signal.SIGTERM)
        if not lib.broken("probe-children-left", self.environment):
            self.stop_own_children()

    def stop_own_children(self):
        """停服务时还在的、服务自己起的子进程（文件头 ③「停服务」）：照 /proc 的父进程号从自己往下找，逐个按进程号先 TERM、至多等 STOP_CHILD_GRACE_SECONDS 再 KILL；
        弄坏开关 no-detach 下起成子进程的尝试（self.children 与它们的子孙）不算在内。"""
        kept = set()
        for child in self.children:
            kept |= {child.pid} | descendant_processes(child.pid)
        leftover = descendant_processes(os.getpid()) - kept
        if not leftover:
            return
        say(f"  · 停服务：自己起的子进程还有 {len(leftover)} 个（{'、'.join(str(pid) for pid in sorted(leftover))}），逐个停")
        for signal_number in (signal.SIGTERM, signal.SIGKILL):
            for process_id in sorted(leftover):
                with contextlib.suppress(ProcessLookupError, PermissionError):
                    os.kill(process_id, signal_number)
            deadline = time.monotonic() + STOP_CHILD_GRACE_SECONDS
            while time.monotonic() < deadline:
                leftover = {pid for pid in leftover if os.path.exists(f"/proc/{pid}") and not process_is_zombie(pid)}
                if not leftover:
                    return
                time.sleep(0.05)

    # ── NFS 共享与第二台用不用得了（task_server_lib.py 文件头「NFS 共享」；本文件头 ②「机器」）──
    def nfs_loop(self):
        """每 nfs_recheck_seconds 复核一次 NFS 共享的前三项（两向探针只在起来时做，结论照留）：第二台没挂上的根记进 mount_problem，peer_view 当第二台此刻用不了；
        随后读一次第二台的余量（队空时排队线程不读），第二台用不了照 note_peer 告警。"""
        while not self.stopping.wait(self.nfs_recheck_seconds):
            try:
                report = lib.nfs_share_report(self.configuration.path, self.environment, probe=False)
            except Exception:   # 复核出事不停服务：打出来、下一回再判
                say("  ! 这一回 NFS 复核出了异常（下一回再判）：\n" + traceback.format_exc())
                continue
            with self.nfs_lock:
                probes = {entry["root"]: entry.get("probe") for entry in self.nfs.get("roots", [])}
            for entry in report["roots"]:
                entry["probe"] = probes.get(entry["root"], entry.get("probe"))
            unmounted = next((entry for entry in report["roots"] if entry.get("mount") != lib.NFS_PASSED), None)
            self.mount_problem = (f"第二台没挂上共享根 {unmounted['root']}（{unmounted['mount']}；{lib.NFS_README}）" if unmounted else "")
            with self.nfs_lock:
                self.nfs = dict(report, probe_checked_at=self.nfs.get("probe_checked_at", self.nfs.get("checked_at")))
            try:   # 队空时排队线程不读余量：第二台用不了也在这里认出来、告警（note_peer）
                self.peer_view(self.resources.snapshot(False))
            except Exception:
                say("  ! 这一回读第二台的余量出了异常（下一回再判）：\n" + traceback.format_exc())

    def nfs_snapshot(self):
        with self.nfs_lock:
            return json.loads(json.dumps(self.nfs))

    def peer_view(self, snapshot):
        """这一轮读的余量照 NFS 复核改：第二台没挂上共享根的当它此刻用不了；顺带记第二台用不用得了（note_peer）。"""
        if self.mount_problem and snapshot.reachable.get(PEER):   # 共享根由本机导出、第二台挂载（文件头「NFS 共享」）：挂载复核只判第二台
            snapshot = dataclasses.replace(snapshot, reachable=dict(snapshot.reachable, **{PEER: False}), down=dict(snapshot.down, **{PEER: self.mount_problem}),
                                           down_kind={host: kind for host, kind in snapshot.down_kind.items() if host != PEER},
                                           reports={host: report for host, report in snapshot.reports.items() if host != PEER})
        if snapshot.expected.get(PEER):
            available = bool(snapshot.reachable.get(PEER)) and PEER in snapshot.reports
            self.note_peer(available, "" if available else (snapshot.down.get(PEER) or "第二台的余量查不成"), kind="" if available else snapshot.down_kind.get(PEER, ""))
        return self.cards_in_service(self.unstarted_slices_filled(snapshot))

    def cards_in_service(self, snapshot):
        """这一轮能派的卡去掉停用着的（清场判出坏、还没满 CARD_QUARANTINE_SECONDS 秒的，quarantined_cards）；到点的从停用里拿掉、回到候选。"""
        now = time.time()
        for entry, since in list(self.quarantined_cards.items()):
            if now - since >= CARD_QUARANTINE_SECONDS:
                self.quarantined_cards.pop(entry, None)
        if not self.quarantined_cards:
            return snapshot
        # 按（台、序号）停：同一张卡在两份显卡先后里额度写得不同也一起停
        return dataclasses.replace(snapshot, cards=[card for card in snapshot.cards if f"{card.address}:{card.index}" not in self.quarantined_cards])

    def unstarted_slices_filled(self, snapshot):
        """slice 还没起的台（重启之后还没起过内存包装；报告里 memory_peaks_lib.SLICE_STARTED_KEY 为假，slice_total 是按整机内存估的）：总上限换成多机配置
        SLICE_MEMORY_CAP_BY_HOST 里那台那一项（包装第一次起就把 slice 设成它），slice 放得下照它减 slice 里已用的；两台同一段。配置读不到的照报告原样。
        弄坏开关 unstarted-slice-estimated 下照原样用估的。"""
        if lib.broken("unstarted-slice-estimated", self.environment):
            return snapshot
        unstarted = [host for host, report in snapshot.reports.items() if (report.get("memory") or {}).get(self.peaks.SLICE_STARTED_KEY) is False]
        if not unstarted:
            return snapshot
        caps, _ = self.configured_slice_caps()
        reports = dict(snapshot.reports)
        for host in unstarted:
            total = int((caps or {}).get(host) or 0)
            if total > 0:
                memory = dict(reports[host]["memory"], slice_total=total)
                memory["slice_room"] = total - int(memory.get("slice_used") or 0)
                reports[host] = dict(reports[host], memory=memory)
        return dataclasses.replace(snapshot, reports=reports)

    def note_peer(self, available, why, task_id=None, kind=""):
        """第二台用不用得了（本文件头 ②「机器」的告警）：从用得了变成用不了、或用不了的原因变了，写一条告警 peer-unavailable；/health 的 peer 一栏读 peer_status（why 照最新的写）。
        kind 给了的按它去重（盘余量不到下限：余量的数每轮在变，不每轮另写一条），没给的按原因的字面去重。弄坏开关 peer-unavailable-silent 下不记、不告警。"""
        if lib.broken("peer-unavailable-silent", self.environment):
            return
        with self.peer_status_lock:
            status = self.peer_status
            if available:
                if not status["available"]:
                    say(f"  · 第二台又用得了（之前：{status['why']}）")
                status.update(available=True, why="", since=None)
                self.peer_alerted = ""
                return
            if status["available"]:
                status.update(available=False, since=round(time.time(), 3))
            status["why"] = why
            if self.peer_alerted == (kind or why):
                return
            self.peer_alerted = kind or why
        self.alert("peer-unavailable", task_id, f"第二台此刻用不了：{why}；派得到第二台的任务这时只派本机",
                   f"照那一句修第二台（连法、多机配置的双机开关、NFS 挂载照 {lib.NFS_README}），修好之后服务下一轮自己认回；python3 task-server/control watch 盯着")

    def note_peer_run_failure(self, task, attempt):
        """那一趟没跑成（peer-run-failed：树建不成、ssh 被重置、那边的准备没过）：每一次写一条告警 peer-unavailable，/health 的 peer 一栏记最近一次。"""
        if lib.broken("peer-unavailable-silent", self.environment):
            return
        why = f"任务 {task['id']} 第 {attempt.get('number')} 次在第二台那一趟没跑成（multi-host-run.sh 退 255）：{lib.log_last_line(log_tail(attempt.get('log', '')))}"
        with self.peer_status_lock:
            self.peer_status["last_run_failure"] = {"time": round(time.time(), 3), "task": task["id"], "why": why}
        self.alert("peer-unavailable", task["id"], why, f"看 task-server/task log {task['id']}；连不上、树建不成照那一句修第二台，服务照重试的规矩重排")

    def peer_health(self):
        with self.peer_status_lock:
            return dict(self.peer_status)

    def check_prune_server(self, task, attempt):
        """命令退 75（prune-server 不可用，task_server_lib.py 文件头「错误类型」）：另起一个线程在仓根跑一次 python3 prune-server/control check
        （至多 PRUNE_SERVER_CHECK_SECONDS 秒），不通（退非 0、起不来、超时、那份文件不在）写一条告警 prune-server-unavailable；只报，不起、不停 prune-server。
        自证里 check 换成 TASK_SERVER_PRUNE_SERVER_CHECK 那一份可执行文件。"""
        fake = lib.selftest_override("TASK_SERVER_PRUNE_SERVER_CHECK", self.environment)
        command = [fake] if fake else [sys.executable, "-B", lib.PRUNE_SERVER_CONTROL, "check"]
        shown = "python3 prune-server/control check"

        def run():
            if not fake and not os.path.isfile(lib.PRUNE_SERVER_CONTROL):
                code, text = None, f"仓里没有 {lib.PRUNE_SERVER_CONTROL}"
            else:
                try:
                    completed = subprocess.run(command, cwd=lib.REPOSITORY_ROOT, capture_output=True, text=True, env=self.environment, stdin=subprocess.DEVNULL,
                                               timeout=PRUNE_SERVER_CHECK_SECONDS)
                    code, text = completed.returncode, completed.stderr + completed.stdout
                except (OSError, subprocess.TimeoutExpired) as error:
                    code, text = None, f"{type(error).__name__}: {error}"
            if code == 0:
                say(f"  · 任务 {task['id']} 第 {attempt.get('number')} 次命令退 75，{shown} 通（只报，不另做什么）")
                return
            self.alert(lib.PRUNE_SERVER_ALERT_KIND, task["id"],
                       f"任务 {task['id']} 第 {attempt.get('number')} 次命令退 75（prune-server 不可用）；{shown} {'退 ' + str(code) if code is not None else '没跑成'}："
                       f"{lib.last_line(text, '没有输出')[:300]}",
                       "照 check 那一句修 prune-server（task-server 不起、不停它：python3 prune-server/control start / install 由主 agent 起）；修好之后重交这条任务")
        threading.Thread(target=run, name=f"prune-server-check-{task['id']}", daemon=True).start()

    # ── ① 对账 ──
    def attempt_paths(self, attempt):
        directory = attempt["directory"]
        return os.path.join(directory, "result.json"), os.path.join(directory, "started"), os.path.join(directory, "scope")

    def settle_ended_attempts(self):
        """任务还 running、它最新那一次尝试却已经 ended 的（上一次定去向半途抛错，或服务在记尝试与定去向之间没了）：补定去向（settle_attempt）。
        任务 2461 就是这样留了两小时的孤儿：本机因此一直算有任务在跑。弄坏开关 ended-attempts-unsettled 下不补（改前的样子）。"""
        if lib.broken("ended-attempts-unsettled", self.environment):
            return
        for task, valid in self.store.select("tasks", "state = 'running'"):
            if not valid:
                continue
            attempts = [record for record, record_valid in self.store.attempts_of(task["id"]) if record_valid]
            if not attempts or attempts[-1].get("state") != "ended" or int(attempts[-1].get("number") or 0) != int(task.get("attempt_count") or 0):
                continue
            attempt = attempts[-1]
            say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试结局 {attempt.get('outcome')} 已记、任务没收尾：现在补收尾")
            self.settle_attempt(task, attempt, None)

    def reconcile(self):
        self.settle_ended_attempts()
        for attempt, valid in self.store.select("attempts", "state IN ('starting', 'running')"):
            found = self.store.task(attempt.get("task_id", 0))
            if found is None:
                continue
            task, task_valid = found
            if task.get("state") == "finished":
                continue
            if not valid or not task_valid:
                self.tampered(task["id"], task, "尝试记录" if not valid else "任务记录")
                continue
            result_path, started_path, _ = self.attempt_paths(attempt)
            if attempt["state"] == "starting" and lib.broken("start-twice-after-restart", self.environment) and not attempt.get("restarted"):
                attempt["restarted"] = True   # 弄坏开关：不看结果、单元与起跑标记，再起一份
                self.launch(task, attempt)
                continue
            result = read_json_file(result_path)
            if result is not None:
                self.finish_attempt(task, attempt, result)
                continue
            now = time.monotonic()
            if now - self.unit_checked_at.get(attempt["id"], 0.0) < self.unit_check_seconds:
                continue
            self.unit_checked_at[attempt["id"]] = now
            if attempt.get("unit"):
                states = unit_states(attempt["unit"], self.environment)
                if states is None:
                    continue
                load_state, active_state = states
            else:   # 弄坏开关 no-detach：起成子进程的那几次，看进程号在不在
                alive = any(child.pid == attempt.get("child_pid") and child.poll() is None for child in self.children)
                load_state, active_state = ("loaded", "active") if alive else ("not-found", "inactive")
            if active_state in lib.RUNNING_UNIT_STATES:
                self.unit_gone_since.pop(attempt["id"], None)
                self.check_stall(task, attempt)
                if attempt["state"] == "starting":
                    attempt["state"] = "running"
                    self.store.update("attempts", attempt)
                continue
            if attempt["state"] == "starting" and load_state == "not-found" and not os.path.exists(started_path):
                self.resume_starting(task, attempt, now)
                continue
            gone_since = self.unit_gone_since.setdefault(attempt["id"], now)
            if now - gone_since < self.lost_grace_seconds or lib.broken("lost-not-recorded", self.environment):
                continue
            self.finish_attempt(task, attempt, read_json_file(result_path))
        self.drive_sharded_parents()

    def resume_starting(self, task, attempt, now):
        """starting、单元不在、起跑标记也没有的那一次（文件头 ①）：拿这条任务的锁、现读任务再定——取消了的照取消收尾（不起）；
        起单元的回执超时过（start_uncertain_at，TS-06）的过了 start_uncertain_grace_seconds 还是这样，确认没起、记 unit-start-failed；
        别的是服务在写了单元名与起单元之间没了：起它一次。"""
        with self.store.task_lock(task["id"]):
            found = self.store.task(task["id"])
            current = found[0] if found is not None and found[1] else task
            if current.get("cancel_requested") or current.get("state") == "finished":
                say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试还没起、任务已取消：不起，照取消收尾")
                self.finish_attempt(current, attempt, None)
                return
            uncertain_at = attempt.get("start_uncertain_at")
            if uncertain_at:
                if time.time() - float(uncertain_at) < self.start_uncertain_grace_seconds:
                    return
                why = (f"systemd-run 回执超时之后照单元名 {attempt.get('unit')} 读回 {self.start_uncertain_grace_seconds:.0f} 秒，单元一直不在、起跑标记也没有："
                       "确认没起")
                attempt.update(state="ended", outcome="unit-start-failed", ended_at=round(time.time(), 3), note=why)
                self.store.update("attempts", attempt)
                self.unit_checked_at.pop(attempt["id"], None)
                self.finalize(current, "unit-start-failed", None, why)
                return
            say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试的单元名写进库之后、起单元之前服务没了：现在起它（只这一次）")
            self.launch(current, attempt)

    def check_stall(self, task, attempt):
        """日志距上一次增长满 stall_seconds、这段时间 CPU 也几乎没涨就停掉这一条（文件头「卡住」）。两台同一段：日志里除起法那一层的进度行之外的字节数（log_content），
        CPU 照那一台的接口读（Machine.cpu_usec：本机读尝试目录的 cpu_usec，第二台读日志里最近一行 MULTI_HOST_RUN_PEER_CPU 的 usage_usec）。
        这一回没停它的，再判疑似卡住（check_suspect：只通知、不停）。"""
        if attempt.get("stalled"):
            return
        elapsed = time.time() - float(attempt.get("started_at") or time.time())
        if elapsed < self.stall_grace_seconds and not lib.broken("stall-grace-skipped", self.environment):
            self.log_growth.pop(attempt["id"], None)   # 宽限里不判、不量：宽限过了从那一刻起量窗口
            self.cpu_windows.pop(attempt["id"], None)
            self.selftest_read(f"stall-grace task={task['id']} attempt={attempt['number']} elapsed={elapsed:.3f}")   # 自证：宽限里判过一次（stall-waits-for-grace 照它放行）
            return
        size = self.log_content(attempt)
        now = time.monotonic()
        cpu = self.machines[attempt["host"]].cpu_usec(attempt)
        if not self.judge_stall_stop(task, attempt, size, cpu, now):
            self.check_suspect(task, attempt, cpu, now)

    def judge_stall_stop(self, task, attempt, size, cpu, now):
        """文件头 ①「卡住」那两条停的规矩：日志与 CPU 都满窗口不涨、远超记录用时而日志满窗口不涨；停了交 True。"""
        if lib.broken("stall-ignored", self.environment):
            return False
        seen = self.log_growth.get(attempt["id"])
        if seen is None:
            # 头一回量（服务刚起来对账、或宽限刚过）：日志上一次涨的时刻取日志文件的修改时刻，不当成此刻（服务重启不把「日志多久没涨」清零）；
            # CPU 的窗口照旧从此刻起量（卡住判法不变）。读不到修改时刻、弄坏开关 stall-clock-reset-on-restart 下取此刻
            self.log_growth[attempt["id"]] = (size, self.log_changed_at(attempt, now), now, cpu, False)
            return False
        if seen[0] != size:
            self.log_growth[attempt["id"]] = (size, now, now, cpu, False)
            return False
        if now - seen[2] < self.stall_seconds:
            return False
        estimate = self.estimated_seconds(task)
        elapsed = time.time() - float(attempt.get("started_at") or time.time())
        # 用户：「一条任务跑得远超记录用时（例如超过五倍）而日志又满 10 分钟不涨，就停掉、打回」：这时不看 CPU（占着 CPU 的死循环也停）
        overran = bool(estimate) and elapsed > OVERRUN_STOP_FACTOR * estimate and not lib.broken("overrun-not-stopped", self.environment)
        if not overran and (seen[3] is None or cpu is None) and not lib.broken("cpu-unread-counted-as-idle", self.environment):
            # 窗口开头或此刻读不到 CPU（刚起、第二台那一趟还在起、换上读法之前起的尝试没有 cpu_usec）：读不到不等于不涨，不停，从此刻再看一个窗口
            self.log_growth[attempt["id"]] = (size, seen[1], now, cpu, True)
            return False
        if (not overran and cpu is not None and seen[3] is not None and (cpu - seen[3]) / 1e6 / (now - seen[2]) >= STALL_CPU_CORES
                and not lib.broken("cpu-ignored-for-stall", self.environment)):
            self.log_growth[attempt["id"]] = (size, seen[1], now, cpu, True)   # 日志不涨而 CPU 在涨：在算，不停，从此刻再看一个窗口
            return False
        attempt["stalled"] = True
        attempt["stalled_after"] = round(now - seen[1], 3)
        peak, cap = self.scope_memory_peak(attempt), int(attempt.get("memory_cap") or 0)
        if peak is not None and cap and peak >= STALL_AT_CAP_FRACTION * cap and not lib.broken("stalled-at-cap-not-retried", self.environment):
            # 顶在上限上卡在回收里：不记 stalled，停掉之后照撞上限办（finish_attempt 记 memory-cap-hit、照重试规矩乘 1.5 重排）
            attempt["stalled_at_cap"] = True
            attempt["stall_note"] = (f"顶在上限上卡住：内存峰值 {lib.mebibytes_text(peak)}、上限 {attempt.get('memory_cap_text') or lib.mebibytes_text(cap)}"
                                     f"（不低于 {STALL_AT_CAP_FRACTION:.0%}），日志 {now - seen[1]:.0f} 秒没涨")
            self.store.update("attempts", attempt)
            say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试{attempt['stall_note']}：停掉它，按撞上限办")
            threading.Thread(target=self.stop_running, args=(task["id"],), daemon=True).start()
            return True
        self.store.update("attempts", attempt)
        if overran:
            attempt["stall_note"] = f"跑了 {elapsed:.0f} 秒、超过记录用时 {estimate:.0f} 秒的 {OVERRUN_STOP_FACTOR:.0f} 倍，日志 {now - seen[1]:.0f} 秒没涨"
            self.store.update("attempts", attempt)
            say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试{attempt['stall_note']}：不看 CPU，停掉它，记 stalled")
        else:
            attempt["stall_note"] = f"日志 {now - seen[1]:.0f} 秒没涨、CPU 这 {now - seen[2]:.0f} 秒也几乎没涨"   # 判卡住的依据记进尝试，结局说明接上它
            self.store.update("attempts", attempt)
            say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试的{attempt['stall_note']}：停掉它，记 stalled")
        threading.Thread(target=self.stop_running, args=(task["id"],), daemon=True).start()
        return True

    def check_suspect(self, task, attempt, cpu, now):
        """疑似卡住（文件头 ①「疑似卡住」）：整个单元的 CPU 满 stall_seconds 平均不到 STALL_CPU_CORES 个核、要卡的这段时间它那张卡上又没有它的计算负载
        （量不到当有负载），不管日志涨不涨：不停，记进尝试的 suspect、写告警、叫醒在等的 wait（它打 lib.SUSPECT_MARK 那一行、退 lib.SUSPECT_EXIT）。
        一段只报一次；疑似之后 CPU 又涨了（相邻两次读之间平均不少于 STALL_CPU_CORES 个核）或卡上又有了它的负载，这一段结束，从此刻再量下一段。读不到 CPU 不当不涨：窗口作废重量。
        弄坏开关 suspect-unreported 下不判；suspect-card-load-ignored 下不看卡上有没有它的负载。"""
        if lib.broken("suspect-unreported", self.environment):
            return
        if cpu is None:
            self.cpu_windows.pop(attempt["id"], None)
            return
        card_busy = bool(task.get("gpu")) and self.card_load(attempt) and not lib.broken("suspect-card-load-ignored", self.environment)
        window = self.cpu_windows.get(attempt["id"])
        if window is None or cpu < window.last_cpu:
            self.cpu_windows[attempt["id"]] = SuspectWindow(now, cpu, now, cpu, card_busy, False)
            return
        if window.suspected:
            rising = (cpu - window.last_cpu) / 1e6 / max(now - window.last_at, 1e-6) >= STALL_CPU_CORES
            if rising or card_busy:   # 这一段结束：CPU 又涨了，或卡上又有了它的负载
                self.end_suspect(task, attempt, "CPU 又涨了" if rising else "卡上又有了它的计算负载")
                self.cpu_windows[attempt["id"]] = SuspectWindow(now, cpu, now, cpu, card_busy, False)
                return
            self.cpu_windows[attempt["id"]] = dataclasses.replace(window, last_at=now, last_cpu=cpu)
            return
        busy_seen = window.card_busy_seen or card_busy
        if now - window.started_at < self.stall_seconds:
            self.cpu_windows[attempt["id"]] = dataclasses.replace(window, last_at=now, last_cpu=cpu, card_busy_seen=busy_seen)
            return
        cores = (cpu - window.started_cpu) / 1e6 / max(now - window.started_at, 1e-6)
        if cores >= STALL_CPU_CORES or busy_seen:   # 在算（CPU 或卡上）：从此刻再量一个窗口
            self.cpu_windows[attempt["id"]] = SuspectWindow(now, cpu, now, cpu, card_busy, False)
            return
        self.cpu_windows[attempt["id"]] = dataclasses.replace(window, last_at=now, last_cpu=cpu, suspected=True)
        previous = attempt.get("suspect") or {}
        card_text = ("；它那张卡 " + attempt["card"] + " 上这段时间没有它的计算负载") if task.get("gpu") and attempt.get("card") else ""
        attempt["suspect"] = {"active": True, "segment": int(previous.get("segment") or 0) + 1, "at": round(time.time(), 3),
                              "window_seconds": round(now - window.started_at, 3), "cores": round(cores, 6), "threshold_cores": STALL_CPU_CORES,
                              "why": f"整个单元的 CPU 这 {now - window.started_at:.0f} 秒平均 {cores:.4f} 个核，不到 {STALL_CPU_CORES} 个核{card_text}（日志涨不涨都不看）"}
        # 先写告警、再记进尝试：wait 那一侧读到 suspect 时告警已在
        self.alert("suspected-stall", task["id"], f"任务 {task['id']} 第 {attempt['number']} 次尝试疑似卡住：{attempt['suspect']['why']}；服务不停它",
                   f"提交它的会话自己查（task-server/task log {task['id']}、task-server/task status {task['id']}），还在算就再起 task-server/task wait {task['id']} 接着等，"
                   f"真卡住了 task-server/task cancel {task['id']}")
        self.store.update("attempts", attempt)
        self.notify_finished(task["id"])   # 叫醒在等的 wait：它照 suspect 打一行、退出

    def unreported_suspect(self, task_id, session):
        """wait 那一侧调：这条任务在跑的那一次正疑似卡住、这一段还没报给提交它的会话，交那一段（连同第几次、哪台、日志），不然交 None。
        报给提交它的会话就记下这一段报过了（记在服务的内存里：只有排队线程写尝试记录，不和它抢着写）；别的会话来等照样看得到、不记。
        弄坏开关 suspect-reported-every-wait 下报过的照报。"""
        found = self.store.task(task_id)
        if found is None:
            return None
        for attempt, valid in self.store.attempts_of(task_id):
            suspect = attempt.get("suspect") or {}
            if not valid or attempt.get("state") not in ACTIVE_ATTEMPT_STATES or not suspect.get("active"):
                continue
            with self.suspect_lock:
                if self.suspect_reported.get(attempt["id"]) == suspect.get("segment") and not lib.broken("suspect-reported-every-wait", self.environment):
                    continue
                if found[0].get("session") == session:
                    self.suspect_reported[attempt["id"]] = suspect.get("segment")
            return dict(suspect, attempt=attempt.get("number"), host=attempt.get("host"), log=attempt.get("log"), card=attempt.get("card", ""))
        return None

    def end_suspect(self, task, attempt, why):
        suspect = dict(attempt.get("suspect") or {}, active=False, ended_at=round(time.time(), 3), ended_because=why)
        attempt["suspect"] = suspect
        self.store.update("attempts", attempt)
        say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试的疑似卡住（第 {suspect.get('segment')} 段）结束：{why}")

    def attention_items(self, task_id):
        """这条任务（分片的、按项估算的连同子任务）此刻要处置的提醒（文件头 ①「要处置的提醒」）：每项 {kind, tier, task, basis, why, next}，task 是提醒说的那一条
        （子任务的是子任务号）。不改任务；六种判法在 attention_of_record 与下面的分片推不动（改主工作区的那一种另起一次 apply-writer-patch.py --read-outcome，workspace_write_item），档按 attention_tier。
        弄坏开关 attention-shard-advance-unreported 下不报分片父任务推不动（别四种的开关在 attention_of_record）。"""
        found = self.store.task(task_id)
        if found is None or not found[1]:
            return []
        records = [found[0]]
        if found[0].get("sharded_entry") or found[0].get("item_entry"):
            try:
                records += self.children_of(task_id)
            except StoreRefused:   # 名下有校验不过的子任务行：不认那几行（summary 与 cancel 已写告警）
                pass
        now = time.time()
        items = []
        for record in records:
            items += self.attention_of_record(record, now)
        failure = dict(self.advance_failures.get(task_id) or {})
        if failure and found[0].get("state") in ("sharded", "estimating") and not lib.broken("attention-shard-advance-unreported", self.environment):
            items.append({"kind": "shard-advance-failed", "tier": int(failure.get("episode") or 1), "task": task_id,
                          "basis": f"推下一步连着出错 {failure.get('count')} 轮：{failure.get('why')}",
                          "why": "服务每一轮对账都重推，一直推不动整件就停在原地",
                          "next": (f"看 journalctl --user -u {lib.UNIT_NAME} -n 200 里「推下一步时出错」那一句前后的栈；修好之前任务 {task_id} 不会往下走，"
                                   f"等不起 task-server/task cancel {task_id}")})
        return items

    def attention_of_record(self, record, now):
        """一条任务记录（父任务或子任务）此刻要处置的提醒：排队过久、跑到记录用时的很多倍、换卡重排（可能已跑成）、清场没收尾。
        弄坏开关 attention-queue-unreported、attention-overrun-unreported、attention-card-replaced-unreported、attention-cleanup-unreported 各关一种。"""
        task_id, state = record["id"], record.get("state")
        attempts = [attempt for attempt, valid in self.store.attempts_of(task_id) if valid]
        items = []
        if state == "queued" and not lib.broken("attention-queue-unreported", self.environment):
            since = max([float(record.get("submitted_at") or now)] + [float(attempt.get("ended_at") or 0) for attempt in attempts])   # 重排的从上一次尝试结束起算
            waited = now - since
            tier = attention_tier(waited, self.queue_attention_seconds)
            if tier:
                items.append({"kind": "queued-too-long", "tier": tier, "task": task_id,
                              "basis": f"已排 {waited:.0f} 秒（{waited / 60:.0f} 分钟），第 {tier} 档（头一档 {self.queue_attention_seconds:.0f} 秒，之后每档翻倍）",
                              "why": self.waiting_text(record) or "排队原因此刻读不到（这一轮还没判到它）",
                              "next": (f"task-server/task status {task_id} 看「在等：…」；前面在跑的占着余量或第二台用不了就等，等不起 task-server/task cancel {task_id}、"
                                       f"改小命令重交；等得起再起 task-server/task wait {task_id}（排满 {self.queue_attention_seconds * 2 ** tier:.0f} 秒再叫一次）")})
        if state == "running" and not lib.broken("attention-overrun-unreported", self.environment):
            active = [attempt for attempt in attempts if attempt.get("state") in ACTIVE_ATTEMPT_STATES]
            elapsed = now - float(active[-1].get("started_at") or now) if active else 0.0
            if elapsed >= self.stall_grace_seconds:   # 先比便宜的：不足宽限的短任务不去查预计用时
                estimate = self.estimated_seconds(record)
                if estimate and estimate > 0:
                    first = max(self.overrun_attention_factor * estimate, self.stall_grace_seconds)
                    tier = attention_tier(elapsed, first)
                    if tier:
                        items.append({"kind": "overrun", "tier": tier, "task": task_id,
                                      "basis": (f"已跑 {elapsed:.0f} 秒，是记录用时 {estimate:.0f} 秒的 {elapsed / estimate:.1f} 倍，"
                                                f"第 {tier} 档（头一档 {first:.0f} 秒，之后每档翻倍）"),
                                      "why": "日志与 CPU 只要还在涨服务就不停它（日志满 10 分钟没涨且超过记录用时 5 倍才停）",
                                      "next": (f"task-server/task log {task_id} 与 task-server/task status {task_id} 看是不是还在做有用的事；还在算就再起 task-server/task wait {task_id}"
                                               f"（跑到 {first * 2 ** tier:.0f} 秒再叫一次），跑飞了 task-server/task cancel {task_id}、改了重交")})
        if not lib.broken("attention-card-replaced-unreported", self.environment):
            last = max([int(attempt.get("number") or 0) for attempt in attempts] or [0])
            replaced = [attempt for attempt in attempts if attempt.get("outcome") == "card-fault" and attempt.get("faulty_cards")
                        and (int(attempt.get("number") or 0) < last or state == "queued")]   # 后面还有尝试、或正排着等换卡的；最后一次就是 card-fault 的收尾由结局的理由行报
            if replaced:
                cards = "、".join(dict.fromkeys(card for attempt in replaced for card in attempt["faulty_cards"]))
                numbers = "、".join(str(attempt.get("number")) for attempt in replaced)
                now_state = {"finished": f"任务已 {record.get('outcome')}", "queued": "任务重新排队、换别的卡", "running": "任务已换别的卡、还在跑"}.get(state, f"任务状态 {state}")
                items.append({"kind": "card-replaced", "tier": len(replaced), "task": task_id,
                              "basis": f"清场判出卡 {cards} 坏（第 {numbers} 次尝试），{now_state}；坏卡停用 {CARD_QUARANTINE_SECONDS // 60} 分钟",
                              "why": "卡坏不是任务的错，服务已换别的卡重排；那张卡不修，之后要卡的任务还会撞上",
                              "next": "弹窗请用户重置那张卡（日志里 GPU_QUIESCE 那一行写着哪台哪张卡与依据）；任务不用重交"})
        if not lib.broken("attention-cleanup-unreported", self.environment):
            for attempt in attempts:
                if attempt.get("cleanup_pending") and attempt.get("state") in ACTIVE_ATTEMPT_STATES:
                    items.append({"kind": "cleanup-pending", "tier": max(1, int(attempt.get("number") or 1)), "task": task_id,
                                  "basis": f"第 {attempt.get('number')} 次尝试的命令已退出，清场没收尾：{attempt['cleanup_pending']}",
                                  "why": "服务保留这一次的资源与显卡占用、每隔几秒重查，查清之前任务不记结局",
                                  "next": (f"task-server/task log {task_id} 看这一次尝试的日志，查清留下的进程或远端连接；恢复之后服务自己重查收尾；"
                                           "不要删进程号、退出码或清理状态文件来放行")})
        if state == "finished" and not record.get("parent_id") and not lib.broken("workspace-write-interrupted-unreported", self.environment):
            item = self.workspace_write_item(record, attempts)
            if item is not None:
                items.append(item)
        return items

    def workspace_write_item(self, record, attempts):
        """改主工作区的命令（modifies_workspace）没正常结束：结局是 WORKSPACE_WRITER_INTERRUPTED_OUTCOMES 之一、或命令被信号杀（command-failed 而退出码在 SIGNAL_EXIT_CODES 里）时，
        可能写了一半，交一项 workspace-write-interrupted（点名要核的文件或补丁目录）；正常结束的、不改工作区的交 None。用户：「这个问题很严重 因为可能会导致补丁没有正常写入」。
        apply-writer-patch.py 有结局文件（补丁目录里的 apply-outcome.tsv）的照它判：起 --read-outcome，打全了、已退回的不报，没打完、打完之后又被改了、读不了的报（连同它打的那几行）；
        结局文件不在的不报（它在动手换之前写开始与文件行，不在就是还没开始换）；带 --dry-run 这类只读参数的不报。结了局的任务只核一次（workspace_write_checks）。
        弄坏开关 apply-outcome-ignored 下不读结局文件，一律照可能写了一半报。"""
        task_id = record["id"]
        if task_id in self.workspace_write_checks:
            return self.workspace_write_checks[task_id]
        outcome, exit_code = record.get("outcome"), record.get("exit_code")
        interrupted = outcome in WORKSPACE_WRITER_INTERRUPTED_OUTCOMES or (outcome == "command-failed" and exit_code in SIGNAL_EXIT_CODES)
        item = None
        if interrupted and self.modifies_workspace(record):
            item = self.workspace_write_judged(record, attempts, outcome, exit_code)
        self.workspace_write_checks[task_id] = item
        return item

    def workspace_write_judged(self, record, attempts, outcome, exit_code):
        task_id, command, cwd = record["id"], list(record.get("command") or []), record.get("cwd") or self.peer_root
        root = os.path.realpath(self.peer_root)
        positions = [index for index, word in enumerate(command) if not word.startswith("-")
                     and os.path.relpath(os.path.normpath(os.path.join(os.path.realpath(cwd), word)), root) in (*WORKSPACE_WRITERS, PATCH_APPLYING_ENTRY)]
        position = positions[0] if positions else 0
        writer = command[position] if positions else (command[0] if command else "")
        after = command[position + 1:]
        named = [word for index, word in enumerate(after) if not word.startswith("-") and (index == 0 or after[index - 1] != "--root")]
        number = max([int(attempt.get("number") or 0) for attempt in attempts] or [0])
        basis = f"第 {number} 次尝试结局 {outcome}（退 {exit_code}）" + ("，命令被信号杀" if outcome == "command-failed" else "")
        status_hint = f"git -C {shlex.quote(cwd)} status --short 与 git -C {shlex.quote(cwd)} diff --stat 看主工作区此刻改了哪几份"
        next_step = (f"核{('这几处：' + '、'.join(named)) if named else '它改的文件'}；{status_hint}；"
                     f"没写全的照原样补齐或退回，核过再重交（task-server/task log {task_id} 看它停在哪一步）")
        if os.path.relpath(os.path.normpath(os.path.join(os.path.realpath(cwd), writer)), root) == WRITER_PATCH_ENTRY:
            if any(word in PATCH_WRITER_READ_ONLY_ARGUMENTS for word in after):
                return None   # 只读的用法（--dry-run、--pending-status 这类）不写主工作区
            if not named:   # --finish-pending、--rollback-pending：照记下的暂存换，看暂存还在不在
                next_step = f"python3 {shlex.quote(writer)} --pending-status 看有没有没换完的，照它打的出路接着换完或退回；{status_hint}"
            else:
                outcome_file = os.path.join(cwd, named[0], PATCH_WRITER_OUTCOME_NAME)
                if not os.path.exists(outcome_file) and not lib.broken("apply-outcome-ignored", self.environment):
                    return None   # 结局文件在动手换之前写：不在就是还没开始换，主工作区没动
                read_back = [sys.executable, os.path.join(cwd, writer) if not os.path.isabs(writer) else writer, "--read-outcome", outcome_file]
                if "--root" in after[:-1]:
                    read_back += ["--root", after[after.index("--root") + 1]]
                if not lib.broken("apply-outcome-ignored", self.environment):
                    try:
                        completed = subprocess.run(read_back, cwd=cwd, capture_output=True, text=True, env=self.environment, stdin=subprocess.DEVNULL,
                                                   timeout=PATCH_WRITER_CHECK_SECONDS)
                        verdict, lines = completed.returncode, (completed.stdout + completed.stderr).strip().splitlines()
                    except (OSError, subprocess.TimeoutExpired) as error:
                        verdict, lines = 2, [f"--read-outcome 起不来或没在 {PATCH_WRITER_CHECK_SECONDS} 秒里退：{type(error).__name__}: {error}"]
                    if verdict in (0, 1):
                        return None   # 打全了、已退回：主工作区不在半截上
                    basis += f"；结局文件 {outcome_file} 判{PATCH_WRITER_VERDICTS.get(verdict, f'退 {verdict}')}：" + " / ".join(line.strip() for line in lines[-6:])
                next_step = (f"python3 {shlex.quote(writer)} --read-outcome {shlex.quote(outcome_file)} 逐个文件看已换上与没换上，"
                             f"再照 python3 {shlex.quote(writer)} --pending-status 的出路接着换完或退回；补丁目录 {named[0]}；{status_hint}")
        return {"kind": "workspace-write-interrupted", "tier": 1, "task": task_id, "basis": basis,
                "why": f"这条命令改主工作区（{writer}），没正常结束，可能写了一半",
                "next": next_step}

    def unreported_attention(self, task_id, session):
        """wait 那一侧调：这条任务此刻要处置的提醒里还没报给提交它的会话、或档又跨过一档的，交这些项（attention_items 的项）；没有交空表。
        报给提交它的会话就记下这一种这一档报过了（记在服务的内存里，同 unreported_suspect；服务重启后再报一次）；别的会话来等照样看得到、不记。
        弄坏开关 attention-reported-every-wait 下报过的照报。"""
        found = self.store.task(task_id)
        if found is None:
            return []
        owner = found[0].get("session") == session
        reported = []
        for item in self.attention_items(task_id):
            key = (item["task"], item["kind"])
            with self.attention_lock:
                if self.attention_reported.get(key, 0) >= item["tier"] and not lib.broken("attention-reported-every-wait", self.environment):
                    continue
                if owner:
                    self.attention_reported[key] = max(self.attention_reported.get(key, 0), item["tier"])
            reported.append(item)
        return reported

    def selftest_advance_fault(self):
        """只在自证里：TASK_SERVER_SELFTEST_ADVANCE_FAILS=<文件>，那份文件在时 drive_sharded_parents 每推一个父任务先抛一个错（演分片父任务推不动）。"""
        arm = lib.selftest_override("TASK_SERVER_SELFTEST_ADVANCE_FAILS", self.environment)
        if arm and os.path.exists(arm):
            raise RuntimeError("自证造的推不动（TASK_SERVER_SELFTEST_ADVANCE_FAILS 点名的文件在）")

    def note_advance_failure(self, parent_id, why):
        """分片父任务这一轮推不动（drive_sharded_parents）：记进内存，attention_items 读。连着推不动是同一回（count 加一），推通之后再推不动是新的一回（档加一）。"""
        with self.attention_lock:
            current = self.advance_failures.get(parent_id)
            if current is None:
                episode = self.advance_episodes.get(parent_id, 0) + 1
                self.advance_episodes[parent_id] = episode
                self.advance_failures[parent_id] = {"episode": episode, "why": why, "count": 1}
            else:
                current.update(why=why, count=current["count"] + 1)

    def note_advance_recovered(self, parent_id):
        with self.attention_lock:
            self.advance_failures.pop(parent_id, None)

    def card_load(self, attempt):
        """这一次尝试此刻在它那张卡上有没有计算负载：照那一台的接口读（Machine.card_busy：本机读 nvidia-smi -i <序号> 的计算进程里有没有它的子孙进程；
        第二台量不到当有）；量不到当有，不报疑似卡住。尝试起在哪台，它那张卡就在哪台（place_on_card 照卡挑台）。"""
        return self.machines[attempt["host"]].card_busy(attempt)

    def log_changed_at(self, attempt, now):
        """日志文件的修改时刻换到 time.monotonic() 的钟上（不晚于 now）；读不到交 now。"""
        if lib.broken("stall-clock-reset-on-restart", self.environment):
            return now
        try:
            age = time.time() - os.path.getmtime(attempt.get("log", ""))
        except OSError:
            return now
        return now - max(0.0, age)

    def scope_cpu_usec(self, attempt):
        """这一次尝试此刻累计用了多少 CPU：attempt_runner 每次采样写进尝试目录的 cpu_usec（这一条子孙进程所在各 scope 的合计，里层包装各开的 scope 也在内）；
        读不到交 None。弄坏开关 outer-scope-cpu-only 下只读外层那一个 scope（改前的样子）。"""
        try:
            if lib.broken("outer-scope-cpu-only", self.environment):
                with open(self.attempt_paths(attempt)[2], encoding="utf-8") as handle:
                    scope = handle.read().strip()
                with open(os.path.join(scope, "cpu.stat"), encoding="utf-8") as handle:
                    return next(int(line.split()[1]) for line in handle if line.startswith("usage_usec "))
            with open(os.path.join(attempt.get("directory", ""), "cpu_usec"), encoding="utf-8") as handle:
                return int(handle.read().strip())
        except (OSError, ValueError, StopIteration, IndexError):
            return None

    def scope_memory_peak(self, attempt):
        """这一次尝试外层 scope（内存包装给它开的、尝试目录的 scope 文件记着）此刻的 memory.peak，字节；读不到（第二台、还没采到 scope）交 None。"""
        try:
            with open(self.attempt_paths(attempt)[2], encoding="utf-8") as handle:
                scope = handle.read().strip()
            with open(os.path.join(scope, "memory.peak"), encoding="utf-8") as handle:
                return int(handle.read().strip())
        except (OSError, ValueError):
            return None

    def log_content(self, attempt):
        """→ 日志里算「在涨」的那部分字节数（读不到交 -1）。两台同一段：整份的大小减去起法那一层打的进度行（MULTI_HOST_RUN_STEP、MULTI_HOST_RUN_PEER_CPU）；
        从上次读到的位置接着读整行，最近一行 MULTI_HOST_RUN_PEER_CPU 的 usage_usec 记进 log_offsets（第二台的 Machine.cpu_usec 读它）。"""
        path = attempt.get("log", "")
        try:
            size = os.path.getsize(path)
        except OSError:
            return -1
        offset, noise, cpu = self.log_offsets.get(attempt["id"], (0, 0, None))
        with contextlib.suppress(OSError), open(path, "rb") as handle:
            while size > offset:   # 一次至多读 LOG_SCAN_BYTES：服务重启之后头一回接着读的是整份日志
                handle.seek(offset)
                chunk = handle.read(min(size - offset, LOG_SCAN_BYTES))
                complete = chunk[:chunk.rfind(b"\n") + 1] or (chunk if len(chunk) == LOG_SCAN_BYTES else b"")   # 一整块都没有换行的照整块往前走
                if not complete:
                    break
                for line in complete.splitlines(keepends=True):
                    text = line.decode("utf-8", "replace").strip()
                    found = PEER_CPU_LINE.match(text)
                    if found:
                        cpu = int(found.group(2)) if found.group(3) == "yes" else cpu
                    if found or text.startswith("MULTI_HOST_RUN_STEP "):
                        noise += len(line)
                offset += len(complete)
        self.log_offsets[attempt["id"]] = (offset, noise, cpu)
        return size - noise

    def finish_attempt(self, task, attempt, result):
        # 有结果不等于收干净。保持 starting/running，现有资源账和 busy_cards 会继续保留这次占用。
        if attempt.get("cleanup_pending") and time.time() < attempt.get("cleanup_retry_at", 0):
            return
        found = self.store.task(task["id"])   # 现读：取消那一侧改过 cancel_requested（不拿对账时读的那一份）
        if found is not None and found[1]:
            task = found[0]
        exit_code = result.get("exit_code") if result else None
        cancelled = bool(task.get("cancel_requested")) and (result is None or bool(result.get("stopped_signal")))
        tail = log_tail(attempt.get("log", ""))
        machine = self.machines[attempt["host"]]
        exit_code, judged_host, runner_exit = machine.command_exit(exit_code, tail)   # 起法那一层打进日志的命令自己的退出码（Machine.command_exit）
        if runner_exit is not None:
            attempt["peer_runner_exit_code"] = runner_exit
        outcome = lib.classify_attempt(exit_code, judged_host, tail, cancelled, self.environment)
        if attempt.get("stalled") and not cancelled and (result is None or result.get("stopped_signal")):
            outcome = "memory-cap-hit" if attempt.get("stalled_at_cap") else "stalled"   # 顶在上限上卡住的按撞上限办（judge_stall_stop）
        if outcome == "card-fault":   # 清场点名的坏卡（task_server_lib.py 文件头「错误类型」card-fault 那一行）：settle_attempt 停用、告警与结局句点名的都是它们
            attempt["faulty_cards"] = lib.card_fault_cards(tail)
            assigned = card_key(attempt.get("card"))
            if lib.broken("card-fault-assigned-card-quarantined", self.environment):   # 弄坏开关：照改前当分到的那张是坏卡
                attempt["faulty_cards"] = [f"{assigned[0]}:{assigned[1]}"] if assigned else []
        if outcome == "squeezed-out":   # 被谁挤掉：同一台此刻在跑的别的尝试（打回的理由 by= 那一栏，task_server_lib.reason_fields）
            attempt["squeezed_by"] = "、".join(f"任务 {other.get('task_id')}（上限 {other.get('memory_cap_text')}）"
                                              for other, _ in self.store.select("attempts", "state IN ('starting', 'running')")
                                              if other.get("host") == attempt.get("host") and other.get("id") != attempt.get("id"))
        if outcome == "squeezed-out" and not lib.broken("squeeze-at-total-not-cap-hit", self.environment):
            # 这一条的上限已经到了那台上限收到的那个数（总上限减那台此刻的余量）：slice 满就是它自己撞了上限（实测：上限与总上限都是 50G 的那一条被内核在 slice 那一层杀，记成被挤掉、原样重排了三次）
            total = self.usable_maxima(self.resources.snapshot(False)).get(attempt.get("host", ""))   # 上限收到的那个数（booked_at_cap）；余量量不到的是 None，不改判
            if total and int(attempt.get("memory_cap") or 0) >= total:
                outcome = "memory-cap-hit"
        peer_clean, peer_note = self.confirm_peer_cleanup(attempt)
        local_clean, leftover = self.stop_leftovers(attempt, result)
        if leftover:
            attempt["leftover_note"] = leftover
        elif local_clean and attempt.get("cleanup_pending"):
            attempt["leftover_note"] = "清理重查已确认，本机进程已收干净"
        if not peer_clean or not local_clean:
            note = "；".join(part for part in (peer_note, leftover) if part)
            changed = attempt.get("cleanup_pending") != note
            attempt.update(cleanup_pending=note, cleanup_retry_at=time.time() + 5)
            self.store.update("attempts", attempt)
            if changed:
                say(f"TASK_CLEANUP time={time.time():.6f} attempt={attempt['id']} card={attempt.get('card', '')} status=pending detail={note}")
                self.alert("cleanup-pending", task["id"], f"任务 {task['id']} 清理尚未确认，保留资源占用：{note}",
                           "服务会重查；检查该尝试日志与远端连接。不要删进程号、退出码或清理状态文件来放行")
            return
        attempt.pop("cleanup_pending", None)
        attempt.pop("cleanup_retry_at", None)
        say(f"TASK_CLEANUP time={time.time():.6f} attempt={attempt['id']} card={attempt.get('card', '')} status=confirmed")
        self.log_growth.pop(attempt.get("id"), None)
        self.log_offsets.pop(attempt.get("id"), None)
        self.cpu_windows.pop(attempt.get("id"), None)   # suspect_reported 不删：wait 那一侧可能刚读到这一次还在跑，删了会把报过的那一段再报一遍
        attempt.update(state="ended", exit_code=exit_code, outcome=outcome, ended_at=round((result or {}).get("ended_at") or time.time(), 3))
        attempt["seconds"] = round(attempt["ended_at"] - attempt.get("started_at", attempt["ended_at"]), 3)
        if result and not lib.broken("resources-unrecorded", self.environment):
            for name in ("memory_peak", "cpu_usec", "cores", "gpu_memory_peak"):
                attempt[name] = result.get(name)
            attempt["build_peak"] = result.get("build_directory_peak")
            if result.get("seconds") is not None:
                attempt["seconds"] = result["seconds"]
        if not lib.broken("resources-unrecorded", self.environment):
            # 起法那一层打进日志的峰值（Machine.ended_peaks：第二台是 multi-host-run.sh 打的 MULTI_HOST_RUN_PEER_MEMORY_PEAK 与 MULTI_HOST_RUN_PEER_BUILD_PEAK）：
            # attempt_runner 量不到的那一项照它记
            for name, value in machine.ended_peaks(tail).items():
                if attempt.get(name) is None:
                    attempt[name] = value
        kept_label = machine.kept_build_label(tail)
        if kept_label and attempt.get("build_directory") and os.path.basename(attempt["build_directory"]) == kept_label:
            # 起法那一层报这一次的编译目录留着了（第二台带 --keep-build-directory 的那一趟）：记成没删，重排到同一台的那一次接着用，换台或有了结局时服务删
            attempt["build_directory_removed"], attempt["build_directory_kept"] = False, True
        if attempt.get("memory_peak") is None and lib.broken("unmeasured-recorded-as-zero", self.environment):
            attempt["memory_peak"] = 0   # 弄坏开关：量不到的记成 0
        self.read_item_lines(task, attempt)   # 按项的 ITEM_PLAN 与 ITEM_DONE（task_server_lib.py 文件头「资源记录」的预计用时）
        self.store.update("attempts", attempt)
        say(f"TASK_RESOURCES_RELEASED time={time.time():.6f} attempt={attempt['id']} card={attempt.get('card', '')} outcome={outcome}")
        self.resources.invalidate()
        self.unit_checked_at.pop(attempt["id"], None)
        self.unit_gone_since.pop(attempt["id"], None)
        try:
            self.settle_attempt(task, attempt, result)
        except Exception as error:   # 尝试已记成 ended：这里出错不能留下「任务 running、尝试 ended」的孤儿，告警、下一轮对账 settle_ended_attempts 补收尾
            say(f"  ✗ 任务 {task['id']} 第 {attempt['number']} 次尝试的结局 {outcome} 已记，定任务去向时出错（{type(error).__name__}: {error}）：下一轮对账补收尾")
            self.alert("settle-failed", task["id"], f"任务 {task['id']} 第 {attempt['number']} 次尝试结局 {outcome} 已记、定去向时出错：{type(error).__name__}: {error}",
                       "服务下一轮对账会补收尾（settle_ended_attempts）；还 running 就看 journalctl --user -u singlefs-task-server.service 里这一句前后的栈")

    def read_item_lines(self, task, attempt):
        """尝试收尾时读日志里每项那几行（task_server_lib.py 文件头「资源记录」的预计用时）：估算子任务的 ITEM_PLAN 记进这一次尝试的 item_plan（{项: {to_judge, reusable}}，
        一行都没有是空表）；命令起的是三条按项入口之一的（lib.item_entry），ITEM_DONE 照 record_item_rates 记进 item_rates 表。别的命令不读日志。
        记速率时库出错只写告警，不挡这一次尝试收尾。弄坏开关 item-done-unrecorded 下不记 ITEM_DONE。"""
        estimate = task.get("role") == lib.ESTIMATE_ROLE
        entry, _ = lib.item_entry(task.get("command") or [], task.get("cwd") or lib.REPOSITORY_ROOT)
        if not estimate and not entry:
            return
        plans, dones = lib.item_lines(attempt.get("log", ""))
        if estimate:
            attempt["item_plan"] = plans
            return
        if lib.broken("item-done-unrecorded", self.environment):
            return
        try:
            self.record_item_rates(entry, task, attempt, dones)
        except (sqlite3.Error, StoreRefused) as error:
            say(f"  ✗ 任务 {task['id']} 第 {attempt.get('number')} 次尝试的每项速率没记成（{type(error).__name__}: {error}）")
            self.alert("item-rates-unrecorded", task["id"], f"任务 {task['id']} 第 {attempt.get('number')} 次尝试的 ITEM_DONE 没记成：{type(error).__name__}: {error}",
                       "下一次同一项跑完会再记；一直记不成就看 journalctl --user -u singlefs-task-server.service 里这一句前后")

    def record_item_rates(self, entry, task, attempt, dones):
        """ITEM_DONE 每行（lib.item_lines）照（入口脚本、项名）记进 item_rates 表最近一次的两个速率（task_server_lib.py 文件头「资源记录」的预计用时）：
        exit 不是 0 的、judged 与 reused 都是 0 的不记；judged 为 0 只更新每复用一条几秒（seconds ÷ reused），reused 为 0 只更新每现判一条几秒（seconds ÷ judged），
        两样都不为 0 的只更新现判的：(seconds − reused × 已记的复用速率) ÷ judged（不小于 0），没有复用速率的取 seconds ÷ judged。校验不过的那一行不动、写告警。"""
        now = round(time.time(), 3)
        for done in dones:
            if done["exit"] != 0 or not (done["judged"] or done["reused"]):
                continue
            key = lib.item_rate_key(entry, done["item"])
            source = {"task": task["id"], "attempt": attempt.get("id"), "seconds": done["seconds"], "judged": done["judged"], "reused": done["reused"], "at": now}
            rows = self.store.select("item_rates", "command_key = ?", (key,))
            current = rows[0] if rows else None
            if current is not None and not current[1]:
                self.alert("record-tampered", task["id"], f"item_rates 里入口 {entry} 的项 {done['item']} 那一行校验不过：这一次的速率不记",
                           "查是谁直接写了库（只有 task-server 读写它）")
                continue
            if done["judged"] == 0:
                fields = {"seconds_per_reused": done["seconds"] / done["reused"], "reused_from": source}
            elif done["reused"] == 0:
                fields = {"seconds_per_judged": done["seconds"] / done["judged"], "judged_from": source}
            else:
                reused_rate = current[0].get("seconds_per_reused") if current is not None else None
                share = done["reused"] * reused_rate if isinstance(reused_rate, (int, float)) else 0.0
                fields = {"seconds_per_judged": max(0.0, done["seconds"] - share) / done["judged"], "judged_from": dict(source, reused_share=round(share, 3))}
            if current is None:
                record = dict({"task_id": 0, "state": lib.ITEM_RATE_STATE, "session": "", "command_key": key, "entry": entry, "item": done["item"],
                               "seconds_per_judged": None, "seconds_per_reused": None, "judged_from": None, "reused_from": None}, **fields)
                try:
                    self.store.insert("item_rates", record)
                    continue
                except sqlite3.IntegrityError:   # 同一刻别处先插了这一项（item_rates_key 唯一）：改那一行
                    rows = self.store.select("item_rates", "command_key = ?", (key,))
                    if not rows or not rows[0][1]:
                        continue
                    current = rows[0]
            self.store.change("item_rates", current[0]["id"], fields)

    def labelled_scope(self, attempt):
        """本机 slice 下这次尝试的 scope；确认没有交空串，无法查询交 None。"""
        label = attempt.get("label") or ""
        if not label:
            return ""
        directory = self.peaks.slice_directory_of(self.environment.get("RUN_WITH_MEMORY_CAP_SLICE") or self.peaks.DEFAULT_SLICE,
                                                  self.environment.get("RUN_WITH_MEMORY_CAP_CGROUP_ROOT") or "/sys/fs/cgroup")
        if directory is None:
            return None
        if directory:
            try:
                with os.scandir(directory) as entries:
                    list(entries)
            except FileNotFoundError:
                return ""
            except OSError:
                return None
        for name, entry in self.peaks.slice_scopes(directory).items():
            if entry.get(self.peaks.SCOPE_LABEL_KEY) == label and name.endswith(".scope"):
                return os.path.join(directory, name)
        return ""

    def confirm_peer_cleanup(self, attempt):
        """新尝试的远端清理回执。失联、包装被杀或回执读不了都保留占用；恢复连接后重试收尾。"""
        path = attempt.get("peer_cleanup_file")
        if not path:   # 升级前的尝试没有此协议；不能凭空推断它写过新回执。
            return True, ""
        record = read_json_file(path)
        if record and record.get("status") in ("not-started", "confirmed"):
            return True, ""
        if record and record.get("status") == "pending":
            try:
                completed = subprocess.run(["bash", self.peer_runner, "--recover-cleanup", path],
                                           capture_output=True, text=True, env=self.environment,
                                           stdin=subprocess.DEVNULL, timeout=60)
                # 不凭退出码放行：回执必须原子写成 confirmed。
                with open(attempt["log"], "a", encoding="utf-8") as handle:
                    handle.write(completed.stdout + completed.stderr)
            except (OSError, subprocess.TimeoutExpired):
                pass
            record = read_json_file(path)
            if record and record.get("status") == "confirmed":
                return True, ""
        return False, f"远端清理回执未确认（{path}），尚不能归还显卡"

    def stop_leftovers(self, attempt, result):
        """尝试正常结束（结果已写、没被停）之后留下的进程（文件头 ①「留下的进程」）：单元里的由 attempt_runner 退出之前自己停掉、回读，记在 result.json 的
        unit_leftovers（它一退 systemd 就收单元、当场杀掉剩下的，服务那时回读不到）；外层 scope（内存包装开的，单元收尾时不跟着停）由服务回读，
        还有进程就发一次 systemctl --user kill --signal=SIGKILL（只对这一次自己的 scope 名），至多等 LEFTOVER_STOP_SECONDS 秒回读 cgroup.procs 空了没有。
        → (确认清空, 说明)。未知与仍有进程都交 False，后续对账重查。"""
        if lib.broken("leftover-processes-kept", self.environment) or not attempt.get("unit"):
            return True, ""
        found, remaining = {}, {}
        unit_leftovers = (result or {}).get("unit_leftovers") or {}
        if unit_leftovers.get("processes"):
            found[attempt["unit"]] = [str(process) for process in unit_leftovers["processes"]]
        # runner 的回读只是当时的快照；还剩或读不了时重新查 cgroup，不用旧 PID 杀可能复用的进程。
        if not unit_leftovers.get("verified") or unit_leftovers.get("remaining"):
            left = unit_processes(attempt["unit"], self.environment)
            if left is None:
                return False, f"单元 {attempt['unit']} 的进程读不了"
            runner_pid = str((result or {}).get("runner_pid") or "")
            left = [pid for pid in left if pid != runner_pid]
            if left:
                found[attempt["unit"]] = left
                with contextlib.suppress(OSError, subprocess.TimeoutExpired):
                    subprocess.run(["systemctl", "--user", "kill", "--signal=SIGKILL", attempt["unit"]],
                                   capture_output=True, env=self.environment, stdin=subprocess.DEVNULL, timeout=30)
                current = unit_processes(attempt["unit"], self.environment)
                if current is None:
                    return False, f"停单元 {attempt['unit']} 后无法回读"
                remaining[attempt["unit"]] = [pid for pid in current if pid != runner_pid]
        scope = str((result or {}).get("scope") or "") or self.labelled_scope(attempt)   # 命令比一轮采样还短时按标签找
        if scope is None:
            return False, "无法查询本机 scope，保留资源占用"
        if scope.startswith("/sys/fs/cgroup/") and scope.endswith(".scope"):
            name = os.path.basename(scope)
            left = cgroup_processes(scope)
            if left is None:
                return False, f"scope {name} 的进程读不了"
            if left:
                with contextlib.suppress(OSError, subprocess.TimeoutExpired):
                    subprocess.run(["systemctl", "--user", "kill", "--signal=SIGKILL", name], capture_output=True, env=self.environment, stdin=subprocess.DEVNULL, timeout=30)
                deadline = time.monotonic() + LEFTOVER_STOP_SECONDS
                still_there = cgroup_processes(scope)
                while still_there and time.monotonic() <= deadline:
                    time.sleep(0.2)
                    still_there = cgroup_processes(scope)
                if still_there is None:
                    return False, f"停 scope {name} 后无法回读"
                found[name], remaining[name] = left, still_there
        if not found:
            return True, ""
        counted = "、".join(f"{name} 里 {len(left)} 个（进程号 {' '.join(left[:8])}{' …' if len(left) > 8 else ''}）" for name, left in found.items())
        still = "、".join(f"{name} 还剩 {len(left)} 个" for name, left in remaining.items() if left)
        text = (f"命令退出之后还留着进程：{counted}（命令起的孙进程没收干净）"
                + (f"，回读 {still}，保留资源占用" if still else "，服务已停掉，回读 cgroup 空了"))
        say(f"  · 第 {attempt.get('number')} 次尝试{text}")
        return not bool(still), text

    def finished_preface(self, task, attempt, outcome):
        """收尾那一刻结局说明前面接的几句（文件头 ①「留下的进程」与 task_server_lib.py 文件头「续跑」）：留下的进程停掉了的那一句；
        结局不是 succeeded、这一次跑超过 lib.LONG_RUN_SECONDS、命令又没登记续跑（lib.resumable_command）的另写「这条跑了 N 分钟、没有续跑：重交会从头跑」。
        弄坏开关 long-run-not-noted 下不写后一句。"""
        parts = [attempt["leftover_note"]] if attempt.get("leftover_note") else []
        seconds = float(attempt.get("seconds") or 0)
        limit = float(lib.selftest_override("TASK_SERVER_SELFTEST_LONG_RUN_SECONDS", self.environment) or lib.LONG_RUN_SECONDS)
        if (outcome not in ("succeeded", "cancelled") and seconds > limit and not lib.broken("long-run-not-noted", self.environment)
                and not lib.resumable_command(task.get("command") or [], task.get("cwd"), self.environment)):
            parts.append(f"这条跑了 {seconds / 60:.0f} 分钟、没有续跑：重交会从头跑")
        return "".join(part + "；" for part in parts)

    def settle_attempt(self, task, attempt, result=None):
        """尝试的结局已记进库（state=ended、outcome）之后定这条任务的去向（文件头「重试」）：告警、取消、钉值与改工作区的不重排、retry_decision 重排或收尾。
        与记尝试分开写：这一步半途出错时尝试已是 ended、任务还 running，由 reconcile 的 settle_ended_attempts 补收尾，不留孤儿；补收尾时 result 是 None。"""
        outcome, exit_code = attempt.get("outcome", ""), attempt.get("exit_code")
        # 停用、告警与结局句点名的是清场那一行的坏卡清单（finish_attempt 记的 faulty_cards），不是这一次分到的那张（task_server_lib.py 文件头「错误类型」card-fault 那一行）
        faulty = list(attempt.get("faulty_cards") or []) if outcome == "card-fault" else []
        faulty_text, assigned_text = "、".join(faulty), attempt.get("card") or "没分到卡"
        if faulty and not lib.broken("card-fault-not-replaced", self.environment):
            for identity in faulty:
                self.quarantined_cards[identity] = time.time()   # 这几张停用 CARD_QUARANTINE_SECONDS 秒，这一条换别的卡重排（retry_decision）
        if outcome == "card-fault" and faulty and not lib.broken("card-fault-unalerted", self.environment):
            self.alert("card-fault", task["id"], f"清场判出卡 {faulty_text} 坏（任务 {task['id']} 第 {attempt['number']} 次尝试，这一次分到的是 {assigned_text}，"
                       f"日志 {attempt.get('log')}）；坏的那几张停用 {CARD_QUARANTINE_SECONDS // 60} 分钟，这一条换别的卡重排",
                       "主 agent 当场弹窗请用户重置那张卡（日志里 GPU_QUIESCE 那一行写着哪台哪张卡与依据）；修好之前它到点会再被试一次，还坏就再停用")
        elif outcome == "card-fault" and not lib.broken("card-fault-unalerted", self.environment):
            self.alert("card-fault", task["id"], f"清场判出有卡坏，日志里清场那一行取不到坏卡清单（faulty-cards=）（任务 {task['id']} 第 {attempt['number']} 次尝试，"
                       f"这一次分到的是 {assigned_text}，日志 {attempt.get('log')}）；一张卡都没停用，这一条照 card-fault 收尾、不重排",
                       "看那份日志里 GPU_QUIESCE 那一行的 cards=（哪台哪张卡与依据），主 agent 当场弹窗请用户重置那张卡；清场那一行没有 faulty-cards= 的，查 scripts/crash/gpu-quiesce-run.sh 是不是换上了")
        if outcome == "peer-run-failed":   # 第二台那一趟没跑成：写告警、/health 的 peer 一栏记最近一次（本文件头 ②「机器」）
            self.note_peer_run_failure(task, attempt)
        if outcome == lib.PRUNE_SERVER_UNAVAILABLE_OUTCOME:   # 命令退 75：起一次 prune-server/control check，不通写告警（只报，不起、不停它）
            self.check_prune_server(task, attempt)
        if outcome == "cancelled":
            self.finalize(task, outcome, exit_code, "提交它的会话取消、单元已停")
            return
        counts = dict(task.get("outcome_counts") or {})
        counts[outcome] = counts.get(outcome, 0) + 1
        # 结局与重排的说明前面写明（task status 的说明一行看得见）：顶在上限上卡住的那一次（judge_stall_stop）
        at_cap = f"{attempt.get('stall_note')}；" if (outcome == "memory-cap-hit" and attempt.get("stalled_at_cap")) or (outcome == "stalled" and attempt.get("stall_note")) else ""   # 判卡住的依据写进说明，不再是「没有说明」
        leftover = f"{attempt['leftover_note']}；" if attempt.get("leftover_note") else ""   # 命令退出之后留下的进程停掉了（stop_leftovers）：重排的说明同样写
        outcome_preface = at_cap
        if outcome == "memory-cap-hit" and attempt.get("prediction") == lib.PINNED_HISTORY and not lib.broken("pinned-cap-grown", self.environment):
            # 钉值撞了上限不放大（文件头 ②「内存」的钉值）：上限就是配置的总上限，到顶照 memory-cap-at-maximum 打回
            self.finalize(task, "memory-cap-at-maximum", exit_code,
                          f"{outcome_preface}{self.finished_preface(task, attempt, 'memory-cap-at-maximum')}钉值（多机配置的 {lib.PIN_SOURCE}）{attempt.get('memory_cap_text', '')} 也撞了上限：钉值不放大，打回",
                          outcome_counts=counts, maximum_host=attempt.get("host", ""), maximum_cap_text=attempt.get("memory_cap_text", ""))
            return
        if outcome in WORKSPACE_WRITER_NO_RETRY_OUTCOMES and self.modifies_workspace(task) and not lib.broken("workspace-writers-retried", self.environment):
            # 改主工作区的命令（往工作区打补丁的合入后验证、换钩子、打补丁、暂存）跑第二遍结果就不同：凡是会重排的结局一律不重排，打回提交方自己改了重交，不绕过
            #（用户：「改主工作区的命令撞上限不再原样重排、打回让提交方修改 后提交，不能绕过」）
            self.finalize(task, outcome, exit_code,
                          f"{outcome_preface}{self.finished_preface(task, attempt, outcome)}这条命令改主工作区，跑第二遍结果就变（补丁已打上的再打会被拒），服务不重排：改好再重交——merge-and-verify 带 --skip-apply、"
                          f"或先在副本里跑；下次这条命令从撞过的上限 {attempt.get('memory_cap_text', '')} × 1.5 起记", outcome_counts=counts)
            return
        if outcome == "card-fault" and not faulty:
            # 取不到坏卡清单：不知道停哪张，一张都不停、不重排（重排多半又派回同一张坏卡），照 card-fault 收尾
            self.finalize(task, outcome, exit_code, f"{outcome_preface}{self.finished_preface(task, attempt, outcome)}清场判出有卡坏，日志里清场那一行取不到坏卡清单（faulty-cards=），"
                          f"一张卡都没停用、不重排（这一次分到的是 {assigned_text}）", outcome_counts=counts)
            return
        if outcome == "card-fault" and not self.cards_in_service(self.resources.snapshot(True)).cards:
            # 坏的那几张停用之后多机配置里一张能派的卡都不剩：换卡重排也派不出去，照 card-fault 收尾（不排着干等停用到点）
            self.finalize(task, outcome, exit_code, f"{outcome_preface}{self.finished_preface(task, attempt, outcome)}清场判出卡 {faulty_text} 坏了，停用之后没有别的卡可派"
                          f"（这一次分到的是 {assigned_text}）", outcome_counts=counts)
            return
        maxima = self.usable_maxima(self.resources.snapshot(bool(task.get("gpu")))) if outcome == "memory-cap-hit" else {}   # 乘 1.5 照上限收到的那个数比
        fixed_host = PEER if self.peer_only_entry(task) else ""
        if fixed_host:   # 弄坏开关 host-lists-restored：照改前只派第二台的入口不换台
            maxima = {fixed_host: maxima.get(fixed_host)} if maxima else {}
        decision = lib.retry_decision(counts, outcome, int(attempt.get("memory_cap") or 0), attempt.get("host", ""), task.get("next_host", "") or fixed_host,
                                      maxima, self.environment)
        if not decision.retry:
            maximum = ({"maximum_host": attempt.get("host", ""), "maximum_cap_text": attempt.get("memory_cap_text", "")}
                       if decision.outcome == "memory-cap-at-maximum" else {})
            self.finalize(task, decision.outcome, exit_code, (outcome_preface + self.finished_preface(task, attempt, decision.outcome) + decision.why).rstrip("；"),
                          outcome_counts=counts, **maximum)
            return
        if lib.broken("retry-build-directory-fresh", self.environment) and not lib.broken("retry-build-directory-kept", self.environment):
            # 弄坏开关：照改前重排时当场删这一份，下一次从零编（task_server_lib.py 文件头「编译目录」的重试复用）
            self.remove_build_directory(task, attempt, result)
            self.store.update("attempts", attempt)
        # 别的情形这一份留着：下一次起在同一台就接着用，换了台在起的时候删（start_attempt 的 kept_build_directory）；最后那一次的在记下结局时删
        number = int(attempt.get("number") or 0)
        try:   # 重排在一个事务里判：任务还 running、这一次是它最新那一次、没被取消（取消那一侧改过的不拿旧字典盖掉）
            task = self.store.change("tasks", task["id"], dict(state="queued", next_cap=decision.cap_bytes, next_host=decision.host or "",
                                                                note=at_cap + leftover + decision.why, outcome_counts=counts),
                                     guard=lambda current: ("cancelled" if current.get("cancel_requested") else
                                                            "" if current.get("state") == "running" and int(current.get("attempt_count") or 0) == number else "state"))
        except StoreRefused as refused:
            if refused.kind == "guard" and str(refused) == "cancelled":
                self.finalize(task, "cancelled", exit_code, f"第 {number} 次尝试 {outcome}，重排之前提交它的会话取消了：不再起", outcome_counts=counts)
            elif refused.kind == "tampered":
                self.tampered(task["id"], task, "任务记录")
            return
        say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试 {outcome}：{at_cap}{decision.why}，上限 {lib.mebibytes_text(decision.cap_bytes)} 重排")
        if task.get("role") == "shard" and lib.broken("shard-retry-restarts-all", self.environment):
            for sibling in self.children_of(task["parent_id"]):   # 弄坏开关：成了的兄弟片也重排、再跑一遍
                if sibling.get("role") == "shard" and sibling["id"] != task["id"] and sibling.get("outcome") == "succeeded":
                    self.store.change("tasks", sibling["id"], dict(state="queued", outcome="", exit_code=None, note="弄坏开关 shard-retry-restarts-all：跟着重排"),
                                      reopen=True)
        self.wake()

    def remove_build_directory(self, task, attempt, result):
        """被下一次尝试顶替的那一次的编译目录（文件头「编译目录」）：有进程开着里面的文件就不删、写告警。"""
        directory = attempt.get("build_directory")
        machine = self.machines.get(attempt.get("host")) or self.machines[LOCAL]
        if not directory or attempt.get("build_directory_removed") or not machine.build_directory_exists(directory):   # 起法那一层自己删的（第二台没留着的）记成已删
            return
        removed, lines = machine.remove_build_directory(directory)   # 两台同一段：本机就地删，第二台经 multi-host-run.sh --remove-build-directory
        attempt["build_directory_removed"] = removed
        if not removed:
            self.alert("build-directory-left", task["id"], f"第 {attempt.get('number')} 次尝试的编译目录 {directory} 没删成：" + "；".join(line.strip() for line in lines),
                       machine.removal_howto(directory))

    def kept_build_directory(self, task, number):
        """重排的这一次（第 number 次）起之前：上一次尝试服务在本机建的编译目录还留着（没删、还在）交那一次的尝试记录，没有交 None
        （文件头「编译目录」的重试复用）。第二台那一份带 --keep-build-directory 跑完留着的（multi-host-run.sh 报了留着，build_directory_kept）同样算，
        没留着的跑完就删了（记成已删），不算。在不在照那一台的接口判（Machine.build_directory_exists）。"""
        if number <= 1:
            return None
        for attempt, valid in self.store.attempts_of(task["id"]):
            machine = self.machines.get(attempt.get("host")) or self.machines[LOCAL]
            if (valid and int(attempt.get("number") or 0) == number - 1 and attempt.get("build_directory") and not attempt.get("build_directory_removed")
                    and machine.build_directory_exists(attempt["build_directory"])):
                return attempt
        return None

    def remove_finished_build_directories(self, task):
        """有了结局的这条任务各次尝试留下的编译目录一律删（文件头「编译目录」；第二台留着的经 multi-host-run.sh --remove-build-directory 删）：
        提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的本机那几份留着现场、
        交给提交它的会话用 task clean 删；有进程还开着里面的文件就不删、写告警（同 remove_build_directory）。弄坏开关 finished-build-kept 下照改前留着。"""
        if lib.broken("finished-build-kept", self.environment):
            return
        # 留现场的只留本机那几份（task clean 删的就是它们）；第二台留着的照删：task clean 不连第二台，改前第二台那一份跑完就删
        keep_local = (task.get("environment") or {}).get("SINGLEFS_KEEP_TEST_TEMPORARY_FILES") == "1"
        handled = set()   # 重排接着用的几次尝试记的是同一个目录：删一次，几条记录都记成已删
        for attempt, valid in self.store.attempts_of(task["id"]):
            directory = attempt.get("build_directory")
            machine = self.machines.get(attempt.get("host")) or self.machines[LOCAL]
            if not valid or not directory or attempt.get("build_directory_removed") or (keep_local and machine.name == LOCAL):
                continue
            if (attempt.get("host"), directory) in handled or not machine.build_directory_exists(directory):
                removed, lines = (attempt.get("host"), directory) in handled, []
            else:
                removed, lines = machine.remove_build_directory(directory)   # 两台同一段：本机就地删，第二台经 multi-host-run.sh --remove-build-directory
            if removed:
                handled.add((attempt.get("host"), directory))
                attempt["build_directory_removed"] = True
                self.store.update("attempts", attempt)
            elif not machine.build_directory_exists(directory):
                continue
            else:
                self.alert("build-directory-left", task["id"], f"第 {attempt.get('number')} 次尝试的编译目录 {directory} 没删成：" + "；".join(line.strip() for line in lines),
                           machine.removal_howto(directory))

    def sweep_finished_build_directories(self):
        """服务起来时扫一遍：有了结局的任务还留着的编译目录照 remove_finished_build_directories 删（在后台线程里删，不挡排队）。"""
        for task, valid in self.store.select("tasks", "state = 'finished'"):
            if valid:
                with contextlib.suppress(Exception):   # 一条删不成（记录坏了、文件在用）不挡别的条；没删成的已写告警
                    self.remove_finished_build_directories(task)

    def remove_scenes(self, task_id):
        """删这条任务（分片的连同子任务）各次尝试的现场目录（task_server_lib.py 文件头「现场」）：→ (删了的 [{path, bytes}], 没删成的 [{path, bytes, kind, why}])。"""
        removed, left = [], []
        try:
            owners = [task_id] + [child["id"] for child in self.children_of(task_id)]
        except StoreRefused as refused:
            if refused.kind != "tampered":
                raise
            return [], [{"path": self.scenes_root, "bytes": 0, "kind": "现场", "why": self.children_refused(task_id, refused, "一份现场都不删")}]
        for owner in owners:
            for attempt, _ in self.store.attempts_of(owner):
                scene = attempt.get("scene_directory")
                if not scene or not os.path.lexists(scene):
                    continue
                size = directory_bytes(scene)
                shutil.rmtree(scene, ignore_errors=True)
                if os.path.lexists(scene):
                    left.append({"path": scene, "bytes": size, "kind": "现场", "why": "rm 之后还在（权限不对或有进程在往里写）"})
                else:
                    removed.append({"path": scene, "bytes": size})
        return removed, left

    def scene_due(self, task_id, now):
        """这条任务的现场到没到该删的时候（task_server_lib.py 文件头「现场」）：库里没有了的到了；校验不过的不动；分片的子任务照父任务；
        要提交方 task wait 取走结局满 scene_keep_seconds，提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的只由 task clean 删（弄坏开关 scene-keep-ignored 下照删）。"""
        found = self.store.task(task_id)
        if found is not None and found[1] and found[0].get("parent_id"):
            found = self.store.task(found[0]["parent_id"])
        if found is None:
            return True
        task, valid = found
        if not valid or task.get("state") != "finished" or not task.get("collected"):
            return False
        if (task.get("environment") or {}).get("SINGLEFS_KEEP_TEST_TEMPORARY_FILES") == "1" and not lib.broken("scene-keep-ignored", self.environment):
            return False
        return now - float(task.get("collected_at") or task.get("finished_at") or now) >= self.scene_keep_seconds

    def sweep_scenes(self):
        """排队线程每一轮看一遍现场根（task_server_lib.py 文件头「现场」）：这一个库的标签的那几个目录（task-<库标签>-<任务号>-<第几次>）到了该删的就删；
        别的标签的不动。弄坏开关 scene-kept-after-collect 下一个都不删。"""
        if lib.broken("scene-kept-after-collect", self.environment):
            return
        try:
            names = os.listdir(self.scenes_root)
        except OSError:
            return
        prefix, now = f"task-{self.store.tag}-", time.time()
        for name in names:
            task_text = name[len(prefix):].rsplit("-", 1)[0] if name.startswith(prefix) else ""
            if task_text.isdigit() and self.scene_due(int(task_text), now):
                shutil.rmtree(os.path.join(self.scenes_root, name), ignore_errors=True)

    def finalize(self, task, outcome, attempt_exit, note="", overwrite=False, push_parent=True, **fields):
        """记下这条任务的结局（一个事务里读—判—写，只并进结局那几栏与 fields；已结了局的不再改，弄坏开关 finished-cancel-overwrites 那一支给 overwrite）。
        → 记下了没有。子任务记下之后驱动它的父任务（drive_parent）；正在推父任务的那一步自己撤兄弟片时给 push_parent=False，不再套一层推。"""
        changes = dict(fields, state="finished", outcome=outcome, exit_code=lib.final_exit_code(outcome, attempt_exit), finished_at=round(time.time(), 3), note=note)
        try:
            record = self.store.change("tasks", task["id"], changes,
                                       guard=None if overwrite else (lambda current: "已结了局" if current.get("state") == "finished" else ""))
        except StoreRefused as refused:
            if refused.kind == "tampered":
                self.tampered(task["id"], task, "任务记录")
            elif refused.kind != "guard":
                raise
            return False
        task.clear()
        task.update(record)
        self.forget_prediction(task["id"])   # 收尾就删掉存的那一份预测（文件头 ②「预测每条只算一次」）
        self.notify_finished(task["id"])
        self.remove_finished_build_directories(task)   # 用户：「服务build之后必须删除历史」——记下结局就删它各次尝试的编译目录
        if lib.broken("scene-removed-at-finish", self.environment):   # 弄坏开关：现场照「有结局就删」那一刻删（task_server_lib.py 文件头「现场」）
            self.remove_scenes(task["id"])
        say(f"  · 任务 {task['id']} 结局 {outcome}（尝试 {task.get('attempt_count', 0)} 次{'；' + note if note else ''}）")
        if task.get("parent_id") and push_parent:
            self.crash_point(f"child-finished-{task.get('role')}")
            self.advance_parent(task)
        return True

    def advance_parent(self, child):
        """子任务记下结局之后推它的父任务：照库里父任务的阶段与子任务集合现推（drive_parent）；这一步没走到的（服务没了、抛错）由每一轮对账补推。"""
        self.drive_parent(child["parent_id"])

    # ── ④ 分片（文件头「分片」与 task_server_lib.py 文件头「分片」）──
    def children_of(self, parent_id):
        """父任务名下的子任务记录（照关联列 task_id 挑）。有一行校验不过（关联列与正文对不上、写进之后被改过，TS-05）抛 StoreRefused("tampered")，
        说明里点名那几行：调用方照 record-tampered 办（不认、不动、写告警、回 409），不当它没事。弄坏开关 child-rows-unverified 下照改前丢掉校验位、整份交回。"""
        rows = self.store.select("tasks", "task_id = ?", (parent_id,))
        unverified = [record.get("id") for record, valid in rows if not valid]
        if unverified and not lib.broken("child-rows-unverified", self.environment):
            raise StoreRefused("tampered", f"任务 {parent_id} 名下有校验不过的子任务记录（行 {unverified}，关联列与正文对不上或写进之后被改过）")
        return [record for record, _ in rows]

    def children_refused(self, parent_id, refused, consequence):
        """children_of 判出名下有校验不过的子任务行（refused）：照 record-tampered 的处理写告警（tampered_cancel_refusal 的写法），交回话里那一句
        「<哪几行校验不过>：<consequence>」；调用方不认那几行、不往下做，回 409。"""
        detail = f"{refused}：{consequence}"
        if (parent_id, detail) not in self.alerted_child_refusals:   # 同一件同一句只告警一次（status 每读一次都会走到这里）
            self.alerted_child_refusals.add((parent_id, detail))
            self.alert("record-tampered", parent_id, detail, "查是谁直接写了库（只有 task-server 读写它）；那几行对应的单元与目录要动由人照记录核过再动")
        return detail

    def shard_directory(self, parent_id):
        return os.path.join(lib.shards_root(), f"{self.store.tag}-{parent_id}")

    def find_child(self, children, role, index=0):
        return next((child for child in children if child.get("role") == role and int(child.get("shard_index") or 0) == index), None)

    def create_child(self, parent, role, index=0, count=0, label=""):
        """建这一步的子任务；同一父任务同一步同一片已有的交已有的那一个（库里 (parent_id, role, shard_index) 唯一，TS-02）。计划、各片、合并与估算都不钉台，哪台由服务挑。
        估算那一步（lib.ESTIMATE_ROLE；task_server_lib.py 文件头「资源记录」的预计用时）起 lib.estimate_command，命令键原文是父任务的键接 lib.ESTIMATE_KEY_MARK，不要卡。"""
        existing = self.find_child(self.children_of(parent["id"]), role, index)
        if existing is not None:
            return existing
        estimate = role == lib.ESTIMATE_ROLE
        base = parent["environment"].get(lib.KEY_VARIABLE)
        if not base and lib.broken("shard-key-from-joined-command", self.environment):
            base = " ".join(parent["command"])   # 弄坏开关：照改前用空格接父命令的词，条目名里带空格的后半截漏进键里
        elif not base:
            # 父命令照词折叠（memory_peaks_lib 的 command_key 收词表：带空格的条目名、路径仍是一个词），再接这一步那一段；
            # 截短照这一段的长度先截父任务那一截，接的那一段不被键长上限截掉（截掉了各片落一个键）
            marker = len(f" {lib.ESTIMATE_KEY_MARK}") if estimate else len(lib.shard_child_key("", role, index, count, self.environment))
            base = self.peaks.command_key(lib.key_words(parent["command"], self.environment))[:max(0, self.peaks.KEY_LENGTH_LIMIT - marker)]
        command = lib.estimate_command(parent["command"]) if estimate else parent["command"]
        key_text = f"{base} {lib.ESTIMATE_KEY_MARK}" if estimate else lib.shard_child_key(base, role, index, count, self.environment)
        environment = dict(parent["environment"], **{lib.KEY_VARIABLE: key_text})
        uses_card = bool(parent.get("gpu")) and role not in ("plan", lib.ESTIMATE_ROLE)
        child = {"command": command, "cwd": parent["cwd"], "environment": environment, "session": parent.get("session", ""),
                 "submitter_pid": parent.get("submitter_pid", 0), "submitted_at": round(time.time(), 3),
                 "command_key": self.command_key(command, key_text), "state": "queued", "outcome": "", "exit_code": None,
                 "collected": True, "next_cap": None, "next_host": "", "outcome_counts": {}, "note": "", "cancel_requested": False, "attempt_count": 0,
                 "task_id": parent["id"], "parent_id": parent["id"], "role": role, "shard_index": index, "shard_count": count,
                 "label": label, "compiles": False, "build_family": parent.get("build_family", "") if role in ("plan", "shard") else "",
                 "gpu": uses_card, "gpu_reason": parent.get("gpu_reason", "") if role not in ("plan", lib.ESTIMATE_ROLE) else "",
                 "gpu_cards": int(parent.get("gpu_cards") or 1) if uses_card else 1, "gpu_cards_problem": parent.get("gpu_cards_problem", "") if uses_card else ""}
        try:
            child = self.store.insert("tasks", child)
        except sqlite3.IntegrityError:   # 库里已有同一步同一片：交那一个；按关联列找不到它的（关联列改过，TS-05）当记录改过
            existing = self.find_child(self.children_of(parent["id"]), role, index)
            if existing is None:
                raise StoreRefused("tampered", f"任务 {parent['id']} 的{role} {index} 已在库里、关联列却不指它") from None
            return existing
        step = {"plan": "计划", "merge": "合并", lib.ESTIMATE_ROLE: "估算"}.get(role, f"第 {index}/{count} 片")
        say(f"  · 任务 {parent['id']} 的{step}排进队：子任务 {child['id']}")
        return child

    def finalize_parent(self, parent, child, outcome, child_exit, note=""):
        failed = {}
        if outcome != "succeeded" and child is not None and not lib.broken("shard-failure-unnamed", self.environment):
            failed = {"task": child["id"], "role": child.get("role"), "index": child.get("shard_index", 0), "count": child.get("shard_count", 0),
                      "maximum_host": child.get("maximum_host", ""), "maximum_cap_text": child.get("maximum_cap_text", "")}
        try:
            parent = self.store.change("tasks", parent["id"], dict(state="finished", outcome=outcome, exit_code=lib.final_exit_code(outcome, child_exit),
                                                                   finished_at=round(time.time(), 3), note=note or (child or {}).get("note", ""), failed_step=failed),
                                       guard=lambda current: "已结了局" if current.get("state") == "finished" else "")
        except StoreRefused as refused:
            if refused.kind == "tampered":
                self.tampered(parent["id"], parent, "任务记录")
            elif refused.kind != "guard":
                raise
            return
        self.notify_finished(parent["id"])
        say(f"  · 分片的任务 {parent['id']} 结局 {outcome}{'（' + note + '）' if note else ''}")
        if outcome == "succeeded":   # 整件成了：各片的产物删掉，计划目录留着
            directory = self.shard_directory(parent["id"])
            with contextlib.suppress(OSError):
                for name in os.listdir(directory):
                    if name.startswith("output-"):
                        shutil.rmtree(os.path.join(directory, name), ignore_errors=True)

    def drive_sharded_parents(self):
        """每一轮对账：还没结局的分片父任务与还在估算的父任务各现推一遍（drive_parent；TS-02：子任务收尾与父任务推进之间服务没了、推进半途抛错的，在这里补推）。
        一条推不动（抛错）只写告警、不挡别的。弄坏开关 sharded-parents-unreconciled 下不推（改前的样子：只在子任务收尾那一刻推一次）。"""
        if lib.broken("sharded-parents-unreconciled", self.environment):
            return
        for parent, valid in self.store.select("tasks", "state IN ('sharded', 'estimating')"):
            try:
                self.selftest_advance_fault()
                self.drive_parent(parent["id"])
                self.note_advance_recovered(parent["id"])
            except Exception as error:
                self.note_advance_failure(parent["id"], f"{type(error).__name__}: {error}")   # wait 叫醒提交方（文件头 ①「要处置的提醒」）
                say(f"  ✗ 分片的任务 {parent['id']} 这一轮推不动（{type(error).__name__}: {error}）：下一轮再推\n" + traceback.format_exc())
                self.alert("shard-advance-failed", parent["id"], f"分片的任务 {parent['id']} 推下一步时出错：{type(error).__name__}: {error}",
                           "服务每一轮对账照库里的阶段再推；一直推不动就看 journalctl --user -u singlefs-task-server.service 里这一句前后的栈")

    def drive_parent(self, parent_id):
        """照库里父任务的阶段（phase：plan、shards、merge、cancelling）与它的子任务集合现推下一步（文件头 ④；TS-02）。幂等：缺的子任务照计划补建
        （同一步同一片只一个），该收尾的收尾，已结了局的不动。计划读通之后片的计划（shard_plan）写进父任务，各片照它补建。
        还在估算的父任务（state=estimating）照 drive_item_estimate 推。"""
        found = self.store.task(parent_id)
        if found is None:
            return
        parent, valid = found
        if not valid:
            self.tampered(parent_id, parent, "任务记录")
            return
        if parent.get("state") not in ("sharded", "estimating"):
            return
        try:
            children = self.children_of(parent_id)
        except StoreRefused as refused:
            if refused.kind != "tampered":
                raise
            self.tampered(parent_id, parent, "子任务记录")
            return
        try:
            if parent.get("state") == "estimating":
                self.drive_item_estimate(parent, children)
            else:
                self.drive_parent_phase(parent, children)
        except StoreRefused as refused:
            if refused.kind != "tampered":
                raise
            self.tampered(parent_id, parent, "子任务记录")

    def drive_item_estimate(self, parent, children):
        """还在估算的父任务（state=estimating；task_server_lib.py 文件头「资源记录」的预计用时）：估算子任务还没建的补建（父任务记下了、子任务还没建服务就没了）；
        子任务有了结局父任务改记 queued、照原来的任务号进队：跑成的把它最后一次尝试记下的 item_plan 并进父任务，没成的、或一行 ITEM_PLAN 都没打的 item_plan 记 null，
        照样进队、没有预计用时。取消了的（cancel_estimating 记了 cancelling）子任务都收尾了才记整件 cancelled。
        弄坏开关 estimate-failure-holds-parent 下没成的、没打 ITEM_PLAN 的父任务一直在估算、不进队。"""
        if parent.get("phase") == "cancelling":
            if all(child.get("state") == "finished" for child in children):
                self.finalize_parent(parent, None, "cancelled", None, "提交它的会话在估算时取消了（估算子任务停了，没进队）")
            return
        estimate = self.find_child(children, lib.ESTIMATE_ROLE)
        if estimate is None:
            self.create_child(parent, lib.ESTIMATE_ROLE)
            self.wake()
            return
        if estimate.get("state") != "finished":
            return
        plan = None
        if estimate.get("outcome") == "succeeded":
            rows = self.store.attempts_of(estimate["id"])
            if rows and rows[-1][1]:
                plan = rows[-1][0].get("item_plan") or None
        if plan is None and lib.broken("estimate-failure-holds-parent", self.environment):
            return
        why = "、一行 ITEM_PLAN 都没打" if estimate.get("outcome") == "succeeded" else ""
        note = "" if plan is not None else f"估算子任务 {estimate['id']} {estimate.get('outcome')}{why}：没有按项的预计用时"
        try:
            self.store.change("tasks", parent["id"], {"state": "queued", "phase": "", "item_plan": plan, "estimate_note": note},
                              guard=lambda current: "" if current.get("state") == "estimating" and not current.get("cancel_requested") else "不在估算或已取消")
        except StoreRefused as refused:
            if refused.kind == "tampered":
                raise
            return
        say(f"  · 任务 {parent['id']} 估算完了、进队排（{note or '每项要现判与能复用的条数：' + json.dumps(plan, ensure_ascii=False, sort_keys=True)[:300]}）")
        self.wake()

    def drive_parent_phase(self, parent, children):
        """drive_parent 的那一步（文件头 ④）：照阶段推，读到的子任务集合是 children（都核过）。"""
        parent_id = parent["id"]
        phase = parent.get("phase") or "plan"
        if phase == "cancelling":   # 取消没停干净（TS-04）：子任务都有了结局才记整件 cancelled
            if all(child.get("state") == "finished" for child in children):
                self.finalize_parent(parent, None, "cancelled", None, "提交它的会话取消了整件（在跑的子任务都停了）")
            return
        if phase == "plan":
            plan = self.find_child(children, "plan")
            if plan is None:   # 父任务记下了、计划那一步还没建（TS-02 的「父任务插入后」）
                self.create_child(parent, "plan")
                self.wake()
                return
            if plan.get("state") != "finished":
                return
            if plan.get("outcome") != "succeeded" and not lib.broken("plan-failure-proceeds", self.environment):   # 弄坏开关：计划没成照拆片
                self.finalize_parent(parent, plan, plan.get("outcome") or "launcher-lost", plan.get("exit_code"), f"计划那一步（子任务 {plan['id']}）{plan.get('outcome')}")
                return
            shards, problem = lib.read_shard_plan(os.path.join(self.shard_directory(parent_id), "plan", lib.PLAN_FILE))
            if problem:
                self.finalize_parent(parent, plan, "shard-plan-refused", None, problem)
                return
            try:
                parent = self.store.change("tasks", parent_id, dict(phase="shards", shard_count=len(shards), shard_plan=shards),
                                           guard=lambda current: "" if current.get("state") == "sharded" and (current.get("phase") or "plan") == "plan" else "阶段已变")
            except StoreRefused:
                return
            phase = "shards"
        if phase == "shards":
            planned = parent.get("shard_plan") or []
            created = False
            for index, shard in enumerate(planned):
                if self.find_child(children, "shard", index) is None:
                    children.append(self.create_child(parent, "shard", index, len(planned), label=shard.get("label", "")))
                    created = True
                    self.crash_point("shard-inserted")
            shards = [self.find_child(children, "shard", index) for index in range(len(planned))]
            failed = self.first_failed_shard(shards)
            if failed is not None:
                note = f"第 {failed.get('shard_index')}/{failed.get('shard_count')} 片（子任务 {failed['id']}）{failed.get('outcome')}"
                if lib.broken("shard-siblings-abandoned", self.environment):   # 弄坏开关：照改前父任务当场收尾，在跑的兄弟片没人收
                    self.finalize_parent(parent, failed, failed.get("outcome") or "launcher-lost", failed.get("exit_code"), note)
                    self.cancel_queued_children(shards, "sibling", f"同一件的子任务 {failed['id']} 没成，不起")
                    return
                # 文件头「分片」：在排的兄弟片当场撤掉；在跑的照跑到底、结局照收（判过的块照样入库），都收齐了父任务才记那一片的结局，不合并
                self.cancel_queued_children(shards, "sibling", f"同一件的子任务 {failed['id']} 没成，不起")
                if all(shard.get("state") == "finished" for shard in shards):
                    self.finalize_parent(parent, failed, failed.get("outcome") or "launcher-lost", failed.get("exit_code"), note)
                return
            done = [shard for shard in shards if shard.get("state") == "finished" and shard.get("outcome") == "succeeded"]
            skipped = [shard for shard in shards if self.cancelled_alone(shard)]
            if not (len(done) + len(skipped) == len(shards) or (done and lib.broken("merge-before-all-green", self.environment))):
                if created:
                    self.wake()
                return
            self.write_outputs(parent, done)
            missing = sorted(int(shard.get("shard_index") or 0) for shard in skipped)
            try:
                parent = self.store.change("tasks", parent_id, {"phase": "merge", "missing_shards": missing},
                                           guard=lambda current: "" if current.get("state") == "sharded" and current.get("phase") == "shards" else "阶段已变")
            except StoreRefused:
                return
            self.crash_point("merge-outputs-written")
            phase = "merge"
        if phase == "merge":
            merge = self.find_child(children, "merge")
            if merge is None:   # 进了合并那一步、合并的子任务还没建（outputs.tsv 照各片最后一次尝试重写）
                if not os.path.exists(os.path.join(self.shard_directory(parent_id), "outputs.tsv")):
                    self.write_outputs(parent, [child for child in children if child.get("role") == "shard" and child.get("outcome") == "succeeded"])
                self.create_child(parent, "merge", 0, parent.get("shard_count", 0))
                self.crash_point("merge-inserted")
                self.wake()
                return
            if merge.get("state") != "finished":
                return
            if merge.get("outcome") == "succeeded" or lib.broken("merge-failure-succeeds", self.environment):   # 弄坏开关：合并没成照记成了
                self.finalize_parent(parent, merge, "succeeded", merge.get("exit_code"))
            else:
                self.finalize_parent(parent, merge, merge.get("outcome") or "launcher-lost", merge.get("exit_code"), f"合并那一步（子任务 {merge['id']}）{merge.get('outcome')}")

    @staticmethod
    def cancelled_alone(shard):
        """人手单独取消的那一片（cancel 记的 cancel_origin=session，结局 cancelled）：不算没成，合并照起、点名缺它（文件头「分片」）。"""
        return shard.get("state") == "finished" and shard.get("outcome") == "cancelled" and shard.get("cancel_origin") == "session"

    def first_failed_shard(self, shards):
        """最先没成的那一片：有了结局、不是 succeeded，人手单独取消的与兄弟片没成被服务撤掉的（cancel_origin=sibling）不算；照记下结局的先后、同一刻取片号小的。没有交 None。"""
        failed = [shard for shard in shards if shard.get("state") == "finished" and shard.get("outcome") != "succeeded" and not self.cancelled_alone(shard)
                  and not (shard.get("outcome") == "cancelled" and shard.get("cancel_origin") == "sibling")]
        return min(failed, key=lambda shard: (float(shard.get("finished_at") or 0), int(shard.get("shard_index") or 0)), default=None)

    def cancel_queued_children(self, children, origin, note):
        """撤掉 children 里还在排的那几条（兄弟片没成：origin=sibling；弄坏开关 parent-cancel-spares-running 下整件被取消也走这里）：拿那一条的锁、
        在一个事务里判还 queued 才记 cancel_requested 与 cancel_origin（起单元那一侧拿同一把锁、claim_attempt 判 cancel_requested，记上之后就不会再起），
        再记 cancelled；判的时候已经起了的不动，照跑到底。记了的那一份字典就地换成库里的新样子。"""
        for child in children:
            if child.get("state") != "queued":
                continue
            with self.store.task_lock(child["id"]):
                try:
                    current = self.store.change("tasks", child["id"], {"cancel_requested": True, "cancel_origin": origin},
                                                guard=lambda row: "" if row.get("state") == "queued" else "不在排")
                except StoreRefused as refused:
                    if refused.kind == "tampered":
                        raise
                    continue
                if self.finalize(current, "cancelled", None, note, push_parent=False):
                    child.clear()
                    child.update(current)

    def write_outputs(self, parent, shards):
        """合并那一步（文件头「分片」③）读的 outputs.tsv：各片最后一次尝试的产物目录（先写临时文件再改名，重写也是同一份）。"""
        lines = []
        for shard in sorted(shards, key=lambda record: record.get("shard_index", 0)):
            attempts = [attempt for attempt, _ in self.store.attempts_of(shard["id"])]
            last = attempts[-1] if attempts else {}
            lines.append(f"{shard.get('shard_index', 0)}\t{last.get('host', '')}\t{last.get('shard_output', '')}")
        directory = self.shard_directory(parent["id"])
        lib.ensure_private_directory(directory)
        outputs = os.path.join(directory, "outputs.tsv")
        temporary = outputs + ".partial"
        with open(temporary, "w", encoding="utf-8") as handle:
            handle.write("".join(line + "\n" for line in lines))
        os.replace(temporary, outputs)

    # ── ② 排队 ──
    def busy_cards(self):
        busy = set()
        for attempt, _ in self.store.select("attempts", "state IN ('starting', 'running')"):
            busy.update(card_keys(attempt.get("card")))   # 一条分到同一台几张的，几张都占着（文件头 ②「GPU」的张数）
        return busy

    def whole_threshold(self, host, snapshot):
        """整片的界（字节）：这台可用上限（总上限减这台此刻的余量，host_capacity；booked_at_cap 收到的那个数）× WHOLE_FRACTION（文件头 ②「整片与碎片按台交替」）。
        只对这一轮派得了的台调（place、Alternation 先滤掉了余量量不到的台）；量不到的抛 ValueError，不退回别的数。"""
        capacity, why = self.host_capacity(host, snapshot)
        if capacity is None:
            raise ValueError(f"整片的界算不了：{why}")
        return int(capacity * WHOLE_FRACTION)

    def is_whole(self, booked, host, snapshot):
        return int(booked or 0) >= self.whole_threshold(host, snapshot)

    def aged(self, task, now):
        """提交满 aged_seconds 还在排：高优先（文件头「排队」）。"""
        return now - float(task.get("submitted_at") or now) >= self.aged_seconds and not lib.broken("aged-task-not-prioritized", self.environment)

    def queue_order(self, queued, now):
        """排队的先后：高优先的在前，各自按原来的次序（排序是稳定的）。"""
        return sorted(queued, key=lambda row: not self.aged(row[0], now))

    def head_judged_apart(self, pass_state=None):
        """只给弄坏开关 health-head-judged-apart 与 health-judges-head：照改前 /health 自己另判一套排头放不放得下，交 HeadVerdict（队空交 None）。
        另判的只看内存（judged_on_host）与那台有没有一张空卡，不看 CPU 闸、编译目录占盘、卡的张数与交替（改前的样子：任务 7568 卡在 CPU 闸、7615 要同一台 3 张卡、
        2365 等一件 /health 不认得的事，都被报成放得下却没起）。pass_state 给了照它取排头、用它的余量；没给（health-judges-head 下 /health 现调）就现读库与余量。"""
        if pass_state is None:
            queued = [row for row in self.store.select("tasks", "state = 'queued'", order="id") if row[1]]
            view = None
        else:
            queued = sorted((row for row in pass_state["queued"] if row[1] and row[0]["id"] not in pass_state["gone"]), key=lambda row: row[0]["id"])
            view = pass_state["view"]
        if not queued:
            return None
        head = self.queue_order(queued, time.time())[0][0]
        gpu = bool(head.get("gpu"))
        view = view or PassView(self, "health")
        snapshot = view.snapshot(gpu)
        if snapshot.hold:
            return HeadVerdict(head["id"], HEAD_BLOCKED, "另判：自证的假余量写了 hold")
        prediction = self.prediction_of(head)
        busy = self.busy_cards()
        for host in self.candidate_hosts(head, snapshot):
            if host not in snapshot.reports or self.host_limit(prediction, host, snapshot)[0] is None:
                continue   # 余量量不到的台这一轮不派（文件头 ②「余量」）
            pending_memory, pending_gpu = view.pending_of(host)
            verdict = self.judged_on_host(head, prediction, host, snapshot, gpu, pending_memory, pending_gpu)[1]
            free_cards = sum(1 for card in snapshot.cards if card.side == host and (card.address, card.index) not in busy)
            if verdict.fits and (not gpu or free_cards):
                return HeadVerdict(head["id"], HEAD_NOT_STARTED, "另判：内存放得下")
        return HeadVerdict(head["id"], HEAD_BLOCKED, "另判：放不下")

    def starved_problem(self, now, fresh=False):
        """派活判排头能起却连着 starved_seconds 没起成：交一句问题，不然交空串。只读派活这一轮给排头的结论（head_verdict，place_queued 记下），不另判放不放得下：
        派活判不能起的（等卡、等负载、等占盘、交替封组、第二台这一轮起满、起之前复核没过，原因都在排队原因里）与判不了的一律不报，不认排队原因里的字样。
        服务暂停着（文件头 ⑤「暂停」）排着不起是要的，交空串；弄坏开关 pause-judges-starved 下照改前另判（暂停时排队线程不判，没有结论可读）。
        弄坏开关 health-head-judged-apart 下照改前另判一套（head_judged_apart，用这一轮的余量）；health-judges-head 下由 /health 现调、fresh 为真：另判一套且现读库与余量；
        blocked-head-counted-as-starved 下派活判不能起的排头也当成判能起没起成。"""
        if self.pause is not None:
            if not lib.broken("pause-judges-starved", self.environment):
                self.fits_since = {}
                return ""
            fresh = True
        if fresh or lib.broken("health-head-judged-apart", self.environment):
            try:
                verdict = self.head_judged_apart(None if fresh else self.pass_state)
            except (OSError, ValueError, KeyError, subprocess.SubprocessError):
                return ""
        else:
            verdict = self.head_verdict
        startable = verdict is not None and (verdict.kind == HEAD_NOT_STARTED
                                             or (verdict.kind == HEAD_BLOCKED and lib.broken("blocked-head-counted-as-starved", self.environment)))
        if not startable:
            self.fits_since = {}
            return ""
        since = self.fits_since.get(verdict.task_id, now)
        self.fits_since = {verdict.task_id: since}
        if now - since > self.starved_seconds and not lib.broken("starved-head-unreported", self.environment):
            return f"任务 {verdict.task_id} 排在最前、派活判它此刻放得下，却 {now - since:.0f} 秒没起成（{verdict.why}）"
        return ""

    def progress_problem(self, now):
        """看进展（文件头「接口」；派活判法自己判错时要有人看得到：队列卡死 4 个多小时、51 条排着只跑 3 条那一次）：有排着的任务、某台此刻照用户定的公式
        （room_by_the_formula）放得下排着的最小那条的记账，而这台连着 progress_seconds 一条都没起，交一句点名哪台、照公式还能记多少、最小那条是哪条；不然交空串。
        不读派活的结论（head_verdict、排队原因、Placement），只读这一轮排队读的余量（PassView，不另读）、库里在排的任务与起没起（progress_started_hosts）。
        派活本来就该等的几种各自照余量另外不算（progress_smallest_fitting）。服务暂停着不判（排着不起是要的），判不了交空串。
        弄坏开关 progress-unwatched 下一律交空串。"""
        try:
            started = self.progress_started_hosts()
            if self.pause is not None or lib.broken("progress-unwatched", self.environment):
                self.progress_since = {}
                return ""
            for host in started:
                self.progress_since.pop(host, None)   # 这台这一轮起过：从此刻重新算
            view = (self.pass_state or {}).get("view")
            queued = [task for task, valid in self.store.select("tasks", "state = 'queued'", order="id") if valid] if view is not None else []
            if not queued:
                self.progress_since = {}
                return ""
            human = self.host_dispatch.human
            problems = []
            for host in MACHINE_ORDER:
                fitting = self.progress_smallest_fitting(host, queued, view)
                if fitting is None:
                    self.progress_since.pop(host, None)
                    continue
                since = self.progress_since.setdefault(host, now)
                if now - since >= self.progress_seconds:
                    room, task_id, booked = fitting
                    problems.append(f"{self.machines[host].label}照公式（配置的总上限减余量减在跑的上限之和）还能记 {human(room)}、排着的最小那条任务 {task_id}（记 {human(booked)}）放得下，"
                                    f"这台却 {now - since:.0f} 秒一条都没起（看进展，不看派活的结论）")
            return "；".join(problems)
        except (OSError, ValueError, KeyError, subprocess.SubprocessError, sqlite3.Error, StoreRefused):
            return ""

    def progress_started_hosts(self):
        """看进展：尝试号比上一轮数到的最大号大的那几次尝试起在哪几台（起了又收尾的也数得到），交台的集合；头一回调只记下此刻的最大号、交空集。"""
        seen = self.progress_attempt_seen
        top, hosts = seen or 0, set()
        for attempt, _ in self.store.select("attempts", "id > ?", (seen if seen is not None else 0,)):
            top = max(top, int(attempt["id"]))
            if seen is not None:
                hosts.add(attempt.get("host"))
        self.progress_attempt_seen = top
        return hosts

    def progress_smallest_fitting(self, host, queued, view):
        """看进展的一台：照公式这台还能记的（room_by_the_formula）放得下排着的最小那条的记账，交 (还能记的, 那条的任务号, 它在这台的记账)，放不下或不判交 None。
        不判的：自证的 hold、这台余量量不到或连不上；这台封着组而服务在这台还有在跑的（文件头 ②「整片与碎片按台交替」：碎片照用户定的交替等腾空）。
        挑最小那条时跳过派不到这台的、这台给不出上限的，与本来就该等的：要卡的这台空卡凑不够它的张数、这台负载过 CPU 闸（cpu_problem）、编译目录的盘不够（build_disk_problem）。
        弄坏开关 progress-gates-ignored 下照公式放得下就算：不看卡、CPU、占盘与封组。"""
        snapshot = view.snapshot(False)
        if snapshot.hold or host not in snapshot.reports or not snapshot.reachable.get(host):
            return None
        room = self.room_by_the_formula(host, snapshot)
        if room is None:
            return None
        gated = not lib.broken("progress-gates-ignored", self.environment)
        if gated and self.fragment_groups.get(host, {}).get("sealed") and any(
                attempt.get("host") == host for attempt, _ in self.store.select("attempts", "state IN ('starting', 'running')")):
            return None
        busy = self.busy_cards()
        smallest = None
        for task in queued:
            uses_card = bool(task.get("gpu"))
            task_snapshot = view.snapshot(uses_card)
            if host not in task_snapshot.reports or host not in self.candidate_hosts(task, task_snapshot):
                continue
            prediction = self.prediction_of(task)
            if self.host_limit(prediction, host, task_snapshot)[0] is None:
                continue
            if gated:
                free_cards = sum(1 for card in task_snapshot.cards if card.side == host and (card.address, card.index) not in busy)
                if uses_card and free_cards < int(task.get("gpu_cards") or 1):
                    continue   # 等卡
                if self.cpu_problem(task, host, task_snapshot) or self.build_disk_problem(task, host, task_snapshot, view):
                    continue   # 等负载、等占盘
            booked = int(self.booked_at_cap(prediction, host, task_snapshot).memory_booked or 0)
            if smallest is None or booked < smallest[1]:
                smallest = (task["id"], booked)
        if smallest is None or smallest[1] > room:
            return None
        return room, smallest[0], smallest[1]

    def place_queued(self):
        """排一轮；排队原因写进一张新表、这一轮排完才换上（status 读不到排了一半的表）。派活给这一轮每条的结论（HeadVerdict）同样记下，
        排头（这一轮头一条判了的：排队的先后，收尾了的、记录坏了的不算）那一条换进 head_verdict，/health 判「判能起却没起成」只读它。"""
        waiting, verdicts = {}, {}
        try:
            self.place_queued_into(waiting, verdicts)
        finally:
            self.waiting_reasons = waiting
            self.head_verdict = next(iter(verdicts.values()), None)

    def place_queued_into(self, waiting, verdicts):
        """排一轮的本体：waiting 收每条排着的在等什么，verdicts 照排队的先后收派活给每条的结论（{任务号: HeadVerdict}，每条只记头一个）。
        自证的 TASK_SERVER_SELFTEST_PLACEMENT_PAUSED 那份文件在时照判、判能起的不起（记判能起却没起成）；TASK_SERVER_SELFTEST_PLACEMENT_REFUSED 那份文件在时
        每条都判成不能起（演派活判法判错，看进展那一道该报）。"""
        self.pass_state = None   # 这一轮没排就是 None

        def judged(task, kind, why=""):
            verdicts.setdefault(task["id"], HeadVerdict(task["id"], kind, why))

        pause = self.pause_in_force()
        queued = self.store.select("tasks", "state = 'queued'", order="id DESC" if lib.broken("queue-reversed", self.environment) else "id")
        if pause is not None:   # 服务暂停着（文件头 ⑤「暂停」）：只排放行的会话交的，别的会话的这一轮不起；排着的任务在等什么由 waiting_text 照暂停写
            queued = [row for row in queued if self.allowed_during_pause(row[0], pause)]
            if not queued:
                return
        if not queued:
            self.pass_state = {"queued": [], "gone": set(), "view": None}
            self.selftest_read("pass view=- queued=0")   # 队空的一轮也记（自证的 next_pass 等它）
            return
        now = time.time()
        self.repass_soon = False
        started_on = {}   # {台: 这一轮在那台真起了几条（尝试在 starting 或 running）}：有的台一轮至多起 Machine.starts_per_pass 条
        queued = self.queue_order(queued, now)
        self.keep_predictions_of({task["id"] for task, _ in queued})   # 不在队里了的（起了、收尾了）删掉存的预测
        view = PassView(self, "pass")   # 这一轮的余量：每台读一次、各条共用（文件头 ②「余量一轮只读一次」）
        gone = set()   # 这一轮起了或收尾了的任务号（判排头时跳过它们）
        self.pass_state = {"queued": queued, "gone": gone, "view": view}
        self.selftest_read(f"pass view={view.number} queued={len(queued)}")
        busy = self.busy_cards()
        gpu_waiting_hosts = set()   # 前面等卡的任务要的卡在哪几台（够它张数的台）：后面要卡的只在这几台上让它，别台照起（任务 7615 要第二台 3 张卡在等，7618 要 1 张也被挡住）
        alternation = None if lib.broken("alternation-disabled", self.environment) else Alternation(self, view, queued, now)   # 文件头 ②「整片与碎片按台交替」

        def behind(position, why):
            for later, _ in queued[position + 1:]:
                waiting[later["id"]] = why

        for position, (task, valid) in enumerate(queued):
            if not valid:
                self.tampered(task["id"], task, "任务记录")
                gone.add(task["id"])
                continue
            if task.get("gpu") and gpu_waiting_hosts and lib.broken("card-wait-blocks-later-cards", self.environment):
                waiting[task["id"]] = "等空卡：前面有要卡的任务在等卡"
                judged(task, HEAD_BLOCKED, waiting[task["id"]])
                continue   # 弄坏开关：照改前前面有一条等卡就挡住后面全部要卡的
            try:
                if self.placement_refused and os.path.exists(self.placement_refused):
                    placement = Placement("blocked", why="自证：TASK_SERVER_SELFTEST_PLACEMENT_REFUSED 那份文件在，派活一律判不能起（演派活判法判错）")
                else:
                    placement = self.place(task, busy, view, None if self.aged(task, now) else alternation)
            except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
                say(f"  ! 任务 {task['id']} 这一轮判不了放不放得下（{type(error).__name__}: {error}），留在队里")
                waiting[task["id"]] = f"这一轮判不了放不放得下（{type(error).__name__}: {error}），下一轮再判"
                judged(task, HEAD_BLOCKED, waiting[task["id"]])
                if lib.broken("judge-error-blocks-queue", self.environment):
                    behind(position, f"排在任务 {task['id']} 后面：它这一轮判不了放不放得下")
                    return   # 弄坏开关：照改前一条判不了就挡住整队
                continue   # 只跳过它这一轮，后面的照判（一条记录坏了不该把整队挡住）
            if placement.kind == "never":
                self.finalize(task, "never-fits", None, placement.why)
                gone.add(task["id"])
                continue
            if placement.kind == "unmet":
                self.finalize(task, "run-condition-unmet", None, placement.why)
                gone.add(task["id"])
                continue
            if placement.kind == "blocked":
                waiting[task["id"]] = placement.why
                judged(task, HEAD_BLOCKED, placement.why)
                if task.get("gpu") and placement.waiting_for_card:
                    if lib.broken("card-wait-blocks-all", self.environment):
                        behind(position, f"排在任务 {task['id']} 后面：它在等空卡")
                        return   # 弄坏开关：等卡的把后面不要卡的也挡住
                    wanted = task.get("gpu_cards") or 1
                    counts = {}
                    for card in view.snapshot(True).cards:
                        counts[card.side] = counts.get(card.side, 0) + 1
                    gpu_waiting_hosts |= {host for host, count in counts.items() if count >= wanted} or set(counts)
                    continue
                if lib.broken("backfill-disabled", self.environment):
                    behind(position, f"排在任务 {task['id']} 后面")
                    return   # 弄坏开关：退回严格排队，排头放不下后面的都等
                continue
            if task.get("gpu") and placement.host in gpu_waiting_hosts:
                waiting[task["id"]] = "等空卡：前面有要卡的任务在等这台的卡"
                judged(task, HEAD_BLOCKED, waiting[task["id"]])
                continue   # 前面等卡的那条要的就是这台的卡：不抢；落在别台的照起
            machine = self.machines[placement.host]
            if (machine.starts_per_pass is not None and started_on.get(placement.host, 0) >= machine.starts_per_pass
                    and not lib.broken("peer-starts-unthrottled", self.environment)):
                waiting[task["id"]] = f"{machine.label}这一轮已起 {machine.starts_per_pass} 条（一齐起太多 ssh 会被重置），{PEER_REPASS_SECONDS:.0f} 秒后再排"
                judged(task, HEAD_BLOCKED, waiting[task["id"]])
                self.repass_soon = True
                continue
            if self.placement_paused and os.path.exists(self.placement_paused):
                judged(task, HEAD_NOT_STARTED, "自证：TASK_SERVER_SELFTEST_PLACEMENT_PAUSED 那份文件在，排队线程判完不起（起之前复核也不做）")
                continue   # 自证：照判、不起
            placement, why = self.launch_recheck(task, placement, view)
            if placement is None:   # 起之前现读的数放不下：留在队里（文件头 ②「起之前复核」）
                waiting[task["id"]] = f"起之前复核没过（现算的预测、现读的余量）：{why}"
                judged(task, HEAD_BLOCKED, waiting[task["id"]])
                continue
            attempt, held = self.start_attempt_unless_paused(task, placement)
            if held:
                return   # 判完放得下、起之前服务暂停了（pause 与这一处共锁）：这一轮到此为止，一条都不再起
            gone.add(task["id"])
            self.selftest_read(f"start task={task['id']} host={placement.host} view={view.number}")   # 自证：哪一轮在哪台起的（peer-starts-throttled 照它数每轮起几条）
            if attempt is not None and attempt.get("state") in ACTIVE_ATTEMPT_STATES:
                judged(task, HEAD_STARTED)
                view.book(placement.host, placement.booked, placement.gpu_booked, placement.disk_booked)   # 这一轮后面的照加过的判（文件头 ②「余量一轮只读一次」）
                if alternation is not None:
                    alternation.started(placement.host, task, placement.booked, view.snapshot(bool(task.get("gpu"))))
                started_on[placement.host] = started_on.get(placement.host, 0) + 1   # 只数真起了的：没起成的（取消了、systemd-run 起不来）不占这一轮的名额
            else:
                judged(task, HEAD_NOT_STARTED, f"起尝试没成：{'没记下尝试' if attempt is None else '尝试 ' + str(attempt.get('state')) + '、' + str(attempt.get('outcome'))}")
                if lib.broken("failed-start-counted", self.environment):   # 弄坏开关：照改前没起成的也算进这一轮在那台起了几条
                    started_on[placement.host] = started_on.get(placement.host, 0) + 1
            busy.update(card_keys(placement.card))

    def host_eligible(self, task, host, snapshot):
        """这一条派不派得到这一台（task_server_lib.py 文件头「挑机器」）：只看这一台此刻连不连得上（Snapshot.reachable），余量由 place 判；共享根外的路径、共享根、
        判定库三种在提交时就拒了（文件头 ②「提交时拒」），这里不再判。没有只派本机的名单：哪个脚本派到第二台跑红了由脚本那一侧修。
        第二台在同一棵共享的树里、同一个绝对路径上跑：改主工作区的工具照样派得过去。"""
        if not snapshot.reachable.get(host):
            return False
        if self.machines[host].always_reachable:
            return True   # 下面两个弄坏开关照改前只派服务所在这一台
        if self.restored_local_only(task):
            return False   # 弄坏开关 host-lists-restored：照改前点到只在本机的名单的不派第二台
        if lib.broken("workspace-writers-local-only", self.environment) and self.modifies_workspace(task):
            return False   # 弄坏开关：照改前把改主工作区的工具只派本机
        return True

    def restored_local_only(self, task):
        """弄坏开关 host-lists-restored 下：命令点到删之前只在本机的那几张名单（RESTORED_HOST_LISTS 的目录与入口；bash -c 这类整条写在一个词里的按词里出现的目录认）交 True；
        开关没开一律 False。"""
        if not lib.broken("host-lists-restored", self.environment):
            return False
        root = os.path.realpath(self.peer_root)
        cwd = os.path.realpath(task["cwd"])
        directories = RESTORED_HOST_LISTS["local_directories"]
        prefixes = [os.path.join(root, directory) + os.sep for directory in directories]
        if (any(os.path.normpath(os.path.join(cwd, word)).startswith(prefix) for word in task["command"] for prefix in prefixes)
                or any(re.search(r"(?:^|[\s\x27\x22=;|&(<>/])" + re.escape(directory) + "/", word) for word in task["command"] for directory in directories if " " in word)):
            return True
        relative = [os.path.relpath(os.path.normpath(os.path.join(cwd, word)), root) for word in task["command"] if not word.startswith("-")]
        return any(path in RESTORED_HOST_LISTS["local_entries"] for path in relative)

    def modifies_workspace(self, task):
        """这条命令改不改主工作区（改了跑第二遍结果就不同，撞上限不原样重排）：不带 --skip-apply 的 merge-and-verify、WORKSPACE_WRITERS 里的工具；
        hook-swap.py 带 --feed-only 的只喂不换、不写文件，不算（任务 2461：喂钩子撞了 44M 被当成改工作区打回）。弄坏开关 feed-only-counted-as-writer 下照算。
        带了 WORKSPACE_READ_ONLY_ARGUMENTS 之一的只读不写，不算（apply-writer-patch.py --dry-run 只在临时树上核：任务 7442 撞 476M、7444 撞 4096M，都被当成改工作区打回；
        自证、列项、读结局同理：任务 7543、7549）。弄坏开关 read-only-arguments-counted-as-writer 下照算。"""
        root = os.path.realpath(self.peer_root)
        cwd = os.path.realpath(task.get("cwd") or root)
        relative = [os.path.relpath(os.path.normpath(os.path.join(cwd, word)), root) for word in task["command"] if not word.startswith("-")]
        if (any(argument in WORKSPACE_READ_ONLY_ARGUMENTS for argument in task["command"])
                and not lib.broken("read-only-arguments-counted-as-writer", self.environment)):
            return False
        if PATCH_APPLYING_ENTRY in relative and "--skip-apply" not in task["command"]:
            return True
        if HOOK_SWAP_ENTRY in relative and "--feed-only" in task["command"] and not lib.broken("feed-only-counted-as-writer", self.environment):
            return False
        return any(path in WORKSPACE_WRITERS for path in relative)

    def placement_refusal(self, command, cwd, environment):
        """提交时拒（文件头 ②「提交时拒」）：依次判 ① 共享根外的路径、② 共享根、③ 判定库、④ 临时 prune-server 包装里会编或要卡的，
        命中头一种交 {problem: 那一句, next_step: 出路}，都没命中交 None。
        environment 是这一条带进任务的环境（split_environment 收下的那几个）。四个弄坏开关各关一种。"""
        if not lib.broken("outside-path-accepted", self.environment):
            refusal = self.find_outside_shared_path(command, cwd)
            if refusal:
                return refusal
        if not lib.broken("shared-roots-outside-accepted", self.environment):
            refusal = self.shared_roots_problem(environment)
            if refusal:
                return refusal
        if not lib.broken("verdict-library-family-accepted", self.environment):
            refusal = self.verdict_library_refusal(command, cwd)
            if refusal:
                return refusal
        if not lib.broken("temporary-service-heavy-accepted", self.environment):
            return self.temporary_service_heavy_refusal(command, cwd, environment)
        return None

    def temporary_service_heavy_refusal(self, command, cwd, environment):
        """④ 临时 prune-server 包装里会编或要卡的（文件头 ②「提交时拒」④）：剥不开（不是这层包装）的交 None；剥出的原命令照 compiles 有会编的 cargo、
        或照 gpu_need 要卡的，交 {problem, next_step}（点名哪一种与原命令），都不是交 None。剥开只用来判，收下的照旧整条执行。"""
        words = lib.unwrapped_temporary_service(command, self.environment)
        if words == list(command):
            return None
        inner = {"command": words, "cwd": cwd, "environment": environment}
        kinds = []
        if self.compiles(inner):
            kinds.append("有会编的 cargo")
        needs_card, reason = self.gpu_need(inner)
        if needs_card:
            kinds.append(f"要卡（{reason}）")
        if not kinds:
            return None
        return {"problem": (f"临时 prune-server 的包装 {lib.TEMPORARY_SERVICE_SCRIPT} {lib.TEMPORARY_SERVICE_FLAG} 里套的原命令{'、'.join(kinds)}："
                            "这层包装只给脚本自证用，服务只照外层那一条给编译目录、分卡与清场，包装里的认不进来；这一条不收、不跑"),
                "next_step": (f"去掉包装重交：task-server/task submit -- {shlex.join(words)}"
                              "（会编或要卡的命令对真服务判，验证包的 cargo test 不套这层包装）")}

    def find_outside_shared_path(self, command, cwd):
        """① 共享根外的路径（文件头 ②「提交时拒」）：读词与 scripts/hosts/multi-host-run.sh 的 multi_host_outside_shared_path 相同：整个词是绝对路径的、「名字=绝对路径」的值、
        词里有空白的（bash -c 那一段）里每一段绝对路径（QUOTED_WORD_PATH；此刻在的、整个词就是一个带空白的路径的照整个词）；两台相同的系统路径（HOST_INDEPENDENT_PATHS）不判。
        相对路径照工作目录解，工作目录在共享根下它们也在。→ 头一个不在的 {problem, next_step}，都在交 None；落在 VOLATILE_ROOT 下的另说一句重启会清。
        紧跟在 ${…}、$(…) 后面的 /… 是展开之后的尾巴（编译目录这类运行时才给的），不当绝对路径；弄坏开关 expansion-tail-as-path 下照改前也当。
        弄坏开关 quoted-word-paths-ignored 下不认词里的路径（改前的样子）。"""
        found = ""
        real_cwd = os.path.realpath(cwd)
        if not self.containing_shared_root(real_cwd):
            found, named = cwd, f"工作目录 {real_cwd}"
        else:
            for word in command:
                if any(character.isspace() for character in word) and not (word.startswith("/") and os.path.lexists(word)):
                    candidates = ([] if lib.broken("quoted-word-paths-ignored", self.environment)
                                  else (QUOTED_WORD_PATH_BEFORE_EXPANSION_TAILS if lib.broken("expansion-tail-as-path", self.environment) else QUOTED_WORD_PATH).findall(word))
                else:
                    value = word if word.startswith("/") else word.partition("=")[2]
                    candidates = [value] if value.startswith("/") else []
                found = next((path for path in candidates
                              if not any(path == entry or (entry.endswith("/") and path.startswith(entry)) for entry in HOST_INDEPENDENT_PATHS)
                              and not self.containing_shared_root(os.path.realpath(path))), "")
                if found:
                    named = f"命令里点到的 {found}"
                    break
        if not found:
            return None
        roots = "、".join(self.shared_roots) or "一个根都没写"
        volatile = any(os.path.normpath(path) == VOLATILE_ROOT or os.path.normpath(path).startswith(VOLATILE_ROOT + os.sep) for path in (found, os.path.realpath(found)))
        lost = (f"；它落在 {VOLATILE_ROOT} 下：重启会清掉 {VOLATILE_ROOT}，任务做完了、重启之后那里的产物就没了" if volatile else "")
        return {"problem": (f"{named} 不在多机配置 {lib.NFS_ROOTS_KEY} 的任何一段下（{roots}）：任务两台都要跑得起，第二台照同一个绝对路径读写，"
                            f"读不到本机那一份、写出的本机也读不到{lost}；这一条不收、不跑"),
                "next_step": f"把它改放 {lib.NFS_ROOTS_KEY} 某一段下（{roots}；要留的产物放共享工作根 {self.work_root} 下），再提交"}

    def verdict_library_refusal(self, command, cwd):
        """③ 判定库（文件头 ②「提交时拒」）：带不带判定库只照登记表 .claude/gate.d/verdict-library-scripts.tsv 判（registry_families），不另问别的脚本：
        命令里（临时 prune-server 的包装剥开、照原命令认）有一个词点到登记表里带判定库的一行（第三列不以「不算：」「无判定库：」起头），而那一行第三列不以 prune-server 起头的，
        交 {problem, next_step}，点名那一族（登记的路径）；登记表读不了、写坏了的同样拒（判不了）。都不是交 None。
        词的认法（registry_family_of）：整个词，词里有空白的（bash -c 那一段）按空白拆开的每一段；照写的、照工作目录解开的、解开符号链接的，等于登记的路径或以「/登记的路径」收尾。
        弄坏开关 registry-service-column-ignored 下第三列写 prune-server 的也拒。"""
        families, problem = self.registry_families()
        if problem:
            return {"problem": f"判不了这条命令带不带判定库：{problem}；这一条不收、不跑",
                    "next_step": f"把登记表 {VERDICT_LIBRARY_REGISTRY} 修好（列的写法在它文件头），再提交"}
        words = lib.unwrapped_temporary_service(command, self.environment)
        found = self.registry_family_of(words, cwd, families)
        if not found:
            return None
        if families[found].startswith(REGISTRY_SERVICE_MARK) and not lib.broken("registry-service-column-ignored", self.environment):
            return None
        return {"problem": (f"这条命令带判定库，那一族是登记表 {VERDICT_LIBRARY_REGISTRY} 的 {found}，第三列还没写 {REGISTRY_SERVICE_MARK}："
                            "判定写在按台的库里，派到哪台就落在哪台，两台读不到同一份；这一条不收、不跑"),
                "next_step": (f"把 {found} 那一族的判定库读写接进 prune-server（.claude/rules/verification.md「判定库只有一份」），"
                              f"再把登记表那一行第三列翻成 {REGISTRY_SERVICE_MARK}，然后提交")}

    def registry_family_of(self, words, cwd, families):
        """words 里头一个点到登记表带判定库那一行的词：交登记的路径，没有交空串（认法见 verdict_library_refusal）。展开不了的词（带 $ 或反引号）不认。"""
        root = os.path.realpath(self.peer_root)
        library = [path for path, where in families.items() if not where.startswith(REGISTRY_NO_LIBRARY_PREFIXES)]
        for word in words:
            for piece in (word.split() if any(character.isspace() for character in word) else [word]):
                piece = piece.strip("'\"")
                if not piece or piece.startswith("-") or "$" in piece or "`" in piece:
                    continue
                piece = piece.partition("=")[2] if "=" in piece and not piece.startswith(("/", ".")) else piece
                if "/" not in piece:
                    continue
                resolved = os.path.normpath(os.path.join(cwd, os.path.expanduser(piece)))
                candidates = {os.path.normpath(piece), resolved, os.path.realpath(resolved)}
                for path in library:
                    registered = os.path.join(root, path)
                    if any(candidate in (path, registered, os.path.realpath(registered)) or candidate.endswith("/" + path) for candidate in candidates):
                        return path
        return ""

    def registry_families(self):
        """登记表 .claude/gate.d/verdict-library-scripts.tsv（相对仓根）：→ ({第一列的路径: 第三列（判定库放哪）}, 读不了或写坏了的那一句；读得了是空串)。
        第三列以「prune-server」起头的那一族判定库读写全在服务里、两台读的是同一份，提交时照收（判定库服务化的会话定的规矩）；以「不算：」「无判定库：」起头的没有判定库。
        表的修改时刻没变就用上一次读的。"""
        path = os.path.join(os.path.realpath(self.peer_root), VERDICT_LIBRARY_REGISTRY)
        try:
            stamp = os.stat(path).st_mtime_ns
        except OSError as error:
            return {}, f"登记表 {path} 读不了（{error}）"
        cached = getattr(self, "registry_cache", None)
        if cached and cached[0] == stamp:
            return cached[1], ""
        families = {}
        try:
            with open(path, encoding="utf-8") as handle:
                for number, line in enumerate(handle, 1):
                    if not line.strip() or line.startswith("#"):
                        continue
                    columns = line.rstrip("\n").split("\t")
                    if len(columns) < 3:
                        return {}, f"登记表 {path} 第 {number} 行不到三列"
                    families[columns[0].strip()] = columns[2].strip()
        except (OSError, UnicodeDecodeError) as error:
            return {}, f"登记表 {path} 读不了（{error}）"
        self.registry_cache = (stamp, families)
        return families, ""

    def containing_shared_root(self, real):
        """real（真路径）落在哪个 NFS 共享根下：交那个根，不在交空串（find_outside_shared_path 与 shared_roots_problem 共用）。"""
        return next((root for root in self.shared_roots if real == root or real.startswith(root.rstrip(os.sep) + os.sep)), "")

    def attempt_shared_roots(self, task):
        """这一次交给命令的两个共享根（task_server_lib.py 文件头「资源记录」后的「共享根」）：{变量名: 值}；任务自己带了的（白名单里的）照任务带的。"""
        values = {lib.DATA_ROOT_VARIABLE: self.data_root, lib.WORK_ROOT_VARIABLE: self.work_root}
        for name in lib.SHARED_ROOT_VARIABLES:
            if task["environment"].get(name):
                values[name] = task["environment"][name]
        return values

    def shared_roots_problem(self, environment):
        """② 共享根（文件头 ②「提交时拒」）：这一次要交给命令的两个共享根（attempt_shared_roots：服务自己的，任务带了的照任务带的）有不在 NFS_SHARED_ROOTS 某一段下的
        交 {problem, next_step}（点名变量与值），都在交 None。相对路径算不在（第二台照它自己的目录解）。"""
        outside = [f"{name}={value}" for name, value in self.attempt_shared_roots({"environment": environment}).items()
                   if not os.path.isabs(value) or not self.containing_shared_root(os.path.realpath(value))]
        if not outside:
            return None
        roots = "、".join(self.shared_roots) or "一个根都没写"
        return {"problem": (f"{'、'.join(outside)} 不在多机配置 {lib.NFS_ROOTS_KEY} 的任何一段下（{roots}）：命令在第二台经它写的，两台读不到同一份；"
                            "这一条不收、不跑"),
                "next_step": (f"配多机配置 {self.configuration.path}：{lib.DATA_ROOT_KEY} 写在 {lib.NFS_ROOTS_KEY} 某一段下、服务环境里的 {lib.WORK_ROOT_VARIABLE} "
                              f"也放在那下面（或往 {lib.NFS_ROOTS_KEY} 加上它所在的根），改好重起服务；是任务自己带的那个变量，就去掉它或改成共享根下的路径，再提交")}

    def candidate_hosts(self, task, snapshot):
        """这一件能派的几台，按先后排（task_server_lib.py 文件头「挑机器」）：重排钉了台的（retry_decision 换到总上限更大的那一台）只那一台；
        第二台派不过去的只本机；分片的各片第二台在前；别的本机在前。
        弄坏开关 host-lists-restored 下照改前点到只在第二台跑的入口的只第二台。"""
        pinned = task.get("next_host") or ""
        if pinned:
            return [pinned]
        if self.peer_only_entry(task):
            return [PEER]
        order = list(MACHINE_ORDER)
        if task.get("role") == "shard" and not lib.broken("shard-not-sent-to-peer", self.environment):
            order.reverse()
        return [host for host in order if self.host_eligible(task, host, snapshot)]

    def peer_only_entry(self, task):
        """弄坏开关 host-lists-restored 下：命令点到删之前只在第二台跑的入口（RESTORED_HOST_LISTS 的 peer_entries）交 (入口, 理由)，没点到交 None；
        开关没开一律 None（没有只派第二台的入口）。词照相对仓根的路径比（与改本机工作区的工具同一个比法）。"""
        if not lib.broken("host-lists-restored", self.environment):
            return None
        root = os.path.realpath(self.peer_root)
        directory = task.get("cwd") or root
        for word in task.get("command") or []:
            if word.startswith("-"):
                continue
            relative = os.path.relpath(os.path.normpath(os.path.join(directory, word)), root)
            if relative in RESTORED_HOST_LISTS["peer_entries"]:
                return relative, "弄坏开关 host-lists-restored：照改前只在第二台跑"
        return None

    def host_margin(self, host, snapshot):
        """这台此刻的余量（文件头 ②「余量」）：→ (字节, 量不到的原因)。取这台自己报告里那一项（host-dispatch.py 的 report_margin：memory_peaks_lib 现量、按 1 GiB 向上取整），
        两台各用各的，不跨轮存；这台没报告（探测失败）或报告里量不到的交 None 与原因，不退回固定的数。
        弄坏开关 margin-local-for-both 下第二台也取本机报告里那一项；margin-fallback-fixed 下量不到的退回 BREAK_FIXED_MARGIN_BYTES。"""
        name = lib.HOST_NAMES.get(host, host)
        source = LOCAL if lib.broken("margin-local-for-both", self.environment) else host   # 弄坏开关：两台都取本机报告里的那一项
        report = snapshot.reports.get(source)
        if report is None:
            return None, f"{name}此刻查不到余量（探测没回报告{'：' + snapshot.down[host] if snapshot.down.get(host) else ''}）"
        margin, problem = self.host_dispatch.report_margin(report["memory"])
        if margin is None and lib.broken("margin-fallback-fixed", self.environment):
            return BREAK_FIXED_MARGIN_BYTES, ""
        if margin is None:
            return None, f"{name}此刻量不到余量（{problem}）"
        return margin, ""

    def host_capacity(self, host, snapshot):
        """这台单条能给到的最大上限（文件头 ②「余量」）：→ (这台总上限 − 这台此刻的余量, 这台这一轮不派的原因)。余量量不到、或总上限不比余量大的交 None 与原因。
        弄坏开关 booked-up-to-slice-total 下不减余量（照改前收到总上限）。"""
        name = lib.HOST_NAMES.get(host, host)
        margin, why = self.host_margin(host, snapshot)
        if margin is None:
            return None, why
        total = self.host_total(host, snapshot)
        if lib.broken("booked-up-to-slice-total", self.environment):
            return total, ""
        if total <= margin:
            human = self.host_dispatch.human
            return None, f"{name}的总上限 {human(total)} 不比此刻的余量 {human(margin)} 大，一条都给不出上限"
        return total - margin, ""

    def pinned_on_host(self, prediction, host, snapshot):
        """钉了的命令在这台上的钉值（字节，文件头 ②「内存」的钉值）：多机配置 SLICE_MEMORY_CAP_BY_HOST 里这台那一项（prediction.memory_by_host，配多少就是多少）
        减这台此刻的余量（host_margin）。调用方先经 host_capacity 判过这台量得到余量。弄坏开关 pinned-cap-this-host-only 下两台都取服务这一台那一项；
        pinned-cap-at-slice-total 下不减余量（照那一项记满）。"""
        source = LOCAL if lib.broken("pinned-cap-this-host-only", self.environment) or host not in prediction.memory_by_host else host
        value = int(prediction.memory_by_host[source])
        if lib.broken("pinned-cap-at-slice-total", self.environment):
            return value
        return value - self.host_margin(host, snapshot)[0]

    def host_limit(self, prediction, host, snapshot):
        """这一条在这台上能给到的最大上限：→ (字节, 这台这一轮不派的原因)（文件头 ②「余量」）。这台没报告、余量量不到、总上限不比余量大，或钉了的命令在这台的钉值
        不大于 0（那一项不比这台此刻的余量大），交 None 与原因；这台这一轮不派它，不退回别的数、不兜底。弄坏开关 pinned-nonpositive-accepted 下钉值不大于 0 的也照派。"""
        name = lib.HOST_NAMES.get(host, host)
        if host not in snapshot.reports:
            return None, f"{name}此刻查不到余量（探测没回报告{'：' + snapshot.down[host] if snapshot.down.get(host) else ''}）"
        capacity, why = self.host_capacity(host, snapshot)
        if capacity is None:
            return None, why
        if prediction is not None and prediction.memory_history == lib.PINNED_HISTORY and prediction.memory_by_host:
            value = self.pinned_on_host(prediction, host, snapshot)
            if value <= 0 and not lib.broken("pinned-nonpositive-accepted", self.environment):
                human = self.host_dispatch.human
                return None, (f"{name}：钉值取多机配置的 {lib.PIN_SOURCE} 里这台那一项，减这台此刻的余量 {human(self.host_margin(host, snapshot)[0])} 之后不大于 0，"
                              f"这台一条都给不出钉值；出路：把多机配置 {self.configuration.path} 的 {lib.PIN_SOURCE} 里这台那一项改大（写的是那台 slice 的总上限），"
                              "或 python3 task-server/control unpin-cap <命令…> 撤掉钉值")
        return capacity, ""

    def usable_maxima(self, snapshot):
        """每台单条能给到的最大上限（retry_decision 乘 1.5 与「上限到了总上限」照它比）：这台总上限减这台此刻的余量（host_capacity）；量不到的是 None（查不到）。"""
        result = {}
        for host, maximum in snapshot.maxima().items():
            result[host] = None if maximum is None else self.host_capacity(host, snapshot)[0]
        return result

    def booked_at_cap(self, prediction, host, snapshot):
        """准入按上限记账（文件头 ②「内存」；用户：「谁超了杀谁 不要连累别人」）：这台上记的账取上限（收到这台的总上限减这台此刻的余量为止，host_limit），不取峰值；
        记账本来就比那个数大的（峰值就装不下）照原样，judge_host 照旧判永远放不下。只对 host_limit 交了数的台调，交不出数的抛 ValueError（不退回别的数）。
        弄坏开关 booked-at-peak 下照改前记峰值。"""
        if lib.broken("booked-at-peak", self.environment):
            return prediction
        limit, why = self.host_limit(prediction, host, snapshot)
        if limit is None:
            raise ValueError(f"这台这一轮不派：{why}")
        if prediction.memory_history == lib.PINNED_HISTORY and prediction.memory_by_host:
            # 钉值按台（文件头 ②「内存」的钉值）：判哪一台就取多机配置 SLICE_MEMORY_CAP_BY_HOST 里那一台那一项、减那台此刻的余量
            value = self.pinned_on_host(prediction, host, snapshot)
            prediction = dataclasses.replace(prediction, memory_predicted=value, memory_cap=value, memory_booked=value)
        # 收到这台总上限减这台此刻的余量为止：slice 里挂着的那一截回收不掉，记满一整台永远放不下
        cap = min(prediction.memory_cap, limit)
        booked = max(int(prediction.memory_booked or 0), int(cap or 0))
        if int(prediction.memory_predicted or 0) <= limit or prediction.memory_history == lib.PINNED_HISTORY:
            booked = min(booked, limit)
            if prediction.memory_history == lib.PINNED_HISTORY:
                prediction = dataclasses.replace(prediction, memory_predicted=min(int(prediction.memory_predicted or 0), limit))   # 钉值与收过的上限同一个口径；峰值本身就装不下的照原样，judge_host 判永远放不下
        return dataclasses.replace(prediction, memory_cap=min(int(prediction.memory_cap or 0), limit), memory_booked=booked)

    def room_by_the_formula(self, host, snapshot):
        """这台还能记多少（用户定的公式，只认这一个）：配置的总上限 − 余量 − 在跑的上限之和。不看实际用量。量不到余量交 None。"""
        margin = self.host_margin(host, snapshot)[0]
        if margin is None:
            return None
        capped = sum(int(attempt.get("memory_cap") or 0) for attempt, _ in self.store.select("attempts", "state IN ('starting', 'running')") if attempt.get("host") == host)
        return snapshot.total(host) - margin - capped

    def verdict_within_the_formula(self, verdict, on_host, host, snapshot):
        """放得下的再过一遍公式（room_by_the_formula）：记账超过它就放不下。"""
        room = self.room_by_the_formula(host, snapshot)
        if not verdict.fits or room is None or int(on_host.memory_booked or 0) <= room:
            return verdict
        return dataclasses.replace(verdict, fits=False, why=f"内存：要记 {int(on_host.memory_booked or 0) >> 20} MiB，配置的总上限减余量减在跑的上限之和只剩 {max(room, 0) >> 20} MiB")

    def shard_cap_fitted_to_room(self, task, on_host, host, snapshot, pending_memory):
        """分片的片：记满上限放不下、预测放得下时，上限与记账一起压到这台此刻放得下的量（task_server_lib.py 文件头「资源记录」的准入）。
        上限之和照旧不超过 slice，超了只杀它自己；压过的上限撞了照「重试」放大，放不下就排着。钉值不压。弄坏开关 shard-cap-not-fitted 下不压。"""
        if task.get("role") != "shard" or on_host.memory_history == lib.PINNED_HISTORY or lib.broken("shard-cap-not-fitted", self.environment):
            return on_host
        room = self.room_by_the_formula(host, snapshot)
        if room is None:
            return on_host
        if int(on_host.memory_predicted or 0) <= room < int(on_host.memory_booked or 0):
            return dataclasses.replace(on_host, memory_cap=room, memory_booked=room)
        return on_host

    def judged_on_host(self, task, prediction, host, snapshot, uses_card, pending_memory, pending_gpu):
        """这一条在这台上的内存放不放得下：→ (这台上的记账, judge_host 的判决)。排队（judge_placement）、起之前复核（launch_recheck）都调这一处（弄坏开关下另判排头的 head_judged_apart 同样）：
        记账取上限（booked_at_cap）、分片的片压到放得下（shard_cap_fitted_to_room），judge_host 判余量，放得下的再过用户定的公式（verdict_within_the_formula），次序不变。
        只对 host_limit 交了数的台调。"""
        on_host = self.shard_cap_fitted_to_room(task, self.booked_at_cap(prediction, host, snapshot), host, snapshot, pending_memory)   # 记账 = 这台上的上限（文件头 ②「内存」）
        verdict = self.host_dispatch.judge_host(host, snapshot.reports[host], on_host, uses_card, pending_memory, pending_gpu,
                                                snapshot.memory_caps.get(host), self.host_margin(host, snapshot)[0])
        return on_host, self.verdict_within_the_formula(verdict, on_host, host, snapshot)

    def host_admission(self, task, prediction, host, snapshot, view, uses_card, pending=None):
        """这一条此刻在这台上放不放得下：→ (这台上的记账, 判决)。派活（judge_placement）与起之前复核（launch_recheck）都只调这一处：
        内存（judged_on_host）放得下的再判编译目录占盘（build_disk_problem）与 CPU 闸（cpu_problem），哪一样放不下判决就是放不下、原因写那一样。
        交替不在这里判（Alternation.blocked）。/health 不另判，只读派活给排头的结论（starved_problem）。只对 host_limit 交了数的台调。
        pending 是调用方已读的这台待涨量（view.pending_of 交的那一对），没给就现读。"""
        pending_memory, pending_gpu = pending or view.pending_of(host)   # 服务账上与派发器账上还没涨到量的，连同这一轮起了的
        on_host, verdict = self.judged_on_host(task, prediction, host, snapshot, uses_card, pending_memory, pending_gpu)
        for why in (self.build_disk_problem(task, host, snapshot, view), self.cpu_problem(task, host, snapshot)):
            if why and verdict.fits:   # 内存放得下、占盘或 CPU 放不下
                verdict = dataclasses.replace(verdict, fits=False, never=False, why=why)
        return on_host, verdict

    def recorded_prediction(self, task, base):
        """服务自己的资源记录（文件头 ②「内存」与 task_server_lib.py 文件头「资源记录」）：同一命令键最近 RECORD_WINDOW 次量到的峰值，
        服务自己的尝试按结束时刻从新到旧排，量不到的不算、不占名额。服务自己的尝试只用跑完的：退 0 的记量到的峰值，撞了这一条上限的按它的上限算；
        别的结局的峰值不进、不占名额。交 (预测, 只有没跑完的)：预测一次都没有是 None；只有没跑完的是「服务自己有这条命令结束了的尝试、一次跑完的都没有」。"""
        measurements, own_ended, completed_seen = self.recorded_measurements(task)
        window = sorted(measurements, reverse=True)[:RECORD_WINDOW]
        unfinished_only = own_ended and not completed_seen
        if not window:
            return None, unfinished_only
        predicted = max(value for _, value in window)
        cap = math.ceil(predicted * lib.CAP_GROWTH_FACTOR / lib.MEBIBYTE) * lib.MEBIBYTE
        return dataclasses.replace(base, memory_predicted=predicted, memory_cap=cap, memory_booked=predicted, memory_history=lib.RECORD_HISTORY), unfinished_only

    def recorded_measurements(self, task):
        """同一命令键服务自己结束了的尝试里进定额度的那几个量（recorded_prediction 与 key_recent_peak 共用一份判法）：
        → ([(结束时刻, 字节)], 有没有结束了的尝试, 有没有跑完的)。跑完（退 0）的记量到的峰值，撞了这一条上限的记它的上限；别的结局不进。"""
        measurements = []
        own_ended = completed_seen = False
        unfinished_counted = lib.broken("unfinished-peaks-counted", self.environment)   # 弄坏开关：哪种结局的峰值都进（改前的样子）
        for attempt, valid in self.store.select("attempts", "command_key = ? AND state = 'ended'", (task["command_key"],), order="id DESC LIMIT 200"):
            if not valid:
                continue
            own_ended = True
            outcome = attempt.get("outcome")
            cap_hit = outcome in lib.RECORD_CAP_HIT_OUTCOMES and not lib.broken("cap-hit-not-counted", self.environment)
            completed = outcome in lib.RECORD_COMPLETED_OUTCOMES
            if not (cap_hit or completed or unfinished_counted):
                continue   # 没跑完的那一次：提前失败的峰值偏小，不代表这条命令要多少
            completed_seen = True
            when, peak = float(attempt.get("ended_at") or 0), attempt.get("memory_peak")
            if cap_hit and attempt.get("memory_cap"):
                measurements.append((when, int(attempt["memory_cap"])))   # 撞了这一条上限：至少要到这个上限
            elif completed and lib.broken("completed-peak-ignored", self.environment):
                continue   # 弄坏开关：跑完那次量到的峰值不进
            elif isinstance(peak, int) and (peak > 0 or lib.broken("unmeasured-recorded-as-zero", self.environment)):
                measurements.append((when, peak))
        return measurements, own_ended, completed_seen

    def key_recent_peak(self, task):
        """这把命令键服务自己记着的近 RECORD_WINDOW 次里最高的一次（文件头 ②「内存」的按近几次最高起跑）：recorded_measurements 那几个量（撞过的上限、跑成的峰值）
        按结束时刻从新到旧取 RECORD_WINDOW 个、取最大，与 recorded_prediction 同一个窗口：出了窗口的不再算，后面几次都跑得小，起点跟着降；没有交 None。
        弄坏开关 record-ignored-for-cap 下同样不认记录（交 None）；history-peak-unwindowed 下照改前不截近几次、全部记录里取最大（只升不降）。"""
        if lib.broken("record-ignored-for-cap", self.environment):
            return None
        measurements, _, _ = self.recorded_measurements(task)
        if not lib.broken("history-peak-unwindowed", self.environment):
            measurements = sorted(measurements, reverse=True)[:RECORD_WINDOW]
        return max((value for _, value in measurements), default=None)

    def history_raised_prediction(self, task, prediction):
        """新提交的头一次与撞上限之后的每次重排都直接起在近几次最高的那一档（文件头 ②「内存」的按近几次最高起跑）：这把键近 RECORD_WINDOW 次的最高（key_recent_peak）
        × 1.5 比 prediction 的上限大就换上（只往上抬，下限照 memory_floor_applied 先兜过）；重排的那一次另照编译族（build_family_peak）与分片的兄弟片
        （shard_family_peak）往上抬（头一次在 computed_prediction 里已看过）。钉值不经这里。弄坏开关 history-peak-ignored 下照改前原样交回。"""
        if lib.broken("history-peak-ignored", self.environment):
            return prediction
        prediction = self.raised_prediction(prediction, self.key_recent_peak(task), lib.KEY_HISTORY_PEAK_HISTORY)
        if task.get("next_cap"):
            prediction = self.raised_prediction(prediction, self.build_family_peak(task), lib.BUILD_FAMILY_HISTORY)
            if task.get("role") == "shard":
                prediction = self.raised_prediction(prediction, self.shard_family_peak(task), lib.SHARD_FAMILY_HISTORY)
        return prediction

    def estimated_seconds(self, task):
        """预计用时的秒数（estimate_of 交的第一样）；没有交 None。"""
        return self.estimate_of(task)[0]

    def estimate_of(self, task):
        """预计用时（task_server_lib.py 文件头「资源记录」的预计用时）→ (秒数或 None, 这个数照哪种算法得的：lib.ESTIMATE_BASIS_*)。
        点了 --item 的三条按项入口（提交时记了 item_entry 的）照 item_estimate（按项）；别的取服务自己库里同一命令键、最近一次跑成的尝试的用时（按命令），
        分片的计划、各片与合并也在这一种里（命令键各带那一步一段，取的是这一步自己的用时）；没有的交 None 与 ESTIMATE_BASIS_NONE。
        弄坏开关 estimate-unreported 下一律没有，item-estimate-by-command 下点了 --item 的也照按命令那一种算，
        shard-plan-merge-unestimated 下计划与合并没有，shard-pieces-unestimated 下各片没有。"""
        role = task.get("role")
        if lib.broken("estimate-unreported", self.environment):
            return None, lib.ESTIMATE_BASIS_NONE
        if (role in ("plan", "merge") and lib.broken("shard-plan-merge-unestimated", self.environment)
                or role == "shard" and lib.broken("shard-pieces-unestimated", self.environment)):
            return None, lib.ESTIMATE_BASIS_NONE
        if task.get("item_entry") and not lib.broken("item-estimate-by-command", self.environment):
            seconds = self.item_estimate(task)
            return seconds, lib.ESTIMATE_BASIS_NONE if seconds is None else lib.ESTIMATE_BASIS_ITEMS
        seconds = self.command_estimate(task)
        return seconds, lib.ESTIMATE_BASIS_NONE if seconds is None else lib.ESTIMATE_BASIS_COMMAND

    def command_estimate(self, task):
        """按命令的预计用时：库里同一命令键（与内存记录同一把键：路径、条目照词折过，--check 的格名照留）最近一次跑成的尝试的用时；没有交 None。
        弄坏开关 estimate-by-exact-command 下照改前只认命令逐词相同的（任务 7585 与 7447 只差补丁目录，内存认得同键、预计用时却没有，封组挡住后面），
        failed-runs-estimated 下退非 0 的也算。"""
        runs = []
        for attempt, valid in self.store.select("attempts", "command_key = ? AND state = 'ended'", (task["command_key"],), order="id DESC LIMIT 200"):
            counted = lib.RECORD_COMPLETED_OUTCOMES + (("command-failed",) if lib.broken("failed-runs-estimated", self.environment) else ())
            if lib.broken("estimate-by-exact-command", self.environment) and attempt.get("command") != task.get("command"):
                continue
            if valid and attempt.get("outcome") in counted and isinstance(attempt.get("seconds"), (int, float)):   # 退非 0 的可能是提前退，不当用时
                runs.append((float(attempt.get("ended_at") or 0), float(attempt["seconds"])))
        return max(runs)[1] if runs else None

    def item_estimate(self, task):
        """按项的预计用时（task_server_lib.py 文件头「资源记录」的预计用时）：点到的各项 to_judge × 每现判一条几秒 + reusable × 每复用一条几秒之和
        （to_judge、reusable 是估算子任务的 ITEM_PLAN，记在任务的 item_plan；两个速率在 item_rates 表）。item_plan 是 null、里面没有某一项、item_rates 没有那一项
        （或那一行校验不过）、要现判而没有现判速率、要复用而没有复用速率的，整条交 None。弄坏开关 item-rate-missing-skipped 下缺的那一项当 0 秒照加
        （item_plan 是 null 的照旧交 None）。"""
        plan = task.get("item_plan")
        skip_missing = lib.broken("item-rate-missing-skipped", self.environment)
        if not isinstance(plan, dict):
            return None
        total = 0.0
        for item in task.get("items") or []:
            counts = plan.get(item)
            rows = self.store.select("item_rates", "command_key = ?", (lib.item_rate_key(task["item_entry"], item),)) if counts else []
            rate = rows[0][0] if rows and rows[0][1] else None
            if counts is None or rate is None:
                if skip_missing:
                    continue
                return None
            seconds = 0.0
            for count, name in ((counts.get("to_judge") or 0, "seconds_per_judged"), (counts.get("reusable") or 0, "seconds_per_reused")):
                if not count:
                    continue
                if not isinstance(rate.get(name), (int, float)):
                    if skip_missing:
                        continue
                    return None
                seconds += count * float(rate[name])
            total += seconds
        return total

    def failed_peak_floor(self, task, prediction):
        """只有没跑完的尝试时（文件头 ②「内存」）：最近一次命令自己退非 0（command-failed）而量到了峰值的尝试，上限与记账不低于那个峰值 × 1.5（按 MiB 向上取整），
        prediction 记 FAILED_PEAK_HISTORY；没有这样的尝试、或那个数不比 prediction 的上限大的照原样。弄坏开关 failed-peak-ignored 下不看。"""
        if lib.broken("failed-peak-ignored", self.environment):
            return prediction
        for attempt, valid in self.store.select("attempts", "command_key = ? AND state = 'ended'", (task["command_key"],), order="id DESC LIMIT 200"):
            peak = attempt.get("memory_peak")
            if not valid or attempt.get("outcome") != "command-failed" or not isinstance(peak, int) or peak <= 0:
                continue
            floor = math.ceil(peak * lib.CAP_GROWTH_FACTOR / lib.MEBIBYTE) * lib.MEBIBYTE
            if floor <= int(prediction.memory_cap or 0):
                return prediction
            return dataclasses.replace(prediction, memory_predicted=peak, memory_cap=floor, memory_booked=floor, memory_history=lib.FAILED_PEAK_HISTORY)
        return prediction

    def unrecorded_prediction(self, task, gpu):
        """没有记录的命令（task_server_lib.py 文件头「资源记录」）：预测、上限与记账都取 UNRECORDED_MEMORY_DEFAULT_BYTES，不乘 1.5；撞了 250 由 retry_decision 放大重排。
        钉值、重排、服务自己的记录都从这一份换掉；显存取 DEFAULT_GPU_PREDICTION_MEBIBYTES。
        弄坏开关 unrecorded-default-grown 下上限与记账都乘 CAP_GROWTH_FACTOR，unrecorded-booked-above-cap 下只把记账乘它。"""
        dispatch = self.host_dispatch
        default = lib.UNRECORDED_MEMORY_DEFAULT_BYTES
        if lib.broken("unrecorded-default-grown", self.environment):
            default = math.ceil(default * lib.CAP_GROWTH_FACTOR / lib.MEBIBYTE) * lib.MEBIBYTE
        booked = default
        if lib.broken("unrecorded-booked-above-cap", self.environment):
            booked = math.ceil(default * lib.CAP_GROWTH_FACTOR / lib.MEBIBYTE) * lib.MEBIBYTE
        prediction = dispatch.Prediction(task["command_key"], default, default, booked, lib.UNRECORDED_HISTORY)
        if gpu:
            prediction.gpu_predicted, prediction.gpu_history = dispatch.DEFAULT_GPU_PREDICTION_MEBIBYTES, "default"
        return prediction

    def prediction_of(self, task):
        """这一条的内存预测（文件头 ②「预测每条只算一次」）：第一次算出后存下，之后直接用，不再读峰值表、不再扫记录；next_cap 变了作废重算。
        弄坏开关 prediction-recomputed-every-pass 下每次都现算。"""
        with self.prediction_lock:
            stored = self.predictions.get(task["id"])
            if stored is not None and stored[0] == task.get("next_cap") and not lib.broken("prediction-recomputed-every-pass", self.environment):
                return stored[1]
            return self.store_prediction(task)

    def fresh_prediction(self, task):
        """起之前复核用：现算一次、换掉存的那一份。"""
        with self.prediction_lock:
            return self.store_prediction(task)

    def store_prediction(self, task):
        prediction = self.computed_prediction(task)
        self.predictions[task["id"]] = (task.get("next_cap"), prediction)
        return prediction

    def forget_prediction(self, task_id):
        with self.prediction_lock:
            self.predictions.pop(task_id, None)
            self.build_disk_predictions.pop(task_id, None)
            self.pin_unreadable_reasons.pop(task_id, None)

    def keep_predictions_of(self, task_ids):
        """只留还在队里的那几条存的预测。"""
        with self.prediction_lock:
            self.predictions = {task_id: stored for task_id, stored in self.predictions.items() if task_id in task_ids}
            self.build_disk_predictions = {task_id: stored for task_id, stored in self.build_disk_predictions.items() if task_id in task_ids}
            self.pin_unreadable_reasons = {task_id: why for task_id, why in self.pin_unreadable_reasons.items() if task_id in task_ids}

    # ── 编译目录占盘（文件头 ②「编译目录占盘」；records/2026-10-31-task-server巡检.md 问题 76）──
    def build_disk_prediction(self, task):
        """这条命令的编译目录占盘预测（字节）：同一命令键最近 RECORD_WINDOW 次跑成（succeeded）而量到了占盘峰值（build_peak，两台都记）的尝试取最大，
        取法同内存峰值（recorded_prediction）；一次都没量到交 0。每条在排任务只算一次。"""
        with self.prediction_lock:
            if task["id"] in self.build_disk_predictions:
                return self.build_disk_predictions[task["id"]]
        measured = []
        for attempt, valid in self.store.select("attempts", "command_key = ? AND state = 'ended'", (task["command_key"],), order="id DESC LIMIT 200"):
            peak = attempt.get("build_peak")
            if valid and attempt.get("outcome") in lib.RECORD_COMPLETED_OUTCOMES and isinstance(peak, int) and not isinstance(peak, bool) and peak > 0:
                measured.append((float(attempt.get("ended_at") or 0), peak))
        predicted = max((peak for _, peak in sorted(measured, reverse=True)[:RECORD_WINDOW]), default=0)
        with self.prediction_lock:
            self.build_disk_predictions[task["id"]] = predicted
        return predicted

    def server_disk_pending(self, host):
        """服务账上这台在跑的每一次尝试编译目录还要涨的占盘：max(0, 它起的时候记的占盘 − 此刻量到的)；此刻量到的照那一台的接口读（Machine.build_bytes：
        本机读尝试目录里 attempt_runner 写的 build_bytes，第二台量不到），量不到的照记的算。"""
        pending = 0
        for attempt, _ in self.store.select("attempts", "state IN ('starting', 'running')"):
            if attempt.get("host") != host or not attempt.get("disk_booked"):
                continue
            used = self.machines[host].build_bytes(attempt) or 0
            pending += max(0, int(attempt["disk_booked"]) - used)
        return pending

    def build_disk_problem(self, task, host, snapshot, view):
        """这一条在这台上编译目录占盘放不放得下：那台盘余量（报告里的 DISK_REPORT_KEY）减去在跑的还要涨的，
        要放得下这条命令的占盘预测加 BUILD_DISK_FLOOR_BYTES；放不下交排队原因（点名占盘），放得下、不会编又没量过、余量读不到的交空串。
        弄坏开关 build-disk-unpredicted 下一律空串（改前只看第二台的下限）。"""
        if lib.broken("build-disk-unpredicted", self.environment):
            return ""
        available = (snapshot.reports.get(host) or {}).get(DISK_REPORT_KEY)
        if not isinstance(available, int) or isinstance(available, bool):
            return ""
        predicted = self.build_disk_prediction(task)
        if not predicted and not task.get("compiles"):
            return ""
        pending = view.disk_pending_of(host)
        needed = predicted + BUILD_DISK_FLOOR_BYTES
        if available - pending >= needed:
            return ""
        return (f"编译目录占盘：这条命令量过的编译目录峰值 {gibibytes_text(predicted)} 加下限 {gibibytes_text(BUILD_DISK_FLOOR_BYTES)} 要 {gibibytes_text(needed)}，"
                f"{lib.HOST_NAMES.get(host, host)}盘此刻剩 {gibibytes_text(available)}、在跑的编译目录还要涨 {gibibytes_text(pending)}，放不下"
                "（等在跑的跑完、task clean 删掉留下的编译目录）")

    def computed_prediction(self, task):
        """现算（扫服务自己的记录；不读内存峰值表）：文件头 ②「内存」。"""
        self.selftest_read(f"predict task={task['id']}")
        base = self.unrecorded_prediction(task, bool(task.get("gpu")))
        pinned = self.pinned_prediction(task, base)
        if pinned is not None:
            return pinned
        if task.get("next_cap"):
            cap = int(task["next_cap"])
            prediction = dataclasses.replace(base, memory_predicted=cap, memory_cap=cap, memory_booked=cap, memory_history="retry")
        else:
            recorded, unfinished_only = ((None, False) if lib.broken("record-ignored-for-cap", self.environment)
                                         else self.recorded_prediction(task, base))
            if recorded is None and unfinished_only:
                # 服务自己只有没跑完的尝试：照没记录办，判红那一次量到的峰值当下界（文件头 ②「内存」）
                prediction = self.failed_peak_floor(task, base)
            else:
                prediction = recorded or base   # 服务的记录里没有：先记小，撞了再放大（文件头 ②「内存」）
                if recorded is not None and self.latest_attempt_hit_cap(task):
                    # 最近一次结束的尝试撞了上限，这一条是调用方再交的：第一次尝试照重排那一次办，兜到 4G（memory_floor_applied 不放过这一种 prediction）
                    prediction = dataclasses.replace(recorded, memory_history=lib.CAP_HIT_RERUN_HISTORY)
            if recorded is None:
                # 同键没有跑完的记录：会编的照同一编译族别的命令跑成的峰值起步（文件头 ②「内存」的编译族），只往上抬
                prediction = self.raised_prediction(prediction, self.build_family_peak(task), lib.BUILD_FAMILY_HISTORY)
            if task.get("role") == "shard":
                # 分片的一片：同一计划的计划那一步与兄弟片已结束的尝试里量到的最大值当下界（文件头 ②「内存」的同一计划共用峰值），只往上抬
                prediction = self.raised_prediction(prediction, self.shard_family_peak(task), lib.SHARD_FAMILY_HISTORY)
        return self.history_raised_prediction(task, self.memory_floor_applied(prediction))

    def raised_prediction(self, prediction, peak, history):
        """peak（字节；None 是没有）× 1.5（按 MiB 向上取整）比 prediction 的上限大才换上：上限与记账都取它、预测记 peak、prediction 记 history；
        不比它大的照原样（只往上抬，不压到没记录的 4G 以下）。"""
        if not peak:
            return prediction
        cap = math.ceil(peak * lib.CAP_GROWTH_FACTOR / lib.MEBIBYTE) * lib.MEBIBYTE
        if cap <= int(prediction.memory_cap or 0):
            return prediction
        return dataclasses.replace(prediction, memory_predicted=max(int(peak), int(prediction.memory_predicted or 0)), memory_cap=cap,
                                   memory_booked=max(cap, int(prediction.memory_booked or 0)), memory_history=history)

    def build_family_peak(self, task):
        """这条的编译族（submit 记的 build_family）最近 RECORD_WINDOW 次跑成而量到峰值的尝试（别的命令键的也算）里最大的峰值；没有族、没有这样的尝试交 None。
        弄坏开关 build-family-ignored 下交 None（改前的样子：会编的 cargo 同键没记录只从 4G 起）。"""
        family = task.get("build_family") or ""
        if not family or lib.broken("build-family-ignored", self.environment):
            return None
        measured = []
        for attempt, valid in self.store.select("attempts", "state = 'ended' AND json_extract(record, '$.build_family') = ?", (family,), order="id DESC LIMIT 200"):
            peak = attempt.get("memory_peak")
            if valid and attempt.get("outcome") in lib.RECORD_COMPLETED_OUTCOMES and isinstance(peak, int) and not isinstance(peak, bool) and peak > 0:
                measured.append((float(attempt.get("ended_at") or 0), peak))
        window = sorted(measured, reverse=True)[:RECORD_WINDOW]
        return max(peak for _, peak in window) if window else None

    def shard_family_peak(self, task):
        """分片的一片的同一计划下界：同一父任务名下计划那一步与别的片已结束的尝试里，跑成的取量到的峰值、撞了上限的取那一次的上限，
        按结束时刻从新到旧取 RECORD_WINDOW 个（与 key_recent_peak 同一个窗口）、取最大；没有交 None。
        子任务行校验不过的不认（交 None）。弄坏开关 shard-siblings-unshared 下交 None（改前的样子：各片各一个键、各自从 4G 爬）；
        history-peak-unwindowed 下照改前不截近几次。"""
        if lib.broken("shard-siblings-unshared", self.environment) or not task.get("parent_id"):
            return None
        try:
            siblings = [child for child in self.children_of(task["parent_id"]) if child.get("role") in ("plan", "shard") and child.get("id") != task.get("id")]
        except StoreRefused:
            return None
        values = []
        for sibling in siblings:
            for attempt, valid in self.store.attempts_of(sibling["id"]):
                if not valid or attempt.get("state") != "ended":
                    continue
                peak = attempt.get("memory_peak")
                when = float(attempt.get("ended_at") or 0)
                if attempt.get("outcome") in lib.RECORD_CAP_HIT_OUTCOMES and attempt.get("memory_cap"):
                    values.append((when, int(attempt["memory_cap"])))   # 撞了这一次的上限：至少要到这个上限
                elif attempt.get("outcome") in lib.RECORD_COMPLETED_OUTCOMES and isinstance(peak, int) and not isinstance(peak, bool) and peak > 0:
                    values.append((when, peak))
        if not lib.broken("history-peak-unwindowed", self.environment):
            values = sorted(values, reverse=True)[:RECORD_WINDOW]
        return max((value for _, value in values), default=None)

    def latest_attempt_hit_cap(self, task):
        """同一命令键服务自己最近一次结束的尝试（按尝试号从新到旧，校验过的头一条）撞了上限（outcome 在 lib.RECORD_CAP_HIT_OUTCOMES）交 True；没有结束的尝试、
        最近一次是别的结局交 False。弄坏开关 cap-hit-rerun-unfloored 下一律 False（改前的样子：再交一遍只照记录 × 1.5）。"""
        if lib.broken("cap-hit-rerun-unfloored", self.environment):
            return False
        for attempt, valid in self.store.select("attempts", "command_key = ? AND state = 'ended'", (task["command_key"],), order="id DESC LIMIT 200"):
            if valid:
                return attempt.get("outcome") in lib.RECORD_CAP_HIT_OUTCOMES
        return False

    def memory_floor(self):
        """每条任务上限的下限（文件头 ②「内存」）：lib.MEMORY_FLOOR_BYTES；自证的小世界里由 TASK_SERVER_SELFTEST_MEMORY_FLOOR_BYTES 换（task_server_lib.py 文件头
        「只在自证里认的变量」）。"""
        override = lib.selftest_override(lib.MEMORY_FLOOR_VARIABLE, self.environment)
        return int(override) if override.isdigit() else lib.MEMORY_FLOOR_BYTES

    def memory_floor_applied(self, prediction):
        """上限低于下限的上限与记账都取下限、预测照原样（问题 92：任务 2419 一条编译取到 371M 撞顶）；钉值与服务自己的峰值记录定的（RECORD_HISTORY）不兜，
        照峰值 × 1.5（文件头 ②「内存」）。弄坏开关 memory-floor-ignored 下照原样，recorded-cap-floored 下记录定的也兜。"""
        floor = self.memory_floor()
        kept = (lib.PINNED_HISTORY,) if lib.broken("recorded-cap-floored", self.environment) else (lib.PINNED_HISTORY, lib.RECORD_HISTORY)
        if (prediction.memory_history in kept or int(prediction.memory_cap or 0) >= floor
                or lib.broken("memory-floor-ignored", self.environment)):
            return prediction
        return dataclasses.replace(prediction, memory_cap=floor, memory_booked=max(int(prediction.memory_booked or 0), floor))

    # ── 钉值（文件头 ②「内存」的钉值；用户：「变异的内存上限直接按照 根目录设置的50gb走」「50 这个是读取的配置 不是随便给的」）──
    def pin_record(self, command_key):
        """库里这个命令键的钉值记录（history 表 state=pin 那一种）；没有交 None。校验不过的不认。"""
        for record, valid in self.store.select("history", "command_key = ? AND state = ?", (command_key, lib.PIN_STATE)):
            if valid:
                return record
        return None

    def configured_slice_caps(self):
        """多机配置 SLICE_MEMORY_CAP_BY_HOST 里两台各自那一项（字节）现读：→ ({"local": 字节, "peer": 字节} 或 None, 读不到的原因)（文件头 ②「内存」的钉值）。
        先照 task_server_lib.py 的 configured_by_host 切、判（每台恰好一项、地址都在 HOSTS 里），再照 Resources.addresses 把这一台与另一台的地址映到 local / peer；
        每一项写法照 scripts/hosts/run-with-memory-cap.sh 读它的那一句（正整数加 K / M / G / T）。"""
        values, problem = lib.configured_by_host(self.configuration.path, lib.PIN_SOURCE, self.environment)
        if values is None:
            return None, problem
        sizes = {}
        for address, text in values.items():   # 先判写法、再认台：写法不对的配置多机配置判法也判不过，认台那一步就说不出是哪一项写错了
            found = SLICE_CAP_VALUE.match(text)
            if not found:
                return None, f"多机配置 {self.configuration.path} 里 {lib.PIN_SOURCE} 写法不对：{address} 那一项「{text}」要写成正整数加 K / M / G / T"
            sizes[address] = int(found.group(1)) * SIZE_UNITS[found.group(2)]
        addresses, problem = self.resources.addresses()
        if addresses is None:
            return None, f"认不出多机配置 {lib.HOSTS_KEY} 里哪一项是服务所在这一台，{lib.PIN_SOURCE} 按台取不了（{problem}）"
        caps = {}
        for host, machine in self.machines.items():
            address = machine.address(addresses)
            if address not in sizes:   # 只在弄坏开关 by-host-missing-accepted 下走到：少了的那一台没有那一项
                return None, f"多机配置 {self.configuration.path} 的 {lib.PIN_SOURCE} 里没有{lib.HOST_NAMES[host]}（{address}）那一项"
            caps[host] = sizes[address]
        return caps, ""

    def pinned_totals(self, record):
        """钉了的命令两台各取的总上限（字节）：现读多机配置 SLICE_MEMORY_CAP_BY_HOST 里两台各自那一项（配多少就是多少，不记字面的数）；
        弄坏开关 pinned-cap-literal 下取钉的那一刻记下的数。→ ({"local": 字节, "peer": 字节} 或 None, 读不到的原因)。"""
        if lib.broken("pinned-cap-literal", self.environment) and isinstance(record.get("bytes_at_pin"), dict):
            return record["bytes_at_pin"], ""
        return self.configured_slice_caps()

    def pin_value(self, record, snapshot):
        """钉值此刻两台各是多少字节（control pin-cap 与 status 报的那个数）：那一台单条能给到的最大上限 = pinned_totals 里那一项减那一台此刻的余量
        （host_margin；派活时 booked_at_cap 照同一个口径现算，不存）。→ ({台: 字节}（量得到余量的那几台）或 None, 原因）：配置读不到交 None 与原因；
        哪一台量不到余量的不给它的数，原因写它。弄坏开关 pinned-cap-at-slice-total 下不减余量。"""
        caps, problem = self.pinned_totals(record)
        if caps is None:
            return None, problem
        values, problems = {}, []
        for host, value in caps.items():
            margin, why = self.host_margin(host, snapshot)
            if margin is None:
                problems.append(why)
                continue
            values[host] = value if lib.broken("pinned-cap-at-slice-total", self.environment) else value - margin
        return values, "；".join(problems)

    def pin_refusal(self, record, snapshot):
        """control pin-cap 当场拒的那一句（文件头 ②「内存」的钉值）：量得到余量的哪一台，钉值（那一项减此刻的余量）不大于 0，就交那一句与出路；都大于 0 交空串。
        弄坏开关 pinned-nonpositive-accepted 下一律交空串。"""
        if lib.broken("pinned-nonpositive-accepted", self.environment):
            return ""
        caps, _ = self.pinned_totals(record)
        values, _ = self.pin_value(record, snapshot)
        human = self.host_dispatch.human
        low = [f"{lib.HOST_NAMES.get(host, host)}那一项 {human(caps[host])} 减此刻的余量 {human(caps[host] - value)} 是 {human(value)}"
               for host, value in sorted((values or {}).items()) if value <= 0]
        if not low:
            return ""
        return (f"多机配置 {self.configuration.path} 的 {lib.PIN_SOURCE} 里{'；'.join(low)}，不大于 0：照这个数那一台一条都起不来，没钉。"
                f"出路：把多机配置里那一项改大（写的是那台 slice 的总上限，配多少就是多少；余量由 task-server 每台现量、不在配置里），再 pin-cap")

    def pinned_prediction(self, task, base):
        """钉了的命令：预测、上限与记账都取钉值（学到的峰值、next_cap 都不顶掉它）；没钉的交 None。钉了而配置读不到的交一份 memory_history 是
        lib.PIN_UNREADABLE_HISTORY 的（数照 base，只给排队的别处算账用），读不到的那一句记进 pin_unreadable_reasons，place 照它记 run-condition-unmet、不起（文件头 ②「内存」的钉值）。
        弄坏开关 pinned-cap-ignored 下不认钉值；pinned-cap-grown 下撞过上限的照 next_cap 放大；pin-unreadable-predicted 下读不到的照改前交 None（照常预测，说一句）。"""
        if lib.broken("pinned-cap-ignored", self.environment):
            return None
        record = self.pin_record(task["command_key"])
        if record is None:
            return None
        if task.get("next_cap") and lib.broken("pinned-cap-grown", self.environment):
            return None
        caps, problem = self.pinned_totals(record)
        if caps is None:
            if lib.broken("pin-unreadable-predicted", self.environment):
                say(f"  ! 任务 {task['id']} 的命令钉了上限（取多机配置的 {lib.PIN_SOURCE}），此刻读不到：{problem}；这一回照常预测")
                return None
            self.pin_unreadable_reasons[task["id"]] = (
                f"这条命令钉了上限（取多机配置的 {lib.PIN_SOURCE} 里派到的那一台那一项），此刻读不到：{problem}；没起，不退回学到的记录。"
                f"出路：把多机配置 {self.configuration.path} 的 {lib.PIN_SOURCE} 写对（一行，<地址>=<正整数加 K / M / G / T> 空白分隔，{lib.HOSTS_KEY} 里每一台恰好一项），"
                f"或 python3 task-server/control unpin-cap <命令…> 撤掉钉值，再重交")
            return dataclasses.replace(base, memory_history=lib.PIN_UNREADABLE_HISTORY)
        # 数先取服务这一台那一项（没落到哪一台时的口径），判哪一台时 booked_at_cap 照 memory_by_host 换成那一台那一项、减那台此刻的余量（pinned_on_host）；
        # 存下来的只是配置里的那一项，余量每一轮现量，不存
        value = caps[LOCAL]
        return dataclasses.replace(base, memory_predicted=value, memory_cap=value, memory_booked=value, memory_history=lib.PINNED_HISTORY, memory_by_host=dict(caps))

    def pin_cap(self, command, pinned, session, process_id):
        """control pin-cap / unpin-cap（接口 POST /pins）：照 memory_peaks_lib 换算命令键，钉上或撤掉；→ 回话（键、此刻钉没钉、这一回改没改、钉值此刻多少）。"""
        key = self.command_key(command, None)
        record = self.pin_record(key)
        changed = False
        snapshot = self.resources.snapshot(False)
        if pinned:   # 那一项不比那台此刻的余量大：当场拒，不钉、不改已有的（文件头 ②「内存」的钉值）
            refusal = self.pin_refusal(record or {}, snapshot)
            if refusal:
                self.operation("pin-cap", 0, session, process_id, f"{key[:200]}（拒了：{refusal[:200]}）")
                return dict(self.pins_reply(key, snapshot=snapshot), refused=refusal)
        if pinned and record is None:
            record = {"state": lib.PIN_STATE, "session": "", "command_key": key, "kind": lib.PIN_STATE, "source": lib.PIN_SOURCE,
                      "command": shlex.join(command)[:500], "pinned_at": round(time.time(), 3)}
            if lib.broken("pinned-cap-literal", self.environment):
                record["bytes_at_pin"] = self.configured_slice_caps()[0]
            self.store.insert("history", record)
            changed = True
        elif not pinned:
            for found, _ in self.store.select("history", "command_key = ? AND state = ?", (key, lib.PIN_STATE)):
                self.store.delete_row("history", found["id"])
                changed = True
        self.operation("pin-cap" if pinned else "unpin-cap", 0, session, process_id, f"{key[:200]}（{'改了' if changed else '原来就是这样'}）")
        with self.prediction_lock:
            self.predictions = {}   # 存的预测作废：在排的下一轮照新的钉值现算
        return self.pins_reply(key, changed, snapshot)

    def pins_reply(self, key, changed=False, snapshot=None):
        record = self.pin_record(key)
        caps, problem = self.pin_value(record, snapshot or self.resources.snapshot(False)) if record is not None else (None, "")
        return {"command_key": key, "pinned": record is not None, "changed": changed, "source": lib.PIN_SOURCE, "bytes_by_host": caps, "problem": problem}

    def server_pending(self, host, report):
        """(内存字节, 显存 MiB)：服务自己账上这台在跑的每一次尝试还没涨到量的那一截 max(0, 上限 − 它的 scope 此刻回收不掉的用量)（文件头 ②「内存」）。
        两台同一段：用量照 report（这一轮读这台余量的那一份报告，没有是 None）里的在跑的 scope（report_scope_usage，scope 照进程环境里的
        TASK_SERVER_ATTEMPT_LABEL 对到尝试的 label 上，memory_peaks_lib.py 文件头「在跑的 scope」），对不上这一次尝试的 label、或没有报告的记整份上限。"""
        if lib.broken("server-pending-ignored", self.environment):
            return 0, 0
        memory, gpu = 0, 0
        usage = self.report_scope_usage(report)
        for attempt, _ in self.store.select("attempts", "state IN ('starting', 'running')"):
            if attempt.get("host") != host:
                continue
            gpu += attempt.get("gpu_booked", 0) or 0
            # 按上限记账（文件头 ②「内存」）：还没涨到量的那一截 = 上限 − 它此刻回收不掉的用量；弄坏开关 booked-at-peak 下照改前用记的峰值
            booked = (attempt.get("booked", 0) if lib.broken("booked-at-peak", self.environment) else attempt.get("memory_cap", 0)) or 0
            used = usage.get(attempt.get("label") or "")
            memory += booked if used is None else max(0, booked - used)   # 报告里没有它的 scope：整份上限，不记 0
        return memory, gpu

    def report_scope_usage(self, report):
        """{标签: 回收不掉的用量（字节）}：一台报告里 slice 下各个在跑的 scope（memory_peaks_lib.SCOPES_REPORT_KEY，那份文件头「在跑的 scope」）。
        没标签的、用量读不了（不是非负整数）的不进；几个 scope 带同一个标签取最小的那一份（文件头 ②「内存」）。报告是 None 或没有这一项交空表。"""
        usage = {}
        scopes = ((report or {}).get("memory") or {}).get(self.peaks.SCOPES_REPORT_KEY)
        for entry in (scopes.values() if isinstance(scopes, dict) else ()):
            if not isinstance(entry, dict):
                continue
            label, used = entry.get(self.peaks.SCOPE_LABEL_KEY), entry.get(self.peaks.SCOPE_USED_KEY)
            if not label or not isinstance(label, str) or not isinstance(used, int) or isinstance(used, bool) or used < 0:
                continue
            usage[label] = min(used, usage.get(label, used))
        return usage

    def place(self, task, busy_cards, view, alternation=None):
        """这一轮判这一条放不放得下、放哪台（judge_placement）；判成永远放不下而多机配置里有一台此刻连不上的，改判此刻放不下、排着等它回来再判
        （一台停机不把只有它放得下的任务判死）。弄坏开关 down-host-judged-never 下照改前判永远放不下。"""
        placement = self.judge_placement(task, busy_cards, view, alternation)
        if placement.kind != "never" or lib.broken("down-host-judged-never", self.environment):
            return placement
        snapshot = view.snapshot(bool(task.get("gpu")))
        down = [host for host in MACHINE_ORDER if snapshot.expected.get(host) and not snapshot.reachable.get(host)]
        if not down:
            return placement
        names = "、".join(self.machines[host].label for host in down)
        return dataclasses.replace(placement, kind="blocked", why=f"{placement.why}；{names}此刻用不了，等它回来再判放不放得下")

    def judge_placement(self, task, busy_cards, view, alternation=None):
        """这一轮判这一条放不放得下、放哪台：预测用存的那一份（prediction_of），余量与待涨量用这一轮读的那一份（view）。
        alternation 给了（没满时限的）：每台再照交替判（Alternation.blocked），内存放得下而交替不让起的改判放不下；封着组而插得进空的（Alternation.gap_fill）
        交替不挡、内存照常判，起的 Placement 记插空那一句（gap_filled），内存放不下的排队原因写插空那一句与内存那一句。"""
        gpu = bool(task.get("gpu"))
        snapshot = view.snapshot(gpu)
        if snapshot.hold:
            return Placement("blocked", why="自证的假余量写了 hold")
        hosts = self.candidate_hosts(task, snapshot)
        if task.get("role") == "shard":   # 这一片要拿到它所在那一台在 HOSTS 里的地址（task_server_lib.py 文件头「分片」②）：认不出就不起
            address, problem = self.shard_host_address(LOCAL)
            if not address:
                return Placement("unmet", why=problem)
        peer_only = self.peer_only_entry(task) if hosts == [PEER] else None
        if len(hosts) == 1 and not snapshot.reachable.get(hosts[0]) and not lib.broken("pinned-peer-unavailable-waits", self.environment):
            only, why = self.machines[hosts[0]].label, snapshot.down.get(hosts[0], "")   # 钉了台（重排换台、弄坏开关下只在第二台跑的入口）而那一台此刻连不上
            if peer_only:
                return Placement("never", why=f"钉在{only}（只在第二台跑的入口 {peer_only[0]}：{peer_only[1]}）而{only}此刻用不了：{why}")
            return Placement("never", why=f"钉在{only}而{only}此刻用不了：{why}")
        if peer_only and snapshot.reachable.get(PEER) and not self.host_eligible(task, PEER, snapshot):
            return Placement("never", why=f"只在第二台跑的入口 {peer_only[0]}（{peer_only[1]}），这一条派不过去：工作目录要是仓根、参数里的本机路径要带得过去、"
                                          "不点只在本机的目录与写本机工作区的工具（task_server_lib.py 文件头「挑机器」）")
        if gpu and not snapshot.cards:
            if not lib.broken("cardless-gpu-launched", self.environment):
                return Placement("unmet", why="GPU 任务而多机配置没点名能用的卡（GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED、GPU_CARDS_BY_PRIORITY）"
                                              + (f"：{snapshot.cards_problem}" if snapshot.cards_problem else ""))
            gpu = False   # 弄坏开关 cardless-gpu-launched：没点名卡也照不要卡的任务起
        if gpu and task.get("gpu_cards_problem"):   # 源文件头的张数声明写坏了：不猜几张，打回（文件头 ②「GPU」的张数）
            return Placement("unmet", why=f"要几张卡的声明写坏了：{task['gpu_cards_problem']}。出路：改成一个正整数（# task-server-gpu-cards: <张数>）再重交")
        prediction = self.prediction_of(task)
        if prediction.memory_history == lib.PIN_UNREADABLE_HISTORY:   # 钉了而配置读不到：不起，记 run-condition-unmet（pinned_prediction）
            return Placement("unmet", why=self.pin_unreadable_reasons.get(task["id"], f"这条命令钉了上限而多机配置的 {lib.PIN_SOURCE} 此刻读不到"))
        verdicts, free_after_pending, remaining, booked = {}, {}, {}, {}
        unusable = {}   # {台: 这一轮不派它的原因}：余量量不到（探测失败、cgroup 读不出）、总上限不比余量大、钉值不大于 0（文件头 ②「余量」）
        for host in hosts:
            if host not in snapshot.reports:
                if snapshot.down_kind.get(host) != DISK_FLOOR_KIND:   # 盘余量不到下限的那台不是量不到余量，照 down_text 写
                    unusable[host] = self.host_limit(prediction, host, snapshot)[1]
                continue
            limit, why = self.host_limit(prediction, host, snapshot)
            if limit is None:
                unusable[host] = why
                continue
            pending = view.pending_of(host)
            pending_memory = pending[0]
            on_host, verdicts[host] = self.host_admission(task, prediction, host, snapshot, view, gpu, pending)   # 内存、编译目录占盘（文件头 ②「编译目录占盘」）与 CPU 闸
            booked[host] = on_host.memory_booked
            if alternation is not None:
                why = alternation.blocked(task, host, booked[host], snapshot)   # 内存上放不下的也走一遍：按队列先后轮到它就超过界的照样封组
                gap = alternation.gap_fills.get((task["id"], host), "")
                if why and verdicts[host].fits:
                    verdicts[host] = dataclasses.replace(verdicts[host], fits=False, why=why)
                elif gap and not verdicts[host].fits:   # 插得进空、内存放不下：两句都写
                    verdicts[host] = dataclasses.replace(verdicts[host], why=f"{gap}；但{verdicts[host].why}")
            free_after_pending[host] = snapshot.reports[host]["memory"]["free"] - pending_memory
            remaining[host] = min(free_after_pending[host], int(snapshot.reports[host]["memory"].get("slice_room") or 0) - pending_memory,
                                  self.host_total(host, snapshot) - pending_memory)   # 均衡挑台用：空闲、slice 余量、总上限减已记的账三者的小者
        # 哪一台此刻用不了（盘余量不到下限、余量查不成、连不上这类；task_server_lib.py 文件头「挑机器」）：排队原因带上那一句，不只写别的台放不下
        down_text = "".join(f"（{self.machines[host].label}此刻用不了：{snapshot.down[host]}）" for host in MACHINE_ORDER
                            if snapshot.expected.get(host) and snapshot.down.get(host) and host not in snapshot.reports and host not in unusable)
        unusable_text = "；".join(unusable.values())
        if unusable and len(unusable) == len(hosts) and not lib.broken("margin-unmeasured-waits", self.environment):
            # 能派的每一台这一轮都量不到余量（或钉值不大于 0）：运行条件不满足，不起，不退回固定的数（task_server_lib.py 文件头「错误类型」的 run-condition-unmet）
            return Placement("unmet", why=f"能派的每一台这一轮都给不出上限，不起：{unusable_text}。出路：照上面那一句修好那台的余量读数"
                                          "（singlefs-heavy.slice 与它的子 cgroup 的 memory.current、memory.stat 读得了，systemctl --user 查得了，第二台连得上）或多机配置，再重交")
        if not verdicts:
            return Placement("blocked", why="能派的那几台此刻都查不到余量" + down_text + (f"（{unusable_text}）" if unusable_text else ""))
        common = {"gpu_booked": prediction.gpu_predicted if gpu else 0, "history": prediction.memory_history, "disk_booked": self.build_disk_prediction(task)}
        if gpu:
            return self.gap_filled(self.place_on_card(prediction, verdicts, hosts, snapshot, busy_cards, common, booked, int(task.get("gpu_cards") or 1)),
                                   task, alternation)
        fitting = [host for host in hosts if host in verdicts and verdicts[host].fits]
        if (lib.broken("peer-overflow-blocked", self.environment) and hosts[:1] == [LOCAL] and LOCAL in verdicts and not verdicts[LOCAL].fits
                and not verdicts[LOCAL].never):
            fitting = [host for host in fitting if host == LOCAL]   # 弄坏开关：照改前本机在前的放不下也不派第二台
        if task.get("role") == "shard":
            chosen = fitting[0] if fitting else None   # 分片照计划：放得下的里取排在最前的那台，不按均衡改台
        elif lib.broken("balance-ignored", self.environment):
            chosen = fitting[0] if fitting else None   # 弄坏开关：放得下的里取排在最前的，不比哪台更空
        else:   # 放得下的里挑放下之后剩余占这台总上限比例大的，一样大取排在前面的（task_server_lib.py 文件头「挑机器」）
            chosen = max(fitting, key=lambda host: ((remaining[host] - booked[host]) / max(1, self.host_total(host, snapshot)),
                                                    -hosts.index(host))) if fitting else None
        if chosen is None and lib.broken("launch-when-unplaceable", self.environment):
            chosen = next(iter(verdicts))
        if chosen is not None:
            return self.gap_filled(Placement("start", host=chosen, cap=verdicts[chosen].cap, booked=booked[chosen], **common), task, alternation)
        reasons = "；".join(f"{self.machines[host].label}：{verdict.why}" for host, verdict in verdicts.items())
        reasons += down_text + (f"；{unusable_text}（这一轮不派）" if unusable_text else "")
        if all(verdict.never for verdict in verdicts.values()) and not unusable:   # 有台量不到余量：它下一轮可能放得下，不判永远放不下
            return Placement("never", why=reasons)
        return Placement("blocked", why=reasons)

    @staticmethod
    def gap_filled(placement, task, alternation):
        """起在一台上的这一条是封着组插空进去的（Alternation.gap_fill）：Placement 记上插空那一句，起的时候记进尝试的 gap_fill；别的照原样。"""
        if placement.kind != "start" or alternation is None:
            return placement
        return dataclasses.replace(placement, gap_fill=alternation.gap_fills.get((task["id"], placement.host), ""))

    def launch_recheck(self, task, placement, view):
        """起之前复核（文件头 ②「起之前复核」）：现算这一条的预测、现读它要去的那台的余量与待涨量（缓存作废），按现读的数再判一次那台放不放得下。
        → (放得下时照现算的数改过上限与记账的 Placement，放不下时 None, 放不下的那一句)。现读的那份换进这一轮，后面的照它判。
        弄坏开关 launch-recheck-skipped 下不复核；launch-when-unplaceable 那一种照它的样子起（放不下也起），同样不复核。"""
        if lib.broken("launch-recheck-skipped", self.environment) or lib.broken("launch-when-unplaceable", self.environment):
            return placement, ""
        host = placement.host
        snapshot_gpu = bool(task.get("gpu"))
        uses_card = bool(placement.card)
        prediction = self.fresh_prediction(task)
        if prediction.memory_history == lib.PIN_UNREADABLE_HISTORY:   # 排队那一轮读得到、起之前读不到：这一轮不起，下一轮 place 记 run-condition-unmet
            self.selftest_read(f"recheck task={task['id']} host={host} fits=0")
            return None, self.pin_unreadable_reasons.get(task["id"], f"钉值读不到（多机配置的 {lib.PIN_SOURCE}）")
        snapshot = self.unstarted_slices_filled(self.resources.recheck(snapshot_gpu, host, view.snapshot(snapshot_gpu)))
        view.refresh(host, snapshot_gpu, snapshot)
        limit, why = self.host_limit(prediction, host, snapshot)   # 现读的这一份里这台量不到余量、或钉值不大于 0：这一轮不起（文件头 ②「余量」）
        if limit is None:
            self.selftest_read(f"recheck task={task['id']} host={host} fits=0")
            return None, why
        on_host, verdict = self.host_admission(task, prediction, host, snapshot, view, uses_card)
        self.selftest_read(f"recheck task={task['id']} host={host} fits={int(verdict.fits)}")
        if not verdict.fits:
            return None, f"{lib.HOST_NAMES.get(host, host)}：{verdict.why}"
        return dataclasses.replace(placement, cap=verdict.cap, booked=on_host.memory_booked, gpu_booked=prediction.gpu_predicted if uses_card else 0,
                                   history=prediction.memory_history), ""

    def cpu_problem(self, task, host, snapshot):
        """CPU 闸：这台此刻的负载（余量报告里 1 分钟负载与可跑进程数的大者）+ 起了不到 CPU_SETTLE_SECONDS 秒的尝试的线程 + 这一条的线程
        超过核数 × CPU_OVERSUBSCRIPTION，交排队原因；放得下或报告里没有 CPU 那一项交空串。弄坏开关 cpu-gate-ignored 下不判。"""
        if lib.broken("cpu-gate-ignored", self.environment):
            return ""
        cpu = (snapshot.reports[host]["memory"] or {}).get(self.peaks.CPU_REPORT_KEY) or {}
        cpus = cpu.get("cpus")
        loads = [value for value in (cpu.get("load1"), cpu.get("procs_running")) if value is not None]
        if not cpus or not loads:
            return ""
        now = time.time()
        settling = sum(int(attempt.get("threads") or 0) for attempt, _ in self.store.select("attempts", "state IN ('starting', 'running')")
                       if attempt.get("host") == host and attempt.get("started_at") and 0 <= now - float(attempt["started_at"]) < CPU_SETTLE_SECONDS)
        wanted = self.threads_of(task) or 1
        load, limit = max(loads), cpus * CPU_OVERSUBSCRIPTION
        if load + settling + wanted <= limit:
            return ""
        return (f"CPU：这台此刻负载 {load:.0f}、{CPU_SETTLE_SECONDS} 秒内刚起的还记着 {settling} 线程，再起这一条（{wanted} 线程）"
                f"超过 {cpus} 核 × {CPU_OVERSUBSCRIPTION} = {limit}，等负载降下来")

    def host_total(self, host, snapshot):
        """这台的总上限（字节）：Snapshot.total（slice 的总上限，另有收到的数取小者）。"""
        return snapshot.total(host)
    def place_on_card(self, prediction, verdicts, hosts, snapshot, busy_cards, common, booked, wanted=1):
        """要卡的这一条挑卡（文件头 ②「GPU」）：wanted 是要同一台的几张（gpu_cards_wanted；文件头 ②「GPU」的张数）。照卡的优先级看，头一张空卡所在、
        那台的空卡凑得出 wanted 张、内存又放得下的那一台，取它空卡里照优先级的前 wanted 张整组分给这一条（原样那几项用空格接，GPU_CARDS_BY_PRIORITY 交这一串，
        清场照这一串整组清）；多机配置点名的卡哪台都凑不出 wanted 张的判永远放不下（never-fits；有卡停用着时只排着，停用到点就回来）。"""
        cards = list(reversed(snapshot.cards)) if lib.broken("card-priority-ignored", self.environment) else snapshot.cards
        usable = [card for card in cards if card.side in hosts and card.side in verdicts]
        free = [card for card in usable if (card.address, card.index) not in busy_cards or lib.broken("card-busy-ignored", self.environment)]
        free_by_host = {}
        for card in free:
            free_by_host.setdefault(card.side, []).append(card)
        for card in free:
            group = free_by_host[card.side][:wanted]
            if len(group) == wanted and verdicts[card.side].fits:
                return Placement("start", host=card.side, cap=verdicts[card.side].cap, booked=booked[card.side], card=" ".join(each.entry for each in group), **common)
        if not usable:
            return Placement("unmet", why="多机配置点名的卡没有一张在这个任务能派的那几台上")
        configured = {}
        for card in usable:
            configured[card.side] = configured.get(card.side, 0) + 1
        if wanted > 1 and max(configured.values()) < wanted and not self.quarantined_cards:
            counted = "、".join(f"{self.machines[host].label} {count} 张" for host, count in configured.items())
            return Placement("never", why=f"这一条要同一台的 {wanted} 张卡（源文件头的张数声明），多机配置点名的卡每台至多 {max(configured.values())} 张（{counted}）：哪台都凑不出")
        if all(verdicts[host].never for host in {card.side for card in usable}):
            return Placement("never", why="；".join(verdict.why for verdict in verdicts.values()))
        short = not any(len(group) >= wanted for group in free_by_host.values())
        return Placement("blocked", waiting_for_card=short,
                         why=(f"等空卡（要同一台的 {wanted} 张）" if wanted > 1 else "等空卡") if short else "；".join(verdict.why for verdict in verdicts.values()))

    # ── ③ 起 ──
    def threads_of(self, task):
        """这一件的线程数（task_server_lib.py 文件头「线程」）：一律是登记的每台默认重活线程数（用户：「每次都开31」）。"""
        return self.threads

    def attempt_command(self, task, placement, label, environment, build):
        """这一次尝试在单元里起的那一条（文件头 ③「起」）：两台同一段——命令外面套 capped.sh <线程数>，再交那一台的起法（Machine.command：本机是内存包装，
        第二台是 multi-host-run.sh --peer；build 是这一次的编译目录，没给是 None）；要卡的最外层再套清场。"""
        threads = [] if lib.broken("threads-unset", self.environment) else [str(self.threads_of(task))]
        capped = ["bash", lib.THREAD_WRAPPER, *threads] if threads else []
        argv = self.machines[placement.host].command(task, placement, label, environment, capped, build)
        # 要卡的一律套清场（文件头 ③「起」）；弄坏开关 entry-quiesce-skipped 下按名字认的那几样照改前当成自己清场、不套
        unwrapped = lib.broken("entry-quiesce-skipped", self.environment) and str(task.get("gpu_reason", "")).startswith(NAMED_GPU_REASON)
        if task.get("gpu") and not unwrapped:
            argv = ["bash", self.gpu_quiesce, "--"] + argv
        return argv

    def shard_plan_directory(self, task):
        return os.path.join(self.shard_directory(task["parent_id"]), "plan")

    def shard_host_address(self, host):
        """分片的一片所在那一台在 HOSTS 里的地址（task_server_lib.py 文件头「分片」②的 TASK_SERVER_SHARD_HOST）：→ (地址或空串, 认不出的原因)。
        服务照 Resources.addresses 认出自己这一台：派本机（local）的是这一台的地址，派第二台（peer）的是另一台的地址。"""
        addresses, problem = self.resources.addresses()
        if addresses is None:
            return "", (f"分片要把这一片所在那一台在多机配置 {lib.HOSTS_KEY} 里的地址交给命令（{lib.SHARD_HOST_VARIABLE}），此刻认不出 {lib.HOSTS_KEY} 里哪一项是"
                        f"服务所在这一台：{problem}")
        return self.machines[host].address(addresses), ""

    def shard_environment(self, task, placement, label, number, environment, address):
        """分片那几步的环境（task_server_lib.py 文件头「分片」）：→ 这一片的产物目录（共享工作根下，两台同一个绝对路径）。
        address 是这一片所在那一台在 HOSTS 里的地址（shard_host_address）；弄坏开关 shard-host-role-passed 下照改前给 local / peer。"""
        role = task["role"]
        directory = self.shard_directory(task["parent_id"])
        lib.ensure_private_directory(directory)
        plan = self.shard_plan_directory(task)
        lib.ensure_private_directory(plan)
        environment[lib.SHARD_PHASE_VARIABLE] = role
        environment[lib.SHARD_PLAN_VARIABLE] = plan
        if role == "merge":
            environment[lib.SHARD_OUTPUTS_VARIABLE] = os.path.join(directory, "outputs.tsv")
            found = self.store.task(task["parent_id"])
            if found is not None and found[1] and not lib.broken("shard-missing-unnamed", self.environment):   # 父任务校验不过的 claim_attempt 拒、这一次不起
                environment[lib.SHARD_MISSING_VARIABLE] = ",".join(str(index) for index in found[0].get("missing_shards") or [])
            return ""
        if role == "plan":
            return ""
        environment[lib.SHARD_VARIABLE] = f"{task['shard_index']}/{task['shard_count']}"
        environment[lib.SHARD_HOST_VARIABLE] = placement.host if lib.broken("shard-host-role-passed", self.environment) else address
        output = os.path.join(directory, f"output-{task['shard_index']}-attempt-{number}")
        lib.ensure_private_directory(output)
        environment[lib.SHARD_OUTPUT_VARIABLE] = output
        return output

    def start_attempt(self, task, placement):
        """起这一次尝试，交它的尝试记录；没起（判放得下之后被取消了、父任务已结了局、状态变了）交 None（文件头 ③「起」）。
        先照现读的任务（不拿调用方手里的旧字典）备好目录、环境与说明，再拿这条任务的锁：claim_attempt 一个事务里判 queued 且没取消、分尝试号、
        插尝试、改任务（TS-01、TS-03），之后起单元，起完才放锁——取消在起之前到的不起，在起之后到的等单元起完再停、回读。
        弄坏开关 cancel-race-unguarded 下照改前：开头判一次、不拿锁、先插尝试再拿调用方那份旧字典整行写回任务。"""
        unguarded = lib.broken("cancel-race-unguarded", self.environment)
        current = self.store.task(task["id"])
        if current is None or not current[1] or current[0].get("state") != "queued" or current[0].get("cancel_requested"):
            if current is not None and current[1] and current[0].get("state") == "queued" and current[0].get("cancel_requested"):
                self.finalize(current[0], "cancelled", None, "判放得下之后、起之前被取消了，没起")
            return None   # 这一轮判放得下之后、起之前被取消了（接口那一侧当场收尾）：不起
        if not unguarded:
            task = current[0]
        shard_address = ""
        if task.get("role") == "shard":   # 这一片所在那一台的地址（文件头「分片」②）：place 判过认得出，这里现取；配置在这一瞬之间改坏了的照 run-condition-unmet 收尾、不起
            shard_address, problem = self.shard_host_address(placement.host)
            if not shard_address:
                self.finalize(task, "run-condition-unmet", None, problem)
                return None
        number = int(task.get("attempt_count", 0)) + 1
        if number > 1 and lib.broken("retry-overwrites-attempts", self.environment):
            for earlier, _ in self.store.attempts_of(task["id"]):   # 弄坏开关：重试这一次覆盖前几次的记录
                self.store.delete_attempt(earlier["id"])
        label = f"task-{self.store.tag}-{task['id']}-{number}"
        directory = os.path.join(self.configuration.attempts_directory, label)
        for path in (self.configuration.logs_directory, self.configuration.attempts_directory, directory):
            lib.ensure_private_directory(path)
        log = os.path.join(self.configuration.logs_directory, f"task-{self.store.tag}-{task['id']}.attempt-{number}.log")
        environment = {name: value for name, value in self.environment.items() if not name.startswith("TASK_SERVER_")}
        environment.update(task["environment"])
        environment.update(TASK_SERVER_TASK=str(task["id"]), TASK_SERVER_ATTEMPT=str(number))
        peer_cleanup_file = os.path.join(directory, "peer-cleanup.json") if placement.host == PEER else ""
        if peer_cleanup_file:
            write_json_atomically(peer_cleanup_file, {"status": "not-started", "label": label})
            environment["TASK_SERVER_PEER_CLEANUP_FILE"] = peer_cleanup_file
        if not lib.broken("memory-cap-not-passed", self.environment):   # 这一次的内存上限（字节，十进制）交给命令（文件头 ③「环境」）
            environment[lib.MEMORY_CAP_VARIABLE] = str(int(placement.cap))
        if not lib.broken("source-repository-not-passed", self.environment):   # 提交方那一份仓的根交给命令（文件头 ③「环境」）：第二台照它拼两台一致的路径
            environment[lib.SOURCE_REPOSITORY_VARIABLE] = os.path.realpath(self.peer_root)
        if not lib.broken("shared-roots-not-passed", self.environment):   # 两个共享根交给命令（文件头 ③「环境」）：第二台原样带本机的值，不经 ~ 取
            environment.update(self.attempt_shared_roots(task))
        machine = self.machines[placement.host]
        for each in self.machines.values():
            environment.pop(each.admission_variable, None)
        if not (machine.admission_break and lib.broken(machine.admission_break, self.environment)):   # 那一台最外层那一层只收服务直接起的这一次（文件头 ③「起」）
            environment[machine.admission_variable] = f"{task['id']}.{number}"
        if not lib.broken("attempt-label-not-passed", self.environment):   # 两台的 scope 照它对到这一次尝试上（server_pending、memory_peaks_lib.py 文件头「在跑的 scope」）
            environment[lib.ATTEMPT_LABEL_VARIABLE] = label
        environment[lib.THREAD_ADMITTED_VARIABLE] = f"{task['id']}.{number}"   # capped.sh 只认服务套的这一层（文件头 ③「起」）
        if task.get("gpu"):
            environment[lib.QUIESCE_ADMITTED_VARIABLE] = f"{task['id']}.{number}"   # 清场包装只认服务套的这一层
        if task.get("session"):
            environment[lib.SESSION_VARIABLE] = task["session"]
        if placement.card:
            environment["GPU_CARDS_BY_PRIORITY"] = placement.card
        if lib.in_selftest(self.environment):
            for name in (lib.SELFTEST_MARK_VARIABLE, "TASK_SERVER_SELFTEST_WORLD"):
                environment[name] = self.environment.get(name, "")
        # 给内存包装的两样（文件头 ③「环境」；包装读了就 unset，第二台的由 multi-host-run.sh 设给那边那一层）：退出时峰值写进尝试目录（task_server_lib.py 文件头
        # 「资源记录」的退出时峰值，弄坏开关 peak-read-only-by-sampling 下不给），现场目录在共享工作根下（同一份文件头「现场」）
        exit_peak_file = "" if lib.broken("peak-read-only-by-sampling", self.environment) else os.path.join(directory, lib.EXIT_PEAK_FILE_NAME)
        if exit_peak_file:
            environment[lib.PEAK_OUTPUT_VARIABLE] = exit_peak_file
        scene_directory = os.path.join(self.scenes_root, label)
        environment[lib.SCENE_VARIABLE] = scene_directory
        build_directory, made_here, reused = "", True, False
        if (task.get("compiles") and not any(task["environment"].get(name) for name in lib.TARGET_DIRECTORY_VARIABLES)
                and not lib.broken("target-directory-unset", self.environment)):
            # 文件头「编译目录」：两台同一段，照那一台的接口（Machine.build_directory）给这一次尝试自己的编译目录；服务在本机建的任务记下结局时删（设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的留给 task clean），
            # 那一台的起法自己建的（multi-host-run.sh 在第二台本地盘上 builds/<标签>/）跑完由它删，服务只记下在哪
            build_directory, made_here = machine.build_directory(label)
            kept = self.kept_build_directory(task, number)
            reusable = made_here or not lib.broken("peer-retry-build-directory-fresh", self.environment)   # 弄坏开关：照改前第二台那一份不接着用
            if kept is not None and reusable and kept.get("host") == placement.host and not lib.broken("retry-build-directory-fresh", self.environment):
                # 重排的这一次与上一次在同一台、上一次的编译目录还留着：接着用那一份（task_server_lib.py 文件头「编译目录」的重试复用；两台同一段：
                # 本机经 CARGO_TARGET_DIR 指过去，第二台经 multi-host-run.sh --reuse-build-directory 点名那一份，PeerMachine.command）
                build_directory, reused = kept["build_directory"], True
            elif kept is not None and not lib.broken("retry-build-directory-kept", self.environment):
                # 换了台：上一次留着的那一份这一次用不上，当场删（有进程开着里面的文件就不删、写告警）
                self.remove_build_directory(task, kept, None)
                self.store.update("attempts", kept)
            if made_here and build_directory:
                os.makedirs(build_directory, mode=0o700, exist_ok=True)
                environment["CARGO_TARGET_DIR"] = os.path.join(build_directory, "target")
        shard_output = self.shard_environment(task, placement, label, number, environment, shard_address) if task.get("role") in lib.SHARD_ROLES else ""
        argv = self.attempt_command(task, placement, label, environment, BuildDirectory(build_directory, reused) if build_directory else None)
        specification = os.path.join(directory, "spec.json")
        detached = not lib.broken("no-detach", self.environment)
        unit = f"singlefs-task-{self.store.tag}-{task['id']}-{number}.service" if detached else ""
        write_json_atomically(specification, {"unit": unit,
            "argv": argv, "cwd": task["cwd"], "environment": environment, "log": log, "directory": directory, "host": placement.host,
            "gpu": bool(placement.card), "fast": lib.fast(self.environment),
            "fake_scope_directory": lib.selftest_override("TASK_SERVER_FAKE_SCOPE_DIRECTORY", self.environment),
            "nvidia_smi": lib.selftest_override("TASK_SERVER_NVIDIA_SMI", self.environment) or "nvidia-smi",
            "build_directory": build_directory if made_here else "", "wake_socket": self.configuration.socket,
            "wake_skipped": lib.broken("runner-wake-skipped", self.environment), "leftovers_kept": lib.broken("leftover-processes-kept", self.environment),
            "leftovers_unit_unchecked": lib.broken("leftovers-outside-own-unit", self.environment), "exit_peak_file": exit_peak_file})
        attempt = {"task_id": task["id"], "number": number, "host": placement.host, "memory_cap": placement.cap, "memory_cap_text": lib.mebibytes_text(placement.cap),
                   "booked": placement.booked, "gpu_booked": placement.gpu_booked, "disk_booked": placement.disk_booked, "card": placement.card,
                   "prediction": placement.history, "gap_fill": placement.gap_fill, "label": label, "peer_cleanup_file": peer_cleanup_file,
                   "unit": unit, "log": log, "directory": directory,
                   "spec": specification, "state": "starting", "session": "", "command_key": task["command_key"], "command": task.get("command"), "started_at": round(time.time(), 3),
                   "threads": self.threads_of(task) if not lib.broken("threads-unset", self.environment) else None, "build_directory": build_directory,
                   "build_directory_removed": bool(build_directory) and not made_here, "build_directory_reused": reused, "build_family": task.get("build_family", ""),
                   "shard_output": shard_output, "scene_directory": scene_directory}
        self.selftest_pause("start-checked")
        if unguarded:
            attempt = self.store.insert("attempts", attempt)
            task.update(state="running", attempt_count=number)
            self.store.update("tasks", task, force=True)
            self.crash_point("before-unit")
            self.launch(task, attempt)
            return attempt
        with self.store.task_lock(task["id"]):
            try:
                attempt, task = self.store.claim_attempt(task["id"], number, attempt, {"state": "running"}, between=lambda: self.crash_point("attempt-claimed"),
                                                         split=lib.broken("attempt-claim-split", self.environment))
            except StoreRefused as refused:
                shutil.rmtree(directory, ignore_errors=True)   # 备好的这一次没起：它自己的尝试目录与编译目录删掉
                if build_directory and made_here and not reused:   # 接着用的上一次那一份不在这里删：任务有了结局时照常删
                    shutil.rmtree(build_directory, ignore_errors=True)
                found = self.store.task(task["id"])
                if refused.kind == "tampered" and found is not None:
                    self.tampered(task["id"], found[0], "任务记录")
                elif refused.kind in ("cancelled", "parent-finished") and found is not None and found[1]:
                    self.finalize(found[0], "cancelled", None, f"{refused}：没起")
                else:
                    say(f"  · 任务 {task['id']} 这一次不起：{refused}")
                return None
            self.crash_point("before-unit")
            self.selftest_pause("claimed")
            self.launch(task, attempt)
        return attempt

    def crash_point(self, name):
        """只在自证里：TASK_SERVER_SELFTEST_CRASH_POINT=<点>:<文件> 那份文件在时，服务在这一点当场退出（文件头 ③「只起一次」与分片恢复那几格用）。"""
        setting = lib.selftest_override("TASK_SERVER_SELFTEST_CRASH_POINT", self.environment)
        point, _, arm = setting.partition(":")
        if point == name and arm and os.path.exists(arm):
            os.remove(arm)
            say(f"  · 自证造的崩溃：{name}")
            os._exit(70)

    def selftest_pause(self, name):
        """只在自证里：TASK_SERVER_SELFTEST_PAUSE_POINT=<点>:<目录>，目录里 arm-<点> 在时删掉它、建 reached-<点>，等 release-<点> 在了才往下走
        （至多 SELFTEST_PAUSE_SECONDS 秒；精确卡住起单元与取消交错的那几格用）。"""
        setting = lib.selftest_override("TASK_SERVER_SELFTEST_PAUSE_POINT", self.environment)
        point, _, directory = setting.partition(":")
        if point != name or not directory or not os.path.exists(os.path.join(directory, f"arm-{name}")):
            return
        os.remove(os.path.join(directory, f"arm-{name}"))
        with open(os.path.join(directory, f"reached-{name}"), "w", encoding="utf-8"):
            pass
        deadline = time.monotonic() + SELFTEST_PAUSE_SECONDS
        while time.monotonic() < deadline and not os.path.exists(os.path.join(directory, f"release-{name}")):
            time.sleep(0.05)

    def launch(self, task, attempt):
        if not os.path.isdir(task["cwd"]):
            started, why = False, f"工作目录 {task['cwd']} 不在"
        else:
            started, why = self.start_unit(task, attempt)
        if started is None:   # 回执超时：起没起不确定（TS-06），照单元名读回再定，判不了留在 starting、账照记
            attempt.update(start_uncertain_at=round(time.time(), 3), note=why)
            readback = self.start_readback(attempt)
            if readback == "started":
                attempt["state"] = "running"
                self.store.update("attempts", attempt)
                say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试：{why}；照单元名读回它起了，照常跑")
                return
            self.store.update("attempts", attempt)
            self.alert("start-uncertain", task["id"], f"任务 {task['id']} 第 {attempt['number']} 次尝试：{why}；照单元名 {attempt.get('unit')} 读回 {readback}，"
                       "起没起还判不了：尝试留在 starting、账照记",
                       f"服务每一轮对账照单元名读回：起了就照常跑，{self.start_uncertain_grace_seconds:.0f} 秒里单元一直不在、起跑标记也没有才记 unit-start-failed；"
                       "要停它照常 task-server/task cancel")
            return
        if not started and not lib.broken("start-failure-ignored", self.environment):
            attempt.update(state="ended", outcome="unit-start-failed", ended_at=round(time.time(), 3), note=why)
            self.store.update("attempts", attempt)
            self.finalize(task, "unit-start-failed", None, why)
            return
        self.crash_point("after-unit")
        attempt["state"] = "running"
        self.store.update("attempts", attempt)
        say(f"  · 任务 {task['id']} 第 {attempt['number']} 次尝试起了：{lib.HOST_NAMES.get(attempt['host'], attempt['host'])}、上限 {attempt['memory_cap_text']}"
            f"{'、卡 ' + attempt['card'] if attempt['card'] else ''}、单元 {attempt['unit'] or '（服务的子进程）'}、日志 {attempt['log']}")

    def start_readback(self, attempt):
        """起单元的回执超时之后照预先定好的单元名读回（TS-06）：→ started（单元在跑，或起跑标记、结果已在）、not-found（单元不在、标记也没有）、
        或一句读不到的原因。"""
        result_path, started_path, _ = self.attempt_paths(attempt)
        if os.path.exists(started_path) or os.path.exists(result_path):
            return "started"
        states = unit_states(attempt.get("unit", ""), self.environment)
        if states is None:
            return "读不到（systemctl --user show 起不来或超时）"
        if states[1] in lib.RUNNING_UNIT_STATES:
            return "started"
        return "not-found" if states[0] == "not-found" else f"LoadState={states[0]}、ActiveState={states[1]}"

    def start_unit(self, task, attempt):
        """起尝试的单元：→ (True 起了 / False 确定没起 / None 回执超时、起没起不确定, 说明)。超时只算不确定（TS-06：systemd-run 是发给用户级 systemd 的请求，
        调用端超时不说明管理器没收下），弄坏开关 start-timeout-as-failure 下照改前当没起。"""
        runner = [sys.executable, "-B", lib.RUNNER_SCRIPT, attempt["spec"]]
        if not attempt.get("unit"):   # 弄坏开关 no-detach：起成服务自己的子进程
            child = subprocess.Popen(runner, cwd=task["cwd"], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            self.children.append(child)
            attempt["child_pid"] = child.pid
            return True, ""
        command = [self.systemd_run, "--user", f"--unit={attempt['unit']}", "--collect", "--quiet", "-p", "KillMode=mixed",
                   "-p", f"TimeoutStopSec={UNIT_STOP_SECONDS}", *([] if lib.broken("nofile-limit-dropped", self.environment) else ["-p", nofile_property()]),
                   f"--working-directory={task['cwd']}", "--", *runner]
        timeout = float(lib.selftest_override("TASK_SERVER_SELFTEST_START_TIMEOUT_SECONDS", self.environment) or START_TIMEOUT_SECONDS)
        try:
            completed = subprocess.run(command, capture_output=True, text=True, env=self.environment, stdin=subprocess.DEVNULL, timeout=timeout)
        except subprocess.TimeoutExpired as error:
            if lib.broken("start-timeout-as-failure", self.environment):
                return False, f"systemd-run 起不来（{type(error).__name__}: {error}）"
            return None, f"systemd-run {timeout:g} 秒没回执（起没起不确定）"
        except OSError as error:
            return False, f"systemd-run 起不来（{type(error).__name__}: {error}）"
        if completed.returncode != 0:
            lines = [line for line in (completed.stderr + completed.stdout).splitlines() if line.strip()]
            return False, f"systemd-run 退 {completed.returncode}：{lines[-1].strip() if lines else '没有输出'}"
        return True, ""

    # ── 提交、取消、取结局（接口那一侧调）──
    def gpu_need(self, task):
        """(要不要卡, 为什么)：只照认法认命令（文件头 ②「GPU」），不看记录；要卡的一律由服务套清场（文件头 ③「起」）。"""
        if lib.broken("gpu-unrecognized", self.environment):
            return False, ""
        try:
            found = recognize_gpu(self.project_code_runs, shlex.join(task["command"]), task["cwd"], dict(task["environment"]))
        except Exception as error:   # 切词库出事：退回只看命令本身那一层（cargo 的特性、按名字认的脚本），不当成不要卡就放过
            say(f"  ! 任务的命令切词出了事（{type(error).__name__}: {error}），GPU 只按命令本身那一层认")
            outermost = types.SimpleNamespace(words=task["command"], launcher=None)
            found = ("带 gpu 特性、跑编出来的代码的 cargo" if cargo_uses_gpu(task["command"], self.project_code_runs, task["cwd"])
                     else f"带 gpu 特性的 python 入口 {python_entry_uses_gpu(task['command'])}" if python_entry_uses_gpu(task["command"])
                     else f"{NAMED_GPU_REASON} {os.path.basename(task['command'][0])}" if script_uses_gpu(outermost) else None)
        if found:
            return True, found
        return False, ""

    def compiles(self, task):
        """命令里命令位置上有会编的 cargo（task_server_lib.py 文件头「编译目录」；认法是 .claude/hooks/lib_workspace_cargo_lock.py 的 cargo_invocations，只 import 不改），
        或是不带 --server-binary 的临时 prune-server 包装（包装自己在编译目录里现编 prune-server；lib.temporary_service_builds_server，文件头「资源记录」命令键那一段后）：
        外层这一条算会编的，派到哪台服务都给编译目录（不带它，本机不给编译目录、包装退 2，第二台却由 multi-host-run.sh 照例给）。"""
        if lib.temporary_service_builds_server(task["command"], task.get("environment"), self.environment):
            return True
        try:
            runs = self.cargo_lock.cargo_invocations(shlex.join(task["command"]), task["cwd"], self.project_code_runs.shell_words, self.project_code_runs, dict(task["environment"]))
            return any(run.subcommand in self.cargo_lock.BUILDING_SUBCOMMANDS for run in runs)
        except Exception as error:   # 切词库出事：只看命令本身那一层，宁可多给一个编译目录
            say(f"  ! 任务的命令切词出了事（{type(error).__name__}: {error}），会不会编只看命令本身那一层")
            index = lib.command_word_index(task["command"])
            words = task["command"][index:]
            return bool(words) and os.path.basename(words[0]) == "cargo" and any(word in self.cargo_lock.BUILDING_SUBCOMMANDS for word in words[1:3])

    def cargo_runs(self, task):
        """命令里命令位置上起的每一条 cargo（认法同 compiles）；切词库出事交空表（编译族与张数照没有 cargo 办，不挡提交）。"""
        try:
            return self.cargo_lock.cargo_invocations(shlex.join(task["command"]), task["cwd"], self.project_code_runs.shell_words, self.project_code_runs,
                                                     dict(task["environment"]))
        except Exception as error:
            say(f"  ! 任务的命令切词出了事（{type(error).__name__}: {error}），编译族与要几张卡照没有 cargo 办")
            return []

    def build_family_of(self, task):
        """这条命令的编译族（task_server_lib.py 文件头「资源记录」的编译族）：命令位置上起的入口脚本文件头声明了的照声明（lib.declared_build_family）；
        不然会编的照它起的每一条会编的 cargo（lib.cargo_build_family，几条用「 ; 」接）；都不是交空串。弄坏开关 build-family-ignored 下交空串。"""
        if lib.broken("build-family-ignored", self.environment):
            return ""
        declared = lib.declared_build_family(task["command"], task["cwd"], self.environment)
        if declared or not task.get("compiles"):
            return declared
        families = []
        for run in self.cargo_runs(task):
            if run.subcommand in self.cargo_lock.BUILDING_SUBCOMMANDS:
                root = self.cargo_lock.work_tree_root(run.directory) if run.directory else None
                workspace = os.path.relpath(run.directory, root) if root else os.path.basename(run.directory or "")
                families.append(lib.cargo_build_family(run.rest, workspace))
        return " ; ".join(dict.fromkeys(families))

    def gpu_cards_wanted(self, task):
        """要卡的这一条要同一台的几张卡（task_server_lib.py 文件头「GPU」的张数）：命令位置上起的脚本文件头、编出来的二进制的同名源、
        cargo test / run / bench 点名的包的源（lib.cargo_card_sources）里写的张数取最大，没写是一张。→ (张数, 写坏的那一句)。
        不要卡的交 (1, "")。弄坏开关 gpu-card-count-ignored 下一律一张（改前的样子）。"""
        if not task.get("gpu") or lib.broken("gpu-card-count-ignored", self.environment):
            return 1, ""
        words = task["command"]
        index = lib.command_word_index(words)
        sources = []
        if index < len(words):
            path = os.path.realpath(os.path.join(task["cwd"], words[index]))
            sources.append(path)
            binary = lib.compiled_binary_source(path, self.environment)
            if binary:
                sources.append(binary)
        for run in self.cargo_runs(task):
            if run.subcommand in GPU_RUNNING_CARGO_SUBCOMMANDS:
                sources += lib.cargo_card_sources(run.rest, self.environment)
        return lib.declared_gpu_cards(list(dict.fromkeys(sources)))

    def nested_problem(self, command, cwd):
        """提交的命令里套了不收的包装、写了不收的参数：命令外层与 bash / sh -c 引号里每一条命令位置上的命令都照 nested_dispatch 判（只钻 -c 那一段，
        不读被起脚本的正文：脚本自己在里面怎么起不归提交这一关）；交第一条说明，没有交空串。"""
        found = lib.nested_dispatch(command, self.environment)
        if found or lib.broken("inner-commands-unchecked", self.environment):
            return found
        return self.inner_nested(command, cwd, 0)

    def inner_nested(self, words, directory, depth):
        index = lib.command_word_index(words)
        if index >= len(words) - 1 or words[index] != "-c" or depth >= MAXIMUM_LAUNCHER_DEPTH:
            return ""
        try:   # 照原样切：按 && || ; | & 与括号切成一条条命令，外壳（包装、nice、timeout）不剥，交给 nested_dispatch 判
            lexer = shlex.shlex(words[index + 1], posix=True, punctuation_chars=True)
            lexer.whitespace_split = True
            tokens = list(lexer)
        except ValueError:   # 引号不配对这类拆不开的：照外层判过的放行（钩子那一层另判写法）
            return ""
        commands, current = [], []
        for token in tokens:
            if token and set(token) <= set("&|;()"):
                commands.append(current)
                current = []
            else:
                current.append(token)
        commands.append(current)
        for command in commands:
            if not command:
                continue
            found = lib.nested_dispatch(command, self.environment) or self.inner_nested(command, directory, depth + 1)
            if found:
                return found
        return ""

    def revalidate_queued(self):
        """服务起来时把还在排的任务照现行的提交规则再核一遍（用户：「按规矩来 如果有问题就按照严格的逻辑，由脚本打回给提交放重发」）：
        环境变量名单、包装前缀与资源参数、编译目录；核不过的记 resubmit-required、写明哪一条，经 wait 叫醒提交它的会话改了重交。服务自己建的分片子任务不核。"""
        for task, valid in self.store.select("tasks", "state = 'queued'", order="id"):
            if not valid:
                continue
            if task.get("parent_id") or lib.broken("queued-not-revalidated", self.environment):
                continue
            problems = []
            _, refused = lib.split_environment(task.get("environment") or {}, self.environment)
            if refused:
                problems.append("环境变量 " + "、".join(name for name, _ in refused) + " 现在不收")
            nested = self.nested_problem(task["command"], task["cwd"])
            if nested:
                problems.append(nested)
            target = lib.target_directory_refusal(task.get("environment") or {}, task["command"], task["cwd"], self.environment)
            if target:
                problems.append(str(target))
            if problems:
                say(f"  · 任务 {task['id']} 照现行规则不再收（{'；'.join(problems)}）：打回提交它的会话重发")
                self.finalize(task, "resubmit-required", None,
                              "服务规则改了，这条照现行规则不再收：" + "；".join(problems) + "。改了再 task-server/task submit 重交")

    def command_key(self, command, explicit_key):
        """命令键（task_server_lib.py 文件头「资源记录」的命令键）：先剥掉不改跑法的环境变量前缀与临时 prune-server 的包装（lib.key_words），
        再照 memory_peaks_lib.command_key 换算；包着临时 prune-server 的前面另接 lib.TEMPORARY_SERVICE_KEY_MARK（同一 scope 里多一份服务进程，与不包的不共用记录）。"""
        key = self.peaks.command_key(lib.key_words(command, self.environment), explicit_key)
        if lib.temporary_service_wrapped(command, self.environment):
            return f"{lib.TEMPORARY_SERVICE_KEY_MARK} {key}"[:self.peaks.KEY_LENGTH_LIMIT]
        return key

    def submit(self, command, cwd, environment, session, process_id):
        task = {"command": command, "cwd": cwd, "environment": environment, "session": session, "submitter_pid": process_id,
                "submitted_at": round(time.time(), 3), "command_key": self.command_key(command, environment.get("RUN_WITH_MEMORY_CAP_KEY")),
                "state": "queued", "outcome": "", "exit_code": None, "collected": False, "next_cap": None, "next_host": "", "outcome_counts": {},
                "note": "", "cancel_requested": False, "attempt_count": 0, "task_id": 0}
        task["gpu"], task["gpu_reason"] = self.gpu_need(task)
        task["gpu_cards"], task["gpu_cards_problem"] = self.gpu_cards_wanted(task)
        task["compiles"] = self.compiles(task)
        task["build_family"] = self.build_family_of(task)
        entry = lib.sharded_entry(command, cwd, self.environment)
        if entry:   # 文件头 ④：父任务不进队，先排计划那一步
            task.update(state="sharded", sharded_entry=entry, phase="plan", shard_count=0)
        item_entry, items = lib.item_entry(command, cwd)
        estimating = False
        if item_entry and items and not entry:   # 按项估预计用时（task_server_lib.py 文件头「资源记录」的预计用时）：先起估算子任务，它有了结局才进队
            task.update(item_entry=item_entry, items=items)
            estimating = not lib.broken("item-estimate-skipped", self.environment)
            if estimating:
                task.update(state="estimating", phase="estimate")
        task = self.store.insert("tasks", task)
        self.operation("submit", task["id"], session, process_id, shlex.join(command)[:300])
        if entry:
            self.crash_point("parent-inserted")
            self.create_child(task, "plan")
        if estimating:
            self.create_child(task, lib.ESTIMATE_ROLE)
        self.wake()
        return task

    def clean(self, task_id, session, process_id):
        """→ (HTTP 状态码, 回话)。删这条任务（分片的连同子任务）留下的编译目录与现场：只许提交它的会话、有了结局才删（task_server_lib.py 文件头「编译目录」「现场」）。
        第二台那一趟的编译目录 multi-host-run.sh 自己删，这里不管。子任务或尝试的记录校验不过（关联列改过这类，TS-05）的一份都不删。"""
        found = self.store.task(task_id)
        if found is None:
            return 404, {"detail": f"没有任务 {task_id}"}
        task, valid = found
        if not valid:
            self.tampered(task_id, task, "任务记录")
            return 409, {"detail": f"任务 {task_id} 的记录校验不过，已记 record-tampered，不认这次删除"}
        if task.get("session") != session and not lib.broken("clean-session-unchecked", self.environment):
            self.operation("clean-refused", task_id, session, process_id, "不是提交它的会话")
            return 403, {"detail": f"任务 {task_id} 不是这个会话提交的（提交它的会话与这一次的不同），不许删它的编译目录"}
        if task.get("state") != "finished":
            return 409, {"detail": f"任务 {task_id} 还没有结局（{task.get('state')}）：等 wait 拿到结局再 clean"}
        try:
            owners = [task] + self.children_of(task_id)
        except StoreRefused as refused:
            if refused.kind != "tampered":
                raise
            return 409, {"detail": self.children_refused(task_id, refused, "一份编译目录与现场都不删")}
        attempts = [(record, record_valid) for owner in owners for record, record_valid in self.store.attempts_of(owner["id"])]
        unverified = [record.get("id") for record, record_valid in attempts if not record_valid]
        if unverified:
            self.alert("record-tampered", task_id, f"任务 {task_id} 名下有校验不过的尝试记录（行 {unverified}）：一份编译目录都不删",
                       "查是谁直接写了库（只有 task-server 读写它）")
            return 409, {"detail": f"任务 {task_id} 名下有校验不过的尝试记录（行 {unverified}，关联列与正文对不上或写进之后被改过）：一份都不删"}
        removed, left = [], []
        for attempt, _ in attempts:
            directory = attempt.get("build_directory")
            if not directory or attempt.get("build_directory_removed") or not os.path.lexists(directory):   # 已删的、起法那一层自己删的（第二台）不在本机删
                continue
            size = directory_bytes(directory)
            if lib.broken("clean-removes-nothing", self.environment):
                done, lines = True, []
            else:
                done, lines = self.build_directories.remove_run_build_directory(directory)
            if done and not os.path.lexists(directory):
                attempt["build_directory_removed"] = True
                self.store.update("attempts", attempt)
                removed.append({"path": directory, "bytes": size})
            else:
                left.append({"path": directory, "bytes": size, "kind": "编译目录", "why": "；".join(line.strip() for line in lines) or "删完还在"})
        if not lib.broken("scene-left-by-clean", self.environment):   # 各次尝试的现场一并删（task_server_lib.py 文件头「现场」）
            scenes_removed, scenes_left = self.remove_scenes(task_id)
            removed += scenes_removed
            left += scenes_left
        # 没删成的写明是哪一份、为什么（照 remove_run_build_directory 交回的原因）：操作日志与回话都带；弄坏开关 clean-failure-unexplained 下照改前只记份数
        explained = not lib.broken("clean-failure-unexplained", self.environment)
        reasons = "；".join(f"{item['kind']} {item['path']}：{item['why']}" for item in left)
        self.operation("clean", task_id, session, process_id, f"删了 {len(removed)} 份、没删成 {len(left)} 份" + (f"：{reasons}" if left and explained else ""))
        if left:
            return 409, {"detail": {"text": reasons if explained else f"没删成 {len(left)} 份", "removed": removed, "left": left if explained else []}}
        return 200, {"removed": removed, "left": []}

    def cancel(self, task_id, session, process_id):
        """→ (HTTP 状态码, 回话)。只许提交它的会话；在跑的停单元、回读停了才回 stopped。
        这条任务的锁拿着判：在队里的当场记 cancelled（起单元那一侧拿同一把锁，判放得下之后起之前到的就不起）；已经起了的等起完、停单元、回读（TS-01）。"""
        found = self.store.task(task_id)
        if found is None:
            return 404, {"detail": f"没有任务 {task_id}"}
        task, valid = found
        if not valid:
            self.tampered(task_id, task, "任务记录")
            return 409, {"detail": f"任务 {task_id} 的记录校验不过，已记 record-tampered，不认这次取消"}
        if task.get("session") != session and not lib.broken("cancel-session-unchecked", self.environment):
            self.operation("cancel-refused", task_id, session, process_id, "不是提交它的会话")
            return 403, {"detail": f"任务 {task_id} 不是这个会话提交的（提交它的会话与这一次的不同），不许取消"}
        if task.get("state") == "estimating":   # 还在估算（task_server_lib.py 文件头「资源记录」的预计用时）：停估算子任务，父任务不进队
            reply = self.cancel_estimating(task, session, process_id)
            if reply is not None:
                return reply
            found = self.store.task(task_id)   # 判的这一会儿估算收尾、父任务已进队：照不分片的任务取消
            if found is None or not found[1]:
                return 409, {"detail": f"任务 {task_id} 的记录读不回或校验不过，不认这次取消"}
            task = found[0]
        try:
            has_children = any(child.get("role") in lib.SHARD_ROLES for child in self.children_of(task_id))   # 估算子任务不算：估算完的父任务照不分片的取消
        except StoreRefused as refused:
            if refused.kind != "tampered":
                raise
            has_children = True   # 名下有校验不过的子任务行：照分片的父任务办，cancel_sharded 判出来回 409、一个单元都不停
        parent = bool(task.get("sharded_entry")) or task.get("state") == "sharded" or has_children   # 分片的父任务：名下有分片子任务的也算
        if parent and not lib.broken("sharded-cancel-finished-first", self.environment):
            return self.cancel_sharded(task, session, process_id)
        if not parent:
            refusal = self.tampered_cancel_refusal(task_id, task)
            if refusal is not None:
                return refusal
        if task.get("state") == "finished" and not parent:
            if lib.broken("finished-cancel-overwrites", self.environment):
                self.finalize(task, "cancelled", None, "弄坏开关 finished-cancel-overwrites：已有结局的也改记 cancelled", overwrite=True)
            return 200, {"stopped": True, "state": "finished", "outcome": task.get("outcome"), "detail": "已经有结局了，没什么可取消"}
        if task.get("state") == "finished":   # 弄坏开关 sharded-cancel-finished-first：改前的样子，分片的父任务已有结局就不再看子任务
            return 200, {"stopped": True, "state": "finished", "outcome": task.get("outcome"), "detail": "已经有结局了，没什么可取消"}
        if task.get("state") == "sharded":   # 弄坏开关 sharded-cancel-finished-first：改前的样子，先给父任务记结局、再停子任务，第二次取消见它已有结局就回停了
            self.store.change("tasks", task_id, {"cancel_requested": True})
            self.operation("cancel", task_id, session, process_id)
            self.finalize_parent(task, None, "cancelled", None, "提交它的会话取消了整件")
            problems = self.cancel_children(task_id)
            if problems:
                return 500, {"stopped": False, "detail": "；".join(problems)}
            return 200, {"stopped": True, "state": "finished", "outcome": "cancelled", "detail": "整件取消了：在排的子任务没起，在跑的单元已停"}
        # 分片的子任务被人手单独取消（不是整件被取消、不是兄弟片没成被服务撤掉）记 cancel_origin=session：只撤这一片，别的片照跑、合并照起（drive_parent_phase）
        alone = {"cancel_origin": "session"} if task.get("parent_id") and not lib.broken("shard-cancel-ends-whole", self.environment) else {}
        with self.store.task_lock(task_id):
            try:
                task = self.store.change("tasks", task_id, dict(alone, cancel_requested=True),
                                         guard=lambda current: "已结了局" if current.get("state") == "finished" else "")
            except StoreRefused as refused:
                if refused.kind != "guard":
                    raise
                outcome = (self.store.task(task_id) or ({},))[0].get("outcome")
                if outcome == "record-tampered":   # 判的这一会儿对账判出校验不过、记了 record-tampered：照样不报停了（TS-05）
                    return self.tampered_cancel_refusal(task_id, dict(task, outcome=outcome))
                return 200, {"stopped": True, "state": "finished", "outcome": outcome, "detail": "已经有结局了，没什么可取消"}
            self.operation("cancel", task_id, session, process_id)
            if task.get("state") == "queued" and not lib.broken("queued-cancel-left-queued", self.environment):
                self.finalize(task, "cancelled", None, "在队里时被取消，没起")
                return 200, {"stopped": True, "state": "finished", "outcome": "cancelled", "detail": "在队里时取消，没起"}
        self.selftest_pause("cancel-unlocked")
        status, reply = self.stop_running(task_id)
        if (status != 200 or lib.broken("cancel-outcome-unchecked", self.environment)   # 弄坏开关：照改前停了就回 stopped，不等结局写回、不核是不是 cancelled
                or lib.broken("cancel-no-readback", self.environment)):
            return status, reply
        return self.cancel_outcome_reply(task_id, reply)

    def cancel_outcome_reply(self, task_id, stopped):
        """停了单元、回读过之后（stopped 是 stop_running 的回话）等对账把结局写回，照写回的结局回（TS-03）：结局是 cancelled 才回 stopped；
        别的结局（取消到的时候命令已自己跑完、退 0 记了 succeeded 这类）回 409、点名那个结局，不报停了；至多等 cancel_settle_seconds，等不到回 500、不报停了。"""
        deadline = time.monotonic() + self.cancel_settle_seconds
        while True:
            found = self.store.task(task_id)
            if found is not None and found[1] and found[0].get("state") == "finished":
                break
            if found is not None and not found[1]:
                return self.tampered_cancel_refusal(task_id, dict(found[0], outcome="record-tampered"))
            if time.monotonic() >= deadline:
                return 500, {"stopped": False, "accepted": True, "state": (found or ({},))[0].get("state"),
                             "detail": f"单元已停、回读过，等了 {self.cancel_settle_seconds:.0f} 秒结局还没写回：结局写回之前不报停了；"
                                       f"task-server/task status {task_id} 看结局"}
            self.wake()
            time.sleep(CANCEL_SETTLE_POLL_SECONDS)
        task = found[0]
        if task.get("outcome") == "record-tampered":
            return self.tampered_cancel_refusal(task_id, task)
        if task.get("outcome") != "cancelled":
            return 409, {"stopped": False, "state": "finished", "outcome": task.get("outcome"), "exit_code": task.get("exit_code"),
                         "detail": f"取消到的时候任务 {task_id} 已自己结束：结局 {task.get('outcome')}（退 {task.get('exit_code')}），不是取消；产物照这个结局认"}
        return 200, dict(stopped, state="finished", outcome="cancelled", detail="单元已停（回读 ActiveState 不在跑、cgroup.procs 空），结局 cancelled 已写回")

    def tampered_cancel_refusal(self, task_id, task):
        """取消一条不分片的任务之前核它名下的记录（TS-05：停止必须拒绝未验证的记录）：名下有校验不过的尝试记录（关联列与正文对不上这类），
        或任务已因校验不过记了 record-tampered（服务不认它名下的单元，一个都没停），回 409、不报停了；都核得过交 None。"""
        unverified = [record.get("id") for record, valid in self.store.attempts_of(task_id) if not valid]
        if unverified:
            self.alert("record-tampered", task_id, f"任务 {task_id} 名下有校验不过的尝试记录（行 {unverified}）：取消不停任何单元",
                       "查是谁直接写了库（只有 task-server 读写它）；那几个单元要停由人照单元名核过再停")
            return 409, {"stopped": False, "detail": f"任务 {task_id} 名下有校验不过的尝试记录（行 {unverified}，关联列与正文对不上或写进之后被改过）：一个单元都不停"}
        if task.get("outcome") == "record-tampered":
            return 409, {"stopped": False, "detail": f"任务 {task_id} 已因记录校验不过记了 record-tampered：服务不认它名下的单元、一个都没停，不报停了；"
                                                     "要停由人照单元名核过再停"}
        return None

    def cancel_estimating(self, parent, session, process_id):
        """取消还在估算的任务（task_server_lib.py 文件头「资源记录」的预计用时）：拿父任务的锁、判还在估算才记 cancel_requested、阶段改 cancelling
        （drive_item_estimate 判 cancel_requested，记上之后就不进队）；估算子任务照取消整件那样停（cancel_children：在排的当场收尾，在跑的停单元、回读），
        都停了给父任务记 cancelled、回 stopped；有没停的回 500（父任务留在 cancelling，再取消照样重停）。判的时候估算已收尾、父任务已进队的交 None，调用方照不分片的任务取消。"""
        parent_id = parent["id"]
        with self.store.task_lock(parent_id):
            try:
                parent = self.store.change("tasks", parent_id, {"cancel_requested": True, "phase": "cancelling"},
                                           guard=lambda current: "" if current.get("state") == "estimating" else "不在估算")
            except StoreRefused as refused:
                if refused.kind != "guard":
                    raise
                return None
        self.operation("cancel", parent_id, session, process_id)
        try:
            problems = self.cancel_children(parent_id)
        except StoreRefused as refused:
            if refused.kind != "tampered":
                raise
            return 409, {"stopped": False, "accepted": True, "detail": self.children_refused(parent_id, refused, "一个单元都不停")}
        if problems:
            return 500, {"stopped": False, "accepted": True, "state": parent.get("state"), "phase": parent.get("phase"),
                         "detail": "取消已记下（任务留在估算的 cancelling，下一次 cancel 照样重停估算子任务）：" + "；".join(problems)}
        self.finalize_parent(parent, None, "cancelled", None, "提交它的会话在估算时取消了（估算子任务停了，没进队）")
        return 200, {"stopped": True, "state": "finished", "outcome": "cancelled", "detail": "估算子任务停了、回读过，任务没进队"}

    def cancel_sharded(self, parent, session, process_id):
        """取消分片的整件（TS-04）：父任务没结局的先记 cancel_requested、阶段改成 cancelling（不先记结局）；在排的子任务当场收尾，在跑的逐个停、回读；
        都停了、读回过才给整件记 cancelled、回 stopped。有没停的回 500（取消已记下、整件留在 cancelling），再次取消照样重查、重停活动的子任务；
        父任务已有结局而子任务还在跑的同样停它们。弄坏开关 sharded-cancel-children-left 下照改前只给父任务记 cancelled、子任务不动。"""
        parent_id = parent["id"]
        was_finished = parent.get("state") == "finished"
        with self.store.task_lock(parent_id):
            if parent.get("state") != "finished":
                with contextlib.suppress(StoreRefused):
                    parent = self.store.change("tasks", parent_id, {"cancel_requested": True, "phase": "cancelling"},
                                               guard=lambda current: "已结了局" if current.get("state") == "finished" else "")
        self.operation("cancel", parent_id, session, process_id)
        if lib.broken("sharded-cancel-children-left", self.environment) and parent.get("state") != "finished":
            self.finalize_parent(parent, None, "cancelled", None, "提交它的会话取消了整件")
            return 200, {"stopped": True, "state": "finished", "outcome": "cancelled", "detail": "（弄坏开关 sharded-cancel-children-left：子任务没动）"}
        try:   # 关联列改过的（TS-05）：认不出哪些子任务是它的，一个都不停
            children = self.children_of(parent_id)
            problems = self.cancel_children(parent_id)
        except StoreRefused as refused:
            if refused.kind != "tampered":
                raise
            return 409, {"stopped": False, "accepted": parent.get("state") != "finished", "detail": self.children_refused(parent_id, refused, "一个单元都不停")}
        if problems:
            return 500, {"stopped": False, "accepted": True, "state": parent.get("state"), "phase": parent.get("phase"),
                         "detail": "取消已记下（整件留在 cancelling，还没回读到停的子任务下一次 cancel 照样重停）：" + "；".join(problems)}
        current = self.store.task(parent_id)
        if current is not None and current[0].get("state") != "finished":
            self.finalize_parent(current[0], None, "cancelled", None, "提交它的会话取消了整件")
        outcome = ((self.store.task(parent_id) or ({},))[0]).get("outcome")
        if not was_finished:
            return 200, {"stopped": True, "state": "finished", "outcome": outcome, "detail": "整件取消了：在排的子任务没起，在跑的单元已停、回读过"}
        if not any(record.get("state") != "finished" for record in children):
            return 200, {"stopped": True, "state": "finished", "outcome": outcome, "detail": "已经有结局了，没什么可取消"}
        return 200, {"stopped": True, "state": "finished", "outcome": outcome, "detail": "整件早有结局；还在跑的子任务单元已停、回读过"}

    def cancel_children(self, parent_id):
        """取消父任务名下还没结局的子任务（整件被取消，cancel_origin=parent）：在排的记 cancelled，在跑的停单元、回读；→ 没停成的那几句。
        弄坏开关 parent-cancel-spares-running 下照兄弟片没成那一路办（cancel_queued_children）：只撤在排的，在跑的不停。"""
        if lib.broken("parent-cancel-spares-running", self.environment):
            self.cancel_queued_children(self.children_of(parent_id), "parent", "整件被取消，没起")
            return []
        problems = []
        for child in self.children_of(parent_id):
            if child.get("state") == "finished":
                continue
            with self.store.task_lock(child["id"]):
                try:
                    child = self.store.change("tasks", child["id"], {"cancel_requested": True, "cancel_origin": "parent"},
                                              guard=lambda current: "已结了局" if current.get("state") == "finished" else "")
                except StoreRefused:
                    continue
                if child.get("state") == "queued":
                    self.finalize(child, "cancelled", None, "整件被取消，没起")
                    continue
            status, reply = self.stop_running(child["id"])
            if status != 200:
                problems.append(f"子任务 {child['id']}：{reply.get('detail', '')}")
        return problems

    def stop_running(self, task_id):
        """停这个任务在跑的那几次尝试的单元、回读停了（取消那一侧用）。→ (HTTP 状态码, 回话)。
        名下有校验不过的尝试记录（关联列 task_id 与正文对不上这类，TS-05）的一个都不停、回 409：认不出那一行是谁的。"""
        if lib.broken("cancel-no-readback", self.environment):
            return 200, {"stopped": True, "state": "running", "detail": "（弄坏开关 cancel-no-readback：没停、没回读）"}
        rows = self.store.attempts_of(task_id)
        unverified = [record.get("id") for record, valid in rows if not valid]
        if unverified:
            self.alert("record-tampered", task_id, f"任务 {task_id} 名下有校验不过的尝试记录（行 {unverified}）：取消不停任何单元",
                       "查是谁直接写了库（只有 task-server 读写它）；那几个单元要停由人照单元名核过再停")
            return 409, {"stopped": False, "detail": f"任务 {task_id} 名下有校验不过的尝试记录（行 {unverified}，关联列与正文对不上或写进之后被改过）：一个单元都不停"}
        attempts = [record for record, _ in rows if record.get("state") in ACTIVE_ATTEMPT_STATES]
        skip_stop = lib.selftest_override("TASK_SERVER_SELFTEST_STOP_FAILS", self.environment)
        skipped = bool(skip_stop) and os.path.exists(skip_stop)
        if skipped:   # 自证：演这一次 systemctl stop 没停成（只这一次）
            os.remove(skip_stop)
        problems = []
        for attempt in attempts:
            if not attempt.get("unit"):
                child = next((child for child in self.children if child.pid == attempt.get("child_pid")), None)
                if child is not None:
                    child.send_signal(signal.SIGTERM)
                    with contextlib.suppress(subprocess.TimeoutExpired):
                        child.wait(timeout=UNIT_STOP_SECONDS)
                continue
            if not skipped:
                with contextlib.suppress(OSError, subprocess.TimeoutExpired):
                    subprocess.run(["systemctl", "--user", "stop", attempt["unit"]], capture_output=True, env=self.environment, stdin=subprocess.DEVNULL,
                                   timeout=UNIT_STOP_SECONDS + 30)
            states = unit_states(attempt["unit"], self.environment)
            processes = unit_processes(attempt["unit"], self.environment)
            if states is None or states[1] in lib.RUNNING_UNIT_STATES or processes is None or processes:
                problems.append(f"单元 {attempt['unit']} 停了之后回读：状态 {states[1] if states else '读不到'}、cgroup 里进程 "
                                f"{'读不到' if processes is None else len(processes)} 个")
            else:
                peer_clean, peer_note = self.confirm_peer_cleanup(attempt)
                if not peer_clean:
                    problems.append(peer_note)
        self.wake()
        if problems:
            return 500, {"stopped": False, "detail": "；".join(problems)}
        return 200, {"stopped": True, "state": "running", "units": [attempt["unit"] for attempt in attempts],
                     "detail": "单元已停（回读 ActiveState 不在跑、cgroup.procs 空），结局由对账写回"}

    def mark_collected(self, task_id):
        with contextlib.suppress(StoreRefused):
            self.store.change("tasks", task_id, lambda current: {"collected": True, "collected_at": round(time.time(), 3)},
                              guard=lambda current: "" if current.get("state") == "finished" and not current.get("collected") else "不用记")

    def summary(self, task_id):
        found = self.store.task(task_id)
        if found is None:
            return None
        task, valid = found
        if not valid:
            self.tampered(task_id, task, "任务记录")
            task, valid = self.store.task(task_id)
        attempts = []
        for attempt, attempt_valid in self.store.attempts_of(task_id):
            attempt["valid"] = attempt_valid
            attempts.append(attempt)
        now = time.time()
        for attempt in attempts:
            if attempt.get("state") in ACTIVE_ATTEMPT_STATES and not lib.broken("running-progress-unreported", self.environment):
                attempt["progress"] = self.running_progress(task, attempt, now)
        summary = {"task": task, "valid": valid, "attempts": attempts}
        pin = self.pin_record(task.get("command_key", "")) if task.get("command_key") else None
        if pin is not None:   # 这条命令钉了上限（文件头 ②「内存」的钉值）：status 写明是钉值、此刻多少
            summary["pinned_cap"] = self.pins_reply(task["command_key"])
        if task.get("state") in ("queued", "sharded"):
            summary["estimated_seconds"], basis = self.estimate_of(task)
            if not lib.broken("estimate-basis-unreported", self.environment):
                summary["estimate_basis"] = basis   # 这个数照哪种算法得的（客户端 status 照它打说明）
            if not lib.broken("waiting-reason-unreported", self.environment):
                summary["waiting_for"] = self.waiting_text(task)
        if task.get("sharded_entry"):
            try:
                children = self.children_of(task_id)
            except StoreRefused as refused:   # 名下有校验不过的子任务行：那几行不列、不认，写告警，children_unverified 点名（TS-05）
                if refused.kind != "tampered":
                    raise
                summary["children_unverified"] = self.children_refused(task_id, refused, "status 不列那几行")
                children = [record for record, valid in self.store.select("tasks", "task_id = ?", (task_id,)) if valid]
            summary["children"] = [self.summary(child["id"]) for child in children]
        if task.get("item_entry"):   # 按项估预计用时的估算子任务（task_server_lib.py 文件头「资源记录」的预计用时）：单列在 item_estimate，不进 children（客户端照 children 取结局那几次尝试）
            estimate = next((record for record, record_valid in self.store.select("tasks", "task_id = ?", (task_id,))
                             if record_valid and record.get("role") == lib.ESTIMATE_ROLE), None)
            if estimate is not None:
                summary["item_estimate"] = self.summary(estimate["id"])
            if task.get("state") == "estimating" and not lib.broken("waiting-reason-unreported", self.environment):
                summary["waiting_for"] = (f"先估算预计用时：估算子任务 {estimate['id']}（{estimate.get('state')}）有了结局才进队" if estimate is not None
                                          else "先估算预计用时：估算子任务还没建")
        return summary

    def running_progress(self, task, attempt, now):
        """在跑的那一次（文件头「卡住」）：已跑秒数、记录用时、日志多少秒没涨、到此刻平均在用核数（本机读 scope 的 cpu.stat）；量不到的是 None。"""
        elapsed = now - float(attempt.get("started_at") or now)
        seen = self.log_growth.get(attempt.get("id"))
        usage = self.machines[attempt["host"]].cpu_usec(attempt)
        cores = round(usage / 1e6 / elapsed, 3) if usage is not None and elapsed > 0 else None
        suspect = attempt.get("suspect") or {}
        return {"elapsed_seconds": round(elapsed, 3), "estimated_seconds": self.estimated_seconds(task),
                "suspect": suspect if suspect.get("active") else None,
                "log_quiet_seconds": None if seen is None else round(time.monotonic() - seen[1], 3), "cores": cores,
                "computing_quietly": bool(seen) and seen[4],
                "threads": self.threads_of(task), "stall_seconds": self.stall_seconds}

    def session_tasks(self, session):
        listed = []
        for task, valid in self.store.select("tasks", "session = ?", (session,)):
            if task.get("parent_id"):
                continue   # 子任务列在父任务名下（/tasks/<父任务号>）
            unclaimed = task.get("state") == "finished" and not task.get("collected")
            if unclaimed and lib.broken("unclaimed-not-listed", self.environment):
                continue
            if task.get("state") != "finished" or unclaimed:
                listed.append({"id": task["id"], "state": task.get("state"), "outcome": task.get("outcome"), "exit_code": task.get("exit_code"),
                               "unclaimed": unclaimed, "valid": valid, "command": shlex.join(task.get("command") or [])[:200],
                               "estimated_seconds": self.estimated_seconds(task) if task.get("state") in ("queued", "sharded") else None,
                               "phase": task.get("phase", ""),
                               "waiting_for": "" if lib.broken("waiting-reason-unreported", self.environment) else self.waiting_text(task)})
        return listed


# ── 接口 ──
def build_application(scheduler):
    from fastapi import FastAPI, HTTPException, Request
    from pydantic import BaseModel
    from starlette.concurrency import run_in_threadpool
    from typing import Dict, List

    application = FastAPI(title="singlefs task-server", docs_url=None, redoc_url=None, openapi_url=None)
    store = scheduler.store
    environment = scheduler.environment
    wait_fallback = scheduler.tick_seconds   # wait 的兜底：结局的通知丢了，至多等一轮主循环再查库（文件头「通知」）
    waiters = {}
    loop_holder = {}

    def release_waiters(task_id):
        for event in waiters.pop(task_id, []):
            event.set()

    def finished(task_id):
        loop = loop_holder.get("loop")
        if loop is not None:
            loop.call_soon_threadsafe(release_waiters, task_id)

    scheduler.finished_listeners.append(finished)

    class SubmitBody(BaseModel):
        command: List[str]
        cwd: str
        environment: Dict[str, str] = {}

    def peer_of(request):
        client = request.client
        if client is None or not str(client.host).startswith("uid-"):
            raise HTTPException(status_code=403, detail="读不到调用方的进程号与 uid（只认经 unix socket 来的）")
        if int(str(client.host)[4:]) != os.getuid():
            raise HTTPException(status_code=403, detail="调用方的 uid 不是服务自己的")
        return int(client.port)

    def session_of(process_id):
        return lib.session_of_peer(process_id, scheduler.boundary, environment)

    @application.get("/health")
    def health():
        """只做库写入试探与排队线程心跳；派活判排头能起却一直没起成、看进展那一句都读排队线程每一轮算好的（文件头「接口」），不现读余量、不起子进程。"""
        writable, problem = store.health_write()
        now = time.time()
        since = now - scheduler.last_tick
        starved = scheduler.starved_problem(now, fresh=True) if lib.broken("health-judges-head", environment) else scheduler.starved_text
        stalled = scheduler.progress_text
        problems = ([] if writable else [f"库写不进：{problem}"]) + ([starved] if starved else []) + ([stalled] if stalled else [])
        counts = {state: store.count("tasks", state) for state in ("queued", "running")}
        pause = None if lib.broken("pause-health-unreported", environment) else scheduler.pause   # 暂停不让 ok 变 false（文件头 ⑤「暂停」）
        other_running = None if pause is None else scheduler.running_outside_allowed(pause, counts["running"])
        report_allow = not lib.broken("pause-allow-unreported", environment)   # 放行的会话、别的会话在跑几个（文件头 ⑤「暂停」）
        return {"ok": not problems, "problem": "；".join(problems), "database_writable": writable, "scheduler_seconds_since_tick": round(since, 3),
                "tag": store.tag, "pid": os.getpid(), **counts, "nfs": scheduler.nfs_snapshot(), "peer": scheduler.peer_health(),
                "paused": pause is not None,
                "pause": None if pause is None else {"reason": pause["reason"], "since": pause["since"], "text": scheduler.pause_text(pause, counts["running"], other_running),
                                                     **({"allow_session": pause["allow_session"], "other_running": other_running} if report_allow else {})}}

    @application.post("/tasks")
    def submit(body: SubmitBody, request: Request):
        process_id = peer_of(request)
        if not body.command:
            raise HTTPException(status_code=400, detail="没给命令")
        if not os.path.isabs(body.cwd) or not os.path.isdir(body.cwd):
            raise HTTPException(status_code=400, detail=f"工作目录 {body.cwd} 不是在的绝对路径")
        carried, refused = lib.split_environment(body.environment, environment)
        if refused:
            raise HTTPException(status_code=400, detail={"refused": [list(pair) for pair in refused]})
        nested = scheduler.nested_problem(body.command, body.cwd)
        if nested:
            raise HTTPException(status_code=400, detail={"nested": nested})
        target = lib.target_directory_refusal(carried, body.command, body.cwd, environment)
        if target:
            raise HTTPException(status_code=400, detail={"target": target})
        refusal = scheduler.placement_refusal(body.command, body.cwd, carried)   # 派不到第二台的三种当场拒、不进库（文件头 ②「提交时拒」）
        if refusal:
            raise HTTPException(status_code=400, detail={"placement": refusal})
        task = scheduler.submit(body.command, body.cwd, carried, session_of(process_id), process_id)
        return {"id": task["id"], "session": task["session"], "gpu": task["gpu"], "gpu_reason": task["gpu_reason"],
                "sharded": task.get("sharded_entry", ""), "compiles": task.get("compiles", False)}

    @application.get("/tasks/{task_id}")
    def show(task_id: int, request: Request):
        peer_of(request)
        found = scheduler.summary(task_id)
        if found is None:
            raise HTTPException(status_code=404, detail=f"没有任务 {task_id}")
        return found

    @application.get("/tasks/{task_id}/wait")
    async def wait(task_id: int, request: Request, timeout: float = 20.0):
        process_id = peer_of(request)
        session = session_of(process_id)
        deadline = time.monotonic() + max(0.0, min(timeout, WAIT_LIMIT_SECONDS))
        loop_holder.setdefault("loop", asyncio.get_running_loop())
        while True:
            event = asyncio.Event()   # 先登记、再查库：查完到登记之间记下的结局不会漏掉
            waiters.setdefault(task_id, []).append(event)
            try:
                found = store.task(task_id)
                if found is None:
                    raise HTTPException(status_code=404, detail=f"没有任务 {task_id}")
                task, valid = found
                if not valid or task.get("state") == "finished":
                    if valid and session == task.get("session"):
                        await run_in_threadpool(scheduler.mark_collected, task_id)
                    summary = await run_in_threadpool(scheduler.summary, task_id)
                    reply = dict(summary, finished=True)
                    if valid and not lib.broken("attention-finished-unreported", scheduler.environment):   # 结局有了还没报的提醒（换卡重排跑成的）：打在理由行前面，退码仍是任务的
                        reply["attention"] = await run_in_threadpool(scheduler.unreported_attention, task_id, session)
                    return reply
                suspect = await run_in_threadpool(scheduler.unreported_suspect, task_id, session)
                if suspect is not None:   # 疑似卡住（文件头 ①「疑似卡住」）：任务照跑，wait 打一行、退出；一段只报一次
                    return {"finished": False, "state": task.get("state"), "suspect": suspect}
                attention = await run_in_threadpool(scheduler.unreported_attention, task_id, session)
                if attention:   # 要处置的提醒（文件头 ①「要处置的提醒」）：任务照旧在排或在跑，wait 打一行、退出；同一种同一档只报一次
                    return {"finished": False, "state": task.get("state"), "attention": attention}
                if time.monotonic() >= deadline:
                    return {"finished": False, "state": task.get("state")}
                with contextlib.suppress(asyncio.TimeoutError):
                    await asyncio.wait_for(event.wait(), timeout=max(0.01, min(wait_fallback, deadline - time.monotonic())))
            finally:
                with contextlib.suppress(ValueError, KeyError):
                    waiters[task_id].remove(event)

    @application.post("/tasks/{task_id}/cancel")
    def cancel(task_id: int, request: Request):
        process_id = peer_of(request)
        status, reply = scheduler.cancel(task_id, session_of(process_id), process_id)
        if status != 200:
            raise HTTPException(status_code=status, detail=reply.get("detail", reply))
        return reply

    @application.post("/tasks/{task_id}/clean")
    def clean(task_id: int, request: Request):
        process_id = peer_of(request)
        status, reply = scheduler.clean(task_id, session_of(process_id), process_id)
        if status != 200:
            raise HTTPException(status_code=status, detail=reply.get("detail", reply))
        return reply

    class PinBody(BaseModel):
        command: List[str]
        pinned: bool

    @application.post("/pins")
    def pin(body: PinBody, request: Request):
        process_id = peer_of(request)
        if not body.command:
            raise HTTPException(status_code=400, detail="没给命令")
        reply = scheduler.pin_cap(body.command, body.pinned, session_of(process_id), process_id)
        if reply.get("refused"):   # 钉值不大于 0（文件头 ②「内存」的钉值）：control 照 HTTP 状态打回、把这一句与出路打出来
            raise HTTPException(status_code=422, detail=reply["refused"])
        return reply

    class PinQuery(BaseModel):
        command: List[str]

    @application.post("/pins/show")
    def show_pin(body: PinQuery, request: Request):
        peer_of(request)
        return scheduler.pins_reply(scheduler.command_key(body.command, None))

    class PauseBody(BaseModel):
        reason: str = ""
        allow_session: str = ""

    @application.post("/pause")
    def pause_queue(body: PauseBody, request: Request):
        process_id = peer_of(request)   # 只认服务自己的 uid：同别的写操作
        return scheduler.pause_service(" ".join(body.reason.split())[:PAUSE_REASON_CHARACTERS], body.allow_session.strip(), session_of(process_id), process_id)

    @application.post("/resume")
    def resume_queue(request: Request):
        process_id = peer_of(request)
        return scheduler.resume_service(session_of(process_id), process_id)

    @application.post("/wake")
    def wake(request: Request):
        peer_of(request)   # 只认经 unix socket、同一 uid 来的（attempt_runner.py 写完结果调它）
        scheduler.wake()
        return {"ok": True}

    @application.get("/sessions/self")
    def mine(request: Request):
        process_id = peer_of(request)
        session = session_of(process_id)
        return {"session": session, "tasks": scheduler.session_tasks(session)}

    return application


def peer_credentials_protocol():
    from uvicorn.protocols.http.h11_impl import H11Protocol

    class PeerCredentialsProtocol(H11Protocol):
        """连上时读 SO_PEERCRED，把 (uid-<uid>, 进程号) 放进 scope 的 client（接口那一侧 peer_of 读它）。"""

        def connection_made(self, transport):
            super().connection_made(transport)
            connection = transport.get_extra_info("socket")
            try:
                raw = connection.getsockopt(socket.SOL_SOCKET, socket.SO_PEERCRED, struct.calcsize("3i"))
                process_id, uid, _ = struct.unpack("3i", raw)
                self.client = (f"uid-{uid}", process_id)
            except (OSError, AttributeError):
                self.client = None
    return PeerCredentialsProtocol


def bind_socket(path):
    """绑 unix socket（600）；同一路径上已有一份在回话交 None；路径太长或绑不上抛 lib.Unavailable（点名那个路径）。"""
    problem = lib.socket_path_problem(path)
    if problem:
        raise lib.Unavailable(problem, "把多机配置里的 TASK_SERVER_RUNTIME_DIRECTORY 改到短一点的目录（task_server_lib.py 文件头「路径」）")
    if os.path.exists(path):
        probe = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        probe.settimeout(2)
        try:
            probe.connect(path)
            probe.close()
            return None
        except OSError:
            os.remove(path)
    listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    try:
        listener.bind(path)
    except OSError as error:
        listener.close()
        raise lib.Unavailable(f"socket {path} 绑不上（{type(error).__name__}: {error}）",
                              "看那个目录在不在、属主与权限对不对；路径太长的照 task_server_lib.py 文件头「路径」把 TASK_SERVER_RUNTIME_DIRECTORY 写短") from None
    os.chmod(path, 0o600)
    listener.listen(128)
    return listener


PROCESS_NAME = b"task-server"   # 服务主线程的进程名（/proc/<pid>/comm）：两台 earlyoom 的 --avoid 照这个名字认它、整机内存吃紧时不挑它


def name_this_process():
    """把服务主线程的进程名设成 PROCESS_NAME（prctl PR_SET_NAME）；设不成只少这一层免杀，不挡服务起。"""
    with contextlib.suppress(OSError, AttributeError):
        import ctypes
        ctypes.CDLL(None, use_errno=True).prctl(15, PROCESS_NAME, 0, 0, 0)   # 15 = PR_SET_NAME


def serve():
    lib.refuse_selftest_variables_or_continue()
    name_this_process()
    environment = dict(os.environ)
    try:
        lib.require_modules(environment)
        missing_tools = [tool for tool in ("systemctl", "systemd-run") if shutil.which(tool) is None]
        if missing_tools:
            raise lib.Unavailable(f"PATH 上没有 {'、'.join(missing_tools)}", "装 systemd（用户级 systemd 起每一次尝试的单元）")
        configuration = lib.read_configuration(environment)
        key = lib.read_signing_key(configuration.signing_key_file)
        threads = registered_threads(environment)
        nfs_report = lib.require_nfs_share(configuration, environment, probe=True)   # 没有 NFS 共享不起（task_server_lib.py 文件头「NFS 共享」），在 Scheduler 建起之前
        lib.shared_data_root(environment)   # 共享数据根与工作根取不到（多机配置没写 SHARED_DATA_ROOT 这类）不起、退 78，出路点名那一行；Scheduler 照同一套取
        lib.work_root()
    except lib.Unavailable as error:
        lib.report_unavailable_and_exit(error, "task-server 起不来")
    os.umask(0o077)
    for directory in (configuration.state_directory, configuration.runtime_directory, configuration.logs_directory, configuration.attempts_directory,
                      configuration.shards_directory, configuration.scenes_directory, configuration.build_directory):
        lib.ensure_private_directory(directory)
    import uvicorn
    try:
        store = Store(configuration.database, key, environment)
    except lib.Unavailable as error:   # 库里有违反业务唯一的行（task_server_lib.py 文件头「库」）
        lib.report_unavailable_and_exit(error, "task-server 起不来")
    scheduler = Scheduler(configuration, store, environment, threads, nfs_report)
    try:
        listener = bind_socket(configuration.socket)
    except lib.Unavailable as error:
        lib.report_unavailable_and_exit(error, "task-server 起不来")
    if listener is None:
        say(f"  ✗ {configuration.socket} 上已有一份 task-server 在回话")
        say(f"  → 怎么办：只留一份；看 systemctl --user status {lib.UNIT_NAME}")
        return 3
    scheduler.revalidate_queued()   # 规则改了的：还在排的照现行提交规则再核，不合的打回提交方重发（文件头「排队」）
    scheduler.thread.start()
    scheduler.nfs_thread.start()
    threading.Thread(target=scheduler.sweep_finished_build_directories, name="task-server-build-sweep", daemon=True).start()   # 起来时清掉已有结局的任务留下的编译目录
    say(f"  · task-server 起了（进程 {os.getpid()}，socket {configuration.socket}，库 {configuration.database}，标签 {store.tag}；"
        f"NFS 共享根 {'、'.join(entry['root'] for entry in nfs_report['roots']) or '没判（弄坏开关 nfs-unchecked）'} 四项都过）")
    config = uvicorn.Config(build_application(scheduler), http=peer_credentials_protocol(), loop="asyncio", lifespan="off", log_level="warning",
                            access_log=False, timeout_graceful_shutdown=5)
    server = uvicorn.Server(config)
    try:
        server.run(sockets=[listener])
    finally:
        scheduler.stop()
        listener.close()
        with contextlib.suppress(FileNotFoundError):
            os.remove(configuration.socket)
        say("  · task-server 停了（在跑的任务在各自的单元里接着跑）")
    return 0


def main(arguments):
    if arguments == ["serve"]:
        return serve()
    print("  ✗ task_server.py 用法：serve", file=sys.stderr)
    print(f"  → 怎么办：服务由用户级 systemd 单元 {lib.UNIT_NAME} 起（{lib.INSTALL_COMMAND}）；提交任务用 task-server/task submit -- <命令…>", file=sys.stderr)
    return lib.USAGE_EXIT


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
