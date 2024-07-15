use log::trace;

use crate::dataset::{
    item::Item,
    spot::{AllRequirements, Spot},
    storage::{ItemSpot, Storage},
};

use super::{
    assertions::{assert_chests, ware_missing_requirements},
    spot::{self, AnyOfAllRequirements, RequirementFlag, SpotName},
    storage,
    supplements::{SpotYaml, SupplementFiles, WeaponsYaml, YamlShop, YamlSpot},
};

#[derive(Clone)]
pub struct Event {
    name: String,
    requirements: AnyOfAllRequirements,
}

fn to_any_of_all_requirements(requirements: Vec<String>) -> Option<AnyOfAllRequirements> {
    if requirements.is_empty() {
        None
    } else {
        Some(AnyOfAllRequirements(
            requirements
                .into_iter()
                .map(|y| {
                    AllRequirements(
                        y.split(',')
                            .map(|z| RequirementFlag::new(z.trim().to_owned()))
                            .collect(),
                    )
                })
                .collect(),
        ))
    }
}

fn parse_item_spot_requirements<T>(
    create: impl Fn(usize, SpotName, Option<AnyOfAllRequirements>) -> T,
    items: Vec<YamlSpot>,
) -> Vec<T> {
    items
        .into_iter()
        .enumerate()
        .map(|(src_idx, spot)| {
            create(
                src_idx,
                SpotName::new(spot.name),
                to_any_of_all_requirements(spot.requirements),
            )
        })
        .collect()
}

fn parse_shop_requirements<T>(
    create: impl Fn(usize, SpotName, Option<AnyOfAllRequirements>) -> T,
    items: Vec<YamlShop>,
) -> Vec<T> {
    items
        .into_iter()
        .enumerate()
        .map(|(src_idx, shop)| {
            create(
                src_idx,
                SpotName::new(shop.names),
                to_any_of_all_requirements(shop.requirements),
            )
        })
        .collect()
}

fn parse_requirements_of_events(items: Vec<YamlSpot>) -> Vec<Event> {
    let mut current: Vec<Event> = items
        .into_iter()
        .map(|x| Event {
            name: x.name,
            requirements: AnyOfAllRequirements(
                x.requirements
                    .into_iter()
                    .map(|y| {
                        AllRequirements(
                            y.split(',')
                                .map(|z| RequirementFlag::new(z.trim().to_owned()))
                                .collect(),
                        )
                    })
                    .collect(),
            ),
        })
        .collect();
    for _ in 0..100 {
        let events: Vec<_> = current
            .iter()
            .filter(|x| {
                x.requirements
                    .0
                    .iter()
                    .all(|y| y.0.iter().all(|z| !z.get().starts_with("event:")))
            })
            .cloned()
            .collect();
        if events.len() == current.len() {
            return current;
        }
        current = current
            .into_iter()
            .map(|x| Event {
                name: x.name,
                requirements: merge_events(x.requirements, &events),
            })
            .collect();
    }
    unreachable!();
}

fn merge_events(requirements: AnyOfAllRequirements, events: &[Event]) -> AnyOfAllRequirements {
    // [['event:a', 'event:b', 'c']]
    // 'event:a': [['d', 'e', 'f']]
    // 'event:b': [['g', 'h'], ['i', 'j']]
    // ↓
    // [['event:b', 'c', 'd', 'e', 'f']]
    // ↓
    // [
    //   ['c', 'd', 'e', 'f', 'g', 'h']
    //   ['c', 'd', 'e', 'f', 'i', 'j']
    // ]
    let mut current = requirements;
    for event in events {
        if current
            .0
            .iter()
            .all(|target_group| !target_group.0.iter().any(|x| x.get() == event.name))
        {
            continue;
        }
        current = AnyOfAllRequirements(
            current
                .0
                .into_iter()
                .flat_map(|target_group| -> Vec<AllRequirements> {
                    if !target_group.0.iter().any(|x| x.get() == event.name) {
                        return vec![target_group];
                    }
                    event
                        .requirements
                        .0
                        .iter()
                        .map(|event_group| -> AllRequirements {
                            AllRequirements(
                                event_group
                                    .0
                                    .clone()
                                    .into_iter()
                                    .chain(
                                        target_group
                                            .0
                                            .clone()
                                            .into_iter()
                                            .filter(|x| {
                                                x.get() != event.name
                                                    && !event_group.0.iter().any(|y| y == x)
                                            })
                                            .collect::<Vec<_>>(),
                                    )
                                    .collect(),
                            )
                        })
                        .collect()
                })
                .collect(),
        );
    }
    current
}

pub fn create_source(supplement_files: &SupplementFiles) -> Storage {
    let start = std::time::Instant::now();
    let weapons: WeaponsYaml = serde_yaml::from_str(&supplement_files.weapons_yml).unwrap();
    let main_weapons = weapons.main_weapons;
    let sub_weapons = weapons.sub_weapons;
    let chests: SpotYaml = serde_yaml::from_str(&supplement_files.chests_yml).unwrap();
    let seals: SpotYaml = serde_yaml::from_str(&supplement_files.seals_yml).unwrap();
    let shops: Vec<YamlShop> = serde_yaml::from_str(&supplement_files.shops_yml).unwrap();
    let events: SpotYaml = serde_yaml::from_str(&supplement_files.events_yml).unwrap();
    let events = parse_requirements_of_events(events.0);

    let mut main_weapons = parse_item_spot_requirements(
        |src_idx, name, requirements| ItemSpot {
            spot: Spot::main_weapon(src_idx, name.clone(), requirements),
            item: Item::main_weapon(src_idx, name.into()),
        },
        main_weapons,
    );
    main_weapons.iter_mut().for_each(|item_spot| {
        if let Some(requirements) = item_spot.spot.requirements_mut().take() {
            *item_spot.spot.requirements_mut() = Some(merge_events(requirements, &events));
        }
    });

    let mut sub_weapons = parse_item_spot_requirements(
        |src_idx, name, requirements| ItemSpot {
            spot: Spot::sub_weapon(src_idx, name.clone(), requirements),
            item: Item::sub_weapon(src_idx, name.into()),
        },
        sub_weapons,
    );
    sub_weapons.iter_mut().for_each(|item_spot| {
        if let Some(requirements) = item_spot.spot.requirements_mut().take() {
            *item_spot.spot.requirements_mut() = Some(merge_events(requirements, &events));
        }
    });

    let mut chests = parse_item_spot_requirements(
        |src_idx, name, requirements| ItemSpot {
            spot: Spot::chest(src_idx, name.clone(), requirements),
            item: Item::chest_item(src_idx, name.into()),
        },
        chests.0,
    );
    chests.iter_mut().for_each(|item_spot| {
        if let Some(requirements) = item_spot.spot.requirements_mut().take() {
            *item_spot.spot.requirements_mut() = Some(merge_events(requirements, &events));
        }
    });

    let mut seals = parse_item_spot_requirements(
        |src_idx, name, requirements| ItemSpot {
            spot: Spot::seal(src_idx, name.clone(), requirements),
            item: Item::seal(src_idx, name.into()),
        },
        seals.0,
    );
    seals.iter_mut().for_each(|item_spot| {
        if let Some(requirements) = item_spot.spot.requirements_mut().take() {
            *item_spot.spot.requirements_mut() = Some(merge_events(requirements, &events));
        }
    });

    let mut shops = parse_shop_requirements(
        |src_idx, name, requirements| {
            let shop_spot = spot::Shop::new(src_idx, name.clone(), requirements);
            let flags = shop_spot.to_strategy_flags();
            storage::Shop {
                spot: Spot::shop(shop_spot),
                items: (
                    Item::shop_item(src_idx, 0, flags.0),
                    Item::shop_item(src_idx, 1, flags.1),
                    Item::shop_item(src_idx, 2, flags.2),
                ),
            }
        },
        shops,
    );
    shops.iter_mut().for_each(|shop| {
        if let Some(requirements) = shop.spot.requirements_mut().take() {
            *shop.spot.requirements_mut() = Some(merge_events(requirements, &events));
        }
    });
    trace!("create_source parse: {:?}", start.elapsed());

    if cfg!(debug_assertions) {
        let start = std::time::Instant::now();
        assert_chests(&chests);
        trace!("assert_chests: {:?}", start.elapsed());
    }
    let start = std::time::Instant::now();
    let storage = Storage::new(main_weapons, sub_weapons, chests, seals, shops);
    trace!("Storage::new: {:?}", start.elapsed());
    if cfg!(debug_assertions) {
        let start = std::time::Instant::now();
        ware_missing_requirements(&storage);
        trace!("ware_missing_requirements: {:?}", start.elapsed());
    }
    storage
}
