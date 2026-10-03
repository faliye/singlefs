#!/usr/bin/env bash
D=/tmp/claude-1000/m3-verus-land-code-r2-sonnet
python3 $D/mutate_glue2.py $D above_water_is_table_size applied_is_zero max_tx_zero passed_swapped_with_failed back_chain_inverted named_all_to_any counter_assert_le key_assert_removed skip_mode_inverted rebuilt_wrong_record > $D/mutate_glue2_out.txt 2>&1
