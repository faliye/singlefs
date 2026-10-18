#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 要的工具与设备登记在 stage-inputs.tsv 本阶段那一行第三列，由 scripts/gate/admission.py gate-preconditions 在阶段里判，没齐判红，不交给 gate.sh 预判
# gate-stage: QEMU 真设备上的新池新建文件、发布 B、第二个实例、发布 D 与抬 F（两块 virtio 盘、设备侧独立录制、漏一道屏障与走页缓存两个对照必须判红）
# gate-fixture: yes
# gate-category: checker-tier 类
# gate-similar: checker-tier-crash-point-replay.sh 它也验 crates 写路的崩溃一致性、也是 checker-tier 类，但在宿主上枚举崩溃状态、判录制流截断重放；这里起 QEMU 虚机在两块 virtio 盘上跑真负载、判设备侧独立录制与程序录制逐项相同，要 QEMU、/dev/kvm 与内核镜像，前提、登记的输入与失败方式都不同
# gate-similar: checker-tier-research-build-and-replay.sh 它的复跑那一格也读 E142 的产物，但比的是纯模型实验的整份输出；这里只从 E142 产物取 path=mkfs 之外那几行段序列与反向链当虚机跑的对照（mkfs 那一行与布局表八登记的值比），判的是真设备上收到的写
# gate-similar: doc-registries.sh 它的 segment-registry 格经同一份 scripts/crash/check-segment-registry.py 读布局表八，判的是登记表与 E142 产物、钉住活代码的用例对不对得上；这里只拿登记表「mkfs 种根」那一行当虚机输出的对照，判的是真设备上 mkfs 发出的步骤
#
# gate-cell: device-streams QEMU 真设备上各档负载跑完，设备侧独立录制与程序录制逐项相同、两个对照红在该红的地方
# 参数解析、每格一个子 shell、汇总与退出码经 lib/stage-cells.sh。
#
# 「程序以为发了什么」与「盘上实际收到了什么」由两条不共享代码的路比：程序那一条是被测程序自己的录制器（宿主上同参数重跑，同字节）；
# 盘那一条是 QEMU 的 blklogwrites 过滤节点在来宾之外按 dm-log-writes 格式记下的每个写（带数据）与每个 FLUSH。
# 不声明 gate-covers：「最终判据」要真实负载 + 崩溃注入 + checker 全绿，这一道没有崩溃注入（欠在 C6（块层语义假设写错））。
#
# 用法：bash .claude/gate.d/checker-tier-qemu-device-streams.sh [选项] [项目根]；不带选项时六个模式里键对不上的那几档并行各起一台虚机。
#   --list                  逐行打格名与判什么
#   --check device-streams  只跑这一格；格名写错退 2
#   --list-items            逐行打六个模式与各自判什么，不编译、不起虚机
#   --item <模式>           只跑点名的模式（给几次取并集；写错退 2、列出可点的模式）：每档照键查判定库，判出的照键写库
#   认不出的选项退 2。
# 读的环境变量：VM_LOG_MB（每块盘的设备侧日志 MiB，默认 4096）；GATE_QEMU_DEVICE_STREAMS_KEEP_WORK=1（判绿也留现场，重录样本时拷四件套用）；
#   SINGLEFS_GATE_FULL=1 不读判定库、全部现跑、照键写回；GATE_QEMU_VERDICT_LIBRARY 盖过判定库的位置。
#
# 复用：每档一条判定，键是宿主与来宾两侧真走到的函数单元、QEMU 版本、内核镜像与 initramfs 的内容摘要、工具链；键对得上的档复用，对不上的现跑；
#   `--item <模式>` 只跑点名的档，判出的照键写库；`SINGLEFS_GATE_FULL=1` 不读库、全部现跑、照键写回。没有整道标记；
#   stage-inputs.tsv 本阶段那一行只登记三个前提与入口，不进任何键。键的组成（逐样进 scripts/crash/lib_verdict_library.py 的组成表，单元键照 scripts/gate/lib_function_units.py 的切法）：
#     宿主脚本单元      scripts/devices/vm-bench.sh 脚本体起的 shell 单元闭包（run_one 与它调的函数，/init 的 here-doc 在 stage_initramfs_root 这个单元里），
#                       连同它点名的外部程序：busybox（sha256 与 --help 第一行）、qemu-system-x86_64（补好的那一份的 sha256 与 --version 第一行）
#     来宾程序单元      按档一把：new_pool_file_creation_on_device 的 main 与这一档的入口 run_the_<档，- 换成 _>_mode 起的 Rust 单元闭包，
#                       去掉只有别的档才走到的单元（别的档入口闭包里有、这一档入口闭包里没有的；guest_keys_by_mode），main 自己的参数解析与分派进每一档；
#                       工具链行（cargo -V、rustc -V、--release --target x86_64-unknown-linux-gnu +crt-static）与 Cargo.lock 里 singlefs-checker-tier 传递引到的外部包。
#                       改一档才走到的函数只作废那一档；管不到的：main 自己点到、又只被别的档入口点到的单元，这一档的键不收（闭包按入口收，切不开 main 里走哪条路）
#     宿主检查程序单元  new_pool_file_creation_device_log_check 的 main 起的 Rust 单元闭包（同上，dev 档）
#     判法单元指纹      本阶段 judge_mode_outputs 与 judge_mode_device_log 的单元闭包（按档查表的 cold_root_of、required_lines_of、forbidden_lines_of、
#                       host_rerun_windows_of、mode_parameters_of 不进，它们这一档的取值进「这一档的判据」与「档位参数」，改一档的表只作废那一档）
#     内核镜像          vm-bench.sh 选中的那份镜像的字节 sha256（不按路径）
#     QEMU              补好的 qemu-system-x86_64 与 qemu-io 的 sha256、--version 第一行、补丁 scripts/devices/qemu/blklogwrites-linearize-over-iov-max.patch 的内容
#     initramfs 清单    vm-bench.sh --initramfs-manifest 这一档的根的清单（只取它标准输出里 entry<TAB> 开头的条目：路径、种类、权限、内容 sha256；
#                       /bench 那一行的内容换成占位，被测二进制由来宾程序单元管），不取 cpio.gz 的哈希（cpio 头里有 mtime）；
#                       跑完核 run_one 打包之前打的那一份与它相同（record_tiers_after_run），不同不记、打头几行差异
#     E142 对照行       E142 产物里这一道真读的那几行（path=mkfs 之外的 name=segments 行、name=root_record 的 back_chain）与布局表八登记的 mkfs 那一行
#     这一档的判据      几张按档查表的函数在这一档交的取值
#     档位参数          /bench 收到的参数串（档名就是它）、mode_parameters_of 交的盘参数、VM_LOG_MB 与调用方环境里会改虚机行为的几个 VM_ 变量
#   判定库在被判那棵树的 git common-dir 的 singlefs-qemu-verdicts/（种类 qemu-tier，一档一项 qemu:<模式>），跨轮留着；判绿的档当趟记，红了整趟不记。
#   每档一行「· <模式>：绿，复用（…）」，现判的原因（变了的单元）写在成功句「按档：现判 N、复用 M（现判的原因：…）」里。
#
# 样本 fixtures/checker-tier-qemu-device-streams.sh/{red,direct-divergence-red,e142-row-missing-red,e142-row-duplicate-red,mkfs-segments-red,segments-step-red,green}（stage-selftest.sh 跑，各放 .gate-cells 点名 device-streams）：
# 样本根放 .qemu-prerecorded，按 <模式>/out.txt、vm-exit、check.txt、check-exit 摆某一轮真跑的原样输出，本阶段不起虚机、不编译，
# 用同一段判定代码判（只供测试的开关：.claude/rules/fs-design.md 五条硬要求第 2 条）。预录档全过这一格退 77：退 0 与真起过虚机的一档在汇总里分不出。
# 走到段序列那一步的样本另放 .claude/kb/layout/01-first-txn.md：只有「八、」标题与登记表「mkfs 种根」一行的合成副本，登记值写那一趟录样本时的值。
# 键样本 fixtures/checker-tier-qemu-device-streams.sh/tier-key-samples-*（.qemu-prerecorded 之外另放 .qemu-key-samples/：environment.tsv 写内核镜像的路径、
#   busybox、qemu-system-x86_64 的 sha256 与版本行、工具链行，initramfs-manifest.tsv 是一份清单，mode-parameters.tsv 是每档的盘参数，change.sh 是第二趟之前施加的改动；
#   setup.sh 现造一棵小树：vm-bench.sh、来宾与宿主检查两个 bin、E142 产物与布局表八）：只判键、不判输出。判定库放这一趟的临时目录，
#   头一趟六档照绿记下，跑 change.sh，第二趟逐档 lookup，打「· <模式>：复用（…）| 现判（<原因>：<变了的单元>）」与「· 按档：现判 N、复用 M」，退 77。几份：
#   init-heredoc（改 run_one 里 /init 的 here-doc，六档都作废）、one-tier-parameters（只改 raise-rollback-floor 一档的盘参数，只那一档作废）、
#   kernel-byte（内核镜像换一个字节，六档都作废）、e142-unread-line（E142 产物里加一行这一道不读的，一档都不作废）、
#   guest-function（改来宾程序里只有 raise-rollback-floor 的入口走到的那个函数，只那一档作废）、
#   guest-dispatch（改来宾程序 main 的参数解析，六档都作废：main 自己的单元进每一档）。
#   另两份不摆 initramfs-manifest.tsv，摆真跑那一条路读的原样（listings/<档>.txt 是 vm-bench.sh --initramfs-manifest 的标准输出，runs/<档>/initramfs.manifest
#   是 run_one 打包之前打的清单），头一趟照真跑那一条路核过清单再记（record_tiers_after_run），change.sh 什么都不改：
#   manifest-recorded（六档清单相同，头一趟记下 6 档，第二趟「现判 0、复用 6」）、manifest-differs（second-instance 跑的时候 /init 变了，那一档不记，第二趟「现判 1、复用 5」）。
#
# 段序列分两处比：path=mkfs 那一行与布局表八（.claude/kb/layout/01-first-txn.md「八、」）「mkfs 种根」那一行登记的段序列、次操作、崩溃状态数、种类串比，
#   登记值经 scripts/crash/check-segment-registry.py --print-mkfs-segments-line 现取（同一份解析器，这里不抄第二份）；那一行的权威是活代码
#   （crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs 的 recorded_stream_matches_the_registered_mkfs_segment_sequence 钉住），
#   E142 的装置是独立手写模型、清零步数与活代码不同，不拿它比这一行。其余四行（instance_acquisition、warm_up、transaction、post_mkfs_stream）
#   照旧与 E142 产物的 name=segments 行逐字比，两边都先滤掉 path=mkfs 那一行。
#
# 弄坏开关（只给样本证红用）：GATE_QEMU_DEVICE_STREAMS_BREAK=direct-divergence-unread 让 direct 那一档只看退出码与宿主读回、不逐盘读 divergence=none 那一行，
#   direct-divergence-red（direct 的 check.txt 里盘 1 那一行改成 device=2、退出码仍是 0）判绿退 77、expect 里那几条 want 找不到。
#   GATE_QEMU_DEVICE_STREAMS_BREAK=e142-all-rows 让取 E142 产物的那一步退回「取复跑表里全部 E142 行的第 4 列」、不数行数：
#   green（复跑表里两行 E142，真仓现在的形态）取到两个文件名，红在「从 replay.sh 的 @driver_e142 那一行解析不出产物文件」、不是预期的退 77，expect 里的 want 对不上；
#   e142-row-missing-red（@driver_e142 那一行缺了）取到别的行的文件名，过了这一处，红不在「第 2 列是 @driver_e142 的 E142 行取到 0 行」。
#   GATE_QEMU_DEVICE_STREAMS_BREAK=mkfs-from-e142 让 path=mkfs 那一行退回与 E142 产物的 path=mkfs 行比、不读布局表八的登记值：
#   mkfs-segments-red（样本登记行的段序列 12+1+1+1+4 改成 14+1+1+1+4，虚机输出与 E142 产物都是 12+1+1+1+4）判绿退 77，
#   红不在「mkfs 段序列与登记表不一致」，expect 里的 want 对不上。
#   GATE_QEMU_DEVICE_STREAMS_BREAK=segments-diff-skipped 让虚机输出与 E142 产物的 name=segments 行（path=mkfs 之外那几行）逐字比对那一步跳过：
#   segments-step-red（只摆 direct 一档，direct/out.txt 里 path=transaction 那一行的段序列 24+2+1+2 改成 23+2+1+2，E142 产物与登记行都不动）判绿退 77，
#   红不在「段序列（path=mkfs 之外那几行）与 E142 产物的 name=segments 行不一致」，expect 里的 want 对不上。
#   GATE_QEMU_DEVICE_STREAMS_BREAK=tier-key-is-the-whole-stage 让每档的键退回整道一把（这一档的判据、档位参数、initramfs 清单换成六档合起来的一份）：
#   tier-key-samples-one-tier-parameters（只改 raise-rollback-floor 一档的盘参数）里 file-overwrite 那一档也报现判，expect 里「· file-overwrite：复用」找不到。
#   GATE_QEMU_DEVICE_STREAMS_BREAK=guest-key-from-main 让来宾程序单元退回六档共用 main 起的一把：tier-key-samples-guest-function（只改 raise-rollback-floor 的入口
#   走到的函数）里 direct 那一档也报现判，expect 里「· direct：复用」「· 按档：现判 1、复用 5」找不到。
#   GATE_QEMU_DEVICE_STREAMS_BREAK=guest-key-without-main 让来宾程序单元只从这一档的入口起算、不收 main：tier-key-samples-guest-dispatch（只改 main 的参数解析）
#   六档都报复用，expect 里「· direct：现判」「· 按档：现判 6、复用 0」找不到。
#   GATE_QEMU_DEVICE_STREAMS_BREAK=manifest-with-listing-lines 让开跑前的清单退回改前的取法（vm-bench.sh 标准输出里抬头那几行留在清单里）：
#   tier-key-samples-manifest-recorded 头一趟六档都报「清单不同……没记」，第二趟「现判 6、复用 0」，expect 里「· 头一趟记下 6 档」找不到。
#   GATE_QEMU_DEVICE_STREAMS_BREAK=manifest-check-skipped 让跑完不核清单、照记：tier-key-samples-manifest-differs 六档都记下、second-instance 也复用，
#   expect 里「· second-instance：现判（无记录）」「· 按档：现判 1、复用 5」找不到。
#   GATE_QEMU_DEVICE_STREAMS_BREAK=reuse-lookup-skipped 让每一档一律不读库：tier-key-samples-three-questions 判红（复用三问 ① 与复用哨兵都红），expect 里的 exit=77 对不上。
# 复用三问与复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「一」「二」）：真跑每一趟六档的组成表写好之后、查库之前抽哨兵（候选按键文件查，与 lookup 分开），
#   抽中强制现判的那一档不读库、照现跑；哨兵被现判、强制现判那一档判红，这一格红。SINGLEFS_GATE_FULL=1 不抽。
#   键样本 tier-key-samples-three-questions（.qemu-key-samples/ 另放 unrelated.sh 与 affected）：头一趟六档照绿记下，同一份输入、unrelated.sh（E142 产物末尾加一行这一道不读的）
#   之后、change.sh（改只有 raise-rollback-floor 走到的 publish_the_fourth_version）之后各查一趟（开着哨兵，现判的照绿记），照三问判、打规范输出，退 77；
#   登记表 .claude/gate.d/verdict-library-scripts.tsv 里这一道的自证命令经 scripts/gate/lib_reuse_checks.py sample 起这一份样本。
#
# 准入：三个前提（QEMU、/dev/kvm、内核镜像）登记在 .claude/gate.d/stage-inputs.tsv 本阶段那一行第三列，判法在 scripts/gate/admission.py gate-preconditions，没齐判红。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_REPOSITORY="$(cd "$(dirname "$0")/../.." && pwd)"
ADMISSION_MODULE="$STAGE_REPOSITORY/scripts/gate/admission.py"
source "$(dirname "${BASH_SOURCE[0]}")/lib/stage-cells.sh"
stage_cell device-streams cell_device_streams "QEMU 真设备上各档负载跑完，设备侧独立录制与程序录制逐项相同、两个对照红在该红的地方" \
  "照上面那句「怎么办」改；单跑一档：bash .claude/gate.d/checker-tier-qemu-device-streams.sh --item <模式>（模式用 --list-items 看）"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

fail() { echo "  ✗ $1"; echo "     → 怎么办：$2"; exit 1; }

MODES=(direct skip-new-pool-file-creation-barrier page-cache file-overwrite second-instance raise-rollback-floor)

list_items=0; root_argument=""; named_modes=()
while (($#)); do
  case "$1" in
    --list-items) list_items=1; shift ;;
    --item)
      if (($# < 2)) || [[ -z "$2" || "$2" == -* ]]; then
        echo "  ✗ --item 后面缺模式名"
        echo "     → 怎么办：写成 --item <模式>（例 --item direct），几档就给几次；可点的模式：${MODES[*]}"
        exit 2
      fi
      named_modes+=("$2"); shift 2 ;;
    --item=*) named_modes+=("${1#--item=}"); shift ;;
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check device-streams、--list-items、--item <模式> 与一个项目根。"
      exit 2 ;;
    *) root_argument="$1"; shift ;;
  esac
done
if ((list_items)); then
  printf '%s\t%s\n' \
    direct "O_DIRECT 真写路：设备侧日志逐项等于程序的录制流，冷重开读回文件，宿主从盘镜像再读回一次" \
    skip-new-pool-file-creation-barrier "对照：盘 0 漏掉「单元 → journal 记录」那道屏障，设备侧比对必须红在盘 0" \
    page-cache "对照：读写走页缓存，设备侧比对必须判红" \
    file-overwrite "新池新建文件之后覆盖写一次（发布 B），冷重开择 (1, 4)" \
    second-instance "发布 B 之后冷重开、可写挂载再发布 C，冷重开择 (2, 8)" \
    raise-rollback-floor "second-instance 走完再发布 D、把 F 抬到 3，冷重开择 (2, 11)"
  exit 0
fi
if ((${#named_modes[@]})); then
  unknown_modes=()
  for named in "${named_modes[@]}"; do
    found=0
    for mode in "${MODES[@]}"; do [[ "$mode" == "$named" ]] && found=1; done
    ((found)) || unknown_modes+=("$named")
  done
  if ((${#unknown_modes[@]})); then
    echo "  ✗ --item 点名的模式不认得：${unknown_modes[*]}"
    echo "     → 怎么办：模式照 bash .claude/gate.d/checker-tier-qemu-device-streams.sh --list-items 列的写：${MODES[*]}"
    exit 2
  fi
  # 按 MODES 的次序只留点名的那几档（判定循环与汇总都按 MODES 走）
  kept_modes=()
  for mode in "${MODES[@]}"; do
    for named in "${named_modes[@]}"; do [[ "$mode" == "$named" ]] && { kept_modes+=("$mode"); break; }; done
  done
  MODES=("${kept_modes[@]}")
fi
ROOT="${root_argument:-$STAGE_REPOSITORY}"
cd "$ROOT" 2>/dev/null || exit 2

# 冷重开之后应当择到的根。前三个模式停在新池新建文件（实例 1、txg 3）；发布 B 走到 txg 4；
# 第二个实例再走一遍「取号 → 写行 → 暖机 ×2 → 发布 C」，落在实例 2 的 txg 8；
# 抬 F 那一档接着发布 D（txg 9）、抬 F 推两次空发布（txg 10、11），落在实例 2 的 txg 11。
cold_root_of() {
  case "$1" in
    direct|skip-new-pool-file-creation-barrier|page-cache) printf '1:3' ;;
    file-overwrite)                               printf '1:4' ;;
    second-instance)                                  printf '2:8' ;;
    raise-rollback-floor)                             printf '2:11' ;;
    *)                                                printf '' ;;
  esac
}

# 这个模式的输出里必须逐条出现的行，一行一条基本正则。六个模式共有的那几条（段序列、事务计数、
# 冷重开、块层 FLUSH 数）在判定循环里另查，这里只列各模式独有的——没有这几条，后加的三档就等于没跑。
required_lines_of() {
  case "$1" in
    file-overwrite|second-instance|raise-rollback-floor)
      # file_overwrite 单元写 24：kb layout/01-first-txn.md「根槽写路径的段序列登记表」一节里覆盖写固定脚本流那一句「A、B、C 各 24」（来宾单测不钉 B 的段序列）
      cat <<'PATTERNS'
name=file_overwrite root_txg=4 transaction=2 released=
name=file_overwrite .*segments=24+2+1+2 closed_form=
name=publish_writes publish=file_overwrite txg=4 write_calls=
name=publish_writes_against_device window=file_overwrite publishes=1 .*matches=true
PATTERNS
      ;;
    *) : ;;
  esac
  case "$1" in
    second-instance|raise-rollback-floor)
      # third_transaction 单元写 28：来宾单测 new_pool_file_creation_on_device.rs 的 second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches 钉的值
      cat <<'PATTERNS'
name=writable_mount instance=2 chosen_root=1:4 rows_written=1 row_publish_root=2:5 warm_up_txgs=6,7 nanoseconds=
name=publish_writes publish=instance_row txg=5 write_calls=
name=publish_writes publish=warm_up txg=6 write_calls=
name=publish_writes publish=warm_up txg=7 write_calls=
name=publish_writes_against_device window=writable_mount publishes=3 .*matches=true
name=third_transaction root_txg=8 transaction=1 released=
name=third_transaction .*segments=28+2+1+2 closed_form=
name=publish_writes publish=third_transaction txg=8 write_calls=
name=publish_writes_against_device window=third_transaction publishes=1 .*matches=true
PATTERNS
      ;;
    *) : ;;
  esac
  case "$1" in
    raise-rollback-floor)
      # fourth_transaction 单元写 28：来宾单测 new_pool_file_creation_on_device.rs 的 raise_rollback_floor_mode_publishes_the_fourth_version_raises_the_floor_to_its_ceiling_above_zero_and_every_window_matches_the_device_layer_count 钉的值
      # raise_rollback_floor：kb layout/01-first-txn.md「根槽写路径的段序列登记表」一节里覆盖写固定脚本流那一句「抬 F 与卸载开头先写的系统配置与上一次的轮换各自成段（2、2）」「抬 F 与卸载推的空发布各 16」，D 的轮换算在 D 自己的窗口里
      cat <<'PATTERNS'
name=fourth_transaction root_txg=9 transaction=2 released=
name=fourth_transaction .*segments=28+2+1+2 closed_form=
name=publish_writes publish=fourth_transaction txg=9 write_calls=
name=publish_writes_against_device window=fourth_transaction publishes=1 .*matches=true
name=raise_rollback_floor floor_before=0 requested_floor=3 ceiling=3 publishes=2 root_txgs=10,11 
name=recover_cold_rollback_floor chosen_root=2:11 rollback_floor_on_disk=3$
name=raise_rollback_floor .*segments=2+16+2+1+2+16+2+1+2 closed_form=
name=publish_writes publish=raise_rollback_floor txg=10 write_calls=
name=publish_writes publish=raise_rollback_floor txg=11 write_calls=
name=publish_writes_against_device window=raise_rollback_floor publishes=2 .*matches=true
PATTERNS
      ;;
    *) : ;;
  esac
}

# 这个模式的输出里不许出现的行：模式参数真的换掉了负载，而不是每一档都跑同一条路。
# 少了这一半，把 MODES 里的新模式换成 `direct` 也照样全绿。
forbidden_lines_of() {
  case "$1" in
    direct|skip-new-pool-file-creation-barrier|page-cache)
      printf '%s\n' 'name=file_overwrite ' 'name=writable_mount ' 'name=third_transaction ' 'name=fourth_transaction ' 'name=raise_rollback_floor ' ;;
    file-overwrite)
      printf '%s\n' 'name=writable_mount ' 'name=third_transaction ' 'name=fourth_transaction ' 'name=raise_rollback_floor ' ;;
    second-instance)
      printf '%s\n' 'name=fourth_transaction ' 'name=raise_rollback_floor ' ;;
    *) : ;;
  esac
}

# 宿主检查照这个模式重跑的段（它的 `name=host_rerun` 行里 windows= 那一串，段名与 new_pool_file_creation_device_log_check.rs
# 的 ProgramWindow::name 相同）：新池新建文件之后那几次发布的写有没有进逐项比对，以这一行为准。只登记新池新建文件之后还有发布的三档。
host_rerun_windows_of() {
  case "$1" in
    file-overwrite)   printf 'mkfs,instance_acquisition,warm_up,new_pool_file_creation,file_overwrite' ;;
    second-instance)      printf 'mkfs,instance_acquisition,warm_up,new_pool_file_creation,file_overwrite,reopen_and_writable_mount,third_transaction' ;;
    raise-rollback-floor) printf 'mkfs,instance_acquisition,warm_up,new_pool_file_creation,file_overwrite,reopen_and_writable_mount,third_transaction,fourth_transaction,raise_rollback_floor' ;;
    *)                    printf '' ;;
  esac
}
# 每一档交给 vm-bench.sh 的盘与镜像参数（档的内容进键，档名不进）。键样本（.qemu-key-samples/mode-parameters.tsv）换成样本里那一张表。
mode_parameters_of() {
  if [[ -n "${QEMU_MODE_PARAMETERS_TABLE:-}" ]]; then
    awk -F'\t' -v mode="$1" '$1 == mode { print $2 }' "$QEMU_MODE_PARAMETERS_TABLE"
    return 0
  fi
  case "$1" in
    *) printf 'VM_DISKS=2 VM_DISK_MB=4096' ;;
  esac
}

# 每一档的判法：虚机输出与宿主检查的逐项判据（判法单元指纹取这两个函数的单元闭包，几张按档查表的函数不进、它们这一档的取值另进「这一档的判据」）。
# 这两个函数在这一格的子 shell 里调：fail 与 exit 只结束这一格；checks 累加判了几项。
judge_mode_outputs() { # judge_mode_outputs <档>
  local mode="$1" out="$work/$1/out.txt" vm_mkfs_segments_lines segments_diff cold_root pattern device_calls_lines line barriers fua flushes
  if [[ "$(cat "$work/$mode/vm-exit")" != 0 ]]; then
    tail -15 "$out" | sed 's/^/        /'
    fail "虚机跑 $mode 没过（vm-bench.sh 的退出码或条数闸）" "单跑：$(mode_parameters_of "$mode") VM_LOG_MB=$QEMU_DEVICE_LOG_MEGABYTES VM_BLKLOGWRITES_DIR=<目录> bash scripts/devices/vm-bench.sh <静态二进制> $mode"
  fi
  checks=$((checks + 1))
  # path=mkfs 那一行：恰好一行，与登记值逐字相同（段序列、次操作、崩溃状态数、种类串四样都在这一行里）
  vm_mkfs_segments_lines="$(grep -ao 'E7RESULT name=segments path=mkfs .*' "$out" | tr -d '\r')"
  if [[ "$vm_mkfs_segments_lines" != "$EXPECTED_MKFS_SEGMENTS_LINE" ]]; then
    fail "虚机跑 $mode 的 mkfs 段序列与登记表不一致（虚机：${vm_mkfs_segments_lines:-一行 path=mkfs 都没有}；${EXPECTED_MKFS_SEGMENTS_SOURCE}：$EXPECTED_MKFS_SEGMENTS_LINE）" \
         "登记表那一行由 crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs 的 recorded_stream_matches_the_registered_mkfs_segment_sequence 钉在活代码上：虚机里的 mkfs 发出的步骤与它不同，先查虚机二进制编的是不是这份 crates/ 与真设备写路差在哪一段；登记表抄错了，照那条用例改 .claude/kb/layout/01-first-txn.md「八、」那一行。"
  fi
  checks=$((checks + 1))
  # 其余几行与 E142 产物逐字比：两边都先滤掉 path=mkfs 那一行（它在上一条与登记表比过）
  if [[ "${GATE_QEMU_DEVICE_STREAMS_BREAK:-}" != segments-diff-skipped ]] \
     && ! segments_diff="$(diff <(grep -ao 'E7RESULT name=segments .*' "$PRODUCT" | tr -d '\r' | grep -v '^E7RESULT name=segments path=mkfs ') \
                                <(grep -ao 'E7RESULT name=segments .*' "$out" | tr -d '\r' | grep -v '^E7RESULT name=segments path=mkfs '))"; then
    printf '%s\n' "$segments_diff" | sed 's/^/        /'   # 「<」是 E142 产物的行，「>」是虚机输出的行：先看是哪一档哪一行差了
    fail "虚机跑 $mode 的段序列（path=mkfs 之外那几行）与 E142 产物的 name=segments 行不一致" "对照 $PRODUCT 与虚机输出 path=mkfs 之外的 name=segments 行（上面「<」是产物、「>」是虚机输出的差异行）；真设备上的写路发出的步骤与装置不同，先查是哪一段。"
  fi
  checks=$((checks + 1))
  cold_root="$(cold_root_of "$mode")"
  [[ -n "$cold_root" ]] || fail "模式 $mode 没有登记冷重开该择到的根" "在 cold_root_of 里给它补一行；新加模式不补这一行，冷重开那一条就恒不判。"
  grep -aq "name=recover_cold outcome=file_read root=$cold_root content_matches=true" "$out" \
    || fail "虚机跑 $mode 冷重开之后没按 ($cold_root) 读回文件" "看 $mode 的 name=recover_cold 行与来宾的 stderr（vm-bench.sh 保留现场用 VM_KEEP=1）；根对不上先查这一档该走到哪一次发布。"
  checks=$((checks + 1))
  grep -aq "name=transaction key_order_mismatches=0 root_txg=3 back_chain=$EXPECTED_BACK_CHAIN" "$out" \
    || fail "虚机跑 $mode 的新池新建文件计数或反向链与产物不符" "产物 name=root_record 行的 back_chain 是 $EXPECTED_BACK_CHAIN；两个运行时计数要是 0。"
  checks=$((checks + 1))
  while IFS= read -r pattern; do
    [[ -n "$pattern" ]] || continue
    grep -aq "$pattern" "$out" || fail "虚机跑 $mode 的输出里没有「$pattern」" "这一档该跑的那次发布没跑、或者结果行改了字段名；先看 $mode 的 out.txt 有哪些 name= 行，再对 crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs 里这一档打的行。"
    checks=$((checks + 1))
  done < <(required_lines_of "$mode")
  while IFS= read -r pattern; do
    [[ -n "$pattern" ]] || continue
    grep -aq "$pattern" "$out" && fail "虚机跑 $mode 的输出里出现了不该有的「$pattern」" "这一档不该跑那次发布：确认送进虚机的模式参数就是 $mode，别的模式的负载跑进来了这一档的判据就全落空。"
    checks=$((checks + 1))
  done < <(forbidden_lines_of "$mode")
  # 来宾块层的 FLUSH 数 = 程序真正交给设备的屏障 + FUA 写（来宾内核是第三条独立的路）。
  # 一行 device_calls 都没有时循环一圈不转，这一条判据就没判：按红记，不按「没有对不上的」放过。
  device_calls_lines=0
  while IFS= read -r line; do
    device_calls_lines=$((device_calls_lines + 1))
    barriers="$(sed -n 's/.* barriers=\([0-9]*\) .*/\1/p' <<<"$line")"
    fua="$(sed -n 's/.* force_unit_access_writes=\([0-9]*\) .*/\1/p' <<<"$line")"
    flushes="$(sed -n 's/.* flush_ios=\([0-9NA]*\).*/\1/p' <<<"$line")"
    [[ "$flushes" =~ ^[0-9]+$ && "$flushes" == "$((barriers + fua))" ]] \
      || fail "虚机跑 $mode 的来宾块层 FLUSH 数（$flushes）不等于屏障 + FUA 写（$barriers + $fua）" "看 name=device_calls 行；读不到（NA）也算红——读不到不等于读到 0。"
    checks=$((checks + 1))
  done < <(grep -ao 'E7RESULT name=device_calls .*' "$out" | tr -d '\r')
  ((device_calls_lines > 0)) \
    || fail "虚机跑 $mode 的输出里一行 name=device_calls 都没有：来宾块层的 FLUSH 数没法与屏障 + FUA 写对账" "看 $mode 的 out.txt：虚机二进制没打这一行、或者行名改了；读不到不等于对得上。"
  return 0
}

verdict() { cat "$work/$1/check-exit"; }
# 宿主检查退出 0 的档，两块盘各要有一行 divergence_window=none divergence=none 的 name=device_log：退出码与结果行两处都判，
# 只看退出码的话宿主检查漏比一块盘、或那块盘的行没打出来，都照样判绿
require_each_device_without_divergence() {
  local mode="$1" device
  for device in 0 1; do
    grep -q "name=device_log device=$device .* divergence_window=none divergence=none" "$work/$mode/check.txt" \
      || { sed 's/^/        /' "$work/$mode/check.txt"
           fail "$mode 盘 $device：宿主检查退出 0，却没有这块盘 divergence=none 的 name=device_log 行" "退出码与结果行对不上，先查 crates/singlefs-checker-tier/src/bin/new_pool_file_creation_device_log_check.rs 的 main 与 compare_one_device。"; }
    checks=$((checks + 1))
  done
}
judge_mode_device_log() { # judge_mode_device_log <档>：设备侧日志的判据，对得上的档记进 matched_device_log_modes，两个对照记进 control_modes
  local mode="$1" host_rerun_windows
  case "$mode" in
    direct)
      [[ "$(verdict direct)" == 0 ]] || { sed 's/^/        /' "$work/direct/check.txt"
        fail "direct：设备侧日志与程序的录制流对不上" "上面 divergence= 指着第一处不一致的下标、程序以为的与盘上收到的；先判是写路的错还是比对口径的错。"; }
      grep -q 'name=host_recover outcome=file_read root=1:3 content_matches=true' "$work/direct/check.txt" \
        || fail "direct：宿主从虚机写出的盘镜像上读不回文件" "看 $work/direct/check.txt 的 name=host_recover 行。"
      checks=$((checks + 2))
      [[ "${GATE_QEMU_DEVICE_STREAMS_BREAK:-}" == direct-divergence-unread ]] || require_each_device_without_divergence direct
      matched_device_log_modes+=("$mode")
      ;;
    skip-new-pool-file-creation-barrier)
      [[ "$(verdict skip-new-pool-file-creation-barrier)" == 1 ]] && grep -q 'name=device_log device=0 .*divergence=at=' "$work/skip-new-pool-file-creation-barrier/check.txt" \
        && grep -q 'name=device_log device=1 .*divergence=none' "$work/skip-new-pool-file-creation-barrier/check.txt" \
        || { sed 's/^/        /' "$work/skip-new-pool-file-creation-barrier/check.txt"
             fail "对照 skip-new-pool-file-creation-barrier 没有红在盘 0：这道比对分不出漏了一道屏障" "比对失去判别力，之前所有「对得上」作废；看 device_log.rs 的 expected_device_events 与解析。"; }
      checks=$((checks + 3))
      control_modes+=("$mode")
      ;;
    page-cache)
      [[ "$(verdict page-cache)" == 1 ]] \
        || { sed 's/^/        /' "$work/page-cache/check.txt"
             fail "对照 page-cache 没有判红：走页缓存的回写与 O_DIRECT 的逐个写在这道比对里分不出来" "比对失去判别力；看设备侧日志里写的条数与长度。"; }
      checks=$((checks + 1))
      control_modes+=("$mode")
      ;;
    file-overwrite|second-instance|raise-rollback-floor)
      # 宿主检查照模式重跑到这一档最后那次发布，这几段与新池新建文件一样逐项比，照 direct 判：退出码 0、两块盘都没有分歧，
      # 且 `name=host_rerun` 列出的段就是这一档该跑的那几段（少一段，那一段的写就没进比对）。
      [[ "$(verdict "$mode")" == 0 ]] || { sed 's/^/        /' "$work/$mode/check.txt"
        fail "$mode：设备侧日志与程序的录制流对不上" "上面 divergence_window= 指着第一处不一致落在哪一段、divergence= 指着下标与两侧各是什么，先判是写路的错还是比对口径的错；退出码 2 是用法错或宿主重跑失败，看 check.txt 末尾那句。"; }
      checks=$((checks + 1))
      require_each_device_without_divergence "$mode"
      host_rerun_windows="$(host_rerun_windows_of "$mode")"
      [[ -n "$host_rerun_windows" ]] || fail "模式 $mode 没有登记宿主检查该重跑的段" "在 host_rerun_windows_of 里给它补一行；不补，宿主少重跑一段也看不出来。"
      grep -q "name=host_rerun mode=$mode windows=$host_rerun_windows operations_by_window=" "$work/$mode/check.txt" \
        || { sed 's/^/        /' "$work/$mode/check.txt"
             fail "$mode：宿主检查重跑的段不是 $host_rerun_windows" "看 check.txt 的 name=host_rerun 行；段名单对不上，先查送给 new_pool_file_creation_device_log_check 的第一个参数是不是 $mode。"; }
      checks=$((checks + 1))
      matched_device_log_modes+=("$mode")
      ;;
    *) fail "模式 $mode 没有登记设备侧日志的判据" "在 judge_mode_device_log 的 case 里给它补一支；不补就等于这一档的设备侧比对一个字都不判。" ;;
  esac
  return 0
}

# ── 每一档一条判定的键（文件头「复用」一段）──
# 按档查表的几个函数：它们在这一档的取值进「这一档的判据」，单元本身不进判法单元指纹（改一档的表只作废那一档）
TIER_TABLE_FUNCTIONS=(cold_root_of required_lines_of forbidden_lines_of host_rerun_windows_of mode_parameters_of)
VERDICT_LIBRARY_MODULE="$STAGE_REPOSITORY/scripts/crash/lib_verdict_library.py"
STAGE_RELATIVE=".claude/gate.d/$(basename "$0")"
digest_text() { printf '%s' "$1" | sha256sum | cut -d' ' -f1; }
digest_file() { if [[ -f "$1" ]]; then sha256sum "$1" | cut -d' ' -f1; else digest_text "没有 $(basename "$1")"; fi; }
# 这一档的判据：几张按档查表的函数在这一档交的取值
tier_table_text() { # tier_table_text <档>
  printf 'cold_root\t%s\n' "$(cold_root_of "$1")"
  printf 'host_rerun_windows\t%s\n' "$(host_rerun_windows_of "$1")"
  required_lines_of "$1" | sed 's/^/required\t/'
  forbidden_lines_of "$1" | sed 's/^/forbidden\t/'
}
# 档位参数：参数串（/bench 收到的 args）、盘与镜像参数、调用方环境里会改虚机行为的几个 VM_ 变量（没设写「未设」）
tier_parameters_text() { # tier_parameters_text <档>
  local variable
  printf 'args\t%s\nparameters\t%s\nVM_LOG_MB\t%s\n' "$1" "$(mode_parameters_of "$1")" "$QEMU_DEVICE_LOG_MEGABYTES"
  for variable in VM_MEM VM_CPUS VM_TIMEOUT VM_IOPS VM_CPU VM_EXTRA_ROOT VM_DISK_PREALLOCATE; do
    printf '%s\t%s\n' "$variable" "${!variable-未设}"
  done
}
# E142 产物里这一道真读的那几行：path=mkfs 之外的 name=segments 行、name=root_record 的 back_chain，与布局表八登记的 mkfs 那一行
e142_reading_text() {
  grep -ao 'E7RESULT name=segments .*' "$PRODUCT" | tr -d '\r' | grep -v '^E7RESULT name=segments path=mkfs '
  printf 'back_chain\t%s\nmkfs\t%s\n' "$EXPECTED_BACK_CHAIN" "$EXPECTED_MKFS_SEGMENTS_LINE"
}
# 单元键（lib_verdict_library.py unit-key）写进文件；拒算（退 4）照写，lookup 读到拒算行就不复用
unit_key_into() { # unit_key_into <输出文件> <unit-key 的参数…>
  local output="$1"; shift
  python3 "$VERDICT_LIBRARY_MODULE" unit-key "$@" > "$output"
  local status=$?
  ((status == 0 || status == 4)) || fail "算不出单元键（lib_verdict_library.py unit-key $* 退 $status）" "先让 python3 scripts/crash/lib_verdict_library.py unit-key 在这棵树上跑通（单元切法在 scripts/gate/lib_function_units.py）。"
  return 0
}
# 来宾程序单元按档算（文件头「复用」一段）：每档一份 $keys_directory/guest-<档>.txt。先一次算出六档入口各自的闭包，
# 把「别的档入口闭包里有、这一档入口闭包里没有」的单元写进 guest-excluded-<档>.txt；再逐档从 main 与这一档的入口起算、去掉那几样
GUEST_PROGRAM=crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs
guest_keys_by_mode() {
  local mode excluded entries exclusions
  if [[ "${GATE_QEMU_DEVICE_STREAMS_BREAK:-}" != guest-key-from-main ]]; then
    python3 - "$VERDICT_LIBRARY_MODULE" "$ROOT" "$GUEST_PROGRAM" "$keys_directory" "${ALL_MODES[@]}" <<'PY' \
      || fail "算不出来宾程序各档入口的闭包（$GUEST_PROGRAM 的 run_the_<档>_mode）" "看上面 python 的报错：入口改了名就照「run_the_<档，- 换成 _>_mode」改回，或改这里的入口名；单元切法在 scripts/gate/lib_function_units.py。"
import os, sys
sys.dont_write_bytecode = True
module_path, root, guest_program, keys_directory = sys.argv[1:5]
modes = sys.argv[5:]
sys.path.insert(0, os.path.dirname(module_path))
from lib_verdict_library import FunctionUnitIndex
index = FunctionUnitIndex(root)
members = {}
for mode in modes:
    closure = index.closure(f"{guest_program}::run_the_{mode.replace('-', '_')}_mode")
    # 与 FunctionUnitIndex.key 的 excluded_units 同一种写法：<文件>::<限定名>
    members[mode] = {f"{unit.file}::{unit.name}" for unit in closure.order}
for mode in modes:
    only_elsewhere = set().union(*(members[other] for other in modes if other != mode)) - members[mode]
    with open(os.path.join(keys_directory, f"guest-excluded-{mode}.txt"), "w", encoding="utf-8") as handle:
        handle.write("".join(f"{name}\n" for name in sorted(only_elsewhere)))
PY
  fi
  for mode in "${ALL_MODES[@]}"; do
    entries=("$GUEST_PROGRAM::main"); exclusions=()
    if [[ "${GATE_QEMU_DEVICE_STREAMS_BREAK:-}" != guest-key-from-main ]]; then
      entries+=("$GUEST_PROGRAM::run_the_${mode//-/_}_mode")
      [[ "${GATE_QEMU_DEVICE_STREAMS_BREAK:-}" == guest-key-without-main ]] && entries=("${entries[@]:1}")
      while IFS= read -r excluded; do exclusions+=(--exclude-unit "$excluded"); done < "$keys_directory/guest-excluded-$mode.txt"
    fi
    unit_key_into "$keys_directory/guest-$mode.txt" "$ROOT" "${entries[@]}" ${exclusions[@]+"${exclusions[@]}"} \
      "${TOOLCHAIN_OPTIONS[@]}" --toolchain "build: --release --target x86_64-unknown-linux-gnu rustflags=-C target-feature=+crt-static" --dependency-of singlefs-checker-tier
  done
}
# 共用的几样单元键与外部输入（宿主脚本、来宾与宿主检查两个程序、判法、内核、QEMU、E142），写进 $keys_directory；每一档的组成表另由 write_tier_table 写
compute_shared_keys() {
  local exclusions=() function_name
  for function_name in "${TIER_TABLE_FUNCTIONS[@]}"; do exclusions+=(--exclude-unit "$STAGE_RELATIVE::$function_name"); done
  unit_key_into "$keys_directory/host.txt" "$ROOT" scripts/devices/vm-bench.sh --external "busybox=$BUSYBOX_IDENTITY" --external "qemu-system-x86_64=$QEMU_SYSTEM_IDENTITY"
  guest_keys_by_mode
  unit_key_into "$keys_directory/check.txt" "$ROOT" crates/singlefs-checker-tier/src/bin/new_pool_file_creation_device_log_check.rs::main \
    "${TOOLCHAIN_OPTIONS[@]}" --toolchain "build: dev profile" --dependency-of singlefs-checker-tier
  unit_key_into "$keys_directory/judging.txt" "$STAGE_REPOSITORY" "$STAGE_RELATIVE::judge_mode_outputs" "$STAGE_RELATIVE::judge_mode_device_log" \
    --exclude-file scripts/crash/lib_verdict_library.py --exclude-file scripts/gate/lib_function_units.py "${exclusions[@]}" --programs-keyed-elsewhere
  E142_DIGEST="$(digest_text "$(e142_reading_text)")"
}
# 一档的组成表（lib_verdict_library.py lookup / record 读的）。弄坏开关 tier-key-is-the-whole-stage 下按档的那几样换成六档合起来的一份（整道一把键）
write_tier_table() { # write_tier_table <档>
  local mode="$1" table_text parameters_text manifest_digest every
  if [[ "${GATE_QEMU_DEVICE_STREAMS_BREAK:-}" == tier-key-is-the-whole-stage ]]; then
    table_text=""; parameters_text=""; manifest_digest=""
    for every in "${ALL_MODES[@]}"; do
      table_text+="$(tier_table_text "$every")"$'\n'; parameters_text+="$(tier_parameters_text "$every")"$'\n'
      manifest_digest+="$(digest_file "$keys_directory/manifest-$every.txt")"
    done
  else
    table_text="$(tier_table_text "$mode")"; parameters_text="$(tier_parameters_text "$mode")"; manifest_digest="$(digest_file "$keys_directory/manifest-$mode.txt")"
  fi
  { printf '宿主脚本单元\t@%s\n' "$keys_directory/host.txt"
    printf '来宾程序单元\t@%s\n' "$keys_directory/guest-$mode.txt"
    printf '宿主检查程序单元\t@%s\n' "$keys_directory/check.txt"
    printf '判法单元指纹\t@%s\n' "$keys_directory/judging.txt"
    printf '内核镜像\t%s\n' "$KERNEL_DIGEST"
    printf 'QEMU\t%s\n' "$QEMU_DIGEST"
    printf 'initramfs 清单\t%s\n' "$(digest_text "$manifest_digest")"
    printf 'E142 对照行\t%s\n' "$E142_DIGEST"
    printf '这一档的判据\t%s\n' "$(digest_text "$table_text")"
    printf '档位参数\t%s\n' "$(digest_text "$parameters_text")"; } > "$keys_directory/$mode.tsv"
}
lookup_tier() { # lookup_tier <档>：交 0 复用、1 现判；原因写进 $tier_reason
  local options=() answer=""
  [[ "${SINGLEFS_GATE_FULL:-}" == 1 ]] && options=(--no-reuse)
  if answer="$(python3 "$VERDICT_LIBRARY_MODULE" lookup "$verdict_library" qemu-tier "qemu:$1" "$keys_directory/$1.tsv" ${options[@]+"${options[@]}"})"; then
    tier_reason="$(printf '%s' "$answer" | cut -f2)"; return 0
  fi
  tier_reason="$(printf '%s' "$answer" | cut -f2)"
  local detail; detail="$(printf '%s' "$answer" | cut -f3)"
  [[ -n "$detail" ]] && tier_reason="$tier_reason：$detail"
  return 1
}
# ── 复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「二」）：六档的组成表都写好之后、查库之前抽 ──
REUSE_CHECKS_MODULE="$STAGE_REPOSITORY/scripts/gate/lib_reuse_checks.py"
QEMU_SCRIPT_KEY=".claude/gate.d/checker-tier-qemu-device-streams.sh"
plan_tier_sentinel() { # plan_tier_sentinel <状态文件> <档…>：候选是这把键在库里有判绿记录的档（verdict-library-candidate 按键文件查，与 lookup 分开）
  local state="$1" mode
  shift
  : > "$state.candidates"
  if [[ "${SINGLEFS_GATE_FULL:-}" == 1 ]]; then
    python3 "$REUSE_CHECKS_MODULE" sentinel-plan "$state" --script "$QEMU_SCRIPT_KEY" --label "QEMU 档" --off "SINGLEFS_GATE_FULL=1（不读判定库）" \
      || fail "复用哨兵没抽成（$REUSE_CHECKS_MODULE sentinel-plan）" "照它的报错修 scripts/gate/lib_reuse_checks.py；抽不成这一趟判不了复用"
    return 0
  fi
  for mode in "$@"; do
    if python3 "$REUSE_CHECKS_MODULE" verdict-library-candidate "$verdict_library" qemu-tier "$keys_directory/$mode.tsv"; then
      printf 'qemu:%s\t绿\n' "$mode" >> "$state.candidates"
    fi
  done
  python3 "$REUSE_CHECKS_MODULE" sentinel-plan "$state" --script "$QEMU_SCRIPT_KEY" --label "QEMU 档" --candidates "$state.candidates" \
    || fail "复用哨兵没抽成（$REUSE_CHECKS_MODULE sentinel-plan）" "照它的报错修 scripts/gate/lib_reuse_checks.py；抽不成这一趟判不了复用"
}
# lookup_tier_with_sentinel <状态文件> <档>：交 0 复用、1 现判；抽中强制现判的那一档不读库（tier_forced=1）。弄坏开关 reuse-lookup-skipped 下一律不读库
lookup_tier_with_sentinel() {
  tier_forced=0
  if python3 "$REUSE_CHECKS_MODULE" sentinel-forced "$1" "qemu:$2" 2>/dev/null; then
    tier_forced=1; tier_reason="复用哨兵抽中：键相同，这一趟不读库"; return 1
  fi
  if [[ "${GATE_QEMU_DEVICE_STREAMS_BREAK:-}" == reuse-lookup-skipped ]]; then
    tier_reason="无记录（弄坏开关 reuse-lookup-skipped：没读库）"; return 1
  fi
  lookup_tier "$2"
}
observe_tier() { # observe_tier <状态文件> <档> <结局> <判定>
  python3 "$REUSE_CHECKS_MODULE" sentinel-observe "$1" "qemu:$2" "$3" "$4" || echo "  ! 复用哨兵没记下 qemu:$2 的结局（这一档这一趟不当哨兵判）"
}
record_tier() { # record_tier <档> <一句判定> <跑法>
  python3 "$VERDICT_LIBRARY_MODULE" record "$verdict_library" qemu-tier "qemu:$1" "$keys_directory/$1.tsv" --summary "$2" --run "$3" >/dev/null \
    || echo "  ! qemu:$1 判绿了，没记进判定库 $verdict_library：下一趟这一档照跑"
}
# 清单的条目，/bench 那一行的内容换成占位：被测二进制由「来宾程序单元」管。run_one 拷出来的清单（VM_INITRAMFS_MANIFEST）只有条目，直接用它；
manifest_without_bench() { awk -F'\t' 'BEGIN { OFS = "\t" } $1 == "bench" { $4 = "<被测二进制，见来宾程序单元>" } { print }' "$1"; }
# vm-bench.sh --initramfs-manifest 的标准输出前面还有它的抬头与「✓ 内核 …」这几行和 kernel、qemu 两行，只取 entry<TAB> 开头的条目。
# 弄坏开关 manifest-with-listing-lines：退回改前的取法（只去掉 entry 前缀与 kernel、qemu 两行，抬头那几行留在清单里，跑完核清单永远不同、一档都不记）
manifest_entries_of_listing() {
  if [[ "${GATE_QEMU_DEVICE_STREAMS_BREAK:-}" == manifest-with-listing-lines ]]; then
    sed 's/^entry\t//' "$1" | grep -v -e '^kernel	' -e '^qemu	' | manifest_without_bench -
  else
    sed -n 's/^entry\t//p' "$1" | manifest_without_bench -
  fi
}
# 跑完每档核一遍清单再记（文件头「复用」一段 initramfs 清单那一行）：<目录>/<档>/initramfs.manifest 是 run_one 打包之前打的那一份，
# 与开跑前算键用的 $keys_directory/manifest-<档>.txt 相同才记；不同的打头几行差异、不记。键样本里 <目录> 是样本摆好的那几份。
# 弄坏开关 manifest-check-skipped：不核清单，照记（键样本 tier-key-samples-manifest-differs 里清单不同的那一档也被复用）
record_tiers_after_run() { # record_tiers_after_run <目录> <一句判定> <QEMU 的版本行> <档…>
  local runs="$1" summary="$2" version="$3" mode manifest_difference
  shift 3
  recorded_count=0
  for mode in "$@"; do
    if [[ "${GATE_QEMU_DEVICE_STREAMS_BREAK:-}" == manifest-check-skipped ]] \
       || { [[ -f "$runs/$mode/initramfs.manifest" ]] \
            && manifest_difference="$(diff <(manifest_without_bench "$runs/$mode/initramfs.manifest") "$keys_directory/manifest-$mode.txt")"; }; then
      record_tier "$mode" "$summary" "$(mode_parameters_of "$mode") VM_LOG_MB=$QEMU_DEVICE_LOG_MEGABYTES vm-bench.sh <来宾程序> $mode（$version）"
      recorded_count=$((recorded_count + 1))
    else
      echo "  ! $mode：run_one 打包之前的清单与开跑前算键用的那一份不同（或没打出来），这一档的判定没记：下一趟照跑"
      if [[ -f "$runs/$mode/initramfs.manifest" ]]; then
        printf '%s\n' "$manifest_difference" | head -n 6 | sed 's/^/        /'   # 「<」是 run_one 那一份，「>」是开跑前算键用的那一份
      fi
    fi
  done
}

# ── 键样本（.qemu-key-samples/，文件头「样本」一段）：只判键、不判输出 ──
run_key_samples() {
  local samples="$ROOT/.qemu-key-samples" mode reused=0 judged=0 name value rest
  verdict_library="$keys_directory/library"
  QEMU_MODE_PARAMETERS_TABLE="$samples/mode-parameters.tsv"
  TOOLCHAIN_OPTIONS=()
  sample_environment() {
    BUSYBOX_IDENTITY=""; QEMU_SYSTEM_IDENTITY=""; QEMU_DIGEST=""; KERNEL_DIGEST=""; TOOLCHAIN_OPTIONS=()
    while IFS=$'\t' read -r name value rest; do
      case "$name" in
        kernel) KERNEL_DIGEST="$(digest_file "$ROOT/$value")" ;;
        busybox) BUSYBOX_IDENTITY="$value=$rest" ;;
        qemu-system-x86_64) QEMU_SYSTEM_IDENTITY="$value=$rest"; QEMU_DIGEST="$(digest_text "$value $rest")" ;;
        toolchain) TOOLCHAIN_OPTIONS+=(--toolchain "$value") ;;
      esac
    done < "$samples/environment.tsv"
    for mode in "${ALL_MODES[@]}"; do
      if [[ -d "$samples/listings" ]]; then   # 样本摆了 vm-bench.sh --initramfs-manifest 的原样输出：照真跑那一条路取条目
        manifest_entries_of_listing "$samples/listings/$mode.txt" > "$keys_directory/manifest-$mode.txt"
      else
        { cat "$samples/initramfs-manifest.tsv"; printf 'args\tf\t644\t%s\n' "$(digest_text "$mode")"; } > "$keys_directory/manifest-$mode.txt"
      fi
    done
    compute_shared_keys
    for mode in "${ALL_MODES[@]}"; do write_tier_table "$mode"; done
  }
  if [[ -d "$samples/runs" ]]; then
    # 样本摆了 run_one 打包之前打的清单（runs/<档>/initramfs.manifest）：头一趟照真跑那一条路核过清单再记，清单不同的那一档不记
    echo "── 头一趟（键样本 .qemu-key-samples：判定库 $verdict_library，六档照 runs/<档>/initramfs.manifest 核过清单再记，不判输出）"
    sample_environment
    record_tiers_after_run "$samples/runs" "键样本头一趟核过清单再记" "键样本" "${ALL_MODES[@]}"
    echo "  · 头一趟记下 $recorded_count 档"
  else
    echo "── 头一趟（键样本 .qemu-key-samples：判定库 $verdict_library，六档照绿记下，不判输出）"
    sample_environment
    for mode in "${ALL_MODES[@]}"; do record_tier "$mode" "键样本头一趟照绿记" "键样本"; done
  fi
  [[ -f "$samples/unrelated.sh" ]] && run_key_samples_three_questions
  echo "── 第二趟（样本 .qemu-key-samples/change.sh 改过之后）"
  (cd "$ROOT" && bash .qemu-key-samples/change.sh) || fail "样本的 .qemu-key-samples/change.sh 没跑成" "照它的报错改样本；它只许改样本根里的文件。"
  EXPECTED_BACK_CHAIN="$(sed -n 's/.*name=root_record .*back_chain=\([0-9][0-9]*\).*/\1/p' "$PRODUCT" | head -1)"
  sample_environment
  for mode in "${ALL_MODES[@]}"; do
    if lookup_tier "$mode"; then
      reused=$((reused + 1)); echo "  · $mode：复用（$tier_reason）"   # gate-lint:detail
    else
      judged=$((judged + 1)); echo "  · $mode：现判（$tier_reason）"   # gate-lint:detail
    fi
  done
  echo "  · 按档：现判 $judged、复用 $reused"
  echo "  ! 键样本（.qemu-key-samples）：只判每一档的键作不作废，不判虚机输出，这一格本次未跑"
  echo "     → 这一档不算真跑过 QEMU：真跑要在仓顶层不带 .qemu-prerecorded 跑一遍本阶段。"
  exit 77
}

# 复用三问（键样本另放 .qemu-key-samples/unrelated.sh 与 affected 的那一份，scripts/gate/lib_reuse_checks.py 文件头「一」）：头一趟六档照绿记下之后，
# 同一份输入、跑 unrelated.sh（闭包外的改动）之后、跑 change.sh（闭包内改一个单元）之后各查一趟（每趟开着复用哨兵，现判的照绿记），
# affected 一行一档；打规范输出「REUSE_SELFTEST script=.claude/gate.d/checker-tier-qemu-device-streams.sh …」，三问与哨兵都过退 77（不判虚机输出），有一样不过判红
run_key_samples_three_questions() {
  local samples="$ROOT/.qemu-key-samples" mode sentinel_red=0 affected
  for mode in "${ALL_MODES[@]}"; do printf 'qemu:%s\t现判\n' "$mode"; done > "$keys_directory/three-first.tsv"
  key_sample_round() { # key_sample_round <名>（变量不叫 name：sample_environment 里 read 的是 name）
    local round_name="$1" state
    echo "── 键样本 $round_name 那一趟（复用三问）"
    sample_environment
    plan_tier_sentinel "$keys_directory/sentinel-$round_name.json" "${ALL_MODES[@]}"
    : > "$keys_directory/three-$round_name.tsv"
    for mode in "${ALL_MODES[@]}"; do
      if lookup_tier_with_sentinel "$keys_directory/sentinel-$round_name.json" "$mode"; then state=复用; elif ((tier_forced)); then state=强制现判; else state=现判; fi
      echo "  · qemu:$mode：绿，$state（$tier_reason）"   # gate-lint:detail
      printf 'qemu:%s\t%s\n' "$mode" "$state" >> "$keys_directory/three-$round_name.tsv"
      observe_tier "$keys_directory/sentinel-$round_name.json" "$mode" "$state" 绿
      [[ "$state" == 复用 ]] || record_tier "$mode" "键样本：这一趟现判照绿记" "键样本"   # 键样本不起虚机：现判的照绿记，下一趟比得出复用
    done
    python3 "$REUSE_CHECKS_MODULE" sentinel-finish "$keys_directory/sentinel-$round_name.json" || sentinel_red=1
  }
  key_sample_round second
  (cd "$ROOT" && bash .qemu-key-samples/unrelated.sh) || fail "样本的 .qemu-key-samples/unrelated.sh 没跑成" "照它的报错改样本；它只许改样本根里的文件。"
  key_sample_round unrelated
  (cd "$ROOT" && bash .qemu-key-samples/change.sh) || fail "样本的 .qemu-key-samples/change.sh 没跑成" "照它的报错改样本；它只许改样本根里的文件。"
  key_sample_round unit
  affected="$(sed -n 's/^\([a-z-]\{1,\}\)$/qemu:\1/p' "$samples/affected" | paste -sd,)"
  if ! python3 "$REUSE_CHECKS_MODULE" three-questions --script "$QEMU_SCRIPT_KEY" --first "$keys_directory/three-first.tsv" --second "$keys_directory/three-second.tsv" \
         --unrelated "$keys_directory/three-unrelated.tsv" --unit-change "$keys_directory/three-unit.tsv" --affected "$affected"; then
    fail "键样本的复用三问没过（红在哪一问写在上面 REUSE_SELFTEST 那一行与它上面几行）" "照 scripts/gate/lib_reuse_checks.py 文件头「一」看是哪一问：① 查库那一段，② 键里多进了闭包外的输入，③ 按档的闭包收多或收少了"
  fi
  ((sentinel_red == 0)) || fail "键样本的复用哨兵判红（上面 ✗ 那几行）" "照上面的出路查 lookup_tier_with_sentinel 与 lookup_tier"
  echo "  ! 键样本（.qemu-key-samples，复用三问）：只判每一档的键作不作废，不判虚机输出，这一格本次未跑"
  echo "     → 这一档不算真跑过 QEMU：真跑要在仓顶层不带 .qemu-prerecorded 跑一遍本阶段。"
  exit 77
}

# ── 格 device-streams（在自己的子 shell 里跑，fail 与 exit 只结束这一格）──
cell_device_streams() {
prerecorded=0
[[ -f "$ROOT/.qemu-prerecorded" ]] && prerecorded=1
key_samples=0
if ((prerecorded)) && [[ -f "$ROOT/.qemu-key-samples/change.sh" && "$(pwd -P)" != "$(cd "$STAGE_REPOSITORY" && pwd -P)" && -z "${GATE_STAGED_FROM:-}" ]]; then key_samples=1; fi
ALL_MODES=(direct skip-new-pool-file-creation-barrier page-cache file-overwrite second-instance raise-rollback-floor)
work=""
keys_directory="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-qemu-keys.XXXXXX")" || fail "建不了临时目录" "看 \$TMPDIR 满没满、有没有写权限，修好再跑。"
# 判绿才清理现场：判红时出路点名的 check.txt、out.txt 都在 $work 底下，留着现场、印出路径，删不删由看的人定；键的临时目录哪条路退都删
cleanup_device_streams() {
  local stage_exit_code=$?
  rm -rf "${keys_directory:?}"
  [[ -n "$work" && "$work" != "$ROOT" ]] || return 0
  if ((stage_exit_code == 0)) && [[ "${GATE_QEMU_DEVICE_STREAMS_KEEP_WORK:-0}" != 1 ]]; then rm -rf "${work:?}"
  elif ((stage_exit_code == 0)); then echo "     ! GATE_QEMU_DEVICE_STREAMS_KEEP_WORK=1，判绿的现场留着：$work（看完自己删）"
  else echo "     ! 判红，现场没清：$work（出路里点名的文件都在这底下，看完自己删）"; fi
}
trap cleanup_device_streams EXIT
QEMU_DEVICE_LOG_MEGABYTES="${VM_LOG_MB:-4096}"
[[ "$QEMU_DEVICE_LOG_MEGABYTES" =~ ^[1-9][0-9]*$ ]] || fail "VM_LOG_MB 写成了「$QEMU_DEVICE_LOG_MEGABYTES」，vm-bench.sh 要的是正整数（MiB）" "写成正整数，例 VM_LOG_MB=4096；不设就用一块数据盘的大小 4096。"

if ((prerecorded == 0)); then
  ((${#named_modes[@]})) && echo "  · 只跑点名的 ${#MODES[@]} 档（${MODES[*]}）：每档照键查判定库，判出的照键写库"
  [[ -f Cargo.toml && -d crates/singlefs-harness ]] || { echo "  ! 没有 crates/singlefs-harness，本阶段跳过（步 0 之前没有装置）"; exit 77; }
  # 三个前置（QEMU、/dev/kvm、内核镜像）登记在 stage-inputs.tsv 本阶段那一行第三列，没齐判红
  python3 "$ADMISSION_MODULE" gate-preconditions "$STAGE_REPOSITORY" "$(basename "$0")" || exit 1
fi

# 复跑表里 E142 有不止一行（第 2 列 @driver_e142 与 @driver_e142_p8g_crates_dump 等）：只取第 2 列恰是 @driver_e142 的那一行的第 4 列，不取全部 E142 行。
e142_row_filter='$1=="E142" && $2=="@driver_e142"'
[[ "${GATE_QEMU_DEVICE_STREAMS_BREAK:-}" == e142-all-rows ]] && e142_row_filter='$1=="E142"'
e142_product_names="$(awk -F'|' "$e142_row_filter"'{print $4}' scripts/experiments/replay.sh)"
e142_product_row_count="$(grep -c . <<<"$e142_product_names")"
if [[ "${GATE_QEMU_DEVICE_STREAMS_BREAK:-}" != e142-all-rows && "$e142_product_row_count" != 1 ]]; then
  fail "scripts/experiments/replay.sh 里第 2 列是 @driver_e142 的 E142 行取到 $e142_product_row_count 行（要恰好 1 行；0 行是这一行被删或改了名，多于 1 行是被重复登记）" \
       "看 scripts/experiments/replay.sh 里 E142|@driver_e142| 开头的行：留一行，第 4 列写这一道对照的 E142 产物文件名；驱动改了名，这里的 @driver_e142 跟着改。"
fi
PRODUCT="research/results/$e142_product_names"
[[ -f "$PRODUCT" ]] || fail "从 replay.sh 的 @driver_e142 那一行解析不出产物文件（得到 $PRODUCT）" "看 scripts/experiments/replay.sh 里 E142|@driver_e142| 那一行的第 4 列，对应的文件在不在 research/results/ 下。"
# 反向链从产物里现取，不写死：写死的数在格式常量一改就过期，红起来像是写路错了。
EXPECTED_BACK_CHAIN="$(sed -n 's/.*name=root_record .*back_chain=\([0-9][0-9]*\).*/\1/p' "$PRODUCT" | head -1)"
[[ -n "$EXPECTED_BACK_CHAIN" ]] || fail "产物里取不出 name=root_record 的 back_chain（$PRODUCT）" "看 $PRODUCT 有没有 name=root_record 那一行、它的 back_chain 字段还在不在。"
# path=mkfs 那一行的对照是布局表八「mkfs 种根」那一行登记的值（文件头「段序列分两处比」）：解析器按本阶段自己的位置取，--root 指被判的仓，
# 样本把 cwd 换成样本根时，读的是样本里那份合成的布局表、跑的仍是仓里的解析器。开虚机之前取，登记表写坏了不白等虚机。
SEGMENT_REGISTRY_SCRIPT="$STAGE_REPOSITORY/scripts/crash/check-segment-registry.py"
if ! registered_mkfs_segments_output="$(python3 "$SEGMENT_REGISTRY_SCRIPT" --root "$ROOT" --print-mkfs-segments-line)"; then
  printf '%s\n' "$registered_mkfs_segments_output" | sed 's/^/        /'
  fail "取不出 .claude/kb/layout/01-first-txn.md「八、」登记表「mkfs 种根」那一行的登记值（段序列、次操作、崩溃状态数、种类串）" \
       "照上面 scripts/crash/check-segment-registry.py 的报错改那一行；它不在就从 git 找回来（门禁 doc-registries 的 segment-registry 格用的是同一份）。"
fi
EXPECTED_MKFS_SEGMENTS_LINE="$registered_mkfs_segments_output"
EXPECTED_MKFS_SEGMENTS_SOURCE="布局表八「mkfs 种根」那一行"
if [[ "${GATE_QEMU_DEVICE_STREAMS_BREAK:-}" == mkfs-from-e142 ]]; then
  EXPECTED_MKFS_SEGMENTS_LINE="$(grep -ao 'E7RESULT name=segments path=mkfs .*' "$PRODUCT" | tr -d '\r')"
  EXPECTED_MKFS_SEGMENTS_SOURCE="E142 产物 $PRODUCT 的 path=mkfs 行"
fi
((key_samples)) && run_key_samples

reused_modes=(); judged_reasons=()
if ((prerecorded)); then
  # 预录档：不起虚机、不编译，只判样本目录里摆好的那几个模式。被判的模式现算，不写死——
  # 写死一份清单，样本里少摆一个模式就会整档静默不判（`show-me-test.md`「跳过清单要与被扫集合出自同一份数据、现算」）。
  work="$ROOT"
  present=()
  missing=()
  for mode in "${MODES[@]}"; do
    if [[ -f "$work/$mode/out.txt" && -f "$work/$mode/vm-exit" && -f "$work/$mode/check.txt" && -f "$work/$mode/check-exit" ]]; then
      present+=("$mode")
    else
      missing+=("$mode")
    fi
  done
  ((${#present[@]})) || fail "预录档：$work 下一个模式的四件套（out.txt / vm-exit / check.txt / check-exit）都没摆全" "按 <模式>/out.txt、vm-exit、check.txt、check-exit 放某一轮真跑留下来的原样输出；模式名取 ${MODES[*]} 里的。"
  MODES=("${present[@]}")
else
  # 判定库：被判那棵树的 git common-dir 里的 singlefs-qemu-verdicts/（跨轮留着；GATE_QEMU_VERDICT_LIBRARY 盖过它）；SINGLEFS_GATE_FULL=1 只写不读
  verdict_library="${GATE_QEMU_VERDICT_LIBRARY:-}"
  if [[ -z "$verdict_library" ]]; then
    common_directory="$(git -C "$ROOT" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" \
      || fail "取不到 $ROOT 的 git common-dir，判定库没处放" "在 git 工作树里跑本阶段，或设 GATE_QEMU_VERDICT_LIBRARY=<固定目录>（不放 /tmp）。"
    verdict_library="$common_directory/singlefs-qemu-verdicts"
  fi
  # 每一档的键：开跑之前算（文件头「复用」一段）。清单、内核与 QEMU 照 vm-bench.sh --initramfs-manifest 交的，与 run_one 同一段代码
  BUSYBOX_PATH="$(command -v busybox)" || fail "找不到 busybox" "装 busybox-static（apt install busybox-static）再跑。"
  BUSYBOX_IDENTITY="$(sha256sum "$BUSYBOX_PATH" | cut -d' ' -f1)=$(busybox --help 2>&1 | head -1)"
  TOOLCHAIN_OPTIONS=(--toolchain "$(cargo -V 2>&1)" --toolchain "$(rustc -V 2>&1)")
  : > "$keys_directory/bench-placeholder"
  for mode in "${MODES[@]}"; do
    env $(mode_parameters_of "$mode") VM_LOG_MB="$QEMU_DEVICE_LOG_MEGABYTES" VM_BLKLOGWRITES_DIR="$keys_directory/blk" \
      bash scripts/devices/vm-bench.sh --initramfs-manifest "$keys_directory/bench-placeholder" "$mode" > "$keys_directory/manifest-raw-$mode.txt" 2>"$keys_directory/manifest-$mode.err" \
      || { sed 's/^/        /' "$keys_directory/manifest-$mode.err"; fail "vm-bench.sh --initramfs-manifest 交不出 $mode 这一档的清单" "照上面的报错修（多半是 busybox、内核镜像或补好的 QEMU 缺了）；它不起虚机。"; }
    manifest_entries_of_listing "$keys_directory/manifest-raw-$mode.txt" > "$keys_directory/manifest-$mode.txt"
  done
  kernel_path="$(sed -n 's/^kernel\t//p' "$keys_directory/manifest-raw-${MODES[0]}.txt")"
  qemu_path="$(sed -n 's/^qemu\t//p' "$keys_directory/manifest-raw-${MODES[0]}.txt")"
  [[ -r "$kernel_path" && -x "$qemu_path" ]] || fail "vm-bench.sh 交的内核镜像（$kernel_path）或 QEMU（$qemu_path）读不了" "照 vm-bench.sh 的出路配好内核镜像（SINGLEFS_KERNEL）与补好的 QEMU（qemu.env）再跑。"
  KERNEL_DIGEST="$(sha256sum "$kernel_path" | cut -d' ' -f1)"
  qemu_version="$("$qemu_path" --version 2>&1 | head -1)"
  QEMU_SYSTEM_IDENTITY="$(sha256sum "$qemu_path" | cut -d' ' -f1)=$qemu_version"
  QEMU_DIGEST="$(digest_text "$QEMU_SYSTEM_IDENTITY $(digest_file "$(dirname "$qemu_path")/qemu-io") $(digest_file "$ROOT/scripts/devices/qemu/blklogwrites-linearize-over-iov-max.patch")")"
  compute_shared_keys
  modes_to_run=()
  echo "  · 每档一条判定（判定库 $verdict_library）："
  forced_modes=()
  for mode in "${MODES[@]}"; do write_tier_table "$mode"; done
  plan_tier_sentinel "$keys_directory/sentinel.json" "${MODES[@]}"
  for mode in "${MODES[@]}"; do
    if lookup_tier_with_sentinel "$keys_directory/sentinel.json" "$mode"; then
      reused_modes+=("$mode"); echo "  · $mode：绿，复用（$tier_reason）"   # gate-lint:detail
      observe_tier "$keys_directory/sentinel.json" "$mode" 复用 绿
    else
      modes_to_run+=("$mode"); judged_reasons+=("$mode：$tier_reason")
      ((tier_forced)) && forced_modes+=("$mode")
    fi
  done
  if ((${#modes_to_run[@]} == 0)); then
    python3 "$REUSE_CHECKS_MODULE" sentinel-finish "$keys_directory/sentinel.json" || fail "复用哨兵判红（上面 ✗ 那几行）" "照上面的出路查 lookup_tier_with_sentinel 与 lookup_tier"
    echo "  ✓ QEMU 真设备：${#MODES[@]} 档（${MODES[*]}）每档的键都对得上上一次判绿的判定，全部复用，没编、没起虚机；按档：现判 0、复用 ${#reused_modes[@]}"
    echo "     ! 这一道没罩到：崩溃注入：${#MODES[@]} 档都没有"
    exit 0
  fi
  MODES=("${modes_to_run[@]}")
  # 虚机二进制要静态链接（initramfs 里没有动态库）：glibc 目标加 +crt-static 编成 static-pie。不编 musl 目标：checker 档默认编 RocksDB 的 C++，本机没有 musl 的 C/C++ 编译器
  # 两条 cargo build 不吃 cargo 自己的缺省（核数）：经 capped.sh default 起，编译并行度取登记的重活默认并行度（.claude/heavy-thread-default，SINGLEFS_THREAD_CAP 设了取它）
  if ! bash "$STAGE_REPOSITORY/scripts/hosts/capped.sh" default cargo build --release --target x86_64-unknown-linux-gnu --config 'target.x86_64-unknown-linux-gnu.rustflags=["-C","target-feature=+crt-static"]' -p singlefs-checker-tier --bin new_pool_file_creation_on_device >/dev/null 2>&1; then
    fail "虚机二进制（glibc 静态 PIE）编不过" "单跑 cargo build --release --target x86_64-unknown-linux-gnu --config 'target.x86_64-unknown-linux-gnu.rustflags=[\"-C\",\"target-feature=+crt-static\"]' -p singlefs-checker-tier --bin new_pool_file_creation_on_device 看报错。"
  fi
  if ! bash "$STAGE_REPOSITORY/scripts/hosts/capped.sh" default cargo build -p singlefs-checker-tier --bin new_pool_file_creation_device_log_check >/dev/null 2>&1; then
    fail "宿主一侧的设备日志检查编不过" "单跑 cargo build -p singlefs-checker-tier --bin new_pool_file_creation_device_log_check 看报错。"
  fi
  work="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-checker-tier-qemu-device-streams.XXXXXX")"
  # 设备侧日志（blklogwrites）连数据记下每块盘收到的每个写，要装得下 mkfs 写零的 768 MiB 与之后的写：默认取一块数据盘的大小（4096 MiB）；
  # vm-bench.sh 自己的 256 MiB 装不下，direct 那几档会在 mkfs 里撞 I/O error。镜像是稀疏文件，只占真写进去的那些。
  # 数据盘或写路再变大，先看判红现场里 log<d>.img 的 du -m。
  missing=()
  BIN="target/x86_64-unknown-linux-gnu/release/new_pool_file_creation_on_device"
  CHECK="target/debug/new_pool_file_creation_device_log_check"
  pids=()
  for mode in "${MODES[@]}"; do
    mkdir -p "$work/$mode"
    ( env $(mode_parameters_of "$mode") VM_LOG_MB="$QEMU_DEVICE_LOG_MEGABYTES" VM_BLKLOGWRITES_DIR="$work/$mode" VM_INITRAMFS_MANIFEST="$work/$mode/initramfs.manifest" \
        bash scripts/devices/vm-bench.sh "$BIN" "$mode" >"$work/$mode/out.txt" 2>&1
      echo "$?" >"$work/$mode/vm-exit" ) &
    pids+=("$!")
  done
  for pid in "${pids[@]}"; do wait "$pid"; done   # 逐个 wait 取退出码：每台虚机把退出码写进 $work/<模式>/vm-exit，紧接着按 MODES 的顺序逐个读
fi

checks=0
for mode in "${MODES[@]}"; do
  judge_mode_outputs "$mode"
  if ((prerecorded == 0)); then
    geometry="$(grep -ao 'E7RESULT name=geometry .*' "$work/$mode/out.txt" | tr -d '\r' | head -1)"
    bytes="$(sed -n 's/.*device_bytes=\([0-9]*\).*/\1/p' <<<"$geometry")"
    physical="$(sed -n 's/.*physical_block_size=\([0-9]*\).*/\1/p' <<<"$geometry")"
    minimum="$(sed -n 's/.*minimum_io=\([0-9]*\).*/\1/p' <<<"$geometry")"
    disks=()
    [[ "$mode" == direct ]] && disks=("$work/$mode/disk0.img" "$work/$mode/disk1.img")
    "$CHECK" "$mode" "$work/$mode/log0.img" "$work/$mode/log1.img" "$bytes" "$physical" "$minimum" "${disks[@]}" >"$work/$mode/check.txt" 2>&1
    echo "$?" >"$work/$mode/check-exit"
  fi
done

judged_device_logs=0
matched_device_log_modes=(); control_modes=()
for mode in "${MODES[@]}"; do
  judge_mode_device_log "$mode"
  judged_device_logs=$((judged_device_logs + 1))
done

[[ "$judged_device_logs" == "${#MODES[@]}" ]] || fail "判了 $judged_device_logs 档设备侧日志，而这一轮跑了 ${#MODES[@]} 档" "两个数对不上就有一档被整个跳过；看上面那个 for 循环。"
uncovered="崩溃注入：${#MODES[@]} 档都没有"
if ((prerecorded)); then
  echo "  ✓ 预录档（.qemu-prerecorded，没起虚机）：判了 ${#MODES[@]} 档 ${MODES[*]}、$checks 项检查全过；样本里没摆的 ${#missing[@]} 档：${missing[*]:-无}"
  echo "     → 这一档不算真跑过 QEMU：真跑要在仓顶层不带 .qemu-prerecorded 跑一遍本阶段。"
  exit 77
fi
# 每档判绿各记一条判定（键是开跑前算的那一份）；run_one 打包之前打的清单与开跑前算键用的那一份核一遍，对不上不记（键罩的不是真跑的那一份）
record_tiers_after_run "$work" "$checks 项检查全过（这一趟 ${#MODES[@]} 档）" "$qemu_version" "${MODES[@]}"
for mode in "${MODES[@]}"; do
  if [[ " ${forced_modes[*]} " == *" $mode "* ]]; then observe_tier "$keys_directory/sentinel.json" "$mode" 强制现判 绿; else observe_tier "$keys_directory/sentinel.json" "$mode" 现判 绿; fi
done
python3 "$REUSE_CHECKS_MODULE" sentinel-finish "$keys_directory/sentinel.json" || fail "复用哨兵判红（上面 ✗ 那几行）" "照上面的出路查 lookup_tier_with_sentinel 与 lookup_tier"
echo "  ✓ QEMU 真设备：${#MODES[@]} 次虚机跑（${MODES[*]}）、$checks 项检查全过；${matched_device_log_modes[*]} 设备侧逐项对得上（宿主照模式重跑到这一档最后那次发布），${#control_modes[@]} 个对照（${control_modes[*]}）都红在该红的地方；按档：现判 ${#MODES[@]}、复用 ${#reused_modes[@]}（现判的原因：$(IFS='；'; printf '%s' "${judged_reasons[*]}")），记下 $recorded_count 档"
echo "     ! 这一道没罩到：$uncovered"
}

stage_cells_run "$ROOT"
