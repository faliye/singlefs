#!/usr/bin/env bash
# 用法：job-list.sh <S> [格…]：打出这个 S 下各格的作业（每行一个，S:测试名:k/n）。格：a b f g holes floor alt pins；不给就全部。
s="$1"; shift
cells=("$@"); [[ ${#cells[@]} -gt 0 ]] || cells=(a b f g holes floor alt pins)
emit() { local name="$1" n="$2" k; for ((k = 0; k < n; k++)); do echo "$s:$name:$k/$n"; done; }
for cell in "${cells[@]}"; do
  case "$cell" in
    a) emit cell_a_with_the_read_faults_of_the_oldest_root 1 ;;
    b) emit cell_b_hole_at_the_next_slot "$((3 * s))" ;;
    f) emit cell_f_pinned_slot_comes_round_to_the_next_slot "$((3 * s))" ;;
    g) emit cell_g_row_publish_of_a_new_instance 7 ;;
    holes) emit z2_two_holes_in_the_ring 9 ;;
    floor) emit z2_hole_and_a_raised_floor 6 ;;
    alt) emit z2_alternating_instances 4 ;;
    pins) emit z2_every_pin_value_on_the_pinned_slot_cell 1 ;;
  esac
done
