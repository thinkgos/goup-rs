# goup

`goup` is an elegant Go version manager write in rust.

[![Rust](https://github.com/thinkgos/goup-rs/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/thinkgos/goup-rs/actions/workflows/rust.yml)
![Crates.io MSRV](https://img.shields.io/crates/msrv/goup-rs)
![Crates.io Total Downloads](https://img.shields.io/crates/d/goup-rs)
![Crates.io Crates](https://img.shields.io/crates/v/goup-rs?style=flat-square)
[![dependency status](https://deps.rs/repo/github/thinkgos/goup-rs/status.svg)](https://deps.rs/repo/github/thinkgos/goup-rs)
[![License](https://img.shields.io/github/license/thinkgos/goup-rs)](https://raw.githubusercontent.com/thinkgos/goup-rs/main/LICENSE)
[![Tag](https://img.shields.io/github/v/tag/thinkgos/goup-rs)](https://github.com/thinkgos/goup-rs/tags)

`goup` is an attempt to fulfill the above features and is heavily inspired by [Rustup](https://rustup.rs/), [golang/dl](https://github.com/golang/dl), [goup](https://github.com/owenthereal/goup), [goenv](https://github.com/syndbg/goenv), [gvm](https://github.com/moovweb/gvm) and [getgo](https://github.com/golang/tools/tree/master/cmd/getgo).

***NOTE***: Please keep in mind that `goup-rs` is still under active development and therefore full backward compatibility is not guaranteed before reaching v1.0.0.

[![asciicast](./assets/goup.gif)](https://asciinema.org/a/662585)

## Features

- Minimum dependencies, only depend on `git`. we may remove this dependency in future.
- Multi-platform compatible (Linux, macOS & Windows).
- Install/Remove Go versions with `goup install/remove [TOOLCHAIN]`.
- Support Installing Go from source with `goup install <nightly|tip|gotip>`.
- List locally installed versions.
- Switch between multiple installed versions.
- Search available version of Go.
- Manage locally cache files(such as `*.tar.gz`, `*.tar.gz.sha256`).
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

- (*Only Linux/MacOS*)Run `goup init`, Got shell startup script at `$HOME/.goup/env`.
- (*Only Linux/MacOS*)Add the Go bin directory to your shell startup script: `echo '. "$HOME/.goup/env"' >> ~/.bashrc` or `echo '. "$HOME/.goup/env"' >> ~/.zshenv`

### Manual(for Linux/MacOS)

If you want to install manually, there are the steps:

- Download the latest `goup` from [Release Page](https://github.com/thinkgos/goup-rs/releases)
- Drop the `goup` executable to your `PATH` and make it executable: `mv GOUP_BIN /usr/local/bin/goup && chmod +x /usr/local/bin/goup`
- Run `goup init`, Got shell startup script at `$HOME/.goup/env`.
- Add the Go bin directory to your shell startup script: `echo '. "$HOME/.goup/env"' >> ~/.bashrc` or `echo '. "$HOME/.goup/env"' >> ~/.zshenv`

### Manual(for Windows)

#### MSI-installers

Install the latest version for your system with the MSI-installers from the [Release Page](https://github.com/thinkgos/goup-rs/releases) section

#### Binary Compressed

- Download the binary compressed file for Windows version from [Release Page](https://github.com/thinkgos/goup-rs/releases), and then unzipping it.
- Move the `goup.exe` to `$YOUR_PATH`.
- Add the `$YOUR_PATH` to windows environment.

## Quick Start

```shell
$ goup install
[2024-01-30T00:38:48Z INFO ] Installing go1.21.10 ...
[2024-01-30T00:38:48Z INFO ] Unpacking /home/thinkgo/.goup/go1.21.10/go1.21.10.linux-amd64.tar.gz ...
[2024-01-30T00:38:48Z INFO ] go1.21.10 installed in /home/thinkgo/.goup/go1.21.10
[2024-01-30T00:38:48Z INFO ] Default Go is set to 'go1.21.10'
$ goup list
| ACTIVE  | VERSION |
|---------|---------|
|    *    | 1.21.10 |
$ go env GOROOT
/home/thinkgo/.goup/current
$ go version
go version go1.21.10 linux/amd64
$ GOUP_GO_HOST=https://golang.google.cn goup install =1.21.10
```

## Usage

### Lists all available Go versions

`goup search [FILTER]`, `[FILTER]` can be follow value **'stable'**, **'unstable'**, **'beta'** or **any regex string**.

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
1.21.10
```

### List all installed Go version located at `$HOME/.goup`

```bash
$ goup list 
+--------+---------+
| Active | Version |
+--------+---------+
|        | 1.21.10 |
+--------+---------+
|   *    | 1.22.3  |
+--------+---------+
|        | tip     |
+--------+---------+
```

### Install specified version of Go

`goup install/update [TOOLCHAIN]`, `[TOOLCHAIN]` can be follow value **'stable'(default)**, **'nightly'**(**'tip'**, **'gotip'**), **'unstable'**, **'beta'** or **'1.21.4'**, `--dry` flag means only install the version, but do not switch.  

`[TOOLCHAIN]` you can use [`semver`](https://semver.org/) syntax to match the version, See [FAQ](#faq)

```bash
$ goup install 1.21.*
[2024-01-30T00:38:48Z INFO ] Installing go1.21.10 ...
[2024-01-30T00:38:48Z INFO ] Unpacking /home/thinkgo/.goup/go1.21.10/go1.21.10.linux-amd64.tar.gz ...
[2024-01-30T00:38:48Z INFO ] go1.21.10 installed in /home/thinkgo/.goup/go1.21.10
[2024-01-30T00:38:48Z INFO ] Default Go is set to 'go1.21.10'
$ goup install =1.21.4 --dry
[2024-01-30T00:38:48Z INFO ] Installing go1.21.4 ...
[2024-01-30T00:38:48Z INFO ] Unpacking /home/thinkgo/.goup/go1.21.4/go1.21.4.linux-amd64.tar.gz ...
[2024-01-30T00:38:48Z INFO ] go1.21.10 installed in /home/thinkgo/.goup/go1.21.4
```

### Switches to selected Go version

`goup use/set [VERSION]`, switches to selected Go version.

```bash
$ goup use 
? Select a version ›
  1.21.5
❯ 1.21.10
  tip
[2024-01-30T00:38:48Z INFO ] Default Go is set to 'go1.21.10'
```

### Remove the specified Go version list

`goup remove/rm [VERSION]...` Remove the specified Go version list. If no version is provided, a prompt will show to select multiple installed Go version

```bash
$ goup rm
? Select multiple version ›
✔ 1.21.5
⬚ 1.21.10
⬚ tip
✔ Select multiple version · 1.21.5
```

### Manage cache archive files

```bash
$ goup cache show --contain-sha256
go1.21.10.linux-amd64.tar.gz
go1.21.10.linux-amd64.tar.gz.sha256

$ goup cache clean
✔ Do you want to clean cache file? · yes
```

### Modify the goup installation

```bash
$ goup self update
Checking target-arch... x86_64-unknown-linux-gnu
Checking current version... v0.9.0
Checking latest released version... v0.9.0
[2024-01-30T00:38:48Z INFO ] Update status: `v0.9.0`!
```

### Environment

```bash
$ goup env
+---------------------------+--------------------------------+---------------------------------------------------------------------------------+
| Key                       | Value                          | Explain                                                                         |
+---------------------------+--------------------------------+---------------------------------------------------------------------------------+
| GOUP_GO_HOST              | https://golang.google.cn                 | Get upstream latest/all go version, use by 'install'/'search'                                |
+---------------------------+--------------------------------+---------------------------------------------------------------------------------+
| GOUP_GO_DOWNLOAD_BASE_URL | https://dl.google.com/go       | Download go archive file base url, use by 'install'                             |
+---------------------------+--------------------------------+---------------------------------------------------------------------------------+
| GOUP_GO_SOURCE_GIT_URL    | https://github.com/golang/go   | Upstream source git url and get upstream go versions, use by 'install'/'search' |
+---------------------------+--------------------------------+---------------------------------------------------------------------------------+
| GOUP_GO_SOURCE_GIT_URL    | https://go.googlesource.com/go | Upstream source git url, use by 'install' the gotip                             |
+---------------------------+--------------------------------+---------------------------------------------------------------------------------+
```

### Autocompletion

`goup completion <SHELL>` Generate the autocompletion script for the specified shell. `<SHELL>` possible values: `bash`, `elvish`, `fish`, `powershell`, `zsh`.

```bash
goup completion zsh > _goup
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
- `goup cache [COMMAND]` Manage cache archive files.
- `goup self <COMMAND>` Modify the goup installation.
- `goup init` write all necessary environment variables and values to `$HOME/.goup/env`.
- `goup env` Show the specified goup environment variables and values.

## How to Debug

Default log level is `Info`. You can use `goup -v <subcommand>` or `goup -vv <subcommand>` to use `Debug` or `Trace` level.

## FAQ

- Compiling and Installing from source code failure?  
  The minimum version of Go required depends on the target version of Go, more information see [source installation instructions](https://go.dev/doc/install/source)
- [`semver`](https://semver.org/)
  - exact(`=`): allow updating to the latest version that exactly the version, so `=1.21.4` means exactly match the version `1.21.4`.
  - greater(`>`): allow updating to the latest version that less than or equal the version, so `>1.21.4` means greater than `1.21.4`.
  - greater equal(`>=`): allow updating to the latest version that less than or equal the version, so `<1.21.4` means greater than or equal to `1.21.4`.
  - less(`<`): allow updating to the latest version that less than or equal the version, so `<1.21.4` means less than `1.21.4`.
  - less equal(`<=`): allow updating to the latest version that less than or equal the version, so `<1.21.4` means less than or equal `1.21.4`.
  - tilde(`~`): allow updating to the latest version that does not change the major and minor version, so `~1.21.4` means greater than or equal `1.21.4`, but less than `1.22.0`.
  - caret(`^`): allow updating to the latest version that does not change the major version, so `^1.21.4` indicates that the version must be greater than or equal to `1.21.4`, but less than `2.0.0`.
  - wildcard(`*`): The operator indicates an arbitrary version. It is usually used to allow all version numbers to match.
    - `1.21.*` match all `1.21.x` versions.
    - `1.*.*` match all `1.x.x` versions.

- Go version 1.20.x or below failed to unpack.
  resolved v0.10.3 above.
  more information see [issue #251](https://github.com/thinkgos/goup-rs/issues/251)

## License

[Apache 2.0](LICENSE)
