using Gxt;
using Gxt.Advisory;
using Newtonsoft.Json;

var alice = GxtSdk.MakeKey();
var bob = GxtSdk.MakeKey();

var idCard = new IdCard { DisplayName = "Bob" };

var bobsIdCard = GxtSdk.MakeIdCard(bob, idCard);
var vEnvelope1 = GxtSdk.VerifyMessage<IdCard>(bobsIdCard);
Console.WriteLine("Bobs ID Card:");
Console.WriteLine(JsonConvert.SerializeObject(vEnvelope1, Formatting.Indented));
Console.WriteLine();

var requests = new List<TradeRequest>
{
    new TradeRequest
    {
        Id = "cf2c7f92-149f-4224-b176-18c7cd0c51d5",
        Wanted = [
            new()
            {
                Id = "weapons.swords.fire_sword",
                Amount = 1,
                Description = "Fiery fire sword of fire damage",
                DisplayName = "Fire Sword",
                Attributes = new List<AttributeModifier>()
                {
                    new AttributeModifier
                    {
                        Id = "damage_types.fire",
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
Console.WriteLine(JsonConvert.SerializeObject(envelope, Formatting.Indented)); ;
Console.WriteLine();
