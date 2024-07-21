use std::{any::type_name, collections::HashSet, fmt};

use super::item::StrategyFlag;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, num_derive::FromPrimitive)]
pub enum FieldId {
    Surface = 0,
    GateOfGuidance,
    MausoleumOfTheGiants,
    TempleOfTheSun,
    SpringInTheSky,
    InfernoCavern,
    ChamberOfExtinction,
    TwinLabyrinthsLeft,
    EndlessCorridor,
    ShrineOfTheMother,
    GateOfIllusion = 11,
    GraveyardOfTheGiants,
    TempleOfMoonlight,
    TowerOfTheGoddess,
    TowerOfRuin,
    ChamberOfBirth,
    TwinLabyrinthsRight,
    DimensionalCorridor,
    TrueShrineOfTheMother,
}

#[derive(Clone)]
pub struct SpotName(String);

impl SpotName {
    pub fn new(name: String) -> Self {
        Self(name)
    }

    pub fn get(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

#[derive(Clone, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct RequirementFlag(String);

impl RequirementFlag {
    pub fn new(requirement: String) -> Self {
        debug_assert!(
            !requirement.starts_with("sacredOrb:")
                || requirement
                    .split(':')
                    .nth(1)
                    .map_or(false, |x| x.parse::<u8>().is_ok())
        );
        Self(requirement)
    }

    pub fn is_sacred_orb(&self) -> bool {
        self.0.starts_with("sacredOrb:")
    }

    pub fn sacred_orb_count(&self) -> u8 {
        self.0.split(':').nth(1).unwrap().parse().unwrap()
    }

    pub fn get(&self) -> &str {
        self.0.as_str()
    }
}

impl PartialEq<StrategyFlag> for RequirementFlag {
    fn eq(&self, other: &StrategyFlag) -> bool {
        self.0 == other.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AllRequirements(pub Vec<RequirementFlag>);

#[derive(Clone, Debug, PartialEq)]
pub struct AnyOfAllRequirements(pub Vec<AllRequirements>);

#[derive(Clone)]
pub struct MainWeaponSpot {
    field_id: FieldId,
    src_idx: usize,
    name: SpotName,
    requirements: Option<AnyOfAllRequirements>,
}

impl MainWeaponSpot {
    pub fn src_idx(&self) -> usize {
        self.src_idx
    }
}

#[derive(Clone)]
pub struct SubWeaponSpot {
    field_id: FieldId,
    src_idx: usize,
    name: SpotName,
    requirements: Option<AnyOfAllRequirements>,
}

impl SubWeaponSpot {
    pub fn src_idx(&self) -> usize {
        self.src_idx
    }
}

#[derive(Clone)]
pub struct ChestSpot {
    field_id: FieldId,
    src_idx: usize,
    name: SpotName,
    requirements: Option<AnyOfAllRequirements>,
}

impl ChestSpot {
    pub fn src_idx(&self) -> usize {
        self.src_idx
    }
}

#[derive(Clone)]
pub struct SealSpot {
    field_id: FieldId,
    src_idx: usize,
    name: SpotName,
    requirements: Option<AnyOfAllRequirements>,
}

impl SealSpot {
    pub fn src_idx(&self) -> usize {
        self.src_idx
    }
}

#[derive(Clone)]
pub struct ShopSpot {
    field_id: FieldId,
    src_idx: usize,
    name: SpotName,
    requirements: Option<AnyOfAllRequirements>,
}

impl ShopSpot {
    pub fn new(
        field_id: FieldId,
        src_idx: usize,
        name: SpotName,
        requirements: Option<AnyOfAllRequirements>,
    ) -> Self {
        if cfg!(debug_assertions) {
            let names: Vec<_> = name.0.split(',').map(|x| x.trim()).collect();
            debug_assert_eq!(names.len(), 3);
        }
        Self {
            field_id,
            src_idx,
            name,
            requirements,
        }
    }

    pub fn src_idx(&self) -> usize {
        self.src_idx
    }

    pub fn to_strategy_flags(&self) -> (StrategyFlag, StrategyFlag, StrategyFlag) {
        let mut names = self.name.0.split(',').map(|x| x.trim());
        (
            StrategyFlag::new(names.next().unwrap().to_string()),
            StrategyFlag::new(names.next().unwrap().to_string()),
            StrategyFlag::new(names.next().unwrap().to_string()),
        )
    }
}

#[derive(Clone)]
pub enum Spot {
    MainWeapon(MainWeaponSpot),
    SubWeapon(SubWeaponSpot),
    Chest(ChestSpot),
    Seal(SealSpot),
    Shop(ShopSpot),
}

impl Spot {
    pub fn main_weapon(
        field_id: FieldId,
        src_idx: usize,
        name: SpotName,
        reqs: Option<AnyOfAllRequirements>,
    ) -> Self {
        Self::MainWeapon(MainWeaponSpot {
            field_id,
            src_idx,
            name,
            requirements: reqs,
        })
    }
    pub fn sub_weapon(
        field_id: FieldId,
        src_idx: usize,
        name: SpotName,
        reqs: Option<AnyOfAllRequirements>,
    ) -> Self {
        Self::SubWeapon(SubWeaponSpot {
            field_id,
            src_idx,
            name,
            requirements: reqs,
        })
    }
    pub fn chest(
        field_id: FieldId,
        src_idx: usize,
        name: SpotName,
        reqs: Option<AnyOfAllRequirements>,
    ) -> Self {
        Self::Chest(ChestSpot {
            field_id,
            src_idx,
            name,
            requirements: reqs,
        })
    }
    pub fn seal(
        field_id: FieldId,
        src_idx: usize,
        name: SpotName,
        reqs: Option<AnyOfAllRequirements>,
    ) -> Self {
        Self::Seal(SealSpot {
            field_id,
            src_idx,
            name,
            requirements: reqs,
        })
    }
    pub fn shop(shop: ShopSpot) -> Self {
        Self::Shop(shop)
    }

    pub fn field_id(&self) -> FieldId {
        match self {
            Self::MainWeapon(x) => x.field_id,
            Self::SubWeapon(x) => x.field_id,
            Self::Chest(x) => x.field_id,
            Self::Seal(x) => x.field_id,
            Self::Shop(x) => x.field_id,
        }
    }

    pub fn requirements(&self) -> Option<&AnyOfAllRequirements> {
        match self {
            Self::MainWeapon(x) => x.requirements.as_ref(),
            Self::SubWeapon(x) => x.requirements.as_ref(),
            Self::Chest(x) => x.requirements.as_ref(),
            Self::Seal(x) => x.requirements.as_ref(),
            Self::Shop(x) => x.requirements.as_ref(),
        }
    }

    pub fn requirements_mut(&mut self) -> &mut Option<AnyOfAllRequirements> {
        match self {
            Self::MainWeapon(x) => &mut x.requirements,
            Self::SubWeapon(x) => &mut x.requirements,
            Self::Chest(x) => &mut x.requirements,
            Self::Seal(x) => &mut x.requirements,
            Self::Shop(x) => &mut x.requirements,
        }
    }

    pub fn is_reachable(
        &self,
        current_strategy_flags: &HashSet<&str>,
        sacred_orb_count: u8,
    ) -> bool {
        let Some(any) = self.requirements() else {
            return true;
        };
        any.0.iter().any(|all| {
            all.0.iter().all(|x| {
                x.is_sacred_orb() && x.sacred_orb_count() <= sacred_orb_count
                    || current_strategy_flags.contains(x.get())
            })
        })
    }
}

impl fmt::Display for Spot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let full_type_name = match self {
            Self::MainWeapon(_) => type_name::<MainWeaponSpot>(),
            Self::SubWeapon(_) => type_name::<SubWeaponSpot>(),
            Self::Chest(_) => type_name::<ChestSpot>(),
            Self::Seal(_) => type_name::<SealSpot>(),
            Self::Shop(_) => type_name::<ShopSpot>(),
        };
        let src_idx = match self {
            Self::MainWeapon(x) => x.src_idx,
            Self::SubWeapon(x) => x.src_idx,
            Self::Chest(x) => x.src_idx,
            Self::Seal(x) => x.src_idx,
            Self::Shop(x) => x.src_idx,
        };
        let name = match self {
            Self::MainWeapon(x) => &x.name,
            Self::SubWeapon(x) => &x.name,
            Self::Chest(x) => &x.name,
            Self::Seal(x) => &x.name,
            Self::Shop(x) => &x.name,
        };
        write!(
            f,
            "{:?}_{}{}({})",
            self.field_id(),
            full_type_name.split("::").last().unwrap(),
            src_idx,
            name.get()
        )
    }
}
