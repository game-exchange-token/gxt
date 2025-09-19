use gxt::advisory::{AttributeModifier, IdCard, Item, ModifierKind, TradeOrder, TradeRequest};
use stringlit::s;

fn main() -> anyhow::Result<()> {
    let alice = gxt::make_key();
    let bob = gxt::make_key();

    let id_card_token = gxt::make_id_card(
        &bob,
        gxt::advisory::IdCard {
            display_name: "Bob".to_string(),
            ..IdCard::default()
        },
    )?;
    let id_card = gxt::verify_message::<gxt::advisory::IdCard>(&id_card_token)?;
    println!("Bobs ID Card:");
    println!("{}", gxt::to_json_pretty(&id_card)?);
    println!();

    let order = TradeOrder {
        all_or_nothing: false,
        note: None,
        requests: vec![TradeRequest {
            id: s!("cf2c7f92-149f-4224-b176-18c7cd0c51d5"),
            wanted: vec![Item {
                id: s!("weapons.swords.fire_sword"),
                description: Some(s!("Fiery fire sword of fire damage")),
                display_name: Some(s!("Fire Sword")),
                amount: 1,
                attributes: vec![AttributeModifier {
                    id: s!("damage_types.fire"),
                    display_name: Some(s!("Fire Damage")),
                    amount: 10,
                    kind: ModifierKind::Percent,
                    ..Default::default()
                }],
                ..Default::default()
            }],
            offered: vec![Item {
                id: s!("gold"),
                amount: 100,
                ..Item::default()
            }],
            data: None,
        }],
    };

    let encrypted_message = gxt::encrypt_message(&alice, &id_card_token, &order, None)?;
    println!("Encrypted Message for Bob:");
    println!("{}", encrypted_message);
    println!();

    let envelope = gxt::decrypt_message::<TradeOrder>(&encrypted_message, &bob)?;

    println!("Decrypted Message for Bob:");
    println!("{}", gxt::to_json_pretty(&envelope)?);
    println!();
    Ok(())
}
