use super::*;

#[derive(Debug, Clone)]
pub struct BoardItem {
    pub position: vec2<Coord>,
    pub item_id: Id,
    pub turns_alive: usize,
    /// Whether the item was used this turn.
    pub used: bool,
}

#[derive(Debug, Clone)]
pub struct InventoryItem {
    /// The id of the board item, if it is present on the board.
    pub on_board: Option<Id>,
    pub kind: ItemKind,
    /// Permanent stats that persist through turns.
    pub perm_stats: ItemStats,
    /// Resolution stats that reset every turn, act as a modifier on the perm_stats.
    /// Call `current_stats()` to get relevant stats for the time.
    pub temp_stats: ItemStats,
}

#[derive(Debug, Clone, Default)]
pub struct ItemStats {
    pub damage: Option<i64>,
}

/// A reference to an item kind or a category of items.
/// Used in synergies.
#[derive(Debug, Clone, Copy)]
pub enum ItemRef {
    Category(ItemCategory),
    Specific(ItemKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemCategory {
    Spooky,
    Tech,
    Weapon,
    Treasure,
    Magic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemKind {
    Sword,
    Forge,
    Boots,
    Map,
    Camera,
    Ghost,
    FireScroll,
    SoulCrystal,
    RadiationCore,
    GreedyPot,
    SpiritCoin,
    Chest,
    MagicTreasureBag,
    ElectricRod,
    MagicWire,
    Melter,
    Phantom,
    CursedSkull,
    KingSkull,
    GoldenLantern,
    CharmingStaff,
    WarpPortal,
    Solitude,
}

impl InventoryItem {
    pub fn current_stats(&self) -> ItemStats {
        self.temp_stats.combine(&self.perm_stats)
    }
}

impl ItemRef {
    pub fn check(&self, item: ItemKind) -> bool {
        match self {
            ItemRef::Category(category) => item.categories().contains(category),
            ItemRef::Specific(kind) => item == *kind,
        }
    }
}

impl ItemKind {
    pub fn all() -> [ItemKind; 23] {
        use ItemKind::*;
        [
            Boots,
            Forge,
            Sword,
            Map,
            Camera,
            Ghost,
            FireScroll,
            SoulCrystal,
            RadiationCore,
            GreedyPot,
            SpiritCoin,
            Chest,
            MagicTreasureBag,
            ElectricRod,
            MagicWire,
            Melter,
            Phantom,
            CursedSkull,
            KingSkull,
            GoldenLantern,
            CharmingStaff,
            WarpPortal,
            Solitude,
        ]
    }

    pub fn categories(&self) -> Vec<ItemCategory> {
        use ItemCategory::*;
        match self {
            ItemKind::Boots => vec![Tech],
            ItemKind::Forge => vec![Magic],
            ItemKind::Sword => vec![Weapon],
            ItemKind::Map => vec![Treasure],
            ItemKind::Camera => vec![Tech],
            ItemKind::Ghost => vec![Spooky],
            ItemKind::FireScroll => vec![Magic, Weapon],
            ItemKind::SoulCrystal => vec![Spooky],
            ItemKind::RadiationCore => vec![Tech, Weapon],
            ItemKind::GreedyPot => vec![Treasure],
            ItemKind::SpiritCoin => vec![Spooky, Treasure],
            ItemKind::Chest => vec![Treasure],
            ItemKind::MagicTreasureBag => vec![Treasure, Magic],
            ItemKind::ElectricRod => vec![Tech, Weapon],
            ItemKind::MagicWire => vec![Magic, Tech],
            ItemKind::Melter => vec![Tech],
            ItemKind::Phantom => vec![Spooky, Weapon],
            ItemKind::CursedSkull => vec![Spooky],
            ItemKind::KingSkull => vec![Treasure, Weapon],
            ItemKind::GoldenLantern => vec![Treasure],
            ItemKind::CharmingStaff => vec![Magic, Weapon],
            ItemKind::WarpPortal => vec![Magic],
            ItemKind::Solitude => vec![Weapon],
        }
    }

    pub fn instantiate(self) -> InventoryItem {
        let damage = match self {
            ItemKind::Boots => None,
            ItemKind::Forge => None,
            ItemKind::Sword => Some(2),
            ItemKind::Map => None,
            ItemKind::Camera => None,
            ItemKind::Ghost => None,
            ItemKind::FireScroll => Some(5),
            ItemKind::SoulCrystal => Some(0),
            ItemKind::RadiationCore => Some(1),
            ItemKind::GreedyPot => Some(1),
            ItemKind::SpiritCoin => None,
            ItemKind::Chest => None,
            ItemKind::MagicTreasureBag => None,
            ItemKind::ElectricRod => Some(2),
            ItemKind::MagicWire => None,
            ItemKind::Melter => None,
            ItemKind::Phantom => Some(1),
            ItemKind::CursedSkull => None,
            ItemKind::KingSkull => Some(3),
            ItemKind::GoldenLantern => None,
            ItemKind::CharmingStaff => Some(0),
            ItemKind::WarpPortal => None,
            ItemKind::Solitude => Some(2),
        };
        InventoryItem {
            on_board: None,
            kind: self,
            perm_stats: ItemStats { damage },
            temp_stats: default(),
        }
    }
}

impl ItemStats {
    pub fn combine(&self, other: &Self) -> Self {
        fn combine<T: Add<T, Output = T>>(value: Option<T>, other: Option<T>) -> Option<T> {
            match value {
                Some(a) => match other {
                    Some(b) => Some(a + b),
                    None => Some(a),
                },
                None => other,
            }
        }

        Self {
            damage: combine(self.damage, other.damage),
        }
    }
}

impl Display for ItemKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ItemKind::Sword => "Sword",
            ItemKind::Forge => "Ancient forge",
            ItemKind::Boots => "Ultra speed boots",
            ItemKind::Map => "Grand map",
            ItemKind::Camera => "Camera",
            ItemKind::Ghost => "Ghost",
            ItemKind::FireScroll => "Fire scroll",
            ItemKind::SoulCrystal => "Soul crystal",
            ItemKind::RadiationCore => "Radiation core",
            ItemKind::GreedyPot => "Greedy pot",
            ItemKind::SpiritCoin => "Spirit coin",
            ItemKind::Chest => "Chest",
            ItemKind::MagicTreasureBag => "Magic treasure bag",
            ItemKind::ElectricRod => "Electric rod",
            ItemKind::MagicWire => "Magic wire",
            ItemKind::Melter => "Melter",
            ItemKind::Phantom => "Phantom",
            ItemKind::CursedSkull => "Cursed skull",
            ItemKind::KingSkull => "King's skull",
            ItemKind::GoldenLantern => "Golden lantern",
            ItemKind::CharmingStaff => "Charming staff",
            ItemKind::WarpPortal => "Warp portal",
            ItemKind::Solitude => "Solitude",
        };
        write!(f, "{}", name)
    }
}
