//! Packed item / weapon codec.
//!
//! Ports `FileFormats/Items/PackedDataHelper.cs`, `PackedItem.cs`,
//! `PackedWeapon.cs`, `PackedAssetReference.cs` and the `AssetLibraryManager`
//! `Decode`/`Encode` bit helpers.
//!
//! A save stores each item as a small obfuscated blob (`InventorySerialNumber`).
//! The blob is: a bit-packed header (version, is-weapon, unique id, 0xFFFF
//! marker, asset-library set id) followed by the packed asset references, padded
//! to 40 bytes with 0xFF, checksummed with a 16-bit fold of the 40-byte CRC-32,
//! trimmed of trailing 0xFF, then XOR-obfuscated with a seed derived from the
//! unique id.

use crate::bitio::{BitReader, BitWriter};
use crate::crc32::crc32;
use crate::error::{Error, Result};
use crate::gameinfo::{AssetConfig, AssetGroup, AssetLibraryConfigs, AssetLibraryManager};

/// A packed asset reference (asset index within a sublibrary).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PackedAssetReference {
    pub asset_index: i32,
    pub sublibrary_index: i32,
    pub use_set_id: bool,
}

impl PackedAssetReference {
    pub const NONE: PackedAssetReference = PackedAssetReference {
        asset_index: -1,
        sublibrary_index: -1,
        use_set_id: false,
    };

    pub fn is_none(&self) -> bool {
        *self == Self::NONE
    }

    fn decode(reader: &mut BitReader, config: AssetConfig) -> Result<Self> {
        let index = reader.read_uint32(config.total_bits() as usize)?;
        if index == config.none_index() {
            return Ok(Self::NONE);
        }
        let asset_index = (index & config.asset_mask()) as i32;
        let sublibrary_index = ((index >> config.asset_bits) & config.sublibrary_mask()) as i32;
        let use_set_id = ((index >> config.asset_bits) & config.use_set_id_mask()) != 0;
        Ok(Self {
            asset_index,
            sublibrary_index,
            use_set_id,
        })
    }

    fn encode(&self, writer: &mut BitWriter, config: AssetConfig) -> Result<()> {
        let index = if *self == Self::NONE {
            config.none_index()
        } else {
            ((self.asset_index as u32) & config.asset_mask())
                | (((self.sublibrary_index as u32) & config.sublibrary_mask()) << config.asset_bits)
                | if self.use_set_id {
                    1u32 << (config.asset_bits + config.sublibrary_bits - 1)
                } else {
                    0
                }
        };
        writer.write_uint32(index, config.total_bits() as usize)
    }
}

/// A decoded item's packed fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackedItem {
    pub item: PackedAssetReference,
    pub balance: PackedAssetReference,
    pub manufacturer: PackedAssetReference,
    pub manufacturer_grade_index: i32,
    pub game_stage: i32,
    pub alpha_part: PackedAssetReference,
    pub beta_part: PackedAssetReference,
    pub gamma_part: PackedAssetReference,
    pub delta_part: PackedAssetReference,
    pub epsilon_part: PackedAssetReference,
    pub zeta_part: PackedAssetReference,
    pub eta_part: PackedAssetReference,
    pub theta_part: PackedAssetReference,
    pub material_part: PackedAssetReference,
    pub prefix_part: PackedAssetReference,
    pub title_part: PackedAssetReference,
}

impl Default for PackedItem {
    fn default() -> Self {
        Self {
            item: PackedAssetReference::NONE,
            balance: PackedAssetReference::NONE,
            manufacturer: PackedAssetReference::NONE,
            manufacturer_grade_index: 0,
            game_stage: 0,
            alpha_part: PackedAssetReference::NONE,
            beta_part: PackedAssetReference::NONE,
            gamma_part: PackedAssetReference::NONE,
            delta_part: PackedAssetReference::NONE,
            epsilon_part: PackedAssetReference::NONE,
            zeta_part: PackedAssetReference::NONE,
            eta_part: PackedAssetReference::NONE,
            theta_part: PackedAssetReference::NONE,
            material_part: PackedAssetReference::NONE,
            prefix_part: PackedAssetReference::NONE,
            title_part: PackedAssetReference::NONE,
        }
    }
}

impl PackedItem {
    fn read(reader: &mut BitReader, configs: &AssetLibraryConfigs) -> Result<Self> {
        let item_parts = configs.config(AssetGroup::ItemParts);
        Ok(Self {
            item: PackedAssetReference::decode(reader, configs.config(AssetGroup::ItemTypes))?,
            balance: PackedAssetReference::decode(reader, configs.config(AssetGroup::BalanceDefs))?,
            manufacturer: PackedAssetReference::decode(
                reader,
                configs.config(AssetGroup::Manufacturers),
            )?,
            manufacturer_grade_index: reader.read_int32(7)?,
            game_stage: reader.read_int32(7)?,
            alpha_part: PackedAssetReference::decode(reader, item_parts)?,
            beta_part: PackedAssetReference::decode(reader, item_parts)?,
            gamma_part: PackedAssetReference::decode(reader, item_parts)?,
            delta_part: PackedAssetReference::decode(reader, item_parts)?,
            epsilon_part: PackedAssetReference::decode(reader, item_parts)?,
            zeta_part: PackedAssetReference::decode(reader, item_parts)?,
            eta_part: PackedAssetReference::decode(reader, item_parts)?,
            theta_part: PackedAssetReference::decode(reader, item_parts)?,
            material_part: PackedAssetReference::decode(reader, item_parts)?,
            prefix_part: PackedAssetReference::decode(reader, item_parts)?,
            title_part: PackedAssetReference::decode(reader, item_parts)?,
        })
    }

    fn write(&self, writer: &mut BitWriter, configs: &AssetLibraryConfigs) -> Result<()> {
        let item_parts = configs.config(AssetGroup::ItemParts);
        self.item
            .encode(writer, configs.config(AssetGroup::ItemTypes))?;
        self.balance
            .encode(writer, configs.config(AssetGroup::BalanceDefs))?;
        self.manufacturer
            .encode(writer, configs.config(AssetGroup::Manufacturers))?;
        writer.write_uint32(self.manufacturer_grade_index as u32, 7)?;
        writer.write_uint32(self.game_stage as u32, 7)?;
        for part in [
            self.alpha_part,
            self.beta_part,
            self.gamma_part,
            self.delta_part,
            self.epsilon_part,
            self.zeta_part,
            self.eta_part,
            self.theta_part,
            self.material_part,
            self.prefix_part,
            self.title_part,
        ] {
            part.encode(writer, item_parts)?;
        }
        Ok(())
    }
}

/// A decoded weapon's packed fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackedWeapon {
    pub weapon_type: PackedAssetReference,
    pub balance: PackedAssetReference,
    pub manufacturer: PackedAssetReference,
    pub manufacturer_grade_index: i32,
    pub game_stage: i32,
    pub body_part: PackedAssetReference,
    pub grip_part: PackedAssetReference,
    pub barrel_part: PackedAssetReference,
    pub sight_part: PackedAssetReference,
    pub stock_part: PackedAssetReference,
    pub elemental_part: PackedAssetReference,
    pub accessory1_part: PackedAssetReference,
    pub accessory2_part: PackedAssetReference,
    pub material_part: PackedAssetReference,
    pub prefix_part: PackedAssetReference,
    pub title_part: PackedAssetReference,
}

impl Default for PackedWeapon {
    fn default() -> Self {
        Self {
            weapon_type: PackedAssetReference::NONE,
            balance: PackedAssetReference::NONE,
            manufacturer: PackedAssetReference::NONE,
            manufacturer_grade_index: 0,
            game_stage: 0,
            body_part: PackedAssetReference::NONE,
            grip_part: PackedAssetReference::NONE,
            barrel_part: PackedAssetReference::NONE,
            sight_part: PackedAssetReference::NONE,
            stock_part: PackedAssetReference::NONE,
            elemental_part: PackedAssetReference::NONE,
            accessory1_part: PackedAssetReference::NONE,
            accessory2_part: PackedAssetReference::NONE,
            material_part: PackedAssetReference::NONE,
            prefix_part: PackedAssetReference::NONE,
            title_part: PackedAssetReference::NONE,
        }
    }
}

impl PackedWeapon {
    fn read(reader: &mut BitReader, configs: &AssetLibraryConfigs) -> Result<Self> {
        let weapon_parts = configs.config(AssetGroup::WeaponParts);
        Ok(Self {
            weapon_type: PackedAssetReference::decode(
                reader,
                configs.config(AssetGroup::WeaponTypes),
            )?,
            balance: PackedAssetReference::decode(reader, configs.config(AssetGroup::BalanceDefs))?,
            manufacturer: PackedAssetReference::decode(
                reader,
                configs.config(AssetGroup::Manufacturers),
            )?,
            manufacturer_grade_index: reader.read_int32(7)?,
            game_stage: reader.read_int32(7)?,
            body_part: PackedAssetReference::decode(reader, weapon_parts)?,
            grip_part: PackedAssetReference::decode(reader, weapon_parts)?,
            barrel_part: PackedAssetReference::decode(reader, weapon_parts)?,
            sight_part: PackedAssetReference::decode(reader, weapon_parts)?,
            stock_part: PackedAssetReference::decode(reader, weapon_parts)?,
            elemental_part: PackedAssetReference::decode(reader, weapon_parts)?,
            accessory1_part: PackedAssetReference::decode(reader, weapon_parts)?,
            accessory2_part: PackedAssetReference::decode(reader, weapon_parts)?,
            material_part: PackedAssetReference::decode(reader, weapon_parts)?,
            prefix_part: PackedAssetReference::decode(reader, weapon_parts)?,
            title_part: PackedAssetReference::decode(reader, weapon_parts)?,
        })
    }

    fn write(&self, writer: &mut BitWriter, configs: &AssetLibraryConfigs) -> Result<()> {
        let weapon_parts = configs.config(AssetGroup::WeaponParts);
        self.weapon_type
            .encode(writer, configs.config(AssetGroup::WeaponTypes))?;
        self.balance
            .encode(writer, configs.config(AssetGroup::BalanceDefs))?;
        self.manufacturer
            .encode(writer, configs.config(AssetGroup::Manufacturers))?;
        writer.write_uint32(self.manufacturer_grade_index as u32, 7)?;
        writer.write_uint32(self.game_stage as u32, 7)?;
        for part in [
            self.body_part,
            self.grip_part,
            self.barrel_part,
            self.sight_part,
            self.stock_part,
            self.elemental_part,
            self.accessory1_part,
            self.accessory2_part,
            self.material_part,
            self.prefix_part,
            self.title_part,
        ] {
            part.encode(writer, weapon_parts)?;
        }
        Ok(())
    }
}

/// A decoded packed slot (item or weapon) with its container-level identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackedData {
    pub is_weapon: bool,
    pub unique_id: i32,
    pub asset_library_set_id: i32,
    pub slot: PackedSlot,
}

impl PackedItem {
    /// Wrap a packed item blob in [`PackedData`] (asset library set 0).
    pub fn into_packed(self, unique_id: i32) -> PackedData {
        PackedData {
            is_weapon: false,
            unique_id,
            asset_library_set_id: 0,
            slot: PackedSlot::Item(self),
        }
    }
}

impl PackedWeapon {
    /// Wrap a packed weapon blob in [`PackedData`] (asset library set 0).
    pub fn into_packed(self, unique_id: i32) -> PackedData {
        PackedData {
            is_weapon: true,
            unique_id,
            asset_library_set_id: 0,
            slot: PackedSlot::Weapon(self),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackedSlot {
    Item(PackedItem),
    Weapon(PackedWeapon),
}

const MAX_BLOB: usize = 40;
const OBFUSCATION_INDEX: usize = 5;

/// Resolve a named slot to its asset group, without mutating anything.
pub fn slot_group(slot: &PackedSlot, name: &str) -> Option<AssetGroup> {
    match slot {
        PackedSlot::Weapon(_) => match name {
            "type" | "weapon_type" => Some(AssetGroup::WeaponTypes),
            "balance" => Some(AssetGroup::BalanceDefs),
            "manufacturer" | "mfr" => Some(AssetGroup::Manufacturers),
            "body" | "grip" | "barrel" | "sight" | "stock" | "elemental" | "accessory1"
            | "acc1" | "accessory2" | "acc2" | "material" | "prefix" | "title" => {
                Some(AssetGroup::WeaponParts)
            }
            _ => None,
        },
        PackedSlot::Item(_) => match name {
            "item" | "type" => Some(AssetGroup::ItemTypes),
            "balance" => Some(AssetGroup::BalanceDefs),
            "manufacturer" | "mfr" => Some(AssetGroup::Manufacturers),
            "alpha" | "beta" | "gamma" | "delta" | "epsilon" | "zeta" | "eta" | "theta"
            | "material" | "prefix" | "title" => Some(AssetGroup::ItemParts),
            _ => None,
        },
    }
}

fn slot_target<'a>(
    slot: &'a mut PackedSlot,
    name: &str,
) -> Option<(&'a mut PackedAssetReference, AssetGroup)> {
    match slot {
        PackedSlot::Weapon(w) => {
            let (r, g) = match name {
                "type" | "weapon_type" => (&mut w.weapon_type, AssetGroup::WeaponTypes),
                "balance" => (&mut w.balance, AssetGroup::BalanceDefs),
                "manufacturer" | "mfr" => (&mut w.manufacturer, AssetGroup::Manufacturers),
                "body" => (&mut w.body_part, AssetGroup::WeaponParts),
                "grip" => (&mut w.grip_part, AssetGroup::WeaponParts),
                "barrel" => (&mut w.barrel_part, AssetGroup::WeaponParts),
                "sight" => (&mut w.sight_part, AssetGroup::WeaponParts),
                "stock" => (&mut w.stock_part, AssetGroup::WeaponParts),
                "elemental" => (&mut w.elemental_part, AssetGroup::WeaponParts),
                "accessory1" | "acc1" => (&mut w.accessory1_part, AssetGroup::WeaponParts),
                "accessory2" | "acc2" => (&mut w.accessory2_part, AssetGroup::WeaponParts),
                "material" => (&mut w.material_part, AssetGroup::WeaponParts),
                "prefix" => (&mut w.prefix_part, AssetGroup::WeaponParts),
                "title" => (&mut w.title_part, AssetGroup::WeaponParts),
                _ => return None,
            };
            Some((r, g))
        }
        PackedSlot::Item(i) => {
            let (r, g) = match name {
                "item" | "type" => (&mut i.item, AssetGroup::ItemTypes),
                "balance" => (&mut i.balance, AssetGroup::BalanceDefs),
                "manufacturer" | "mfr" => (&mut i.manufacturer, AssetGroup::Manufacturers),
                "alpha" => (&mut i.alpha_part, AssetGroup::ItemParts),
                "beta" => (&mut i.beta_part, AssetGroup::ItemParts),
                "gamma" => (&mut i.gamma_part, AssetGroup::ItemParts),
                "delta" => (&mut i.delta_part, AssetGroup::ItemParts),
                "epsilon" => (&mut i.epsilon_part, AssetGroup::ItemParts),
                "zeta" => (&mut i.zeta_part, AssetGroup::ItemParts),
                "eta" => (&mut i.eta_part, AssetGroup::ItemParts),
                "theta" => (&mut i.theta_part, AssetGroup::ItemParts),
                "material" => (&mut i.material_part, AssetGroup::ItemParts),
                "prefix" => (&mut i.prefix_part, AssetGroup::ItemParts),
                "title" => (&mut i.title_part, AssetGroup::ItemParts),
                _ => return None,
            };
            Some((r, g))
        }
    }
}

impl PackedData {
    /// The item's game stage (the level the item is tuned to).
    pub fn game_stage(&self) -> i32 {
        match &self.slot {
            PackedSlot::Weapon(weapon) => weapon.game_stage,
            PackedSlot::Item(item) => item.game_stage,
        }
    }

    pub fn set_game_stage(&mut self, stage: i32) {
        match &mut self.slot {
            PackedSlot::Weapon(weapon) => weapon.game_stage = stage,
            PackedSlot::Item(item) => item.game_stage = stage,
        }
    }

    /// The manufacturer's grade index (drives the item's stat curve).
    pub fn manufacturer_grade(&self) -> i32 {
        match &self.slot {
            PackedSlot::Weapon(weapon) => weapon.manufacturer_grade_index,
            PackedSlot::Item(item) => item.manufacturer_grade_index,
        }
    }

    pub fn set_manufacturer_grade(&mut self, grade: i32) {
        match &mut self.slot {
            PackedSlot::Weapon(weapon) => weapon.manufacturer_grade_index = grade,
            PackedSlot::Item(item) => item.manufacturer_grade_index = grade,
        }
    }

    /// Set a named slot from a `package.asset` name (or `"None"`). Returns the
    /// asset group that was resolved.
    pub fn set_part(
        &mut self,
        manager: &AssetLibraryManager,
        slot: &str,
        name: &str,
    ) -> Result<AssetGroup> {
        let group = slot_group(&self.slot, slot)
            .ok_or_else(|| Error::PackedData(format!("unknown slot '{slot}'")))?;
        let (asset_index, sublibrary_index, use_set_id) =
            manager.find_asset(group, self.asset_library_set_id, name)?;
        let (target, _) = slot_target(&mut self.slot, slot).expect("slot validated above");
        *target = PackedAssetReference {
            asset_index,
            sublibrary_index,
            use_set_id,
        };
        Ok(group)
    }

    /// Read a named slot's reference and group.
    pub fn part(&self, slot: &str) -> Option<(PackedAssetReference, AssetGroup)> {
        let group = slot_group(&self.slot, slot)?;
        let r = match (&self.slot, slot) {
            (PackedSlot::Weapon(w), "type" | "weapon_type") => w.weapon_type,
            (PackedSlot::Weapon(w), "balance") => w.balance,
            (PackedSlot::Weapon(w), "manufacturer" | "mfr") => w.manufacturer,
            (PackedSlot::Weapon(w), "body") => w.body_part,
            (PackedSlot::Weapon(w), "grip") => w.grip_part,
            (PackedSlot::Weapon(w), "barrel") => w.barrel_part,
            (PackedSlot::Weapon(w), "sight") => w.sight_part,
            (PackedSlot::Weapon(w), "stock") => w.stock_part,
            (PackedSlot::Weapon(w), "elemental") => w.elemental_part,
            (PackedSlot::Weapon(w), "accessory1" | "acc1") => w.accessory1_part,
            (PackedSlot::Weapon(w), "accessory2" | "acc2") => w.accessory2_part,
            (PackedSlot::Weapon(w), "material") => w.material_part,
            (PackedSlot::Weapon(w), "prefix") => w.prefix_part,
            (PackedSlot::Weapon(w), "title") => w.title_part,
            (PackedSlot::Item(i), "item" | "type") => i.item,
            (PackedSlot::Item(i), "balance") => i.balance,
            (PackedSlot::Item(i), "manufacturer" | "mfr") => i.manufacturer,
            (PackedSlot::Item(i), "alpha") => i.alpha_part,
            (PackedSlot::Item(i), "beta") => i.beta_part,
            (PackedSlot::Item(i), "gamma") => i.gamma_part,
            (PackedSlot::Item(i), "delta") => i.delta_part,
            (PackedSlot::Item(i), "epsilon") => i.epsilon_part,
            (PackedSlot::Item(i), "zeta") => i.zeta_part,
            (PackedSlot::Item(i), "eta") => i.eta_part,
            (PackedSlot::Item(i), "theta") => i.theta_part,
            (PackedSlot::Item(i), "material") => i.material_part,
            (PackedSlot::Item(i), "prefix") => i.prefix_part,
            (PackedSlot::Item(i), "title") => i.title_part,
            _ => return None,
        };
        Some((r, group))
    }
}

fn seed_from(data: &[u8]) -> u32 {
    // BitConverter.ToUInt32(data, 1).Swap()
    u32::from_le_bytes([data[1], data[2], data[3], data[4]]).swap_bytes()
}

fn bogo_xor(seed: u32, temp: &mut [u8]) {
    let mut xor = ((seed as i32) >> 5) as u32;
    for byte in temp.iter_mut() {
        xor = ((u64::from(xor) * 0x10A8_60C1) % 0xFFFF_FFFB) as u32;
        *byte ^= (xor & 0xFF) as u8;
    }
}

fn bogo_encrypt(seed: u32, buffer: &mut [u8], offset: usize, length: usize) {
    if seed == 0 || length == 0 {
        return;
    }
    let right = ((seed % 32) as usize) % length;
    let left = length - right;
    let mut temp = vec![0u8; length];
    temp[left..left + right].copy_from_slice(&buffer[offset..offset + right]);
    temp[0..left].copy_from_slice(&buffer[offset + right..offset + length]);
    bogo_xor(seed, &mut temp);
    buffer[offset..offset + length].copy_from_slice(&temp);
}

fn bogo_decrypt(seed: u32, buffer: &mut [u8], offset: usize, length: usize) {
    if seed == 0 || length == 0 {
        return;
    }
    let mut temp = buffer[offset..offset + length].to_vec();
    bogo_xor(seed, &mut temp);
    let right = ((seed % 32) as usize) % length;
    let left = length - right;
    buffer[offset..offset + right].copy_from_slice(&temp[left..left + right]);
    buffer[offset + right..offset + length].copy_from_slice(&temp[0..left]);
}

fn fold_check(hash: u32) -> u16 {
    (((hash >> 16) ^ (hash & 0xFFFF)) & 0xFFFF) as u16
}

/// Decode an `InventorySerialNumber` blob.
pub fn unpack(data: &[u8], configs: &AssetLibraryConfigs) -> Result<PackedData> {
    if data.len() < 5 || data.len() > MAX_BLOB {
        return Err(Error::PackedData(format!(
            "invalid packed data length {}",
            data.len()
        )));
    }

    let seed = seed_from(data);
    let mut unobfuscated = data.to_vec();
    bogo_decrypt(
        seed,
        &mut unobfuscated,
        OBFUSCATION_INDEX,
        data.len() - OBFUSCATION_INDEX,
    );

    let file_check = u16::from_be_bytes([unobfuscated[5], unobfuscated[6]]);
    unobfuscated[5] = 0xFF;
    unobfuscated[6] = 0xFF;
    unobfuscated.resize(MAX_BLOB, 0xFF);

    let computed_check = fold_check(crc32(&unobfuscated));
    if file_check != computed_check {
        return Err(Error::PackedData("checksum failure in packed data".into()));
    }

    let mut reader = BitReader::new(&unobfuscated);
    let version = reader.read_uint32(7)? as u8;
    if version != configs.version {
        return Err(Error::PackedData(format!(
            "invalid version {version} in packed data (expected {})",
            configs.version
        )));
    }
    let is_weapon = reader.read_boolean()?;
    let unique_id = reader.read_int32(32)?;
    let marker = reader.read_uint32(16)?;
    if marker != 0xFFFF {
        return Err(Error::PackedData("invalid marker in packed data".into()));
    }
    let asset_library_set_id = reader.read_int32(8)?;

    let slot = if is_weapon {
        PackedSlot::Weapon(PackedWeapon::read(&mut reader, configs)?)
    } else {
        PackedSlot::Item(PackedItem::read(&mut reader, configs)?)
    };

    Ok(PackedData {
        is_weapon,
        unique_id,
        asset_library_set_id,
        slot,
    })
}

/// How to pad an encoded blob to 40 bytes before checksum/obfuscation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlobStyle {
    /// C# reference: 0xFF-fill unused bits and pad, then trim trailing 0xFF.
    Reference,
    /// Game-observed alternative: zero-fill unused bits, pad with 0x00, no trim.
    GameZero,
}

/// Re-encode to an `InventorySerialNumber` blob using the reference style.
pub fn pack(data: &PackedData, configs: &AssetLibraryConfigs) -> Result<Vec<u8>> {
    pack_styled(data, configs, BlobStyle::Reference)
}

/// Re-encode to an `InventorySerialNumber` blob with an explicit padding style.
pub fn pack_styled(
    data: &PackedData,
    configs: &AssetLibraryConfigs,
    style: BlobStyle,
) -> Result<Vec<u8>> {
    let mut writer = BitWriter::new();
    writer.write_uint32(configs.version as u32, 7)?;
    writer.write_boolean(data.is_weapon)?;
    writer.write_uint32(data.unique_id as u32, 32)?;
    writer.write_uint32(0xFFFF, 16)?;
    writer.write_uint32(data.asset_library_set_id as u32, 8)?;
    match &data.slot {
        PackedSlot::Weapon(weapon) => weapon.write(&mut writer, configs)?,
        PackedSlot::Item(item) => item.write(&mut writer, configs)?,
    }

    let bits = writer.position();
    let mut bytes = writer.get_buffer();
    if bytes.len() > MAX_BLOB {
        return Err(Error::PackedData("packed data exceeds 40 bytes".into()));
    }

    match style {
        BlobStyle::Reference => {
            bytes.resize(MAX_BLOB, 0xFF);
        }
        BlobStyle::GameZero => {
            if bits % 8 != 0 {
                let last = bytes.len() - 1;
                let used = (bits % 8) as u32;
                bytes[last] &= ((1u16 << used) - 1) as u8;
            }
            bytes.resize(MAX_BLOB, 0x00);
        }
    }

    let check = fold_check(crc32(&bytes));
    bytes[5] = (check >> 8) as u8;
    bytes[6] = (check & 0xFF) as u8;

    if style == BlobStyle::Reference {
        let mut end = bytes.len();
        let mut i = bytes.len().saturating_sub(1);
        while i > OBFUSCATION_INDEX {
            if bytes[i] != 0xFF {
                end = i + 1;
                break;
            }
            i -= 1;
        }
        bytes.truncate(end);
    }

    let seed = seed_from(&bytes);
    let n = bytes.len() - OBFUSCATION_INDEX;
    bogo_encrypt(seed, &mut bytes, OBFUSCATION_INDEX, n);
    Ok(bytes)
}

// ---------------------------------------------------------------------------
// Community `BL2(...)` item codes (base64 of an `InventorySerialNumber`).
// ---------------------------------------------------------------------------

/// A fresh, non-zero item unique id (Gibbed assigns one on create/duplicate).
pub fn random_unique_id() -> i32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x1234_5678);
    let mut x = nanos ^ 0x9E37_79B9_7F4A_7C15;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    let id = x as u32 as i32;
    if id == 0 {
        1
    } else {
        id
    }
}

/// Wrap an `InventorySerialNumber` in the community `BL2(base64)` form.
pub fn encode_code(serial: &[u8]) -> String {
    use base64::Engine as _;
    format!(
        "BL2({})",
        base64::engine::general_purpose::STANDARD.encode(serial)
    )
}

/// Extract and decode every `BL2(base64)` code in `text`.
pub fn decode_codes(text: &str) -> Vec<Vec<u8>> {
    use base64::Engine as _;
    let mut out = Vec::new();
    for (idx, _) in text.match_indices("BL2(") {
        let rest = &text[idx + 4..];
        let Some(end) = rest.find(')') else {
            continue;
        };
        let b64: String = rest[..end].chars().filter(|c| !c.is_whitespace()).collect();
        if b64.is_empty() {
            continue;
        }
        if let Ok(data) = base64::engine::general_purpose::STANDARD.decode(&b64) {
            if !data.is_empty() {
                out.push(data);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_code_round_trip() {
        let serial: Vec<u8> = (0u8..20).collect();
        let code = encode_code(&serial);
        assert!(code.starts_with("BL2("));
        assert_eq!(decode_codes(&format!("junk {code} trailing")), vec![serial.clone()]);
        assert_eq!(decode_codes(&format!("{code}\n{code}")).len(), 2);
        assert!(decode_codes("nothing here").is_empty());
        assert_ne!(random_unique_id(), 0);
    }

    #[test]
    fn check_fold() {
        assert_eq!(fold_check(0x1234_ABCD), 0xB9F9);
    }
    #[test]
    fn bogo_round_trip() {
        let mut buf: Vec<u8> = (0..30u8).collect();
        let original = buf.clone();
        let seed = 0x1234_5678;
        let n = buf.len() - 5;
        bogo_encrypt(seed, &mut buf, 5, n);
        assert_ne!(&buf[5..], &original[5..]);
        bogo_decrypt(seed, &mut buf, 5, n);
        assert_eq!(buf, original);
    }
}
