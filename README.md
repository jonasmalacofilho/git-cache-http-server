git-cache-http-server – cache remote repositories and serve them over HTTP
==========================================================================

Currently supported client operations are fetch and clone.  Authentication to
the upstream repository is always enforced (for now, only HTTP Basic is
supported), but public repositories can be used as well.

# Usage

```
Usage:
  git-cache-http-server.js [options]

Options:
  -c,--cache-dir <path>   Location of the git cache [default: /var/cache/git]
  -p,--port <port>        Bind to port [default: 8080]
  -h,--help               Print this message
  --version               Print the current version
```

The upstream remote is extracted from the URL, taking the first component as
the remote hostname.

Example:

```
git-cache-http-server --port 1234 --cache-dir /tmp/cache/git &
git clone http://localhost:1234/github.com/jonasmalacofilho/git-cache-http-server
```

If you run your git-cache on a dedicated server or container (i.e. named
gitcache), you can then also configure git to always use your cache like in the
following example (don't use this configuration on the git-cache machine
itself):.

```
git config --global url."http://gitcache:1234/".insteadOf https://
```

# Installing

The only runtime dependency are the official `git` executables.

Some Linux distributions may have a package for this.  Otherwise installing
from sources is easy, just follow the steps in the [working with the Rust
sources](#working-with-the-rust-sources) section.

```
$ # clone the repository (adjust the protocol)
$ git clone https://github.com/jonasmalacofilho/git-cache-http-server

$ # build and install
$ cargo install --release

$ git-cache-http-server --version
git-cache-http-server 0.1.0
```

To install a cache service on Linux systems, check the example
`doc/git-cache-http-server.service` unit file.

For Systemd init users that file should not require major tweaks, other than
specifying a different than default port number or cache directory.  After
installed in the proper Systemd unit path for your distribution:

```
systemctl daemon-reload
systemctl start git-cache-http-server
systemctl enable git-cache-http-server
```

# Working with the Rust sources

Building and installing the software from sources allows you access to the
latest features and fixes and the ability to make changes as well.

As with most Rust projects, `cargo` is used to manage the build, testing and
installing the software.  Make sure you have a Rust (stable) environment, and
`cargo`.

```
$ # clone the repository (adjust the protocol)
$ git clone https://github.com/jonasmalacofilho/git-cache-http-server

$ # build or run locally with debug information
$ cargo build
$ cargo run -- --version

$ # run the test suite
$ cargo test

$ # install globally on the OS (rebuilds in a separate temporary directory)
$ cargo install --release
$ git-cache-http-server 0.1.0
```

# Implementation

The current implementation is somewhat oversimplified; any help in improving it
is greatly appreciated!

References:

 - [Transfer protocols on the Git Book](http://git-scm.com/book/en/v2/Git-Internals-Transfer-Protocols)
 - [Git documentation on the HTTP transfer protocols](https://github.com/git/git/blob/master/Documentation/technical/http-protocol.txt)
 - [Source code for `git-http-backend`](https://github.com/git/git/blob/master/http-backend.c)
 - [~~Source code for the GitLab workhorse~~](https://gitlab.com/gitlab-org/gitlab-workhorse/blob/master/handlers.go)
