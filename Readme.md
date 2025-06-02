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
    Finished `release` profile [optimized] target(s) in 2.48s
     Running `target/release/bench-p2p-method-call firefox`
Sought firefox, found application: Firefox
Would you like to add this application? (Y/n)

Bus connection time:                                                             1.08s
P2P connection time:                                                          534.75ms
P2P speedup:                                                                      2.02
```

## Conclusion

P2P is about twice as fast performing method calls.
Quite the difference.
