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

***注意***: `goup-rs`仍在积极开发中, 因此在达到v1.0.0之前不能保证完全向后兼容

[![asciicast](./assets/goup.gif)](https://asciinema.org/a/662585)

## 特性

- 最小依赖, 依赖于`git`(仅`nightly|tip|gotip`版本需要`git`).
- 跨平台的能力(Linux, macOS & Windows).
- 支持使用`goup install/remove [TOOLCHAIN]` 安装/卸载 Go版本.
- 支持使用`goup install <nightly|tip|gotip>` 从源码安装Go, 需要`git`.
- 支持列出本地已安装的版本.
- 支持在多个已安装的版本中切换.
- 支持搜索可用的Go版本.
- 支持在shell会话中使用特定的Go版本(>= v0.15.x).
- 支持管理本地缓存文件(如 `*.tar.gz`, `*.tar.gz.sha256`).
- 支持`goup`自我更新.
- 支持自定义`GOUP_HOME`(默认`$HOME/.goup`)(>= v0.11.x);
- 友好的提示.
- 应该很快.

`goup` 是对上述特性的一种尝试, 其灵感主要来自于 [Rustup](https://rustup.rs/), [golang/dl](https://github.com/golang/dl), [goup](https://github.com/owenthereal/goup), [goenv](https://github.com/syndbg/goenv), [gvm](https://github.com/moovweb/gvm) and [getgo](https://github.com/golang/tools/tree/master/cmd/getgo).

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
- (*仅支持 Linux/MacOS*) 在shell启动脚本中添加Go的bin目录:
  - bash: `echo '. "$HOME/.goup/env"' >> ~/.bashrc`
  - zsh:  `echo '. "$HOME/.goup/env"' >> ~/.zshenv`
  - fish: `echo 'source ~/.goup/env' >> ~/.config/fish/config.fish`

### 手动安装(Linux/MacOS)

如果您想手动安装, 步骤如下:

- 从[Release Page](https://github.com/thinkgos/goup-rs/releases)下载最新的`goup`.
- 将`goup`可执行文件放到`PATH`中, 并给予可执行权限: `mv GOUP_BIN /usr/local/bin/goup && chmod +x /usr/local/bin/goup`
- 运行`goup init`, 获取到shell启动脚本位于`$HOME/.goup/env`.
- 在shell启动脚本中添加Go的bin目录:
  - bash: `echo '. "$HOME/.goup/env"' >> ~/.bashrc`
  - zsh:  `echo '. "$HOME/.goup/env"' >> ~/.zshenv`
  - fish: `echo 'source ~/.goup/env' >> ~/.config/fish/config.fish`

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
1.21.10  (active, default)
$ go env GOROOT
/home/thinkgo/.goup/current
$ go version
go version go1.21.10 linux/amd64
$ GOUP_GO_REGISTRY_INDEX=https://golang.google.cn goup install =1.21.10
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
1.21.10
1.22.3  (active, default)
tip
```

### 安装指定Go版本

`goup install/update [TOOLCHAIN]`, `[TOOLCHAIN]` 支持的值: **'stable'(default)**, **'nightly'**(**'tip'**, **'gotip'**), **'unstable'**, **'beta'** 或 **'=1.21.4'**, `--dry` 表示只安装对应版本, 但并不切换使用.  

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

### 使用特定的Go版本在shell会话中

`goup shell [VERSION]`, 使用特定的Go版本在shell会话中, 如果没有提供版本, 将提示选择一个已安装的Go版本.

```bash
$ goup shell 1.21.10
? Select a version ›
  1.21.5
❯ 1.21.10
  tip
$ go version
go version go1.21.10 linux/amd64
$ goup list 
1.21.10
1.22.3  (active, default)
tip
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
+------------------------+--------------------------------+-----------------------------------------------------------+
| Key                    | Value                          | Explain                                                   |
+------------------------+--------------------------------+-----------------------------------------------------------+
| GOUP_HOME              | /home/thinkgo/.goup            | Get goup home directory, default: '$HOME/.goup'           |
+------------------------+--------------------------------+-----------------------------------------------------------+
| GOUP_GO_VERSION        | current                        | Shell session target go version, default: 'current'       |
+------------------------+--------------------------------+-----------------------------------------------------------+
| GOUP_GO_REGISTRY_INDEX | https://golang.google.cn       | Registry index of go version                              |
+------------------------+--------------------------------+-----------------------------------------------------------+
| GOUP_GO_REGISTRY       | https://dl.google.com/go       | Registry of go archive file                               |
+------------------------+--------------------------------+-----------------------------------------------------------+
| GOUP_GO_SOURCE_GIT_URL | https://github.com/golang/go   | Source git url, use by tip|nightly or index of go version |
+------------------------+--------------------------------+-----------------------------------------------------------+
| GOUP_GO_SOURCE_GIT_URL | https://go.googlesource.com/go | Source upstream git url, use by tip|nightly               |
+------------------------+--------------------------------+-----------------------------------------------------------+
```

### Shell补全

`goup completion <SHELL>`  为指定shell生成补全脚本. `<SHELL>`支持这些值: `bash`, `elvish`, `fish`, `powershell`, `zsh`.

```bash
goup completion zsh > _goup
```

### 更多信息

执行`goup -h`获取更多信息

## 镜像站

### 索引镜像站

- 官方1(默认): https://golang.google.cn
- 官方2: https://go.dev

### 仓库镜像站

| 仓库 | 地址 | 支持SHA256文件 | 支持HTTP获取压缩包长度 |
|---|---|---|---|---|
| 官方1 | https://dl.google.com/go | ✅ | ✅ |
| 官方2 | https://go.dev/dl | ❌ | ✅ |
| 官方3 | https://golang.org/dl | ❌ | ✅ |
| 阿里云 | https://mirrors.aliyun.com/golang | ❌ | ❌ |
| 南京大学 | https://mirrors.nju.edu.cn/golang | 🔘 | 🔘 |
| 华中科技大学 | https://mirrors.hust.edu.cn/golang | ✅ | ✅ |
| 中国科学技术大学 | https://mirrors.ustc.edu.cn/golang | ✅ | ✅ |

***NOTE***: 镜像站不提供**SHA256校验文件**, 在下载时需要使用`--skip-verify`选项.

### 设置镜像站环境变量

```shell
export GOUP_GO_REGISTRY_INDEX=https://golang.google.cn
export GOUP_GO_REGISTRY=https://mirrors.aliyun.com/golang
```

## 工作原理

- `goup completion <SHELL>` 为指定shell生成补全脚本.
- `goup [help]`  打印此信息或给定子命令的帮助信息.
- `goup install/update/i [TOOLCHAIN]` 下载指定的Go版本到`$HOME/.goup/go<VERSION|tip>/go`并创建一个软链接到`$HOME/.goup/current`.
- `goup use/set [VERSION]` 切换到选择的Go版本.
- `goup ls/list/show` 列出所有位置`$HOME/.goup`已安装的Go版本.
- `goup remove/rm [VERSION]...` 移除指定的Go版本列表.
- `goup search/ls-remote [FILTER]` 列出所有可用的Go版本.
- `goup cache [COMMAND]` 管理缓存归档文件.
- `goup self <COMMAND>` 修改`goup`安装程序.
- `goup init` 将所有必要的环境变量和值写入`$HOME/.goup/env`.
- `goup env`  显示`goup`的环境变量和值.
- `goup shell [VERSION]` 在shell会话中使用特定的Go版本.

## 构建功能标志

- `no-self-update` 关闭自我更新.

## How to Debug

默认日志级别为`Info`. 你可以使用`goup -v <subcommand>` 或 `goup -vv <subcommand>` 来使用 `Debug` 或 `Trace` 等级.

## FAQ

- 编译和安装源代码失败?  
  所需的Go最低版本取决于Go的目标版本, 更多信息请参见[source installation instructions](https://go.dev/doc/install/source)
- [`semver`](https://semver.org/)
  - exact(`=`):  允许更新到与版本完全一致的最新版本, 因此`=1.21.4`表示与版本`1.21.4`完全一致.
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
  看多信息查看issue [#251](https://github.com/thinkgos/goup-rs/issues/251)

- 如何自定义 `GOUP_HOME`? (>= v0.11.x)  
  `goup`使用`$HOME/.goup`目录作为 `GOUP_HOME`. 如果需要自定义`GOUP_HOME`(大多数是Windows用户), 可以设置`GOUP_HOME`环境变量来使用其他目录, 安装`goup`之前, 请确保已设置自定义`GOUP_HOME`环境变量和目标目录权限, 否则可能会导致令人惊讶的结果, 请参阅issue [#265](https://github.com/thinkgos/goup-rs/issues/265) [#270](https://github.com/thinkgos/goup-rs/pull/270)

- 有一些版本没有ha256文件, 如何安装这些版本?
  `goup`(>= v0.12.x) 支持 `--skip-verify` 选项, 如果这些版本没有ha256文件, 你可以尝试添加选项. 请参阅issue [#300](https://github.com/thinkgos/goup-rs/issues/300) [#301](https://github.com/thinkgos/goup-rs/pull/301) [#305](https://github.com/thinkgos/goup-rs/pull/305)

- 如何安装特定版本? 为什么会出现错误`Error: expected comma after minor version number, found 'r'`?
  有时, 我们知道确切的版本, 可以使用 `goup install =1.24.5`, 但有些版本不符合[`semver`](https://semver.org/), 如 `1.25rc1`, 我们可以使用`goup install unstable`, 但这只能安装最新的不稳定版本. 所以我添加了一个 `--use-raw-version` 选项(>= v0.12.x), 这样我们就可以安装任何我们确切知道的版本. 请参阅issue [#299](https://github.com/thinkgos/goup-rs/issues/299) [#307](https://github.com/thinkgos/goup-rs/pull/307)

- 如何在shell会话中使用特定的Go版本?
  `goup`(>= v0.15.x) 支持在一个`shell`会话中指定go版本. 如果你使用`goup shell`, 在`*nix`系统上需要先运行`goup init`, 因为之前的`env`文件较旧且不包含`GOUP_GO_VERSION`环境变量. 在`Windows`系统 上, 仅支持`powershell`, 如果系统的`COMSPEC`已经指向 powershell, 可能无需做任何操作. 请参阅issue [#360](https://github.com/thinkgos/goup-rs/issues/360).

## 许可证

[Apache 2.0](LICENSE)
