import haxe.macro.Compiler;
import haxe.macro.Context;
import sys.io.File;

class BuildUtils {
	public static function addShebang(interpreter:String, ?arg:String) {
		// implementation borrowed from travix
		//	https://github.com/back2dos/travix
		// licensed under the The Unlicense
		Context.onAfterGenerate(function() {
			var out = Compiler.getOutput();
			File.saveContent(out, '#!$interpreter $arg\n\n' + File.getContent(out));
		});
	}

	public static function makeExecutable()
	{
		if (Sys.systemName() == "Windows")
			return;
		Context.onAfterGenerate(function() {
			var out = Compiler.getOutput();
			if (Sys.command("chmod", ["+x", out]) != 0)
				Context.error("chmod +x failed", Context.currentPos());
		});
	}
}

