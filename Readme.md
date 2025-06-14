# bench-p2p

Small program to walk all nodes of a given application's tree and get each node's role.

This is performed first over the common accessibility bus connection and thereafter over the application's P2P connection(s).
Timings are printed for each.

## Objective

Peer-to-peer improves the latency of method calls / method returns by an appreciable amount.

This is a proof of concept on how to implement P2P for [atspi](https://github.com/odilia-app/atspi).

## Clone the crate

```shell
git clone https://github.com/luukvanderduim/p2p-method-call-bench.git
```

## Build

```shell
cd p2p-method-call-bench
cargo build 
```

## invocation

```shell
cargo run
```

But you can try applications or busnames as arguments.

```shell
cargo run -- gedit
```

or for the uptimized build

```shell
cargo run --release -- <executable-name>
```
