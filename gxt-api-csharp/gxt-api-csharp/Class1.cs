using System.Reflection;
using System.Runtime.Serialization;
using Extism.Sdk;
using Newtonsoft.Json;

namespace gxt_csharp
{
    internal static class GxtWasm
    {
        internal static Plugin plugin;
        static GxtWasm()
        {
            byte[] buffer;
            using (var s = Assembly.GetExecutingAssembly().GetManifestResourceStream("gxt.wasm")!)
            {
                buffer = new byte[(int)s.Length];
                s.Read(buffer, 0, (int)s.Length);
            }
            var manifest = new Manifest(new ByteArrayWasmSource(buffer, "gxt.wasm"));
            plugin = new Plugin(manifest, [], true);
        }

        internal static string? Call(string functionName, string input, CancellationToken? cancellationToken = null)
        {
            return plugin.Call(functionName, input, cancellationToken);
        }
    }

    struct IdCardRequest<T>
    {
        public string key;
        public T meta;
    }

    struct EncryptRequest<T>
    {
        public string key;
        public string id_card;
        public T payload;
        public string? parent;
    }

    struct DecryptRequest
    {
        public string message;
        public string key;
    }

    public enum PayloadKind
    {
        [EnumMember(Value = "id")]
        Id,
        [EnumMember(Value = "msg")]
        Msg,
    }

    public struct Envelope<T>
    {
        public byte version;
        public string verification_key;
        public string encryption_key;
        public PayloadKind kind;
        public T payload;
        public string? parent;
        public string id;
        public string signature;
    }

    public static class Gxt
    {
        public static string MakeKey()
        {
            return GxtWasm.Call("make_key", "")!;
        }

        public static string MakeIdCard<T>(string key, T meta)
        {
            var req = new IdCardRequest<T> { key = key, meta = meta };
            return GxtWasm.Call("make_id_card", JsonConvert.SerializeObject(req))!;
        }

        public static Envelope<T> VerifyMessage<T>(string message)
        {
            return JsonConvert.DeserializeObject<Envelope<T>>(GxtWasm.Call("verify_message", message)!);
        }

        public static string EncryptMessage<T>(string key, string id_card, T payload, string? parent = null)
        {
            var req = new EncryptRequest<T> { key = key, id_card = id_card, payload = payload, parent = parent };
            return GxtWasm.Call("encrypt_message", JsonConvert.SerializeObject(req))!;
        }

        public static Envelope<T> DecryptMessage<T>(string message, string key)
        {
            var req = new DecryptRequest { message = message, key = key };
            return JsonConvert.DeserializeObject<Envelope<T>>(GxtWasm.Call("decrypt_message", JsonConvert.SerializeObject(req))!);
        }
    }
}
