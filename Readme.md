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
    Finished `release` profile [optimized] target(s) in 0.03s
     Running `target/release/bench-p2p-method-call fox`
Sought fox, partially matches application: Firefox
Would you like to add this application? (Y/n)

The tree counts should be the same.
Bus tree node count:                                                             20217
P2P tree node count:                                                             20217

Bus connection time:                                                             3.18s
P2P connection time:                                                             1.63s
P2P speedup:                                                                      1.95
```

## Conclusion

P2P is often around twice as fast performing method calls.
Quite the difference.
