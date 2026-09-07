You are computing storage arithmetic for a filesystem design comparison. Do not summarize, do not praise. Do not use any markdown emphasis such as bold or italics. Answer in English. Answer all four items, number them 1, 2, 3, 4. Show the arithmetic for each. Keep each item under 250 words.

Setting. A filesystem allocates disk in fixed units of 32768 bytes. Each unit carries a self describing header. Two designs are being compared for storing many small files.

Design A, called padding, is what the filesystem does today: every file gets its own unit, and a file shorter than a unit is padded to the full 32768 bytes. A unit header of 105 bytes sits inside that space.

Design B, called packing: many small files share one unit, called a container, each file occupying a slot. A container has a header of 103 bytes plus a slot directory costing 16 bytes per slot. A container being filled is held in memory and is written out whole to a new location on each publish; once it is full it is closed and never modified again. Modifying a file that lives in a closed container means copying that file out into its own unit and leaving its old slot dead. Dead slots are only reclaimed when a background compaction pass rewrites the container.

Workload: 100000 files of 1024 bytes each.

Compute, showing your arithmetic:

1. Occupied bytes for design A and for design B when all files are written once and none are modified. Give the ratio. State how many files fit in one container under design B given the header and slot directory costs.

2. Device bytes written during the initial write of all 100000 files, for both designs. For design B, express the answer as a function of m, the number of files written between two consecutive publishes, and give the value at m equal to 1, at m equal to 8, and at m equal to the container capacity. State at which value of m design B writes as many bytes as design A, if any.

3. Now let a fraction f of the files be overwritten exactly once, with no compaction. Give occupied bytes for design B as a function of f, and find the value of f at which design B occupies as much as design A. Then state what that value means in words.

4. Failure domain. One unit becomes unreadable due to an uncorrectable error. State how many files are lost under each design, and give the ratio. Then state, for a filesystem storing 100000 small files, the expected number of files lost per unreadable unit under each design.

For each item state any assumption you had to make, and say which assumption most affects the answer.
