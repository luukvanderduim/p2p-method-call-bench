# bench-p2p

Small program to walk all nodes of a given application's tree and get each node's role.

This is performed first over the public accessibility bus connection and thereafter over the application's P2P connection.
Timings are printed for each.

This branch defaults to xfce4-panel because, we succeed in building a tree through the normal bus,
but fail through P2P. Inspection shows object "/org/a11y/atspi/accessible/1" has one child which is itself.

This is a bug.
Parents must not be their own children and vice-versa.
This prevents anyone from walking an accessibility tree or caching it up-front.

## How to obtain this branch

```shell
git clone --single-branch --branch xfce4-panel https://github.com/luukvanderduim/p2p-method-call-bench.git
```

## Build

```shell
cd p2p-method-call-bench
cargo build 
```

## invocation

This branch will default to xfce4-panel, so it requires no arguments.

```shell
cargo run
```

But you can try other applications.
Note that due to debugging, this branch is very verbose.

```shell
cargo run -- gedit
```

or for the uptimized build

```shell
cargo run --release -- <executable-name>
```
