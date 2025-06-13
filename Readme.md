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
luuk@ ~/c/bench-p2p-method-call (main)> cargo rr -- gedit
    Finished `release` profile [optimized] target(s) in 0.03s
     Running `target/release/bench-p2p-method-call gedit`
Toolkit:                                                                           gtk
Toolkit version:                                                               3.24.49

The tree counts should be the same.
Bus tree node count:                                                               500
P2P tree node count:                                                               500

Bus connection time:                                                          109.62ms
Avg per node (Bus):                                                           219.23µs

P2P connection time:                                                           42.54ms
Avg per node (P2P):                                                            85.08µs

P2P speedup:                                                                      2.58
```

## Conclusion

P2P is faster.
One can observe between 1.6 - 2.0x speedups performing method calls.
