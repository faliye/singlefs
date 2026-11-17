# W2 第 1 次：步数 ÷ 用户这一次换下的单元数（下一次发布 mapped 之差 + 1，外生，不吃固定点）；再比同一改动档、总量差几百倍的两次 W2。
BEGIN{FS="\t"; OFS=" "}
$1=="publish"&&$4=="p2a"&&$6=="same_mount"&&$7=="W2"&&($8==1||$8==2){m[$3,$8]=$16; b[$3,$8]=$24; rs[$3,$8]=$13; rec[$3,$8]=$18; af[$3,$8]=$11}
function al(x,y,s){return sprintf("%.3f",log(y/x)/log(s))}
END{n=split("base data_units_100 data_units_1000 data_units_10000",p," ")
 for(i=1;i<=n;i++){k=p[i]; u[k]=m[k,1]-m[k,2]+1; v[k]=b[k,1]/u[k]; print k,"W2_1_steps="b[k,1],"user_units="u[k],"steps_per_user_unit="sprintf("%.1f",v[k]),"released_sum="rs[k,1],"released_per_user_unit="sprintf("%.2f",rs[k,1]/u[k]),"final_roles="af[k,1]}
 print "alpha_steps_per_user_unit_on_axis="al(v["base"],v["data_units_10000"],1000),"alpha_released_per_user_unit_on_axis="al(rs["base",1]/u["base"],rs["data_units_10000",1]/u["data_units_10000"],1000)
 a="data_units_100";c="files_1000_units_100"
 print "same_change_cell",a,"W2_1 steps="b[a,1],"Rs="rs[a,1],"mapped="m[a,1],"records="rec[a,1],c,"W2_2 steps="b[c,2],"Rs="rs[c,2],"mapped="m[c,2],"records="rec[c,2]
 print "alpha_raw_on_records="al(b[a,1],b[c,2],rec[c,2]/rec[a,1]),"alpha_raw_on_mapped="al(b[a,1],b[c,2],m[c,2]/m[a,1]),"alpha_ratioRs_on_records="al(b[a,1]/rs[a,1],b[c,2]/rs[c,2],rec[c,2]/rec[a,1])
}
