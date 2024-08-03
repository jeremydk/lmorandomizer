use std::collections::HashSet;

use anyhow::Result;
use log::{info, trace};
use rand::Rng;

use crate::{
    randomizer::spoiler::{items::Items, spots::Spots},
    script::{data::script::Script, editor::apply_storage},
};

use super::{
    spoiler::{make_rng, spoiler},
    spoiler_log::{CheckpointRef, SpoilerLogRef},
    storage::{item::StrategyFlag, Storage},
    RandomizeOptions,
};

pub fn randomize_items<'a>(
    script: &mut Script,
    source: &'a Storage,
    options: &RandomizeOptions,
) -> Result<SpoilerLogRef<'a>> {
    let start = std::time::Instant::now();
    assert_unique(source);
    trace!("Assertion in {:?}", start.elapsed());

    let start = std::time::Instant::now();
    let (shuffled, spoiler_log) = shuffle(source, options);
    trace!("Randomized items in {:?}", start.elapsed());

    let start = std::time::Instant::now();
    assert_unique(&shuffled);
    apply_storage(script, &shuffled)?;
    trace!("Replaced items in {:?}", start.elapsed());
    Ok(spoiler_log)
}

fn create_shuffled_storage(source: &Storage, spoiler_log: &SpoilerLogRef) -> Storage {
    let mut storage = source.clone();
    for checkpoint in spoiler_log
        .progression
        .iter()
        .flat_map(|sphere| &sphere.0)
        .chain(&spoiler_log.maps)
    {
        match checkpoint {
            CheckpointRef::MainWeapon(main_weapon) => {
                let content = main_weapon.spot.main_weapon();
                storage.main_weapons.get_mut(&content).unwrap().item = main_weapon.item.clone();
            }
            CheckpointRef::SubWeapon(sub_weapon) => {
                let key = (sub_weapon.spot.field_number(), sub_weapon.spot.sub_weapon());
                storage.sub_weapons.get_mut(&key).unwrap().item = sub_weapon.item.clone();
            }
            CheckpointRef::Chest(chest) => {
                let key = (chest.spot.field_number(), chest.spot.item());
                storage.chests.get_mut(&key).unwrap().item = chest.item.clone();
            }
            CheckpointRef::Seal(seal) => {
                let content = seal.spot.seal();
                storage.seals.get_mut(&content).unwrap().item = seal.item.clone();
            }
            CheckpointRef::Shop(shop) => {
                let items = &mut storage
                    .shops
                    .iter_mut()
                    .find(|x| x.spot.items() == shop.spot.items())
                    .unwrap()
                    .items;
                items.iter_mut().zip(&shop.items).for_each(|(old, new)| {
                    *old = new.cloned();
                });
            }
            CheckpointRef::Rom(rom) => {
                let content = rom.spot.rom();
                storage.roms.get_mut(&content).unwrap().item = rom.item.clone();
            }
            CheckpointRef::Event(_) => {}
        }
    }
    storage
}

fn random_spoiler<'a>(
    rng: &mut impl Rng,
    source: &'a Storage,
    options: &RandomizeOptions,
) -> SpoilerLogRef<'a> {
    let start = std::time::Instant::now();
    let items = &Items::new(source);
    let spots = &Spots::new(source);
    debug_assert_eq!(
        spots.shops.len() - items.consumable_items().len(),
        spots
            .shops
            .iter()
            .map(|spot| (!spot.name.is_consumable()) as usize)
            .sum::<usize>(),
    );
    debug_assert_eq!(
        spots
            .shops
            .iter()
            .map(|spot| spot.name.is_consumable() as usize)
            .sum::<usize>(),
        items.consumable_items().len()
    );
    trace!("Prepared items and spots in {:?}", start.elapsed());

    let thread_count = std::thread::available_parallelism().unwrap().get();
    std::thread::scope(|scope| {
        for i in 0..100000 {
            let handles: Vec<_> = (0..thread_count)
                .map(|_| rng.next_u64())
                .map(|seed| scope.spawn(move || spoiler(seed, options, items, spots)))
                .collect();
            let Some(spoiler_log) = handles.into_iter().filter_map(|h| h.join().unwrap()).next()
            else {
                continue;
            };
            info!("Shuffle was tried: {} times", (i + 1) * thread_count);
            return spoiler_log;
        }
        unreachable!();
    })
}

fn shuffle<'a>(source: &'a Storage, options: &RandomizeOptions) -> (Storage, SpoilerLogRef<'a>) {
    let mut rng = make_rng(&options.seed);
    let spoiler_log = random_spoiler(&mut rng, source, options);
    let storage = create_shuffled_storage(source, &spoiler_log);
    (storage, spoiler_log)
}

fn assert_unique(storage: &Storage) {
    if cfg!(not(debug_assertions)) {
        return;
    }
    let mut names = HashSet::new();

    storage
        .main_weapons
        .values()
        .map(|x| ("weapon", &x.item))
        .chain(storage.sub_weapons.values().map(|x| ("weapon", &x.item)))
        .chain(storage.chests.values().map(|x| ("chest", &x.item)))
        .chain(storage.seals.values().map(|x| ("seal", &x.item)))
        .chain(
            storage
                .shops
                .iter()
                .flat_map(|x| &x.items)
                .filter_map(|item| item.as_ref())
                .map(|item| ("shop", item)),
        )
        .for_each(|(item_type, item)| {
            if !item.name.is_consumable()
                && ![
                    StrategyFlag::new("shellHorn".to_owned()),
                    StrategyFlag::new("finder".to_owned()),
                ]
                .contains(&item.name)
            {
                let key = format!("{}:{:?}", item_type, &item.name);
                if names.contains(&key) {
                    panic!("Duplicate item: {}", key);
                }
                names.insert(key);
            }
        });
}

#[cfg(test)]
mod tests {
    use sha3::Digest;

    use crate::{
        app::read_game_structure_files_debug, dataset::game_structure::GameStructure,
        randomizer::storage::create_source::create_source,
    };

    use super::*;

    #[tokio::test]
    async fn test_shuffle() -> Result<()> {
        let game_structure_files = read_game_structure_files_debug().await?;
        let game_structure = GameStructure::new(game_structure_files)?;
        let opts = RandomizeOptions {
            seed: "test".to_owned(),
            shuffle_secret_roms: true,
            need_glitches: false,
            absolutely_shuffle: false,
        };
        let source = create_source(&game_structure, &opts)?;
        let (shuffled, spoiler_log) = shuffle(&source, &opts);

        let shuffled_str = format!("{:?}", shuffled);
        let shuffled_hash = hex::encode(sha3::Sha3_512::digest(shuffled_str));
        const EXPECTED_SHUFFLED_HASH: &str = "7d416fe875f11d80cc00ced83f63b7563bd145c4d8cc8003c00439f43349e17ab82e92c28391bddce262a6681dca19c8fac5c1605aa7f9f86389ad7d6b5ed5c2";
        assert_eq!(shuffled_hash, EXPECTED_SHUFFLED_HASH);

        let spoiler_log_str = format!("{}", spoiler_log.to_owned());
        let spoiler_log_hash = hex::encode(sha3::Sha3_512::digest(spoiler_log_str));
        const EXPECTED_SPOILER_LOG_HASH: &str = "cc18277c285d1ed1c6d0e4d7f1f2042e0845a898032ccfb7457e98e32c0cbd41b08815042c941cf7559b2c9993d412ed788d7bc933a5deb2dc7d7382300f2b8c";
        assert_eq!(spoiler_log_hash, EXPECTED_SPOILER_LOG_HASH);

        Ok(())
    }
}
