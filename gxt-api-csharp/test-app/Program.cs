using gxt_csharp;

var alice = Gxt.MakeKey();
var bob = Gxt.MakeKey();

Meta meta;
meta.name = "Bob";

var bobsIdCard = Gxt.MakeIdCard(bob, meta);
var vEnvelope1 = Gxt.VerifyMessage<Meta>(bobsIdCard);
Console.WriteLine("Bobs ID Card:");
Console.WriteLine(Newtonsoft.Json.JsonConvert.SerializeObject(vEnvelope1, Newtonsoft.Json.Formatting.Indented));
Console.WriteLine();

var encryptedMessage = Gxt.EncryptMessage(alice, bobsIdCard, new Body { hello = "world" });
Console.WriteLine("Encrypted Message for Bob:");
Console.WriteLine(encryptedMessage);
Console.WriteLine();

var envelope = Gxt.DecryptMessage<Body>(encryptedMessage, bob);
Console.WriteLine("Decrypted Message for Bob:");
Console.WriteLine(Newtonsoft.Json.JsonConvert.SerializeObject(envelope, Newtonsoft.Json.Formatting.Indented)); ;
Console.WriteLine();

struct Meta
{
    public string name;
}

struct Body
{
    public string hello;
}
