class Version {
	public static macro function readPkg()
	{
		var f = sys.io.File.getContent("package.json");
		var pkg = haxe.Json.parse(f);
		return macro $v{(pkg.version:String)};
	}
}

