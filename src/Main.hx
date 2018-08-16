import js.node.*;
import js.node.http.*;
import js.Promise;

class Main {
	static function safeUser(basic:String)
	{
		var basic = basic.split(":");
		if (basic.length != 2)
			throw "ERR: invalid Basic HTTP authentication";
		var user = basic[0];
		var pwd = basic[1];
		if ((user == pwd || pwd == "" || ~/oauth/.match(pwd)) && user.length > 5)
			user = user.substr(0, 5) + "...";
		return user;
	}

	static function parseAuth(s:String)
	{
		if (s == null)
			return null;
		var parts = s.split(" ");
		if (parts[0] != "Basic")
			throw "ERR: HTTP authentication schemes other than Basic not supported";
		return {
			authorization : s,
			basic : haxe.crypto.Base64.decode(parts[1]).toString()
		}
	}

	static function getParams(req:IncomingMessage)
	{
		var r = ~/^\/(.+)(.git)?\/(info\/refs\?service=)?(git-[^-]+-pack)$/;
		if (!r.match(req.url))
			throw 'Cannot deal with url';
		return {
			repo : r.matched(1),
			auth : parseAuth(req.headers["authorization"]),
			service : r.matched(4),
			isInfoRequest : r.matched(3) != null
		}
	}

	static function clone(remote, local, callback)
	{
		ChildProcess.exec('git clone --quiet --mirror "$remote" "$local"', callback);
	}

	static function fetch(remote, local, callback)
	{
		ChildProcess.exec('git -C "$local" remote set-url origin "$remote"', function(err, stdout, stderr) {
			ChildProcess.exec('git -C "$local" fetch --quiet', callback);
		});
	}

	static function authenticate(params, infos, callback)
	{
		trace('INFO: authenticating on the upstream repo $infos');
		var req:ClientRequest;
		if (proxyAgent == null) {
			req = Https.request('https://${params.repo}/info/refs?service=${params.service}', callback);
		}
		else {
			var opts:Https.HttpsRequestOptions = {};
			opts.protocol = "https:";
			opts.host = params.repo;
			opts.path = '/info/refs?service=${params.service}';
			opts.agent = proxyAgent;
			req = Https.request(opts, callback);
		}

		req.setHeader("User-Agent", "git/");
		if (params.auth != null)
			req.setHeader("Authorization", params.auth.authorization);
		req.end();
	}

	static function update(remote, local, infos, callback)
	{
		if (!updatePromises.exists(local)) {
			updatePromises[local] = new Promise(function(resolve, reject) {
				trace('INFO: updating: fetching from $infos');
				fetch(remote, local, function (ferr, stdout, stderr) {
					if (ferr != null) {
						trace("WARN: updating: fetch failed");
						trace(stdout);
						trace(stderr);
						trace("WARN: continuing with clone");
						clone(remote, local, function (cerr, stdout, stderr) {
							if (cerr != null) {
								trace(stdout);
								trace(stderr);
								resolve('ERR: git clone exited with non-zero status: ${cerr.code}');
							} else {
								trace("INFO: updating via clone: success");
								resolve(null);
							}
						});
					} else {
						trace("INFO: updating via fetch: success");
						resolve(null);
					}
				});
			})
			.then(function(success) {
				updatePromises.remove(local);
				return Promise.resolve(success);
			})
			.catchError(function(err) {
				updatePromises.remove(local);
				return Promise.reject(err);
			});
		} else {
			trace("INFO: reusing existing promise");
		}
		return updatePromises[local]
		.then(function(nothing:Dynamic) {
			trace("INFO: promise fulfilled");
			callback(null);
		}, function(err:Dynamic) {
			callback(err);
		});
	}

	static function handleRequest(req:IncomingMessage, res:ServerResponse)
	{
		try {
			trace('${req.method} ${req.url}');
			var params = getParams(req);
			var infos = '${params.repo}';
			if (params.auth != null)
				infos += ' (user ${safeUser(params.auth.basic)})';

			switch ([req.method == "GET", params.isInfoRequest]) {
			case [false, false], [true, true]:  // ok
			case [m, i]: throw 'isInfoRequest=$i but isPOST=$m';
			}

			if (params.service != "git-upload-pack")
				throw 'Service ${params.service} not supported yet';

			var remote = if (params.auth == null)
				'https://${params.repo}';
			else
				'https://${params.auth.basic}@${params.repo}';
			var local = Path.join(cacheDir, params.repo);

			authenticate(params, infos, function (upRes) {
				switch (upRes.statusCode) {
				case 401, 403, 404:
					res.writeHead(upRes.statusCode, upRes.headers);
					res.end();
					return;
				case 200:  // ok
				}

				if (params.isInfoRequest) {
					update(remote, local, infos, function (err) {
						if (err != null) {
							trace('ERR: $err');
							trace(haxe.CallStack.toString(haxe.CallStack.exceptionStack()));
							res.statusCode = 500;
							res.end();
							return;
						}
						res.statusCode = 200;
						res.setHeader("Content-Type", 'application/x-${params.service}-advertisement');
						res.setHeader("Cache-Control", "no-cache");
						res.write("001e# service=git-upload-pack\n0000");
						var up = ChildProcess.spawn(params.service, ["--stateless-rpc", "--advertise-refs", local]);
						up.stdout.pipe(res);
						up.stderr.on("data", function (data) trace('${params.service} stderr: $data'));
						up.on("exit", function (code) {
							if (code != 0)
								res.end();
							trace('INFO: ${params.service} done with exit $code');
						});
					});
				} else {
					res.statusCode = 200;
					res.setHeader("Content-Type", 'application/x-${params.service}-result');
					res.setHeader("Cache-Control", "no-cache");
					var up = ChildProcess.spawn(params.service, ["--stateless-rpc", local]);
                                        // If we receive gzip content, we must unzip
                                        if (req.headers['content-encoding'] == 'gzip')
                                                req.pipe(Zlib.createUnzip()).pipe(up.stdin);
                                        else
                                                req.pipe(up.stdin);
					up.stdout.pipe(res);
					up.stderr.on("data", function (data) trace('${params.service} stderr: $data'));
					up.on("exit", function (code) {
						if (code != 0)
							res.end();
						trace('${params.service} done with exit $code');
					});
				}
			});
		} catch (err:Dynamic) {
			trace('ERROR: $err');
			trace(haxe.CallStack.toString(haxe.CallStack.exceptionStack()));
			res.statusCode = 500;
			res.end();
		}
	}

	static var updatePromises = new Map<String, Promise<Dynamic>>();
	static var cacheDir = "/tmp/var/cache/git/";
	static var listenPort = 8080;
	static var proxyAgent = null;
	static var usage = "
A caching Git HTTP server.

Serve local mirror repositories over HTTP/HTTPS, updating them as they are requested.

Usage:
  git-cache-http-server.js [options]

Options:
  -c,--cache-dir <path>   Location of the git cache [default: /var/cache/git]
  -p,--port <port>        Bind to port [default: 8080]
  -h,--help               Print this message
  --version               Print the current version
";

	static function main()
	{
		var options = js.npm.Docopt.docopt(usage, { version : Version.readPkg() });
		cacheDir = options["--cache-dir"];
		listenPort = Std.parseInt(options["--port"]);
		if (listenPort == null || listenPort < 1 || listenPort > 65535)
			throw 'Invalid port number: ${options["--port"]}';

		trace('INFO: cache directory: $cacheDir');
		trace('INFO: listening to port: $listenPort');

		var env = Sys.environment();
		var proxyUrl = env["http_proxy"];
		if (proxyUrl == null)
			proxyUrl = env["HTTP_PROXY"];
		if (proxyUrl != null)
			proxyAgent = new HttpsProxyAgent(proxyUrl);

		var server = Http.createServer(handleRequest);
		server.setTimeout(120*60*1000); // 120 * 60 seconds * 1000 msecs
		server.listen(listenPort);
	}
}
