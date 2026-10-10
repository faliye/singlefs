#!/usr/bin/env bash
W=$HOME/.local/share/singlefs/work/m3-batch10z-b-code-r1-sonnet
cd $W/k3
STAGES="lr3full" ./run.sh m24-lr3-raise-no-restore
STAGES="lr3full" ./run.sh m23-prebatch-raise-reads-disk
STAGES="lr3full" ./run.sh m22-prebatch-raise-reads-disk-and-mapping-off-in-raise
echo ALLDONE3 > $W/k3/out/ALLDONE3
