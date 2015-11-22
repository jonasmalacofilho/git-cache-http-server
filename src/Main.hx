import js.node.*;
import js.node.http.*;

class Main {
	static function parseAuth(s:String)
	{
		var parts = s.split(" ");
		if (parts[0] != "Basic")
			throw "HTTP authentication schemes other than Basic not supported";
		return haxe.crypto.Base64.decode(parts[1]);
	}

	static function getParams(req:IncomingMessage)
	{
		var r = ~/^\/(.+)(.git)?\/(info\/refs\?service=)?(git-[^-]+-pack)$/;
		if (!r.match(req.url))
			throw 'Cannot deal with url';
		return {
			repo : r.matched(1),
			auth : req.headers["authorization"],
			service : r.matched(4),
			isInfoRequest : r.matched(3) != null
		}
	}

	static function clone(remote, local, callback)
	{
		ChildProcess.exec('git clone --quiet --mirror "$remote" "$local"', callback);
	}

	static function fetch(local, callback)
	{
		ChildProcess.exec('git -C "$local" fetch --quiet', callback);
	}

	static function authenticate(params, callback)
	{
		trace("authenticating on the upstream repo");
		var req = Https.request('https://${params.repo}/info/refs?service=${params.service}', callback);
		req.setHeader("User-Agent", "git/");
		if (params.auth != null)
			req.setHeader("Authorization", params.auth);
		req.end();
	}

	static function update(remote, local, callback)
	{
		trace("updating: fetching");
		fetch(local, function (ferr, stdout, stderr) {
			if (ferr != null) {
				trace("updating: fetch failed, cloning");
				clone(remote, local, function (cerr, stdout, stderr) {
					if (cerr != null)
						throw 'git clone exited with non-zero status: ${cerr.code}';
					trace("updating: success");
					callback();
				});
			} else {
				trace("updating: success");
				callback();
			}
		});
	}

	static function handleRequest(req:IncomingMessage, res:ServerResponse)
	{
		try {
			trace('${req.method} ${req.url}');
			var params = getParams(req);

			switch ([req.method == "GET", params.isInfoRequest]) {
			case [false, false], [true, true]:  // ok
			case [m, i]: throw 'isInfoRequest=$i but isPOST=$m';
			}

			if (params.service != "git-upload-pack")
				throw 'Service ${params.service} not supported yet';

			var remote = if (params.auth == null)
				'https://${params.repo}';
			else
				'https://${parseAuth(params.auth)}@${params.repo}';
			var local = Path.join(cacheDir, params.repo);

			authenticate(params, function (upRes) {
				switch (upRes.statusCode) {
				case 401, 403, 404:
					res.writeHead(upRes.statusCode, upRes.headers);
					res.end();
					return;
				case 200:  // ok
				}

				if (params.isInfoRequest) {
					update(remote, local, function () {
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
							trace('${params.service} done with exit $code');
						});
					});
				} else {
					res.statusCode = 200;
					res.setHeader("Content-Type", 'application/x-${params.service}-result');
					res.setHeader("Cache-Control", "no-cache");
					var up = ChildProcess.spawn(params.service, ["--stateless-rpc", local]);
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

	// settings
	static var listenPort = 8080;
	static var cacheDir = "/tmp/var/cache/git/";

	static function main()
	{
		Http.createServer(handleRequest).listen(listenPort);
	}
}

