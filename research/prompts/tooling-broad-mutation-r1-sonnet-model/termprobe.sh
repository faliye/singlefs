trap 'echo trap-ran >> termprobe.out; exit 143' TERM
{ sleep 6 2>&3 3>&-; } 3>&2 2>/dev/null
echo after-sleep >> termprobe.out
