#!/usr/bin/env python3
"""task-server 的服务：唯一读写库的进程。FastAPI 跑在 uvicorn 上，只开 unix socket（${运行时目录}/server.sock，目录 700、socket 600），不开 TCP。
用户级 systemd 服务 singlefs-task-server.service 起它（python3 task-server/control install 装）；人与 agent 不直接起它，也不直接读写库。
路径、错误类型、重试、资源记录、编译目录、分片、环境变量名单与身份写在 task_server_lib.py 文件头。
代码按职责分在同目录的几份模块里，这一份 import 它们、再导出同名（task_server_lib.py 文件头「这一个目录里的文件」列了各份）：task_server_constants.py（常量与小函数）、
task_server_store.py（Store）、task_server_machines.py（Machine、LocalMachine、PeerMachine、Resources 与读进程的函数）、task_server_scheduler_lifecycle.py、
task_server_scheduler_notifying.py、task_server_scheduler_building.py、task_server_scheduler_admission.py、task_server_scheduler_starting.py（Scheduler 的方法，
按职责分在五份里）；Scheduler 类（__init__）、build_application、serve、main 留在这一份。

  task_server.py serve      常驻；收到 TERM / INT 停下（在跑的任务在各自的单元里接着跑，不杀；服务自己起的探测子进程停掉再退，下面 ③「停服务」）、退 0。
                            同一个 socket 上已有一份在回话退 3；必须依赖、多机配置、签名密钥、每台默认重活线程数不在退 78；socket 路径太长或绑不上退 78
                            （点名那个路径；太长的另报它有多少字节、上限多少，task_server_lib.py 文件头「路径」）；只在自证里认的变量设在了别处退 2；
                            NFS 共享判不过（task_server_lib.py 文件头「NFS 共享」：多机配置没写 NFS_SHARED_ROOTS、哪个根的目录、导出、挂载或两向探针不过）退 78，
                            一句话写明哪个根、哪一项，出路指 task-server/README.md「## NFS 共享」；这一判在 Scheduler 建起之前。
                            NFS 判过之后取共享数据根与工作根（task_server_lib.shared_data_root、work_root：照 .claude/hooks/lib_work_root.py，环境里的 SINGLEFS_DATA_ROOT，
                            没设取多机配置的 SHARED_DATA_ROOT，要落在 NFS_SHARED_ROOTS 某一段下）：取不到退 78，出路点名往 multi-host.env 加 SHARED_DATA_ROOT 那一行；不补缺省。

接口（HTTP over unix socket，JSON；调用方的进程号与 uid 由 SO_PEERCRED 拿，uid 不是服务自己的拒 403；会话号照 task_server_lib.session_of_peer 从进程树上读）：
  GET  /health                    {ok, problem, database_writable, scheduler_seconds_since_tick, tag, queued, running}；problem 只报四样：库写不进；派活判排头能起却连着三轮没起成
                                  （用户：「查清楚为什么总是误报」「根改」：/health 不另判放不放得下，排队线程每一轮排完记下派活给排头的结论（HeadVerdict：判能起而起了、
                                  判能起却没起成及原因、判不能起及原因），只有判能起却没起成的才算，判不能起的（等负载、排在排满时限的任务后面、第二台一轮起满、起之前复核没过……）一律不报，
                                  不认排队原因里的字样；starved_problem）；看进展：某台照公式（room_by_the_formula）放得下排着的最小那条、却连着 PROGRESS_STALL_SECONDS（10 分钟）
                                  一条都没起（不读派活的结论，兜派活自己判错；等负载、等占盘、照排队先后那两条规则等着的不算；progress_problem）；
                                  本机读不到在跑尝试的单元状态（下面 ①「读不到单元状态」：对账记下的那一段，unit_unreadable_report；另带 unit_unreadable 栏逐次列出
                                  {attempt, task, number, host, unit, seconds, presence, detail}；弄坏开关 unit-unreadable-health-unreported 下不报）。
                                  两样都由排队线程每一轮排完算好、存下（starved_text、progress_text），/health 只读存的那一句，不现读余量、不起子进程；queued、running 只数行
                                  （弄坏开关 health-judges-head 下照改前现判排头，health-head-judged-apart 下照改前另判一套，progress-unwatched 下不看进展）；
                                  另带 nfs（每个共享根的四项结论：起来时判的，前三项每 NFS_RECHECK_SECONDS 由复核线程更新）与 peer（第二台用不用得了、原因、从哪一刻起、
                                  最近一次那一趟没跑成；note_peer 记的）：两栏都只读存的，不让 ok 变 false（第二台用不了满一段时间由服务自己发消息给会话，下面 ⑥「服务健康」）；
                                  再带 booking_drift（记账对账判出的漂移：memory 是 {台: {since, rounds, booked, used, attempts: [{label, task, booked, used, cap, why}…], text}}；
                                  没有漂移是空表；只读存的，不让 ok 变 false；下面 ②「内存」的记账对账）；
                                  再带 booking_ledger（在跑记账的逐条明细，记账对账每一轮换上，refresh_booking_ledger：{台: {at, read, slice_total, margin, caps, used, booked, room, slice_room,
                                  rows: [{task, number, label, cap, used, booked, basis, at_cap, text}…], title, total}}；basis 是 stopped、no-scope、measured，说为什么记这么多；
                                  只读存的，不让 ok 变 false；control bookings 逐台打出来，不连第二台）；
                                  再带 paused（暂停着没有）与 pause（{reason, since, allow_session, other_running, text}，text 是「暂停中（原因：…，起于…）、在跑 N 个」，放行了一个会话的接
                                  「、放行会话 <号>」与「、别的会话在跑 M 个」，没暂停是 null）：暂停不让 ok 变 false，暂停中不判「判能起却没起成」，也不看进展（下面 ⑤「暂停」）
  POST /tasks                     {command: [词…], cwd: 绝对路径, environment: {名: 值}, inbox: {socket, token}, whole_package_confirmed: 确认文件}
                                  → {id, session, state, sharded, compiles, inbox, warnings, confirmation, confirm_timeout_seconds}；收下的任务记待确认（state=awaiting-confirmation）、不进队，
                                  confirmation 是确认单（下面 ⑦）；inbox 是会话的收件地址（task submit 读会话环境里的 CLAUDE_CODE_MESSAGING_SOCKET、CLAUDE_CODE_MESSAGING_TOKEN，
                                  读不到的客户端当场拒；下面 ⑥），两样都在才记在任务上，回话的 inbox 只说记没记；令牌不进日志、status、/tasks/<号> 的回话（summary 只留 socket）；环境变量照名单再核、命令里套了包装或派发器的拒、
                                  命令外层与 bash / sh -c 里写了退役变量「名=值」的拒（task_server_lib.py 文件头「环境变量」）、
                                  调用方给的编译目录指到仓里的拒、派不到第二台的三种、临时 prune-server 包装里会编的与不点名包的全仓编译拒（400；这五种的 detail 是
                                  {placement: {problem, next_step}}，下面 ②「提交时拒」；整包测试与不指定范围的跑法没带确认标识的同样 400 {placement}，②「提交时拒」⑥）。
                                  这些拒全在 Scheduler.submit_refusal 一处判，服务起来时重核在排的（revalidate_queued）照同一份判
  POST /tasks/{号}/confirm        {items: 项数, tasks: 这个会话在排在跑的条数} → {confirmed, state, detail}：只许提交它的会话；任务要在待确认；两个数都与服务此刻算的相同才进队，
                                  对不上回 409、detail 是 {mismatch, items, tasks, lines, problem, next_step}（服务算的两个数与确认单）；不在待确认的回 409（下面 ⑦）
  POST /tasks/{号}/long-run-confirm  {why: 理由} → {state, detail}：只许提交它的会话；任务要在跑、累计满两小时、没带 --full-gate；--why 去掉空白不到 12 个字的回 409（下面 ⑦「跑满两小时」）
  GET  /tasks/{号}                任务与它每一次尝试的记录；待确认的带 confirmation（此刻现算的确认单）；排着的带
                                  waiting_for（上一轮判的在等什么）与 queue_position（上一轮全局排第几：position、在排共 queued 条、前面排满时限的 aged_ahead 条）；分片的父任务带 children（各子任务与它们的尝试）；
  GET  /tasks/{号}/wait?timeout=秒  至多等这么久（不超过 60）：结局有了回 {finished: true, …}，没有回 {finished: false}；结局交给提交它的会话时记成已取；
                                  在跑的那一次正疑似卡住、这一段还没报给提交它的会话的回 {finished: false, suspect: {…}}（下面 ①「疑似卡住」）；
                                  这条任务（分片的连同子任务）有还没报给提交它的会话的要处置的提醒（或又跨过一道提醒门槛）的回 {finished: false, attention: [{kind, threshold, task, basis, why, next}…]}，
                                  结局有了的同样在 {finished: true, …} 里带 attention（下面 ①「要处置的提醒」）
  POST /tasks/{号}/cancel         只许提交它的会话：在队里的当场记 cancelled；在跑的 systemctl --user stop 它的单元、回读 ActiveState 与 cgroup.procs 停了，
                                  再等对账写回结局（至多 cancel_settle_seconds）：结局是 cancelled 才回 {stopped: true}；取消到的时候命令已自己结束、写回的是别的结局
                                  （succeeded 这类）回 409 {stopped: false, outcome}，等不到回 500，都不报停了（cancel_outcome_reply；弄坏开关 cancel-outcome-unchecked）。
                                  拿这条任务的锁判（下面 ③「起」）：判放得下之后、记尝试之前到的不起，记了尝试之后到的等单元起完再停、回读（TS-01）。
                                  分片的一片（子任务号）单独取消：只撤那一片，别的片照跑、合并照起并点名缺它（task_server_lib.py 文件头「分片」的「只取消一片」）。
                                  名下有校验不过的尝试或子任务记录（关联列改过，TS-05）、或任务已因校验不过记了 record-tampered 的，一个单元都不停、回 409，不报停了。
                                  分片的父任务连同它在排、在跑的子任务一起取消：父任务先记 cancel_requested、阶段改 cancelling（不先记结局），在排的子任务当场收尾、在跑的逐个停、
                                  回读，再等每一条停了的在库里记成 finished（同不分片的那一种等法，几条共用一个 cancel_settle_seconds；cancel_children），
                                  都到了才给整件记 cancelled、删产物、回 stopped：整件的结局时刻不早于子任务；有没停的、等不到结局的回 500（取消已记下，accepted，点名哪几条子任务），
                                  再取消照样重查、重停活动的子任务；父任务已有结局而子任务还在跑的同样停它们（TS-04）。弄坏开关 parent-finalized-before-children 下停了单元不等结局。
                                  取消的不留产物（用户：「取消的任务要自己删掉产物 不要残留」；remove_cancelled_artifacts）：回读到单元停了、回 stopped 之前，在同一次取消里删这条任务
                                  （分片的连同子任务）每一次尝试的现场目录（「这个加入流程中 让task server去做」；设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的也删）；
                                  本机组里的编译目录不删：它归这棵树的组，尝试停了就放回（task_server_lib.py 文件头「编译目录」）。派到第二台的，照那一次尝试日志里 multi-host-run.sh
                                  的 ⑥ 打的「⑥ 删了第二台这一趟的本地目录 …（回读不在了）」认第二台本地那一份与 runs/ 下三个文件删了（取消时在跑的那一次没有这一行算没删成；
                                  清理回执 peer-cleanup.json 还是 not-started、日志里也没有建了这一趟的本地目录之后才打的两行的，是还没建，不算没删成、不列，
                                  同 confirm_peer_cleanup 认 not-started；弄坏开关 cancel-not-started-receipt-unread 下照算没删成）；那一趟接着用组里那一个的，⑥ 没等到退出码连它一起删，
                                  收尾时组里删掉那一行。删前的大小（第二台本地盘上的量不到，照写）与删后回读不在了才算删了，
                                  逐份写进每条任务最后一次尝试的日志，回话带 artifacts {removed, left, lines}，没删成的进 left、写告警 build-directory-left，task cancel 照它退 5。
                                  弄坏开关 cancelled-artifacts-kept 下不删、不报。Store.update 守库里已 ended 的尝试不写回别的状态（弄坏开关 attempt-row-overwritten 下不守）
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
  ① 对账：每一次在跑的尝试读它的结果（attempts/<标签>/result.json，attempt_runner.py 写）。有结果：照 classify_attempt 定类型、照 retry_decision 定重排还是收尾（撞了 250 的那一次下一次的上限乘 1.5，     到或超过那台总上限的照「重试」打回；脚本永远不报自己要多少内存（用户 2026-11-13 定「脚本永远不要报自己要多少」）：上限只照命令键的记录定、撞了乘 1.5，命令打的任何「要多少」行一律不认）。
     没有结果：每 UNIT_CHECK_SECONDS 秒读一次单元的 LoadState 与 ActiveState（照输出判）：在跑的照等；
     状态还是「起单元之前」、起跑标记不在、单元不在（LoadState=not-found）的，是服务在写了单元名与起单元之间没了的那一次：起它一次（同名单元在跑时 systemd-run 自己拒，不会起第二份）；
     别的单元不在了又没留结果的，过 LOST_GRACE_SECONDS 秒记 launcher-lost。
     读不到单元状态（unit_states 交 None：systemctl --user show 起不来、超时或没打 ActiveState）时判不出单元在不在：留了 result.json 的照上面照它的退出码收尾；
     没留的不杀、不收尾、不记结局，尝试照记 running，不设到点就收尾的时限，改用不经 systemd 的办法核这一次还有没有进程（unit_unreadable_gone、attempt_presence）：
     本机读单元那一层（attempt_runner 与它起的包装都在本机的单元里，派到第二台的也是）——起跑标记 started 记的 runner 进程号在 /proc 里在不在，
     本机用户级管理器的 cgroup 树（/sys/fs/cgroup/user.slice/user-<uid>.slice/user@<uid>.service）里名字是这个单元的目录与 attempt_runner 写的 scope 文件点名的目录、
     名字以 MEMORY_CAP_SCOPE_PREFIX 起头而里面有进程的环境带这一次尝试标签的 scope，各自的 cgroup.procs 读出来空不空；派到第二台、清理回执 pending 的，
     第二台照同一段（task_server_lib.attempt_presence_readings；第二台经多机配置的读法 Peer.run 送过去起，同进展读数）读 multi-host-run.sh 记的进程号文件与 scope 单元名文件
     （runs/<标签>.pid、runs/<标签>.scope）；回执 not-started、confirmed 的第二台没有这一次的进程。核法只读 /proc 与 cgroupfs，不起 systemctl、不发信号；
     cgroup 目录不在而上一层列得出算空（已收掉），读不了、起跑标记不在、清理回执读不到、第二台连不上都算核不出。
     核在后台线程里做（probe_unit_presence，每 unit_check_seconds 至多一回；第二台那一趟带 ssh，排队线程不等它），对账照最近一回定（lib.attempt_presence_verdict）：
     有一个进程号在、或有一个 cgroup 里有进程，是还在（照记 running）；都读得到、都空、记下的进程号都不在，连着满 LOST_GRACE_SECONDS 才照没留结果收尾（再读一次 result.json，
     没有的记 launcher-lost；收尾时留下的进程不再经 systemctl 回读，照这一回核的认，stop_leftovers）；有一样核不出、又没有在的，一直等。
     读不到的那一段提醒发起任务的会话（要处置的提醒 unit-unreadable，下面「要处置的提醒」），/health 的 problem 与 unit_unreadable 栏报本机读不到哪几次尝试的单元状态
     （control check 照 /health 不健康退 78）；头一回读不到、之后每跨过一道提醒门槛各往服务日志打一行
     「TASK_UNIT_UNREADABLE time=… attempt=<尝试记录号> task=<任务号> number=<第几次> host=<尝试派在哪台> unit=<单元> seconds=<连着读不到几秒> presence=<alive|gone|unverified|unprobed> detail=…」，
     核到都不在、照没留结果收尾的那一刻再打一行（presence=gone、detail 写核到的）。读到了单元状态就清掉这一段的记账。
     弄坏开关 unit-unreadable-exit-file-unread 下读不到的那一段不认 result.json（核到没了也照没留结果收），unit-unreadable-absent-cgroup-unconfirmed 下 cgroup 目录不在不算空、算核不出，
     unit-unreadable-lost-unverified 下读不到就当场照没留结果收尾（不核进程，收尾照常经 systemctl 回读、停留下的进程），unit-unreadable-unreminded 下不报要处置的提醒，
     unit-unreadable-health-unreported 下 /health 不报。
     续起「起单元之前没了」的那一次拿这条任务的锁、现读任务：
     取消了的照取消收尾、不起（resume_starting）。起单元回执超时过的（start_uncertain_at，下面 ③「起」）单元不在、起跑标记也没有的，过 START_UNCERTAIN_GRACE_SECONDS
     还是这样才确认没起、记 unit-start-failed；单元在跑的照常 running。
     每一轮另把还没结局的分片父任务各现推一遍（drive_sharded_parents，下面 ④；弄坏开关 sharded-parents-unreconciled 下不推）。
     第二台的结局（task_server_lib.py 文件头「错误类型」）：日志里有 multi-host-run.sh 打的「MULTI_HOST_RUN_STEP step=peer-command event=end … exit=<码>」的，
     照那个码（第二台那条命令自己的退出码）定结局，命令自己退 255 记 command-failed、不重排；没有这一行才是 multi-host-run.sh 那一层没跑成（peer-run-failed）。
     弄坏开关 peer-step-exit-ignored 下不看这一行。第二台那一层的内存峰值与编译目录占盘峰值照日志里的 MULTI_HOST_RUN_PEER_MEMORY_PEAK、MULTI_HOST_RUN_PEER_BUILD_PEAK
     记进那一次尝试（不打的记「没量到」），同本机一样进定额度（弄坏开关 peer-memory-peak-ignored 下不读内存峰值那一行）。
     留下的进程（stop_leftovers）：尝试正常结束（有结果、没被停）之后，单元里的由 attempt_runner 退出之前自己停掉、回读，记进 result.json 的 unit_leftovers
     （它一退 systemd 就收单元、当场杀掉剩下的，服务那时已回读不到）；外层 scope（结果里记的 scope，或按尝试标签在 slice 里找到的）由服务回读 cgroup.procs，
     还有进程就对这一次自己的 scope 名发一次 systemctl --user kill --signal=SIGKILL，至多等 LEFTOVER_STOP_SECONDS 秒回读 cgroup 空了没有；
     结局（与重排）的说明前面写留下了几个、哪几个进程号、回读空了没有。弄坏开关 leftover-processes-kept 下两处都不停（runner 照 spec 的 leftovers_kept）。
     收尾回读还有进程或读不了时，尝试保持 starting/running 并写 cleanup_pending，资源占用继续保留，后续对账重查。
     单元已停先放记账（release_stopped_booking）：取消那一侧回读到单元停了（is-active 的输出不在跑、cgroup.procs 读到而且空）、本机尝试收尾一开头同样读到停了的那一刻起，
     这一次尝试不再进 CPU 与内存的在跑记账（unit_stopped、booking_attempts：running_threads、room_by_the_formula、beside_whole_capped、server_pending）；删目录、取峰值、
     落结局、第二台的清理回执照旧做完，记 ended 时清掉这一条；清理没确认（cleanup_pending）的照旧数。读不到状态或 cgroup.procs 的不算停（决不当它停了）。
     弄坏开关 unit-stopped-still-booked 下不放。
     被外部信号杀掉（external_kill_note、kill_log_lines）：命令被不是服务发的 TERM / KILL 终止（构建工具报「signal: 15, SIGTERM」而自己退 1、或退 128 + 信号号），照退出码记的结局是 command-failed、
     服务这一次没有发过停的信号（service_stop_step 空）、单元自己也没收到停的信号（result 的 stopped_signal 为 0）时，读这一台尝试开跑前几秒到结束后 EXTERNAL_KILL_LOG_AFTER_SECONDS 秒的
     earlyoom 日志（journalctl -t earlyoom：「sending SIGTERM to process <号> …」）与内核日志（journalctl -k：「Killed process <号> (…」），点名的进程号在这一次尝试进程树里见过的
     进程号（attempt_runner.py 每轮采样记进 result.json 的 process_ids；Machine.process_ids_seen）里的，记 killed-externally（尝试记 external_kill_note：哪份日志的哪一行、哪个进程号）、
     同一上限重排至多 EXTERNAL_KILL_RETRIES 次（task_server_lib.py 的 retry_decision；改主工作区的命令不重排）。日志读不到、没有对上的进程号仍记 command-failed，不猜。
     第二台的尝试服务这一侧拿不到进程号，由 scripts/hosts/multi-host-run.sh 在第二台那一侧记下这一趟进程树的进程号，命令退非 0 之后读第二台的 earlyoom 与内核日志、
     点名这些进程号之一被杀的打一行「MULTI_HOST_RUN_PEER_EXTERNAL_KILL source=<earlyoom|kernel> pid=<号> line=<日志原行>」（PEER_EXTERNAL_KILL_LINE，log_content 记进 peer_external_kill，
     不算日志在涨），服务读到这一行照本机那一路记 killed-externally、同一上限重排（peer_external_kill_note）；没有这一行的仍记 command-failed。
     内核那一行是「Memory cgroup out of memory」（task_server_lib.py 的 MEMORY_CGROUP_KILL_TEXT：某个内存组到了它自己的上限，不是整机内存紧张）的两台都不认：
     本机的 external_kill_pid 不交进程号，第二台那一行 multi-host-run.sh 照打、服务按 line= 的内容分；照原来的路判（命令自己退非 0，或这一条自己的 scope 撞了上限的照撞上限）。
     弄坏开关 memory-cgroup-kill-counted 下这一种也认。
     进程树里的进程每秒采样一次，活得比一秒短的进程号可能没记到。弄坏开关 external-kill-unrecognized 下不读日志、不认这一行，一律照退出码记，peer-external-kill-line-ignored 下只不认第二台那一行。
     新的远端尝试另有 peer-cleanup.json：起远端命令之前先落 pending，确认进程停下并核目录没有持有者后才写 confirmed；
     失联保留远端目录和 PID/启动时间/退出码，包装已退出时对账经 multi-host-run.sh --recover-cleanup 重查。升级前无此回执的尝试按旧协议处理。
     同一次尝试同一时刻只起一趟 --recover-cleanup（对账、停单元的后台线程与取消那一侧都会来确认；confirm_peer_cleanup 按尝试号拿锁），后到的等它放下再读回执判
     （弄坏开关 peer-cleanup-recovered-concurrently 下各起各的）。
     收尾不在排队线程里做（finish_off_loop）：对账判出该收尾的那一次（有结果、对不回来、起单元之前取消了的），交一个后台线程做
     finish_attempt：第二台的清理回执（--recover-cleanup 经 ssh）、留下的进程回读、第二台那一个编译目录留没留着记进组里（settle_build_slot）都在那个线程里，
     排队线程只看结果，照常对账别的尝试、派活；同一次尝试同一时刻只一个线程在做（finishing），对账与补收尾（settle_ended_attempts）见到它在做就跳过，做完放下、叫醒排队。
     状态机不变：清理没确认的照旧保持 starting/running、记 cleanup_pending。弄坏开关 finish-in-scheduler 下照在排队线程里当场做。
     跑得久而失败（finished_preface）：结局不是 succeeded、这一次跑超过 task_server_lib.LONG_RUN_SECONDS、命令又没声明续跑的，说明前面写「这条跑了 N 分钟、没有续跑：
     重交会从头跑」（task_server_lib.py 文件头「续跑」）。
     命令退 75（prune-server 不可用）：记 prune-server-unavailable，另起一个线程在仓根跑一次 python3 prune-server/control check（PRUNE_SERVER_CHECK_SECONDS），
     不通写告警 prune-server-unavailable；只报，不起、不停 prune-server（check_prune_server）。
     卡住（用户：「就是距离上一次输出日志10分钟没有日志输出」，又选「日志与 CPU 都不涨才停」，又说「至少要延迟十分钟后在判定吧」）：尝试起来后先宽限
     STALL_GRACE_SECONDS（600）秒不判；宽限过了，日志距上一次增长满 STALL_SECONDS（600）秒、
     这一个窗口里 scope 的 CPU 平均不到 STALL_CPU_CORES 个核，服务停掉它、记 stalled（CPU 在涨的是在算，照跑，再看下一个窗口）、
     照常经 wait 叫醒提交它的会话；CPU 本机读 attempt_runner 写的 cpu_usec（各 scope 合计），第二台读日志里 multi-host-run.sh 每轮打的 MULTI_HOST_RUN_PEER_CPU 一行，
     这一行、MULTI_HOST_RUN_PEER_MEMORY 行（下面「顶在上限上卡住」）、MULTI_HOST_RUN_PEER_IO_WAIT 行（下面「卡在 IO 上」）、MULTI_HOST_RUN_PEER_MEMORY_STAT 行（下面「内存最高那一刻」）、MULTI_HOST_RUN_PEER_GPU 行（下面「第二台的显存」）、MULTI_HOST_RUN_PEER_THREADS 行（② 「CPU 闸」的可跑线程数）与 MULTI_HOST_RUN_STEP 行不算日志在涨。已跑多久、日志多久没涨、到此刻平均在用核数
     只写进 task status（running_progress），核数过少不停；头一回量一次尝试（服务刚起来对账在跑的、或宽限刚过）时「日志上一次涨」取日志文件的修改时刻
     （读不到取此刻；服务重启不把日志多久没涨清零），CPU 的窗口照旧从此刻起量。服务不照用时停任务（没有「远超上次用时就停」这一条；用户：「没有预计用时这个东西」）。
     有进展的不停（judge_stall_stop、note_window_head、note_activity、activity_text）：开新窗口的那一轮（note_window_head）与日志没涨的每一轮另读进程内存与峰值（Machine.memory_now）；
     窗口满时进程内存或峰值这一窗口涨了至少 ACTIVITY_MEMORY_BYTES 的不停，从此刻再看一个窗口（贴着上限的不算，照下面「顶在上限上卡住」）。
     弄坏开关 progress-ignored-for-stall 下不看，stall-window-head-unread 下开窗那一轮不读头一份（一轮排队比窗口还长时窗口里只有一份读数，在涨的也当没进展）。
     命令在等 prune-server 回话（判定库客户端一问一答）而没有别的输出时，靠判定库进度行（lib.VERDICT_LIBRARY_PROGRESS_LOG_MARK「PRUNE_SERVER_CLIENT_PROGRESS <说明>」）算日志在涨、不被判卡住；
     服务读不到客户端此刻是不是在等回话（一问一答没有可读的状态，判定库服务不暴露每条连接在不在等），只认这一行；弄坏开关 verdict-library-progress-uncounted 下这一行不算在涨。
     命令自己按固定间隔打的心跳行（lib.HEARTBEAT_LOG_MARK「TASK_SERVER_HEARTBEAT <说明>」；task_server_lib.py 文件头「命令自己打的两种登记行」）不算日志在涨：
     编译、等锁一阵子不出输出的命令靠它在任务日志里留活着的痕迹，真卡死时只剩心跳行，满窗口、CPU 也不涨照样停；夹着真输出的不停。弄坏开关 heartbeat-counted-as-log 下心跳行照算在涨。
     服务重启不杀在跑的任务（它们在各自的单元里），重启之后照这一步对回来。
     顶在上限上卡住（judge_stall_stop；任务 1866 第 4 次：上限 13824M、峰值 13.50 GiB、日志 1288 秒没涨）：照上面两条判卡住的那一刻，这一次尝试的内存
     此刻或峰值已到这一次上限的 STALL_AT_CAP_FRACTION（98%）的，照样停掉，但不记 stalled：记 memory-cap-hit、照重试规矩乘 1.5 重排（钉值的打回 memory-cap-at-maximum），
     结局与重排的说明前面写「顶在上限上卡住：…」；两样都读不到（还没采到 scope、第二台那一层这一轮没打那一行）的照记 stalled。
     判卡住停的是判它的那一次尝试：起的后台线程把那一次的尝试号交给 stop_running、只停那一个单元（线程起来到读库之间那一次自己退了、
     重排起了下一次的，下一次照跑）；取消那一侧照旧按任务停全部。弄坏开关 stall-stop-by-task 下后台线程按任务号停在跑的全部。
     内存只算进程用的（memory.stat 的 anon 加 shmem，memory_peaks_lib.py 的 process_memory_bytes；同 task-server/README.md「算这台还放得下多少」）：页缓存撑满 scope 上限时
     内核回收它、不因此杀进程（任务 8853：anon 921M、file 22163M、memory.peak 顶到上限 50176M），只有进程内存会撞 OOM，撞上限照旧看 memory.events 的 oom_kill 与包装的退出码。
     内存两台同一段判，按台不同的只在读法（Machine.memory_now：本机此刻读外层 scope（尝试目录的 scope 文件记着）的 anon 加 shmem，峰值读 attempt_runner 每轮写进尝试目录的
     memory_peak（它见过的最大）；第二台读日志里 multi-host-run.sh 每轮打的最近一行
     「MULTI_HOST_RUN_PEER_MEMORY poll=<轮数> current_bytes=<字节> peak_bytes=<字节> read=<yes|no>」（PEER_MEMORY_LINE；那一层内存包装 scope 这一轮的 anon 加 shmem 与
     见过的最大，read=no 是这一轮没读到：此刻交 None、峰值照留上一次读到的））。弄坏开关 memory-peak-with-page-cache 下本机照改前读 memory.current 与 memory.peak
     （attempt_runner 记的峰值同样照 memory.peak，attempt_runner.py 文件头 ⑥）。
     贴着上限而 CPU 在涨的同样算卡住（任务 7723 第 2 次：上限 6144M，scope 的 memory.current 6442393600、memory.max 6442450944，memory.pressure full avg300=2.62，
     日志 37702 秒没涨，CPU 平均 2.526 核在回收里打转，跑了 37775 秒没人停）：日志满 STALL_SECONDS 没涨、此刻内存不低于上限的 STALL_AT_CAP_FRACTION 的，
     CPU 在涨、读不到都不算在算，照这一条停掉、记 memory-cap-hit、乘 1.5 重排；此刻内存离上限远的、读不到此刻内存的照上面看 CPU（在涨的照跑）。
     弄坏开关 at-cap-busy-cpu-kept-running 下贴着上限而 CPU 在涨的照跑（改前的样子），at-cap-fraction-ignored 下读得到此刻内存就当贴着上限、不比上限，
     peer-memory-line-unread 下第二台不读 MULTI_HOST_RUN_PEER_MEMORY 那一行（此刻与峰值都交 None；那一行照旧不算日志在涨）。
     卡在 IO 上（任务 8608、8609、8611、8619：本机根盘被一条全目标编译写到盘忙 84%、/proc/pressure/io some avg10=82.44，本机导出的 NFS 跟着慢，第二台上 find、git diff、
     往工作根 mv 的进程都在 D 状态、wchan 是 rpc_wait_bit_killable / nfs_wait_bit_killable，卡了十几分钟，日志不涨、CPU 20 分钟只涨 1.9 秒，被当成卡住停掉；它们是在等 IO）：
     日志没涨的每一轮另读一次这一次尝试此刻在 D 状态（不可中断的 IO 等待）的进程数（note_io_wait；两台同一段，按台不同的只在读法 Machine.io_blocked：本机数尝试目录 started
     记的 runner 进程号连同它的子孙与外层 scope 的 cgroup.procs 里各进程 /proc/<号>/stat 的状态字段；第二台读日志里 multi-host-run.sh 每轮打的
     「MULTI_HOST_RUN_PEER_IO_WAIT poll=<轮数> blocked=<D 状态的进程数> read=<yes|no>」（PEER_IO_WAIT_LINE，这一行不算日志在涨；上一回之后读到几行取最少的，有一行 read=no 算这一回读不到））。
     日志与 CPU 都满窗口不涨、而这一个窗口里每一轮都读到了、都有进程在 D 状态的，不判 stalled、不停，从此刻再看一个窗口；有一轮没有 D 状态的进程或读不到的照原判法判。
     贴着上限那一条照停（贴着上限时回收本身也让进程进 D 状态，不当在等 IO）。每一轮都有 D 状态的进程满 IO_WAIT_ATTENTION_MINUTES 分钟的另报要处置的提醒 io-wait（下面）。
     弄坏开关 io-wait-ignored 下不看 D 状态（改前的样子）。
     内存最高那一刻（撞上限那一次尝试的结局说明与推送写它；任务 10279 峰值 45.46 GiB，包装退出时读的 memory.stat 只剩 anon 311296，进程已被杀、内存已释放）：
     本机由 attempt_runner.py 每轮读外层 scope 的 memory.current，比见过的最高还高就存一份 memory.stat 的 anon、file、file_dirty、file_writeback、shmem，随 result.json 的 memory_stat_at_peak 交回；
     第二台读日志里 multi-host-run.sh 每轮打的「MULTI_HOST_RUN_PEER_MEMORY_STAT poll=<轮数> current_bytes=<字节> anon=… file=… file_dirty=… file_writeback=… shmem=… read=<yes|no>」
     （PEER_MEMORY_STAT_LINE，不算日志在涨），留 current_bytes 最大的那一行（log_content 记进 peer_memory_stat）。finish_attempt 照它写 peak_memory_note（lib.peak_memory_note）。
     弄坏开关 peak-memory-stat-read-at-exit 下两台都留最后读到的那一份（进程被杀之后读到的）。
     跑满两小时提醒（提交时的门禁之外一条任务跑过两小时要提交它的会话确认一次；long_run_unconfirmed_items，判法在下面「要处置的提醒」的 long-run-unconfirmed）：这条任务各次尝试在跑的时长累加
     （结束了的取记下的用时，在跑的取此刻已跑的；排队等的不算；撞了内存上限、照大一档重来的计时归零，只从最后一次撞上限之后那一次起算；run_limit_accumulated），
     满 RUN_LIMIT_SECONDS（7200 秒）服务不停它、不改任务，记要处置的提醒 long-run-unconfirmed；分片的子任务按自己的尝试各自累计（long_run_confirmed_of）。
     不提醒的只有交的时候带了全量门禁标识的（task submit --full-gate：只认提交时的六道门禁、scripts/crash/crash-amplification.sh 或 scripts/gate/gate-staged.sh、不带 --check 与 --item、
     也不是只查不跑的写法（--selftest、--list、--list-items、--help 这类，lib.query_only_entry），lib.commit_gate_entry，
     别的命令带它提交时拒；分片的子任务照父任务）与确认过的（下面 ⑦「跑满两小时」）；任务 9604 那种带 --check 的门禁、merge-and-verify、入口脚本、实验都提醒。
     弄坏开关 long-run-unconfirmed-silent 下不提醒，long-run-cap-hits-counted 下撞上限的那几次也累加，
     full-gate-flag-ignored 下带了也照提醒，long-run-confirmed-still-reminded 下确认过的照提醒。
     四小时硬上限（下面 ⑦「四小时硬上限」）：累计的算法同这一条（run_limit_accumulated），满 RUN_HARD_LIMIT_SECONDS（14400 秒）服务停这一次尝试、记结局 over-four-hours 打回；确认过的同样停，只有带 --full-gate 的不判。
     疑似卡住（用户：「不自己终止，告诉负责的对话 自己检查」；check_suspect）：同一宽限过后，整个单元的 CPU 满 STALL_SECONDS 秒平均不到
     STALL_CPU_CORES 个核，不管日志涨不涨：不停它，尝试记 suspect（依据）、写告警 suspected-stall、叫醒在等的 wait（它打 TASK_SERVER_SUSPECT 一行、退 task_server_lib.SUSPECT_EXIT，任务照跑，
     再起 wait 接着等）、task status 写「疑似卡住」与依据。一段只报给提交它的会话一次（服务内存里记报过的段号）；之后 CPU 又涨了（相邻两次读之间平均不少于 STALL_CPU_CORES 个核）
     这一段结束、从此刻再量下一段；读不到 CPU 窗口作废重量。这一次照上面两条停了的不再报疑似。
     要处置的提醒（用户在弹窗里选「task wait 自己报」；三方判决 research/prompts/watchdog-task-half-r1-main-verification.md「三、怎么办」岔路表甲；attention_items）：
     删掉看门狗盯任务那一半之后没人报的五种情形，加改主工作区的命令没正常结束、日志长期不涨而服务不停它、卡在 IO 上而服务不停它、卡上有它的计算进程而服务不停它那四种，
     再加预算贴着整台、跑得久而核没用起来、按给的上限开并行那三种与连着撞上限还在重排那一种（用户：「既然加了给对应对话发消息的功能 就就该把异常和修改意见交给对应的对话」：依据写量到的数、why 写为什么、next 写修改意见），服务不停、不改任务，只在 wait 那一侧交给提交它的会话。
     每一种怎么判、动作、修改意见与没处置时照 task_server_lib.py 文件头「异常表」那一行：next 的头一段是那一行的修改意见（attention_item 经 lib.anomaly_advice 取，
     代码里不另写出路句），后面接这一回的具体数；时长判的那几种在依据末尾写下一回提醒的门槛（「排满 X 秒再提醒一回」这类）；下面每一种只写判法与依据。分片的任务连同它的子任务一起判（提醒点名的 task 是子任务号）：
       queued-too-long   任务在 queued：从提交（重排的从上一次尝试结束）起排满 QUEUE_ATTENTION_MINUTES 分钟，依据写已排多久与排队原因（waiting_reasons），出路照异常表 queued-too-long 那一行（核「在等：…」点名的是哪一台、哪一样）；
       quiet-log-busy    在跑、这一次尝试的日志（不算起法那一层的进度行，log_content）距上一次增长满 QUIET_ATTENTION_MINUTES 分钟，而服务最近一个窗口没停它是因为
                         CPU 在涨或读不到 CPU（上面「卡住」：日志与 CPU 都不涨才停；log_growth 记的免停缘由是 SPARED_CPU_BUSY、SPARED_CPU_UNREAD）
                         （任务 7723 那样在回收里打转的能跑上十个钟头）；按 D 状态（io-wait）免停的、
                         还没判过一个窗口的不报这一条；依据写日志多久没涨，出路照异常表 quiet-log-busy 那一行（核停在哪一步、是不是把输出攒着不打）；弄坏开关 attention-quiet-busy-unreported 下不报，
                         quiet-log-busy-any-exemption 下照改前不看免停缘由；
                         日志不涨而有进展的不报（任务 17334 顶层日志 66200 秒没涨，测试进程的标准输出另指到运行目录里的 record.log 在涨）：日志满 stall_seconds 没涨之后，
                         每隔 stall_seconds 的一半读一回这一趟进程树里各进程打开写着的普通文件与大小（顶层日志不算）（note_attempt_progress；
                         两台同一段 task_server_lib.attempt_progress_readings，第二台经多机配置的读法 Peer.run 送过去起，进程树从清理回执点名的 runs/<标签>.pid 往下），
                         写着的文件比上一回大了算有进展；多久没涨从日志上一次涨与最近一回读到进展里晚的那一刻起算（弄坏开关 quiet-log-busy-progress-ignored 下不看进展，
                         open-file-growth-ignored 下不看文件，open-file-write-mode-unchecked 下只读打开的文件也算写着的）；
       io-wait           在跑、这一次尝试日志没涨的每一轮都读到有进程在 D 状态（上面「卡在 IO 上」，io_waits 记的从哪一刻起），连着满 IO_WAIT_ATTENTION_MINUTES 分钟
                         （中间有一轮没有、读不到或日志涨了就从头算）；依据写最近一轮 D 状态的进程数与本机此刻 /proc/pressure/io 的 some avg60，出路照异常表 io-wait 那一行
                         （核那台的盘与 NFS：/proc/pressure/io、谁在大量写盘、第二台的挂载），IO 缓过来它自己接着跑；弄坏开关 attention-io-wait-unreported 下不报；
       cleanup-pending   在跑的尝试 cleanup_pending（命令已退出、收尾回读没收干净，资源占用保留；finish_attempt）；
       unit-unreadable   在跑的尝试对账连着读不到它的单元状态（上面「读不到单元状态」，unit_unreadable 记的从哪一刻起）满 UNIT_UNREADABLE_ATTENTION_SECONDS（60 秒），之后每翻一倍再报；
                         依据写尝试派在哪台、本机读不到哪个单元的状态、不经 systemd 最近一回核到的（还有进程在、进程都不在、核不出、还没核完，连同点名的进程号与 cgroup），
                         出路照异常表 unit-unreadable 那一行（核本机用户级 systemd、核依据里点名的进程）；服务不杀、不收尾、照记 running（unit_unreadable_items）。
                         弄坏开关 unit-unreadable-unreminded 下不报；
       shard-advance-failed  drive_sharded_parents 推父任务抛错（note_advance_failure 记在内存里、整份另记进库，推通了删；换服务后照库接着数这一回与开始过几回，
                             弄坏开关 advance-episodes-memory-only 下只记内存）；
       workspace-write-interrupted  改主工作区的命令（modifies_workspace）有了结局而没正常结束：结局是 WORKSPACE_WRITER_INTERRUPTED_OUTCOMES 之一（撞上限、被挤掉、
         第二台没跑成、撞顶、取消、卡住被停、对不回来、满四小时硬上限被停；进队那一刻就打回、一次都没起的 over-four-hours 不算）或命令被信号杀（command-failed 而退出码 128 + 信号号，SIGNAL_EXIT_CODES），可能写了一半
         （用户：「这个问题很严重 因为可能会导致补丁没有正常写入」）；依据写哪一次尝试、什么结局，出路点名要核的文件或补丁目录与 git status。
         apply-writer-patch.py 有结局文件（补丁目录里的 PATCH_WRITER_OUTCOME_NAME）的照它判：服务起一次它的 --read-outcome，打全了（退 0）、已退回（退 1）的不报，
         别的报、依据带它打的那几行；结局文件不在的不报（它在动手换之前写开始与文件行）；带 --dry-run 这类只读参数的不报（workspace_write_item，结了局的只核一次）。
         hook-swap.py 同样起一次它的 --read-outcome（带它自己的 --root、--scratch，hook_swap_read_back）：换完了（退 0）、已退回（退 1）的不报；没换完（退 3）而输出点名的反向规格
         在同一条命令里另一条换钩子的 --scratch 下的不报（结局文件按仓只有一份、记最后写开始行的那一趟，真换起手先读它、上一趟停在半截就不写开始行）；别的报、依据带它打的那几行。
         包在 bash -c 串、命令位置上起的脚本正文里的写工具逐条判，有一条可能写了一半就报（workspace_write_judged）；包着的打补丁的补丁目录照那一条命令里认得出的 cd 与赋值算
         （shell_text_runs），算不出的（循环变量、位置参数、命令替换、cd 到算不出的目录，花括号组、heredoc 正文里有 cd 的整段，串里那一条自己带 env -C 的）不读结局文件、照可能写了一半报。
         只报给有结局的任务，所以这一行打在 TASK_SERVER_REASON 之前、退码仍是任务的（下面「wait 读到还没报的提醒」）。
         弄坏开关 workspace-write-interrupted-unreported 下不报，apply-outcome-ignored 下不读结局文件、一律照可能写了一半报，
         wrapped-writers-first-only 下包着的只认头一条写的，wrapped-writer-paths-literal 下补丁目录照原样、照任务的工作目录拼，
         brace-group-not-layered、heredoc-body-as-commands、string-env-chdir-unread 下花括号组里的 cd、heredoc 正文里的 cd、串里的 env -C 照平铺着跑算，
         swap-outcome-unread 下换钩子不读结局、一律报，swap-outcome-owner-ignored 下换钩子没换完的不看反向规格是谁的、一律报。
       oversized-budget  在排、在跑，排队那一刻判出这一条在某一台上的记账不小于那台的整片界（下面 ②「预算」，budget_check 记进 budget_notices）：每条只报一次；
                         依据写各台的记账、可用上限、整片界与记账从哪来（记录峰值几次、没有记录的缺省、钉值……），出路照异常表 oversized-budget 那一行（核实范围、分片与同时起的个数）；钉了上限的照样报。
                         同一条在排的，排队那一轮还没对它判过「在每一台上都起不来」（idle_short_judged 里没有它：交的时候 confirmation 的 budget_check 就记下的那一项、服务刚起来这一轮还没判到它、排在排满时限的任务后面没判到、有一台判不成）不报；
                         判过之后 booking-beyond-idle-machine 开着的也不报（下一项罩住它：起不来，「照跑」的出路走不通）；那一项没了、或这一条起来了再照报，在跑的照报。这样同一条两种都成立时只推下一项那一行。
                         弄坏开关 oversized-budget-silent 下不记，oversized-reported-before-queue-judged 下没判过也报，oversized-under-idle-short-reported 下两种照报；
       booking-beyond-idle-machine  在排，排队那一轮判出这一条在每一台上都起不来（idle_short_text）：每一台此刻放不下，而且就算服务在那台跑的任务全收尾、放出它们占的内存
                         （各尝试此刻回收不掉的用量，读不到的取它的上限），整机空闲或 slice 放得下的也不够要的，或那台本来就判永远放不下；分片的片、有一台量不到余量、连不上、盘不够的不判。
                         记进 idle_short_notices（note_idle_short，只在内存里，判出不是这种了就删），依据逐台写放出多少之后还差什么，出路照异常表那一行；只推一回、不再推。
                         这一条不算「排在前面还没起的」，排满时限也不挡后面的（place_queued_into）：它等的是整机自己空下来，排在前面的哪条跑完都帮不上它；
                         整机空闲回到够要的它自己起，不记 never-fits（空闲随机器此刻的用法变）。
                         弄坏开关 idle-short-silent 下不记，idle-short-blocks-later 下照一般放不下的那样挡后面，idle-short-unjudged 下不判；
       long-run-idle-cores  在跑、这一次尝试跑满 LONG_RUN_ATTENTION_SECONDS（600 秒）、到此刻平均在用核数不到给的线程数的 LONG_RUN_IDLE_CORE_FRACTION（24/31，线程数 31 时即不到 24 核）：
                         每次尝试只报一次（提醒门槛是尝试的次数），不随时长再报、不再推，依据写跑了多久、给几线程、平均核数与峰值，出路照异常表 long-run-idle-cores 那一行（资源运用是否合理、
                         脚本能不能并行或切片）；只提醒、照跑，读不到 CPU 的不报（long_run_idle_items）。
                         弄坏开关 long-run-idle-silent 下不报，long-run-idle-quarter-threshold 下界是四分之一；
       threads-over-registered  在跑的尝试，服务每一轮读的线程读数（note_threads，② 「CPU 闸」的可跑线程数那一路）里线程总数超过登记数的 THREADS_OVER_REGISTERED_FACTOR（2）倍、
                         其中可跑的（状态 R）也超过登记数、占线程最多的那一个进程自己的线程数也超过登记数的 2 倍（一堆进程加起来多的不报），连着 THREADS_OVER_REGISTERED_ROUNDS（3）轮（同一个采样号读过的不再算一轮，有一轮不超就从头数）：
                         只起一堆不干活线程的编译器、链接器（rustc、ld 的线程多半在睡）可跑的到不了登记数，不报；每次尝试只推一回（提醒门槛是尝试的次数，判出之后不再改），
                         依据写那一轮的线程总数、可跑的个数、登记数与占线程最多的进程名，出路照异常表 threads-over-registered 那一行；只提醒、照跑、不停它。
                         弄坏开关 threads-over-registered-silent 下不报，threads-over-registered-idle-counted 下不看可跑的超没超；
       long-run-split    任务（分片的一片按自己的；分片的父任务没有自己的尝试，不判整件的挂钟）出结局，服务记的用时（各次尝试的 seconds；撞了内存上限、照大一档重来的，从最后一次撞上限之后那一次起算，
                         settled_run_seconds）超过 lib.LONG_RUN_SPLIT_SECONDS（7200 秒）、又不在 .claude/long-run-split-exempt 里（表里的项名等于命令的某个词，long_run_split_exempt）：依据写这是按片或按整条任务的用时判的
                         （.claude/rules/long-run-split.md「判据」：日志里取不出那一项的起止时拿片或整条任务的用时代替），出路指到那份规则「怎么做」；出结局的任务由 push_outcomes 发结局之前那一次与 wait 的结局回话带出；
                         只推一回、不再推。弄坏开关 long-run-split-silent 下不报，long-run-split-cap-hits-counted 下撞上限的那几次也累加，long-run-split-exempt-ignored 下不看豁免表；
       gpu-cases-not-run  任务（分片的一片按自己的；分片的父任务没有自己的尝试；取消的不判）出结局，最后那一次尝试的日志里有用例「本次未跑」的标记行：记号是 GPU_NOT_RUN_MARK
                         （NOT_RUN_NO_GPU_ADAPTER）整词，一行里每一处都认、一处一条（gpu_not_run_test）：前面是行首、空白或别的非标识符字符（标识符字符只算 ASCII 字母、数字、下划线），后面是空白（含制表符）或行尾，
                         ANSI 颜色码先去掉再看；更长或更短的词（`CANNOT_RUN_…` 里含的、`NOT_RUN_NO_GPU_ADAPTER_X`、`NOT_RUN_NO_GPU_ADAPTER:`）不认。边界、颜色码与一行里每一处各是一截的认法
                         从门禁那一份库 scripts/gate/lib_libtest_result_lines.py 的 marker_pieces 导入（那一份认 NOT_RUN_ 一族的名字，这里只认 GPU 这一个，别的名字只用来划截）。
                         点名的用例取这一处记号自己那一截里 test= 的那一段、不取行首那个测试名。
                         测试不带 --nocapture 时，通过的测试打的字被测试框架收走、不进日志，服务读不到这一行、不出这一项：没提醒不等于用例都跑了，要核「本次未跑」的命令带 --nocapture。
                         log_content 读日志的那一遍里逐行认（按 LOG_SCAN_BYTES 一块块读，不另读、不整份进内存），
                         数记号的处数、记前 GPU_NOT_RUN_TESTS_SHOWN 条 test= 点名的用例，finish_attempt 记进尝试的 NOT_RUN_MARKS_COLUMN（record_gpu_not_run_marks）；结局说明（finished_preface）
                         多一段「这一趟有 N 条用例本次未跑」、逐条点名、余下只报条数，同时推这一项（gpu_cases_not_run_items），出路照异常表 gpu-cases-not-run 那一行；
                         退出码与结局种类照命令的（命令退 0 仍是 succeeded）；只推一回、不再推。弄坏开关 gpu-cases-not-run-unread 下不认这种行，结局说明不写、不推，
                         gpu-not-run-line-start-only 下只认行首，gpu-not-run-word-edges-unchecked 下不判两侧词界，gpu-not-run-color-codes-kept 下不先去颜色码，
                         gpu-not-run-first-mark-only 下一行只算先出现的那一处；
       workspace-write-blocked  换钩子的命令（scripts/gate/hook-swap.py，不带 --feed-only）退 7（上一趟换上没写完成，拒绝再换、一个字节没写）：报给被挡的这个会话，依据点名上一趟疑似是库里更早的哪条
                         被杀的换钩子任务（workspace_write_blocked_items），出路照异常表那一行；只推一回、不再推。弄坏开关 workspace-write-blocked-unreported 下不报；
       previous-swap-running  换钩子的命令（同上，hook_swap_command）退 9（上一趟换上还没写完成、起它的进程还在这一台上，拒绝再换、一个字节没写）或退 10（同一个 --scratch 或这个仓的真换正被另一趟 hook-swap.py 占着，没建树、没写文件）：
                         报给被挡的这个会话（previous_swap_running_items），出路照异常表那一行（退 9 的等那一趟写完成再原样重交，退 10 的等占着的那一趟结束再原样重交），退 10 的另按占着的是哪一种写：
                         服务读最后一次尝试的日志（swap_busy_cause），日志里以「✗ 草稿目录」起头的拒绝说明是同一个 --scratch 被占，出路另写两趟里有一趟是 --dry-run 的
                         （这一条自己是，或占着的那一趟是）可以换一个 --scratch（干跑不占这个仓、真换先占这个仓，两趟真换换了草稿目录照样被仓占着）；以「✗ 这个仓的换上」起头的拒绝说明是这个仓的真换被占
                         （结局文件按仓只有一份），换草稿目录没有用、不叫换 --scratch；两种都出现的取最后出现的；读不到日志或两种都没有的写两种都可能、叫照日志里的拒绝说明办、不给只对一种成立的出路；
                         包着的换钩子（bash -c 串、命令位置上起的脚本正文里的，wrapped_hook_swap）带 --dry-run 这类只读参数的同样报，同命令那一层。
                         只推一回、不再推。弄坏开关 previous-swap-running-unreported 下都不报，swap-scratch-busy-unreported 下退 10 的不报，swap-busy-cause-unread 下退 10 的不读日志、一律按同一个 --scratch 被占写，
                         swap-busy-unknown-guessed 下认不出的按同一个 --scratch 被占写，wrapped-read-only-hook-swap-unread 下包着的只认写的那几条；
       long-run-unconfirmed  在跑、累计满两小时（上面 ①「跑满两小时提醒」）、没确认、不是带 --full-gate 的：提醒门槛是 1 加撞过上限的次数（撞了上限归零，再满界算跨过一道新门槛），
                         依据写累计与各次用时、命令、范围声明、日志末几行，出路照异常表 long-run-unconfirmed 那一行（核范围与用时，不对的 task cancel 改范围重交，对的照整行 task confirm <号> --why 确认）；
                         只提醒、照跑；没确认的照 ANOMALY_REPUSHED_KINDS 再推。没有会话的（人手 --manual）不推，这一次尝试的日志接一行 lib.LONG_RUN_LOG_MARK 起头的（long_run_sessionless_note，每次尝试一行），status 打一行「长跑待确认：…」，
                         同样可以确认。弄坏开关 long-run-unconfirmed-silent 下不报，long-run-sessionless-unnoted 下人手交的不写日志那一行；
       fills-any-cap     在排、在跑，这条命令键最近 FILLS_CAP_RUNS（3）次跑成的尝试峰值都不低于各自上限的 FILLS_CAP_FRACTION（90%）（任务 8979：按给的上限开工作进程，
                         7 行也记了 45.6 GiB）：依据写三次的上限与峰值，出路照异常表 fills-any-cap 那一行（「这条命令按给的上限开并行，核实」；fills_any_cap_items；只读进程内存口径的行）。
                         撞了上限的那几次不算：撞的那一次峰值本来就贴着上限，照上限一档档撞上去的（任务 9616 那条静态核 6.0、9.0、13.5 GiB）不是按上限开并行。
                         上限照记下的峰值定的那几次（尝试的 prediction 一栏是 record）也不算：峰值贴着上限是定法的结果。
                         弄坏开关 fills-any-cap-silent 下不报，fills-any-cap-counts-cap-hits 下撞了上限的也算进那几次，fills-any-cap-counts-record-caps 下照记录定上限的也算。
       repeated-cap-hits 在排、或下一次在跑（刚收尾而还没定去向的那一刻不判），这条任务末尾连着 lib.LOOP_LEAK_REMINDER_CAP_HITS（3）次已收尾的尝试结局都是 memory-cap-hit
                         （lib.trailing_cap_hits；任务 9868 四次撞上限峰值 1.51、5.50、6.88、9.57 GiB，9731 六次一路到 31G）：每条任务只报一回（不在 ANOMALY_REPUSHED_KINDS 里，
                         第四次再撞也不再报）；依据写连着撞了几次、第几次到第几次，项另带 loop_leak（lib.loop_leak_check_text：要查的两类与各次上限、峰值、是不是一次比一次大、
                         是不是都顶满、最后一次内存最高那一刻的明细），wait 那一行在 basis= 之后打 loop_leak=、推给会话的正文在「出路：」之后另起一行；出路照异常表 repeated-cap-hits 那一行，
                         任务照旧乘 1.5 重排（repeated_cap_hits_item）。弄坏开关 repeated-cap-hits-silent 下不报。
     提醒门槛（reminder_threshold，项里的 threshold）：时长判的那几种，满上面那个时长提醒一回，满它的 2 倍、4 倍……每翻一倍再提醒一回，threshold 是跨过了几道门槛，
     另带 reminded_after_seconds（这一回跨过的那道门槛，秒）；cleanup-pending 的门槛是尝试的次数，shard-advance-failed 的是开始推不动过几回。
     同一条任务同一种提醒、同一道门槛只报一次：报给提交它的会话就记下报到了哪道门槛（别的会话来等照样看得到、不记），跨过下一道门槛才再叫。
     报到的门槛记进库里提醒说的那一条任务的行（正文的 reminded_thresholds：{种类: 报到的最高门槛}，报过的那几回在 reminder_history；经 Store.change 只并进这两栏；note_attention_reported），
     服务重启后照库认、不重发（会话 46 的任务 8819 那 30 片发过一轮，主 agent 重启服务换批之后同一道门槛又发一轮）；服务内存里另留一份照读（reminded_thresholds），库写不进时只记内存、照旧不重报。
     任务行里没有这两栏的当没报过：照此刻跨过的门槛报一回，第几回从 1 数起。
     弄坏开关 attention-reported-memory-only 下照改前只记在内存里，服务重启后再报一次。
     排队过久那一种，排队原因还没判出来（服务刚起来、排队线程这一轮还没判到它；waiting_text 交空串）的这一轮不报、不发，等判出来那一轮再报，原因不空
     （上面那一回重启后的第二轮原因写的是「排队原因此刻读不到」）；弄坏开关 attention-queue-reasonless-sent 下照改前照报、原因写读不到。
     wait 读到还没报的提醒就返回：任务没结局的回 {finished: false, attention}，客户端打 TASK_SERVER_ATTENTION 一行、退 ATTENTION_EXIT；结局已有的（出结局时才判的那几种）
     {finished: true, attention} 打在理由行之前，退码仍是任务的。阈值取值的理由写在常量上（QUEUE_ATTENTION_MINUTES、QUIET_ATTENTION_MINUTES、IO_WAIT_ATTENTION_MINUTES）。
     TASK_SERVER_ATTENTION 那一行的 why= 超过 lib.ATTENTION_WHY_LIMIT（300）个字符的按字符截到那么长、尾巴接「…」（lib.attention_line）；发给会话的那一条另有一行「原因：」写全文，
     全文在那一条里只出现这一回；task status 的「在等：」不截。弄坏开关 attention-why-unclipped 下照全文写。
     这几种与疑似卡住、结局，服务另直接发消息给提交它的会话（下面 ⑥），与 wait 同一份记账：发过的 wait 不再报，wait 报过的不再发。
     服务健康（库写不进、排头能起却一直没起成、看进展、单元不在、NFS、第二台用不了）不归 wait，服务自己盯、发给每个有任务在排在跑的会话（下面 ⑥「服务健康」）；
     服务自己挂了发不了：task wait 连续连不上满 60 秒就退出报错，task submit 连不上当场报。
     弄坏开关 attention-queue-unreported、attention-cleanup-unreported、attention-shard-advance-unreported 各关一种，
     attention-finished-unreported 下结局有了的 wait 不带提醒，attention-reported-every-wait 下报过的照报。
  ② 排队：在排的任务按任务号从小到大逐个看（同一个队，轻重不分，不另设延迟；重排的回到原来的位置；分片的子任务与普通任务同一个队）。行核不过的不起、记 record-tampered、写告警。
     两台同一段逻辑（用户 2026-11-02：「两台型号都是一样的 根本没有必要写 if 第一台 else 第二台这种代码」）：记账、判放不放得下、判卡住与 CPU 用量、记峰值、
       上限收法、编译目录、挑台与盘余量对每台走同一段，台是参数（多机配置 HOSTS 里的地址、那台的总上限与余量、编译根）；按台不同的只在「这一台怎么读、怎么起」的接口
       Machine 里（LocalMachine：服务所在这一台，直接读 cgroup、起内存包装；PeerMachine：HOSTS 里另一台，经 multi-host-run.sh --peer 起、经 ssh 探测），
       台的先后是 MACHINE_ORDER。只给自证证红的弄坏开关里照改前的样子分台的那几处（host-lists-restored、workspace-writers-local-only、peer-overflow-blocked、
       margin-local-for-both、pinned-cap-this-host-only）不算。
     机器：照 task_server_lib.py 文件头「挑机器」：放得下的几台里挑放下之后剩余占比大的（按余量均衡）；撞上限重排不钉台，到了这台最大才换到总上限更大的那台；
       没有只派一台的名单（用户：「【只能在本机跑】 是不能接受的。必须尽快解决 让错误暴露错来」）：计划、合并与各片也不钉台，哪台由余量定；派到第二台跑红的照红，
       由脚本那一侧修，服务不替它挡（弄坏开关 host-lists-restored 下照改前放回 RESTORED_HOST_LISTS 那几张名单与计划的 pin）。第二台在同一棵共享的树里、同一个绝对路径上跑
       （仓、工作根与会话记录目录经 NFS 两台挂在同一绝对路径）；派不派哪一台只看那一台此刻用不用得了与余量（host_eligible），派不过去的三种在提交时就拒了（下面「提交时拒」）。
       改主工作区的工具（hook-swap.py、apply-writer-patch.py、stage-mine.py、scripts/kb/ 下的）派得到第二台（改的就是同一棵树）；合入后验证（merge-and-verify.sh）只在自己拷的快照上
       打补丁与验、主工作区一个字节不写（它文件头「快照（不拿锁）」），不算改主工作区（modifies_workspace）；
       同一件的兄弟片不避开彼此（task_server_lib.py 文件头「挑机器」）：放不放得下只看内存额度，同一台能同时跑几片。
     提交那一路的进出日志（SubmitTrace）：POST /tasks 在事件循环上记下收到的时刻，活在线程池里做（submit_in_thread），回话时打一行（以 SUBMIT_TRACE_TEXT 起头）：
       收到的时刻、结局（收下、拒了哪一种、出错），与进线程、提交时拒、认会话、认任务、记库、提交时警告、确认单、回话各做完时距收到多少秒；平常一条提交只这一行。
       一步本身用了 SUBMIT_TRACE_SLOW_SECONDS（5 秒）以上的当场另打一行：客户端等回话等满 30 秒退 78 时，日志里看得到慢在哪一步。弄坏开关 submit-trace-silent 下不打。
     提交时拒（用户：「改成提交时当场拒（推荐）：不再悄悄改派本机：碰到这三种就在提交时报错、点名是哪个路径或哪一族、出路写怎么改，任务不跑。」；placement_refusal）：
       POST /tasks 收到时判，命中头一种就回 400、不进库、不起尝试，task submit 退 3、打那一句与出路；分片的子任务不另判（父任务提交时判过）。
       每一种在 task_server_lib.py 文件头「异常表」有一行（lib.SUBMIT_REFUSAL_KINDS；提交时警告两种是 lib.SUBMIT_WARNING_KINDS）。
       ① 共享根外的路径（find_outside_shared_path）：工作目录与命令里的绝对路径（只认整个 shell 词：以 / 起头的整个词、「名字=绝对路径」「--选项=绝对路径」的值、
          shell -c 脚本词照 shell 词切开之后的每一个；别的词中间夹着的 / 不算；两台相同的系统路径 HOST_INDEPENDENT_PATHS 不判）有不在多机配置 NFS_SHARED_ROOTS 任何一段下的：
          点名那个路径，出路是改放某一段下。
          落在 VOLATILE_ROOT（/tmp）下的同样拒，那一句另说重启会清 /tmp（问题 93，用户：「防止重启后 内容丢失，但是被认为做完了」）。弄坏开关 outside-path-accepted 下不判。
       ② 共享根（shared_roots_problem）：这一次要交给命令的 SINGLEFS_DATA_ROOT、SINGLEFS_WORK_ROOT（服务自己的，或任务带的白名单变量）有不在 NFS 共享根下的：
          点名变量与值，出路是配多机配置。弄坏开关 shared-roots-outside-accepted 下不判。
       ③ 判定库（verdict_library_refusal）：带不带判定库只照登记表 .claude/gate.d/verdict-library-scripts.tsv 判（registry_families，不另问别的脚本）：
          命令里有词点到登记表里带判定库的一行、而那一行第三列不以 prune-server 起头的：点名那一族（登记的路径），出路是那一族接进 prune-server、把第三列翻成 prune-server；
          登记表读不了、写坏了的同样拒（判不了）。临时 prune-server 的包装剥开、照原命令认。弄坏开关 verdict-library-family-accepted 下不判。
       ④ 临时 prune-server 包装里会编的（temporary_service_heavy_refusal；用户：「旧脚本不兼容 我再说一次 如果错了让 对方重新提交」）：这层包装
          （python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service [--server-binary <路径>] -- <原命令>）只给脚本自证用；
          照 task_server_lib.unwrapped_temporary_service 剥出原命令（只用来判，收下的照旧整条执行），原命令照 compiles 的认法有会编的 cargo 的，
          点名原命令，出路是去掉包装重交（验证包的 cargo test 对真服务判）。服务不剥包装去迁就：编译目录只照外层那一条给，
          包装里的认不进来（任务 4229 包装里套 cargo test --release -p singlefs-verification --features gpu，记成不编，编进了仓根 target/）。
          包装自己要现编的另算（量到的：任务 6778 派在本机、退 0，跑的是改包装之前的旧版，里层命令与 6834、6835 不同，不能当本机的样本；
          任务 6834、6835 同一条命令派到第二台、退 0，第二台的编译目录由 multi-host-run.sh 照例给了。推的、没有任务实测过：compiles 为 false 的包装派本机，
          服务不给编译目录，包装的 resolve_binary 没有 CARGO_TARGET_DIR 就退 2）：
          不带 --server-binary 的包装在服务给的编译目录里现编 prune-server，compiles 把外层这一条认成会编的（lib.temporary_service_builds_server），两台都由服务给编译目录；
          带了 --server-binary 的照旧不编。弄坏开关 temporary-service-build-unrecognized 下不认。
          自带临时服务的两份脚本单独的 --selftest（prune_server_client_selftest.py 还有 --selftest-families、prune-server-load-simulation.py）不带 --server-binary 与 PRUNE_SERVER_BINARY 时
          同样自己现编，同样按会编的算（lib.own_server_selftest_split），两台都给编译目录；带了二进制的照旧不编。弄坏开关 temporary-service-own-server-unrecognized 下这两份不认。
          弄坏开关 temporary-service-heavy-accepted 下不判（改前的样子：照外层收下）。
       ⑤ 不点名包的全仓编译（whole_workspace_cargo_refusal；用户：「8105打回 让他重新写重新确认范围」：任务 8105 不带 -p 的 cargo build --offline --all-targets
          预测 49.0 GiB，同一天 8275、8325 同一条，8104、8324 不带 -p 的 cargo clippy --all-targets --all-features，各会话撤掉后都带 -p 重交）：
          提交的这一条在命令位置上直接起的、会编的 cargo（WHOLE_WORKSPACE_SUBCOMMANDS：build、check、clippy、test、run、bench、doc 与它们的单字母别名；认法同 compiles，
          bash -c 与 cd 跟着走，入口脚本与门禁里面起的 cargo 不读、不判；fmt、metadata、tree、clean 不编，不判）带 --workspace / --all 的拒；没带 -p / --package 的，
          照 cargo 找清单的走法（lib.cargo_manifest_scope：起的目录往上第一份 Cargo.toml）落到虚拟清单（只有 [workspace]、没有 [package]：不点名就编每一个成员）的拒，
          起的目录认不出、清单读不了的同样拒（判不了）；落到包自己的清单（scripts/experiment-apparatus、prune-server 这类自成工作区的包，或 cd 进了一个成员的目录）只编那一个包，照收；
          往上一份清单都没有的 cargo 自己就报错，不判。那一句点名哪一条 cargo、在哪个目录起、照哪一份清单；出路是点名要编的包（-p <包>，验证包 -p singlefs-verification），
          整仓要跑的走 merge-and-verify 或提交时的门禁。弄坏开关 whole-workspace-cargo-accepted 下不判。
       ⑥ 整包测试与不指定范围的跑法没带确认标识（whole_package_refusal；.claude/agent-common.md「日常开发的测试不跑整个包」；.claude/main-agent.md「一轮怎么开、怎么收」；
          异常表 whole-package-unconfirmed）：照命令实际跑什么判（判法在 task_server_lib.py 的 whole_package_cargo_test、whole_package_script、unscoped_run_entry）——
          整包测试：命令位置上直接起的 cargo test 不点名 --test、--bin、带测试名的 --lib，或带 --lib --tests、--workspace（带 --workspace 的先被 ⑤ 拒）；
          .claude/gate.d/crash-point-replay.sh 的 --check 点了 product-gpu-real-card-tests、点了 crash-case-replay 而不带 --crash-cases-only、或点了 feature-gated-tests
          而没带 feature-gated:<文件名> 项，点哪一格、点几格都算（--check crash-case-replay --crash-cases-only --item crash-case:<名> 只判点名的崩溃枚举用例，
          --check feature-gated-tests --item feature-gated:<文件名> 只编、只跑点名的挂特性文件，都不算）；
          不给补丁目录、或带 --all-packages 的 scripts/gate/merge-and-verify.sh（带 --all-packages 的那份脚本自己也退 2；补丁改到工作区根的 Cargo.toml、Cargo.lock、.cargo/
          或不落在任何包里的路径的，它判不出测试目标、测试那一步不跑、退 3，不整包跑，这里不另判）。
          不指定范围的跑法：lib.UNSCOPED_RUN_ENTRIES 表里的门禁与入口不带选条参数——提交时的六道门禁与 gate-staged.sh 不带 --check、--item；scripts/crash/crash-point-replay.sh、
          scripts/crash/crash-amplification.sh 不带 --item；scripts/experiment-apparatus/replay.sh 不给实验号（--admission-only 只问准入，不算）；加一种照表加一行。
          带全量门禁标识的（task submit --full-gate，同 ① 的跑满两小时提醒）不判这一种。COMMIT_GATE_ENTRIES（含 scripts/crash/crash-amplification.sh）里不带选条参数而没带
          全量门禁标识的先拒，整包确认标识顶替不了（弄坏开关 commit-gate-without-full-gate-accepted 下不判）。用户点头要在提交之外跑的别的入口照同一个确认标识交。
          弄坏开关 whole-package-cells-all-three-only 下 --check 要三格全点才算整包，merge-all-packages-accepted 下 --all-packages 不算，unscoped-gate-accepted 下不指定范围的照收，
          whole-package-feature-items-uncounted 下点了挂特性文件的 feature-gated-tests 照算整包，whole-package-any-item-exempts 下 feature-gated-tests 带了任何一个 --item 都不算。
          没带 task submit --whole-package-confirmed <确认文件> 的拒；带了的只判形式（lib.whole_package_confirmation_problem：绝对路径、在共享根下、读得到、
          有 ①②③ 起头的三段、每段去掉空白至少 12 个字），不合的同样拒、点名缺哪一段。核过照收，任务记 whole_package_confirmed（那份文件的路径），
          task status、列表、推给会话的正文与日志头打一行「整包已确认：<文件>」（flag_lines）；带它照样受两小时提醒（① 的跑满两小时提醒）。
          形式不合的那一句不论是不是整包测试都拒（给了标识就判它）。弄坏开关 whole-package-unconfirmed-accepted 下整包测试没带标识也照收；
          whole-package-confirmed-exempts-two-hours 下带标识的不提醒。
       ⑦ 启动测试与验证没点名（unnamed_run_refusal；records/2026-11-17-subagent账本自上而下清欠.md 第二百二十九节「收口要做的」；异常表 unnamed-run）：
          判据只一份，task_server_lib.py 的 unnamed_run_problem（范围声明闸 .claude/hooks/run-scope-guard.sh 调同一个函数），对提交的命令与 bash -c 里命令位置上的每一条
          （lib.shell_inner_commands）各问一回：cargo test 没在 `--` 之后带 --exact 与测试全名；scripts/gate/merge-and-verify.sh、scripts/crash/crash-point-replay.sh、
          scripts/crash/crash-amplification.sh 没带 --item；scripts/mutation/prove-red.sh 没给变异名；.claude/gate.d/ 的门禁阶段 --item、--check 都没带，或 --check 点了
          起测试的格（lib.TEST_STARTING_GATE_CELLS）而没带 --item；门禁 crash-point-replay 带了 --item 而照它要跑的格里还有整格跑的（lib.crash_gate_whole_package_cells：
          没给 --check 而点了崩溃枚举用例的照旧九格，crash-case-replay 不带 --crash-cases-only，feature-gated-tests 没点 feature-gated:<文件名> 项，product-gpu-real-card-tests
          不收点名）——拒，说明写哪一种没点名与照这条命令改成点名的样子（门禁 crash-point-replay 那一种写加 --check crash-case-replay --crash-cases-only）。只查不跑的写法（--list、--list-items、--selftest、
          --no-run、`-- --list` 这类）不判。带 --full-gate（提交时那一趟，full_gate_refusal 先核过）或 --whole-package-confirmed（⑥ 先核过确认文件的形式）的不判。
          入口脚本与门禁里面起的不读（只判提交的这一条）。弄坏开关 unnamed-run-refusal-skipped 下不判；判据里的开关见 unnamed_run_problem。
          自证的小世界照 lib.UNNAMED_RUN_SELFTEST_VARIABLE 写 off 不判（各格演别的规矩），判它的 unnamed-* 几格照真的判。
       ⑧ 入口文件头写了领卡、要卡的声明（card_declaration_refusal；.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」；异常表 card-declaration）：
          提交的命令与 bash -c 里命令位置上的每一条起的脚本，文件头前 lib.DECLARED_HEADER_LINES 行里写了 lib.CARD_DECLARATION_MARKS 之一（# task-server-card-lease:、
          # task-server-gpu-cards:、# task-server-gpu-arguments:）的拒，说明头一句是「卡归 GPU 批处理验证服务，任务不领卡」、点名那份脚本与写了的那几种，出路写删掉那一行、
          要卡的那一段交 GPU 服务判（判法 lib.card_declaration_problem）。服务不认这几种声明、不照没写那样收（.claude/agent-common.md「里程碑 17 收尾之前不兼容任何旧东西」）。
          弄坏开关 card-declarations-accepted 下照收。
       ⑨ 一份副本上证红进行中、会编译的命令落在这份副本里，或对一份副本的证红开跑时副本里已有会编译的任务（copy_proof_refusal；异常表 copy-under-proof）：
          证红（scripts/mutation/prove-red.sh --copy <副本>）在副本里就地改源码再还原，同一份副本里别的编译编到的是打了变异的源码。两头各判，只比同一份副本，不编译的命令不判：
          会编译的命令（compiles）、工作目录与它命令位置上会编的 cargo 起的目录（cd、-C、--manifest-path 之后的，copy_proof_roots）落在某份副本里，而这份副本上有证红——
          在排、在跑、待确认的证红任务（lib.prove_red_copies 认 --copy 点名的副本）、留着还原日志 .prove-red-pending（被停在施加变异之后，源码此刻带着变异）、或副本锁持有者记录
          .prove-red.owner 在 300 秒内刷新过（lib.copy_proof_marker，从起的目录往上找；副本锁自己、记录的写法在 scripts/gate/apply-writer-patch.py 的「副本锁」）——拒，
          说明点名那份副本、哪条证红或哪一种标记，出路写等那条证红出结局再交（留着还原日志的重跑同一条证红命令，它开头把源码写回）或另拷一份副本；
          提交的是对某份副本的证红，而副本里已有会编译的任务在排、在跑、待确认（另一条证红不算：副本锁自己拒）——拒，点名那条任务，出路写等它出结局再交证红或换一份副本。
          这一判只在交任务那一刻（submit_in_thread，拿 copy_proof_lock 与记库同锁），不在 submit_refusal 里：服务起来时重核在排的不判它，在排的两条互相看见会一起打回。
          证红脚本里不做这一判：它在服务给的单元里跑，读不到别的任务；扫本机进程认「副本里有没有在编」只看得见这一台。
          弄坏开关 copy-proof-build-accepted、copy-proof-start-accepted 下各不判一头，copy-proof-every-copy-judged 下不比副本，copy-proof-every-command-judged 下不看会不会编译。
     提交时警告（用户：「绕过的 那个要做成提醒 谁做错了严厉警告」；repository_target_bypass）：照收照跑、不拒，服务当场打警告，警告进三处——提交回复（客户端打到标准错误）、
       这条任务每一次尝试的日志头（start_attempt 在起单元之前写进日志，分片的子任务照父任务的）、status（task status <号> 与不带号的会话列表）。头一行以「✗ 严厉警告：」起头，
       点名交任务的会话号与任务号、绕过了什么；第二行以「→ 怎么办：」起头写该怎么改。判在 POST /tasks 收下之后、进库之前，记进任务的 target_bypass（几处说明），
       三处照它与会话号、任务号现拼（bypass_warning_lines）。
       ① 取仓根 target/ 下的二进制，绕过服务给的编译目录（任务 7937–7944：长跑脚本默认取仓根 target/ 下合并前的旧二进制，跑出缺包的红）：
          仓根是服务的仓根与主工作树（草稿与临时目录下的仓副本不算）；命令里的词（整个词、「名=值」的值、bash -c 词里的每一段，相对的照工作目录解）落在仓根 target/ 下的，
          或命令位置上起的本仓脚本正文里有指向仓根 target/ 的路径字面（前面接仓根变量、$(dirname …)/../.. 这类拼法、$(git rev-parse --show-toplevel)，接 target/release/ 或
          target/debug/；工作目录就是仓根的另认裸写的 target/release/、target/debug/），就算命中（lib.repository_target_references，读法同读脚本头的声明：command_word_index 剥过之后那一个词）。
          不算：服务给的编译目录与调用方给的 CARGO_TARGET_DIR 指的目录（都在仓外）、草稿与临时目录下的 target/、正文里的注释与弄坏开关那一支、
          自证那一支（带 --selftest 起的不读正文；名字带 selftest 或以 cells 收尾的函数体不读：memory_peaks_lib.py 的 key_fold_cells 那种样本里造的假路径）。
          弄坏开关 repository-target-bypass-unwarned 下不判。
       ② 会话名下有结局没人取（用户同一句；unclaimed_before、unclaimed_warning_lines）：判在 POST /tasks 收下之后、进库之前：交任务的会话名下有父任务（子任务不算）
          结局满 UNCLAIMED_WARNING_MINUTES（30 分钟）还没交给它（collected 为假：服务发的结局消息没送到，也没起 wait 取），记进新任务的 unclaimed_at_submit，
          三处照它现拼：头一行「✗ 严厉警告：会话 <号> 有 <N> 条任务跑完了没人取（<号>、…）：…」，第二行「→ 怎么办：task-server/task wait <号> 取走（…）…」。
          没有会话的（人手）不判。弄坏开关 unclaimed-unwarned 下不判。
       三处都经 submit_warning_lines 取 ①、② 两样（分片的子任务照父任务的）。
       第二台用不了要告警（note_peer）：余量那一轮判出它用不了（连不上、余量查不成、多机配置判不过、NFS 复核判出没挂上共享根、盘余量不到 DISK_FLOOR_BYTES）、
       或那一趟没跑成（peer-run-failed：ssh 被重置、第二台那边的准备没过），写告警 peer-unavailable（用得了变用不了、原因变了才再写一条；盘不够这一种按 DISK_FLOOR_KIND 只写一条），
       /health 的 peer 一栏写明原因，满 PEER_UNAVAILABLE_NOTIFY_SECONDS 服务发「服务健康」给会话（下面 ⑥）；这时只在本机排着的任务排队原因带「第二台此刻用不了：<那一句>」。
       盘余量（Resources.disk_problem；task_server_lib.py 文件头「挑机器」的「盘余量」）：两台同一个下限。读每台余量的那一次（同一个缓存周期）另判那台编译目录所在盘
       （本机是 TASK_SERVER_BUILD_DIRECTORY 所在盘；第二台是多机配置 BUILD_ROOT_BY_HOST 里它那一项，multi-host-run.sh 在它下面建每一趟的本地目录）的余量：
       报告里有 DISK_REPORT_KEY 照它，没有就照那一台的接口现量（Machine.disk_free：本机读盘，第二台起 PEER_DISK_PROBE_TEXT）；不到 DISK_FLOOR_BYTES（50G）或查不成，
       这一轮不给那一台的报告、不派它，原因写「<台>盘只剩 X（下限 Y）」；够了下一轮自己认回。弄坏开关 disk-floor-unchecked 下不判。
     内存：上限与记账只有一条定法，只照服务自己库里的尝试记录（task_server_lib.py 文件头「资源记录」）；脚本与调用方不报内存量，父任务与同一趟的兄弟片的记录不借；自己这把键没有相当的记录的（片照片数），
     与没有记录的命令一样从 4G 起，别的键的记录不借。
       同一命令键最近 RECORD_WINDOW 次跑完的尝试（退 0 的记量到的峰值，撞了这一条上限的记那一次的上限，命令退 250 之前报了要多少的再取不小于报的量加余量，cap_hit_measurement；别的结局不进、不占名额；recorded_measurements）
       取最大，预测、上限与记账都等于它（上限按 MiB 向上取整，不乘 1.5；recorded_prediction）。库里没有这把键跑完的尝试的，预测、上限与记账都取
       task_server_lib.UNRECORDED_MEMORY_DEFAULT_BYTES（4G；unrecorded_prediction）。撞了 250 由 retry_decision 乘 1.5 重排，重排的那一次照它定的上限（next_cap；computed_prediction）。
       钉值先于这一条（pinned_prediction）：分片的各片吃父任务的钉值，计划与合并不吃。
       弄坏开关 recorded-cap-floored 下把下限抬到 4G，钉值之外低于 4G 的上限与记账都抬到 4G（有记录的头一次、重排都兜；memory_floor、memory_floor_applied）。
       内存上限的下限 lib.MEMORY_FLOOR_BYTES（1 GiB；task_server_lib.py 文件头「资源记录」的内存上限下限）：记录的峰值只算进程内存，上限还要装页缓存，峰值很小的命令
       （任务 11127 峰值 5 MiB，上限 3M、5M、8M 都退 250）照记录定不出装得下的上限；computed_prediction 把钉值之外的预测（有记录的头一次、重排、没有记录的）上限与记账都取到
       不低于它（memory_floor_applied），预测的峰值照原样；记账照这个上限（booked_at_cap），收到台的上限为止，所以下限不超过台能给的最大上限；钉值不经这里（钉值低于下限也照钉值）。
       弄坏开关 cap-floor-skipped 下不设下限。换文件、打补丁的工具（workspace_swap_tool：换钩子、打补丁）的下限另取 WORKSPACE_WRITER_FLOOR_BYTES（4 GiB，workspace_writer_floor）：
       撞上限被杀会把换到一半的结局文件留在主工作区、之后所有会话的换上都被挡，所以不照峰值抠；合入后验证不在其内（照这把键的记录定上限）；自证里换了内存下限的世界不加这一条。弄坏开关 workspace-writer-cap-unpadded 下不加。
       读尝试记录只读峰值口径一栏是「进程内存」的行（current_metric；task_server_lib.py 文件头「资源记录」的峰值口径一栏），弄坏开关 old-metric-peaks-read 下连旧行一起读。
     余量（用户：「这部分最大上限看起来要动态计算」「2gb 要动态计算 … 向上取整」；host_margin、host_capacity、host_limit）：每台单条能给到的最大上限 =
       那台 slice 的总上限（多机配置 SLICE_MEMORY_CAP_BY_HOST 里那台那一项，配多少就是多少；第二台另取它那一项的小者）− 那台此刻的余量。
       余量是那台此刻 singlefs-heavy.slice 里回收不掉、又不归任何在跑任务的那一截，按 1 GiB 向上取整（scripts/hosts/memory_peaks_lib.py 文件头「余量」：怎么量、
       读哪几项）；它随那台的余量报告一起读（Resources.snapshot，第二台照 host-dispatch.py 的 probe_peer 在第二台上量），两台各用各的，不存、不跨轮缓存：
       派活、记账、整片的界、钉值、撞上限重排与「上限到了总上限」、起之前复核都照判的那一刻那一份报告里的那一台的数。配置里没有余量的键（写了照多机配置判法 crash-point-replay-shard-configuration-check.sh 认不得的键判不过，不另认）。
       哪一台量不到余量（没报告、cgroup 读不出、第二台探测失败）、或总上限不比余量大、或钉了的命令在那台的钉值不大于 0：那一台这一轮不派，排队原因写哪一台、为什么；
       能派的每一台都这样：不起，结局记 run-condition-unmet（退 78），说明写每一台的原因与出路。不退回固定的数或旧值，也没有让它退回的配置键与运行开关（下面几个弄坏开关只给自证证红）。
       弄坏开关 margin-fallback-fixed 下量不到的退回 2G；margin-local-for-both 下第二台也用本机的余量；margin-unmeasured-waits 下每一台都量不到时照排着等；
       booked-up-to-slice-total 下单条的上限不减余量（照改前收到总上限；钉值照减）。
       准入记账（用户：「当然， 谁超了杀谁 不要连累别人」「检查内存分配 不得大于配置中的设定」；booked_at_cap）：交给 judge_host 的记账量 = 这一次在那台上的上限；
       预测峰值、上限与记账都收到那台总上限减那台此刻的余量为止，钉值、记录、缺省、撞上限重排四种来历一样收，记录比整台大的也不判永远放不下。
       预测峰值比某一台的可用上限大、别处可能装得下（能派的台里有装得下的、有这一轮给不出可用上限的，或多机配置里有一台此刻连不上）的，那一台照永远放不下判、不派
       （hosts_short_of_record，judge_placement、预算那一判与看进展照同一处判），任务去装得下的那台或排着等；能派的每一台都比它小、也没有别处的，
       照判的那一台收到的数起跑，收到最大还撞上限的照 retry_decision 打回 memory-cap-at-maximum。
       弄坏开关 booking-over-host-limit 下钉值之外峰值比可用上限大的照原样记（judge_host 判永远放不下）；record-over-every-host-never-fits 下每一台都比它小时照永远放不下判。
       在跑的记账（running_booked、running_booking）：只照此刻实测记。报告里有它的 scope 的记 min(它的上限, scope 此刻回收不掉的用量)，不加固定余量、不看任何预先估的数；
       报告里没有它的 scope 的（刚起、读不了）记整份上限。各自的上限（memory.max）照旧，超了只杀它自己。
       公式（room_by_the_formula）= 配置的总上限 − 余量 − 在跑的记账之和：记账之和不超过配置的总上限；在跑的上限之和多大都不进公式。
       每条的上限（host_limit）照总上限判（收到总上限减余量为止，上面准入记账）。看进展与记账明细都照同一个总额。
       内存放不下的排队原因（memory_wait_text；judge_host 那一句照它换，memory_verdict_reworded）写明两个数各是什么：账面还能记多少（总上限、余量、在跑的记账之和）、
       slice 实测还放得下多少（总上限减 slice 里进程用的，再减余量与刚起还没读数的那几条之后剩多少），整机空闲减待涨量不够要的空闲的另写一句，
       末尾点名这台记账最多的至多 MEMORY_WAIT_TOP_RUNNING 条在跑任务与各自的记账、实占。弄坏开关 memory-wait-text-unnamed 下照 judge_host 与公式原来那一句写。
       弄坏开关 running-booked-at-cap 下在跑的一律照上限记，running-booked-with-margin 下有 scope 的在实测上加 1 GiB 记（不超过上限）。
       记账对账（audit_bookings，booking_audit_loop）：服务每 BOOKING_AUDIT_SECONDS（60 秒）逐台比一回
       「这台在跑的尝试（booking_attempts）的内存记账之和」与「对账的基准：这台实测的用量（报告里各 scope 回收不掉的用量之和）」；
       记账之和不少于基准的 BOOKING_DRIFT_RATIO（3）倍、而且多出至少 BOOKING_DRIFT_BYTES（8 GiB），连着 BOOKING_DRIFT_ROUNDS（3）轮，判「记账漂移」：
       漂移只来自报告里没有它的 scope、记整份上限的那几条；起跑不满两分钟、照规矩记整份上限的那几条（running_booking 的 young）对账时照实占算、不算漂移
       （弄坏开关 booking-audit-young-counted 下照整份上限算）；
       逐条列各尝试记多少、实占多少、为什么记这么多（取的是此刻实测、实测到了上限，还是报告里没有它的 scope 记整份上限，booking_row），写进 /health 的 booking_drift 一栏（memory：{台: 明细}）与 control check 的输出（「记账漂移」行），并推给被它挡住、还在排的任务所在的会话
       （排队原因里写着「在跑的记账之和」的；push_booking_drift）：同一件事对同一个会话判出来当场推一回，之后照 reminder_threshold（门槛从 BOOKING_DRIFT_PUSH_SECONDS 起、每翻一倍一道）每跨过一道门槛再推一回，同一道门槛不重推，
       对得上了忘掉、再漂移再推。只报不停任务。弄坏开关 booking-audit-skipped 下不对账。
       slice 兜底（slice_floor_problem）：那台 slice 实测还放得下的（报告的 slice_room：总上限 − slice 里进程用的内存）要不小于 SLICE_ROOM_FLOOR_BYTES：
       不到下限的这一轮不派新任务（slice_floor_verdict），排队原因以 SLICE_FLOOR_TEXT 起头、写明是实测余量不够（内存那一判也放不下的接在后面；看进展不算它）。弄坏开关 slice-floor-ignored 下不判。
       每条的上限从哪来（computed_prediction）：只认这把命令键自己的记录（recorded_prediction：近 RECORD_WINDOW 次最大）；自己没有记录的从 4096M 起，别的键、同一趟的兄弟片都不借，
       撞了上限乘 CAP_GROWTH_FACTOR 重来。
       整机内存线（machine_memory_problem）：余量报告（scripts/hosts/memory_peaks_lib.py 的 headroom）带这台整机的 mem_total、mem_available 与 swap_total、swap_free；
       可用内存占总量的比例与 swap 空闲占总量的比例都低于 MACHINE_MEMORY_LINE_FRACTION（12%，earlyoom 在两样都低于 10% 时才杀进程，这一道在它之前停派）的，这一轮不往这台派新任务，
       排队原因以 MACHINE_MEMORY_LINE_TEXT 起头、写两个比例与两个字节数；只挡新派，不停在跑的（看进展不算它）。没有 swap 的机器 swap 空闲按 0% 算（可用内存低于线就停派），报告里读不到这几项的不判。
       弄坏开关 machine-memory-line-ignored 下不判。
       判放不放得下照 host-dispatch.py 的 judge_host，减去的「还没涨到量」那一截 = 服务自己账上这台在跑的每一次尝试 max(0, 它的记账 − 它的 scope 此刻回收不掉的用量)
       加派发器账上刚派出去的那几笔（server_pending）。两台同一段：用量照这一轮读那一台余量的那一份报告（probe_local / probe_peer 量的
       memory_peaks_lib.SCOPES_REPORT_KEY，第二台是同一次 ssh；scope 照里面进程环境的 TASK_SERVER_ATTEMPT_LABEL——每次尝试两台都给，task_server_lib.ATTEMPT_LABEL_VARIABLE——
       对到尝试的 label 上，memory_peaks_lib.py 文件头「在跑的 scope」；几个 scope 带同一个标签（命令里再套内存包装另开的平级 scope）取用量最小的那一份，不往少里记待涨量）。
       报告里没有这一条的 scope（还没起来、读不了、这一轮那台没报告）记整份上限，不记 0。第二台那一层内存包装不排队、不记账（run-with-memory-cap.sh 文件头「排队、记账、挑机器、
       定额度都归仓根 task-server/」那一句、multi-host-run.sh 文件头 ⑤），记账只在这一处，不重复扣。准入只在这里判一处：
       放得下才起；放不下的留在库里排着，不起包装进程去空等；排头放不下时后面的照下面「排队先后」两条规则；
       没满时限的排着的一律不留账（用户：「现在都在数据库里面 记什么账？？？？  每一条能不能启动 是启动时候才查的 记账做什么？」）。
       哪台都等多久也放不下记 never-fits；点名第二台而它用不了同样记 never-fits。
     排队先后（place_queued_into、jump_refusal；用户定「1 超过两个小时的任务阻塞后续的任务。2 插队的任务的时长不得超过正在运行的任务剩余的时长。没有上次运行时间的任务不得插队」）：
       提交满 AGED_WAIT_SECONDS（两小时）还在排的排到最前、先判（Scheduler.aged、queue_order）。只有两条规则，不交替、不封组、不插空、不留位：
       ① 排满时限还起不来的任务（内存、CPU，不论等的是哪一样）挡住排在它后面的全部任务：它起来之前后面的都不起，排队原因写「排在任务 N 后面：它已排满时限」。
          弄坏开关 aged-blocked-lets-others-start 下后面的照常判、照常起。
       ② 前面有没起的任务（没排满时限）时，后面的先起算插队，只拦争同一样资源的，按前面那条缺的是什么分：
          缺的只是某一棵树的编译目录名额（判放得下的各台都只缺它：编译目录组满，Placement.build_tree、pool_only_tree）：名额是那棵树在那一台的编译目录，
          后面的起来不占它，放得下就起（不要编译目录的、编译目录是调用方给的、别的树的）；只拦同一棵树也要服务给编译目录的，排队原因写「它等的是这棵树的编译目录名额…不插队」。
          缺的是内存、CPU、占盘、磁盘压力这类：后面的只许这样的先起：它有上次的运行时长（last_run_seconds：这把命令键上次真跑完用了多久，是记录、不是估的），
          这个时长不超过那台在跑的任务剩余时长里最长的那个（剩余 = 那条任务上次的运行时长减已跑；没有上次时长、或已跑过上次时长的不算数）；
          在跑的都算不出剩余时长时，有上次运行时长的许先起（用户定）；没有上次运行时长的不插队，排队原因写「不插队」并点名它缺的是内存、CPU 这类、不是编译目录名额。
          前面两种都有时两样都判。弄坏开关 jump-length-unchecked 下放得下就起；jump-refused-any-resource 下缺编译目录名额的也照缺内存、CPU 的那一种判。
       例外（idle_short_text）：这一条在每一台上就算服务在那台跑的任务全收尾、整机空闲也不够要的（它要的空闲贴着那台可用上限，整机空闲到不了），它等的不是排在前面的任务：
          不算「排在前面还没起的」，排满时限也不挡后面的；排队原因里接 IDLE_SHORT_TEXT 那一句与各台各是什么，同时记一条提醒 booking-beyond-idle-machine（上面 ①）。
       服务不预计用时（用户：「没有预计用时这个东西。我们整个项目现在不预计用时。仅仅预测内存。」）：内存照记录定上限、撞了乘 1.5；
       一项实际跑满四小时停掉打回、跑满两小时提醒拆分（下面 ⑦；带 --full-gate 的提交门禁不受这两条限）。
     预算（用户：「一个任务预算是 50GB 明显是有问题的」；budget_check）：排队每一轮判过 never 与 unmet 之后，这一条在派得了的某一台上的记账（booked_at_cap：收到那台
       可用上限为止，上面准入记账）不小于那台整片界的照常排，同时记一条要处置的提醒 oversized-budget（上面 ①）；预测峰值比那台大、别处可能装得下而不派那台的
       （hosts_short_of_record）不算那一台。记账收到可用上限为止，没有超过可用上限的记账，这一判不打回。判不了的这一轮不判。
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
       同一条拿它自己那一把锁算（prediction_task_locks），扫记录、读钉值不拿存预测那几张表的锁（prediction_lock）：排队线程算这一条时，确认单与别的任务取预测不等它。
     CPU 闸（cpu_problem；调度溢出审核 CPU-1、CPU-2）：max(那台此刻的负载（1 分钟负载与可跑进程数的大者）, 服务在那台在跑的尝试的线程之和) + 这一条的线程 ≤ 核数 × CPU_OVERSUBSCRIPTION（3.4375，32 核的台线是 110）
       才派（弄坏开关 cpu-oversubscription-three 下照 CPU_OVERSUBSCRIPTION_LOOSE 倍）；在跑的线程一直记着（在 D 状态等 IO 的负载里看不出），与负载取大、不相加。在跑的每一条记几线程（running_threads）：
       起跑满 CPU_MEASURED_AFTER_SECONDS、服务量满了一段的，记它最近量满的那一段的在用核数（latest_running_cores：任务忙完一段闲下来，账上跟着降）：服务每一轮（check_stall）与算账时各记一回
       Machine.cpu_usec 的累计读数（note_running_sample，窗口 CPU_MEASURED_AFTER_SECONDS 秒、留起点那一端），两端相隔满 CPU_MEASURED_AFTER_SECONDS 的算一段的在用核数（两端读数之差 ÷ 相隔秒数），
       取最近那一段，向上取整、至少 1、不超过登记数。各段取最大的那一份（峰值段核数，note_peak_segment、peak_segment_running_cores）只给收尾记的 peak_segment_cores、cpu_gate_threads
       与名额让不让（small_slot_released）用，向上取整、至少 1、
       不超过登记数（弄坏开关 cpu-running-cores-over-registered 下不限；cpu-cores-by-average 下照到此刻的平均在用核数记）：前半段编译、录制用核少，后半段满核的任务按后半段记，不按整次平均记低。
       可跑线程数（机器挤满时在用核数被压低：几条各要 31 核的挤在一台上各只拿到七到十核，照在用核数记，下一条同键的就照十核放进来）：服务每一轮（check_stall 的 note_threads）
       读一回这一次尝试此刻的线程读数（Machine.thread_reading → ThreadReading：本机读 attempt_runner.py 每轮采样写进尝试目录的 lib.THREADS_FILE_NAME，那一条进程树里
       /proc/<号>/task/<线程>/stat 状态是 R 的线程数、线程总数与占线程最多的进程；第二台读日志里 multi-host-run.sh 每轮打的
       「MULTI_HOST_RUN_PEER_THREADS poll=<轮数> runnable=<R 的线程数> total=<线程总数> top=<线程数>:<进程名> read=<yes|no>」（PEER_THREADS_LINE，这一行不算日志在涨；
       上一回读之后接到的几行里 read=yes 的取最大，一行 read=yes 都没有的这一回读不到，一行都没接到不动））。在跑的那一支记
       min(登记数, max(最近那一段的在用核数向上取整, 最近一回读到的可跑线程数))（running_latest_runnable；收尾记的峰值段那一份照 running_peak_runnable：各段取大再取大）；
       读不到线程读数的那一轮只照在用核数。
       收尾时记进尝试 cpu_gate_threads（CPU_GATE_THREADS_COLUMN，finish_cpu_gate_threads）= min(登记数, max(收尾记的峰值段核数, 读到过的可跑线程数的最大))，峰值段核数没记的不记；
       记录读这一栏，没有这一栏的旧尝试不读（等同没有记录）。弄坏开关 cpu-runnable-threads-unread 下不记可跑线程数、只看在用核数。
       靠名额起的让不让出名额（small_slot_released）与收尾记的 peak_segment_cores、cpu_gate_threads 只看峰值段那一支（连同可跑线程数）。
       起跑已满 CPU_MEASURED_AFTER_SECONDS 而服务还没量满一段的（刚重启）照到此刻的累计 CPU ÷ 已跑秒数记（cgroup 记的累计重启不丢；弄坏开关 cpu-restart-unmeasured-by-record 下照下一句）。
       起跑没满 CPU_MEASURED_AFTER_SECONDS 的、量不到的与这一条自己要起的取法相同（threads_by_record）：
       同一命令键近 RECORD_WINDOW 次跑到头的（跑成，或命令自己退非 0 而不是被信号杀的；被停的、撞上限的、被外部信号杀的不算）CPU 闸线程数（尝试收尾时记的 cpu_gate_threads，finish_cpu_gate_threads）
       取最大、向上取整、至少 1、不超过登记数，没有记录（没记 cpu_gate_threads 的尝试不读，这一栏之前的旧尝试同样不读）、尝试没有命令键的量到之前先按登记的线程数记（用户定）；
       在跑的那一条按尝试号存下头一次取的（running_threads_cache），收尾时删；起之前的那一条存在 admission_threads_cache，
       跟内存预测一起清。弄坏开关 threads-record-succeeded-only 下只认跑成的，cpu-wanted-by-registered-threads 下起之前的一律照登记数，cpu-running-unmeasured-registered 下在跑却没实测到的一律照登记数，
       cpu-unrecorded-by-one-core 下没有记录的先按 1 核，cpu-cores-by-average 下记录照尝试的平均在用核数（cores）取平均，
       cpu-threads-registered-only 下在跑的一律照登记数记（32 核的台同时至多 2 条）。
       要核数少的名额（cpu_judgement）：超线的这一条要的核数不超过 CPU_SMALL_TASK_CORES（2）时照样放行，每台同时在跑的、靠名额起的尝试不超过 CPU_SMALL_TASK_SLOTS（4）；
       名额满了照排队，原因写「CPU_SMALL_SLOT_TEXT N 个已用满」。靠名额起的尝试记 cpu_small_slot（一句说明，起的时候照 judge_placement 与起之前复核那一份余量判，cpu_small_slot_note），
       名额照库里在跑的尝试数（booking_attempts），服务重启后照库接着认；起跑那一行（「尝试起了」）写明靠名额起的。读不到核数或负载的那一支不走名额。
       靠名额起的起跑满 CPU_MEASURED_AFTER_SECONDS、峰值段核数（peak_segment_running_cores，与 running_threads 同一个取法）超过 CPU_SMALL_TASK_CORES 的不再占名额（small_slot_released：
       没有核数记录的重活先按 1 核过了闸，量出来不小就把名额让给真要一两个核的）；尝试上的 cpu_small_slot 与起跑那一行照留，量不到 CPU、还没量满一段的照占着。
       弄坏开关 cpu-small-slots-unlimited 下名额不设上限，cpu-small-slots-withheld 下不给名额，cpu-small-slot-heavy-still-counted 下实测超了的也照占。
       余量报告里读不到核数或负载的那一台这一轮不派，排队原因以 CPU_UNREAD_TEXT 起头。弄坏开关 cpu-threads-settling-only 下只记刚起 CPU_SETTLE_BREAK_SECONDS 秒的、与负载相加，cpu-unread-passed 下读不到的放行。
       排队原因里的负载拆开写：余量报告的 CPU 一项带 blocked_threads（此刻状态 D、多半在等 IO 的线程数；本机读 /proc 逐个线程的 stat，第二台走同一份报告，memory_peaks_lib 的 cpu_report），
       原因写成「负载 N（其中等 IO 的线程 M）」；闸的判法不看这一项。弄坏开关 cpu-load-io-unsplit 下不拆开写。
     第二台的余量不在排队线程里读（loop 审核发现 3；Resources.peer_probe_loop）：一个后台线程（task-server-peer-probe）每 PEER_PROBE_REFRESH_SECONDS 连第二台读一回（多机配置的判法、探测与 df
       各带 PEER_PROBE_TIMEOUT_SECONDS 的时限），派活、NFS 复核取它最近一份（peer_snapshot_entry），不在自己的线程里等 ssh；读不成的那一回记下，PEER_PROBE_FAILURE_HOLD_SECONDS 之内不再去连、
       当第二台此刻用不了；最近一份比 PEER_PROBE_STALE_SECONDS 旧的同样当用不了。起之前复核照旧现读那一台（带同样的时限，
       还在不再去连的那一段里的不连）。尝试收尾时对第二台的清理回执仍在排队线程里做。弄坏开关 peer-probe-in-scheduler 下在调用方的线程里现连。
     排队一轮的读数（loop 审核发现 1、6、8、12）：在跑的尝试库没变就不重读（Store.running_attempts）；
       预算那一判的「记账从哪来」只在记 oversized-budget 时算（budget_check）；每一轮开头先清 repass_soon，队空、暂停着的那一轮主循环照 TICK_SECONDS 睡。
       弄坏开关 running-attempts-reread、pass-estimates-recomputed、budget-source-every-pass、repass-kept-when-idle 各退回那一处。
     余量一轮只读一次（PassView）：每台的余量（Resources.snapshot）、服务账上还没涨到量的那一截（server_pending）在这一轮第一次用到时读，
       这一轮里各条共用；这一轮起了一条，它的记账加进这一轮那台的待涨量，后面的照加过的判，同一份余量不判给两条。/health 判排头另读一份。
     编译目录占盘（records/2026-10-31-task-server巡检.md 问题 76；build_disk_problem）：会编的、或这条命令量过编译目录占盘的，那一台盘余量（本机是
       TASK_SERVER_BUILD_DIRECTORY 所在盘，第二台是多机配置 BUILD_ROOT_BY_HOST 里它那一项所在盘，报告里都是 DISK_REPORT_KEY）减去服务账上
       那台在跑的尝试编译目录还要涨的（起的时候记的占盘 − 此刻量到的，量不到的照记的算；server_disk_pending、Machine.build_bytes）要放得下「这条命令的占盘预测 + BUILD_DISK_FLOOR_BYTES」，
       放不下就排着、排队原因点名「编译目录占盘」；预测是同一命令键最近 RECORD_WINDOW 次跑成而量到了占盘峰值的最大值（取法同内存，build_disk_prediction），
       起了的尝试记 disk_booked。弄坏开关 build-disk-unpredicted 下不判。接着用组里那一个的，那一个已占的盘在余量里扣过了：预测先扣掉组里空着的那几个记下的占盘里最大的
       （free_build_slot_bytes；组里每一个的占盘在每次尝试收尾时照量到的峰值记，settle_build_slot），在跑的还要涨的照记账减此刻量到的算。弄坏开关 build-disk-reuse-uncredited 下不扣。
     编译目录组（build_pool_problem、take_build_slot；task_server_lib.py 文件头「编译目录」）：服务给编译目录的（会编、调用方没给），这棵树在那一台的组里有空着的、
       或还没满 BUILD_POOL_SLOTS_PER_TREE 个才放得下，都占着又满了就排着、排队原因点名「编译目录组满」；判放得下之后、起之前组被同一轮别的那一条拿满了的，这一次不起、留在队里下一轮再判。
     磁盘压力（io_pressure_problem；多机配置 TASK_SERVER_IO_PRESSURE_LIMIT，task_server_lib.py 文件头「路径」）：放行会编译的任务到一台之前，读那一台 /proc/pressure/io 的 full avg300
       （Resources.note_io_pressure：本机读文件、第二台经同一条连接起 PEER_IO_PRESSURE_PROBE_TEXT，记进那一台余量报告的 IO_PRESSURE_REPORT_KEY），超过阈值就排着；
       排队原因（status 的「在等：」）以「磁盘压力：」起头，点名哪一台、读数与阈值；不会编译的任务不看它；多机配置没写阈值就不按磁盘压力排（不补缺省）；
       阈值在服务起来之后才被改坏的，会编译的任务这一轮不派、原因写那一句；这台读不到读数的不按压力排。起之前复核现读的那一份同样带读数。
       弄坏开关 io-pressure-ignored 下不判，io-pressure-applies-to-all 下不会编译的也排，io-pressure-limit-defaulted 下没写阈值的补缺省，io-pressure-peer-reads-local 下第二台照本机的读数判，
       io-pressure-wait-unnamed 下排队原因不点名磁盘压力、这一台与读数。
     起之前复核：判出要起的那一条，起之前现算一次它的预测、现读一次它要去的那台的余量与待涨量（缓存作废），按现读的数再判一次那台放不放得下；
       放得下才起，上限与记账照现算的；放不下的留在队里，排队原因写「起之前复核没过」与现读的那一句；现读的那份换进这一轮，后面的照它判。
       现读的那一台照排队那一轮同样判：盘余量不到下限或查不成的这一轮不派（Resources.recheck 调 disk_problem；第二台的探测不带盘余量，现量的那一份记进报告，
       编译目录占盘照它判；弄坏开关 recheck-disk-unchecked 下不判）；现读的那份过一遍 peer_view（复核这一刻第二台没挂上共享根的当它此刻用不了；
       弄坏开关 recheck-nfs-unchecked 下不过）；现读的那份换进这一轮（PassView.refresh）。
     卡（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）：卡归 GPU 批处理验证服务（gpu-verification-server/），它照多机配置开卡、一直持着；这个服务只调配 CPU：
       不按命令或脚本名字认谁要卡、不分卡、不领卡、不清场、不收卡，派活只看内存、CPU 闸、编译目录占盘与磁盘压力。入口文件头写了领卡、要卡的声明的提交时就拒（上面「提交时拒」⑧）；
       服务自己环境里带着的选卡变量（task_server_lib.CARD_CHOICE_VARIABLES）不交给尝试（start_attempt；弄坏开关 card-variables-passed-to-attempt 下照交）；
       起的那一条外面不套清场包装（attempt_command）。
     不认合入锁（用户 2026-11-02：「可以 按这个做 不要拿锁了」；验证在快照上跑）：主工作树正在合入时，会编的与派第二台的照常排、照常起。
  ③ 起：每次尝试一个脱离服务的用户级单元 singlefs-task-<库标签>-<任务号>-<第几次>.service（systemd-run --user --collect，KillMode=mixed），先把单元名写进库再起，只起一次。
     照现读的任务（不拿调用方手里的旧字典）备好目录、环境与说明之后拿这条任务的锁（Store.task_lock），claim_attempt 一个事务里判 queued 且没取消、父任务没结局也没在取消整件（admission 审核 F10）、
     分尝试号、插尝试、改任务（TS-01、TS-03；库里 (task_id, number) 唯一），之后起单元，起完才放锁；判不过的不起、备好的目录删掉，取消了的记 cancelled。
     弄坏开关 cancel-race-unguarded 下照改前不拿锁、先插尝试再拿旧字典整行写回任务；attempt-claim-split 下照改前插尝试与改任务分两次提交。
     systemd-run 超时（START_TIMEOUT_SECONDS）只算「起没起不确定」（TS-06：它是发给用户级 systemd 的请求，调用端超时不说明管理器没收下）：照预先定好的单元名
     读回状态与起跑标记，起了的照常 running；判不了的尝试留在 starting、账照记、写告警 start-uncertain，交给对账（上面 ①）。弄坏开关 start-timeout-as-failure 下照改前当没起。
     单元里跑 attempt_runner.py，它照说明起这一条：
       本机：bash scripts/hosts/run-with-memory-cap.sh <上限> bash <仓根>/scripts/hosts/capped.sh <线程数> <命令…>，环境里设 RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER=<任务号>.<第几次>
             （内存包装不再排第二遍，① 每条上限与 ② slice 总上限照旧做内核层兜底）；会编的 cargo 另设 CARGO_TARGET_DIR，指这棵树在本机的组里拿到的那一个（task_server_lib.py 文件头「编译目录」）
       第二台：bash scripts/hosts/multi-host-run.sh --peer --memory-cap <上限> --label <标签> -- env <白名单变量>=<值>… bash <仓根>/scripts/hosts/capped.sh <线程数> <命令…>，
               在任务的工作目录里起：multi-host-run.sh 在第二台同一个绝对路径里 env -C 起（仓与工作根在 NFS 共享根下，两台同一棵树；multi-host-run.sh 文件头 ③ ⑤），
               不送、不取回、不改写路径、不留第二台的树；写出的文件就在共享的树与共享根下，两台都读得到。编译目录是这棵树在第二台的组里那一个：新开的由 multi-host-run.sh
               在第二台本地盘上建，接着用的带 --reuse-build-directory，都带 --keep-build-directory 跑完留着（服务不设 CARGO_TARGET_DIR，task clean 不管它）。分片那几片同样：计划目录与产物目录在共享工作根下（task_server_lib.py 文件头「分片」），交绝对路径。
               env 那一段带任务的白名单变量与下面「环境」点名的那几个；本机的路径与编译目录、按台取值的（PEER_HOST_SPECIFIC_VARIABLES）、值是本机绝对路径而不在共享根下的不带。
               这一层的环境里另设 MULTI_HOST_RUN_ADMITTED_BY_TASK_SERVER=<任务号>.<第几次>（task_server_lib.py 的 PEER_ADMITTED_VARIABLE）：multi-host-run.sh --peer
               只收服务直接起的这一次，任务里面再带 --peer（两台都一样）它拒、退 255（那份文件头 ①）；env 那一段不带它，本机的尝试不设。弄坏开关 peer-admission-unmarked 下照改前不设。
               弄坏开关 peer-tree-copied 下照改前拷树：加 --send <工作目录> <树里的落点>、命令换到落点里起（multi-host-run.sh 已不认 --send，退 2）。
       只认服务直接起的那一层：内存包装、capped.sh 与 multi-host-run.sh --peer 各认一个准入标记（task_server_lib.py 的 ADMITTED_VARIABLE、
       THREAD_ADMITTED_VARIABLE、PEER_ADMITTED_VARIABLE，值都是 <任务号>.<第几次>），各自读掉、不交给命令，
       于是任务里面的脚本再套这几样当场被拒（退 2 或那一份的用法错退出码），出路是删掉那一层。capped.sh 的那个两台都设（第二台的经 env 那一段带过去）；
       内存包装那个本机设在环境里，第二台的由 multi-host-run.sh 照它拿到的那一份设给第二台那一层内存包装。
     环境：服务自己的环境，加上任务带的白名单变量、TASK_SERVER_TASK、TASK_SERVER_ATTEMPT、TASK_SERVER_ATTEMPT_LABEL（这一次尝试的标签，两台都给：scope 照它对到尝试上）、TASK_SERVER_SOURCE_REPOSITORY（提交方那一份仓的根，本机绝对路径，第二台也给、
       拼两台一致的路径用）、SINGLEFS_DATA_ROOT 与 SINGLEFS_WORK_ROOT（服务这一侧的共享数据根与工作根的真路径，任务自己带了的照任务的；第二台原样带本机的值，
       脚本不经 ~ 与 $HOME 取：第二台的家目录是另一个）、TASK_SERVER_MEMORY_CAP_BYTES（这一次的内存上限，字节、十进制；
       第二台的经 multi-host-run.sh 那一段 env 带过去）、CLAUDE_CODE_SESSION_ID（提交它的会话，读得到才设）、
       分片那几步的 TASK_SERVER_SHARD_* 与 RUN_WITH_MEMORY_CAP_KEY（task_server_lib.py 文件头「分片」）、给内存包装的 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT（退出时峰值，
       task_server_lib.py 文件头「资源记录」）与 RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY（现场目录，同一份文件头「现场」；两样都只给包装那一层，包装读了就 unset、
       第二台的由 multi-host-run.sh 设给那边那一层，env 那一段不带）。服务自己环境里的选卡变量不交（上面「卡」）。systemd-run 退非 0 记 unit-start-failed。
     套在外层单元里的服务（自证起的临时服务，与起在一次尝试里的服务；任务 11815 被取消后第二台留着它的自证里临时服务起的 3 个尝试单元，只能等它们自己超时退）：
       服务在自证里（lib.in_selftest）或自己的环境里带着尝试标签（ATTEMPT_LABEL_VARIABLE）时，起来时照 /proc/self/cgroup 认出自己所在的用户级单元
       （task_server_lib.py 的 enclosing_user_unit：user@<uid>.service 之下离它最近的那一段 .service 或 .scope；nested_outer_unit），起每个尝试单元都带 -p BindsTo=<那个单元> -p After=<那个单元>：
       外层单元一停（取消、卡住被停、撞上限，外层的内存包装停它的 scope），systemd 当场停这些尝试单元，不靠它们自己超时；两台同一段（第二台的外层是 multi-host-run.sh 在那边开的 scope）。
       独立的服务单元（真服务）不带：尝试单元本来就要在服务重启之后接着跑。认不出外层单元（不在用户级管理器之下）的照不带起、服务日志打一句。
       弄坏开关 nested-units-unbound 下不带。
     停服务（收到 TERM / INT）：排队线程与 NFS 复核线程停下（各至多 STOP_JOIN_SECONDS）之后，服务自己起的子进程还在的（去第二台探测余量、复核挂载的 bash、ssh、python3 这类，
       KillMode=process 下它们留在服务单元的 cgroup 里，control stop 回读不空）逐个按进程号 TERM、至多等 STOP_CHILD_GRACE_SECONDS 再 KILL，停完才退
       （stop_own_children；弄坏开关 no-detach 下起成子进程的尝试照旧只发 TERM、不算在内）。弄坏开关 probe-children-left 下照改前不等、不停。
  ④ 分片：父任务不进队，子任务（计划、各片、合并）进队；子任务收尾时（advance_parent）与每一轮对账 drive_parent 照库里父任务的阶段与子任务集合现推下一步：计划成了照 plan.json 建各片，
     片都成了（人手单独取消的那几片不算缺，合并时点名，记在父任务的 missing_shards）建合并，合并成了整件成；
     有一步没成整件照那一步的类型收尾、点名是哪一步；一片没成时兄弟片照排照跑，各片收齐了才收尾（task_server_lib.py 文件头「分片」）。
     子任务记录经 children_of 读，有一行校验不过的抛 StoreRefused("tampered")：推父任务的记 record-tampered，status、clean、取消整件写告警、不认那几行（children_refused）。父任务的阶段（phase：plan、shards、merge、cancelling）与片的计划
     （shard_plan，计划读通之后写进父任务）都在库里，推法幂等：缺的子任务照阶段与计划补建（库里 (parent_id, role, shard_index) 唯一，同一步同一片只一个），
     子任务终态提交之后、建了一片之后、写了 outputs.tsv 之后、建了合并之后服务没了的，重启之后与每一轮对账照库里的样子接着推（TS-02）。
     计划那一步往计划目录下放的测试二进制（lib.SHARD_PLAN_BINARIES_NAME 那个子目录；.claude/gate.d/crash-point-replay.sh 文件头「分片」plan）：父任务有了结局（成、败、取消、打回都算；
     finalize_parent）与提交它的会话 task clean 时服务删（shard_binaries_removal）。只删这一个子目录，计划目录别的文件留着；它照真路径要恰好是这一条父任务自己的计划目录下的那一份，
     是符号链接、真路径落在别处的不删，服务日志打一行。删了几份、多大与不删的原因写进操作日志（action shard-binaries）。
     弄坏开关 shard-binaries-kept 下不删，shard-binaries-path-unchecked 下不核路径、照真路径删。
  ⑤ 暂停（control pause / resume；换 prune-server 的二进制、清判定库旧行这类要一段没有任务在跑的空档的活用）：暂停着，服务照收提交、照答 status / wait / log / cancel / health，
     排队线程不起任何新尝试（place_queued_into 开头与每一次 start_attempt 之前都判；分片的子任务、重试重排回队的都是新尝试，一样不起），
     对账照做（在跑的照跑到结局，写回结局、重排回队、分片父任务照推，只是回到队里的不起）；续起「起单元之前没了」的那一次（resume_starting）是在跑的尝试的延续，不算新尝试，照起。
     放行一个会话（control pause --allow-session <会话号>；管 prune-server 的会话要在空档里经 task submit 交清判定库的命令，暂停着起不来，放开之后再起又会和别的会话新起的任务撞）：
     暂停着，任务记的会话（POST /tasks 时 session_of 取的，task status 里「会话 …」那一栏，tasks 表 session 列）等于放行的会话的，照常排、照常起——排队线程只排这些、
     start_attempt_unless_paused 只放这些（Scheduler.allowed_during_pause）；分片的子任务继承父任务的会话，重试重排回队的还是同一条任务，所以都随提交它的会话。
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
  ⑥ 发消息给会话（用户：「不是你通知 而是 task-server 通知发起的对话」「包括报错 长时间没有响应 都可以这样通知」；inbox_loop，一个后台线程，
     每 INBOX_ROUND_SECONDS 一轮，记下结局时 notify_finished 当场叫醒它）：task submit 交来的收件地址记在任务上（上面「接口」POST /tasks），出事往会话的收件 socket 发一条。
     收件地址（inbox_socket_of）：每次发之前按任务记的会话号读会话登记（~/.claude/sessions/*.json，一份一个进程；读法照
     scripts/agents/lib_session_registry.py，lib.session_inbox_socket），取会话号相同、进程还活着的那几份里启动最晚的一份的 messagingSocketPath，一轮里同一个会话只读一次；
     令牌取交任务时带的。会话重启换了 socket、交时的 socket 已经不在的，照新的发；找不到的（会话已关、登记读不了）照下面「发不出去」写告警 inbox-unreachable。
     提醒、疑似卡住与服务健康只发给此刻找得到 socket 的（inbox_reachable；找不到的这一轮不发、不记发失败，wait 照取），结局照发、发不出写告警。
     任务之外的进程空转（watch_runaway_processes，runaway_loop 每 RUNAWAY_SCAN_SECONDS 一轮、两台都扫）：不在任务 slice 里、不是内核线程、累计 CPU 满 RUNAWAY_MINIMUM_CPU_SECONDS 且 CPU 累计占已跑时长不低于 RUNAWAY_MINIMUM_FRACTION 的进程（lib.runaway_processes），
     写告警 runaway-process，只发给它所属的会话（顺着父进程找到登记的 Claude Code 进程；地址取那个会话最近一条找得到收件 socket 的任务；谁的进程提醒谁，不发给别的会话），每 RUNAWAY_RENOTIFY_SECONDS 再报一次；认不出所属的（第二台的、发起的会话已退出的）不等、直接按号终止，写告警 runaway-process-stopped，不发给任何会话；不碰的（lib.runaway_unowned_stoppable：不在登录会话的 scope 里的，也就是 systemd 单元与系统服务；RUNAWAY_PROTECTED_COMMANDS 点名的；本机登记的会话进程与它们的祖先）只写告警。
     认得出所属会话的本机进程报满 RUNAWAY_REPORTS_BEFORE_STOP 次还在，下一次到点服务按进程号停掉（先 TERM、过 RUNAWAY_STOP_GRACE_SECONDS 还在发 KILL，回读 /proc）并告诉那个会话；所属会话发不到（没有任务可取地址）的照样数次数、到次数照样停。只扫这个账号的进程。报过几次、停没停整份另记进库（RESTART_SURVIVING_STATES），换服务后照库接着数（弄坏开关 runaway-reports-memory-only 下只记内存）；上次报它之后才起的同号进程是另一个，从零数（弄坏开关 runaway-reused-pid-carried 下照号接着数）。弄坏开关 runaway-unowned-not-stopped 下认不出所属的不停，runaway-protected-stopped 下不碰的也停， runaway-process-unreported 下不报，runaway-slice-counted 下任务 slice 里的也算，runaway-process-not-stopped 下只报不停，runaway-sent-to-every-session 下发给服务健康的那几个会话。
     弄坏开关 inbox-socket-from-submit-only 下照交任务时记的那一个 socket。往那个 unix socket 发一条：
     先发认证行 {"type":"auth","token":<会话令牌>}，再发一行 {"type":"user","message":{"role":"user","content":<正文>}}，两行先备好再连、一次写完（post_to_inbox；
     行的写法与来源在 task-server/README.md「## 发消息给会话」，文档页没写行格式，升级 Claude Code 之后要复核）。收方空着时 Claude Code 照它起新的一轮。
     正文第一行先写类别与任务此刻的状态，只看第一行就分得出是什么：「task-server：结局（任务已结束）任务 <号> <中文名>（<种类>，退 <码>；第 1 回）」（outcome_headline）、
     「task-server：提醒（任务还在排 / 还在跑，不是结局）任务 <号> <中文名>（<种类>；第 <N> 回，上一回在 <日期>）」（reminder_headline）、
     「task-server：疑似卡住（任务还在跑、服务没停它）任务 <号> 疑似卡住（suspected-stall，第 <段> 段；第 1 回）」（suspect_headline）、或「task-server：服务健康 <哪一样>」；每个种类带中文名（lib.KIND_NAMES），
     英文种类名在括号里；提醒说的是子任务的，状态照那一片的记录（分片的父任务一直是 sharded，三片都在排的提醒写还在排）；第 N 回是这一种提醒这条任务报给会话的第几回（任务行的 reminder_history 记着回数与时刻），重推的写上一回在哪一天。后面只写事实与出路（照 wait 打的那几行：依据、原因、出路；
     确认标识那几行「长跑已确认」「整包已确认」「全量门禁」在依据与出路之后；再是日志），末尾是 wait 那一行原样一个字不动。
     弄坏开关 inbox-headline-state-omitted、inbox-headline-round-omitted、inbox-headline-name-omitted 下首行各不写类别与状态、第几回、中文名，inbox-flag-lines-before-evidence 下确认标识的行排在依据之前。
     发哪几样：
       结局（push_outcomes）：父任务有了结局（wait 会打 TASK_SERVER_RESULT 的每一种）就发，先发还没报的提醒（改主工作区的没正常结束这类），再发结局；
         只发一次；发出去了记成已取（同 wait 把结局交给提交它的会话），会话已经用 wait 取了的不发。这一轮处理到某一条时抛错（库读不了、写不进这类），
         那一条与排在它后面还没处理的放回待发、下一轮接着发，已经发出去记了已取的不放回（不是「发不出去」：那一种照下面不再发）。
         弄坏开关 pushed-outcome-not-collected 下发出去了也不记已取，inbox-pending-dropped-on-error 下抛错时照改前不放回。
       在跑进度（push_progress；用户定的）：任务（分片的连同子任务）有尝试在跑，从最早那一次起跑满 PROGRESS_PUSH_MINUTES（58 分钟）给提交它的会话发一条进度
         （起跑以来多久、子任务几条结束几条在跑几条在排、每一次在跑的尝试在哪一台、起跑以来多久、平均在用几核、
         日志多久没涨（日志不涨时读到过进展的接着写哪份文件在涨；note_attempt_progress）与日志末行；progress_attempt_line），之后每满 58 分钟再发一条；只报、不停、不用处置。
         上一回的时刻记在任务行（progress_pushed_at、progress_pushed_rounds），服务重启后不重发同一回。弄坏开关 progress-not-pushed 下不发，
         progress-line-progress-unnamed 下不写读到的进展。
       结局没人取（repush_unclaimed_outcomes；异常表 unclaimed-outcome）：父任务的结局满 UNCLAIMED_WARNING_MINUTES（30 分钟）还没交给提交它的会话的（发结局的那一条没送到、
         也没起 wait），再推一回，之后每翻一倍（60、120 分钟…）再推；正文第一行「task-server：结局（任务已结束，这条结局还没人取）任务 <号> <中文名>（<种类>，退 <码>；第 <回> 回，上一回在 <日期>）」（上一回的时刻记在任务行的 outcome_repushed_at），写这条结局还没人取、
         此刻还差什么（那一行的修改意见），后面接结局那一条整段；推出去了记成已取。推到第几回记进任务行的 outcome_repushed，服务重启后不重推同一回。
         弄坏开关 unclaimed-outcome-not-repushed 下不再推。
       要处置的提醒与疑似卡住（push_reminders）：在排、在跑的任务每一轮照 attention_items 与尝试的 suspect 判，还没报的发；记账与 wait 同一份（reminded_thresholds、
         suspect_reported）：同一条任务同一种提醒同一道门槛只发一次，发过的 wait 不再报，wait 报过的不再发。弄坏开关 inbox-reported-every-round 下报过的照发。
         没处置时（attention_repush；异常表那几行的「没处置时」）：时长判的每翻一倍跨过一道门槛再推；门槛不随时长涨的（ANOMALY_REPUSHED_KINDS）同一道门槛报过之后还在的，
         这道门槛头一回报出去满 ANOMALY_REPUSH_SECONDS（30 分钟）再推第 1 回、之后每翻一倍再推，第一行写「（<种类>，没处置、第 <回> 回再推；第 <N> 回）」，接两行：
         「之前推过：…」（任务行 reminder_history 记的最近几回：隔了多久、当时的依据）与「此刻还差：…」（这一回的修改意见）；只发消息，wait 那一侧不再报。
         弄坏开关 anomaly-not-repushed 下不再推。
         出路是「确要这么多就照跑」的那几种（ACKNOWLEDGED_KINDS：oversized-budget、fills-any-cap）由提交它的会话 task-server/task confirm <号> --why '<为什么>' 记照跑
         （任务行 anomaly_acknowledged {种类: {why, at}}，不论跑没跑满两小时的界），记过的那一种不再再推；弄坏开关 anomaly-acknowledgement-ignored 下照推。
         认的是此刻开着的与库里记着报过的（budget_notices 只在内存，服务换上之后在跑的那一条没有开着的 oversized-budget）；弄坏开关 anomaly-acknowledge-open-only 下只认开着的。
         分片的子任务那几项按父任务、种类、提醒门槛、原因合成一条（attention_groups；任务 8819 拆成 30 片同时排满 30 分钟，逐片各发一条是 30 条
         只差子任务号的消息）：第一行「task-server：提醒（…，不是结局）任务 <父任务号> <中文名>（<种类>，子任务 <号的区间>，共 <几> 片；第 <N> 回）」，依据、出路与末尾那一行照号最小的那一片
         （merged_attention_text）；只有一片的照旧单发。说子任务的头一轮先压一轮（各片各记提交时刻，相差几毫秒的跨过一轮的边界会拆开），下一轮连同新跨过门槛的一起发；
         发结局之前那一次不压。task wait 那一侧照旧逐片各报一行。弄坏开关 inbox-attention-per-shard 下照改前逐片各发一条、不压。
       服务健康（watch_health、health_problems）：库写不写得进（试写一行）、派活判排头能起却一直没起成与看进展（/health 的 problem 读的那两句）、自己的单元在不在
         （systemctl --user is-active 的输出不在 RUNNING_UNIT_STATES）、NFS 共享前三项（nfs_loop 复核的那一份）、第二台用不了满 PEER_UNAVAILABLE_NOTIFY_SECONDS（180 秒）、
         GPU 服务卡坏与判不了的块（gpu_card_failures：另一条线程至多每 GPU_CARD_FAILURE_CHECK_SECONDS 秒起一回 gpu-verification-server/control cards-failed，只读两台 GPU 服务
         此刻的进程打的卡坏行与判不了的行、不碰卡；同一台同一个服务进程同一张卡算一样，出路是 GPU_CARD_FAILED_NEXT_STEP：请用户重置那张卡；同一台同一条流判不了的几块
         并成一样（不带进程号，正文写块数与头几块的块号），出路照各块的 claim= 写（gpu_block_not_judgeable_next_step：放了的另一台会去试，没放的等认领租期过）、
         两台都判不了的查那条流；头一回查出还往告警记录写一条 gpu-card-failed 或 gpu-block-not-judgeable，
         写过的种类记在换服务不丢的那一份里（RESTART_SURVIVING_STATES 的 gpu_alerted），服务重启之后不再写。cards-failed 说某一台没读成（unreadable、unreachable）时
         留着那一台上一回读到的，不当成好了（lib.gpu_service_problems_merged）；记着发过的 GPU 那几样什么时候忘掉照 gpu_health_settled：头一回读完之前不忘，
         卡坏的那一台自服务起来读成过才照读到的办（gpu_hosts_read），判不了的流不因这一回没读到就忘（GPU 服务重起之后同一条流再判不了不重发）；
         卡坏行取不到卡号、判不了的行取不到族、流键头、块号、认领或认领的值认不得的，那几行认不出：报一样「GPU 服务的卡坏行认不出」，不把卡号退成「?」、不猜认领）；
         NFS 复核连第二台的 ssh 一次超时先当「慢」：进复查期（NFS_SLOW_GRACE_SECONDS 秒）隔一会儿复查一回（task_server_lib.py 文件头「NFS 共享」的挂载，peer_shell_rechecked），
         期里通了的那一次只往服务日志记一行「第二台慢（超时、复查通了）：…」、记进 /health 的 nfs 一栏（slow_last：{at, slow}），不推、不当第二台用不了、照派第二台；
         复查期满还不通才算「挂载」不过，正文写「连不上第二台（ssh 超时 …，复查期 … 秒里隔 … 秒复查了 … 回仍不通…）」
         （三个会话各报过一次 30 秒超时的「用不了」，同一刻手跑 ssh 秒回、ping 0.09 ms，第二台 load average 约 60）。弄坏开关 peer-health-single-timeout-reported 下头一次超时就报，peer-recheck-once-only 下复查期里只复查一回；
         查出一样，往每个名下有在排、在跑任务又记着收件地址的会话发一条，出路是 HEALTH_NEXT_STEP；同一样对同一个会话只发一次，这一轮查不出了（好了）就忘掉，再坏再发；
         没有要发给的会话时不查。服务自己挂了发不了：task wait 连续连不上满 60 秒退出报错、task submit 连不上当场报，不另加。
         弄坏开关 health-unwatched 下不查不发，health-alerted-every-round 下每一轮照发。
     发不出去（照会话号找不到、socket 不在、连不上、会话已关；send_to_session）：不重试到死，写一行告警 inbox-unreachable（服务日志、告警记录）、往那条任务最后一次尝试的日志接一行
     lib.INBOX_LOG_MARK 起头的（不算日志在涨），任务照旧；那一样不再发（结局另照「结局没人取」满时限再推）。客户端读不到收件地址的当场拒（task 的 submit；异常表 inbox-missing），
     没记会话令牌的任务（人手经接口直交的）服务日志记一行「这条任务没有会话的收件地址」，出事只经 wait 取。
     要处置的提醒的记账与 wait 同一份，记在库里（上面 ①「要处置的提醒」），服务重启后照库认、不重发；疑似卡住报过的段、服务健康每一样发给了哪几个会话、记账漂移每一件推给每个会话推到哪道提醒门槛，
     都记在换服务不丢的那几份里（RESTART_SURVIVING_STATES 的 suspect_reported、health_notified、booking_drift_notified；弄坏开关 suspect-reported-memory-only、health-notified-memory-only、
     booking-drift-notified-memory-only 下只记内存），服务重启后同一件不再发；结局没人取推到第几回在任务行（outcome_repushed、outcome_repushed_at）；发失败过的（inbox_failed）记在服务内存里，
     那一件没送到过，重启后再试不算重发；
     重启前记下了结局而还没发的不补发（wait 照取，满 30 分钟没人取的照「结局没人取」再推，交任务时照上面 ②「提交时警告」②警告）。
     弄坏开关 inbox-not-sent 下一条都不发，inbox-auth-omitted 下不发认证行。
  ⑦ 交任务的二次确认（只拦长任务：提交它的会话确认这一趟的范围与个数对不对；submit、long_task_judgement、confirmation、confirm、expire_unconfirmed）：
     只有交的时候判成长任务的才要确认（long_task_judgement，判据四条，照下面的次序判、先中的就算，依据写进任务行 long_reason 与确认单「判成长任务」那一行）：
     ① 同一命令键库里记过的用时（completed_runs 的近 RECORD_WINDOW 次跑到退出的尝试，取最大）超过 lib.LONG_TASK_SECONDS（1800 秒），弄坏开关 long-task-record-unread 下不看，
     long-task-latest-only 下只看最近一次；② 不指定范围的整趟：范围声明种类是 whole（lib.declared_scope_kind）、带 --full-gate 或带 --whole-package-confirmed，弄坏开关 long-task-whole-unjudged 下不判；
     ③ 服务要拆成分片的（任务行 sharded_entry），弄坏开关 long-task-shard-unjudged 下不判；
     ④ 库里这把命令键没有用时记录、又是会编译的 cargo 测试（命令位置上有 cargo test，cargo_runs）或按项跑的门禁（命令里点了 --item，lib.named_scope），弄坏开关 long-task-unrecorded-unjudged 下不判。
     其余一律直接进队、不出确认单：服务的控制命令与脚本、scripts/gate/hook-swap.py、门禁单格（--check）与各脚本的 --selftest、.sh 与 Python 小脚本、记过的用时都不超过 1800 秒的命令；
     直接进队的任务行记 direct_queue（不是长任务的依据一句，回话带着，客户端在提交输出里打一行「直接进队：<依据>」）、state 照旧是 queued，不经下面的确认；
     弄坏开关 short-task-held 下不分长短一律待确认，direct-reason-unreported 下回话不带 direct_queue，long-task-reason-unsaid 下确认单不写「判成长任务」那一行。
     task confirm <号> --items <N> --tasks <M> 对直接进队的任务回 200「已在队里，不用确认」（只许提交它的会话），不核两个数，不让先交再确认的脚本与会话断掉；弄坏开关 confirm-direct-refused 下回 409。
     自证的小世界缺省写 lib.CONFIRM_RULE_SELFTEST_VARIABLE=all：不分长短一律待确认（演确认单的那几格照旧），判长短的几格写空串照真的判。
     POST /tasks 收下的判成长任务的任务记 state=awaiting-confirmation（lib.AWAITING_CONFIRMATION_STATE）、awaiting_state 记确认之后该进的状态（queued，分片的 sharded），
     不进队、不建子任务、不起尝试；回话带确认单（confirmation）：整条命令、判成长任务的依据、SINGLEFS_RUN_SCOPE 的四段原文、服务从命令里认出的项数 N（lib.named_scope：--item、--check 的每一格、
     点名的 --test、--bin、--lib 与 --exact 之后的测试名各几个，一个都没点的：整包测试、不带选条参数的门禁与入口、别的 cargo 写「整包、不指定范围」，别的写「范围由脚本里面定，服务算不出」，scope_is_whole 判，弄坏开关 scope-inside-script-unnamed 下一律写前一句；命令是 bash / sh -c 的，外层一个都没点时读脚本里每一条命令位置上点的与起的入口
     （lib.shell_inner_commands，不读被起脚本的正文；弄坏开关 shell-inner-scope-unread 下不读）、
     崩溃用例数与域（lib.crash_scope_text，弄坏开关 crash-scope-unreported 下不打：点名的 crash-case 项数与 --crash-expand-up-to 的值，合入后验证与崩溃点重放入口没点的写「范围由脚本里面定，服务算不出」）、这个会话此刻在排在跑的父任务条数 M（不算它自己与别的待确认的）
     与其中命令键相同或点的项交叠（同一条入口里点了同一项）的逐条列号、预测内存与记录来源（prediction_of、budget_source；算过的预测留着，进队之后照用）、
     异常表里交的时候就判得出的几种（budget_check：贴着整片界的 oversized-budget；fills_any_cap_items），连同那一行的修改意见。
     同一会话的确认串行（session_lock，按会话号的锁）：核两个数与改状态在同一把锁里，后到的照先到的进队之后的条数核。
     确认写 task-server/task confirm <号> --items <N> --tasks <M>（lib.confirm_command）：只许提交它的会话；两个数都与服务此刻现算的相同才进队（拿这条任务的锁改 awaiting_state、
     建子任务；任务号是提交时刻的，照它排位置），对不上回 409、打出服务算的两个数与确认单（异常表 confirm-counts-mismatch）。分片的父任务确认一次，子任务由服务建、不再要确认。
     满 CONFIRM_TIMEOUT_SECONDS（1800 秒）没确认的服务取消（结局 cancelled，说明以「没确认，已取消」起头），照结局发给会话（异常表 unconfirmed）。待确认的取消当场收尾。
     task status <号> 给待确认的打此刻现算的确认单。
     弄坏开关 confirm-skipped 下判成长任务的也交了就进队（不等确认，任务行记 direct_queue），confirm-counts-unchecked 下不核两个数，unconfirmed-kept 下满时限也不取消。
     打回建新号：凡打回的结局，异常表修改意见与推给会话的出路都提醒「不要使用原来的任务号，用 task-server/task submit 创建新的任务号」（lib.BOUNCE_NEW_TASK_REMINDER；弄坏开关 bounce-new-id-reminder-omitted 下不带）。
     跑满两小时（task-server/task confirm <号> --why '<为什么要跑这么久、范围为什么是这些>'，lib.long_run_confirm_command；confirm_long_run）：提醒 long-run-unconfirmed 的出路写两条：
     不对的 task-server/task cancel <号>、改范围重交；对的照整行确认。确认只许提交它的会话（没有会话的人手交的同样照会话号比，空对空）、只对在跑、累计满界、没带 --full-gate 的任务；
     --why 去掉空白至少 LONG_RUN_WHY_MINIMUM_CHARACTERS（12）个字（lib.long_run_confirmation_problem）；不合回 403 或 409，客户端退 3。确认了记 long_run_confirmed {why, at}，之后不再提醒，
     卡住、内存上限、占卡不用、封队列这些判法照旧。标识在 task status、列表、推给会话的正文与日志头都打一行「长跑已确认：<why>」（long_run_lines，经 submit_warning_lines 进前三处），
     确认那一刻在跑的那一次尝试的日志另接一行 lib.LONG_RUN_LOG_MARK 起头的；分片的子任务各确认各的，父任务的 status 连同各子任务打。
     弄坏开关 long-run-confirmed-flag-hidden 下不打标识那一行，long-run-short-why-accepted 下不判字数，long-run-confirm-unchecked 下不核会话、在不在跑与满没满界。
     四小时硬上限（异常表 over-four-hours；judge_run_hard_limit、run_hard_limit_verdict、parent_run_hard_limit）：带 --full-gate 的之外一律判，填过 why、确认过的（long_run_confirmed）同样判，--why 放不过；没有别的放宽。判点：
     跑着的：check_stall 每一轮（连同宽限里，重排之后刚起的那一次照样）算累计（run_limit_accumulated，撞了内存上限的归零），满 RUN_HARD_LIMIT_SECONDS（14400 秒）记 run_hard_limit_stop、只停这一次尝试（stop_over_limit_attempt，没停成的下一轮再停，服务重启了同样接着判、接着停），
     它停下来记结局 lib.RUN_HARD_LIMIT_OUTCOME（这一会儿命令自己退了的照它自己的结局走），settle_attempt 不重排、照结局 over-four-hours 收尾，退 229，推给提交它的会话。服务重启时已在跑的任务同样照这一条判。
     分片（用户定「分片的四小时硬上限按每一片自己的累计用时算」）：各片、计划与合并的子任务各按自己的尝试累计（第一个判点），整件不另判（parent_run_hard_limit 交 None）：
     排队等内存、等 CPU 把片一片片串起来的挂钟不算进谁的账。一片满界的照第一个判点停那一片、记 over-four-hours，兄弟片照排、照跑到底，收齐了照「一步没成」收：父任务记那一片的 over-four-hours，不合并。
     弄坏开关 shard-whole-by-wall-clock 下父任务另算整件有片在跑的挂钟（时段取并集，lib.covered_seconds；片之和不算、全都在排的时段不算、撞了内存上限的那一片它那几次不算），
     满界记 phase=lib.RUN_HARD_LIMIT_PHASE 与 run_hard_limit_stop：在排的子任务当场 cancelled（cancel_origin=hard-limit），在跑的由各自的 check_stall 停（run_hard_limit_verdict 看父任务的阶段），都收尾了父任务才记 over-four-hours。
     改主工作区的命令同样停：结局在 WORKSPACE_WRITER_INTERRUPTED_OUTCOMES 里，wait 与推送照 workspace-write-interrupted 提醒核现场；进队那一刻就打回、一次都没起的主工作区没动，不报。
     被停的这一次不进上次运行时长的记录（不在 lib.RUN_COMPLETED_OUTCOMES）与内存记录（不在 lib.RECORD_COMPLETED_OUTCOMES）。
     弄坏开关 run-hard-limit-skipped 下跑着的两个判点（各次尝试与父任务）都不判，run-hard-limit-full-gate-judged 下带 --full-gate 的也判，run-hard-limit-confirmed-spared 下确认过的不停，run-hard-limit-writers-spared 下改主工作区的不停，
     run-hard-limit-children-spared 下子任务不按自己的累计判，run-hard-limit-parent-summed 下整件的挂钟（shard-whole-by-wall-clock 那一支）取片之和，
     run-hard-limit-outcome-unmarked 下停的那一次不记 lib.RUN_HARD_LIMIT_OUTCOME，run-hard-limit-writer-unreminded 下 over-four-hours 不报 workspace-write-interrupted，
     run-hard-limit-shard-sum-judged 下分片的父任务的片之和也算，
     run-hard-limit-sheet-silent 下确认单不写，run-hard-limit-exclusive 下恰好等于上限不算到，run-hard-limit-loose 下比上限少一秒也算到。
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
import importlib.util  # noqa: E402
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
from task_server_constants import (
    ACKNOWLEDGED_KINDS, ACTIVE_ATTEMPT_STATES, ACTIVITY_MEMORY_BYTES, AGED_WAIT_SECONDS, ANOMALY_REPUSHED_KINDS, ANOMALY_REPUSH_SECONDS,
    ATTENTION_HISTORY_KEPT, BOOKED_FROM_CAP, BOOKED_FROM_USAGE, BOOKED_WHILE_YOUNG, BOOKED_WITHOUT_SCOPE, BOOKING_AUDIT_SECONDS, BOOKING_DRIFT_BYTES,
    BOOKING_DRIFT_PUSH_SECONDS, BOOKING_DRIFT_RATIO, BOOKING_DRIFT_ROUNDS, BOOKING_DRIFT_ROWS_SHOWN, BREAK_FIXED_MARGIN_BYTES,
    BUILD_DISK_FLOOR_BYTES, BUILD_POOL_FULL_PREFIX, BUILD_POOL_SLOTS_PER_TREE, BUILD_POOL_SWEEP_SECONDS, CANCEL_SETTLE_MARGIN_SECONDS,
    CANCEL_SETTLE_POLL_SECONDS, CLEANUP_RETRY_SECONDS, CONFIRM_TIMEOUT_SECONDS, COPY_PROOF_COPY_ADVICE, COPY_PROOF_STATE_TEXT,
    CPU_GATE_THREADS_COLUMN, CPU_MEASURED_AFTER_SECONDS, CPU_OVERSUBSCRIPTION, CPU_OVERSUBSCRIPTION_LOOSE, CPU_SETTLE_BREAK_SECONDS,
    CPU_SMALL_SLOT_TEXT, CPU_SMALL_TASK_CORES, CPU_SMALL_TASK_SLOTS, CPU_UNREAD_TEXT, DISK_FLOOR_BYTES, DISK_FLOOR_KIND, DISK_REPORT_KEY,
    ENDED_TAMPERED_FIELDS, EXTERNAL_KILL_LOG_AFTER_SECONDS, EXTERNAL_KILL_LOG_SECONDS, FAST_INBOX_ROUND_SECONDS, FAST_LOST_GRACE_SECONDS,
    FAST_NFS_RECHECK_SECONDS, FAST_PEER_PROBE_REFRESH_SECONDS, FAST_PEER_UNAVAILABLE_NOTIFY_SECONDS, FAST_PROGRESS_STALL_SECONDS,
    FAST_RESIDENT_PRUNE_SECONDS, FAST_RUNAWAY_SCAN_SECONDS, FAST_SCENE_KEEP_AFTER_COLLECT_SECONDS, FAST_START_UNCERTAIN_GRACE_SECONDS,
    FAST_STARVED_SECONDS, FAST_TICK_SECONDS, FAST_UNIT_CHECK_SECONDS, FAST_UNIT_UNREADABLE_ATTENTION_SECONDS, FILLS_CAP_FRACTION, FILLS_CAP_RUNS,
    GATE_MARKER_LIBRARY_PATH, GPU_BLOCK_NOT_JUDGEABLE_NEXT_STEP_TAIL, GPU_CARDS_FAILED_READ_SECONDS, GPU_CARD_FAILED_NEXT_STEP,
    GPU_CARD_FAILURE_CHECK_SECONDS, GPU_HEALTH_KIND_PREFIXES, GPU_LINES_UNPARSED_KIND, GPU_LINES_UNPARSED_NEXT_STEP, GPU_NOT_RUN_BREAK_SWITCHES,
    GPU_NOT_RUN_MARK, GPU_NOT_RUN_MARK_FORM_WITHOUT_EDGES, GPU_NOT_RUN_TEST, GPU_NOT_RUN_TESTS_SHOWN, GPU_NOT_RUN_UNNAMED, GPU_NOT_RUN_WORD,
    HEALTH_NEXT_STEP, HOOK_SWAP_ENTRY, HOST_INDEPENDENT_PATHS, IDLE_SHORT_TEXT, INBOX_DRAIN_SECONDS, INBOX_HEADLINE, INBOX_ROUND_SECONDS, INBOX_SEND_SECONDS,
    IO_PRESSURE_PATH, IO_PRESSURE_REPORT_KEY, IO_WAIT_ATTENTION_MINUTES, JUDGE_HOST_MEMORY_PREFIX, LEFTOVER_STOP_SECONDS, LISTING_ONLY_ARGUMENTS,
    LOCAL, LOG_CONTENT_LOCKS, LOG_CONTENT_LOCKS_GUARD, LOG_SCAN_BYTES, LOG_TAIL_BYTES, LONG_RUN_ATTENTION_SECONDS, LONG_RUN_IDLE_CORE_FRACTION,
    LONG_RUN_IDLE_QUARTER_BREAK, LONG_RUN_SPLIT_EXEMPT_FILE, LOST_GRACE_SECONDS, MACHINE_MEMORY_LINE_FRACTION, MACHINE_MEMORY_LINE_TEXT,
    MAXIMUM_LAUNCHER_DEPTH, MEMORY_WAIT_TOP_RUNNING, MERGE_AND_VERIFY_ENTRY, NFS_RECHECK_SECONDS, NOT_RUN_MARKS_COLUMN, PATCH_WRITER_CHECK_SECONDS,
    PATCH_WRITER_OUTCOME_NAME, PATCH_WRITER_READ_ONLY_ARGUMENTS, PATCH_WRITER_VERDICTS, PAUSE_META_NAME, PAUSE_REASON_CHARACTERS,
    PEAK_SEGMENT_CORES_COLUMN, PEER, PEER_BUILD_REMOVE_SECONDS, PEER_CLEANUP_LOCKS, PEER_CLEANUP_LOCKS_GUARD, PEER_CPU_LINE, PEER_DISK_PROBE_TEXT,
    PEER_EXTERNAL_KILL_LINE, PEER_HOST_SPECIFIC_VARIABLES, PEER_IO_PRESSURE_PROBE_TEXT, PEER_IO_WAIT_LINE, PEER_MEMORY_LINE, PEER_MEMORY_STAT_LINE,
    PEER_PROBE_FAILURE_HOLD_SECONDS, PEER_PROBE_REFRESH_SECONDS, PEER_PROBE_STALE_SECONDS, PEER_PROBE_TIMEOUT_SECONDS, PEER_RECHECK_SECONDS,
    PEER_REPASS_SECONDS, PEER_SCOPE_LEFTOVERS_MARK, PEER_SCOPE_LEFTOVERS_SHOWN, PEER_STARTS_PER_PASS, PEER_THREADS_LINE, PEER_TRIP_BUILT_MARKS,
    PEER_TRIP_READ_BACK_MARK, PEER_TRIP_REMOVED_MARK, PEER_UNAVAILABLE_NOTIFY_SECONDS, PREVIOUS_SWAP_RUNNING_EXIT_CODE, PROCESS_LIST_SECONDS,
    PROGRESS_PUSH_LISTED_ATTEMPTS, PROGRESS_PUSH_MINUTES, PROGRESS_PUSH_TAIL_CHARACTERS, PROGRESS_STALL_SECONDS, PRUNE_SERVER_CHECK_SECONDS,
    QUEUE_ATTENTION_MINUTES, QUIET_ATTENTION_MINUTES, QUOTED_WORD_PATH, RECORD_WINDOW, REGISTRY_NO_LIBRARY_PREFIXES, REGISTRY_SERVICE_MARK,
    RESIDENT_PRUNE_SECONDS, RESIDENT_RETENTION_SECONDS, RESIDENT_TASK_TABLES, RESOURCE_CACHE_SECONDS, RESTART_SURVIVING_META_PREFIX,
    RESTART_SURVIVING_STATES, RESTORED_HOST_LISTS, RUNAWAY_MINIMUM_CPU_SECONDS, RUNAWAY_MINIMUM_FRACTION, RUNAWAY_RENOTIFY_SECONDS,
    RUNAWAY_REPORTS_BEFORE_STOP, RUNAWAY_SCAN_SECONDS, RUNAWAY_STOP_GRACE_SECONDS, RUNNING_BOOKED_SUM_TEXT, RUNNING_BOOKING_MARGIN_BREAK_BYTES,
    RUNNING_PEAK_SAMPLE_SECONDS, RUN_HARD_LIMIT_SECONDS, RUN_LIMIT_SECONDS, SCENE_KEEP_AFTER_COLLECT_SECONDS, SELFTEST_PAUSE_SECONDS,
    SELF_CGROUP_PATH, SHELL_COMMAND_NAMES, SHELL_OPTIONS_WITH_VALUE, SHELL_WORD_PATH, SHELL_WORD_PATH_WITH_EXPANSION_TAILS, SIGNAL_EXIT_CODES,
    SIZE_UNITS, SLICE_CAP_VALUE, SLICE_FLOOR_TEXT, SLICE_ROOM_FLOOR_BYTES, SPARED_ACTIVITY, SPARED_CPU_BUSY, SPARED_CPU_UNREAD, SPARED_IO_WAIT,
    STALL_AT_CAP_FRACTION, STALL_CPU_CORES, STALL_GRACE_SECONDS, STALL_SECONDS, START_TIMEOUT_SECONDS, START_UNCERTAIN_GRACE_SECONDS, STARVED_ROUNDS,
    STOP_CHILD_GRACE_SECONDS, STOP_JOIN_SECONDS, SUBMIT_TRACE_SLOW_SECONDS, SUBMIT_TRACE_TEXT, SWAP_SCRATCH_BUSY_EXIT_CODE, SubmitTrace,
    THREADS_OVER_REGISTERED_FACTOR, THREADS_OVER_REGISTERED_ROUNDS, TICK_SECONDS, UNCLAIMED_WARNING_MINUTES, UNIT_CGROUPS_UNREADABLE_FILE,
    UNIT_CHECK_SECONDS, UNIT_STATES_UNREADABLE_FILE, UNIT_STOP_SECONDS, UNIT_UNREADABLE_ATTENTION_SECONDS, UNIT_UNREADABLE_HEALTH_SHOWN,
    UNIT_UNREADABLE_HEALTH_TEXT, UNPOOLED_SWEEP_SUMMARY_MARK, VERDICT_LIBRARY_REGISTRY, VOLATILE_ROOT, WAIT_LIMIT_SECONDS, WHOLE_FRACTION, WHOLE_WORKSPACE_FLAGS,
    WHOLE_WORKSPACE_NEXT_STEP, WHOLE_WORKSPACE_SUBCOMMANDS, WORKSPACE_BLOCKED_EXIT_CODE, WORKSPACE_READ_ONLY_ARGUMENTS, WORKSPACE_WRITERS,
    WORKSPACE_WRITER_FLOOR_BYTES, WORKSPACE_WRITER_INTERRUPTED_OUTCOMES, WORKSPACE_WRITER_NO_RETRY_OUTCOMES, WRITER_PATCH_ENTRY, append_log_lines,
    append_private_line, artifact_size_text, association_id, cancelled_artifact_lines, cpu_reading, directory_bytes, earlier_reading,
    gate_marker_library, gpu_not_run_test, gpu_not_run_text, headline_class, keyed_lock, kind_label, log_tail, nofile_property, outcome_headline,
    own_unit_state, post_to_inbox, push_round_tail, read_json_file, reminder_headline, say, suspect_headline, task_scope_text, task_state_phrase,
    write_json_atomically, write_text_atomically, _GATE_MARKER_LIBRARY_SPECIFICATION,
)
from task_server_store import Store, StoreRefused
from task_server_machines import (
    ActivityWindow, BuildDirectory, HEAD_BLOCKED, HEAD_NOT_STARTED, HEAD_STARTED, HeadVerdict, IoWaitWindow, LocalMachine, MACHINE_ORDER, Machine,
    PassView, PeerMachine, PeerReading, Placement, Resources, Snapshot, SuspectWindow, ThreadReading, activity_text, activity_window_after,
    attention_event, cgroup_processes, descendant_processes, df_available_bytes, gibibytes_text, io_pressure_full_avg300, io_pressure_text,
    next_reminder_seconds, peer_build_root, process_is_zombie, process_run_state, recorded_text, registered_threads, reminder_threshold,
    shell_script_word_indexes, shell_word_paths, signal_name, task_id_ranges, thread_reading_of, unit_processes, unit_states,
)
from task_server_scheduler_lifecycle import SchedulerLifecycle
from task_server_scheduler_notifying import SchedulerNotifying
from task_server_scheduler_building import SchedulerBuilding
from task_server_scheduler_admission import SchedulerAdmission
from task_server_scheduler_starting import SchedulerStarting


class Scheduler(SchedulerLifecycle, SchedulerNotifying, SchedulerBuilding, SchedulerAdmission, SchedulerStarting):
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
        self.finishing = set()   # 收尾此刻在后台线程里做着的尝试记录 id（finish_off_loop）
        self.finishing_lock = threading.Lock()
        self.unit_check_seconds = FAST_UNIT_CHECK_SECONDS if speedy else UNIT_CHECK_SECONDS
        self.lost_grace_seconds = FAST_LOST_GRACE_SECONDS if speedy else LOST_GRACE_SECONDS
        self.unit_unreadable_attention_seconds = FAST_UNIT_UNREADABLE_ATTENTION_SECONDS if speedy else UNIT_UNREADABLE_ATTENTION_SECONDS
        self.cancel_settle_seconds = self.lost_grace_seconds + 2 * self.unit_check_seconds + CANCEL_SETTLE_MARGIN_SECONDS
        self.alerted_child_refusals = set()   # children_refused 已告警过的 (父任务号, 那一句)
        self.start_uncertain_grace_seconds = FAST_START_UNCERTAIN_GRACE_SECONDS if speedy else START_UNCERTAIN_GRACE_SECONDS
        self.scene_keep_seconds = FAST_SCENE_KEEP_AFTER_COLLECT_SECONDS if speedy else SCENE_KEEP_AFTER_COLLECT_SECONDS
        self.build_disk_predictions = {}   # {任务号: 编译目录占盘的预测（字节）}：与内存预测一样每条在排任务只算一次（文件头 ②「编译目录占盘」）
        self.admission_threads_cache = {}   # {任务号: (线程数, 照什么记的)}：CPU 闸里这一条自己要起的记几线程（admission_threads），每条在排任务只算一次
        self.running_threads_cache = {}     # {尝试 id: (线程数, 照什么记的)}：在跑却还没实测到核数的尝试在 CPU 闸里记几线程（unmeasured_threads），每次尝试只算一次
        # 编译目录组（task_server_lib.py 文件头「编译目录」）：拿一个、收尾记留没留着、扫树时挑要删的，都在这把锁里读写 build_slots；
        # 拿到之后到尝试插进库（claim_attempt）之前，那一行记在 build_slot_reservations 里（占着，别的起法不拿它）；扫树删的那几行删之前记在 build_slots_removing 里
        self.build_pool_lock = threading.Lock()
        self.build_slot_reservations = set()
        self.build_slots_removing = set()
        self.build_pool_swept_at = 0.0               # 上一回在后台起扫树的时刻（time.monotonic；loop 至多每 BUILD_POOL_SWEEP_SECONDS 起一回）
        self.build_pool_sweep_lock = threading.Lock()   # 一回扫树拿着它跑到完，还没完时不再起下一回（start_build_pool_sweep）
        self.starved_seconds = FAST_STARVED_SECONDS if speedy else STARVED_ROUNDS * self.tick_seconds
        self.aged_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_AGED_SECONDS", environment) or AGED_WAIT_SECONDS)
        # slice 兜底的下限（slice_floor_problem）：自证的小世界由 lib.SLICE_ROOM_FLOOR_VARIABLE 换（各格写 0：不判；那几格写数照它判）
        floor_override = lib.selftest_override(lib.SLICE_ROOM_FLOOR_VARIABLE, environment)
        self.slice_room_floor = int(floor_override) if floor_override.isdigit() else SLICE_ROOM_FLOOR_BYTES
        self.placement_paused = lib.selftest_override("TASK_SERVER_SELFTEST_PLACEMENT_PAUSED", environment)
        self.placement_refused = lib.selftest_override("TASK_SERVER_SELFTEST_PLACEMENT_REFUSED", environment)
        self.progress_seconds = FAST_PROGRESS_STALL_SECONDS if speedy else PROGRESS_STALL_SECONDS
        self.stall_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_STALL_SECONDS", environment) or STALL_SECONDS)
        self.stall_grace_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS", environment) or STALL_GRACE_SECONDS)
        # {尝试号: (日志字节数, 上一次看到它变的时刻, 这一个 CPU 窗口开头的时刻, 那时的 CPU, 上一个窗口满了为什么没停：SPARED_* 之一，没判过或日志涨了是空串)}（判卡住）
        self.log_growth = {}
        self.log_offsets = {}   # 第二台的尝试：{尝试号: (日志读到哪, 算在涨的字节数, 最近的 usage_usec)}
        self.hard_limit_stops = set()   # 照四小时硬上限判停、正在停的尝试号（stop_over_limit_attempt）
        self.peer_memory = {}   # 第二台的尝试：{尝试号: (最近一行 MULTI_HOST_RUN_PEER_MEMORY 的此刻内存或 None, 读到过的峰值或 None)}（PeerMachine.memory_now）
        self.peer_memory_stat = {}   # 第二台的尝试：{尝试号: MULTI_HOST_RUN_PEER_MEMORY_STAT 里 current_bytes 最大的那一行 {current, anon, file, file_dirty, file_writeback, shmem}}（文件头 ①「内存最高那一刻」）
        self.peer_external_kill = {}   # 第二台的尝试：{尝试号: (来源, 被杀的进程号, 日志原行)}，multi-host-run.sh 命令退非 0 之后打的 MULTI_HOST_RUN_PEER_EXTERNAL_KILL 那一行（external_kill_note 读）
        # 两台的尝试：{尝试号: (标记处数, [点名的用例，至多 GPU_NOT_RUN_TESTS_SHOWN 条])}，log_content 读到的 GPU_NOT_RUN_MARK 记号（gpu_not_run_test 认的整词记号，一行几处记几处；record_gpu_not_run_marks 记进尝试、放掉）
        self.gpu_not_run_marks = {}
        self.peer_io_wait = {}   # 第二台的尝试：{尝试号: PeerMachine.io_blocked 上一次读时并起来的几行 MULTI_HOST_RUN_PEER_IO_WAIT 里最少的 blocked，有一行 read=no 是 None}
        self.peer_threads = {}   # 第二台的尝试：{尝试号: PeerMachine.thread_reading 上一次读时并起来的几行 MULTI_HOST_RUN_PEER_THREADS 里 read=yes 的取最大的 ThreadReading，都是 read=no 是 None}
        # 第二台的尝试：{尝试号: [行…]}，log_content 每一回读到的 MULTI_HOST_RUN_PEER_IO_WAIT（blocked 或 None）与 MULTI_HOST_RUN_PEER_THREADS（ThreadReading 或 None）
        # 接在后面，PeerMachine.io_blocked、thread_reading 读时并起来、清空：两次读之间不论 log_content 被调几回、哪个线程调的，读到的行一行不丢
        self.peer_io_rounds = {}
        self.peer_thread_rounds = {}
        self.thread_overruns = {}   # 线程总数远超登记数（note_threads）：{尝试号: {sample: 上一轮的采样号, streak: 连着超了几轮, flagged: 判出那一轮的 (ThreadReading, 登记数) 或 None}}
        self.io_waits = {}   # 判卡住：{尝试号: IoWaitWindow}（文件头 ①「卡在 IO 上」；日志涨了就删）
        self.activity_windows = {}   # 判卡住：{尝试号: ActivityWindow}（文件头 ①「卡住」的有进展；窗口换了从头记）
        # 日志不涨时的进展读数（note_attempt_progress；文件头 ①「要处置的提醒」的 quiet-log-busy）：上一回读的时刻（monotonic）、上一回读到的写着的文件与大小、
        # 最近一回读到进展的 (时刻（monotonic）, 给人看的那半句)
        self.progress_read_at = {}
        self.progress_files = {}
        self.progress_noted = {}
        self.service_stopped_attempts = {}   # {尝试号: stop_running 对它发停的时刻}：收尾时 signal_note 照它说服务发没发过停的信号（finish_attempt 删）
        self.proc_root = lib.selftest_override("TASK_SERVER_SELFTEST_PROC_ROOT", environment) or "/proc"   # 本机读进程状态的目录（只在自证里换成假的）
        self.cpu_windows = {}   # 判疑似卡住：{尝试号: SuspectWindow}（文件头 ①「疑似卡住」）
        self.suspect_reported = {}   # {尝试号: 报给提交它的会话的那一段的段号}：一段只报一次
        self.suspect_lock = threading.Lock()
        # 要处置的提醒（文件头 ①「要处置的提醒」）：提醒门槛的起点；报到的门槛记进库里任务的行、内存里另留一份（reminded_thresholds）；分片父任务推不动的现状由 drive_sharded_parents 记
        self.queue_attention_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS", environment) or QUEUE_ATTENTION_MINUTES * 60)
        self.quiet_attention_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_QUIET_ATTENTION_SECONDS", environment) or QUIET_ATTENTION_MINUTES * 60)
        self.io_wait_attention_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_IO_WAIT_ATTENTION_SECONDS", environment) or IO_WAIT_ATTENTION_MINUTES * 60)
        self.long_run_attention_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_LONG_RUN_ATTENTION_SECONDS", environment) or LONG_RUN_ATTENTION_SECONDS)
        # {任务号: oversized-budget 那一项}：排队那一刻判出记账不小于某一台整片界的（budget_check），attention_of_record 照它报；收尾时删（forget_prediction），
        # 只在内存里：服务重启后下一轮排队照现算的预测再判一次，报到的门槛在库里（reminded_thresholds），不重发
        self.budget_notices = {}
        # {任务号: booking-beyond-idle-machine 那一项}：排队那一轮判出这一条在每一台上都起不来的（note_idle_short），attention_of_record 照它报；收尾时删（forget_prediction），只在内存里
        self.idle_short_notices = {}
        # {任务号}：排队那一轮对这一条的「在每一台上都起不来」那一判有了结论的（note_idle_short_judged：放得下，或放不下而 idle_short_text 真对每一台判过、不管判出什么）；
        # attention_of_record 在排的 oversized-budget 要等它，免得抢在 booking-beyond-idle-machine 之前；收尾时删（forget_prediction），只在内存里
        self.idle_short_judged = set()
        self.reminded_thresholds = {}   # {(任务号, 种类): 报给提交它的会话的最高提醒门槛}：同一种同一道门槛只报一次；库里那一份（任务行的 reminded_thresholds）读过的也并进来
        self.attention_held = set()    # {(任务号, 种类, 提醒门槛)}：说的是子任务、发消息那一路头一轮先压着的提醒（push_attention；发了或发失败就删）
        self.reminder_histories = {}  # {(任务号, 种类): [{threshold, repush, at, basis, round}…]}：报过的那几回（库里任务行的 reminder_history 另存一份；reminder_history_of 读）
        self.workspace_write_checks = {}   # {任务号: workspace-write-interrupted 那一项或 None}：结了局的改工作区命令只核一次（workspace_write_item）
        self.attention_lock = threading.Lock()
        # 这两张另记进库（RESTART_SURVIVING_STATES 的 shard_advance），换服务后照库接着数（restore_restart_surviving_state）
        self.advance_failures = {}   # {分片父任务号: {"episode": 第几回开始推不动, "why": 最近一次的错, "count": 这一回连着推不动几轮}}：推通了就删
        self.advance_episodes = {}   # {分片父任务号: 推不动开始过几回}：推通之后再推不动是新的一回（提醒门槛加一）
        self.journalctl = lib.selftest_override("TASK_SERVER_JOURNALCTL", environment) or "journalctl"   # 读本机的 earlyoom 与内核日志（文件头 ①「被外部信号杀掉」）
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
        self.prediction_lock = threading.Lock()   # 管 predictions 这几张存预测的表：只在取、存、删的那一下拿着，现算（扫库、读钉值）不在这把锁里
        self.prediction_task_locks = {}   # 每条任务一把锁（keyed_lock 管，放下就删）：排队线程、确认单与弄坏开关 health-judges-head 下 /health 的另判排头同一条不算两遍，别的任务不等它算完
        self.prediction_generation = 0   # 存的预测整表作废（钉值改了）一回涨一：作废之前开算的那一份算完不存（store_prediction）
        self.view_numbers = itertools.count(1)   # PassView 的编号（自证的读数记录里分各轮）
        world = lib.selftest_override("TASK_SERVER_SELFTEST_WORLD", environment)
        self.reads_log = os.path.join(world, "scheduler-reads") if world else ""   # 只在自证里：排队读预测与余量的每一次记一行（selftest_read）
        self.reads_lock = threading.Lock()
        self.repass_soon = False   # 第二台这一轮起满了 PEER_STARTS_PER_PASS 条：主循环隔 PEER_REPASS_SECONDS 再排
        self.waiting_reasons = {}   # 上一轮判的每条排着的任务在等什么：{任务号: 一句}（status 打出来）
        self.queue_positions = {}   # 上一轮排队的先后：{任务号: (排第几, 在排共几条, 前面排满时限的几条)}（status 打出来；全局的，不分会话）
        self.fits_since = {}   # 派活从哪一刻起一直判排头能起却没起成：{任务号: 时刻}（/health 判「判能起却没起成」）
        self.memory_wrapper = lib.selftest_override("TASK_SERVER_MEMORY_WRAPPER", environment) or lib.MEMORY_WRAPPER
        self.peer_runner = lib.selftest_override("TASK_SERVER_PEER_RUNNER", environment) or lib.PEER_RUNNER
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
        # 读不到单元状态的那一段（文件头 ①「读不到单元状态」；unit_unreadable_gone 记，读到了单元状态、收尾时 forget_unit_unreadable 清）
        self.unit_unreadable = {}   # {尝试记录 id: {since: 从哪一刻起（monotonic）连着读不到, task, number, host, unit}}（提醒与 /health 读它）
        self.unit_unreadable_noted = {}   # {尝试记录 id: 这一段已往服务日志打到的最高提醒门槛}
        self.unit_presence = {}   # {尝试记录 id: (lib.PRESENCE_*, 说明, 核完的时刻 monotonic)}：不经 systemd 最近一回核到的（probe_unit_presence 写）
        self.unit_presence_probing = set()   # 此刻在后台核着的尝试记录 id
        self.unit_presence_lock = threading.Lock()
        self.children = []
        self.thread = threading.Thread(target=self.loop, name="task-server-scheduler", daemon=True)
        # NFS 共享（task_server_lib.py 文件头「NFS 共享」）：起来时判过的那一份，前三项由 nfs_loop 每 nfs_recheck_seconds 复核；/health 的 nfs 一栏读它
        self.nfs = nfs_report
        self.nfs_slow_last = None   # 最近一回复查期里连得上的「慢」（{at, slow: [那几句]}）：nfs_loop 记，/health 的 nfs 一栏带 slow_last；只进服务日志与 /health，不推
        self.nfs_lock = threading.Lock()
        self.shared_roots = [os.path.realpath(entry["root"]) for entry in nfs_report.get("roots", [])]   # 第二台照同一绝对路径读写的目录（提交时拒的 ① ②）
        self.mount_problem = ""   # 复核判出第二台没挂上共享根的那一句：这时当第二台此刻用不了（peer_view）
        self.nfs_recheck_seconds = FAST_NFS_RECHECK_SECONDS if speedy else NFS_RECHECK_SECONDS
        self.nfs_thread = threading.Thread(target=self.nfs_loop, name="task-server-nfs-recheck", daemon=True)
        # 第二台用不用得了（note_peer）：/health 的 peer 一栏读它；peer_alerted 是上一条 peer-unavailable 告警写的原因（同一个原因不再写）
        self.peer_status = {"available": True, "why": "", "since": None, "last_run_failure": None}
        self.peer_alerted = ""
        self.peer_status_lock = threading.Lock()
        # 发消息给会话（文件头 ⑥）：一个后台线程；结局记下时 notify_finished 把任务号放进 inbox_pending、叫醒它
        self.inbox_round_seconds = FAST_INBOX_ROUND_SECONDS if speedy else INBOX_ROUND_SECONDS
        self.peer_unavailable_notify_seconds = FAST_PEER_UNAVAILABLE_NOTIFY_SECONDS if speedy else PEER_UNAVAILABLE_NOTIFY_SECONDS
        self.unclaimed_warning_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_UNCLAIMED_WARNING_SECONDS", environment) or UNCLAIMED_WARNING_MINUTES * 60)
        self.confirm_timeout_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_CONFIRM_TIMEOUT_SECONDS", environment) or CONFIRM_TIMEOUT_SECONDS)
        self.run_limit_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS", environment) or RUN_LIMIT_SECONDS)
        self.run_hard_limit_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_RUN_HARD_LIMIT_SECONDS", environment) or RUN_HARD_LIMIT_SECONDS)
        self.anomaly_repush_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_ANOMALY_REPUSH_SECONDS", environment) or ANOMALY_REPUSH_SECONDS)
        self.progress_push_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_PROGRESS_PUSH_SECONDS", environment) or PROGRESS_PUSH_MINUTES * 60)
        self.progress_pushed = {}   # {任务号: 上一回发在跑进度的时刻}（push_progress；任务行的 progress_pushed_at 另存一份，服务重启后取两份里晚的）
        self.unclaimed_repushed = {}   # {任务号: 结局没人取再推到第几回}（repush_unclaimed_outcomes；内存里一份，任务行的 outcome_repushed 另存一份，服务重启后取两份大的、不重推同一回）
        self.inbox_sockets = {}        # 这一轮发消息照会话号找到的收件 socket：{会话号: (路径, 找不到时的那一句)}（inbox_socket_of；每一轮开头清掉）
        self.inbox_pending = set()   # 记下了结局、还没发给会话的任务号
        self.inbox_failed = set()    # 发不出去过的事件（("outcome", 号)、(号, 种类, 提醒门槛)、("suspect", 尝试号, 段号)）：不重试到死，写过告警就不再发
        self.resident_prune_seconds = FAST_RESIDENT_PRUNE_SECONDS if speedy else RESIDENT_PRUNE_SECONDS
        self.resident_pruned_at = 0.0   # forget_settled_entries 上一回清的时刻（time.time）
        self.session_locks = {}   # session_lock 的锁表：{会话号: [锁, 拿着与在等的个数]}
        self.session_locks_guard = threading.Lock()
        self.copy_proof_lock = threading.Lock()   # 提交时拒 ⑨（copy_proof_refusal）与记库在同一把锁里：证红与会编译的命令同时交的两条，后到的看得见先到的
        self.tamper_lock = threading.RLock()   # tampered：同一行几个线程同时判出核不过，只头一个告警、写回
        self.health_notified = {}    # {服务健康的种类: 已发过的会话}：同一样只发一次，好了（这一轮不再查出）就删，再坏再发
        # GPU 服务卡坏（gpu_card_failures）：上一回读到的 [(键, 那一句, 台, 认领)]（lib.gpu_service_report）、读完的时刻、在不在读；写过告警的种类。几样由 gpu_cards_failed_lock 管
        self.gpu_cards_failed = []
        self.gpu_cards_failed_read_at = 0.0
        self.gpu_cards_failed_reading = False
        self.gpu_cards_failed_known = False   # 服务起来之后读完过一回没有（watch_health 照它不在头一回读完之前忘掉 GPU 那几样）
        self.gpu_hosts_read = set()           # 服务起来之后读成过的台（lib.gpu_service_hosts_read；gpu_health_settled 照它不忘掉一直没读成的那一台名下的）
        self.gpu_lines_unparsed = []          # 上一回认不出的卡坏行、判不了的行（lib.gpu_service_report）
        self.gpu_cards_failed_alerted = set()
        self.gpu_cards_failed_lock = threading.Lock()
        self.inbox_lock = threading.Lock()
        self.inbox_event = threading.Event()
        self.inbox_thread = threading.Thread(target=self.inbox_loop, name="task-server-inbox", daemon=True)
        self.runaway_scan_seconds = FAST_RUNAWAY_SCAN_SECONDS if speedy else RUNAWAY_SCAN_SECONDS
        # {(台, 进程号): {reported_at, count, alerted, stopped}}：任务之外的进程空转（watch_runaway_processes），这一轮不在了的忘掉；
        # 另记进库（RESTART_SURVIVING_STATES），换服务后照库接着数
        self.runaway_notified = {}
        self.runaway_renotify_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_RUNAWAY_RENOTIFY_SECONDS", environment) or RUNAWAY_RENOTIFY_SECONDS)
        self.class_durations = (0.0, {})   # (算的时刻, {类: [跑成的用时…]})：class_average_seconds 的缓存，只有排队线程读写
        self.running_peak_samples = {}   # {尝试 id: {first_seen, samples: [(时刻, 累计 CPU 微秒)…]}}：峰值段核数的读数（note_peak_segment），窗口 CPU_MEASURED_AFTER_SECONDS、起点之前留最新的一条
        self.running_peak_cores = {}     # {尝试 id: 到此刻各段在用核数的最大}：note_peak_segment 记，finish_attempt 记 ended 时删
        self.running_latest_cores = {}     # {尝试 id: 最近量满的那一段的在用核数}：note_peak_segment 记，latest_running_cores 读（CPU 闸里在跑的那一条照此刻的记，不照到此刻的最大）
        self.running_latest_runnable = {}  # {尝试 id: 最近一回读到的可跑线程数}：note_threads 记，latest_running_cores 读
        self.running_peak_runnable = {}  # {尝试 id: 到此刻读到过的可跑线程数的最大}：note_threads 记，peak_segment_running_cores、finish_cpu_gate_threads 读，finish_attempt 记 ended 时删
        self.running_usage_lock = threading.Lock()
        # 记账对账（audit_bookings，booking_audit_loop 每 booking_audit_seconds 一轮）：booking_audit_state 记每一项连着几轮不对，booking_drift 是判出来的（/health 的 booking_drift 一栏读），
        # booking_drift_notified 记每一件推给每个会话推到了哪道提醒门槛；三张表都由 booking_drift_lock 管
        self.booking_audit_seconds = BOOKING_AUDIT_SECONDS
        self.booking_audit_state = {}
        self.booking_drift = {"memory": {}}
        self.booking_drift_notified = {}
        self.booking_drift_lock = threading.Lock()
        self.booking_ledger = {}   # {台: 在跑记账的逐条明细}：记账对账每一轮换上（refresh_booking_ledger），/health 的 booking_ledger 一栏读（booking_drift_lock 管）
        self.booking_audit_thread = threading.Thread(target=self.booking_audit_loop, name="task-server-booking-audit", daemon=True)
        self.unit_stopped = set()   # 单元已确认停了（release_stopped_booking）而收尾还没做完的尝试号：booking_attempts 不数它们（CPU 与内存的在跑记账）
        self.runaway_thread = threading.Thread(target=self.runaway_loop, name="task-server-runaway", daemon=True)
        # 第二台的余量（Resources.peer_probe_loop；文件头 ②「第二台的余量不在排队线程里读」）：自证的假余量不连第二台，不起它
        self.peer_probe_thread = threading.Thread(target=self.resources.peer_probe_loop, args=(self.stopping,), name="task-server-peer-probe", daemon=True)
        self.outer_unit = self.nested_outer_unit()   # 套在外层单元里的服务（文件头 ③）：尝试单元照它 BindsTo；真服务是空串
        self.restart_surviving_lock = threading.Lock()   # 换服务不丢的那几份：取内存表、比上次写的、写库在同一把锁里，后改的不被先取的那一份盖掉
        self.restart_surviving_written = {}   # {库里的名字: 上次写进库的正文（canonical_json）}：没变的不重写
        self.restore_restart_surviving_state()

    # ── 记录 ──
    def operation(self, action, task_id, session, process_id=0, note=""):
        if lib.broken("operations-unlogged", self.environment):
            return
        record = {"time": round(time.time(), 3), "action": action, "task": task_id, "session": session, "pid": process_id, "note": note}
        self.store.insert("operations", dict(record))
        with contextlib.suppress(OSError):
            append_private_line(self.configuration.operations_log, lib.canonical_json(record))


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
        inbox: Dict[str, str] = {}   # 会话的收件地址 {socket, token}（文件头 ⑥；task submit 从会话的环境读）
        whole_package_confirmed: str = ""   # 整包确认文件（task submit --whole-package-confirmed；文件头 ②「提交时拒」⑥）
        manual: bool = False   # 人手交（task submit --manual）：Scheduler.manual_refusal 判
        full_gate: bool = False   # 全量门禁标识（task submit --full-gate）：Scheduler.full_gate_refusal 判

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
        """只做库写入试探与排队线程心跳；派活判排头能起却一直没起成、看进展那一句都读排队线程每一轮算好的（文件头「接口」），不现读余量、不起子进程；
        读不到单元状态那一句与 unit_unreadable 栏读对账记下的那一段（Scheduler.unit_unreadable_report，文件头 ①「读不到单元状态」）。
        弄坏开关 unit-unreadable-health-unreported 下不报那一句、unit_unreadable 栏是空表。"""
        writable, problem = store.health_write()
        now = time.time()
        since = now - scheduler.last_tick
        starved = scheduler.starved_problem(now, fresh=True) if lib.broken("health-judges-head", environment) else scheduler.starved_text
        stalled = scheduler.progress_text
        unreadable, unreadable_text = ([], "") if lib.broken("unit-unreadable-health-unreported", environment) else scheduler.unit_unreadable_report()
        problems = ([] if writable else [f"库写不进：{problem}"]) + ([starved] if starved else []) + ([stalled] if stalled else []) + ([unreadable_text] if unreadable_text else [])
        counts = {state: store.count("tasks", state) for state in ("queued", "running")}
        pause = None if lib.broken("pause-health-unreported", environment) else scheduler.pause   # 暂停不让 ok 变 false（文件头 ⑤「暂停」）
        other_running = None if pause is None else scheduler.running_outside_allowed(pause, counts["running"])
        report_allow = not lib.broken("pause-allow-unreported", environment)   # 放行的会话、别的会话在跑几个（文件头 ⑤「暂停」）
        return {"ok": not problems, "problem": "；".join(problems), "database_writable": writable, "scheduler_seconds_since_tick": round(since, 3),
                "tag": store.tag, "pid": os.getpid(), **counts, "nfs": scheduler.nfs_snapshot(), "peer": scheduler.peer_health(), "booking_drift": scheduler.booking_drift_snapshot(), "booking_ledger": scheduler.booking_ledger_snapshot(), "unit_unreadable": unreadable,
                "paused": pause is not None,
                "pause": None if pause is None else {"reason": pause["reason"], "since": pause["since"], "text": scheduler.pause_text(pause, counts["running"], other_running),
                                                     **({"allow_session": pause["allow_session"], "other_running": other_running} if report_allow else {})}}

    @application.post("/tasks")
    async def submit(body: SubmitBody, request: Request):
        # 收到的时刻在事件循环上记（SubmitTrace）：活在线程池里做，排进线程等了多久也记得下；回话时打一行进出日志
        trace = SubmitTrace(scheduler.environment)
        outcome = "出错"
        try:
            reply = await run_in_threadpool(submit_in_thread, body, request, trace)
            outcome = "收下"
            return reply
        except HTTPException as refused:
            outcome = f"拒了（{'、'.join(refused.detail) if isinstance(refused.detail, dict) else '请求不对'}）"
            raise
        except Exception as error:
            outcome = f"出错（{type(error).__name__}）"
            raise
        finally:
            trace.finish(outcome)

    def submit_in_thread(body, request, trace):
        trace.step("进线程")
        process_id = peer_of(request)
        if not body.command:
            raise HTTPException(status_code=400, detail="没给命令")
        if not os.path.isabs(body.cwd) or not os.path.isdir(body.cwd):
            raise HTTPException(status_code=400, detail=f"工作目录 {body.cwd} 不是在的绝对路径")
        carried, refusal = scheduler.submit_refusal(body.command, body.cwd, body.environment, body.whole_package_confirmed, body.full_gate)   # 提交时拒的全部当场拒、不进库（文件头 ②「提交时拒」）
        trace.step("提交时拒")
        if refusal is not None:
            raise HTTPException(status_code=400, detail={refusal[0]: refusal[1]})
        session = session_of(process_id)
        manual_problem = scheduler.manual_refusal(body.manual, session)
        trace.step("认会话")
        if manual_problem is not None:
            raise HTTPException(status_code=400, detail={"placement": manual_problem})
        with scheduler.copy_proof_lock:   # 提交时拒 ⑨：判与记库同一把锁，证红与会编译的命令同时交的两条，后到的看得见先到的
            copy_proof_problem = scheduler.copy_proof_refusal(body.command, body.cwd, carried)
            if copy_proof_problem is not None:
                raise HTTPException(status_code=400, detail={"placement": copy_proof_problem})
            task = scheduler.submit(body.command, body.cwd, carried, session, process_id, {} if body.manual else body.inbox, whole_package=body.whole_package_confirmed,
                                    full_gate=body.full_gate, trace=trace)
        trace.task_id = task["id"]
        reply = {"id": task["id"], "session": task["session"], "state": task.get("state"),
                 "sharded": task.get("sharded_entry", ""), "compiles": task.get("compiles", False), "inbox": bool(task.get("inbox")),
                 "warnings": scheduler.submit_warning_lines(task),   # 提交时警告（文件头 ②「提交时警告」）：客户端打到标准错误
                 "direct_queue": "" if lib.broken("direct-reason-unreported", scheduler.environment) else task.get("direct_queue", "")}   # 没判成长任务、直接进队的依据（文件头 ⑦）：客户端打一行「直接进队：…」
        trace.step("提交时警告")
        if task.get("state") == lib.AWAITING_CONFIRMATION_STATE:   # 交任务的二次确认（文件头 ⑦）：确认单与确认的时限
            reply.update(confirmation=scheduler.confirmation(task), confirm_timeout_seconds=scheduler.confirm_timeout_seconds)
            trace.step("确认单")
        return reply

    class ConfirmBody(BaseModel):
        items: int
        tasks: int

    @application.post("/tasks/{task_id}/confirm")
    def confirm(task_id: int, body: ConfirmBody, request: Request):
        process_id = peer_of(request)
        status, reply = scheduler.confirm(task_id, session_of(process_id), process_id, body.items, body.tasks)
        if status != 200:
            raise HTTPException(status_code=status, detail=reply.get("detail", reply))
        return reply

    @application.post("/tasks/{task_id}/promote")
    def promote(task_id: int, request: Request):
        process_id = peer_of(request)
        status, reply = scheduler.promote(task_id, session_of(process_id), process_id)
        if status != 200:
            raise HTTPException(status_code=status, detail=reply.get("detail", reply))
        return reply

    class LongRunConfirmBody(BaseModel):
        why: str

    @application.post("/tasks/{task_id}/long-run-confirm")
    def confirm_long_run(task_id: int, body: LongRunConfirmBody, request: Request):
        process_id = peer_of(request)
        status, reply = scheduler.confirm_long_run(task_id, session_of(process_id), process_id, body.why)
        if status != 200:
            raise HTTPException(status_code=status, detail=reply.get("detail", reply))
        return reply

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
                found = await run_in_threadpool(store.task, task_id)   # 读库、核签名放进线程池：事件循环上收提交与别的请求，不让一群等结局的轮着读库挡住它
                if found is None:
                    raise HTTPException(status_code=404, detail=f"没有任务 {task_id}")
                task, valid = found
                if not valid or task.get("state") == "finished":
                    if valid and session == task.get("session"):
                        await run_in_threadpool(scheduler.mark_collected, task_id)
                    summary = await run_in_threadpool(scheduler.summary, task_id)
                    reply = dict(summary, finished=True)
                    if valid and not lib.broken("attention-finished-unreported", scheduler.environment):   # 结局有了还没报的提醒（出结局时才判的那几种）：打在理由行前面，退码仍是任务的
                        reply["attention"] = await run_in_threadpool(scheduler.unreported_attention, task_id, session)
                    return reply
                suspect = await run_in_threadpool(scheduler.unreported_suspect, task_id, session)
                if suspect is not None:   # 疑似卡住（文件头 ①「疑似卡住」）：任务照跑，wait 打一行、退出；一段只报一次
                    return {"finished": False, "state": task.get("state"), "suspect": suspect}
                attention = await run_in_threadpool(scheduler.unreported_attention, task_id, session)
                if attention:   # 要处置的提醒（文件头 ①「要处置的提醒」）：任务照旧在排或在跑，wait 打一行、退出；同一种同一道提醒门槛只报一次
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
        multiplexing_problem = lib.enable_ssh_multiplexing(environment, configuration.state_directory)   # 服务起的 ssh 一律复用连接（task_server_lib.py 文件头「主机」）
        if multiplexing_problem:
            say(f"  ! {multiplexing_problem}")
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
    scheduler.inbox_thread.start()   # 发消息给会话、盯服务健康（文件头 ⑥）
    scheduler.runaway_thread.start()   # 任务之外的进程空转（文件头 ⑥）
    scheduler.booking_audit_thread.start()   # 记账对账（文件头 ②「内存」的记账对账）
    if not scheduler.resources.fake_path:
        scheduler.peer_probe_thread.start()   # 第二台的余量（文件头 ②「第二台的余量不在排队线程里读」）
    scheduler.start_build_pool_sweep(periodic=False)   # 起来时扫一遍：树不在了的编译目录组整组删（task_server_lib.py 文件头「编译目录」）
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
        with contextlib.suppress(Exception):   # 复用的主连接逐个关掉（task_server_lib.py 文件头「主机」）；关不成的留到 ControlPersist 到点
            lib.close_ssh_masters(environment, configuration.path)
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
