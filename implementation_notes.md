# Implementation notes

## References

 - [Pro Git: Git Internals - Transfer Protocols](https://git-scm.com/book/en/v2/Git-Internals-Transfer-Protocols)
 - [Pro Git: Git on the Server - The Protocols](https://git-scm.com/book/en/v2/Git-on-the-Server-The-Protocols)
 - [git Documentation: Git Protocol Capabilities](https://github.com/git/git/blob/master/Documentation/technical/protocol-capabilities.txt)
 - [git Documentation: Git Wire Protocol, Version 2](https://github.com/git/git/blob/master/Documentation/technical/protocol-v2.txt)
 - [git Documentation: HTTP transfer protocols](https://github.com/git/git/blob/master/Documentation/technical/http-protocol.txt)
 - [git Documentation: Packfile transfer protocols](https://github.com/git/git/blob/master/Documentation/technical/pack-protocol.txt)
 - [git: `git-http-backend.c`](https://github.com/git/git/blob/master/http-backend.c)
 - [GitLab Architecture Overview: Git Request Cycle](https://gitlab.com/gitlab-org/gitlab/-/blob/9e404d35ecca9e8afae2c844ad45261e81972eb2/doc/development/architecture.md#gitlab-git-request-cycle)
 - [GitLab Gitaly: `internal/service/smarthttp/*.go`](https://gitlab.com/gitlab-org/gitaly/-/tree/19e2caa3a8a9fe390b568dd8d2b2a565be6094a7/internal/service/smarthttp)

## Cache operations

```
          ______ api ________
client <- (cache <- upstream)
client -> ((fwd) -> upstream)
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

## Use case

```
             /github.com/foo/bar                       /foo/bar
client: GET, upstream_spec, credentials -> cache: GET, upstream_url, credentials;
						  "pipe" response to client
client: POST, upstream_spec, credentials -> cache: POST, upstream_url, credentials (update myself);
						   "tee" stdin/stdout `git-upload-pack` to client socket (serve)

let client: TcpStream = ...;
let upstream_url = ...;
let credentials = ...;

update(local_path, upstream_url, credentials) {
	exec_with(local_path, credentials, "git fetch https://github.com/jonasmalacofilho/git-cache-http-server '+refs/*:refs/*'")
}

get_info_refs_to_client(client, ...) {
	update(local_path, upstream_url, credentials); // git fetch <url with credentials> or Authorization: Basic ...
	let local_path = format!("/var/cache/git/{}", upstream_url);
	tee_git_service("git-upload-pack", &client, infos=true); // <cache> <-> <client socket>
	client.flush();
}

tee_upload_service_to_client(client, ...) {
	let local_path = format!("/var/cache/git/{}", upstream_url);
	tee_git_service("git-upload-pack", &client, infos=false); // <cache> <-> <client socket>
	client.flush();
}

// stuff to deal with:
// progress (inserts stuff in the stream)
// capabilities (modify stream to adjust capabilities)

let repository = cache_clone#1("github.com/jonasmalacofilho/git-cache-http-server", credentials); // cache: clones
let repository = cache_clone#2("github.com/jonasmalacofilho/git-cache-http-server", credentials); // cache: fetch

cache_fetch#1(&mut repository, "github.com/jonasmalacofilho/git-cache-http-server", credentials); // cache: fetch
cache_fetch#2(&mut repository, "github.com/jonasmalacofilho/git-cache-http-server", credentials); // cache: fetch
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

## Tricky bits

- document that oauth token should not omit the username: because TODO
