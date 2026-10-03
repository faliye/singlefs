#!/usr/bin/env bash
set -u
D=/tmp/claude-1000/m3-code-r2-opus
s=$(date +%s); bash $D/model/run-wrongs2.sh 500 Z0_zero_length_before_the_request_check:h5 AC_only_region_zero_device_in_the_three_other_entries:entries
echo "格 Z0 h5、AC entries 用时 $(( $(date +%s) - s )) 秒；下一格 COMBO_RG_RP_Z0_E1 全包，估 35 分钟" >> $D/progress.md
s=$(date +%s); bash $D/model/run-full-par.sh 601 COMBO_RG_RP_Z0_E1
echo "格 COMBO_RG_RP_Z0_E1 全包 用时 $(( $(date +%s) - s )) 秒；下一格 COMBO_EC_R3 全包，估同上" >> $D/progress.md
s=$(date +%s); bash $D/model/run-full-par.sh 602 COMBO_EC_R3
echo "格 COMBO_EC_R3 全包 用时 $(( $(date +%s) - s )) 秒；下一格 H6 停进程" >> $D/progress.md
bash $D/model/h6-kill.sh /tmp/claude-1000/m3-code-r2-opus/target/debug/deps/mkfs_on_image_files_punches_the_zeroed_ranges-a42a635dc651a477
echo "格 H6 完" >> $D/progress.md
