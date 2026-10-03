#!/usr/bin/env bash
set -u
cd /tmp/claude-1000/m3-formal-r3-sonnet
bash work/batch-e165.sh
cd verus && bash run-verus-batch.sh $(ls -d variants/*/ | sed 's:/$::')
echo master-done > ../master.done
