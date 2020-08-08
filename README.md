# A caching Git HTTP server

Cache remote repositories and serve them over HTTP.

~~Currently supported client operations are fetch and clone, and authentication to
the upstream repository is always enforced.~~

## Starting the caching server

```
$ git-cache-http-server --help
Usage:
  git-cache-http-server.js [options]

Options:
  -c,--cache-dir <path>   Location of the git cache [default: /var/cache/git]
  -p,--port <port>        Bind to port [default: 8080]
  -h,--help               Print this message
  --version               Print the current version

$ git-cache-http-server
```

## Using from git clients

The general idea is that instead of the upstream URL, the client should be
configured to use a corresponding URL on the caching server.

The server takes the upstream remote from the URL, taking the first component as
the remote host, and the rest as the path to the specific repository.

For example, assuming a local cache server at `localhost:8080`, cloning this
repository would require:

```
git clone http://localhost:8080/github.com/jonasmalacofilho/git-cache-http-server
```

If you the cache server runs on a dedicated box or container, it is also
possible to configure git to always use the cache for any HTTPS remote (but do
not use this on the system that hosts the caching server itself):

```
git config --global url."http://git-cache:8080/".insteadOf https://
```

## Installing

To be written.

## Working with the Rust sources

To be written.

## Implementation

Useful references:

 - [Pro Git: Git on the Server - The Protocols](https://git-scm.com/book/en/v2/Git-on-the-Server-The-Protocols)
 - [Pro Git: Git Internals - Transfer Protocols](http://git-scm.com/book/en/v2/Git-Internals-Transfer-Protocols)
 - [git Documentation: HTTP transfer protocols](https://github.com/git/git/blob/master/Documentation/technical/http-protocol.txt)
 - [git: `git-http-backend.c`](https://github.com/git/git/blob/master/http-backend.c)
 - [~~Source code for the GitLab workhorse~~](https://gitlab.com/gitlab-org/gitlab-workhorse/blob/master/handlers.go)
