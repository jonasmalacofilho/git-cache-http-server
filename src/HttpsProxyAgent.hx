import js.node.https.Agent;

@:jsRequire('http-proxy-agent')
extern class HttpsProxyAgent extends js.node.https.Agent {
    function new(proxy:String);
}