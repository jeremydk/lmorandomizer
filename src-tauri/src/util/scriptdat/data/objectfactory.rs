use anyhow::Result;

use crate::{
    dataset::item::Item, randomizer::items::SubWeaponNumber,
    util::scriptdat::data::lm_object::LMStart,
};

use super::lm_object::LMObject;

pub fn to_object_for_shutter(old_obj: &LMObject, start_flag: i32, item: &Item) -> Result<LMObject> {
    Ok(match item.r#type.as_ref() {
        "mainWeapon" => create_main_weapon(old_obj, item)?,
        "subWeapon" => create_sub_weapon(old_obj, item)?,
        "equipment" => LMObject {
            number: 1,
            x: old_obj.x,
            y: old_obj.y,
            op1: 40,
            op2: item.number as i32,
            op3: item.flag,
            op4: -1,
            starts: starts_that_hide_when_startup(old_obj, start_flag)?,
        },
        "rom" => LMObject {
            number: 1,
            x: old_obj.x,
            y: old_obj.y,
            op1: 40,
            op2: 100 + item.number as i32,
            op3: item.flag,
            op4: -1,
            starts: starts_that_hide_when_startup(old_obj, start_flag)?,
        },
        "seal" => create_seal(old_obj, item)?,
        _ => unreachable!(),
    })
}

pub fn to_object_for_special_chest(old_obj: &LMObject, item: &Item) -> Result<LMObject> {
    Ok(match item.r#type.as_ref() {
        "mainWeapon" => create_main_weapon(old_obj, item)?,
        "subWeapon" => {
            debug_assert!(!(item.number == SubWeaponNumber::AnkhJewel as i8 && item.count > 1));
            create_sub_weapon(old_obj, item)?
        }
        "equipment" => LMObject {
            number: 1,
            x: old_obj.x,
            y: old_obj.y,
            op1: 40,
            op2: item.number as i32,
            op3: item.flag,
            op4: -1,
            starts: get_starts_without_old_flag(old_obj)?,
        },
        "rom" => LMObject {
            number: 1,
            x: old_obj.x,
            y: old_obj.y,
            op1: 40,
            op2: 100 + item.number as i32,
            op3: item.flag,
            op4: -1,
            starts: get_starts_without_old_flag(old_obj)?,
        },
        "seal" => create_seal(old_obj, item)?,
        _ => unreachable!(),
    })
}

pub fn to_objects_for_chest(old_obj: &LMObject, item: &Item) -> Result<Vec<LMObject>> {
    Ok(match item.r#type.as_ref() {
        "mainWeapon" => {
            debug_assert!(!(item.number == SubWeaponNumber::AnkhJewel as i8 && item.count > 1));
            create_main_weapon_chest(old_obj, item)?
        }
        "subWeapon" => {
            debug_assert!(!(item.number == SubWeaponNumber::AnkhJewel as i8 && item.count > 1));
            create_sub_weapon_chest(old_obj, item)?
        }
        "equipment" => vec![LMObject {
            number: 1,
            x: old_obj.x,
            y: old_obj.y,
            op1: old_obj.op1,
            op2: item.number as i32,
            op3: item.flag,
            op4: -1,
            starts: starts_as_is(old_obj, item)?,
        }],
        "rom" => vec![LMObject {
            number: 1,
            x: old_obj.x,
            y: old_obj.y,
            op1: old_obj.op1,
            op2: 100 + item.number as i32,
            op3: item.flag,
            op4: -1,
            starts: starts_as_is(old_obj, item)?,
        }],
        "seal" => create_seal_chest(old_obj, item)?,
        _ => unreachable!(),
    })
}

fn create_main_weapon(old_obj: &LMObject, item: &Item) -> Result<LMObject> {
    Ok(LMObject {
        number: 77,
        x: old_obj.x,
        y: old_obj.y,
        op1: item.number as i32,
        op2: item.flag,
        op3: -1,
        op4: -1,
        starts: starts_as_is(old_obj, item)?,
    })
}

fn create_main_weapon_chest(old_obj: &LMObject, item: &Item) -> Result<Vec<LMObject>> {
    Ok(vec![
        create_empty_chest(old_obj, item)?,
        LMObject {
            number: 77,
            x: old_obj.x,
            y: old_obj.y,
            op1: item.number as i32,
            op2: item.flag,
            op3: -1,
            op4: -1,
            starts: starts_that_hide_when_startup_and_taken(old_obj, item)?,
        },
    ])
}

fn create_sub_weapon(old_obj: &LMObject, item: &Item) -> Result<LMObject> {
    Ok(LMObject {
        number: 13,
        x: old_obj.x,
        y: old_obj.y,
        op1: item.number as i32,
        op2: item.count as i32,
        op3: item.flag,
        op4: -1,
        starts: starts_as_is(old_obj, item)?,
    })
}

fn create_sub_weapon_chest(old_obj: &LMObject, item: &Item) -> Result<Vec<LMObject>> {
    Ok(vec![
        create_empty_chest(old_obj, item)?,
        LMObject {
            number: 13,
            x: old_obj.x,
            y: old_obj.y,
            op1: item.number as i32,
            op2: item.count as i32,
            op3: item.flag,
            op4: -1,
            starts: starts_that_hide_when_startup_and_taken(old_obj, item)?,
        },
    ])
}

fn create_seal(old_obj: &LMObject, item: &Item) -> Result<LMObject> {
    Ok(LMObject {
        number: 71,
        x: old_obj.x,
        y: old_obj.y,
        op1: item.number as i32,
        op2: item.flag,
        op3: -1,
        op4: -1,
        starts: starts_as_is(old_obj, item)?,
    })
}

fn create_seal_chest(old_obj: &LMObject, item: &Item) -> Result<Vec<LMObject>> {
    Ok(vec![
        create_empty_chest(old_obj, item)?,
        LMObject {
            number: 71,
            x: old_obj.x,
            y: old_obj.y,
            op1: item.number as i32,
            op2: item.flag,
            op3: -1,
            op4: -1,
            starts: starts_that_hide_when_startup_and_taken(old_obj, item)?,
        },
    ])
}

fn create_empty_chest(old_obj: &LMObject, item: &Item) -> Result<LMObject> {
    Ok(LMObject {
        number: 1,
        x: old_obj.x,
        y: old_obj.y,
        op1: old_obj.op1,
        op2: -1,
        op3: old_obj.op1,
        op4: -1,
        starts: starts_as_is(old_obj, item)?,
    })
}

fn starts_that_hide_when_startup_and_taken(
    old_chest_obj: &LMObject,
    item: &Item,
) -> Result<Vec<LMStart>> {
    debug_assert_eq!(old_chest_obj.number, 1);
    let mut vec = vec![
        LMStart {
            number: 99999,
            value: true,
        },
        LMStart {
            number: old_chest_obj.as_chest_item()?.open_flag,
            value: true,
        },
        LMStart {
            number: item.flag,
            value: false,
        },
    ];
    vec.append(
        &mut get_starts_without_old_flag(old_chest_obj)?
            .into_iter()
            .filter(|x| x.number != 99999)
            .collect::<Vec<_>>(),
    );
    Ok(vec)
}

fn starts_that_hide_when_startup(old_obj: &LMObject, start_flag: i32) -> Result<Vec<LMStart>> {
    let mut vec = vec![
        LMStart {
            number: 99999,
            value: true,
        },
        LMStart {
            number: start_flag,
            value: true,
        },
    ];
    vec.append(
        &mut get_starts_without_old_flag(old_obj)?
            .into_iter()
            .filter(|x| x.number != 99999)
            .collect(),
    );
    Ok(vec)
}

fn starts_as_is(old_obj: &LMObject, item: &Item) -> Result<Vec<LMStart>> {
    let mut vec = get_starts_without_old_flag(old_obj)?;
    let item_flag = old_obj.get_item_flag()?;
    if old_obj.starts.iter().any(|x| x.number == item_flag) {
        vec.push(LMStart {
            number: item.flag,
            value: false,
        });
    }
    Ok(vec)
}

fn get_starts_without_old_flag(old_obj: &LMObject) -> Result<Vec<LMStart>> {
    let item_flag = old_obj.get_item_flag()?;
    Ok(old_obj
        .starts
        .iter()
        .filter(|x| x.number != item_flag)
        .map(|x| LMStart {
            number: x.number,
            value: x.value,
        })
        .collect())
}
