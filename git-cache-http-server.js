var __http = require("http");
var __child_process = require("child_process");

var listen = 8080;
var cacheLocation = "/tmp/var/cache/git/";

var upPat = /^\/(.+)(.git)?\/info\/refs\?service=git-upload-pack$/;

__http.createServer(function (req, res) {
	console.log(req.method + " " + req.url);

	var isUp = upPat.exec(req.url);
	if (isUp) {
		var remote = isUp[1];
		// TODO don't allow arbitrary paths to be constructed with ".."
		var local = cacheLocation + remote + ".git";
		console.log("remote: " + remote);
		console.log("local: " + local);

		// TODO check our cache
		// TODO fetch or clone
		
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
			if (code != 0) {
				console.log("git-upload-pack: exit code: " + code);
				res.end();
			}
		});
	} else {
		console.log("FUCKK!K!!!");
		res.writeHead(500);
		res.end();
	}
}).listen(listen);

