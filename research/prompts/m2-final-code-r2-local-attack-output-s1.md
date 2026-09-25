W.max_s
The formula from FACT 8 is (R × S) − 1 where R is 3 and S is 16. Calculation: 3×16=48, 48−1=47. Result: 47. This would be refuted by S being different from 16.

W.table_bytes
The formula is 1 + (W.max_s × 16). Calculation: 1 + (47×16)=753. Result: 753. This would be refuted by W.max_s being different from 47.

W.fits_in_slot
The start offset is 481 and table size is 753, so 481 + 753 = 1234. The slot size is 4096. Since 1234 ≤ 4096, yes. This would be refuted by W.table_bytes being greater than 3615.

AC.leaf_cap
The formula is floor((16384 − 159)/34). Calculation: 16384−159=16225, 16225/34=477.205, floor to 477. Result: 477. This would be refuted by format-2 node size not being 16384 bytes.

AC.internal_cap
The formula is floor((16384 − 159)/108). Calculation: 16384−159=16225, 16225/108=150.231, floor to 150. Result: 150. This would be refuted by format-2 node size not being 16384 bytes.

MP.leaf_cap
The formula is floor((16384 − 169)/55). Calculation: 16384−169=16215, 16215/55=294.818, floor to 294. Result: 294. This would be refuted by format-2 node size not being 16384 bytes.

MP.internal_cap
The formula is floor((16384 − 169)/113). Calculation: 16384−169=16215, 16215/113=143.5, floor to 143. Result: 143. This would be refuted by format-2 node size not being 16384 bytes.

ROWS.height_at_15
For h=1: capacity = 477 × 1 = 477 ≥ 15. Result: 1. This would be refuted by AC.leaf_cap being less than 15.

ROWS.min_devices_to_split
D=1: 9; D=2:15; D=3:21; D=4:27; D=5:33; D=6:39; D=7:45; D=8:51; D=9:57; D=10:63; D=11:69; D=12:75; D=13:81; D=14:87; D=15:93; D=16:99; D=17:105; D=18:111; D=19:117; D=20:123; D=21:129; D=22:135; D=23:141; D=24:147; D=25:153; D=26:159; D=27:165; D=28:171; D=29:177; D=30:183; D=31:189; D=32:195; D=33:201; D=34:207; D=35:213; D=36:219; D=37:225; D=38:231; D=39:237; D=40:243; D=41:249; D=42:255; D=43:261; D=44:267; D=45:273; D=46:279; D=47:285; D=48:291; D=49:297; D=50:303; D=51:309; D=52:315; D=53:321; D=54:327; D=55:333; D=56:339; D=57:345; D=58:351; D=59:357; D=60:363; D=61:369; D=62:375; D=63:381; D=64:387; D=65:393; D=66:399; D=67:405; D=68:411; D=69:417; D=70:423; D=71:429; D=72:435; D=73:441; D=74:447; D=75:453; D=76:459; D=77:465; D=78:471; D=79:477; D=80:483. Result: 80. This would be refuted by AC.leaf_cap being different from 477.

REC.named_cap
The formula is floor((4096 − 311)/56). Calculation: 4096−311=3785, 3785/56=67.589, floor to 67. Result: 67. This would be refuted by named item size not being 56 bytes.
