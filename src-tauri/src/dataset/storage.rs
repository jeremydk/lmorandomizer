use super::{item::Item, spot::Spot};

#[derive(Clone, Debug)]
pub struct ItemSpot {
    pub spot: Spot,
    pub item: Item,
}

#[derive(Clone, Debug)]
pub struct Shop {
    pub spot: Spot,
    pub items: (Item, Item, Item),
}
impl Shop {
    pub fn count_general_items(&self) -> usize {
        !self.items.0.name.is_consumable() as usize
            + !self.items.1.name.is_consumable() as usize
            + !self.items.2.name.is_consumable() as usize
    }
}

#[derive(Default)]
pub struct StorageIndices {
    pub main_weapon_spot_idx: usize,
    pub sub_weapon_spot_idx: usize,
    pub chest_idx: usize,
    pub seal_chest_idx: usize,
}

#[derive(Clone, Debug)]
pub struct Storage {
    pub main_weapons: Vec<ItemSpot>,
    pub sub_weapons: Vec<ItemSpot>,
    pub chests: Vec<ItemSpot>,
    pub seals: Vec<ItemSpot>,
    pub shops: Vec<Shop>,
}

impl Storage {
    pub fn new(
        main_weapons: Vec<ItemSpot>,
        sub_weapons: Vec<ItemSpot>,
        chests: Vec<ItemSpot>,
        seals: Vec<ItemSpot>,
        shops: Vec<Shop>,
    ) -> Self {
        Self {
            main_weapons,
            sub_weapons,
            chests,
            seals,
            shops,
        }
    }

    pub fn all_items(&self) -> impl Iterator<Item = &Item> {
        self.main_weapons
            .iter()
            .chain(&self.sub_weapons)
            .chain(&self.chests)
            .chain(&self.seals)
            .map(|x| &x.item)
            .chain(
                self.shops
                    .iter()
                    .flat_map(|x| [&x.items.0, &x.items.1, &x.items.2]),
            )
    }
}
