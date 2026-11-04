//! On-flash DB2 (`StgWindsor`) biometric database records, WINBIO SID identity encoding,
//! and RSA-signed flash write-enable / cleanup helpers (`0x45..0x4b`, `0x06`, `0x1a`).

use crate::error::{DriverError, Result, SensorStatus};
use crate::protocol::{ByteReader, SensorSession, WireOpcode, decode_hex_const};
use crate::usb::Transport;
use crate::{drv_debug, drv_info, drv_warn};

/// Default DB2 user storage container name on Synaptics Metallica MIS (`06cb:009a`).
pub const STG_WINDSOR_NAME: &str = "StgWindsor";

/// 3,621-byte RSA-signed `db_write_enable` (`0x06 0x02 ...`) command for `06cb:009a`,
/// decoded at compile time.
const DB_WRITE_ENABLE_009A: [u8; 3621] = decode_hex_const(
    "0602000001f48001074892b6c57deb7889b5ebf86bc3040f6d91ff1f68765f046591184be08cf36c154b7ec5368139d0f9532382214379aff3ffbfe4659e2f274e864bd0ad660f99e21da2bab677dbfa907a66ce110c180d2ddc5dfe40b8ed975cbedffc11631f12f8bd646a0ee82d44d2a6c1ec9cfbd40f485cb3d9124376b97b4a3349b0a730adda626d8ac28ec20e886aab1b8851deee3431c4d89c8bb3e787eaa9c0323dfe583d5424d36436e4435043e04fd4ea46b1fb2507ca6f0eb03baf27c84b819cbc96cec31a78045eb648339e2aa4789e7672d933936005f4720c8ffdc1ea23a4f30a1cdc8f6e87775c241b9ab1566f7771857cc4703d571f1106c526f95232925a6a93ec8e919022fbe303a515f9aaa8ca215072069311dd3f97d9a4f56259bab3a1b7a8582d6dc2f92d49f023d6f25a05837e1536a633e252ef645225f4293955041a0d54dcb1d1dd7e097b7839de5fde2a6ce999966d712b4cb2fd9d7830031da55d9faa99f866fbb7e520566efba43c2509286bf28e1a20c6a836db8a1fa4cb9b8d193780aab592d41653839670129066ac56f1268e6f761337f768555e13c5d68137c60f83dba8dc3863e00e73fd3af21e23a566daa67f3f14dd934e323651167021ca6b82a6103cb30be849446e2f54dde64a05377052b57332e9bf08a18cf52da2a13ebbd55e60333f8bc319e1457f38ec5d4839ec0ecd034825bdeaf64926858c6e8c2df418717b5f67135abc038835d3e4e1aa809546fd0d7f01066a71537f96bd1ecec3687583e1b511bf48c2776f4670158e56164c62da20f671764c785c352fc3cce22cefa20760acff8f45efb54a934f9834d54f9701dedacd4d383ac01f8cca92562eec774a58da6f55da252c491ee2ab58ff769f89a9649d3956682ca7d06bbf33f9a935b781dfc21b123b16694424e72d6a3e6781dcf195effd36470a4eab0fdc74e87102879ec81fea6549920cce454ac7813997b82d51e7b8c1ee24fad389904478f847654ec3a63bc595b9a7dde798db5c0b6f244901f239e7674c98eebb42b66e8956a733c3796586280a19a1df8a69224acd2556f7ec2e27cae37c69b332b2c0ec85991ae48722f988935f658b9cf32f46dfc6d96a5a36f18b6bf9f657b59b3da42414e4d56c0a24485aa298d2d0d1b177e7d0dafe602a4fb4f423def4bdb010fdc626c947587e19e7e4b0e6f9f2da41c29a8f1903d0d2803365fe0a113abba19220141d1ac7cec683962030d3f6591f98ea3dd09162715e5c12f40332b47c53164532827e5596fb2cc0aa8f31683cc63ec14c034c6f3d2c70b8c47611b4c5cb5348a2559fb162a780a2b403b0120a6846e27d6057a3ab9e1b18915ae2039e81cc6c50d2a14d5913617bacae78fe9b91e9e49d2e82def47565c12ff938b182f8ce941d2781b773479538c76ed9f7d4469f6fe5ba7f6e3ad98871b2866f0ef4f36277daa76c1042c83f77df0ff2e2639540bb355ea84273411c453081bd1e1035c402c53190d0bd905e8d01fc3787c65b69172cca5b234e92e358463bbb8d23e38c74a3a8e2735542b996ba5ec22c5095a777b6775a728df59c3560c7f36b83d55f819f196573f8fd356379fe9a5e7cecb376395e01309e2005b29e3b160cb74c6a5856093480dd06aea5fb3fbe23e004f8d7a38fd07866cdf24161391cc756f6ff71ff072e308b35e259435111bee09ddf2b8df99d0f2c2e8edaa4ecaabc6975a58f23bb6bfc94ebcbbba0d581f16be9d043c4e410b321c6df424ecaeea94edbe5801eb786199124222b091e5b33bad6761445a8a6606d0e781c07a6f91cd5fe188ddb9f9e17f5e07b0cba319c52e5fb03f53df570f82ddb603d305b72a2406bc7c1a37f920405f8f14d3ddf5d836ba68d83c1a8d7f1a41d148cc34b1ef996ddfb43ef19d2fbf0adcad301a473497739eaa10bbce85e15c32f1d90c8ab8605d0ae941eb914086592d087a521fde33a676cdfb94a4247f60f51edd37294511e92ec71a9a54bab68a0edaabdcb2c1a3adea778f416e39200af4c517dd7152bb72476c5d1413f04704615d795300f3a091214f4e4ac2ef41969c81f8fc08610864907b2e6edfa5fdb0926b6f064b2a1c3b8c7b631cc75663cedad5e71868abc9bac678e430144613cb0e51982b9e019099026b069bb7a4dc376cdd6a3c59566317976213672754fac87df85953cdc0de276fb8742f48ba218d4202fe6f865834152979d6da9b473e5d476c0aaa68491f545091b879c01986078d64fa5f49f60e615cb865f154f48b45173a1dc85f2eb1128652290bd383cdedcd8f280117e60be034ce224f92673934ed9e0077d5f7899f4e0eee097933a35e40f205d84a10733f492da619802ff70d9b949ca0c2bcb9ba68c290f2ef9a20a3bf496834c66956a8ec4179266999d9f87bdfc14aea865f0487e2be10a64becba69547d01658935e63703986a56d6ce38fe66dbf61d754ba9a1a278353913422e4f2e4100c5962999a3eaa3e1672bc73edcfcc7525a2d3dbe95683b4bf38f7444ac0f470f0e98079916e4e1fba3fcd5b082fc2772e63b5e0663f87638a163858f590845240a8c22dacf6f7999c431a2ab5204a7da7839c9a932608c7f83a87d1d73d7d8b2fec65abb95221fada44365fe21061dbcde52cb84cbfe9f061c4dab3be86002e7683eed16c23c687ce61c5d923ffbab40bee6ae93ed7f857f304e5eb16ec6d088563524e90d916e41a3a8c7777e29731f0f45c125082c423a5c02704c07c6fc19b1c4838ee3eabe12562829e67581d312c720b792a41744dec1e157426ab75136d31ee2f208147039091453c0b0e3970c5624d7a53df8076e9d1625d2c8e693e0e9a81e23862dca78921b66ca4c3c5ed35b0b5ed2e24622eb216ba0ba6e0c0eaf97c754eeb3db4a506d5854a3edc92d0111af3d2135a998729123f03d0f9366bb0d2c681cfc62c59bcd75c6b410d8e6997cca55c989f010393d6c242f7ce1ea71c6f262e498855584347b04ce26cce2e822b8c6b7b4937148a45c947073b300f7c72b6e78c4231078d80531b7f9317c1bb4d6070f299e9a97731b1befeeec2dae0a1a0364568acbebab069a4b90147776ff7e7f79c1cc98b2fe6214792501554f4195783b0f9188ccfe96ad8cd29f5463409c2054e4a2496ee65eaa1fcda3d7764cd3e8431e44a2b05e64aa2f9fb0d13456bfea9c91ec2d90d0099e7e395dce818650dcaf8bdfe23b4c6443f5c690b18ead221a6c2bcd34572ffb83b3332eafde6e25b37ff3ac6da0c3cc697b996265caa5a53ce44570303d7d111f44c635119595c247e86a32083f2865501752f93e3024b2e2b6d82d0c03b745bfd809af7e8e1349d1a79bed51bba415064701a2a7890e8f39937c6d2f563b0747bd94f1b698624b4fd17dfdf68ffdc0450c26d771f8ff4fb01a26ff8f64eb5b6d9153f5ce29d9dfcf84ca230a4c212401b437d1137f83a44f7a98a9fd1bc3d883e6227ce369ed32a960550aa863f3d014de7494dead34fced1d7b4eab651d499033589446fb5a1564557d63e724941e77ae3f46b79703d06277d87356999b51f61893d31c7231b0c635f1d83ab38a0dce544f5f6803861d6e3d7e70d617ecc593920b1ab9006bdc7bff34a8b36a7601eb170a040156b4567ab37f55fdf2d466fca9374277322f21811d02c7bc599c9ed5c2b1fe7b6baa19b1b0a30f79f8641b97bf66491dca0b4c0341367aa5acec1398bb37c037d81ac2368db49c5d5720bbfb7466ba616c70c7d834286303047357da0e9a34fc14b00c17a0a02f6a62a5b52976b00ed67bb2d0aa1b4a8a93100b799e1839695bdae9b98e75c8df5d8340d158be60379a6f626af052ad55c5cea01f806048e937f87e01e725e67620364e511afd288b25953e9ade343b596068608190fa5c4df114c93d3c8deca929c066d8bae5ac2d607e3f94d68a5d3554827a64735a43c462bc3682cc1664411f592c9456f53da1026f51459a0cf20cc171b9b6bede47ce57d845dffe1025c6eb240615da151106a5601b75c24c673d6ea818d60c31f414aeaa55597b40cc4f2ed2b3850d366084a52513420b013695e2bfcb0dbfad0014975c67471a3807528d15730802a4428842c6368c72650b3161265d6b86007264cf093a317fee4ee388e7721a02434c514324cbf85cb57f709b53fdf69624adc29b85518f1a051f2473ad9384d7ac57c2a780ab72506ba925fa39992dd2d0b00ffd8c38645d55c2ca2ae94cf4ffa372284a28a13797e25eb0d950c08371656a889e6189f83b9c0c8e06952b34fe13ccb5c3b2c82f2d988f6d9a233f1a9e64de97218be12ee7a8e8463d82131624ce167e744a3ca3915c78e6e76362d06090e2a7ed60ea7437c84838d8aa75f096c9a928e403e2428550c98de8c43bd25e24520aef6ee534ee1d4702175e6b25d030b87941845cefc1dc289cee33c72139f29839af81cb6a097d114311a01736f479bdae32a59398fc4a7494d034fc8dc5f2ba8af93fc4c576b703967ae593780413b44b98f4baba9d3fd7b55715ad5e5c41f9361a42a757d9a6d7220a9467e19f739877076164c142d40bbae9501312c394dc0233dc5868814162bfc1f10bd4663b285dd2d005f3bc3dad2ff023f7e81b799b1b323b37e82fc99dc81291cf93cc04a0e05aa674bcfd3bc0d930a10d0957ec7712b8cc78375dd904eb5a468296015dab1babb076786f305c8ad90ca3947b150da79cb94037e970e9180437ea34c72771d673000826741fe759fcdc2b03558331fdf5b89d6e3f25a05241f32f139e898126aec8b1715cac0208831fb1205f9efb75538755b2d83931c7ad9e252c88c8af3c5df62fb99653aff99e6c6c051a9a1241381cd5ce1307261f866575caea0a3e847286ecc67d7d9aa18f48ef2a5e5f183286127f8b9aa2caa0869ec5e474a70e5427dc2f0488b134d201241dae68ed3996869453247bb50d8bc3d3c909951e5a47b1e899610347ea8d7193346bfe754c289ad1ca454b9c92a07527b95a1fe508d0b7c8da5b9047d277508ff615c9dc9ab11596aa88d0c9734a45d81f0393219bead587d3a6f9d07c470f2abf8d7c6992228bf0ab6ef79e46599bb0a60",
);

/// Summary of the on-flash DB2 biometric database (`WireOpcode::Db2GetDbInfo` = `0x45`, 66-byte response).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Db2DatabaseInfo {
    pub version_major: u32,
    pub version_minor: u32,
    pub total_bytes: u32,
    pub used_bytes: u32,
    pub free_bytes: u32,
    pub total_objects: u16,
    pub root_ids: Vec<u16>,
}

impl Db2DatabaseInfo {
    /// Parse a [`Db2DatabaseInfo`] from the payload following the 2-byte status word of `0x45`.
    ///
    /// # Arguments
    /// * `payload` - Raw response bytes following the initial 2-byte status header of `0x45`.
    pub fn from_response_payload(payload: &[u8]) -> Result<Self> {
        let mut r = ByteReader::new(payload, "Db2GetDbInfo (0x45)");
        r.ensure(24)?;
        let version_major = r.read_u32_le()?;
        let version_minor = r.read_u32_le()?;
        let total_bytes = r.read_u32_le()?;
        let used_bytes = r.read_u32_le()?;
        let free_bytes = r.read_u32_le()?;
        let total_objects = r.read_u16_le()?;
        let nroots = r.read_u16_le()? as usize;

        let mut root_ids = Vec::with_capacity(nroots);
        for chunk in r.remaining_slice().as_chunks::<2>().0.iter().take(nroots) {
            root_ids.push(u16::from_le_bytes(*chunk));
        }

        Ok(Self {
            version_major,
            version_minor,
            total_bytes,
            used_bytes,
            free_bytes,
            total_objects,
            root_ids,
        })
    }
}

/// Enrolled finger entry inside a DB2 user record (`0x4a`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Db2FingerEntry {
    pub dbid: u16,
    pub subtype: u16,
    pub storage_id: u16,
    pub value_size: u16,
}

impl Db2FingerEntry {
    /// Return the human-readable fprintd/WINBIO finger name for this entry's [`subtype`](Self::subtype).
    ///
    /// # Arguments
    /// * `&self` - Reference to the [`Db2FingerEntry`].
    pub fn finger_name(&self) -> &'static str {
        finger_subtype_name(self.subtype)
    }
}

/// User record and enrolled fingers returned by [`SensorSession::db2_get_user`] (`0x4a`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Db2UserRecord {
    pub record_id: u16,
    pub sid_string: Option<String>,
    pub identity_bytes: Vec<u8>,
    pub fingers: Vec<Db2FingerEntry>,
}

/// Storage container (`StgWindsor`) returned by [`SensorSession::db2_get_user_storage`] (`0x4b`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Db2UserStorage {
    pub record_id: u16,
    pub name: String,
    pub users: Vec<(u16, u16)>,
}

/// Map a finger subtype ID (`1..=10` or `0xf5..=0xfe`) to its fprintd/WINBIO human-readable name.
///
/// # Arguments
/// * `subtype` - Numeric Synaptics/WINBIO finger subtype identifier (`1..=10` or `0xf5..=0xfe`).
pub fn finger_subtype_name(subtype: u16) -> &'static str {
    match subtype {
        1 => "right-thumb",
        2 => "right-index-finger",
        3 => "right-middle-finger",
        4 => "right-ring-finger",
        5 => "right-little-finger",
        6 => "left-thumb",
        7 => "left-index-finger",
        8 => "left-middle-finger",
        9 => "left-ring-finger",
        10 => "left-little-finger",
        0xf5 => "WINBIO_FINGER_UNSPECIFIED_POS_01",
        0xf6 => "WINBIO_FINGER_UNSPECIFIED_POS_02",
        0xf7 => "WINBIO_FINGER_UNSPECIFIED_POS_03",
        0xf8 => "WINBIO_FINGER_UNSPECIFIED_POS_04",
        0xf9 => "WINBIO_FINGER_UNSPECIFIED_POS_05",
        0xfa => "WINBIO_FINGER_UNSPECIFIED_POS_06",
        0xfb => "WINBIO_FINGER_UNSPECIFIED_POS_07",
        0xfc => "WINBIO_FINGER_UNSPECIFIED_POS_08",
        0xfd => "WINBIO_FINGER_UNSPECIFIED_POS_09",
        0xfe => "WINBIO_FINGER_UNSPECIFIED_POS_10",
        _ => "unknown-finger",
    }
}

/// Format the canonical Windows Biometric Framework (`WINBIO`) SID string for a Linux `uid`.
///
/// # Arguments
/// * `uid` - Numeric Linux user ID (e.g. `1000`).
pub fn format_sid_for_uid(uid: u32) -> String {
    format!("S-1-5-21-111111111-1111111111-1111111111-{uid}")
}

/// Parse a DB2 identity byte blob (`[3u32, len: u32, sid_bytes...]`) into a `S-1-5-21-...` string.
///
/// # Arguments
/// * `b` - Raw 76-byte (`0x4c`) DB2 user identity payload read from `0x4a`.
pub fn parse_sid_identity(b: &[u8]) -> Option<String> {
    let mut r = ByteReader::new(b, "SID identity");
    let typ = r.read_u32_le().ok()?;
    if typ != 3 {
        return None;
    }
    let len = r.read_u32_le().ok()? as usize;
    if len < 8 {
        return None;
    }
    let sid = r.read_slice(len).ok()?;
    let revision = sid[0];
    let subcnt = sid[1] as usize;
    if sid.len() < 8 + 4 * subcnt {
        return None;
    }
    let mut auth: u64 = 0;
    for &byte in &sid[2..8] {
        auth = (auth << 8) | (byte as u64);
    }
    let mut parts = Vec::with_capacity(subcnt);
    for chunk in sid[8..8 + 4 * subcnt].as_chunks::<4>().0 {
        parts.push(u32::from_le_bytes(*chunk).to_string());
    }
    Some(format!("S-{revision}-{auth}-{}", parts.join("-")))
}

/// Encode a Linux numeric `uid` into the 76-byte (`0x4c`) padded DB2 SID identity blob
/// (`S-1-5-21-111111111-1111111111-1111111111-<uid>`).
///
/// # Arguments
/// * `uid` - Numeric Linux user ID (e.g. `1000`) to embed as the final SID sub-authority.
pub fn encode_sid_identity_for_uid(uid: u32) -> Vec<u8> {
    let subauth: [u32; 5] = [21, 111_111_111, 1_111_111_111, 1_111_111_111, uid];
    let mut sid = Vec::with_capacity(8 + subauth.len() * 4);
    sid.push(1u8); // revision = 1
    sid.push(subauth.len() as u8);
    sid.extend_from_slice(&[0, 0, 0, 0, 0, 5]); // authority = 5 (big-endian 48-bit)
    for &sa in &subauth {
        sid.extend_from_slice(&sa.to_le_bytes());
    }
    let mut out = Vec::with_capacity(0x4c);
    out.extend_from_slice(&3u32.to_le_bytes());
    out.extend_from_slice(&(sid.len() as u32).to_le_bytes());
    out.extend_from_slice(&sid);
    if out.len() < 0x4c {
        out.resize(0x4c, 0);
    }
    out
}

/// Pack a completed enrollment `template` and `final_tid` into the DB2 type `0x000b` finger record payload.
///
/// # Arguments
/// * `subtype` - Numeric finger subtype (`1..=10`).
/// * `template` - Encrypted biometric template blob returned by the final `0x6b` enrollment step.
/// * `final_tid` - Template identifier (`tag 3`) returned by the final `0x6b` enrollment step.
pub fn encode_db2_template_record(subtype: u16, template: &[u8], final_tid: &[u8]) -> Vec<u8> {
    let mut inner = Vec::with_capacity(8 + template.len() + final_tid.len());
    inner.extend_from_slice(&1u16.to_le_bytes());
    inner.extend_from_slice(&(template.len() as u16).to_le_bytes());
    inner.extend_from_slice(template);
    inner.extend_from_slice(&2u16.to_le_bytes());
    inner.extend_from_slice(&(final_tid.len() as u16).to_le_bytes());
    inner.extend_from_slice(final_tid);

    let mut tinfo = Vec::with_capacity(8 + inner.len() + 0x20);
    tinfo.extend_from_slice(&subtype.to_le_bytes());
    tinfo.extend_from_slice(&3u16.to_le_bytes());
    tinfo.extend_from_slice(&(inner.len() as u16).to_le_bytes());
    tinfo.extend_from_slice(&0x0020u16.to_le_bytes());
    tinfo.extend_from_slice(&inner);
    tinfo.extend_from_slice(&[0u8; 0x20]);
    tinfo
}

impl<T: Transport> SensorSession<T> {
    /// Query the sensor's on-flash DB2 biometric database statistics (`WireOpcode::Db2GetDbInfo` = `0x45`).
    ///
    /// # Arguments
    /// * `&mut self` - Active [`SensorSession`].
    pub fn db2_get_info(&mut self) -> Result<Db2DatabaseInfo> {
        let payload = self.execute_cmd(&[WireOpcode::Db2GetDbInfo as u8], 0x42)?;
        Db2DatabaseInfo::from_response_payload(&payload)
    }

    /// Query a user record (`type = 5`) and its enrolled fingers (`0x4a`, 7B req `[0x4a, dbid_lo, dbid_hi, 0, 0, 0, 0]`).
    ///
    /// # Arguments
    /// * `&mut self` - Active [`SensorSession`].
    /// * `user_dbid` - 16-bit DB2 object ID of the user record to inspect.
    pub fn db2_get_user(&mut self, user_dbid: u16) -> Result<Db2UserRecord> {
        let mut req = [0u8; 7];
        req[0] = WireOpcode::Db2GetObjectInfo as u8;
        req[1..3].copy_from_slice(&user_dbid.to_le_bytes());

        let payload = self.execute_cmd(&req, 256)?;
        let mut r = ByteReader::new(&payload, "Db2GetObjectInfo (0x4a)");
        r.ensure(8)?;
        let record_id = r.read_u16_le()?;
        let fingercnt = r.read_u16_le()? as usize;
        let _reserved = r.read_u16_le()?;
        let identitysz = r.read_u16_le()? as usize;

        let ftab_bytes = r.read_slice((8 * fingercnt).min(r.remaining_len()))?;
        let mut fingers = Vec::with_capacity(fingercnt);
        for chunk in ftab_bytes.as_chunks::<8>().0 {
            fingers.push(Db2FingerEntry {
                dbid: u16::from_le_bytes([chunk[0], chunk[1]]),
                subtype: u16::from_le_bytes([chunk[2], chunk[3]]),
                storage_id: u16::from_le_bytes([chunk[4], chunk[5]]),
                value_size: u16::from_le_bytes([chunk[6], chunk[7]]),
            });
        }

        let identity_bytes = r.read_slice(identitysz.min(r.remaining_len()))?.to_vec();
        let sid_string = parse_sid_identity(&identity_bytes);

        Ok(Db2UserRecord {
            record_id,
            sid_string,
            identity_bytes,
            fingers,
        })
    }

    /// Query a DB2 user storage container (e.g. `"StgWindsor"`, opcode `0x4b`).
    ///
    /// # Arguments
    /// * `&mut self` - Active [`SensorSession`].
    /// * `dbid` - Storage object ID (`0` when looking up by `name`).
    /// * `name` - Null-terminated storage container name (e.g. [`STG_WINDSOR_NAME`]).
    pub fn db2_get_user_storage(&mut self, dbid: u16, name: &str) -> Result<Db2UserStorage> {
        let mut name_bytes = name.as_bytes().to_vec();
        if !name_bytes.is_empty() {
            name_bytes.push(0);
        }
        let mut req = Vec::with_capacity(5 + name_bytes.len());
        req.push(WireOpcode::Db2GetStorage as u8);
        req.extend_from_slice(&dbid.to_le_bytes());
        req.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        req.extend_from_slice(&name_bytes);

        let payload = self.execute_cmd(&req, 256)?;
        let mut r = ByteReader::new(&payload, "Db2GetStorage (0x4b)");
        r.ensure(8)?;
        let record_id = r.read_u16_le()?;
        let usercnt = r.read_u16_le()? as usize;
        let namesz = r.read_u16_le()? as usize;
        let _reserved = r.read_u16_le()?;

        let utab_bytes = r.read_slice((4 * usercnt).min(r.remaining_len()))?;
        let mut users = Vec::with_capacity(usercnt);
        for chunk in utab_bytes.as_chunks::<4>().0 {
            let uid = u16::from_le_bytes([chunk[0], chunk[1]]);
            let vsz = u16::from_le_bytes([chunk[2], chunk[3]]);
            users.push((uid, vsz));
        }

        let name_slice = r.read_slice(namesz.min(r.remaining_len()))?;
        let parsed_name = String::from_utf8_lossy(name_slice)
            .trim_end_matches('\0')
            .to_string();

        Ok(Db2UserStorage {
            record_id,
            name: parsed_name,
            users,
        })
    }

    /// Delete an object from the on-flash DB2 database (`WireOpcode::Db2DeleteObject` = `0x48`,
    /// 3B req `[0x48, obj_id_lo, obj_id_hi]` -> 2B resp).
    ///
    /// # Arguments
    /// * `&mut self` - Active [`SensorSession`].
    /// * `obj_id` - 16-bit DB2 object ID (`dbid`) to delete from sensor flash.
    pub fn db2_delete_object(&mut self, obj_id: u16) -> Result<()> {
        let mut req = [0u8; 3];
        req[0] = WireOpcode::Db2DeleteObject as u8;
        req[1..3].copy_from_slice(&obj_id.to_le_bytes());
        match self.execute_cmd(&req, 2) {
            Ok(_) => {
                drv_info!("Deleted DB2 object dbid=0x{obj_id:04x} ({obj_id}) from sensor flash");
                Ok(())
            }
            Err(DriverError::CommandFailed {
                status: SensorStatus::ObjectNotFound,
                ..
            }) => {
                drv_debug!("DB2 object dbid=0x{obj_id:04x} already absent (0x04b3)");
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Delete all enrolled DB2 template objects matching `(uid, subtype)` inside `StgWindsor`,
    /// returning the number of deleted templates.
    ///
    /// # Arguments
    /// * `&mut self` - Active [`SensorSession`].
    /// * `uid` - Numeric Linux user ID whose enrolled template(s) should be removed.
    /// * `subtype` - Numeric finger subtype (`1..=10`) to match.
    pub fn db2_delete_user_finger(&mut self, uid: u32, subtype: u16) -> Result<usize> {
        let target_sid = format_sid_for_uid(uid);
        let stg = self.db2_get_user_storage(0, STG_WINDSOR_NAME)?;
        let mut deleted = 0usize;
        for (user_dbid, _) in &stg.users {
            if let Ok(usr) = self.db2_get_user(*user_dbid)
                && usr.sid_string.as_deref() == Some(target_sid.as_str())
            {
                for f in usr.fingers.iter().filter(|f| f.subtype == subtype) {
                    drv_warn!(
                        "Removing DB2 template dbid=0x{:04x} for UID {uid}, subtype={subtype} ({})",
                        f.dbid,
                        finger_subtype_name(subtype)
                    );
                    self.db2_delete_object(f.dbid)?;
                    deleted += 1;
                }
            }
        }
        Ok(deleted)
    }

    /// Send the 3,621-byte RSA-signed `db_write_enable` (`0x06 0x02 ...`) command required before
    /// modifying DB2 flash records or running `0x6b` enrollment updates on `06cb:009a`.
    ///
    /// # Arguments
    /// * `&mut self` - Active [`SensorSession`].
    pub fn db_write_enable(&mut self) -> Result<()> {
        let _ = self.execute_cmd(&DB_WRITE_ENABLE_009A, 16)?;
        Ok(())
    }

    /// Send flash cleanup/commit command `0x1a` (ignoring `0x0491` `NothingToCommit`).
    ///
    /// # Arguments
    /// * `&mut self` - Active [`SensorSession`].
    pub fn db_call_cleanups(&mut self) -> Result<()> {
        let raw = self.execute_raw(&[WireOpcode::DbCleanup as u8], 16)?;
        if raw.len() >= 2 {
            let status = SensorStatus::from(u16::from_le_bytes([raw[0], raw[1]]));
            if status.is_ok() || status == SensorStatus::NothingToCommit {
                return Ok(());
            }
            return Err(DriverError::CommandFailed {
                opcode: WireOpcode::DbCleanup as u8,
                opcode_name: "DbCleanup",
                status,
            });
        }
        Ok(())
    }

    /// Create a new DB2 record (`0x47`) wrapped with `db_write_enable` (`0x06`) and `db_call_cleanups` (`0x1a`).
    ///
    /// # Arguments
    /// * `&mut self` - Active [`SensorSession`].
    /// * `parent_id` - Parent DB2 object ID (`storage_id` for user records, `user_dbid` for finger templates).
    /// * `typ` - DB2 record type (`5` for user record, `0x000b` for finger template record).
    /// * `storage_id` - Containing storage object ID (e.g. `StgWindsor` `record_id`).
    /// * `data` - Serialized record payload bytes.
    pub fn db2_new_record(
        &mut self,
        parent_id: u16,
        typ: u16,
        storage_id: u16,
        data: &[u8],
    ) -> Result<u16> {
        let _ = self.db2_get_info()?;
        self.db_write_enable()?;
        let res = (|| -> Result<u16> {
            let mut req = Vec::with_capacity(9 + data.len());
            req.push(WireOpcode::Db2WriteObject as u8);
            req.extend_from_slice(&parent_id.to_le_bytes());
            req.extend_from_slice(&typ.to_le_bytes());
            req.extend_from_slice(&storage_id.to_le_bytes());
            req.extend_from_slice(&(data.len() as u16).to_le_bytes());
            req.extend_from_slice(data);
            let payload = self.execute_cmd(&req, 16)?;
            let mut r = ByteReader::new(&payload, "Db2WriteObject (0x47)");
            r.read_u16_le()
        })();
        let _ = self.db_call_cleanups();
        res
    }

    /// Look up the DB2 user record (`type = 5`) for `uid` in `StgWindsor`, or create it if absent.
    ///
    /// # Arguments
    /// * `&mut self` - Active [`SensorSession`].
    /// * `uid` - Numeric Linux user ID to look up or create in `StgWindsor`.
    pub fn db2_lookup_or_create_user(&mut self, uid: u32) -> Result<(u16, u16)> {
        let stg = self.db2_get_user_storage(0, STG_WINDSOR_NAME)?;
        let id_data = encode_sid_identity_for_uid(uid);

        let mut req = Vec::with_capacity(7 + id_data.len());
        req.push(WireOpcode::Db2GetObjectInfo as u8);
        req.extend_from_slice(&0u16.to_le_bytes());
        req.extend_from_slice(&stg.record_id.to_le_bytes());
        req.extend_from_slice(&(id_data.len() as u16).to_le_bytes());
        req.extend_from_slice(&id_data);

        let raw = self.execute_raw(&req, 256)?;
        let mut r = ByteReader::new(&raw, "db2_lookup_or_create_user (0x4a)");
        let status = SensorStatus::from(r.read_u16_le()?);
        if status == SensorStatus::ObjectNotFound {
            drv_info!("Creating new DB2 user record in StgWindsor for UID {uid}");
            let user_dbid = self.db2_new_record(stg.record_id, 5, stg.record_id, &id_data)?;
            return Ok((user_dbid, stg.record_id));
        } else if status.is_ok() {
            let user_dbid = r.read_u16_le()?;
            return Ok((user_dbid, stg.record_id));
        }
        Err(DriverError::CommandFailed {
            opcode: WireOpcode::Db2GetObjectInfo as u8,
            opcode_name: "Db2GetObjectInfo",
            status,
        })
    }
}
