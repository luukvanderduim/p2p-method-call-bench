# bench-p2p

Small program to walk all nodes of a given application's tree and get each node's role.

This is performed first over the public accessibility bus connection and thereafter over the application's P2P connection.
Timings are printed for each.

## invocation

```shell
cargo run --release -- firefox
```

## Results

```console
   Compiling bench-p2p-method-call v0.1.0 (/home/luuk/code/bench-p2p-method-call)
    Finished `release` profile [optimized] target(s) in 2.45s
     Running `target/release/bench-p2p-method-call fox`
Sought fox, partially matches application: Firefox
Would you like to add this application? (Y/n)

The tree counts should be the same.
Bus tree node count:                                                              1161
P2P tree node count:                                                              1161

Bus connection time:                                                          178.55ms
Avg per node (Bus):                                                           153.79µs

P2P connection time:                                                           93.82ms
Avg per node (P2P):                                                            80.81µs

P2P speedup:                                                                      1.90
```

## Conclusion

P2P is faster.
One can observe between 1.6 - 2.0x speedups performing method calls.
