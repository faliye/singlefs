#!/usr/bin/env bash
# 样本脚本：32 条开关各弄坏一格，其中 ca 那一条下面点名的项写的是 cb 那一格（点名错了：复跑要判它不符）。开关趟起跑时在 running/ 下登一个以进程号为名的文件、
# 退出前删掉，等 running/ 里同时登着 PEERS 趟（哪一趟看到过就建 .parallel-reached，之后的照放），等满 WAIT_SECONDS 秒还没等到就建 .parallel-missed；
# 有 .parallel-missed 的每一趟多打一行带自己开关值的没点名的失败行。基准趟不等。PEERS 是样本根 .break-switch-replay-jobs 写的线程数。
# 弄坏开关：
#   FANOUT_BREAK=aa  弄坏 aa 那一格
#       fanout-aa（「✗ 自检：aa 那一格」）
#   FANOUT_BREAK=ab  弄坏 ab 那一格
#       fanout-ab（「✗ 自检：ab 那一格」）
#   FANOUT_BREAK=ac  弄坏 ac 那一格
#       fanout-ac（「✗ 自检：ac 那一格」）
#   FANOUT_BREAK=ad  弄坏 ad 那一格
#       fanout-ad（「✗ 自检：ad 那一格」）
#   FANOUT_BREAK=ba  弄坏 ba 那一格
#       fanout-ba（「✗ 自检：ba 那一格」）
#   FANOUT_BREAK=bb  弄坏 bb 那一格
#       fanout-bb（「✗ 自检：bb 那一格」）
#   FANOUT_BREAK=bc  弄坏 bc 那一格
#       fanout-bc（「✗ 自检：bc 那一格」）
#   FANOUT_BREAK=bd  弄坏 bd 那一格
#       fanout-bd（「✗ 自检：bd 那一格」）
#   FANOUT_BREAK=ca  弄坏 ca 那一格
#       fanout-ca（「✗ 自检：cb 那一格」）
#   FANOUT_BREAK=cb  弄坏 cb 那一格
#       fanout-cb（「✗ 自检：cb 那一格」）
#   FANOUT_BREAK=cc  弄坏 cc 那一格
#       fanout-cc（「✗ 自检：cc 那一格」）
#   FANOUT_BREAK=cd  弄坏 cd 那一格
#       fanout-cd（「✗ 自检：cd 那一格」）
#   FANOUT_BREAK=da  弄坏 da 那一格
#       fanout-da（「✗ 自检：da 那一格」）
#   FANOUT_BREAK=db  弄坏 db 那一格
#       fanout-db（「✗ 自检：db 那一格」）
#   FANOUT_BREAK=dc  弄坏 dc 那一格
#       fanout-dc（「✗ 自检：dc 那一格」）
#   FANOUT_BREAK=dd  弄坏 dd 那一格
#       fanout-dd（「✗ 自检：dd 那一格」）
#   FANOUT_BREAK=ea  弄坏 ea 那一格
#       fanout-ea（「✗ 自检：ea 那一格」）
#   FANOUT_BREAK=eb  弄坏 eb 那一格
#       fanout-eb（「✗ 自检：eb 那一格」）
#   FANOUT_BREAK=ec  弄坏 ec 那一格
#       fanout-ec（「✗ 自检：ec 那一格」）
#   FANOUT_BREAK=ed  弄坏 ed 那一格
#       fanout-ed（「✗ 自检：ed 那一格」）
#   FANOUT_BREAK=fa  弄坏 fa 那一格
#       fanout-fa（「✗ 自检：fa 那一格」）
#   FANOUT_BREAK=fb  弄坏 fb 那一格
#       fanout-fb（「✗ 自检：fb 那一格」）
#   FANOUT_BREAK=fc  弄坏 fc 那一格
#       fanout-fc（「✗ 自检：fc 那一格」）
#   FANOUT_BREAK=fd  弄坏 fd 那一格
#       fanout-fd（「✗ 自检：fd 那一格」）
#   FANOUT_BREAK=ga  弄坏 ga 那一格
#       fanout-ga（「✗ 自检：ga 那一格」）
#   FANOUT_BREAK=gb  弄坏 gb 那一格
#       fanout-gb（「✗ 自检：gb 那一格」）
#   FANOUT_BREAK=gc  弄坏 gc 那一格
#       fanout-gc（「✗ 自检：gc 那一格」）
#   FANOUT_BREAK=gd  弄坏 gd 那一格
#       fanout-gd（「✗ 自检：gd 那一格」）
#   FANOUT_BREAK=ha  弄坏 ha 那一格
#       fanout-ha（「✗ 自检：ha 那一格」）
#   FANOUT_BREAK=hb  弄坏 hb 那一格
#       fanout-hb（「✗ 自检：hb 那一格」）
#   FANOUT_BREAK=hc  弄坏 hc 那一格
#       fanout-hc（「✗ 自检：hc 那一格」）
#   FANOUT_BREAK=hd  弄坏 hd 那一格
#       fanout-hd（「✗ 自检：hd 那一格」）
PEERS=8
WAIT_SECONDS=10
if [[ "${1:-}" == --selftest ]]; then
  switch_value="${FANOUT_BREAK:-}"
  if [[ -n "$switch_value" ]]; then
    mkdir -p running
    : > "running/$$"
    trap 'rm -f "running/$$"' EXIT
    polls=0
    while [[ ! -e .parallel-reached && ! -e .parallel-missed ]]; do
      running_count=0
      for entry in running/*; do [[ -e "$entry" ]] && running_count=$((running_count + 1)); done
      if ((running_count >= PEERS)); then : > .parallel-reached; break; fi
      if ((polls >= WAIT_SECONDS * 10)); then : > .parallel-missed; break; fi
      sleep 0.1
      polls=$((polls + 1))
    done
    [[ ! -e .parallel-missed ]] || echo "  ✗ 自检：同时在跑的不到 $PEERS 趟（这一趟 FANOUT_BREAK=$switch_value）"
    echo "  ✗ 自检：$switch_value 那一格：应当红，实际绿"
    exit 1
  fi
  echo "  ✓ 自检通过（查了 32 格）"
fi
