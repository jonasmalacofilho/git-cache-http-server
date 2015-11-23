A caching Git HTTP server
============================

Serve local mirror repositories over HTTP, automatically updating them as needed.

Features:

 - supports fetch/clone
 - enforced authentication to upstream before allowing any request
   - supports `Basic` HTTP authentication
 - supports HTTP (and HTTPS if installed behind Nginx or some other HTTP proxy server)

