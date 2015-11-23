package js.npm;

@:jsRequire("docopt")
extern class Docopt {
	static function docopt(doc:String, args : { ?argv:Array<String>, ?help:Bool, ?version:String, ?options_first:Bool, ?exit:Bool }):haxe.DynamicAccess<Dynamic>;
}

