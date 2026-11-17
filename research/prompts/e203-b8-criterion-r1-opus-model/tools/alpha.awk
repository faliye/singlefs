BEGIN{FS="\t"
 naxis=split("files data_units unlinks unit_area",axis," ")
 pts["files"]="base files_100 files_1000 files_10000"; pts["data_units"]="base data_units_100 data_units_1000 data_units_10000"; pts["unlinks"]="base unlinks_100 unlinks_1000 unlinks_10000"; pts["unit_area"]="files_1000_unit_area_2_pow_14 files_1000 files_1000_unit_area_2_pow_20 files_1000_unit_area_2_pow_24"
 span["files"]=1000;span["data_units"]=1000;span["unlinks"]=1000;span["unit_area"]=1024
 stepcol["b8"]=24;stepcol["b8pre"]=25;stepcol["b1"]=26;stepcol["b1pre"]=27;stepcol["b4"]=28
 nd=split("raw Rs W RsW Rf Af RfAf",dn," ")
}
function den(name){ if(name=="raw")return 1; if(name=="Rs")return $13; if(name=="W")return $12; if(name=="RsW")return $13+$12; if(name=="Rf")return $14; if(name=="Af")return $11; if(name=="RfAf")return $14+$11 }
$1=="publish"{for(sn in stepcol){c=stepcol[sn]; for(i=1;i<=nd;i++){d=dn[i]; v=den(d); if(v<=0)continue; k=$4 SUBSEP sn SUBSEP d SUBSEP $3; r=$c/v
   if(r>mx[k])mx[k]=r
   if($c>pk[$4 SUBSEP sn SUBSEP $3]||!(($4 SUBSEP sn SUBSEP $3) in pk)){pk[$4 SUBSEP sn SUBSEP $3]=$c}
   if($6=="same_mount"&&$7=="W1"&&$8==1&&$9==0)fi[k]=r
   if($6=="remounted"&&$7=="W1"&&$8==1&&$9==0)fr[k]=r
   if($7=="W1"||$7=="W4"){if(r>w[k])w[k]=r}
  }}
  for(sn in stepcol){c=stepcol[sn]; k=$4 SUBSEP sn SUBSEP $3; if(!(k in pv)||$c>pv[k]){pv[k]=$c; for(i=1;i<=nd;i++){d=dn[i]; v=den(d); atp[k SUBSEP d]=(v>0?$c/v:0)}}}
}
function al(a,b,s){ if(a<=0||b<=0)return "NA"; return sprintf("%.3f",log(b/a)/log(s)) }
END{ narm=split("p2a p1",arms," "); ns=split("b8 b8pre b1 b1pre b4",sts," ");
 for(ai=1;ai<=narm;ai++){arm=arms[ai]; for(si=1;si<=ns;si++){sn=sts[si]; for(ax=1;ax<=naxis;ax++){x=axis[ax]; split(pts[x],p," "); line=arm" "sn" "x
   for(i=1;i<=nd;i++){d=dn[i]; lo=arm SUBSEP sn SUBSEP d SUBSEP p[1]; hi=arm SUBSEP sn SUBSEP d SUBSEP p[4];
     line=line" | "d":F="al(fi[lo],fi[hi],span[x])" P="al(atp[arm SUBSEP sn SUBSEP p[1] SUBSEP d],atp[arm SUBSEP sn SUBSEP p[4] SUBSEP d],span[x])" M="al(mx[lo],mx[hi],span[x])" w="al(w[lo],w[hi],span[x])}
   print line}}}}
