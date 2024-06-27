use std::num::NonZero;

use anyhow::{anyhow, bail, Result};
use num_traits::FromPrimitive;

use crate::script::file::dat::{byte_code_to_text, text_to_byte_code};

use super::items::{Equipment, Rom, SubWeapon};

pub fn parse(text: &str) -> Result<(ShopItem, ShopItem, ShopItem)> {
    debug_assert_eq!(text.chars().count(), 7 * 3);
    let data = text_to_byte_code(text);
    debug_assert_eq!(data.len(), 7 * 3);
    let mut iter = (0..3)
        .map(|i| i * 7)
        .map(|x| ShopItem::from_bytes(&data[x..x + 7]));
    Ok((
        iter.next().unwrap()?,
        iter.next().unwrap()?,
        iter.next().unwrap()?,
    ))
}

pub fn stringify(items: (ShopItem, ShopItem, ShopItem)) -> Result<String> {
    let data: Vec<_> = [items.0, items.1, items.2]
        .iter()
        .flat_map(|x| x.to_bytes())
        .collect();
    Ok(byte_code_to_text(&data))
}

pub struct ShopSubWeaponBody {
    pub sub_weapon: SubWeapon,
    price: u16,
    pub set_flag: u16,
}

pub struct ShopSubWeaponAmmo {
    pub sub_weapon: SubWeapon,
    price: u16,
    pub count: NonZero<u8>,
    pub set_flag: u16,
}

pub struct ShopEquipment {
    pub equipment: Equipment,
    price: u16,
    pub set_flag: u16,
}

pub struct ShopRom {
    pub rom: Rom,
    price: u16,
    pub set_flag: u16,
}

pub enum ShopItem {
    SubWeaponBody(ShopSubWeaponBody),
    SubWeaponAmmo(ShopSubWeaponAmmo),
    Equipment(ShopEquipment),
    Rom(ShopRom),
}

impl ShopItem {
    pub fn sub_weapon_body(sub_weapon: SubWeapon, price: u16, set_flag: u16) -> Self {
        Self::SubWeaponBody(ShopSubWeaponBody {
            sub_weapon,
            price,
            set_flag,
        })
    }
    pub fn sub_weapon_ammo(
        sub_weapon: SubWeapon,
        price: u16,
        count: NonZero<u8>,
        set_flag: u16,
    ) -> Self {
        Self::SubWeaponAmmo(ShopSubWeaponAmmo {
            sub_weapon,
            price,
            count,
            set_flag,
        })
    }
    pub fn equipment(equipment: Equipment, price: u16, set_flag: u16) -> Self {
        Self::Equipment(ShopEquipment {
            equipment,
            price,
            set_flag,
        })
    }
    pub fn rom(rom: Rom, price: u16, set_flag: u16) -> Self {
        Self::Rom(ShopRom {
            rom,
            price,
            set_flag,
        })
    }

    fn from_bytes(data: &[u8]) -> Result<Self> {
        let shop_item_type = data[0] - 1;
        let number = data[1] - 1;
        let price = (((data[2] - 1) as u16) << 8) + data[3] as u16;
        let set_flag = (((data[5] - 1) as u16) << 8) + data[6] as u16; // 254 * 256 + 255 is no set flag
        match shop_item_type {
            0 => NonZero::new(data[4] - 1).map_or_else(
                || {
                    Ok(Self::sub_weapon_body(
                        SubWeapon::from_u8(number)
                            .ok_or_else(|| anyhow!("Invalid subweapon number: {}", number))?,
                        price,
                        set_flag,
                    ))
                },
                |count| {
                    Ok(Self::sub_weapon_ammo(
                        SubWeapon::from_u8(number)
                            .ok_or_else(|| anyhow!("Invalid subweapon number: {}", number))?,
                        price,
                        count,
                        set_flag,
                    ))
                },
            ),
            1 => Ok(Self::equipment(
                Equipment::from_u8(number)
                    .ok_or_else(|| anyhow!("Invalid equipment number: {}", number))?,
                price,
                set_flag,
            )),
            // NOTE: 占いセンセーション(72) count is not as specified. It has 1 in it, not 0.
            2 => Ok(Self::Rom(ShopRom {
                rom: Rom(number),
                price,
                set_flag,
            })),
            _ => bail!("Invalid shop item type: {}", shop_item_type),
        }
    }

    fn to_bytes(&self) -> [u8; 7] {
        [
            match self {
                Self::SubWeaponBody(_) | Self::SubWeaponAmmo(_) => 0,
                Self::Equipment(_) => 1,
                Self::Rom(_) => 2,
            } + 1,
            self.number() + 1,
            ((self.price() >> 8) + 1) as u8,
            (self.price() % 0x100) as u8,
            self.count().map_or(0, |x| x.get()) + 1,
            ((self.set_flag() >> 8) + 1) as u8,
            (self.set_flag() % 0x100) as u8,
        ]
    }

    pub fn number(&self) -> u8 {
        match self {
            Self::SubWeaponBody(x) => x.sub_weapon as u8,
            Self::SubWeaponAmmo(x) => x.sub_weapon as u8,
            Self::Equipment(x) => x.equipment as u8,
            Self::Rom(x) => x.rom.0,
        }
    }
    pub fn price(&self) -> u16 {
        match self {
            Self::SubWeaponBody(x) => x.price,
            Self::SubWeaponAmmo(x) => x.price,
            Self::Equipment(x) => x.price,
            Self::Rom(x) => x.price,
        }
    }
    pub fn count(&self) -> Option<NonZero<u8>> {
        match self {
            Self::SubWeaponBody(_) => None,
            Self::SubWeaponAmmo(x) => Some(x.count),
            Self::Equipment(_) => None,
            Self::Rom(_) => None,
        }
    }
    pub fn set_flag(&self) -> u16 {
        match self {
            Self::SubWeaponBody(x) => x.set_flag,
            Self::SubWeaponAmmo(x) => x.set_flag,
            Self::Equipment(x) => x.set_flag,
            Self::Rom(x) => x.set_flag,
        }
    }
}
