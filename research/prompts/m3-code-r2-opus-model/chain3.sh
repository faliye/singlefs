#!/usr/bin/env bash
set -u
D=/tmp/claude-1000/m3-code-r2-opus
s=$(date +%s); bash $D/model/run-wrongs2.sh 300 AC_only_region_zero_device_in_the_three_other_entries:y3 EC_any_self_verified_root_in_the_three_other_entries:y3 R3_reread_twice_after_the_mount:y2t Z0_zero_length_before_the_request_check:h5 E1_reinterrupted_punch_reported:none
echo "格 AC/EC y3、R3 y2t、Z0 h5、E1 编译 用时 $(( $(date +%s) - s )) 秒；下一格 COMBO_RG_RP_Z0_E1 全包，估 25 分钟" >> $D/progress.md
s=$(date +%s); bash $D/model/run-full-par.sh 401 COMBO_RG_RP_Z0_E1
echo "格 COMBO_RG_RP_Z0_E1 全包 用时 $(( $(date +%s) - s )) 秒；下一格 COMBO_EC_R3 全包，估同上" >> $D/progress.md
s=$(date +%s); bash $D/model/run-full-par.sh 402 COMBO_EC_R3
echo "格 COMBO_EC_R3 全包 用时 $(( $(date +%s) - s )) 秒；完" >> $D/progress.md
