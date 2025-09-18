use serde::{Deserialize, Serialize};

#[cfg(feature = "schemas")]
use schemars::JsonSchema;

/// A completed trade
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub struct Trade {
    /// the trade order
    pub order: TradeOrder,

    /// the response to the trade order
    pub response: TradeResponse,
}

/// Type alias for generic data. This can be used for minor extensions of the data.
pub type OpaqueData = serde_json::Value;

/// Represents a trade order consisting of multiple trade requests.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub struct TradeOrder {
    /// The trade requests contained in this order.
    pub requests: Vec<TradeRequest>,
    /// Whether all requests must be fulfilled together.
    pub all_or_nothing: bool,
    /// Optional note for the trade order.
    pub note: Option<String>,
}

/// Represents the response to a trade order.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub struct TradeResponse {
    /// The result of evaluating the trade order.
    pub result: TradeResult,
    /// Optional note explaining the response.
    pub note: Option<String>,
}

/// Possible outcomes when processing a trade order.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub enum TradeResult {
    /// The trade order was canceled.
    Cancellation {
        /// The trade order that was canceled.
        order: TradeOrder,
    },
    /// The trade order was fully fulfilled.
    Fulfillment {
        /// The trade requests that were executed.
        trades: Vec<TradeRequest>,
    },
    /// The trade order was partially fulfilled.
    ///
    /// Should only be used when `all_or_nothing` was set to false in the
    /// original trade order.
    Partial {
        /// The trade requests that were successfully fulfilled.
        fulfilled: Vec<TradeRequest>,
        /// The trade requests that could not be fulfilled.
        unfulfilled: Vec<TradeRequest>,
    },
}

/// Represents a single trade request, with the wanted and offered items.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub struct TradeRequest {
    /// A unique identifier of a trade request.
    /// This makes it easier to match fulfillments to requests.
    pub id: String,
    /// The wanted items.
    pub wanted: Vec<Item>,
    /// The items offered for fulfilling the trade.
    pub offered: Vec<Item>,
    /// Optional note for this trade request.
    pub note: Option<String>,
}

/// A tradable item, such as gold, equipment or consumables.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub struct Item {
    /// Identifier for the item in the game.
    ///
    /// This is intended to map the item to an in-game item.
    pub id: String,
    /// The name of the item that should is shown to the player.
    pub name: String,
    /// Optional description of the item.
    pub description: Option<String>,
    /// Quantity of the item.
    pub amount: usize,
    /// The type of item.
    pub kind: ItemKind,
    /// Optional note for this item.
    pub note: Option<String>,
}

/// The type of an item.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub enum ItemKind {
    /// Consumable item with effects.
    Consumable {
        /// Who the consumable targets.
        target: TargetType,
        /// Effects that occur when consumed.
        effects: Vec<Effect>,
    },
    /// Equipable item with attributes.
    Equipment {
        /// Slot type where this equipment is worn.
        slot: SlotType,
        /// Attribute modifiers applied by this equipment.
        attributes: Vec<AttributeModifier>,
    },
    /// Valuable item with rarity and classification.
    Valuable {
        /// Rarity of the valuable item.
        rarity: Rarity,
        /// Kind of valuable item.
        kind: ValuableKind,
    },
    /// Custom item data, used for item kinds not already covered here.
    Custom(
        /// Custom item data.
        OpaqueData,
    ),
}

/// The target type for consumables.
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub enum TargetType {
    /// Self use.
    User,
    /// Friendly target.
    Ally,
    /// Hostile target.
    Enemy,
    /// Non-player character.
    Npc,
    /// All friendly targets.
    Friendly,
    /// Any target.
    Any,
}

/// Effects that can be caused by consumables or abilities.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub enum Effect {
    /// Heal health points.
    Heal {
        /// Amount of health restored.
        amount: i32,
    },
    /// Restore mana points.
    RestoreMana {
        /// Amount of mana restored.
        amount: i32,
    },
    /// Restore stamina points.
    RestoreStamina {
        /// Amount of stamina restored.
        amount: i32,
    },
    /// Apply a positive modifier.
    Buff(
        /// Attribute modifier to apply.
        AttributeModifier,
    ),
    /// Apply a negative modifier.
    DeBuff(
        /// Attribute modifier to apply as a debuff.
        AttributeModifier,
    ),
    /// Deal damage of a certain element.
    Damage {
        /// Amount of damage dealt.
        amount: i32,
        /// Element of the damage.
        kind: Element,
    },
    /// Cure a specific status effect.
    CureStatus {
        /// Status effect to cure.
        status: StatusEffect,
    },
    /// Inflict a status effect.
    InflictStatus {
        /// Status effect to inflict.
        status: StatusEffect,
    },
    /// Teleport to a location.
    Teleport {
        /// Identifier or name of the location.
        location: String,
    },
    /// Custom effect.
    Custom(
        /// Custom effect data.
        OpaqueData,
    ),
}

/// Possible status effects that can affect characters.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub enum StatusEffect {
    /// Poison damage over time.
    Poison,
    /// Burn damage over time.
    Burn,
    /// Frozen state.
    Freeze,
    /// Stunned state.
    Stun,
    /// Blinded, reducing accuracy.
    Blind,
    /// Silenced, disabling abilities.
    Silence,
    /// Slowed movement or actions.
    Slow,
    /// General weakness.
    Weakness,
    /// Custom status effect.
    Custom(
        /// Custom status effect description.
        String,
    ),
}

/// Equipment slots for items.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub enum SlotType {
    /// Head slot.
    Head,
    /// Body slot.
    Body,
    /// Arm slot.
    Arm,
    /// Hand slot.
    Hand,
    /// Leg slot.
    Leg,
    /// Foot slot.
    Foot,
    /// Neck slot.
    Neck,
    /// Finger slot.
    Finger,
    /// Shield slot.
    Shield,
    /// Weapon slot with a specific kind.
    Weapon {
        /// Type of weapon equipped in this slot.
        kind: WeaponKind,
    },
}

/// Different weapon categories.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub enum WeaponKind {
    /// A simple blunt weapon.
    Club,
    /// A spiked blunt weapon.
    Mace,
    /// A heavy striking weapon.
    Hammer,
    /// A chained blunt weapon.
    Flail,
    /// A long staff or rod.
    Staff,
    /// A small bladed weapon.
    Dagger,
    /// A standard bladed weapon.
    Sword,
    /// A large two-handed sword.
    Greatsword,
    /// A one-handed axe.
    Axe,
    /// A larger battle axe.
    BattleAxe,
    /// A massive two-handed axe.
    GreatAxe,
    /// A thrusting spear.
    Spear,
    /// A polearm with an axe head.
    Halberd,
    /// A curved farming blade repurposed as weapon.
    Scythe,
    /// A burning torch.
    Torch,
    /// A mining pickaxe used as weapon.
    Pickaxe,
    /// A shovel as an improvised weapon.
    Shovel,
    /// A farming sickle.
    Sickle,
    /// A butcher's cleaver.
    Cleaver,
    /// A throwing knife.
    ThrowingKnife,
    /// A light throwing spear.
    Javelin,
    /// A throwing axe.
    ThrowingAxe,
    /// A curved returning weapon.
    Boomerang,
    /// A sling for stones.
    Sling,
    /// A simple bow.
    Bow,
    /// A longbow for distance.
    Longbow,
    /// A composite bow with stronger pull.
    CompositeBow,
    /// A crossbow.
    Crossbow,
    /// A heavy crossbow.
    HeavyCrossbow,
    /// A repeating crossbow.
    RepeatingCrossbow,
    /// A blowgun.
    Blowgun,
    /// A flintlock pistol.
    FlintlockPistol,
    /// A musket.
    Musket,
    /// A rifle.
    Rifle,
    /// A revolver.
    Revolver,
    /// A shotgun.
    Shotgun,
    /// A cannon.
    Cannon,
    /// A magical wand.
    Wand,
    /// A magical staff.
    MagicStaff,
    /// A magical orb.
    Orb,
    /// A spellbook.
    Spellbook,
    /// A whip.
    Whip,
    /// A flail with a chain.
    ChainFlail,
    /// Claw weapons.
    Claw,
    /// Custom weapon.
    Custom(
        /// Custom weapon type.
        String,
    ),
}

/// Attribute modifier structure.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub struct AttributeModifier {
    /// The attribute being modified.
    pub attribute: Attribute,
    /// How the modification is applied.
    pub kind: ModifierKind,
    /// Value of the modification.
    pub amount: i32,
    /// Optional note about the modifier.
    pub note: Option<String>,
}

/// Kind of modifier applied.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub enum ModifierKind {
    /// Flat modification.
    Flat,
    /// Percentage-based modification.
    Percent,
}

/// Attributes that can be modified by equipment or effects.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub enum Attribute {
    /// General attack power.
    Attack,
    /// General defense power.
    Defense,

    /// Maximum health.
    MaxHealth,
    /// Maximum stamina.
    MaxStamina,
    /// Maximum mana.
    MaxMana,

    /// Health regeneration.
    HealthRegeneration,
    /// Stamina regeneration.
    StaminaRegeneration,
    /// Mana regeneration.
    ManaRegeneration,

    /// Critical hit chance.
    CritChance,
    /// Critical hit damage.
    CritDamage,
    /// Accuracy rating.
    Accuracy,
    /// Attack speed.
    AttackSpeed,
    /// Casting speed.
    CastSpeed,
    /// Attack or ability range.
    Range,

    /// Chance to evade.
    Evasion,
    /// Chance to block.
    BlockChance,

    /// Movement speed.
    MoveSpeed,
    /// Carrying capacity.
    CarryCapacity,

    /// Additional damage of a specific element.
    Damage {
        /// Element of the damage.
        kind: Element,
    },
    /// Resistance to a specific element.
    Resistance {
        /// Element resisted.
        kind: Element,
    },

    /// Custom attribute.
    Custom(
        /// Custom attribute data.
        OpaqueData,
    ),
}

/// Types of elemental damage or resistance.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub enum Element {
    /// Physical element.
    Physical,
    /// Fire element.
    Fire,
    /// Cold element.
    Cold,
    /// Lightning element.
    Lightning,
    /// Poison element.
    Poison,
    /// Arcane element.
    Arcane,
    /// Holy element.
    Holy,
    /// Shadow element.
    Shadow,
    /// Custom element.
    Custom(
        /// Custom element description.
        String,
    ),
}

/// Types of valuables.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub enum ValuableKind {
    /// A gem.
    Gem,
    /// A jewel.
    Jewel,
    /// Precious metal.
    PreciousMetal,
    /// Artifact.
    Artifact,
    /// Relic.
    Relic,
    /// Currency.
    Currency,
    /// Custom valuable kind.
    Custom(
        /// Custom valuable kind description.
        String,
    ),
}

/// Rarity levels for items.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub enum Rarity {
    /// Common rarity.
    Common,
    /// Uncommon rarity.
    Uncommon,
    /// Rare rarity.
    Rare,
    /// Epic rarity.
    Epic,
    /// Legendary rarity.
    Legendary,
    /// Mythic rarity.
    Mythic,
    /// Custom named rarity.
    CustomNamed(
        /// Custom rarity name.
        String,
    ),
    /// Custom rarity as numeric value.
    CustomValue(
        /// Custom numeric rarity value.
        i64,
    ),
}
