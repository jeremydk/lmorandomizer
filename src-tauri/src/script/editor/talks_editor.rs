use std::collections::BTreeMap;

use anyhow::{bail, Result};
use log::warn;
use regex::Regex;

use crate::{
    randomizer::{self, storage},
    script::{
        data::{
            item::Item,
            object::ItemShop,
            script::Script,
            shop_items_data::{self, ShopItem},
            talk::Talk,
        },
        enums,
    },
};

fn hide_overflow(kana: &str) -> String {
    let mut count = 0;
    kana.chars()
        .filter_map(|c| {
            match count {
                ..=21 => {
                    if !matches!(c, 'ﾞ' | 'ﾟ') {
                        count += 1;
                    }
                    Some(c)
                }
                // Overflow is represented by whitespace
                22 => {
                    if !matches!(c, 'ﾞ' | 'ﾟ') {
                        Some(' ')
                    } else {
                        Some(c)
                    }
                }
                _ => {
                    if !matches!(c, 'ﾞ' | 'ﾟ') {
                        Some(' ')
                    } else {
                        None
                    }
                }
            }
        })
        .collect()
}

fn to_name_talk_number(item: enums::ShopItem) -> usize {
    match item {
        enums::ShopItem::Rom(rom) => rom as usize,
        enums::ShopItem::Equipment(equipment) => 500 + equipment as usize,
        enums::ShopItem::SubWeapon(sub_weapon) => 645 + sub_weapon as usize,
    }
}

fn to_name(item: enums::ShopItem, talks: &[Talk]) -> Result<Talk> {
    let talk_number = to_name_talk_number(item);
    let Some(talk) = talks.get(talk_number).cloned() else {
        bail!("script broken: talk_number={}", talk_number)
    };
    Ok(talk)
}

fn replace_shop_item_talk(
    talks: &[Talk],
    talk_number: usize,
    old: enums::ShopItem,
    new: enums::ShopItem,
) -> Result<Talk> {
    let old_item_name = to_name(old, talks)?;
    let new_item_name = to_name(new, talks)?;
    let Some(talk) = talks.get(talk_number) else {
        bail!("script broken: talk_number={}", talk_number)
    };
    const ITEM_NAME_NORMALIZE_MAP: [(&str, &str); 19] = [
        (r"^ﾊﾂﾀﾞﾝﾄｳ", "はつたﾞんとう"),       // 3
        (r"^ﾏｼﾞｮｳﾃﾞﾝｾﾂ", "魔しﾞょうてﾞんせつ"), // 42
        (r"あくま", "ｱｸﾏ"),                 // 47
        (r"ふういん", "ﾌｳ印"),              // 62
        (r"ましﾞゅつ", "魔しﾞゅつ"),          // 66
        (r"^うらない", "ｺﾅﾐのうらない"),    // 72
        (r"^F1SP", "F1ｽﾋﾟﾘｯﾄ"),              // 74
        (r"^ﾊﾞｸﾀﾞﾝ", "はﾞくたﾞん"),             // 4
        (r"^しﾞゅうたﾞん", "たﾞんかﾞん"),       // 12
        (r"^ｶﾌﾞﾄ", "かふﾞと"),                // 32
        (r"^時をとめる", "時の"),           // 43
        (r"^ｽｷｬﾅ", "にせｽｷｬﾅｰ"),            // 45
        // NOTE: English doesn't work because of the complexity of "a" etc.
        //       Moreover, there is an error in the English patch
        //       (there is a mistake where a bomb is mistaken for a flare gun).
        //       I'm not a native speaker and cannot work on this problem.
        //       If anyone cares to help, please contribute.
        (r"MSX 2", "MSX2"),                     // 40
        (r"lamp", "Lamp of Time"),              // 43
        (r"a Scanner", "a Fake Scanner"),       // 45
        (r"Throwing Knives", "Throwing Knife"), // 1
        (r"(?i)Flares", "Flare Gun"),           // 3
        (r"Shield", "Silver Shield"),           // 10
        (r"ammo", "Ammunition"),                // 12
    ];
    let normalized_talk =
        &ITEM_NAME_NORMALIZE_MAP
            .iter()
            .fold(talk.to_string().to_owned(), |name, &(from, to)| {
                Regex::new(from)
                    .unwrap()
                    .replace(name.as_ref(), to)
                    .to_string()
            });
    let result = Regex::new(&format!("(?i){}", old_item_name))
        .unwrap()
        .replace(normalized_talk, new_item_name.to_string());
    if &result == normalized_talk {
        warn!(
            "failed to replace shop item name: talk={}, old={}, new={}",
            talk, old_item_name, new_item_name,
        );
    }
    Ok(Talk::from_text(&hide_overflow(&result)))
}

fn to_dataset_shop_item_from_item(item: Option<&Item>) -> Option<enums::ShopItem> {
    match item {
        None => None,
        Some(Item::Equipment(dataset)) => Some(enums::ShopItem::Equipment(dataset.content)),
        Some(Item::Rom(dataset)) => Some(enums::ShopItem::Rom(dataset.content)),
        Some(Item::SubWeapon(dataset)) => Some(enums::ShopItem::SubWeapon(dataset.content)),
        Some(Item::Seal(_)) | Some(Item::MainWeapon(_)) => unreachable!(),
    }
}

fn is_consumable(item: &Item) -> bool {
    match item {
        Item::SubWeapon(sub_weapon) => sub_weapon.amount > 0,
        Item::Rom(_) | Item::Equipment(_) | Item::Seal(_) | Item::MainWeapon(_) => false,
    }
}

fn replace_items(
    old: (ShopItem, ShopItem, ShopItem),
    new: (Option<Item>, Option<Item>, Option<Item>),
) -> (ShopItem, ShopItem, ShopItem) {
    let mut items = [(old.0, new.0), (old.1, new.1), (old.2, new.2)]
        .into_iter()
        .map(|(old_item, new_item)| {
            let Some(new_item) = new_item else {
                return old_item;
            };
            let price = if is_consumable(&new_item) {
                new_item.price().unwrap()
            } else {
                old_item.price()
            };
            ShopItem::from_item(new_item, price)
        });
    (
        items.next().unwrap(),
        items.next().unwrap(),
        items.next().unwrap(),
    )
}

fn create_shop_item_talks(
    talks: &[Talk],
    base_talk_number: u16,
    old: [enums::ShopItem; 3],
    new: [Option<enums::ShopItem>; 3],
) -> Result<Vec<(usize, Talk)>> {
    old.into_iter()
        .enumerate()
        .zip(new)
        .flat_map(|((idx, old), new)| new.map(|new| (idx, old, new)))
        .filter(|(_, old, new)| old != new)
        .map(|(idx, old, new)| {
            let talk_number = base_talk_number as usize + 1 + idx;
            let new_talk = replace_shop_item_talk(talks, talk_number, old, new)?;
            Ok((talk_number, new_talk))
        })
        .collect()
}

pub fn replace_shops(
    talks: &mut [Talk],
    script: &Script,
    script_shops: &[ItemShop],
    dataset_shops: &[storage::Shop],
) -> Result<()> {
    let dataset_shops: BTreeMap<_, Vec<_>> =
        dataset_shops.iter().fold(BTreeMap::new(), |mut map, shop| {
            map.entry(shop.spot.items()).or_default().push(shop);
            map
        });
    for dataset_shop in dataset_shops.values() {
        let create_item = |item: Option<&randomizer::storage::item::Item>| {
            item.as_ref().map(|x| Item::new(&x.src, script)).transpose()
        };
        let new_items = (
            create_item(dataset_shop.iter().find(|x| x.idx == 0).map(|x| &x.item))?,
            create_item(dataset_shop.iter().find(|x| x.idx == 1).map(|x| &x.item))?,
            create_item(dataset_shop.iter().find(|x| x.idx == 2).map(|x| &x.item))?,
        );
        let new_dataset_shop_items = [
            to_dataset_shop_item_from_item(new_items.0.as_ref()),
            to_dataset_shop_item_from_item(new_items.1.as_ref()),
            to_dataset_shop_item_from_item(new_items.2.as_ref()),
        ];

        let Some(script_shop) = script_shops.iter().find(|script_shop| {
            let old = ShopItem::to_spot_shop_items(script_shop.items());
            let items = dataset_shop[0].spot.items();
            enums::ShopItem::matches_items(old, items)
        }) else {
            bail!("shop not found: {:?}", dataset_shop[0].spot.items())
        };
        let talk_number = script_shop.item_data_talk_number();

        let old = {
            let Some(talk) = talks.get(talk_number as usize) else {
                bail!("script broken: talk_number={}", talk_number)
            };
            shop_items_data::parse(talk)?
        };
        let new = new_items;
        let new_shop_talk = shop_items_data::stringify(replace_items(old, new))?;

        let old = ShopItem::to_spot_shop_items(script_shop.items());
        let new = new_dataset_shop_items;
        let new_shop_item_talks = create_shop_item_talks(talks, talk_number, old, new)?;

        let Some(talk) = talks.get_mut(talk_number as usize) else {
            bail!("script broken: talk_number={}", talk_number)
        };
        *talk = new_shop_talk;
        for (talk_number, new_talk) in new_shop_item_talks {
            let Some(talk) = talks.get_mut(talk_number) else {
                bail!("script broken: talk_number={}", talk_number)
            };
            *talk = new_talk;
        }
    }
    Ok(())
}
