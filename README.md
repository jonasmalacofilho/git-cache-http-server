A caching Git HTTP server
============================

Mirror remote repositories and serve them over HTTP, automatically updating
them as needed.

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

Requirements: `nodejs` and `git`.

Install: `npm install --global git-cache-http-server`.

To install as a service, check the `doc/git-cache-http-server.service` example
service file.

For Systemd init users, this file should not require major tweaks other than
specifying a different than default port number or cache directory.  After
installed in the proper Systemd unit path for your distribution, issue:

```
systemctl daemon-reload
systemctl start git-cache-http-server
```

# Working on the sources

This is only needed if you want to change the Haxe source code in `src/`.

Requirements: [Haxe](https://haxe.org) (`haxe` and `haxelib`).  If you prefer
to manage the build dependencies manually, check out [`build.hxml`](build.hxml)
for the required libraries.

```
haxelib newrepo
haxelib install build.hxml
haxe build.hxml
```

And in order to use the locally modified version: `npm link` (this changes the
global NPM space).

# Implementation

The current implementation is somewhat oversimplified; any help in improving it
is greatly appreciated!

References:

 - [Transfer protocols on the Git Book](http://git-scm.com/book/en/v2/Git-Internals-Transfer-Protocols)
 - [Git documentation on the HTTP transfer protocols](https://github.com/git/git/blob/master/Documentation/technical/http-protocol.txt)
 - [Source code for the GitLab workhorse](https://gitlab.com/gitlab-org/gitlab-workhorse/blob/master/handlers.go)
 - [Source code for `git-http-backend`](https://github.com/git/git/blob/master/http-backend.c)
