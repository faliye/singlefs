BEGIN{FS="\t"; na=split("real H_scan_every_publish H_scan_only_big H_rewrite_everything",arm," ")
 nx=split("files data_units unlinks",ax," ")
 lo["files"]="base";hi["files"]="files_10000";lo["data_units"]="base";hi["data_units"]="data_units_10000";lo["unlinks"]="base";hi["unlinks"]="unlinks_10000"}
function st(a){ if(a=="real")return $24; if(a=="H_scan_every_publish")return $24+$18; if(a=="H_scan_only_big")return $24+($13>=1000?$18:0); if(a=="H_rewrite_everything")return $24+100*$16 }
function dn(a){ return a=="H_rewrite_everything"?$13+$16:$13 }
function fin(a){ return a=="H_rewrite_everything"?$14+$11+$16:$14+$11 }
$1=="publish"&&$4=="p2a"{for(i=1;i<=na;i++){a=arm[i];k=a SUBSEP $3;s=st(a);d=dn(a);f=fin(a)
 if(s>pk[k]){pk[k]=s;pkr[k]=s/d;pkf[k]=s/f}
 r=s/d; if(r>mx[k])mx[k]=r
 if($6=="same_mount"&&$7=="W1"&&$8==1&&$9==0){fr[k]=r;fs[k]=s;fx[k]=$18}
 if($7=="W1"||$7=="W4"){if(s>ws[k]){ws[k]=s;wx[k]=$18}}
}}
function al(a,b,s){ if(a<=0||b<=0||s<=1)return "NA"; return sprintf("%.3f",log(b/a)/log(s)) }
END{print "arm axis | A_ratioRs_at_step_peak B_ratioRs_max C_ratioRs_W1_1 L_ratioFinal_at_step_peak | D_raw_W1_1 E_raw_step_peak | G_raw_W1W4peak_on_records J_ratioRs_W1_1_on_records";
 for(i=1;i<=na;i++){a=arm[i];for(j=1;j<=nx;j++){x=ax[j];l=a SUBSEP lo[x];h=a SUBSEP hi[x];
  print a,x,"|",al(pkr[l],pkr[h],1000),al(mx[l],mx[h],1000),al(fr[l],fr[h],1000),al(pkf[l],pkf[h],1000),"|",al(fs[l],fs[h],1000),al(pk[l],pk[h],1000),"|",al(ws[l],ws[h],wx[h]/wx[l]),al(fr[l],fr[h],fx[h]/fx[l])}}
 print "out-of-axis W1/1 on records from base: "; n=split("files_30000 files_1000_units_10 files_1000_units_100",o," "); for(t=1;t<=n;t++){h="real" SUBSEP o[t]; l="real" SUBSEP "base"; print o[t],"raw",al(fs[l],fs[h],fx[h]/fx[l]),"ratioRs",al(fr[l],fr[h],fx[h]/fx[l]),"raw_W1W4peak",al(ws[l],ws[h],wx[h]/wx[l])}
}
