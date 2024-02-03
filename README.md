# goup

`goup` is an elegant Go version manager write in rust.

[![Rust](https://github.com/thinkgos/goup-rs/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/thinkgos/goup-rs/actions/workflows/rust.yml)
![Crates.io MSRV](https://img.shields.io/crates/msrv/goup-rs)
![Crates.io Total Downloads](https://img.shields.io/crates/d/goup-rs)
![Crates.io Crates](https://img.shields.io/crates/v/goup-rs?style=flat-square)
[![Licence](https://img.shields.io/github/license/thinkgos/goup-rs)](https://raw.githubusercontent.com/thinkgos/goup-rs/main/LICENSE)
[![Tag](https://img.shields.io/github/v/tag/thinkgos/goup-rs)](https://github.com/thinkgos/goup-rs/tags)
[![Rust](https://github.com/thinkgos/goup-rs/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/thinkgos/goup-rs/actions/workflows/rust.yml)

`goup` is an attempt to fulfill the above features and is heavily inspired by [Rustup](https://rustup.rs/), [golang/dl](https://github.com/golang/dl), [goup](https://github.com/owenthereal/goup), [goenv](https://github.com/syndbg/goenv), [gvm](https://github.com/moovweb/gvm) and [getgo](https://github.com/golang/tools/tree/master/cmd/getgo).

***NOTE***: Please keep in mind that `goup-rs` is still under active development and therefore full backward compatibility is not guaranteed before reaching v1.0.0.

## Features

- Minimum dependencies, only depend on `git`. we may remove this dependency in future.
- Multi-platform compatible (Linux, macOS & Windows).
- Install/Remove Go versions with `goup install/remove`. Such as `tip` version.
- List locally installed versions.
- Switch between multiple versions.
- Search available version of Go.
- Upgrade `goup` itself.
- Friendly prompt.
- Should be pretty fast.

## Installation

### Cargo

Alternatively, you can also install it using `cargo`.

```shell
cargo install goup-rs
```

or

```shell
cargo install goup-rs --git https://github.com/thinkgos/goup-rs
```

### Manual(for Linux/MacOS)

If you want to install manually, there are the steps:

- Download the latest `goup` from [Release Page](https://github.com/thinkgos/goup-rs/releases)
- Drop the `goup` executable to your `PATH` and make it executable: `mv GOUP_BIN /usr/local/bin/goup && chmod +x /usr/local/bin/goup`
- Run `goup init`, Got shell startup script at `$HOME/.goup/env`.
- Add the Go bin directory to your shell startup script: `echo '. "$HOME/.goup/env"' >> ~/.bashrc` or `echo '. "$HOME/.goup/env"' >> ~/.zshenv`

### Manual(for Windows)

I do not have Windows. So you know, welcome PR.

## Quick Start

```shell
$ goup install
[2024-01-30T00:38:48Z INFO ] Installing go1.21.6 ...
[2024-01-30T00:38:48Z INFO ] Unpacking /home/thinkgo/.goup/go1.21.6/go1.21.6.linux-amd64.tar.gz ...
[2024-01-30T00:38:48Z INFO ] go1.21.6 installed in /home/thinkgo/.goup/go1.21.6
[2024-01-30T00:38:48Z INFO ] Default Go is set to 'go1.21.6'
$ goup list
| VERSION | ACTIVE |
|---------|--------|
| 1.21.6  |   *    |
$ go env GOROOT
/home/thinkgo/.goup/current
$ go version
go version go1.21.6 linux/amd64
$ GOUP_GO_HOST=https://golang.google.cn goup install 1.21.6
```

## Usage

### Lists all available Go versions from `https://golang.org/dl`

`goup search [FILTER]`, `[FILTER]` can be follow value 'stable', "unstable", 'beta' or any regex string.

```bash
$ goup search
1
...
1.21rc4
1.22rc1
$ goup search stable
1
...
1.21.4
1.21.5
1.21.6
```

### List all installed Go version located at `$HOME/.goup`

```bash
$ goup list 
+---------+--------+
| Version | Active |
+---------+--------+
| 1.21.5  |        |
+---------+--------+
| 1.21.6  |   *    |
+---------+--------+
| tip     |        |
+---------+--------+
```

### Install specified version of Go

`goup install/update [TOOLCHAIN]`, `[TOOLCHAIN]` can be follow value 'stable'(default), 'nightly'('tip', 'gotip'), 'unstable', 'beta' or '1.21.4'('go1.21.4'), `--dry` flag means only install the version, but do not switch

```bash
$ goup install
[2024-01-30T00:38:48Z INFO ] Installing go1.21.6 ...
[2024-01-30T00:38:48Z INFO ] Unpacking /home/thinkgo/.goup/go1.21.6/go1.21.6.linux-amd64.tar.gz ...
[2024-01-30T00:38:48Z INFO ] go1.21.6 installed in /home/thinkgo/.goup/go1.21.6
[2024-01-30T00:38:48Z INFO ] Default Go is set to 'go1.21.6'
$ goup install 1.21.4 --dry
[2024-01-30T00:38:48Z INFO ] Installing go1.21.4 ...
[2024-01-30T00:38:48Z INFO ] Unpacking /home/thinkgo/.goup/go1.21.4/go1.21.4.linux-amd64.tar.gz ...
[2024-01-30T00:38:48Z INFO ] go1.21.6 installed in /home/thinkgo/.goup/go1.21.4
```

### Switches to selected Go version

`goup use/set [VERSION]`, switches to selected Go version.

```bash
$ goup use 
? Select a version ›
  1.21.5
❯ 1.21.6
  tip
[2024-01-30T00:38:48Z INFO ] Default Go is set to 'go1.21.6'
```

### Remove the specified Go version list

`goup remove/rm [VERSION]...` Remove the specified Go version list. If no version is provided, a prompt will show to select multiple installed Go version

```bash
$ goup rm
? Select multiple version ›
✔ 1.21.5
⬚ 1.21.6
⬚ tip
✔ Select multiple version · 1.21.5
```

### Modify the goup installation

```bash
$ goup self update
Checking target-arch... x86_64-unknown-linux-gnu
Checking current version... v0.3.0
Checking latest released version... v0.3.0
[2024-01-30T00:38:48Z INFO ] Update status: `v0.3.0`!
```

### Environment

```bash
$ goup env
+---------------------------+--------------------------------+---------------------------------------------------------------------------------+
| Key                       | Value                          | Explain                                                                         |
+---------------------------+--------------------------------+---------------------------------------------------------------------------------+
| GOUP_GO_HOST              | https://go.dev                 | Get upstream latest go version, use by 'install'                                |
+---------------------------+--------------------------------+---------------------------------------------------------------------------------+
| GOUP_GO_DOWNLOAD_BASE_URL | https://dl.google.com/go       | Download go archive file base url, use by 'install'                             |
+---------------------------+--------------------------------+---------------------------------------------------------------------------------+
| GOUP_GO_SOURCE_GIT_URL    | https://github.com/golang/go   | Upstream source git url and get upstream go versions, use by 'install'/'search' |
+---------------------------+--------------------------------+---------------------------------------------------------------------------------+
| GOUP_GO_SOURCE_GIT_URL    | https://go.googlesource.com/go | Upstream source git url, use by 'install' the gotip                             |
+---------------------------+--------------------------------+---------------------------------------------------------------------------------+
```

### More information

`goup -h` get more information

## How it works

- `goup completion <SHELL>` Generate the autocompletion script for the specified shell.
- `goup [help]` Print this message or the help of the given subcommand(s).
- `goup install/update [TOOLCHAIN]` downloads specified version of Go to`$HOME/.goup/go<VERSION|tip>/go` and symlinks it to `$HOME/.goup/current`.
- `goup use/set [VERSION]` switches to selected Go version.
- `goup ls/list/show` list all installed Go version located at `$HOME/.goup`.
- `goup remove/rm [VERSION]...` remove the specified Go version list.
- `goup search [FILTER]` lists all available Go versions.
- `goup upgrade` upgrades `goup`, deprecated in future version, use `goup self update` instead
- `goup self <COMMAND>` Modify the goup installation.
- `goup init` write all necessary environment variables and values to `$HOME/.goup/env`.
- `goup env` Show the specified goup environment variables and values.

## FAQ

- TODO...

## License

[Apache 2.0](LICENSE)
