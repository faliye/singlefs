/^E7RESULT name=history/ { n++; for(i=1;i<=NF;i++){ split($i,a,"="); if(a[1] ~ /^w3_(q1_events|gone_events|parent_missing|row_root_unknown|steps_judged|red_steps_|q1_red_|gone_red_)/) s[a[1]]+=a[2] } }
END { printf "histories=%d", n; PROCINFO["sorted_in"]="@ind_str_asc"; for(k in s) printf " %s=%d", k, s[k]; printf "\n" }
