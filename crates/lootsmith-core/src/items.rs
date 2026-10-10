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
use crate::gameinfo::{AssetConfig, AssetGroup, AssetLibraryConfigs};

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackedSlot {
    Item(PackedItem),
    Weapon(PackedWeapon),
}

const MAX_BLOB: usize = 40;
const OBFUSCATION_INDEX: usize = 5;

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

#[cfg(test)]
mod tests {
    use super::*;

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
