# govm

`govm` is an elegant Go version manager.This is [goup](https://github.com/owenthereal/goup)'s a `rust` implementation.

There are a bunch of solutions to install Go or manage Go versions outside of a package manager:
[golang/dl](https://github.com/golang/dl), [getgo](https://github.com/golang/tools/tree/master/cmd/getgo), [gvm](https://github.com/moovweb/gvm), [goenv](https://github.com/syndbg/goenv), to name a few.

I want a Go version manager that:

* Has a minimum prerequisite to install, e.g., does not need a Go compiler to pre-exist.
* Is installed with a one-liner.
* Runs well on all operating systems (at least runs well on *uix as a start).
* Installs any version of Go (any version from [golang.org/dl](https://golang.org/dl) or tip) and switches to it.
* Does not inject magic into your shell.
* Is written in Go.

`govm` is an attempt to fulfill the above features and is heavily inspired by [Rustup](https://rustup.rs/), [golang/dl](https://github.com/golang/dl) and [getgo](https://github.com/golang/tools/tree/master/cmd/getgo).

## Installation

### One-liner

```shell
cargo install govm --git https://github.com/thinkgos/govm
```

### Manual

If you want to install manually, there are the steps:

* Download the latest `govm` from `https://github.com/thinkgos/govm/releases`
* Drop the `govm` executable to your `PATH` and make it executable: `mv GOVM_BIN /usr/local/bin/govm && chmod +x /usr/local/bin/govm`
* Add the Go bin directory to your shell startup script: `echo 'export PATH="$HOME/.go/current/bin:$PATH"' >> ~/.bashrc`

## Quick Start

```shell
$ govm install
Installing go1.21.4 ...
Unpacking /home/thinkgo/.go/go1.21.4/go1.21.4.linux-amd64.tar.gz ...
Success: go1.21.1 installed in /home/thinkgo/.go/go1.21.4
Default Go is set to 'go1.21.4'
$ govm list
| VERSION | ACTIVE |
|---------|--------|
| 1.21.4  |   *    |
$ go env GOROOT
/home/thinkgo/.go/current
$ go version
go version go1.21.4 linux/amd64
$ GOVM_GO_HOST=golang.google.cn govm install 
```

## How it works

* `govm completion <SHELL>` Generate the autocompletion script for the specified shell.
* `govm [help]` Print this message or the help of the given subcommand(s).
* `govm install/update [VERSION]` downloads specified version of Go to`$HOME/.go/VERSION` and symlinks it to `$HOME/.go/current`.
* `govm use/set <VERSION>` switches to selected Go version.
* `govm ls/list` list all installed Go version located at `$HOME/.go/current`.
* `govm remove/rm [VERSION]...` removes the specified Go version.
* `govm search [VERSION]` lists all available Go versions from `https://golang.org/dl`.
* `govm upgrade` upgrades `govm`.

## License

[Apache 2.0](LICENSE)