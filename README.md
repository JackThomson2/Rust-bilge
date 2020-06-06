# Rust-bilge
Emulating and playing game https://yppedia.puzzlepirates.com/Bilging

Will use a multithreaded DFS spliting at the top level depending on the num of cores. A concurrent hashmap will be used to cache results.


                         GETMOVES()
                   /      /       \      \
                 Moves split between threads
                / | \  / | \    / | \   / | \   
       EACH THREAD GETS ALL MOVES AND SOLVES TO SET DEPTH. 


# Performance tracking

## Run with `bilgebot bench`

Using my desktop at depth 6

| Date      | Speed | Changes |
| ----------- | ----------- | ----------- |
| 06/06/20      | 344ms      | This was the base test |
|    |         |                         |