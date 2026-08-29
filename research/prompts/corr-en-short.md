Answer in English, about 300 words, structured.

Question: in a copy-on-write (COW) filesystem, fsync can be implemented two ways:
(A) immediately perform a full checkpoint, writing the dirty leaves, all ancestor
nodes, and a new root;
(B) write only the dirty leaves plus one journal record, deferring the ancestor
nodes to the next checkpoint.

Explain how these two differ in what crash recovery must do, and identify the main
cost of (B).
