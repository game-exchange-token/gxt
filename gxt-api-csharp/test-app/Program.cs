using gxt_api_csharp;

var alice = Gxt.MakeKey();
var bob = Gxt.MakeKey();

Meta meta;
meta.name = "Bob";

var bobsIdCard = Gxt.MakeIdCard(bob, meta);
var vEnvelope1 = Gxt.VerifyMessage<Meta>(bobsIdCard);
Console.WriteLine(Newtonsoft.Json.JsonConvert.SerializeObject(vEnvelope1, Newtonsoft.Json.Formatting.Indented));

var encryptedMessage = Gxt.EncryptMessage(alice, bobsIdCard, new Body { hello = "world" });
var vEnvelope2 = Gxt.VerifyMessage<Meta>(encryptedMessage);
Console.WriteLine(Newtonsoft.Json.JsonConvert.SerializeObject(vEnvelope2, Newtonsoft.Json.Formatting.Indented));

var envelope = Gxt.DecryptMessage<Body>(encryptedMessage, bob);
Console.WriteLine(Newtonsoft.Json.JsonConvert.SerializeObject(envelope, Newtonsoft.Json.Formatting.Indented)); ;

struct Meta
{
    public string name;
}

struct Body
{
    public string hello;
}
