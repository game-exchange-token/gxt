using Newtonsoft.Json;


namespace Gxt.Advisory
{
    using Newtonsoft.Json.Converters;
    using System.Collections.Generic;


    /// <summary>
    /// Represents a trade order consisting of multiple trade requests.
    /// </summary>
    public class TradeOrder
    {
        /// <summary>
        /// The trade requests contained in this order.
        /// </summary>
        required public List<TradeRequest> Requests { get; set; } = new();

        /// <summary>
        /// Whether all requests must be fulfilled together.
        /// </summary>
        required public bool AllOrNothing { get; set; }

        /// <summary>
        /// Optional note for the trade order.
        /// </summary>
        public string? Note { get; set; }
    }

    /// <summary>
    /// Represents the response to a trade order.
    /// </summary>
    public class TradeResponse
    {
        /// <summary>
        /// The original trade order.
        /// </summary>
        required public TradeOrder Order { get; set; }

        /// <summary>
        /// The trade requests that were executed.
        /// </summary>
        required public List<TradeRequest> Trades { get; set; } = new();

        /// <summary>
        /// Optional note explaining the response.
        /// </summary>
        public string? Note { get; set; }
    }

    /// <summary>
    /// Represents a single trade request, with the wanted and offered items.
    /// </summary>
    public class TradeRequest
    {
        /// <summary>
        /// A unique identifier of a trade request.
        /// This makes it easier to match fulfillments to requests.
        /// </summary>
        required public string Id { get; set; }

        /// <summary>
        /// The wanted items.
        /// </summary>
        required public List<Item> Wanted { get; set; } = new();

        /// <summary>
        /// The items offered for fulfilling the trade.
        /// </summary>
        required public List<Item> Offered { get; set; } = new();

        /// <summary>
        /// Optional opaque data specific to the game.
        /// </summary>
        public System.Text.Json.Nodes.JsonNode? Data { get; set; }
    }

    /// <summary>
    /// A tradable item, such as gold, equipment or consumables.
    /// </summary>
    public class Item
    {
        /// <summary>
        /// Identifier for the item in the game.
        /// </summary>
        required public string Id { get; set; }

        /// <summary>
        /// The name of the item that should be shown to the player.
        /// </summary>
        public string? DisplayName { get; set; }

        /// <summary>
        /// Description of the item.
        /// </summary>
        public string? Description { get; set; }

        /// <summary>
        /// The attributes of the item.
        /// </summary>
        public List<AttributeModifier> Attributes { get; set; } = new();

        /// <summary>
        /// Quantity of the item.
        /// </summary>
        required public uint Amount { get; set; }

        /// <summary>
        /// Optional opaque data specific to the game.
        /// </summary>
        public System.Text.Json.Nodes.JsonNode? Data { get; set; }
    }

    /// <summary>
    /// An attribute that is changed by using or equipping the item.
    /// </summary>
    public class AttributeModifier
    {
        /// <summary>
        /// Identifier for the Attribute in the game.
        /// </summary>
        required public string Id { get; set; }

        /// <summary>
        /// The name of the attribute that should be shown to the player.
        /// </summary>
       public string? DisplayName { get; set; }

        /// <summary>
        /// Amount change for the attribute.
        /// </summary>
        required public int Amount { get; set; }

        /// <summary>
        /// How the amount should be applied.
        /// </summary>
        required public ModifierKind Kind { get; set; }

        /// <summary>
        /// Optional opaque data specific to the game.
        /// </summary>
        public System.Text.Json.Nodes.JsonNode? Data { get; set; }
    }

    /// <summary>
    /// What kind of attribute modifier it is.
    /// </summary>
    [JsonConverter(typeof(StringEnumConverter))]
    public enum ModifierKind
    {
        /// <summary>
        /// Flat increase.
        /// </summary>
        Flat,

        /// <summary>
        /// Percent increase.
        /// </summary>
        Percent
    }

}
