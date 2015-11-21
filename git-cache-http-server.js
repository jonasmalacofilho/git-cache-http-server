var __child_process = require("child_process");
var __fs = require("fs");
var __http = require("http");
var __mkdir_p = require("mkdir-p");

var listen = 8080;
var cacheLocation = "/tmp/var/cache/git/";

var pat = /^\/(.+)(.git)?\/(info\/refs\?service=)?git-upload-pack$/;

function err(msg, code, res)
{
	console.log("ERROR: " + msg);
	res.writeHead(code);
	res.end();
}

__http.createServer(function (req, res) {
	console.log(req.method + " " + req.url);
	var parts = pat.exec(req.url);

	if (!parts)
		return err("invalid url: " + req.url, 404, res);
	if (parts[3] != null && req.method != "GET")
		return err("first request must use GET", 404, res);
	if (parts[3] == null && req.method != "POST")
		return err( "second request must use POST", 404, res);

	var remote = parts[1];
	// TODO don't allow arbitrary paths to be constructed with ".."
	var local = cacheLocation + remote + ".git/";
	console.log("remote: " + remote);
	console.log("local: " + local);

	if (parts[3] != null) {
		__fs.stat(local + "objects", function (err, stats) {
			// fetch or clone
			if (err) {
				console.log("mirror: clonning");
				__mkdir_p.sync(local);
				__child_process.execSync("git clone --quiet --mirror https://" + remote + " " + local);
			} else {
				console.log("mirror: fetching");
				__child_process.execSync("git -C " + local + " fetch --quiet --force");
			}
			// respond
			res.statusCode = 200;
			res.setHeader("Content-Type", "application/x-git-upload-pack-advertisement");
			res.setHeader("Cache-Control", "no-cache");
			res.write("001e# service=git-upload-pack\n0000");
			var up = __child_process.spawn("git-upload-pack", ["--stateless-rpc", "--advertise-refs", local]);
			up.stdout.pipe(res);
			up.stderr.on("data", function (data) {
				console.log("git-upload-pack: stderr: " + data);
			});
			up.on("exit", function (code) {
				console.log("git-upload-pack: exit code: " + code);
				if (code != 0) {
					res.end();
				}
			});
		});
	} else {
		res.statusCode = 200;
		res.setHeader("Content-Type", "application/x-git-upload-pack-result");
		res.setHeader("Cache-Control", "no-cache");
		var up = __child_process.spawn("git-upload-pack", ["--stateless-rpc", local]);
		req.pipe(up.stdin);
		up.stdout.pipe(res);
		up.stderr.on("data", function (data) {
			console.log("git-upload-pack: stderr: " + data);
		});
		up.on("exit", function (code) {
			console.log("git-upload-pack: exit code: " + code);
			if (code != 0) {
				console.log("git-upload-pack: exit code: " + code);
				res.end();
			}
		});
	}
}).listen(listen);

