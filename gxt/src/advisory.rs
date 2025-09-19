use serde::{Deserialize, Serialize};

/// Simple meta data for an ID card.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct IdCard {
    /// The name the player wants to be displayed as.
    pub display_name: String,
    /// Optional opaque data specific to the game.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<OpaqueData>,
}

/// Type alias for generic data. This can be used for minor extensions of the data.
pub type OpaqueData = serde_json::Value;

/// Represents a trade order consisting of multiple trade requests.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct TradeOrder {
    /// The trade requests contained in this order.
    pub requests: Vec<TradeRequest>,
    /// Whether all requests must be fulfilled together.
    pub all_or_nothing: bool,
    /// Optional note for the trade order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Represents the response to a trade order.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct TradeResponse {
    /// The original trade order.
    pub order: TradeOrder,
    /// The trade requests that were executed.
    pub trades: Vec<TradeRequest>,
    /// Optional note explaining the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}
/// Represents a single trade request, with the wanted and offered items.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct TradeRequest {
    /// A unique identifier of a trade request.
    /// This makes it easier to match fulfillments to requests.
    pub id: String,
    /// The wanted items.
    pub wanted: Vec<Item>,
    /// The items offered for fulfilling the trade.
    pub offered: Vec<Item>,
    /// Optional opaque data specific to the game.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<OpaqueData>,
}

/// A tradable item, such as gold, equipment or consumables.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Item {
    /// Identifier for the item in the game.
    pub id: String,
    /// The name of the item that should be shown to the player.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// The optional description of the item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The attributes of the item.
    pub attributes: Vec<AttributeModifier>,
    /// Quantity of the item.
    pub amount: u32,
    /// Optional opaque data specific to the game.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<OpaqueData>,
}

/// An attribute that is changed by using or equipping the item.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct AttributeModifier {
    /// Identifier for the Attribute in the game.
    pub id: String,
    /// The name of the attribute that should be shown to the player.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Amount change for the attribute.
    pub amount: i32,
    /// How the amount should be applied.
    pub kind: ModifierKind,
    /// Optional opaque data specific to the game.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<OpaqueData>,
}

/// What kind of attribute modifier it is.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub enum ModifierKind {
    /// Flat increase.
    #[default]
    Flat,
    /// Percent increase.
    Percent,
}
