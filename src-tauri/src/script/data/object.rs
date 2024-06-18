use anyhow::{bail, Result};

use super::shop_items_data::{self, ShopItem};

pub struct MainWeapon {
    pub main_weapon_number: u8,
    pub flag: i32,
}

pub struct SubWeapon {
    pub sub_weapon_number: u8,
    pub count: u16,
    pub flag: i32,
}

pub struct ChestItem {
    pub chest_item_number: i16,
    pub open_flag: i32,
    pub flag: i32,
}

pub struct Seal {
    pub seal_number: u8,
    pub flag: i32,
}

pub struct Shop {
    pub talk_number: i32,
    pub items: (ShopItem, ShopItem, ShopItem),
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct LMStart {
    pub number: i32,
    pub value: bool,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Object {
    pub number: u16,
    pub x: i32,
    pub y: i32,
    pub op1: i32,
    pub op2: i32,
    pub op3: i32,
    pub op4: i32,
    pub starts: Vec<LMStart>,
}

impl Object {
    pub fn to_main_weapon(&self) -> Result<Option<MainWeapon>> {
        if self.number != 77 {
            return Ok(None);
        }
        Ok(Some(MainWeapon {
            main_weapon_number: u8::try_from(self.op1)?,
            flag: self.op2,
        }))
    }

    pub fn to_sub_weapon(&self) -> Result<Option<SubWeapon>> {
        if self.number != 13 {
            return Ok(None);
        }
        Ok(Some(SubWeapon {
            sub_weapon_number: u8::try_from(self.op1)?,
            count: u16::try_from(self.op2)?,
            flag: self.op3,
        }))
    }

    pub fn to_chest_item(&self) -> Result<Option<ChestItem>> {
        if self.number != 1 {
            return Ok(None);
        }
        Ok(Some(ChestItem {
            chest_item_number: i16::try_from(self.op2)?,
            open_flag: self.op1,
            flag: self.op3,
        }))
    }

    pub fn to_seal(&self) -> Result<Option<Seal>> {
        if self.number != 71 {
            return Ok(None);
        }
        Ok(Some(Seal {
            seal_number: u8::try_from(self.op1)?,
            flag: self.op2,
        }))
    }

    pub fn to_shop(&self, talks: &[String]) -> Result<Option<Shop>> {
        if !(self.number == 14 && self.op1 <= 99) {
            return Ok(None);
        }
        Ok(Some(Shop {
            talk_number: self.op4,
            items: shop_items_data::parse(&talks[usize::try_from(self.op4)?])?,
        }))
    }

    pub fn get_item_flag(&self) -> Result<i32> {
        Ok(match self.number {
            77 => self.to_main_weapon()?.unwrap().flag,
            13 => self.to_sub_weapon()?.unwrap().flag,
            1 => self.to_chest_item()?.unwrap().flag,
            71 => self.to_seal()?.unwrap().flag,
            _ => bail!("invalid number: {}", self.number),
        })
    }
}
