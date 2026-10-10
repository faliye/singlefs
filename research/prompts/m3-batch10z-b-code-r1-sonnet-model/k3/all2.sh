#!/usr/bin/env bash
W=$HOME/.local/share/singlefs/work/m3-batch10z-b-code-r1-sonnet
cd $W/k3
mkdir -p out2
cp out/m19-mapping-fallback-off-in-raise.txt out2/m19-first-stages.txt
STAGES="credential slotbad reuse corrupt onepass" ./run.sh m19-mapping-fallback-off-in-raise
STAGES="lib newfile credential slotbad onepass" ./run.sh m20-rereadvalid-some-empty
STAGES="lib newfile onepass" ./run.sh m21-onepass-abandoned-arm-none
echo ALLDONE2 > $W/k3/out/ALLDONE2
