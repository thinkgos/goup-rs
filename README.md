# goup

`goup` is an elegant Go version manager.

There are a bunch of solutions to install Go or manage Go versions outside of a package manager:
[golang/dl](https://github.com/golang/dl), [getgo](https://github.com/golang/tools/tree/master/cmd/getgo), [gvm](https://github.com/moovweb/gvm), [goenv](https://github.com/syndbg/goenv), to name a few.


`goup` is an attempt to fulfill the above features and is heavily inspired by [Rustup](https://rustup.rs/), [golang/dl](https://github.com/golang/dl), [goup](https://github.com/owenthereal/goup) and [getgo](https://github.com/golang/tools/tree/master/cmd/getgo).

## Installation

### One-liner

```shell
cargo install goup-rs --git https://github.com/thinkgos/goup-rs
```

or

```shell
cargo install goup-rs
```

### Manual

If you want to install manually, there are the steps:

* Download the latest `goup` from `https://github.com/thinkgos/goup-rs/releases`
* Drop the `goup` executable to your `PATH` and make it executable: `mv GOUP_BIN /usr/local/bin/goup && chmod +x /usr/local/bin/goup`
* Add the Go bin directory to your shell startup script: `echo 'export PATH="$HOME/.goup/current/bin:$PATH"' >> ~/.bashrc`

## Quick Start

```shell
$ goup install
Installing go1.21.4 ...
Unpacking /home/thinkgo/.goup/go1.21.4/go1.21.4.linux-amd64.tar.gz ...
Success: go1.21.1 installed in /home/thinkgo/.goup/go1.21.4
Default Go is set to 'go1.21.4'
$ goup list
| VERSION | ACTIVE |
|---------|--------|
| 1.21.4  |   *    |
$ go env GOROOT
/home/thinkgo/.goup/current
$ go version
go version go1.21.4 linux/amd64
$ GOUP_GO_HOST=https://golang.google.cn goup install 1.21.4
```

## How it works

* `goup completion <SHELL>` Generate the autocompletion script for the specified shell.
* `goup [help]` Print this message or the help of the given subcommand(s).
* `goup install/update [VERSION]` downloads specified version of Go to`$HOME/.goup/<VERSION>/go` and symlinks it to `$HOME/.goup/current`.
* `goup use/set <VERSION>` switches to selected Go version.
* `goup ls/list/show` list all installed Go version located at `$HOME/.goup`.
* `goup remove/rm [VERSION]...` removes the specified Go version.
* `goup search [VERSION]` lists all available Go versions from `https://golang.org/dl`.
* `goup upgrade` upgrades `goup`.
* `goup init` write all necessary environment variables and values to `$HOME/.goup/env`.

## License

[Apache 2.0](LICENSE)
