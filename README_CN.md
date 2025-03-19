# goup

[English](https://github.com/thinkgos/goup-rs/blob/main/README.md)

`goup` 是一个纯Rust编写的优雅的Go版本管理工具.

[![Rust](https://github.com/thinkgos/goup-rs/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/thinkgos/goup-rs/actions/workflows/rust.yml)
![Crates.io MSRV](https://img.shields.io/crates/msrv/goup-rs)
![Crates.io Total Downloads](https://img.shields.io/crates/d/goup-rs)
![Crates.io Crates](https://img.shields.io/crates/v/goup-rs?style=flat-square)
[![dependency status](https://deps.rs/repo/github/thinkgos/goup-rs/status.svg)](https://deps.rs/repo/github/thinkgos/goup-rs)
[![License](https://img.shields.io/github/license/thinkgos/goup-rs)](https://raw.githubusercontent.com/thinkgos/goup-rs/main/LICENSE)
[![Tag](https://img.shields.io/github/v/tag/thinkgos/goup-rs)](https://github.com/thinkgos/goup-rs/tags)

`goup` 是对上述特性的一种尝试，其灵感主要来自于 [Rustup](https://rustup.rs/), [golang/dl](https://github.com/golang/dl), [goup](https://github.com/owenthereal/goup), [goenv](https://github.com/syndbg/goenv), [gvm](https://github.com/moovweb/gvm) and [getgo](https://github.com/golang/tools/tree/master/cmd/getgo).

***注意***: `goup-rs`仍在积极开发中, 因此在达到v1.0.0之前不能保证完全向后兼容

[![asciicast](./assets/goup.gif)](https://asciinema.org/a/662585)

## 特性

- 最小依赖, 仅依赖于`git`. 此依赖将会在未来删除.
- 跨平台的能力(Linux, macOS & Windows).
- 支持使用`goup install/remove [TOOLCHAIN]` 安装/卸载 Go版本.
- 支持使用`goup install <nightly|tip|gotip>` 从源码安装Go.
- 支持列出本地已安装的版本.
- 支持在多个已安装的版本中切换.
- 支持搜索可用的Go版本.
- 支持管理本地缓存文件(如 `*.tar.gz`, `*.tar.gz.sha256`).
- 支持`goup`自我更新.
- 友好的提示.
- 应该很快.

## 安装

### 使用Cargo

或者, 也可以使用`cargo`安装.

```shell
cargo install goup-rs
```

或

```shell
cargo install goup-rs --git https://github.com/thinkgos/goup-rs
```

- (*仅支持 Linux/MacOS*) 运行`goup init`, 获取到shell启动脚本位于`$HOME/.goup/env`.
- (*仅支持 Linux/MacOS*) 在shell启动脚本中添加Go的bin目录: `echo '. "$HOME/.goup/env"' >> ~/.bashrc` 或者 `echo '. "$HOME/.goup/env"' >> ~/.zshenv`

### 手动安装(Linux/MacOS)

如果您想手动安装, 步骤如下:

- 从[Release Page](https://github.com/thinkgos/goup-rs/releases)下载最新的`goup`.
- 将`goup`可执行文件放到`PATH`中, 并给予可执行权限: `mv GOUP_BIN /usr/local/bin/goup && chmod +x /usr/local/bin/goup`
- 运行`goup init`, 获取到shell启动脚本位于`$HOME/.goup/env`.
- 在shell启动脚本中添加Go的bin目录: `echo '. "$HOME/.goup/env"' >> ~/.bashrc` or `echo '. "$HOME/.goup/env"' >> ~/.zshenv`

### 手动安装(Windows)

#### MSI安装

从[Release Page](https://github.com/thinkgos/goup-rs/releases)下载最新的`goup`的MSI安装程序并运行.

#### 二进制安装

- 从[Release Page](https://github.com/thinkgos/goup-rs/releases)下载最新的`goup`的二进制程序并解压.
- 将`goup.exe`移至`$YOUR_PATH`.
- 将`$YOUR_PATH`加到windows环境变量中.

## 快速入门

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

## 使用方法

### 列出所有可用的go版本

`goup search [FILTER]`, `[FILTER]`支持的值: **'stable'**, **'unstable'**, **'beta'** 或 **any regex string**.

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

### 列出所有位于`$HOME/.goup`已安装的Go版本

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

### 安装指定Go版本

`goup install/update [TOOLCHAIN]`, `[TOOLCHAIN]` 支持的值: **'stable'(default)**, **'nightly'**(**'tip'**, **'gotip'**), **'unstable'**, **'beta'** 或 **'1.21.4'**, `--dry` 表示只安装对应版本, 但并不切换使用.  

`[TOOLCHAIN]` 支持[`semver`](https://semver.org/)语法匹配对应版本, 详情查看[FAQ](#faq)

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

### 切换到选定的Go版本

`goup use/set [VERSION]`, 切换到选定的Go版本.

```bash
$ goup use 
? Select a version ›
  1.21.5
❯ 1.21.10
  tip
[2024-01-30T00:38:48Z INFO ] Default Go is set to 'go1.21.10'
```

### 删除指定的 Go 版本列表

`goup remove/rm [VERSION]...` 删除指定的 Go 版本列表. 如果没有提供版本, 将提示选择多个已安装的 Go 版本

```bash
$ goup rm
? Select multiple version ›
✔ 1.21.5
⬚ 1.21.10
⬚ tip
✔ Select multiple version · 1.21.5
```

### 管理缓存归档文件

```bash
$ goup cache show --contain-sha256
go1.21.10.linux-amd64.tar.gz
go1.21.10.linux-amd64.tar.gz.sha256

$ goup cache clean
✔ Do you want to clean cache file? · yes
```

### 修改`goup`安装程序

```bash
$ goup self update
Checking target-arch... x86_64-unknown-linux-gnu
Checking current version... v0.9.0
Checking latest released version... v0.9.0
[2024-01-30T00:38:48Z INFO ] Update status: `v0.9.0`!
```

### 环境变量值

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

### Shell补全

`goup completion <SHELL>`  为指定shell生成补全脚本. `<SHELL>`支持这些值: `bash`, `elvish`, `fish`, `powershell`, `zsh`.

```bash
goup completion zsh > _goup
```

### 更多信息

执行`goup -h`获取更多信息

## 工作原理

- `goup completion <SHELL>` 为指定shell生成补全脚本.
- `goup [help]`  打印此信息或给定子命令的帮助信息.
- `goup install/update [TOOLCHAIN]` 下载指定的Go版本到`$HOME/.goup/go<VERSION|tip>/go`并创建一个软链接到`$HOME/.goup/current`.
- `goup use/set [VERSION]` 切换到选择的Go版本.
- `goup ls/list/show` 列出所有位置`$HOME/.goup`已安装的Go版本.
- `goup remove/rm [VERSION]...` 移除指定的Go版本列表.
- `goup search [FILTER]` 列出所有可用的 o版本.
- `goup cache [COMMAND]` 管理缓存归档文件.
- `goup self <COMMAND>` 修改`goup`安装程序.
- `goup init` 将所有必要的环境变量和值写入`$HOME/.goup/env`.
- `goup env`  显示`goup`的环境变量和值.

## How to Debug

默认日志级别为`Info`. 你可以使用`goup -v <subcommand>` 或 `goup -vv <subcommand>` 来使用 `Debug` 或 `Trace` 等级.

## FAQ

- 编译和安装源代码失败?  
  所需的Go最低版本取决于Go的目标版本, 更多信息请参见[source installation instructions](https://go.dev/doc/install/source)
- [`semver`](https://semver.org/)
  - exact(`=`):  允许更新到与版本完全一致的最新版本, 因此`=1.21.4`表示与版本`1.21.4`完全一致。
  - greater(`>`): 允许更新到大于该版本的最新版本, 因此`>1.21.4`表示大于`1.21.4`.
  - greater equal(`>=`): 允许更新到大于或等于该版本的最新版本, 因此 `>1.21.4` 表示大于或等于`1.21.4`.
  - less(`<`): 允许更新到小于该版本的最新版本, 因此`>1.21.4`表示大于`1.21.4`.
  - less equal(`<=`): 允许更新到小于或等于该版本的最新版本, 因此 `>1.21.4` 表示小于或等于`1.21.4`.
  - tilde(`~`): 允许更新到不改变主,次版本的最新版本, 因此`~1.21.4`表示大于或等于 `1.21.4`, 但小于 `1.22.0`.
  - caret(`^`): 允许更新到不改变主要版本的最新版本, 因此 `^1.21.4` 表示版本必须大于或等于 `1.21.4`, 但小于 `2.0.0`.
  - wildcard(`*`): 此运算符表示任意版本. 它通常用于允许所有版本号匹配.
    - `1.21.*` 匹配所有`1.21.x`版本.
    - `1.*.*` 匹配所有`1.x.x`版本.

- Go版本小于等于1.20.x解压失败.
  大于v0.10.3版本已解决.
  看多信息查看[issue #251](https://github.com/thinkgos/goup-rs/issues/251)

## 许可证

[Apache 2.0](LICENSE)
