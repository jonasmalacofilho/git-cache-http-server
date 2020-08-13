# Implementation notes

## References

 - [Pro Git: Git Internals - Transfer Protocols](https://git-scm.com/book/en/v2/Git-Internals-Transfer-Protocols)
 - [Pro Git: Git on the Server - The Protocols](https://git-scm.com/book/en/v2/Git-on-the-Server-The-Protocols)
 - [git Documentation: Git Protocol Capabilities](https://github.com/git/git/blob/master/Documentation/technical/protocol-capabilities.txt)
 - [git Documentation: Git Wire Protocol, Version 2](https://github.com/git/git/blob/master/Documentation/technical/protocol-v2.txt)
 - [git Documentation: HTTP transfer protocols](https://github.com/git/git/blob/master/Documentation/technical/http-protocol.txt)
 - [git Documentation: Packfile transfer protocols](https://github.com/git/git/blob/master/Documentation/technical/pack-protocol.txt)
 - [git: `git-http-backend.c`](https://github.com/git/git/blob/master/http-backend.c)
 - [~~Source code for the GitLab workhorse~~](https://gitlab.com/gitlab-org/gitlab-workhorse/blob/master/handlers.go)

## Cache operations

```
client <- cache <- upstream
client -> (fwd) -> upstream
```

## Cache components

```
git-cache-http-server:
  + single client cache API: transparently (?) cache
  + git smart http server:
    + git smart protocol: implements git-<name>-services
    + http server:
      + server: handle multiple connections
      + http: handle HTTP requests and responses
      + url parsing: parse URLs
```


## Git Smart Protocol

Clone/fetch:
- client runs git-fetch-pack, which connects to git-upload-pack on the server
- GET info/refs?service=git-upload-pack: get refs
- POST git-upload-pack: get data

Push:
- client runs git-send-pack, which connects to git-receive-pack on the server
- GET info/refs?service=git-receive-pack: get refs
- POST git-receive-pack: send data

Headers:
- never cache
- valid responses are: 200 (ok), 404 (not found), 410 (gone), 304 (not modified) and 403 (forbidden)
- response content-type must be: application/x-$servicename-advertisement

Protocol versions:
- version 2 exists: if supported by libgit2

Capabilities:
- of interest: side-band-64k (to multiplex progress information)
- server advertises
- client puts them into effect
- may require conversions
