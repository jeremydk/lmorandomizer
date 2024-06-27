use std::num::NonZero;

use crate::script::data::items;

use super::supplements::StrategyFlag;

#[derive(Clone, Debug)]
pub struct MainWeapon {
    pub name: StrategyFlag,
    pub content: items::MainWeapon,
    pub flag: u16,
}

#[derive(Clone, Debug)]
pub struct SubWeapon {
    pub name: StrategyFlag,
    pub content: items::SubWeapon,
    pub count: Option<NonZero<u8>>,
    pub flag: u16,
}

#[derive(Clone, Debug)]
pub struct Equipment {
    pub name: StrategyFlag,
    pub content: items::Equipment,
    pub flag: u16,
}

#[derive(Clone, Debug)]
pub struct Rom {
    pub name: StrategyFlag,
    pub content: items::Rom,
    pub flag: u16,
}

#[derive(Clone, Debug)]
pub struct Seal {
    pub name: StrategyFlag,
    pub content: items::Seal,
    pub flag: u16,
}

#[derive(Clone, Debug)]
pub enum Item {
    MainWeapon(MainWeapon),
    SubWeapon(SubWeapon),
    Equipment(Equipment),
    Rom(Rom),
    Seal(Seal),
}

impl Item {
    #[inline]
    fn initial_assert(number: i8, flag: u16, is_sub_weapon: bool) {
        debug_assert!(
            [494, 524].contains(&flag)
                || (684..=883).contains(&flag)
                || is_sub_weapon && flag == 65279,
            "invalid value: {flag} ({number})"
        );
    }
    pub fn main_weapon(name: StrategyFlag, content: items::MainWeapon, flag: u16) -> Self {
        Self::initial_assert(content as i8, flag, false);
        Self::MainWeapon(MainWeapon {
            name,
            content,
            flag,
        })
    }
    pub fn sub_weapon(
        name: StrategyFlag,
        content: items::SubWeapon,
        count: Option<NonZero<u8>>,
        flag: u16,
    ) -> Self {
        Self::initial_assert(content as i8, flag, true);
        Self::SubWeapon(SubWeapon {
            name,
            content,
            count,
            flag,
        })
    }
    pub fn equipment(name: StrategyFlag, content: items::Equipment, flag: u16) -> Self {
        Self::initial_assert(content as i8, flag, false);
        Self::Equipment(Equipment {
            name,
            content,
            flag,
        })
    }
    pub fn rom(name: StrategyFlag, content: items::Rom, flag: u16) -> Self {
        Self::initial_assert(content.0 as i8, flag, false);
        Self::Rom(Rom {
            name,
            content,
            flag,
        })
    }
    pub fn seal(name: StrategyFlag, content: items::Seal, flag: u16) -> Self {
        Self::initial_assert(content as i8, flag, false);
        Self::Seal(Seal {
            name,
            content,
            flag,
        })
    }

    pub fn name(&self) -> &StrategyFlag {
        match self {
            Self::MainWeapon(x) => &x.name,
            Self::SubWeapon(x) => &x.name,
            Self::Equipment(x) => &x.name,
            Self::Rom(x) => &x.name,
            Self::Seal(x) => &x.name,
        }
    }

    pub fn flag(&self) -> u16 {
        match self {
            Self::MainWeapon(x) => x.flag,
            Self::SubWeapon(x) => x.flag,
            Self::Equipment(x) => x.flag,
            Self::Rom(x) => x.flag,
            Self::Seal(x) => x.flag,
        }
    }

    // chests -> equipments / rom
    // chests <- subWeapon / subWeaponAmmo / equipments / rom / sign
    // shops -> equipments / rom
    // shops <- subWeapon / subWeaponAmmo / equipments / rom
    pub fn can_display_in_shop(&self) -> bool {
        self.flag() % 256 != 0
            && match self {
                Self::MainWeapon(_) => false,
                Self::SubWeapon(x) => {
                    x.count.is_some()
                        || x.content == items::SubWeapon::Pistol
                        || x.content == items::SubWeapon::Buckler
                        || x.content == items::SubWeapon::HandScanner
                }
                Self::Equipment(x) => {
                    x.content != items::Equipment::Map && x.content != items::Equipment::SacredOrb
                }
                Self::Rom(_) => true,
                Self::Seal(_) => false,
            }
    }
}
