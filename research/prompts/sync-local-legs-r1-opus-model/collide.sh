#!/usr/bin/env bash
# Two ask-local.sh runs (draft copy, fake mode) redirected to the same s1: the first stands for the foreground call the harness
# moved to the background, the second for "the same command" relaunched with run_in_background (three-way-local-attack.md step 3).
# usage: bash collide.sh <draft dir> <case> <textFirst> <delayFirst> <textSecond> <delaySecond> <gapSeconds>
set -u
D=$1; C=$2; M=$(cd "$(dirname "$0")" && pwd)
AL=$D/repo/research/scripts/ask-local.sh
cd "$D/collide"; cp -f "$M/prompt-local-attack.md" prompt-local-attack.md
rm -f "c$C-s1.md" "b$C-"*.rc
export AI_CENTER_DIR=$D/fakecenter
{ ASK_LOCAL_FAKE_TEXT=$M/$3 ASK_LOCAL_FAKE_DELAY=$4 bash "$AL" prompt-local-attack.md > "c$C-s1.md"; echo "$?" > "b$C-1.rc"; } &
sleep "$7"
{ ASK_LOCAL_FAKE_TEXT=$M/$5 ASK_LOCAL_FAKE_DELAY=$6 bash "$AL" prompt-local-attack.md > "c$C-s1.md"; echo "$?" > "b$C-2.rc"; } &
wait
n=$(ls "b$C-"*.rc | wc -l); [ "$n" -eq 2 ] || { echo "batch void: $n rc files"; exit 9; }
echo "case $C rc1=$(cat b$C-1.rc) rc2=$(cat b$C-2.rc) bytes=$(wc -c < c$C-s1.md)"
echo "--- oov-check (step 5 command):"; python3 "$D/repo/research/scripts/oov-check.py" "c$C-s1.md" prompt-local-attack.md; echo "oov rc=$?"
echo "--- corruption-check:"; python3 "$D/repo/research/scripts/corruption-check.py" "c$C-s1.md" prompt-local-attack.md; echo "corruption rc=$?"
