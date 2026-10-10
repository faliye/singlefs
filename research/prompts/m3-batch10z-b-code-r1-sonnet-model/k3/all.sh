#!/usr/bin/env bash
W=$HOME/.local/share/singlefs/work/m3-batch10z-b-code-r1-sonnet
cd $W/k3
STAGES="newfile lr3 onepass" ./run.sh m2-recorded-first-only
STAGES="newfile credential lr3 onepass" ./run.sh m3-constructor-instance
STAGES="newfile credential lr3 onepass" ./run.sh m4-constructor-txg
STAGES="lib newfile credential slotbad reuse" ./run.sh m5-badatmount-some-empty
STAGES="lib newfile credential slotbad reuse" ./run.sh m6-allzero-some-empty
STAGES="lib newfile" ./run.sh m7-validroot-some-empty
STAGES="lib newfile slotbad onepass" ./run.sh m8-onepass-valid-some-empty
STAGES="lib newfile" ./run.sh m9-abandoned-compare-txg-only
STAGES="lib newfile" ./run.sh m10-abandoned-compare-instance-only
STAGES="lib newfile" ./run.sh m11-onepass-compare-txg-only
STAGES="lib newfile" ./run.sh m12-onepass-compare-instance-only
STAGES="lib newfile" ./run.sh m13-accessor-slot-off-by-one
STAGES="newfile" ./run.sh m14-key-txg-zero
STAGES="newfile corrupt credential lr3" ./run.sh m15-b-credential-always
STAGES="newfile reuse reuseslow" ./run.sh m16-count-recorded-as-unreadable
STAGES="newfile lr3 onepass" ./run.sh m17-rebuilt-occupant-empty-placements
STAGES="newfile lr3" ./run.sh m18-raise-filter-removed
STAGES="newfile lr3full" ./run.sh m19-mapping-fallback-off-in-raise
echo ALLDONE > $W/k3/out/ALLDONE
