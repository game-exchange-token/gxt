using Gxt;
using Gxt.Advisory;

var alice = GxtSdk.MakeKey();
var bob = GxtSdk.MakeKey();

Meta meta;
meta.name = "Bob";

var bobsIdCard = GxtSdk.MakeIdCard(bob, meta);
var vEnvelope1 = GxtSdk.VerifyMessage<Meta>(bobsIdCard);
Console.WriteLine("Bobs ID Card:");
Console.WriteLine(Newtonsoft.Json.JsonConvert.SerializeObject(vEnvelope1, Newtonsoft.Json.Formatting.Indented));
Console.WriteLine();

var requests = new List<TradeRequest>
{
    new TradeRequest
    {
        Id = "cf2c7f92-149f-4224-b176-18c7cd0c51d5",
        Wanted = [
            new()
            {
                Id = "sword",
                Amount = 1,
                Description = "Firey fire sword of fire damage",
                DisplayName = "Fire Sword",
                Attributes = new List<AttributeModifier>()
                {
                    new AttributeModifier
                    {
                        Id = "fire_damage",
                        DisplayName = "Fire Damage",
                        Amount = 10,
                        Kind = ModifierKind.Percent,
                    }
                }
            }
        ],
        Offered = [
            new() {
                Id = "gold",
                Amount = 100,
                Attributes = []
            }
        ]
    }
};

var order = new TradeOrder { Requests = requests, AllOrNothing = false };


var encryptedMessage = GxtSdk.EncryptMessage(alice, bobsIdCard, order);
Console.WriteLine("Encrypted Message for Bob:");
Console.WriteLine(encryptedMessage);
Console.WriteLine();

var envelope = GxtSdk.DecryptMessage<TradeOrder>(encryptedMessage, bob);
Console.WriteLine("Decrypted Message for Bob:");
Console.WriteLine(Newtonsoft.Json.JsonConvert.SerializeObject(envelope, Newtonsoft.Json.Formatting.Indented)); ;
Console.WriteLine();

struct Meta
{
    public string name;
}

struct Payload
{
    public string hello;
}
