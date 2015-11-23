A caching Git HTTP server
============================

Serve local mirror repositories over HTTP, automatically updating them as needed.

Features:

 - fetch & clone through the "smart" HTTP transfer protocol
 - automatic cloning and syncing of the mirror
 - enforced authentication to upstream before allowing any request
   - public repositories
   - Basic HTTP authentication

References:

 - [Transfer protocols on the Git Book](http://git-scm.com/book/en/v2/Git-Internals-Transfer-Protocols)
 - [Git documentation on the HTTP transfer protocols](https://github.com/git/git/blob/master/Documentation/technical/http-protocol.txt)
 - [Source code for the GitLab workhorse](https://gitlab.com/gitlab-org/gitlab-workhorse/blob/master/handlers.go)
 - [Source code for `git-http-backend`](https://github.com/git/git/blob/master/http-backend.c)

