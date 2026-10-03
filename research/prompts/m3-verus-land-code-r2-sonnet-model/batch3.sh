#!/usr/bin/env bash
D=/tmp/claude-1000/m3-verus-land-code-r2-sonnet
python3 $D/mutate_new_core.py $D > $D/mutate_new_core_out.txt 2>&1
python3 $D/mutate_glue.py $D > $D/mutate_glue_out.txt 2>&1
