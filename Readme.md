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

## Output

```console
uuk@nippertje ~/c/p/p2p-method-call-bench (xfce4-panel)> cargo rr
    Finished `release` profile [optimized] target(s) in 0.03s
     Running `target/release/bench-p2p-method-call`
Toolkit:                                                                           gtk
Toolkit version:                                                               3.24.49

Building A11yNode tree for :1.7
A11yproxy (1) for /org/a11y/atspi/accessible/root
A11yproxy (2) for /org/a11y/atspi/accessible/3
A11yproxy (3) for /org/a11y/atspi/accessible/4
A11yproxy (4) for /org/a11y/atspi/accessible/14
A11yproxy (5) for /org/a11y/atspi/accessible/15
A11yproxy (6) for /org/a11y/atspi/accessible/17
A11yproxy (7) for /org/a11y/atspi/accessible/16
A11yproxy (8) for /org/a11y/atspi/accessible/13
A11yproxy (9) for /org/a11y/atspi/accessible/18
A11yproxy (10) for /org/a11y/atspi/accessible/20
A11yproxy (11) for /org/a11y/atspi/accessible/19
A11yproxy (12) for /org/a11y/atspi/accessible/12
A11yproxy (13) for /org/a11y/atspi/accessible/21
A11yproxy (14) for /org/a11y/atspi/accessible/11
A11yproxy (15) for /org/a11y/atspi/accessible/22
A11yproxy (16) for /org/a11y/atspi/accessible/10
A11yproxy (17) for /org/a11y/atspi/accessible/23
A11yproxy (18) for /org/a11y/atspi/accessible/2
A11yproxy (19) for /org/a11y/atspi/accessible/1
A11yproxy (20) for /org/a11y/atspi/accessible/2
A11yproxy (21) for /org/a11y/atspi/accessible/3
A11yproxy (22) for /org/a11y/atspi/accessible/4
A11yproxy (23) for /org/a11y/atspi/accessible/9
A11yproxy (24) for /org/a11y/atspi/accessible/24
A11yproxy (25) for /org/a11y/atspi/accessible/25
A11yproxy (26) for /org/a11y/atspi/accessible/8
A11yproxy (27) for /org/a11y/atspi/accessible/7
A11yproxy (28) for /org/a11y/atspi/accessible/26
A11yproxy (29) for /org/a11y/atspi/accessible/28
A11yproxy (30) for /org/a11y/atspi/accessible/30
A11yproxy (31) for /org/a11y/atspi/accessible/34
A11yproxy (32) for /org/a11y/atspi/accessible/32
A11yproxy (33) for /org/a11y/atspi/accessible/29
A11yproxy (34) for /org/a11y/atspi/accessible/27
A11yproxy (35) for /org/a11y/atspi/accessible/6
A11yproxy (36) for /org/a11y/atspi/accessible/5
A11yproxy (37) for /org/a11y/atspi/accessible/31
A11yproxy (38) for /org/a11y/atspi/accessible/1
A11yproxy (39) for /org/a11y/atspi/accessible/1
A11yproxy (40) for /org/a11y/atspi/accessible/2
A11yproxy (41) for /org/a11y/atspi/accessible/3
A11yproxy (42) for /org/a11y/atspi/accessible/4
Building A11yNode tree for :1.7
A11yproxy (1) for /org/a11y/atspi/accessible/root
A11yproxy (2) for /org/a11y/atspi/accessible/3
A11yproxy (3) for /org/a11y/atspi/accessible/4
A11yproxy (4) for /org/a11y/atspi/accessible/14
A11yproxy (5) for /org/a11y/atspi/accessible/15
A11yproxy (6) for /org/a11y/atspi/accessible/17
A11yproxy (7) for /org/a11y/atspi/accessible/16
A11yproxy (8) for /org/a11y/atspi/accessible/13
A11yproxy (9) for /org/a11y/atspi/accessible/18
A11yproxy (10) for /org/a11y/atspi/accessible/20
A11yproxy (11) for /org/a11y/atspi/accessible/19
A11yproxy (12) for /org/a11y/atspi/accessible/12
A11yproxy (13) for /org/a11y/atspi/accessible/21
A11yproxy (14) for /org/a11y/atspi/accessible/11
A11yproxy (15) for /org/a11y/atspi/accessible/22
A11yproxy (16) for /org/a11y/atspi/accessible/10
A11yproxy (17) for /org/a11y/atspi/accessible/23
A11yproxy (18) for /org/a11y/atspi/accessible/2
A11yproxy (19) for /org/a11y/atspi/accessible/1
A11yproxy (20) for /org/a11y/atspi/accessible/1
A11yproxy (21) for /org/a11y/atspi/accessible/1
Object (:1.10, /org/a11y/atspi/accessible/1) is not unique for this tree.
Error: "Objects must be unique when visiting the each node in the tree."
```

## Notes

The first pass, via the accessibility bus is fine. All (bus_name, object_path)'s are unique in the tree.

However we see in the second pass that we have a circuolar reference.
Inspecting that object, reveals that that objects child points claims to be itself. Which is pretty unique.
