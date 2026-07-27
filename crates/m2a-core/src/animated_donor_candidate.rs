//! Exact M0 animated-donor candidates built from the static Meshy M0 geometry
//! and the project-owned animated Meshy H1 donor rig.

use std::fmt;

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    animated_donor::{
        AnimatedDonorRetargetReportV1, retarget_static_mesh_to_animated_donor_v1,
        retarget_static_mesh_to_animated_donor_v2, retarget_static_mesh_to_animated_donor_v3,
        retarget_static_mesh_to_animated_donor_v4, retarget_static_mesh_to_animated_donor_v5,
    },
    erf::ErfArchive,
    glb::{EmbeddedImageDecodeLimitsV1, GlbLimits, decode_embedded_image_to_tga_v1, ingest_glb},
    hak::{HakResourceInputV1, HakWriterOptionsV1},
    mdl::{
        MdlFormatProfileV1, MdlMaterialTextureBindingV1, MdlStateProjectionProfileV1,
        MdlWriterOptionsV1, NodeReport, verify_direct_creature_state_projection_v1,
        write_binary_mdl_with_animations,
    },
    model_pipeline::{
        M6ByteIdentityV1, materialize_direct_creature_runtime_clips,
        resolve_base_color_image_index_v1,
    },
    package::{PackageManifestV1, write_model_package_v1},
    profile_a::{
        AuroraCreatureNodeV1, ProfileAOptionsV1, RigSegmentDeformationV1,
        convert_profile_a_with_animations_v1, derive_meshy_h1_profile_and_mapping_v1,
    },
    proof_module::{
        BinaryCreatureModuleIdentityV1, BinaryCreatureMultiFixtureModuleReadbackV1,
        BinaryCreatureOwnedFixtureV1, M0RuntimeDirectionV1, M0RuntimePositionV1,
        build_binary_creature_multi_fixture_module_v1,
        inspect_binary_creature_multi_fixture_module_v1,
    },
    tga::{TgaWriterOptionsV1, write_tga_v1},
    two_da::{
        TwoDaAppendRequestV1, TwoDaCellAssignmentV1, TwoDaCellValueV1, TwoDaLimitsV1,
        append_two_da_row_v1, inspect_two_da_v2, read_two_da_row_v2,
    },
};

pub const M0_R33_MODULE_RESREF: &str = "m2a_m0r33";
pub const M0_R33_AREA_RESREF: &str = "m2a_m0a33";
pub const M0_R33_HAK_RESREF: &str = "m2a_m0r33";
pub const M0_R33_MODEL_RESREF: &str = "m2a_m0p33";
pub const M0_R33_TEXTURE_RESREF: &str = "m2a_m0t01";
pub const M0_R33_FIXTURE_ID: &str = "m0_fixture";
pub const M0_R33_FIXTURE_TEMPLATE_RESREF: &str = "nw_dwarfmerc001";
pub const M0_R33_FIXTURE_DISPLAY_NAME: &str = "Meshy M0 animated-donor fixture";
pub const M0_R33_APPEARANCE_LABEL: &str = "M2A_M0_MESHY_SKIN";
pub const M0_R33_APPEARANCE_ROW: u16 = 15_100;
pub const M0_R33_ACCEPTED_R32_PROOF_SHA256: &str =
    "1c787faf0780a44637264051ab4e049b02266d512db15145789113cabad53345";
pub const M0_R33_SOURCE_SHA256: &str =
    "aac32ee6197457653b9b247a8f360230cfa0709cb2b2989d85aa155e1699ece1";
pub const M0_R33_DONOR_SHA256: &str =
    "3071664994aec7d71f8a6fb8808587161dab9e09816f1a78b8562380e967485f";
pub const M0_R33_BASE_APPEARANCE_SHA256: &str =
    "815c0b3bce0895e9f17d4b92cb02a6d34366267b5a4b9081dece0f4eee7d7a1a";
pub const M0_R33_TEXTURE_SHA256: &str =
    "079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b";
pub const M0_R33_MODEL_SHA256: &str =
    "b82b6d7b9260a05cebb7bb93ed75f2938a210a01bd01bd397f1b63fc60fde25e";

pub const M0_R34_MODULE_RESREF: &str = "m2a_m0r34";
pub const M0_R34_AREA_RESREF: &str = "m2a_m0a34";
pub const M0_R34_HAK_RESREF: &str = "m2a_m0r34";
pub const M0_R34_MODEL_RESREF: &str = "m2a_m0p34";
pub const M0_R34_TEXTURE_RESREF: &str = M0_R33_TEXTURE_RESREF;
pub const M0_R34_FIXTURE_ID: &str = M0_R33_FIXTURE_ID;
pub const M0_R34_FIXTURE_TEMPLATE_RESREF: &str = M0_R33_FIXTURE_TEMPLATE_RESREF;
pub const M0_R34_FIXTURE_DISPLAY_NAME: &str = M0_R33_FIXTURE_DISPLAY_NAME;
pub const M0_R34_APPEARANCE_LABEL: &str = M0_R33_APPEARANCE_LABEL;
pub const M0_R34_APPEARANCE_ROW: u16 = M0_R33_APPEARANCE_ROW;
pub const M0_R34_ACCEPTED_R33_PROOF_SHA256: &str =
    "dd0f74ab0cb7494f0a95ca63aa01f20eb88f067cc79ca9bbd079de16afdfea82";
pub const M0_R34_SOURCE_SHA256: &str = M0_R33_SOURCE_SHA256;
pub const M0_R34_DONOR_SHA256: &str = M0_R33_DONOR_SHA256;
pub const M0_R34_BASE_APPEARANCE_SHA256: &str = M0_R33_BASE_APPEARANCE_SHA256;
pub const M0_R34_TEXTURE_SHA256: &str = M0_R33_TEXTURE_SHA256;
pub const M0_R34_MODEL_SHA256: &str =
    "2fe4ad1ae4354335008916cbff3e0f724fedf0119f30f07aa8d5341c3d5b4af5";

pub const M0_R35_MODULE_RESREF: &str = "m2a_m0r35";
pub const M0_R35_AREA_RESREF: &str = "m2a_m0a35";
pub const M0_R35_HAK_RESREF: &str = "m2a_m0r35";
pub const M0_R35_MODEL_RESREF: &str = "m2a_m0p35";
pub const M0_R35_TEXTURE_RESREF: &str = M0_R34_TEXTURE_RESREF;
pub const M0_R35_FIXTURE_ID: &str = M0_R34_FIXTURE_ID;
pub const M0_R35_FIXTURE_TEMPLATE_RESREF: &str = M0_R34_FIXTURE_TEMPLATE_RESREF;
pub const M0_R35_FIXTURE_DISPLAY_NAME: &str = M0_R34_FIXTURE_DISPLAY_NAME;
pub const M0_R35_APPEARANCE_LABEL: &str = M0_R34_APPEARANCE_LABEL;
pub const M0_R35_APPEARANCE_ROW: u16 = M0_R34_APPEARANCE_ROW;
pub const M0_R35_ACCEPTED_R34_PROOF_SHA256: &str =
    "862cc0cfa2869aafdd382d548db57bebf2330d9a8db80a7885e3ceb023755098";
pub const M0_R35_SOURCE_SHA256: &str = M0_R34_SOURCE_SHA256;
pub const M0_R35_DONOR_SHA256: &str = M0_R34_DONOR_SHA256;
pub const M0_R35_BASE_APPEARANCE_SHA256: &str = M0_R34_BASE_APPEARANCE_SHA256;
pub const M0_R35_TEXTURE_SHA256: &str = M0_R34_TEXTURE_SHA256;
pub const M0_R35_MODEL_SHA256: &str =
    "779d93fa762980ef17448762ba97d8f8775b03335b1d9561b8c2b6483772b3e0";

pub const M0_R36_MODULE_RESREF: &str = "m2a_m0r36";
pub const M0_R36_AREA_RESREF: &str = "m2a_m0a36";
pub const M0_R36_HAK_RESREF: &str = "m2a_m0r36";
pub const M0_R36_MODEL_RESREF: &str = "m2a_m0p36";
pub const M0_R36_TEXTURE_RESREF: &str = M0_R35_TEXTURE_RESREF;
pub const M0_R36_FIXTURE_ID: &str = M0_R35_FIXTURE_ID;
pub const M0_R36_FIXTURE_TEMPLATE_RESREF: &str = M0_R35_FIXTURE_TEMPLATE_RESREF;
pub const M0_R36_FIXTURE_DISPLAY_NAME: &str = M0_R35_FIXTURE_DISPLAY_NAME;
pub const M0_R36_APPEARANCE_LABEL: &str = M0_R35_APPEARANCE_LABEL;
pub const M0_R36_APPEARANCE_ROW: u16 = M0_R35_APPEARANCE_ROW;
pub const M0_R36_ACCEPTED_R35_PROOF_SHA256: &str =
    "7479bbe57093e7e74f34aee5474ef6bc2d8a44a2f7fe843101c2322c57ea3d7c";
pub const M0_R36_SOURCE_SHA256: &str = M0_R35_SOURCE_SHA256;
pub const M0_R36_DONOR_SHA256: &str = M0_R35_DONOR_SHA256;
pub const M0_R36_BASE_APPEARANCE_SHA256: &str = M0_R35_BASE_APPEARANCE_SHA256;
pub const M0_R36_TEXTURE_SHA256: &str = M0_R35_TEXTURE_SHA256;
pub const M0_R36_MODEL_SHA256: &str =
    "459b9954d377c1daab9b12c73a2bf9a64507b5f3cf6d2a6a2ea7d751f680963a";

pub const M0_R37_MODULE_RESREF: &str = "m2a_m0r37";
pub const M0_R37_AREA_RESREF: &str = "m2a_m0a37";
pub const M0_R37_HAK_RESREF: &str = "m2a_m0r37";
pub const M0_R37_MODEL_RESREF: &str = "m2a_m0p37";
pub const M0_R37_TEXTURE_RESREF: &str = M0_R36_TEXTURE_RESREF;
pub const M0_R37_FIXTURE_ID: &str = M0_R36_FIXTURE_ID;
pub const M0_R37_FIXTURE_TEMPLATE_RESREF: &str = M0_R36_FIXTURE_TEMPLATE_RESREF;
pub const M0_R37_FIXTURE_DISPLAY_NAME: &str = M0_R36_FIXTURE_DISPLAY_NAME;
pub const M0_R37_APPEARANCE_LABEL: &str = M0_R36_APPEARANCE_LABEL;
pub const M0_R37_APPEARANCE_ROW: u16 = M0_R36_APPEARANCE_ROW;
pub const M0_R37_ACCEPTED_R36_PROOF_SHA256: &str =
    "d91bd0f724903ee8ffdd915a86c12f3d3e64282f9f45a208ca330ae3ed857775";
pub const M0_R37_SOURCE_SHA256: &str = M0_R36_SOURCE_SHA256;
pub const M0_R37_DONOR_SHA256: &str = M0_R36_DONOR_SHA256;
pub const M0_R37_BASE_APPEARANCE_SHA256: &str = M0_R36_BASE_APPEARANCE_SHA256;
pub const M0_R37_TEXTURE_SHA256: &str = M0_R36_TEXTURE_SHA256;
pub const M0_R37_MODEL_SHA256: &str =
    "48746e6e0b19bedbdcc8a364ff96cd583848dfa38e06971706bfb69b0341f676";

pub const M0_R38_MODULE_RESREF: &str = "m2a_m0r38";
pub const M0_R38_AREA_RESREF: &str = "m2a_m0a38";
pub const M0_R38_HAK_RESREF: &str = "m2a_m0r38";
pub const M0_R38_MODEL_RESREF: &str = "m2a_m0p38";
pub const M0_R38_TEXTURE_RESREF: &str = M0_R37_TEXTURE_RESREF;
pub const M0_R38_FIXTURE_ID: &str = M0_R37_FIXTURE_ID;
pub const M0_R38_FIXTURE_TEMPLATE_RESREF: &str = M0_R37_FIXTURE_TEMPLATE_RESREF;
pub const M0_R38_FIXTURE_DISPLAY_NAME: &str = M0_R37_FIXTURE_DISPLAY_NAME;
pub const M0_R38_APPEARANCE_LABEL: &str = M0_R37_APPEARANCE_LABEL;
pub const M0_R38_APPEARANCE_ROW: u16 = M0_R37_APPEARANCE_ROW;
pub const M0_R38_ACCEPTED_R37_PROOF_SHA256: &str =
    "bf739f2f1d040dd408d06dd6ee0de4f4a70463c359e5e0754737dcb484ab88fd";
pub const M0_R38_SOURCE_SHA256: &str = M0_R37_SOURCE_SHA256;
pub const M0_R38_DONOR_SHA256: &str = M0_R37_DONOR_SHA256;
pub const M0_R38_BASE_APPEARANCE_SHA256: &str = M0_R37_BASE_APPEARANCE_SHA256;
pub const M0_R38_TEXTURE_SHA256: &str = M0_R37_TEXTURE_SHA256;
pub const M0_R38_MODEL_SHA256: &str =
    "039d07cd937430d83006c7d0176aa7659440265417fb7fe53bf73b405563c248";

pub const M0_R39_MODULE_RESREF: &str = "m2a_m0r39";
pub const M0_R39_AREA_RESREF: &str = "m2a_m0a39";
pub const M0_R39_HAK_RESREF: &str = "m2a_m0r39";
pub const M0_R39_MODEL_RESREF: &str = "m2a_m0p39";
pub const M0_R39_TEXTURE_RESREF: &str = M0_R38_TEXTURE_RESREF;
pub const M0_R39_FIXTURE_ID: &str = M0_R38_FIXTURE_ID;
pub const M0_R39_FIXTURE_TEMPLATE_RESREF: &str = M0_R38_FIXTURE_TEMPLATE_RESREF;
pub const M0_R39_FIXTURE_DISPLAY_NAME: &str = M0_R38_FIXTURE_DISPLAY_NAME;
pub const M0_R39_APPEARANCE_LABEL: &str = M0_R38_APPEARANCE_LABEL;
pub const M0_R39_APPEARANCE_ROW: u16 = M0_R38_APPEARANCE_ROW;
pub const M0_R39_ACCEPTED_R38_PROOF_SHA256: &str =
    "59e8e461013b109c13611ab22d2112e5997ef044b3f43a26bcbc26dafb552db6";
pub const M0_R39_SOURCE_SHA256: &str = M0_R38_SOURCE_SHA256;
pub const M0_R39_DONOR_SHA256: &str = M0_R38_DONOR_SHA256;
pub const M0_R39_BASE_APPEARANCE_SHA256: &str = M0_R38_BASE_APPEARANCE_SHA256;
pub const M0_R39_TEXTURE_SHA256: &str = M0_R38_TEXTURE_SHA256;
pub const M0_R39_MODEL_SHA256: &str =
    "fab5ab98e9225c1553947f17994441273ae4c9bbd5a1d14034721ee3be2d86db";

pub const M0_R40_MODULE_RESREF: &str = "m2a_m0r40";
pub const M0_R40_AREA_RESREF: &str = "m2a_m0a40";
pub const M0_R40_HAK_RESREF: &str = "m2a_m0r40";
pub const M0_R40_MODEL_RESREF: &str = "m2a_m0p40";
pub const M0_R40_TEXTURE_RESREF: &str = M0_R39_TEXTURE_RESREF;
pub const M0_R40_FIXTURE_ID: &str = M0_R39_FIXTURE_ID;
pub const M0_R40_FIXTURE_TEMPLATE_RESREF: &str = M0_R39_FIXTURE_TEMPLATE_RESREF;
pub const M0_R40_FIXTURE_DISPLAY_NAME: &str = "Meshy M0 rigid-donor fixture";
pub const M0_R40_APPEARANCE_LABEL: &str = M0_R39_APPEARANCE_LABEL;
pub const M0_R40_APPEARANCE_ROW: u16 = M0_R39_APPEARANCE_ROW;
pub const M0_R40_ACCEPTED_R39_PROOF_SHA256: &str =
    "f023957be74395baaa8228ac7b7683118aaa127849cf385e4f5badd4b871694a";
pub const M0_R40_SOURCE_SHA256: &str = M0_R39_SOURCE_SHA256;
pub const M0_R40_DONOR_SHA256: &str = M0_R39_DONOR_SHA256;
pub const M0_R40_BASE_APPEARANCE_SHA256: &str = M0_R39_BASE_APPEARANCE_SHA256;
pub const M0_R40_TEXTURE_SHA256: &str = M0_R39_TEXTURE_SHA256;
pub const M0_R40_MODEL_SHA256: &str =
    "1809b05370e77f2c2559ec3e6fb354518bb5695ed48600954033175377fc0a40";

pub const M0_R41_MODULE_RESREF: &str = "m2a_m0r41";
pub const M0_R41_AREA_RESREF: &str = "m2a_m0a41";
pub const M0_R41_HAK_RESREF: &str = "m2a_m0r41";
pub const M0_R41_MODEL_RESREF: &str = "m2a_m0p41";
pub const M0_R41_TEXTURE_RESREF: &str = M0_R40_TEXTURE_RESREF;
pub const M0_R41_FIXTURE_ID: &str = M0_R40_FIXTURE_ID;
pub const M0_R41_FIXTURE_TEMPLATE_RESREF: &str = M0_R40_FIXTURE_TEMPLATE_RESREF;
pub const M0_R41_FIXTURE_DISPLAY_NAME: &str = "Meshy M0 full-state rigid fixture";
pub const M0_R41_APPEARANCE_LABEL: &str = M0_R40_APPEARANCE_LABEL;
pub const M0_R41_APPEARANCE_ROW: u16 = M0_R40_APPEARANCE_ROW;
pub const M0_R41_ACCEPTED_R40_PROOF_SHA256: &str =
    "1a043cb45bcf74f34c69dfc5352c996c297604f82a2114181220eec135b6f10a";
pub const M0_R41_SOURCE_SHA256: &str = M0_R40_SOURCE_SHA256;
pub const M0_R41_DONOR_SHA256: &str = M0_R40_DONOR_SHA256;
pub const M0_R41_BASE_APPEARANCE_SHA256: &str = M0_R40_BASE_APPEARANCE_SHA256;
pub const M0_R41_TEXTURE_SHA256: &str = M0_R40_TEXTURE_SHA256;
pub const M0_R41_MODEL_SHA256: &str =
    "c6a341ac8fc432d3d3f5862f67e1fcbcad72bb1ab3d2d241ce6421a976261f6b";

pub const H2_R42_MODULE_RESREF: &str = "m2a_h2r42";
pub const H2_R42_AREA_RESREF: &str = "m2a_h2a42";
pub const H2_R42_HAK_RESREF: &str = "m2a_h2r42";
pub const H2_R42_MODEL_RESREF: &str = "m2a_h2p42";
pub const H2_R42_TEXTURE_RESREF: &str = "m2a_h2t42";
pub const H2_R42_FIXTURE_ID: &str = "h2_fixture";
pub const H2_R42_FIXTURE_TEMPLATE_RESREF: &str = "m2a_h2utc42";
pub const H2_R42_FIXTURE_DISPLAY_NAME: &str = "Meshy H2 turquoise clockwork sentinel";
pub const H2_R42_STOCK_CONTROL_ID: &str = "stock_control";
pub const H2_R42_STOCK_CONTROL_TEMPLATE_RESREF: &str = "m2a_h2ctrl42";
pub const H2_R42_STOCK_CONTROL_DISPLAY_NAME: &str = "Stock Hook Horror control";
pub const H2_R42_APPEARANCE_LABEL: &str = "M2A_H2_CLOCKWORK_SENTINEL";
pub const H2_R42_APPEARANCE_ROW: u16 = 15_100;
pub const H2_R42_STOCK_CONTROL_APPEARANCE_ROW: u16 = 102;
pub const H2_R42_ACCEPTED_R41_PROOF_SHA256: &str =
    "38ca22bf01bb264461a5c1419071cf18fff2b6762569a313004d1b8ee906aa6b";
pub const H2_R42_SOURCE_SHA256: &str =
    "f8cf0af21c8143a62b64c490a81dd2855ad3c3f9865922e3854f84b714dec3a3";
pub const H2_R42_BASE_APPEARANCE_SHA256: &str = M0_R33_BASE_APPEARANCE_SHA256;
pub const H2_R42_MODEL_SHA256: &str =
    "20fe29854f01880b0b9d5b4f0a5e703c76f858eeebb01a00ea3a29a4913969d4";
pub const H2_R42_TEXTURE_SHA256: &str =
    "03169b1493ed4b2f5269ed6fa611aef787e219cddd08661c9135a7f82191c167";
pub const H2_R42_APPEARANCE_SHA256: &str =
    "134f3e8a33c1103716b0b2b6f0e68f4686aa7144c1da2bb7447785a7627fff65";
pub const H2_R42_HAK_SHA256: &str =
    "6d50bd1f784a8d1b10b36b09e32565781674def6b4f4367d803ac7acced6da69";
pub const H2_R42_MODULE_SHA256: &str =
    "c22741a72ab19a4b6e43620902ec9530e73d251c67de2beb23a2fef60e2b3c10";
pub const H2_R42_CONTRACT_SHA256: &str =
    "7f06b7d1f33e134963ec110ca20242e65f37a8c959e0817b0fa62c997a0d2782";

pub const H2_R43_MODULE_RESREF: &str = "m2a_h2r43";
pub const H2_R43_AREA_RESREF: &str = "m2a_h2a43";
pub const H2_R43_HAK_RESREF: &str = "m2a_h2r43";
pub const H2_R43_MODEL_RESREF: &str = "m2a_h2p43";
pub const H2_R43_TEXTURE_RESREF: &str = "m2a_h2t43";
pub const H2_R43_FIXTURE_ID: &str = "h2_fixture";
pub const H2_R43_FIXTURE_TEMPLATE_RESREF: &str = "m2a_h2utc43";
pub const H2_R43_FIXTURE_DISPLAY_NAME: &str = "Meshy H2 H1-root clockwork sentinel";
pub const H2_R43_STOCK_CONTROL_ID: &str = "stock_control";
pub const H2_R43_STOCK_CONTROL_TEMPLATE_RESREF: &str = "m2a_h2ctrl43";
pub const H2_R43_STOCK_CONTROL_DISPLAY_NAME: &str = "Stock Hook Horror control";
pub const H2_R43_APPEARANCE_LABEL: &str = "M2A_H2_H1_ROOT_SENTINEL";
pub const H2_R43_APPEARANCE_ROW: u16 = 15_100;
pub const H2_R43_STOCK_CONTROL_APPEARANCE_ROW: u16 = 102;
pub const H2_R43_ACCEPTED_R42_PROOF_SHA256: &str =
    "ee855285cfe83fac4de461ac1fb76af1bc39e327332124f58d79220934abaefd";
pub const H2_R43_SOURCE_SHA256: &str = H2_R42_SOURCE_SHA256;
pub const H2_R43_BASE_APPEARANCE_SHA256: &str = H2_R42_BASE_APPEARANCE_SHA256;
pub const H2_R43_MODEL_SHA256: &str =
    "5e169877c2f66d68f7bc8cae58e73087f346ff51b5652b37bc787e59c4c38d2a";
pub const H2_R43_TEXTURE_SHA256: &str =
    "03169b1493ed4b2f5269ed6fa611aef787e219cddd08661c9135a7f82191c167";
pub const H2_R43_APPEARANCE_SHA256: &str =
    "63111bfbf1af7c1f9f9dfcdca9ad573f3f2051a0842fb171ecd86ea5969bb114";
pub const H2_R43_HAK_SHA256: &str =
    "d4381dbdc9aee5f0dab2b8159a414586a913b7ed9bf68fe48515a951cdebbdb1";
pub const H2_R43_MODULE_SHA256: &str =
    "ef2b48b80e4c01b455a04246c8e1da8f839171d9e9b4f9e835b846738f0a0c89";
pub const H2_R43_CONTRACT_SHA256: &str =
    "931ec8cdd92685886d7ea342c7078b3340c40214d1d56a7bbcc0da5831547028";

const H2_R42_SOURCE_BYTE_LENGTH: u64 = 8_234_708;
const H2_R43_SOURCE_BYTE_LENGTH: u64 = H2_R42_SOURCE_BYTE_LENGTH;

const M0_R33_SOURCE_BYTE_LENGTH: u64 = 8_581_684;
const M0_R33_DONOR_BYTE_LENGTH: u64 = 7_944_380;
const M0_R33_BASE_APPEARANCE_BYTE_LENGTH: u64 = 6_901_169;
const M0_R33_BASE_APPEARANCE_ROWS: u32 = 15_100;
const M0_R33_REQUIRED_CLIPS: [&str; 7] = [
    "cpause1",
    "cappear",
    "cwalk",
    "crun",
    "ca1slashl",
    "cdamagel",
    "cdead",
];
const M0_R34_REQUIRED_CLIPS: [&str; 7] = M0_R33_REQUIRED_CLIPS;
const M0_R35_REQUIRED_CLIPS: [&str; 7] = M0_R34_REQUIRED_CLIPS;
const M0_R36_REQUIRED_CLIPS: [&str; 7] = M0_R35_REQUIRED_CLIPS;
const M0_R37_REQUIRED_CLIPS: [&str; 7] = M0_R36_REQUIRED_CLIPS;
const M0_R38_REQUIRED_CLIPS: [&str; 7] = M0_R37_REQUIRED_CLIPS;
const M0_R39_REQUIRED_CLIPS: [&str; 7] = M0_R38_REQUIRED_CLIPS;
const M0_R40_REQUIRED_CLIPS: [&str; 7] = M0_R39_REQUIRED_CLIPS;
const M0_R41_REQUIRED_CLIPS: [&str; 7] = M0_R40_REQUIRED_CLIPS;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M0R33ResourceBindingV1 {
    pub resref: String,
    pub byte_length: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M0R33SkinReadbackV1 {
    pub model_name: String,
    pub node_count: usize,
    pub skin_node_count: usize,
    pub weighted_vertex_count: usize,
    pub active_bone_count: usize,
    pub animation_names: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M0R33AnimatedDonorCandidateContractV1 {
    pub schema_version: u32,
    pub profile: String,
    pub candidate_admissible: bool,
    pub admitted_by_r32_proof_sha256: String,
    pub diagnosed_failure_boundary: String,
    pub intended_functional_delta: Vec<String>,
    pub source: M6ByteIdentityV1,
    pub donor: M6ByteIdentityV1,
    pub base_appearance: M6ByteIdentityV1,
    pub retarget: AnimatedDonorRetargetReportV1,
    pub skin_readback: M0R33SkinReadbackV1,
    pub model: M0R33ResourceBindingV1,
    pub texture: M0R33ResourceBindingV1,
    pub appearance_two_da: M0R33ResourceBindingV1,
    pub hak: M0R33ResourceBindingV1,
    pub module: M0R33ResourceBindingV1,
    pub package_manifest: PackageManifestV1,
    pub binary_scene: BinaryCreatureMultiFixtureModuleReadbackV1,
    pub materialization_count: u32,
    pub toolset_model_visibility: String,
    pub toolset_proof_completeness: String,
    pub nwn_model_visibility: String,
    pub nwn_proof_completeness: String,
}

#[derive(Clone, Debug)]
pub struct M0R33AnimatedDonorCandidateArtifactV1 {
    pub model: Vec<u8>,
    pub texture: Vec<u8>,
    pub appearance_two_da: Vec<u8>,
    pub hak: Vec<u8>,
    pub module: Vec<u8>,
    pub contract: M0R33AnimatedDonorCandidateContractV1,
    pub contract_json: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M0R34SkinReadbackV1 {
    pub model_name: String,
    pub base_node_count: usize,
    pub skin_node_count: usize,
    pub weighted_vertex_count: usize,
    pub active_bone_count: usize,
    pub animation_names: Vec<String>,
    pub animation_node_counts: Vec<usize>,
    pub state_projection_profile: MdlStateProjectionProfileV1,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M0R34AnimatedDonorCandidateContractV1 {
    pub schema_version: u32,
    pub profile: String,
    pub candidate_admissible: bool,
    pub admitted_by_r33_proof_sha256: String,
    pub diagnosed_failure_boundary: String,
    pub intended_functional_delta: Vec<String>,
    pub source: M6ByteIdentityV1,
    pub donor: M6ByteIdentityV1,
    pub base_appearance: M6ByteIdentityV1,
    pub retarget: AnimatedDonorRetargetReportV1,
    pub skin_readback: M0R34SkinReadbackV1,
    pub model: M0R33ResourceBindingV1,
    pub texture: M0R33ResourceBindingV1,
    pub appearance_two_da: M0R33ResourceBindingV1,
    pub hak: M0R33ResourceBindingV1,
    pub module: M0R33ResourceBindingV1,
    pub package_manifest: PackageManifestV1,
    pub binary_scene: BinaryCreatureMultiFixtureModuleReadbackV1,
    pub materialization_count: u32,
    pub toolset_model_visibility: String,
    pub toolset_proof_completeness: String,
    pub nwn_model_visibility: String,
    pub nwn_proof_completeness: String,
}

#[derive(Clone, Debug)]
pub struct M0R34AnimatedDonorCandidateArtifactV1 {
    pub model: Vec<u8>,
    pub texture: Vec<u8>,
    pub appearance_two_da: Vec<u8>,
    pub hak: Vec<u8>,
    pub module: Vec<u8>,
    pub contract: M0R34AnimatedDonorCandidateContractV1,
    pub contract_json: Vec<u8>,
}

pub type M0R34AnimatedDonorCandidateErrorV1 = M0R33AnimatedDonorCandidateErrorV1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M0R35SkinReadbackV1 {
    pub model_name: String,
    pub base_node_count: usize,
    pub skin_node_count: usize,
    pub weighted_vertex_count: usize,
    pub active_bone_count: usize,
    pub active_inline_slot_count: usize,
    pub unused_inline_tail_count: usize,
    pub unused_inline_tail_value: i16,
    pub animation_names: Vec<String>,
    pub animation_node_counts: Vec<usize>,
    pub format_profile: MdlFormatProfileV1,
    pub state_projection_profile: MdlStateProjectionProfileV1,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M0R35AnimatedDonorCandidateContractV1 {
    pub schema_version: u32,
    pub profile: String,
    pub candidate_admissible: bool,
    pub admitted_by_r34_proof_sha256: String,
    pub diagnosed_failure_boundary: String,
    pub intended_functional_delta: Vec<String>,
    pub source: M6ByteIdentityV1,
    pub donor: M6ByteIdentityV1,
    pub base_appearance: M6ByteIdentityV1,
    pub retarget: AnimatedDonorRetargetReportV1,
    pub skin_readback: M0R35SkinReadbackV1,
    pub model: M0R33ResourceBindingV1,
    pub texture: M0R33ResourceBindingV1,
    pub appearance_two_da: M0R33ResourceBindingV1,
    pub hak: M0R33ResourceBindingV1,
    pub module: M0R33ResourceBindingV1,
    pub package_manifest: PackageManifestV1,
    pub binary_scene: BinaryCreatureMultiFixtureModuleReadbackV1,
    pub materialization_count: u32,
    pub toolset_model_visibility: String,
    pub toolset_proof_completeness: String,
    pub nwn_model_visibility: String,
    pub nwn_proof_completeness: String,
}

#[derive(Clone, Debug)]
pub struct M0R35AnimatedDonorCandidateArtifactV1 {
    pub model: Vec<u8>,
    pub texture: Vec<u8>,
    pub appearance_two_da: Vec<u8>,
    pub hak: Vec<u8>,
    pub module: Vec<u8>,
    pub contract: M0R35AnimatedDonorCandidateContractV1,
    pub contract_json: Vec<u8>,
}

pub type M0R35AnimatedDonorCandidateErrorV1 = M0R33AnimatedDonorCandidateErrorV1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M0R36SkinReadbackV1 {
    pub model_name: String,
    pub base_node_count: usize,
    pub rig_node_count: usize,
    pub skin_node_count: usize,
    pub weighted_vertex_count: usize,
    pub active_bone_count: usize,
    pub active_inline_slot_count: usize,
    pub unused_inline_tail_count: usize,
    pub unused_inline_tail_value: i16,
    pub aurora_root_name: String,
    pub skeleton_root_name: String,
    pub aurora_root_tree_ordinal: usize,
    pub aurora_root_forward_slot: i16,
    pub first_active_inline_ordinal: i16,
    pub last_active_inline_ordinal: i16,
    pub animation_names: Vec<String>,
    pub animation_node_counts: Vec<usize>,
    pub format_profile: MdlFormatProfileV1,
    pub state_projection_profile: MdlStateProjectionProfileV1,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M0R36AnimatedDonorCandidateContractV1 {
    pub schema_version: u32,
    pub profile: String,
    pub candidate_admissible: bool,
    pub admitted_by_r35_proof_sha256: String,
    pub diagnosed_failure_boundary: String,
    pub intended_functional_delta: Vec<String>,
    pub source: M6ByteIdentityV1,
    pub donor: M6ByteIdentityV1,
    pub base_appearance: M6ByteIdentityV1,
    pub retarget: AnimatedDonorRetargetReportV1,
    pub skin_readback: M0R36SkinReadbackV1,
    pub model: M0R33ResourceBindingV1,
    pub texture: M0R33ResourceBindingV1,
    pub appearance_two_da: M0R33ResourceBindingV1,
    pub hak: M0R33ResourceBindingV1,
    pub module: M0R33ResourceBindingV1,
    pub package_manifest: PackageManifestV1,
    pub binary_scene: BinaryCreatureMultiFixtureModuleReadbackV1,
    pub materialization_count: u32,
    pub toolset_model_visibility: String,
    pub toolset_proof_completeness: String,
    pub nwn_model_visibility: String,
    pub nwn_proof_completeness: String,
}

#[derive(Clone, Debug)]
pub struct M0R36AnimatedDonorCandidateArtifactV1 {
    pub model: Vec<u8>,
    pub texture: Vec<u8>,
    pub appearance_two_da: Vec<u8>,
    pub hak: Vec<u8>,
    pub module: Vec<u8>,
    pub contract: M0R36AnimatedDonorCandidateContractV1,
    pub contract_json: Vec<u8>,
}

pub type M0R36AnimatedDonorCandidateErrorV1 = M0R33AnimatedDonorCandidateErrorV1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M0R37SkinReadbackV1 {
    pub model_name: String,
    pub base_node_count: usize,
    pub rig_node_count: usize,
    pub skin_node_count: usize,
    pub weighted_vertex_count: usize,
    pub active_bone_count: usize,
    pub active_inline_slot_count: usize,
    pub unused_inline_tail_count: usize,
    pub unused_inline_tail_value: i16,
    pub aurora_root_name: String,
    pub skeleton_root_name: String,
    pub skin_parent_name: String,
    pub aurora_root_tree_ordinal: usize,
    pub aurora_root_forward_slot: i16,
    pub first_active_inline_ordinal: i16,
    pub last_active_inline_ordinal: i16,
    pub animation_names: Vec<String>,
    pub animation_node_counts: Vec<usize>,
    pub format_profile: MdlFormatProfileV1,
    pub state_projection_profile: MdlStateProjectionProfileV1,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M0R37AnimatedDonorCandidateContractV1 {
    pub schema_version: u32,
    pub profile: String,
    pub candidate_admissible: bool,
    pub admitted_by_r36_proof_sha256: String,
    pub diagnosed_failure_boundary: String,
    pub intended_functional_delta: Vec<String>,
    pub source: M6ByteIdentityV1,
    pub donor: M6ByteIdentityV1,
    pub base_appearance: M6ByteIdentityV1,
    pub retarget: AnimatedDonorRetargetReportV1,
    pub skin_readback: M0R37SkinReadbackV1,
    pub model: M0R33ResourceBindingV1,
    pub texture: M0R33ResourceBindingV1,
    pub appearance_two_da: M0R33ResourceBindingV1,
    pub hak: M0R33ResourceBindingV1,
    pub module: M0R33ResourceBindingV1,
    pub package_manifest: PackageManifestV1,
    pub binary_scene: BinaryCreatureMultiFixtureModuleReadbackV1,
    pub materialization_count: u32,
    pub toolset_model_visibility: String,
    pub toolset_proof_completeness: String,
    pub nwn_model_visibility: String,
    pub nwn_proof_completeness: String,
}

#[derive(Clone, Debug)]
pub struct M0R37AnimatedDonorCandidateArtifactV1 {
    pub model: Vec<u8>,
    pub texture: Vec<u8>,
    pub appearance_two_da: Vec<u8>,
    pub hak: Vec<u8>,
    pub module: Vec<u8>,
    pub contract: M0R37AnimatedDonorCandidateContractV1,
    pub contract_json: Vec<u8>,
}

pub type M0R37AnimatedDonorCandidateErrorV1 = M0R33AnimatedDonorCandidateErrorV1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M0R38SkinReadbackV1 {
    pub model_name: String,
    pub base_node_count: usize,
    pub rig_node_count: usize,
    pub skin_node_count: usize,
    pub weighted_vertex_count: usize,
    pub active_bone_count: usize,
    pub active_inline_slot_count: usize,
    pub unused_inline_tail_count: usize,
    pub unused_inline_tail_value: i16,
    pub aurora_root_name: String,
    pub skeleton_root_name: String,
    pub skin_parent_name: String,
    pub aurora_root_tree_ordinal: usize,
    pub aurora_root_forward_slot: i16,
    pub first_active_inline_ordinal: i16,
    pub last_active_inline_ordinal: i16,
    pub animation_names: Vec<String>,
    pub animation_node_counts: Vec<usize>,
    pub animation_scale_controller_count: usize,
    pub format_profile: MdlFormatProfileV1,
    pub state_projection_profile: MdlStateProjectionProfileV1,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M0R38AnimatedDonorCandidateContractV1 {
    pub schema_version: u32,
    pub profile: String,
    pub candidate_admissible: bool,
    pub admitted_by_r37_proof_sha256: String,
    pub diagnosed_failure_boundary: String,
    pub intended_functional_delta: Vec<String>,
    pub source: M6ByteIdentityV1,
    pub donor: M6ByteIdentityV1,
    pub base_appearance: M6ByteIdentityV1,
    pub retarget: AnimatedDonorRetargetReportV1,
    pub skin_readback: M0R38SkinReadbackV1,
    pub model: M0R33ResourceBindingV1,
    pub texture: M0R33ResourceBindingV1,
    pub appearance_two_da: M0R33ResourceBindingV1,
    pub hak: M0R33ResourceBindingV1,
    pub module: M0R33ResourceBindingV1,
    pub package_manifest: PackageManifestV1,
    pub binary_scene: BinaryCreatureMultiFixtureModuleReadbackV1,
    pub materialization_count: u32,
    pub toolset_model_visibility: String,
    pub toolset_proof_completeness: String,
    pub nwn_model_visibility: String,
    pub nwn_proof_completeness: String,
}

#[derive(Clone, Debug)]
pub struct M0R38AnimatedDonorCandidateArtifactV1 {
    pub model: Vec<u8>,
    pub texture: Vec<u8>,
    pub appearance_two_da: Vec<u8>,
    pub hak: Vec<u8>,
    pub module: Vec<u8>,
    pub contract: M0R38AnimatedDonorCandidateContractV1,
    pub contract_json: Vec<u8>,
}

pub type M0R38AnimatedDonorCandidateErrorV1 = M0R33AnimatedDonorCandidateErrorV1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M0R39SkinReadbackV1 {
    pub model_name: String,
    pub base_node_count: usize,
    pub rig_node_count: usize,
    pub skin_node_count: usize,
    pub weighted_vertex_count: usize,
    pub active_bone_count: usize,
    pub active_inline_slot_count: usize,
    pub unused_inline_tail_count: usize,
    pub unused_inline_tail_value: i16,
    pub aurora_root_name: String,
    pub skeleton_root_name: String,
    pub skin_parent_name: String,
    pub aurora_root_tree_ordinal: usize,
    pub aurora_root_forward_slot: i16,
    pub base_root_controller_count: usize,
    pub first_active_inline_ordinal: i16,
    pub last_active_inline_ordinal: i16,
    pub animation_names: Vec<String>,
    pub animation_node_counts: Vec<usize>,
    pub animation_scale_controller_count: usize,
    pub format_profile: MdlFormatProfileV1,
    pub state_projection_profile: MdlStateProjectionProfileV1,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M0R39AnimatedDonorCandidateContractV1 {
    pub schema_version: u32,
    pub profile: String,
    pub candidate_admissible: bool,
    pub admitted_by_r38_proof_sha256: String,
    pub diagnosed_failure_boundary: String,
    pub intended_functional_delta: Vec<String>,
    pub source: M6ByteIdentityV1,
    pub donor: M6ByteIdentityV1,
    pub base_appearance: M6ByteIdentityV1,
    pub retarget: AnimatedDonorRetargetReportV1,
    pub skin_readback: M0R39SkinReadbackV1,
    pub model: M0R33ResourceBindingV1,
    pub texture: M0R33ResourceBindingV1,
    pub appearance_two_da: M0R33ResourceBindingV1,
    pub hak: M0R33ResourceBindingV1,
    pub module: M0R33ResourceBindingV1,
    pub package_manifest: PackageManifestV1,
    pub binary_scene: BinaryCreatureMultiFixtureModuleReadbackV1,
    pub materialization_count: u32,
    pub toolset_model_visibility: String,
    pub toolset_proof_completeness: String,
    pub nwn_model_visibility: String,
    pub nwn_proof_completeness: String,
}

#[derive(Clone, Debug)]
pub struct M0R39AnimatedDonorCandidateArtifactV1 {
    pub model: Vec<u8>,
    pub texture: Vec<u8>,
    pub appearance_two_da: Vec<u8>,
    pub hak: Vec<u8>,
    pub module: Vec<u8>,
    pub contract: M0R39AnimatedDonorCandidateContractV1,
    pub contract_json: Vec<u8>,
}

pub type M0R39AnimatedDonorCandidateErrorV1 = M0R33AnimatedDonorCandidateErrorV1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M0R40RigidReadbackV1 {
    pub model_name: String,
    pub base_node_count: usize,
    pub rig_node_count: usize,
    pub rigid_mesh_node_count: usize,
    pub skin_node_count: usize,
    pub triangle_count: usize,
    pub duplicated_vertex_count: usize,
    pub active_parent_bone_count: usize,
    pub aurora_root_name: String,
    pub skeleton_root_name: String,
    pub base_root_controller_count: usize,
    pub animation_names: Vec<String>,
    pub animation_node_counts: Vec<usize>,
    pub animation_scale_controller_count: usize,
    pub format_profile: MdlFormatProfileV1,
    pub state_projection_profile: MdlStateProjectionProfileV1,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M0R40AnimatedDonorCandidateContractV1 {
    pub schema_version: u32,
    pub profile: String,
    pub candidate_admissible: bool,
    pub admitted_by_r39_proof_sha256: String,
    pub diagnosed_failure_boundary: String,
    pub intended_functional_delta: Vec<String>,
    pub source: M6ByteIdentityV1,
    pub donor: M6ByteIdentityV1,
    pub base_appearance: M6ByteIdentityV1,
    pub retarget: AnimatedDonorRetargetReportV1,
    pub rigid_readback: M0R40RigidReadbackV1,
    pub model: M0R33ResourceBindingV1,
    pub texture: M0R33ResourceBindingV1,
    pub appearance_two_da: M0R33ResourceBindingV1,
    pub hak: M0R33ResourceBindingV1,
    pub module: M0R33ResourceBindingV1,
    pub package_manifest: PackageManifestV1,
    pub binary_scene: BinaryCreatureMultiFixtureModuleReadbackV1,
    pub materialization_count: u32,
    pub toolset_model_visibility: String,
    pub toolset_proof_completeness: String,
    pub nwn_model_visibility: String,
    pub nwn_proof_completeness: String,
}

#[derive(Clone, Debug)]
pub struct M0R40AnimatedDonorCandidateArtifactV1 {
    pub model: Vec<u8>,
    pub texture: Vec<u8>,
    pub appearance_two_da: Vec<u8>,
    pub hak: Vec<u8>,
    pub module: Vec<u8>,
    pub contract: M0R40AnimatedDonorCandidateContractV1,
    pub contract_json: Vec<u8>,
}

pub type M0R40AnimatedDonorCandidateErrorV1 = M0R33AnimatedDonorCandidateErrorV1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M0R41RigidReadbackV1 {
    pub model_name: String,
    pub base_node_count: usize,
    pub rig_node_count: usize,
    pub rigid_mesh_node_count: usize,
    pub skin_node_count: usize,
    pub triangle_count: usize,
    pub duplicated_vertex_count: usize,
    pub active_parent_bone_count: usize,
    pub aurora_root_name: String,
    pub skeleton_root_name: String,
    pub base_root_controller_count: usize,
    pub animation_names: Vec<String>,
    pub animation_node_counts: Vec<usize>,
    pub animation_mesh_identity_counts: Vec<usize>,
    pub animation_scale_controller_count: usize,
    pub format_profile: MdlFormatProfileV1,
    pub state_projection_profile: MdlStateProjectionProfileV1,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M0R41AnimatedDonorCandidateContractV1 {
    pub schema_version: u32,
    pub profile: String,
    pub candidate_admissible: bool,
    pub admitted_by_r40_proof_sha256: String,
    pub diagnosed_failure_boundary: String,
    pub intended_functional_delta: Vec<String>,
    pub source: M6ByteIdentityV1,
    pub donor: M6ByteIdentityV1,
    pub base_appearance: M6ByteIdentityV1,
    pub retarget: AnimatedDonorRetargetReportV1,
    pub rigid_readback: M0R41RigidReadbackV1,
    pub model: M0R33ResourceBindingV1,
    pub texture: M0R33ResourceBindingV1,
    pub appearance_two_da: M0R33ResourceBindingV1,
    pub hak: M0R33ResourceBindingV1,
    pub module: M0R33ResourceBindingV1,
    pub package_manifest: PackageManifestV1,
    pub binary_scene: BinaryCreatureMultiFixtureModuleReadbackV1,
    pub materialization_count: u32,
    pub toolset_model_visibility: String,
    pub toolset_proof_completeness: String,
    pub nwn_model_visibility: String,
    pub nwn_proof_completeness: String,
}

#[derive(Clone, Debug)]
pub struct M0R41AnimatedDonorCandidateArtifactV1 {
    pub model: Vec<u8>,
    pub texture: Vec<u8>,
    pub appearance_two_da: Vec<u8>,
    pub hak: Vec<u8>,
    pub module: Vec<u8>,
    pub contract: M0R41AnimatedDonorCandidateContractV1,
    pub contract_json: Vec<u8>,
}

pub type M0R41AnimatedDonorCandidateErrorV1 = M0R33AnimatedDonorCandidateErrorV1;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct H2R42VisibilityCandidateContractV1 {
    pub schema_version: u32,
    pub profile: String,
    pub candidate_admissible: bool,
    pub admitted_by_r41_proof_sha256: String,
    pub diagnosed_failure_boundary: String,
    pub intended_functional_delta: Vec<String>,
    pub source: M6ByteIdentityV1,
    pub base_appearance: M6ByteIdentityV1,
    pub model: M0R33ResourceBindingV1,
    pub texture: M0R33ResourceBindingV1,
    pub appearance_two_da: M0R33ResourceBindingV1,
    pub hak: M0R33ResourceBindingV1,
    pub module: M0R33ResourceBindingV1,
    pub package_manifest: PackageManifestV1,
    pub binary_scene: BinaryCreatureMultiFixtureModuleReadbackV1,
    pub appearance_row: u16,
    pub triangle_count: usize,
    pub rig_node_count: usize,
    pub rigid_mesh_node_count: usize,
    pub skin_node_count: usize,
    pub animation_names: Vec<String>,
    pub state_projection_profile: MdlStateProjectionProfileV1,
    pub materialization_count: u32,
    pub toolset_model_visibility: String,
    pub toolset_proof_completeness: String,
    pub nwn_model_visibility: String,
    pub nwn_proof_completeness: String,
}

#[derive(Clone, Debug)]
pub struct H2R42VisibilityCandidateArtifactV1 {
    pub model: Vec<u8>,
    pub texture: Vec<u8>,
    pub appearance_two_da: Vec<u8>,
    pub hak: Vec<u8>,
    pub module: Vec<u8>,
    pub contract: H2R42VisibilityCandidateContractV1,
    pub contract_json: Vec<u8>,
}

pub type H2R42VisibilityCandidateErrorV1 = M0R33AnimatedDonorCandidateErrorV1;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct H2R43H1RootLayoutCandidateContractV1 {
    pub schema_version: u32,
    pub profile: String,
    pub candidate_admissible: bool,
    pub admitted_by_r42_proof_sha256: String,
    pub diagnosed_failure_boundary: String,
    pub intended_functional_delta: Vec<String>,
    pub source: M6ByteIdentityV1,
    pub base_appearance: M6ByteIdentityV1,
    pub model: M0R33ResourceBindingV1,
    pub texture: M0R33ResourceBindingV1,
    pub appearance_two_da: M0R33ResourceBindingV1,
    pub hak: M0R33ResourceBindingV1,
    pub module: M0R33ResourceBindingV1,
    pub package_manifest: PackageManifestV1,
    pub binary_scene: BinaryCreatureMultiFixtureModuleReadbackV1,
    pub appearance_row: u16,
    pub triangle_count: usize,
    pub rig_node_count: usize,
    pub rigid_mesh_node_count: usize,
    pub skin_node_count: usize,
    pub base_root_controller_count: usize,
    pub animation_names: Vec<String>,
    pub format_profile: MdlFormatProfileV1,
    pub state_projection_profile: MdlStateProjectionProfileV1,
    pub materialization_count: u32,
    pub toolset_model_visibility: String,
    pub toolset_proof_completeness: String,
    pub nwn_model_visibility: String,
    pub nwn_proof_completeness: String,
}

#[derive(Clone, Debug)]
pub struct H2R43H1RootLayoutCandidateArtifactV1 {
    pub model: Vec<u8>,
    pub texture: Vec<u8>,
    pub appearance_two_da: Vec<u8>,
    pub hak: Vec<u8>,
    pub module: Vec<u8>,
    pub contract: H2R43H1RootLayoutCandidateContractV1,
    pub contract_json: Vec<u8>,
}

pub type H2R43H1RootLayoutCandidateErrorV1 = M0R33AnimatedDonorCandidateErrorV1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M0R33AnimatedDonorCandidateErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for M0R33AnimatedDonorCandidateErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for M0R33AnimatedDonorCandidateErrorV1 {}

pub fn build_m0_r33_animated_donor_candidate_v1(
    source_glb: &[u8],
    donor_glb: &[u8],
    base_appearance_two_da: &[u8],
) -> Result<M0R33AnimatedDonorCandidateArtifactV1, M0R33AnimatedDonorCandidateErrorV1> {
    require_exact_input(
        source_glb,
        M0_R33_SOURCE_BYTE_LENGTH,
        M0_R33_SOURCE_SHA256,
        "sourceGlb",
    )?;
    require_exact_input(
        donor_glb,
        M0_R33_DONOR_BYTE_LENGTH,
        M0_R33_DONOR_SHA256,
        "donorGlb",
    )?;
    require_exact_input(
        base_appearance_two_da,
        M0_R33_BASE_APPEARANCE_BYTE_LENGTH,
        M0_R33_BASE_APPEARANCE_SHA256,
        "baseAppearanceTwoDa",
    )?;
    let appearance_inspection =
        inspect_two_da_v2(base_appearance_two_da, &TwoDaLimitsV1::default())
            .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if appearance_inspection.physical_row_count != M0_R33_BASE_APPEARANCE_ROWS {
        return Err(candidate_error(
            "M2A-R33-APPEARANCE-TRUST-ROOT",
            "baseAppearanceTwoDa",
            "r33 requires the exact 15100-row full runtime appearance table",
        ));
    }

    let retarget = retarget_static_mesh_to_animated_donor_v1(
        source_glb,
        donor_glb,
        &MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64V1,
            state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
            state_projection_provenance: None,
            model_resource_resref: M0_R33_MODEL_RESREF.to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: M0_R33_TEXTURE_RESREF.to_owned(),
            }],
        },
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if retarget.report.model_sha256 != M0_R33_MODEL_SHA256
        || retarget.report.rig_node_count != 24
        || retarget.report.skin_segment_count != 1
        || retarget.report.active_bone_count != 22
        || retarget.report.animation_clip_names != M0_R33_REQUIRED_CLIPS
    {
        return Err(candidate_error(
            "M2A-R33-RETARGET-CONTRACT",
            "retarget.report",
            frozen_retarget_drift_message(
                "r33",
                M0_R33_MODEL_SHA256,
                &retarget.report.model_sha256,
                "exact M0/H1 retarget differs from the admitted weighted runtime profile",
            ),
        ));
    }
    let skin_readback = inspect_skin_contract(&retarget.model.inspection)?;

    let ingest = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        candidate_error(
            error.code,
            error.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            error.message,
        )
    })?;
    let creature = retarget.conversion.creature.as_ref().ok_or_else(|| {
        candidate_error(
            "M2A-R33-RETARGET-CONTRACT",
            "retarget.conversion.creature",
            "successful retarget has no creature output",
        )
    })?;
    let texture_selection = resolve_base_color_image_index_v1(&ingest, creature)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let image = decode_embedded_image_to_tga_v1(
        source_glb,
        texture_selection.source_image_index,
        &GlbLimits::default(),
        &EmbeddedImageDecodeLimitsV1::default(),
    )
    .map_err(|error| {
        candidate_error(
            error.code,
            error
                .json_path
                .unwrap_or_else(|| "sourceGlb.images".to_owned()),
            error.message,
        )
    })?;
    let texture = write_tga_v1(&image, &TgaWriterOptionsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if sha256_bytes(&texture.payload) != M0_R33_TEXTURE_SHA256 {
        return Err(candidate_error(
            "M2A-R33-TEXTURE-IDENTITY",
            "texture",
            "r33 must preserve the exact M0 texture payload",
        ));
    }

    let appearance = append_r33_appearance(base_appearance_two_da)?;
    if appearance.report.appended_row_index != M0_R33_APPEARANCE_ROW {
        return Err(candidate_error(
            "M2A-R33-APPEARANCE-ROW",
            "appearance.appendedPhysicalRow",
            "r33 fixture must bind physical row 15100",
        ));
    }
    require_appearance_cell(
        &appearance.payload,
        M0_R33_APPEARANCE_ROW,
        "LABEL",
        M0_R33_APPEARANCE_LABEL,
    )?;
    require_appearance_cell(&appearance.payload, M0_R33_APPEARANCE_ROW, "MODELTYPE", "S")?;
    require_appearance_cell(
        &appearance.payload,
        M0_R33_APPEARANCE_ROW,
        "RACE",
        M0_R33_MODEL_RESREF,
    )?;

    let resources = vec![
        HakResourceInputV1 {
            resref: M0_R33_MODEL_RESREF.to_owned(),
            resource_type: 2002,
            payload: retarget.model.payload.clone(),
        },
        HakResourceInputV1 {
            resref: M0_R33_TEXTURE_RESREF.to_owned(),
            resource_type: 3,
            payload: texture.payload.clone(),
        },
        HakResourceInputV1 {
            resref: "appearance".to_owned(),
            resource_type: 2017,
            payload: appearance.payload.clone(),
        },
    ];
    let package = write_model_package_v1(&resources, &HakWriterOptionsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let archive = ErfArchive::parse(&package.hak.payload).map_err(|error| {
        candidate_error(error.code, format!("hak@{}", error.offset), error.context)
    })?;
    for (resref, resource_type, expected) in [
        (M0_R33_MODEL_RESREF, 2002, retarget.model.payload.as_slice()),
        (M0_R33_TEXTURE_RESREF, 3, texture.payload.as_slice()),
        ("appearance", 2017, appearance.payload.as_slice()),
    ] {
        let actual = archive.find(resref, resource_type).map_err(|error| {
            candidate_error(error.code, format!("hak@{}", error.offset), error.context)
        })?;
        if actual != expected {
            return Err(candidate_error(
                "M2A-R33-HAK-READBACK",
                format!("hak.{resref}:{resource_type}"),
                "generated HAK resource differs from its exact input payload",
            ));
        }
    }

    let fixtures = [BinaryCreatureOwnedFixtureV1 {
        id: M0_R33_FIXTURE_ID.to_owned(),
        template_resref: M0_R33_FIXTURE_TEMPLATE_RESREF.to_owned(),
        display_name: M0_R33_FIXTURE_DISPLAY_NAME.to_owned(),
        appearance_row: M0_R33_APPEARANCE_ROW,
        position: M0RuntimePositionV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
        },
        orientation: M0RuntimeDirectionV1 { x: 1.0, y: 0.0 },
    }];
    let module = build_binary_creature_multi_fixture_module_v1(
        &BinaryCreatureModuleIdentityV1 {
            module_resref: M0_R33_MODULE_RESREF.to_owned(),
            area_resref: M0_R33_AREA_RESREF.to_owned(),
            hak_resref: M0_R33_HAK_RESREF.to_owned(),
        },
        &fixtures,
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let binary_scene = inspect_binary_creature_multi_fixture_module_v1(&module.payload)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if binary_scene != module.readback
        || binary_scene.ordered_hak_resrefs != [M0_R33_HAK_RESREF]
        || binary_scene.fixtures != fixtures
    {
        return Err(candidate_error(
            "M2A-R33-MODULE-READBACK",
            "module",
            "runtime-complete r33 MOD differs from its exact identity or fixture",
        ));
    }

    let contract = M0R33AnimatedDonorCandidateContractV1 {
        schema_version: 1,
        profile: "M0_R33_ANIMATED_DONOR_SKIN_CANDIDATE_V1".to_owned(),
        candidate_admissible: true,
        admitted_by_r32_proof_sha256: M0_R33_ACCEPTED_R32_PROOF_SHA256.to_owned(),
        diagnosed_failure_boundary:
            "r32 rigid mesh plus controller-free type-5 state runtime representation".to_owned(),
        intended_functional_delta: vec![
            "replace rigid deformation with donor-derived weighted SkinMesh".to_owned(),
            "replace controller-free state projections with donor-motion lifecycle clips"
                .to_owned(),
        ],
        source: byte_identity(source_glb),
        donor: byte_identity(donor_glb),
        base_appearance: byte_identity(base_appearance_two_da),
        retarget: retarget.report,
        skin_readback,
        model: resource_binding(M0_R33_MODEL_RESREF, &retarget.model.payload),
        texture: resource_binding(M0_R33_TEXTURE_RESREF, &texture.payload),
        appearance_two_da: resource_binding("appearance", &appearance.payload),
        hak: resource_binding(M0_R33_HAK_RESREF, &package.hak.payload),
        module: resource_binding(M0_R33_MODULE_RESREF, &module.payload),
        package_manifest: package.manifest,
        binary_scene,
        materialization_count: 1,
        toolset_model_visibility: "not_tested".to_owned(),
        toolset_proof_completeness: "missing".to_owned(),
        nwn_model_visibility: "not_tested".to_owned(),
        nwn_proof_completeness: "missing".to_owned(),
    };
    let mut contract_json = serde_json::to_vec_pretty(&contract).map_err(|error| {
        candidate_error(
            "M2A-R33-CONTRACT-SERIALIZATION",
            "contract",
            error.to_string(),
        )
    })?;
    contract_json.push(b'\n');

    Ok(M0R33AnimatedDonorCandidateArtifactV1 {
        model: retarget.model.payload,
        texture: texture.payload,
        appearance_two_da: appearance.payload,
        hak: package.hak.payload,
        module: module.payload,
        contract,
        contract_json,
    })
}

pub fn verify_m0_r33_animated_donor_candidate_v1(
    contract: &M0R33AnimatedDonorCandidateContractV1,
    source_glb: &[u8],
    donor_glb: &[u8],
    base_appearance_two_da: &[u8],
    module: &[u8],
    hak: &[u8],
) -> Result<(), M0R33AnimatedDonorCandidateErrorV1> {
    let replay =
        build_m0_r33_animated_donor_candidate_v1(source_glb, donor_glb, base_appearance_two_da)?;
    if contract != &replay.contract || module != replay.module || hak != replay.hak {
        return Err(candidate_error(
            "M2A-R33-REPLAY-MISMATCH",
            "candidate",
            "contract, MOD or HAK differs from the exact-input deterministic replay",
        ));
    }
    Ok(())
}

pub fn build_m0_r34_animated_donor_candidate_v1(
    source_glb: &[u8],
    donor_glb: &[u8],
    base_appearance_two_da: &[u8],
) -> Result<M0R34AnimatedDonorCandidateArtifactV1, M0R34AnimatedDonorCandidateErrorV1> {
    require_exact_r34_input(
        source_glb,
        M0_R33_SOURCE_BYTE_LENGTH,
        M0_R34_SOURCE_SHA256,
        "sourceGlb",
    )?;
    require_exact_r34_input(
        donor_glb,
        M0_R33_DONOR_BYTE_LENGTH,
        M0_R34_DONOR_SHA256,
        "donorGlb",
    )?;
    require_exact_r34_input(
        base_appearance_two_da,
        M0_R33_BASE_APPEARANCE_BYTE_LENGTH,
        M0_R34_BASE_APPEARANCE_SHA256,
        "baseAppearanceTwoDa",
    )?;
    let appearance_inspection =
        inspect_two_da_v2(base_appearance_two_da, &TwoDaLimitsV1::default())
            .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if appearance_inspection.physical_row_count != M0_R33_BASE_APPEARANCE_ROWS {
        return Err(candidate_error(
            "M2A-R34-APPEARANCE-TRUST-ROOT",
            "baseAppearanceTwoDa",
            "r34 requires the exact 15100-row full runtime appearance table",
        ));
    }

    let retarget = retarget_static_mesh_to_animated_donor_v1(
        source_glb,
        donor_glb,
        &MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64V1,
            state_projection_profile:
                MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
            state_projection_provenance: None,
            model_resource_resref: M0_R34_MODEL_RESREF.to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: M0_R34_TEXTURE_RESREF.to_owned(),
            }],
        },
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if retarget.report.model_sha256 != M0_R34_MODEL_SHA256
        || retarget.report.rig_node_count != 24
        || retarget.report.skin_segment_count != 1
        || retarget.report.active_bone_count != 22
        || retarget.report.animation_clip_names != M0_R34_REQUIRED_CLIPS
    {
        return Err(candidate_error(
            "M2A-R34-RETARGET-CONTRACT",
            "retarget.report",
            frozen_retarget_drift_message(
                "r34",
                M0_R34_MODEL_SHA256,
                &retarget.report.model_sha256,
                "exact M0/H1 retarget differs from the admitted rig-only state profile",
            ),
        ));
    }
    verify_direct_creature_state_projection_v1(
        &retarget.model.inspection,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
        None,
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let skin_readback = inspect_r34_skin_contract(&retarget.model.inspection)?;

    let ingest = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        candidate_error(
            error.code,
            error.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            error.message,
        )
    })?;
    let creature = retarget.conversion.creature.as_ref().ok_or_else(|| {
        candidate_error(
            "M2A-R34-RETARGET-CONTRACT",
            "retarget.conversion.creature",
            "successful retarget has no creature output",
        )
    })?;
    let texture_selection = resolve_base_color_image_index_v1(&ingest, creature)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let image = decode_embedded_image_to_tga_v1(
        source_glb,
        texture_selection.source_image_index,
        &GlbLimits::default(),
        &EmbeddedImageDecodeLimitsV1::default(),
    )
    .map_err(|error| {
        candidate_error(
            error.code,
            error
                .json_path
                .unwrap_or_else(|| "sourceGlb.images".to_owned()),
            error.message,
        )
    })?;
    let texture = write_tga_v1(&image, &TgaWriterOptionsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if sha256_bytes(&texture.payload) != M0_R34_TEXTURE_SHA256 {
        return Err(candidate_error(
            "M2A-R34-TEXTURE-IDENTITY",
            "texture",
            "r34 must preserve the exact M0 texture payload",
        ));
    }

    let appearance = append_r34_appearance(base_appearance_two_da)?;
    if appearance.report.appended_row_index != M0_R34_APPEARANCE_ROW {
        return Err(candidate_error(
            "M2A-R34-APPEARANCE-ROW",
            "appearance.appendedPhysicalRow",
            "r34 fixture must bind physical row 15100",
        ));
    }
    require_r34_appearance_cell(
        &appearance.payload,
        M0_R34_APPEARANCE_ROW,
        "LABEL",
        M0_R34_APPEARANCE_LABEL,
    )?;
    require_r34_appearance_cell(&appearance.payload, M0_R34_APPEARANCE_ROW, "MODELTYPE", "S")?;
    require_r34_appearance_cell(
        &appearance.payload,
        M0_R34_APPEARANCE_ROW,
        "RACE",
        M0_R34_MODEL_RESREF,
    )?;

    let resources = vec![
        HakResourceInputV1 {
            resref: M0_R34_MODEL_RESREF.to_owned(),
            resource_type: 2002,
            payload: retarget.model.payload.clone(),
        },
        HakResourceInputV1 {
            resref: M0_R34_TEXTURE_RESREF.to_owned(),
            resource_type: 3,
            payload: texture.payload.clone(),
        },
        HakResourceInputV1 {
            resref: "appearance".to_owned(),
            resource_type: 2017,
            payload: appearance.payload.clone(),
        },
    ];
    let package = write_model_package_v1(&resources, &HakWriterOptionsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let archive = ErfArchive::parse(&package.hak.payload).map_err(|error| {
        candidate_error(error.code, format!("hak@{}", error.offset), error.context)
    })?;
    for (resref, resource_type, expected) in [
        (M0_R34_MODEL_RESREF, 2002, retarget.model.payload.as_slice()),
        (M0_R34_TEXTURE_RESREF, 3, texture.payload.as_slice()),
        ("appearance", 2017, appearance.payload.as_slice()),
    ] {
        let actual = archive.find(resref, resource_type).map_err(|error| {
            candidate_error(error.code, format!("hak@{}", error.offset), error.context)
        })?;
        if actual != expected {
            return Err(candidate_error(
                "M2A-R34-HAK-READBACK",
                format!("hak.{resref}:{resource_type}"),
                "generated HAK resource differs from its exact input payload",
            ));
        }
    }

    let fixtures = [BinaryCreatureOwnedFixtureV1 {
        id: M0_R34_FIXTURE_ID.to_owned(),
        template_resref: M0_R34_FIXTURE_TEMPLATE_RESREF.to_owned(),
        display_name: M0_R34_FIXTURE_DISPLAY_NAME.to_owned(),
        appearance_row: M0_R34_APPEARANCE_ROW,
        position: M0RuntimePositionV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
        },
        orientation: M0RuntimeDirectionV1 { x: 1.0, y: 0.0 },
    }];
    let module = build_binary_creature_multi_fixture_module_v1(
        &BinaryCreatureModuleIdentityV1 {
            module_resref: M0_R34_MODULE_RESREF.to_owned(),
            area_resref: M0_R34_AREA_RESREF.to_owned(),
            hak_resref: M0_R34_HAK_RESREF.to_owned(),
        },
        &fixtures,
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let binary_scene = inspect_binary_creature_multi_fixture_module_v1(&module.payload)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if binary_scene != module.readback
        || binary_scene.ordered_hak_resrefs != [M0_R34_HAK_RESREF]
        || binary_scene.fixtures != fixtures
    {
        return Err(candidate_error(
            "M2A-R34-MODULE-READBACK",
            "module",
            "runtime-complete r34 MOD differs from its exact identity or fixture",
        ));
    }

    let contract = M0R34AnimatedDonorCandidateContractV1 {
        schema_version: 1,
        profile: "M0_R34_ANIMATED_DONOR_RIG_ONLY_STATE_CANDIDATE_V1".to_owned(),
        candidate_admissible: true,
        admitted_by_r33_proof_sha256: M0_R34_ACCEPTED_R33_PROOF_SHA256.to_owned(),
        diagnosed_failure_boundary:
            "r33 local type-5 states projected the base SkinMesh identity as m2a_seg_1".to_owned(),
        intended_functional_delta: vec![
            "preserve the exact r33 base rig, weighted SkinMesh, binds and controller tracks"
                .to_owned(),
            "omit every renderable mesh/skin leaf from each type-5 local-animation state"
                .to_owned(),
        ],
        source: byte_identity(source_glb),
        donor: byte_identity(donor_glb),
        base_appearance: byte_identity(base_appearance_two_da),
        retarget: retarget.report,
        skin_readback,
        model: resource_binding(M0_R34_MODEL_RESREF, &retarget.model.payload),
        texture: resource_binding(M0_R34_TEXTURE_RESREF, &texture.payload),
        appearance_two_da: resource_binding("appearance", &appearance.payload),
        hak: resource_binding(M0_R34_HAK_RESREF, &package.hak.payload),
        module: resource_binding(M0_R34_MODULE_RESREF, &module.payload),
        package_manifest: package.manifest,
        binary_scene,
        materialization_count: 1,
        toolset_model_visibility: "not_tested".to_owned(),
        toolset_proof_completeness: "missing".to_owned(),
        nwn_model_visibility: "not_tested".to_owned(),
        nwn_proof_completeness: "missing".to_owned(),
    };
    let mut contract_json = serde_json::to_vec_pretty(&contract).map_err(|error| {
        candidate_error(
            "M2A-R34-CONTRACT-SERIALIZATION",
            "contract",
            error.to_string(),
        )
    })?;
    contract_json.push(b'\n');

    Ok(M0R34AnimatedDonorCandidateArtifactV1 {
        model: retarget.model.payload,
        texture: texture.payload,
        appearance_two_da: appearance.payload,
        hak: package.hak.payload,
        module: module.payload,
        contract,
        contract_json,
    })
}

pub fn verify_m0_r34_animated_donor_candidate_v1(
    contract: &M0R34AnimatedDonorCandidateContractV1,
    source_glb: &[u8],
    donor_glb: &[u8],
    base_appearance_two_da: &[u8],
    module: &[u8],
    hak: &[u8],
) -> Result<(), M0R34AnimatedDonorCandidateErrorV1> {
    let replay =
        build_m0_r34_animated_donor_candidate_v1(source_glb, donor_glb, base_appearance_two_da)?;
    if contract != &replay.contract || module != replay.module || hak != replay.hak {
        return Err(candidate_error(
            "M2A-R34-REPLAY-MISMATCH",
            "candidate",
            "contract, MOD or HAK differs from the exact-input deterministic replay",
        ));
    }
    Ok(())
}

pub fn build_m0_r35_animated_donor_candidate_v1(
    source_glb: &[u8],
    donor_glb: &[u8],
    base_appearance_two_da: &[u8],
) -> Result<M0R35AnimatedDonorCandidateArtifactV1, M0R35AnimatedDonorCandidateErrorV1> {
    require_exact_r35_input(
        source_glb,
        M0_R33_SOURCE_BYTE_LENGTH,
        M0_R35_SOURCE_SHA256,
        "sourceGlb",
    )?;
    require_exact_r35_input(
        donor_glb,
        M0_R33_DONOR_BYTE_LENGTH,
        M0_R35_DONOR_SHA256,
        "donorGlb",
    )?;
    require_exact_r35_input(
        base_appearance_two_da,
        M0_R33_BASE_APPEARANCE_BYTE_LENGTH,
        M0_R35_BASE_APPEARANCE_SHA256,
        "baseAppearanceTwoDa",
    )?;
    let appearance_inspection =
        inspect_two_da_v2(base_appearance_two_da, &TwoDaLimitsV1::default())
            .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if appearance_inspection.physical_row_count != M0_R33_BASE_APPEARANCE_ROWS {
        return Err(candidate_error(
            "M2A-R35-APPEARANCE-TRUST-ROOT",
            "baseAppearanceTwoDa",
            "r35 requires the exact 15100-row full runtime appearance table",
        ));
    }

    let retarget = retarget_static_mesh_to_animated_donor_v1(
        source_glb,
        donor_glb,
        &MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2,
            state_projection_profile:
                MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
            state_projection_provenance: None,
            model_resource_resref: M0_R35_MODEL_RESREF.to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: M0_R35_TEXTURE_RESREF.to_owned(),
            }],
        },
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if retarget.report.model_sha256 != M0_R35_MODEL_SHA256
        || retarget.report.rig_node_count != 24
        || retarget.report.skin_segment_count != 1
        || retarget.report.active_bone_count != 22
        || retarget.report.animation_clip_names != M0_R35_REQUIRED_CLIPS
    {
        return Err(candidate_error(
            "M2A-R35-RETARGET-CONTRACT",
            "retarget.report",
            frozen_retarget_drift_message(
                "r35",
                M0_R35_MODEL_SHA256,
                &retarget.report.model_sha256,
                "exact M0/H1 retarget differs from the admitted zero-terminated SkinMesh profile",
            ),
        ));
    }
    verify_direct_creature_state_projection_v1(
        &retarget.model.inspection,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
        None,
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let skin_readback = inspect_r35_skin_contract(&retarget.model.inspection)?;

    let ingest = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        candidate_error(
            error.code,
            error.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            error.message,
        )
    })?;
    let creature = retarget.conversion.creature.as_ref().ok_or_else(|| {
        candidate_error(
            "M2A-R35-RETARGET-CONTRACT",
            "retarget.conversion.creature",
            "successful retarget has no creature output",
        )
    })?;
    let texture_selection = resolve_base_color_image_index_v1(&ingest, creature)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let image = decode_embedded_image_to_tga_v1(
        source_glb,
        texture_selection.source_image_index,
        &GlbLimits::default(),
        &EmbeddedImageDecodeLimitsV1::default(),
    )
    .map_err(|error| {
        candidate_error(
            error.code,
            error
                .json_path
                .unwrap_or_else(|| "sourceGlb.images".to_owned()),
            error.message,
        )
    })?;
    let texture = write_tga_v1(&image, &TgaWriterOptionsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?
        .payload;
    if sha256_bytes(&texture) != M0_R35_TEXTURE_SHA256 {
        return Err(candidate_error(
            "M2A-R35-TEXTURE-IDENTITY",
            "texture",
            "r35 must preserve the exact M0 texture payload",
        ));
    }
    let appearance = append_r35_appearance(base_appearance_two_da)?;
    if appearance.report.appended_row_index != M0_R35_APPEARANCE_ROW {
        return Err(candidate_error(
            "M2A-R35-APPEARANCE-ROW",
            "appearance.appendedPhysicalRow",
            "r35 fixture must bind physical row 15100",
        ));
    }
    require_r35_appearance_cell(
        &appearance.payload,
        M0_R35_APPEARANCE_ROW,
        "LABEL",
        M0_R35_APPEARANCE_LABEL,
    )?;
    require_r35_appearance_cell(&appearance.payload, M0_R35_APPEARANCE_ROW, "MODELTYPE", "S")?;
    require_r35_appearance_cell(
        &appearance.payload,
        M0_R35_APPEARANCE_ROW,
        "RACE",
        M0_R35_MODEL_RESREF,
    )?;

    let resources = vec![
        HakResourceInputV1 {
            resref: M0_R35_MODEL_RESREF.to_owned(),
            resource_type: 2002,
            payload: retarget.model.payload.clone(),
        },
        HakResourceInputV1 {
            resref: M0_R35_TEXTURE_RESREF.to_owned(),
            resource_type: 3,
            payload: texture.clone(),
        },
        HakResourceInputV1 {
            resref: "appearance".to_owned(),
            resource_type: 2017,
            payload: appearance.payload.clone(),
        },
    ];
    let package = write_model_package_v1(&resources, &HakWriterOptionsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let archive = ErfArchive::parse(&package.hak.payload).map_err(|error| {
        candidate_error(error.code, format!("hak@{}", error.offset), error.context)
    })?;
    for (resref, resource_type, expected) in [
        (M0_R35_MODEL_RESREF, 2002, retarget.model.payload.as_slice()),
        (M0_R35_TEXTURE_RESREF, 3, texture.as_slice()),
        ("appearance", 2017, appearance.payload.as_slice()),
    ] {
        let actual = archive.find(resref, resource_type).map_err(|error| {
            candidate_error(error.code, format!("hak@{}", error.offset), error.context)
        })?;
        if actual != expected {
            return Err(candidate_error(
                "M2A-R35-HAK-READBACK",
                format!("hak.{resref}:{resource_type}"),
                "generated HAK resource differs from its exact input payload",
            ));
        }
    }

    let fixtures = [BinaryCreatureOwnedFixtureV1 {
        id: M0_R35_FIXTURE_ID.to_owned(),
        template_resref: M0_R35_FIXTURE_TEMPLATE_RESREF.to_owned(),
        display_name: M0_R35_FIXTURE_DISPLAY_NAME.to_owned(),
        appearance_row: M0_R35_APPEARANCE_ROW,
        position: M0RuntimePositionV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
        },
        orientation: M0RuntimeDirectionV1 { x: 1.0, y: 0.0 },
    }];
    let module = build_binary_creature_multi_fixture_module_v1(
        &BinaryCreatureModuleIdentityV1 {
            module_resref: M0_R35_MODULE_RESREF.to_owned(),
            area_resref: M0_R35_AREA_RESREF.to_owned(),
            hak_resref: M0_R35_HAK_RESREF.to_owned(),
        },
        &fixtures,
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let binary_scene = inspect_binary_creature_multi_fixture_module_v1(&module.payload)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if binary_scene != module.readback
        || binary_scene.ordered_hak_resrefs != [M0_R35_HAK_RESREF]
        || binary_scene.fixtures != fixtures
    {
        return Err(candidate_error(
            "M2A-R35-MODULE-READBACK",
            "module",
            "runtime-complete r35 MOD differs from its exact identity or fixture",
        ));
    }

    let contract = M0R35AnimatedDonorCandidateContractV1 {
        schema_version: 1,
        profile: "M0_R35_NATIVE_ZERO_TERMINATED_SKIN_PALETTE_CANDIDATE_V1".to_owned(),
        candidate_admissible: true,
        admitted_by_r34_proof_sha256: M0_R35_ACCEPTED_R34_PROOF_SHA256.to_owned(),
        diagnosed_failure_boundary: "exact r34 rendered its weighted SkinMesh in Toolset but not in NWN; r34 filled all 42 unused inline palette slots with -1 while the native animated SkinMesh witness uses a zero terminator consumed by the Aurora palette loop".to_owned(),
        intended_functional_delta: vec![
            "preserve the exact r34 source, donor rig, weighted SkinMesh, inverse binds, controller tracks and rig-only type-5 states".to_owned(),
            "replace only the 42 unused extended64 inline palette entries from -1 to native-style zero terminators before candidate resref packaging".to_owned(),
        ],
        source: byte_identity(source_glb),
        donor: byte_identity(donor_glb),
        base_appearance: byte_identity(base_appearance_two_da),
        retarget: retarget.report,
        skin_readback,
        model: resource_binding(M0_R35_MODEL_RESREF, &retarget.model.payload),
        texture: resource_binding(M0_R35_TEXTURE_RESREF, &texture),
        appearance_two_da: resource_binding("appearance", &appearance.payload),
        hak: resource_binding(M0_R35_HAK_RESREF, &package.hak.payload),
        module: resource_binding(M0_R35_MODULE_RESREF, &module.payload),
        package_manifest: package.manifest,
        binary_scene,
        materialization_count: 1,
        toolset_model_visibility: "not_tested".to_owned(),
        toolset_proof_completeness: "missing".to_owned(),
        nwn_model_visibility: "not_tested".to_owned(),
        nwn_proof_completeness: "missing".to_owned(),
    };
    let mut contract_json = serde_json::to_vec_pretty(&contract).map_err(|error| {
        candidate_error(
            "M2A-R35-CONTRACT-SERIALIZATION",
            "contract",
            error.to_string(),
        )
    })?;
    contract_json.push(b'\n');

    Ok(M0R35AnimatedDonorCandidateArtifactV1 {
        model: retarget.model.payload,
        texture,
        appearance_two_da: appearance.payload,
        hak: package.hak.payload,
        module: module.payload,
        contract,
        contract_json,
    })
}

pub fn verify_m0_r35_animated_donor_candidate_v1(
    contract: &M0R35AnimatedDonorCandidateContractV1,
    source_glb: &[u8],
    donor_glb: &[u8],
    base_appearance_two_da: &[u8],
    module: &[u8],
    hak: &[u8],
) -> Result<(), M0R35AnimatedDonorCandidateErrorV1> {
    let replay =
        build_m0_r35_animated_donor_candidate_v1(source_glb, donor_glb, base_appearance_two_da)?;
    if contract != &replay.contract || module != replay.module || hak != replay.hak {
        return Err(candidate_error(
            "M2A-R35-REPLAY-MISMATCH",
            "candidate",
            "contract, MOD or HAK differs from the exact-input deterministic replay",
        ));
    }
    Ok(())
}

pub fn build_m0_r36_animated_donor_candidate_v1(
    source_glb: &[u8],
    donor_glb: &[u8],
    base_appearance_two_da: &[u8],
) -> Result<M0R36AnimatedDonorCandidateArtifactV1, M0R36AnimatedDonorCandidateErrorV1> {
    require_exact_r36_input(
        source_glb,
        M0_R33_SOURCE_BYTE_LENGTH,
        M0_R36_SOURCE_SHA256,
        "sourceGlb",
    )?;
    require_exact_r36_input(
        donor_glb,
        M0_R33_DONOR_BYTE_LENGTH,
        M0_R36_DONOR_SHA256,
        "donorGlb",
    )?;
    require_exact_r36_input(
        base_appearance_two_da,
        M0_R33_BASE_APPEARANCE_BYTE_LENGTH,
        M0_R36_BASE_APPEARANCE_SHA256,
        "baseAppearanceTwoDa",
    )?;
    let appearance_inspection =
        inspect_two_da_v2(base_appearance_two_da, &TwoDaLimitsV1::default())
            .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if appearance_inspection.physical_row_count != M0_R33_BASE_APPEARANCE_ROWS {
        return Err(candidate_error(
            "M2A-R36-APPEARANCE-TRUST-ROOT",
            "baseAppearanceTwoDa",
            "r36 requires the exact 15100-row full runtime appearance table",
        ));
    }

    let retarget = retarget_static_mesh_to_animated_donor_v2(
        source_glb,
        donor_glb,
        &MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2,
            state_projection_profile:
                MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
            state_projection_provenance: None,
            model_resource_resref: M0_R36_MODEL_RESREF.to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: M0_R36_TEXTURE_RESREF.to_owned(),
            }],
        },
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if retarget.report.model_sha256 != M0_R36_MODEL_SHA256
        || retarget.report.rig_node_count != 25
        || retarget.report.skin_segment_count != 1
        || retarget.report.active_bone_count != 22
        || retarget.report.animation_clip_names != M0_R36_REQUIRED_CLIPS
    {
        return Err(candidate_error(
            "M2A-R36-RETARGET-CONTRACT",
            "retarget.report",
            frozen_retarget_drift_message(
                "r36",
                M0_R36_MODEL_SHA256,
                &retarget.report.model_sha256,
                "exact M0/H1 retarget differs from the admitted dedicated Aurora Root profile",
            ),
        ));
    }
    verify_direct_creature_state_projection_v1(
        &retarget.model.inspection,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
        None,
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let skin_readback = inspect_r36_skin_contract(&retarget.model.inspection)?;

    let ingest = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        candidate_error(
            error.code,
            error.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            error.message,
        )
    })?;
    let creature = retarget.conversion.creature.as_ref().ok_or_else(|| {
        candidate_error(
            "M2A-R36-RETARGET-CONTRACT",
            "retarget.conversion.creature",
            "successful retarget has no creature output",
        )
    })?;
    let texture_selection = resolve_base_color_image_index_v1(&ingest, creature)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let image = decode_embedded_image_to_tga_v1(
        source_glb,
        texture_selection.source_image_index,
        &GlbLimits::default(),
        &EmbeddedImageDecodeLimitsV1::default(),
    )
    .map_err(|error| {
        candidate_error(
            error.code,
            error
                .json_path
                .unwrap_or_else(|| "sourceGlb.images".to_owned()),
            error.message,
        )
    })?;
    let texture = write_tga_v1(&image, &TgaWriterOptionsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?
        .payload;
    if sha256_bytes(&texture) != M0_R36_TEXTURE_SHA256 {
        return Err(candidate_error(
            "M2A-R36-TEXTURE-IDENTITY",
            "texture",
            "r36 must preserve the exact M0 texture payload",
        ));
    }
    let appearance = append_r36_appearance(base_appearance_two_da)?;
    if appearance.report.appended_row_index != M0_R36_APPEARANCE_ROW {
        return Err(candidate_error(
            "M2A-R36-APPEARANCE-ROW",
            "appearance.appendedPhysicalRow",
            "r36 fixture must bind physical row 15100",
        ));
    }
    require_r36_appearance_cell(
        &appearance.payload,
        M0_R36_APPEARANCE_ROW,
        "LABEL",
        M0_R36_APPEARANCE_LABEL,
    )?;
    require_r36_appearance_cell(&appearance.payload, M0_R36_APPEARANCE_ROW, "MODELTYPE", "S")?;
    require_r36_appearance_cell(
        &appearance.payload,
        M0_R36_APPEARANCE_ROW,
        "RACE",
        M0_R36_MODEL_RESREF,
    )?;

    let resources = vec![
        HakResourceInputV1 {
            resref: M0_R36_MODEL_RESREF.to_owned(),
            resource_type: 2002,
            payload: retarget.model.payload.clone(),
        },
        HakResourceInputV1 {
            resref: M0_R36_TEXTURE_RESREF.to_owned(),
            resource_type: 3,
            payload: texture.clone(),
        },
        HakResourceInputV1 {
            resref: "appearance".to_owned(),
            resource_type: 2017,
            payload: appearance.payload.clone(),
        },
    ];
    let package = write_model_package_v1(&resources, &HakWriterOptionsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let archive = ErfArchive::parse(&package.hak.payload).map_err(|error| {
        candidate_error(error.code, format!("hak@{}", error.offset), error.context)
    })?;
    for (resref, resource_type, expected) in [
        (M0_R36_MODEL_RESREF, 2002, retarget.model.payload.as_slice()),
        (M0_R36_TEXTURE_RESREF, 3, texture.as_slice()),
        ("appearance", 2017, appearance.payload.as_slice()),
    ] {
        let actual = archive.find(resref, resource_type).map_err(|error| {
            candidate_error(error.code, format!("hak@{}", error.offset), error.context)
        })?;
        if actual != expected {
            return Err(candidate_error(
                "M2A-R36-HAK-READBACK",
                format!("hak.{resref}:{resource_type}"),
                "generated HAK resource differs from its exact input payload",
            ));
        }
    }

    let fixtures = [BinaryCreatureOwnedFixtureV1 {
        id: M0_R36_FIXTURE_ID.to_owned(),
        template_resref: M0_R36_FIXTURE_TEMPLATE_RESREF.to_owned(),
        display_name: M0_R36_FIXTURE_DISPLAY_NAME.to_owned(),
        appearance_row: M0_R36_APPEARANCE_ROW,
        position: M0RuntimePositionV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
        },
        orientation: M0RuntimeDirectionV1 { x: 1.0, y: 0.0 },
    }];
    let module = build_binary_creature_multi_fixture_module_v1(
        &BinaryCreatureModuleIdentityV1 {
            module_resref: M0_R36_MODULE_RESREF.to_owned(),
            area_resref: M0_R36_AREA_RESREF.to_owned(),
            hak_resref: M0_R36_HAK_RESREF.to_owned(),
        },
        &fixtures,
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let binary_scene = inspect_binary_creature_multi_fixture_module_v1(&module.payload)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if binary_scene != module.readback
        || binary_scene.ordered_hak_resrefs != [M0_R36_HAK_RESREF]
        || binary_scene.fixtures != fixtures
    {
        return Err(candidate_error(
            "M2A-R36-MODULE-READBACK",
            "module",
            "runtime-complete r36 MOD differs from its exact identity or fixture",
        ));
    }

    let contract = M0R36AnimatedDonorCandidateContractV1 {
        schema_version: 1,
        profile: "M0_R36_DEDICATED_UNWEIGHTED_AURORA_ROOT_CANDIDATE_V1".to_owned(),
        candidate_admissible: true,
        admitted_by_r35_proof_sha256: M0_R36_ACCEPTED_R35_PROOF_SHA256.to_owned(),
        diagnosed_failure_boundary: "exact r35 rendered in Toolset but not in NWN, disproving the unused -1 palette tail as the sole cause; r35 also collapsed the required model-named Aurora Root onto the heavily weighted Hips skeleton joint, unlike native SkinMesh witnesses whose tree ordinal zero root is not active in the skin palette".to_owned(),
        intended_functional_delta: vec![
            "preserve the exact r35 source, donor joints, bind-local transforms, weighted SkinMesh, inverse binds, controller tracks, zero-terminated palette and rig-only type-5 states".to_owned(),
            "insert one identity, unweighted Aurora Root named after the model resref above the preserved Hips joint so tree ordinal zero is excluded from the active skin palette".to_owned(),
        ],
        source: byte_identity(source_glb),
        donor: byte_identity(donor_glb),
        base_appearance: byte_identity(base_appearance_two_da),
        retarget: retarget.report,
        skin_readback,
        model: resource_binding(M0_R36_MODEL_RESREF, &retarget.model.payload),
        texture: resource_binding(M0_R36_TEXTURE_RESREF, &texture),
        appearance_two_da: resource_binding("appearance", &appearance.payload),
        hak: resource_binding(M0_R36_HAK_RESREF, &package.hak.payload),
        module: resource_binding(M0_R36_MODULE_RESREF, &module.payload),
        package_manifest: package.manifest,
        binary_scene,
        materialization_count: 1,
        toolset_model_visibility: "not_tested".to_owned(),
        toolset_proof_completeness: "missing".to_owned(),
        nwn_model_visibility: "not_tested".to_owned(),
        nwn_proof_completeness: "missing".to_owned(),
    };
    let mut contract_json = serde_json::to_vec_pretty(&contract).map_err(|error| {
        candidate_error(
            "M2A-R36-CONTRACT-SERIALIZATION",
            "contract",
            error.to_string(),
        )
    })?;
    contract_json.push(b'\n');

    Ok(M0R36AnimatedDonorCandidateArtifactV1 {
        model: retarget.model.payload,
        texture,
        appearance_two_da: appearance.payload,
        hak: package.hak.payload,
        module: module.payload,
        contract,
        contract_json,
    })
}

pub fn verify_m0_r36_animated_donor_candidate_v1(
    contract: &M0R36AnimatedDonorCandidateContractV1,
    source_glb: &[u8],
    donor_glb: &[u8],
    base_appearance_two_da: &[u8],
    module: &[u8],
    hak: &[u8],
) -> Result<(), M0R36AnimatedDonorCandidateErrorV1> {
    let replay =
        build_m0_r36_animated_donor_candidate_v1(source_glb, donor_glb, base_appearance_two_da)?;
    if contract != &replay.contract || module != replay.module || hak != replay.hak {
        return Err(candidate_error(
            "M2A-R36-REPLAY-MISMATCH",
            "candidate",
            "contract, MOD or HAK differs from the exact-input deterministic replay",
        ));
    }
    Ok(())
}

fn inspect_r36_skin_contract(
    inspection: &crate::mdl::InspectionReport,
) -> Result<M0R36SkinReadbackV1, M0R36AnimatedDonorCandidateErrorV1> {
    let [aurora_root] = inspection.node_tree.roots.as_slice() else {
        return Err(candidate_error(
            "M2A-R36-MDL-READBACK",
            "model.nodeTree.roots",
            "r36 requires exactly one dedicated Aurora Root",
        ));
    };
    let skeleton_root = aurora_root
        .children
        .iter()
        .find(|node| node.name == "Hips")
        .ok_or_else(|| {
            candidate_error(
                "M2A-R36-MDL-READBACK",
                "model.nodeTree.roots[0].children",
                "r36 must preserve Hips immediately below the dedicated Aurora Root",
            )
        })?;
    let mut skins = Vec::new();
    collect_skin_nodes(&inspection.node_tree.roots, &mut skins);
    let animation_node_counts = inspection
        .animations
        .iter()
        .map(|animation| animation.node_tree.node_count)
        .collect::<Vec<_>>();
    if inspection.model.name != M0_R36_MODEL_RESREF
        || aurora_root.name != M0_R36_MODEL_RESREF
        || aurora_root.parent_offset.is_some()
        || skeleton_root.parent_offset != Some(aurora_root.offset)
        || inspection.node_tree.node_count != 26
        || skins.len() != 1
        || inspection.animations.len() != M0_R36_REQUIRED_CLIPS.len()
        || animation_node_counts.iter().any(|count| *count != 25)
    {
        return Err(candidate_error(
            "M2A-R36-MDL-READBACK",
            "model",
            "r36 requires a 26-node base rooted by the model-named Aurora Root, preserved child Hips, one SkinMesh, seven animations and 25 rig-only nodes per state",
        ));
    }
    let skin = skins[0].skin.as_ref().expect("collected skin node");
    let active_bone_count = skin
        .node_to_bone_map
        .iter()
        .copied()
        .filter(|value| *value >= 0)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let active_inline_slot_count = skin
        .bone_references
        .iter()
        .flat_map(|references| references.iter().copied())
        .filter(|reference| *reference != u16::MAX)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let active_inline = skin
        .inline_mapping
        .get(..active_inline_slot_count)
        .ok_or_else(|| {
            candidate_error(
                "M2A-R36-SKIN-READBACK",
                "model.skin.inlineMapping",
                "active inline slot count is outside the 64-entry palette",
            )
        })?;
    let unused_inline_tail = skin
        .inline_mapping
        .get(active_inline_slot_count..)
        .ok_or_else(|| {
            candidate_error(
                "M2A-R36-SKIN-READBACK",
                "model.skin.inlineMapping",
                "active inline slot count is outside the 64-entry palette",
            )
        })?;
    if skin.node_to_bone_map.len() != 26
        || skin.node_to_bone_map[0] != -1
        || skin.q_header.used != 26
        || skin.t_header.used != 26
        || skin.constants_header.used != 26
        || skin.vertex_weights.len() != 2_380
        || skin.bone_references.len() != skin.vertex_weights.len()
        || active_bone_count != 22
        || active_inline_slot_count != 22
        || active_inline != (1_i16..=22).collect::<Vec<_>>()
        || unused_inline_tail.len() != 42
        || unused_inline_tail.iter().any(|value| *value != 0)
        || skin.vertex_weights.iter().any(|weights| {
            let sum = weights.iter().sum::<f32>();
            !sum.is_finite() || (sum - 1.0).abs() > 0.00001
        })
    {
        return Err(candidate_error(
            "M2A-R36-SKIN-READBACK",
            "model.skin",
            "r36 SkinMesh must exclude tree ordinal zero while preserving 22 dense active joints, normalized weights and a zero-terminated unused tail",
        ));
    }
    let animation_names = inspection
        .animations
        .iter()
        .map(|animation| animation.name.clone())
        .collect::<Vec<_>>();
    if animation_names != M0_R36_REQUIRED_CLIPS {
        return Err(candidate_error(
            "M2A-R36-ANIMATION-READBACK",
            "model.animations",
            "r36 animation names or order differ from the exact r35 inventory",
        ));
    }
    Ok(M0R36SkinReadbackV1 {
        model_name: inspection.model.name.clone(),
        base_node_count: inspection.node_tree.node_count,
        rig_node_count: 25,
        skin_node_count: skins.len(),
        weighted_vertex_count: skin.vertex_weights.len(),
        active_bone_count,
        active_inline_slot_count,
        unused_inline_tail_count: unused_inline_tail.len(),
        unused_inline_tail_value: 0,
        aurora_root_name: aurora_root.name.clone(),
        skeleton_root_name: skeleton_root.name.clone(),
        aurora_root_tree_ordinal: 0,
        aurora_root_forward_slot: skin.node_to_bone_map[0],
        first_active_inline_ordinal: active_inline[0],
        last_active_inline_ordinal: active_inline[active_inline.len() - 1],
        animation_names,
        animation_node_counts,
        format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2,
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
    })
}

pub fn build_m0_r37_animated_donor_candidate_v1(
    source_glb: &[u8],
    donor_glb: &[u8],
    base_appearance_two_da: &[u8],
) -> Result<M0R37AnimatedDonorCandidateArtifactV1, M0R37AnimatedDonorCandidateErrorV1> {
    require_exact_r37_input(
        source_glb,
        M0_R33_SOURCE_BYTE_LENGTH,
        M0_R37_SOURCE_SHA256,
        "sourceGlb",
    )?;
    require_exact_r37_input(
        donor_glb,
        M0_R33_DONOR_BYTE_LENGTH,
        M0_R37_DONOR_SHA256,
        "donorGlb",
    )?;
    require_exact_r37_input(
        base_appearance_two_da,
        M0_R33_BASE_APPEARANCE_BYTE_LENGTH,
        M0_R37_BASE_APPEARANCE_SHA256,
        "baseAppearanceTwoDa",
    )?;
    let appearance_inspection =
        inspect_two_da_v2(base_appearance_two_da, &TwoDaLimitsV1::default())
            .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if appearance_inspection.physical_row_count != M0_R33_BASE_APPEARANCE_ROWS {
        return Err(candidate_error(
            "M2A-R37-APPEARANCE-TRUST-ROOT",
            "baseAppearanceTwoDa",
            "r37 requires the exact 15100-row full runtime appearance table",
        ));
    }

    let retarget = retarget_static_mesh_to_animated_donor_v3(
        source_glb,
        donor_glb,
        &MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2,
            state_projection_profile:
                MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
            state_projection_provenance: None,
            model_resource_resref: M0_R37_MODEL_RESREF.to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: M0_R37_TEXTURE_RESREF.to_owned(),
            }],
        },
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if retarget.report.model_sha256 != M0_R37_MODEL_SHA256
        || retarget.report.rig_node_count != 25
        || retarget.report.skin_segment_count != 1
        || retarget.report.active_bone_count != 22
        || retarget.report.animation_clip_names != M0_R37_REQUIRED_CLIPS
    {
        return Err(candidate_error(
            "M2A-R37-RETARGET-CONTRACT",
            "retarget.report",
            frozen_retarget_drift_message(
                "r37",
                M0_R37_MODEL_SHA256,
                &retarget.report.model_sha256,
                "exact M0/H1 retarget differs from the admitted direct-root SkinMesh profile",
            ),
        ));
    }
    verify_direct_creature_state_projection_v1(
        &retarget.model.inspection,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
        None,
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let skin_readback = inspect_r37_skin_contract(&retarget.model.inspection)?;

    let ingest = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        candidate_error(
            error.code,
            error.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            error.message,
        )
    })?;
    let creature = retarget.conversion.creature.as_ref().ok_or_else(|| {
        candidate_error(
            "M2A-R37-RETARGET-CONTRACT",
            "retarget.conversion.creature",
            "successful retarget has no creature output",
        )
    })?;
    let texture_selection = resolve_base_color_image_index_v1(&ingest, creature)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let image = decode_embedded_image_to_tga_v1(
        source_glb,
        texture_selection.source_image_index,
        &GlbLimits::default(),
        &EmbeddedImageDecodeLimitsV1::default(),
    )
    .map_err(|error| {
        candidate_error(
            error.code,
            error
                .json_path
                .unwrap_or_else(|| "sourceGlb.images".to_owned()),
            error.message,
        )
    })?;
    let texture = write_tga_v1(&image, &TgaWriterOptionsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?
        .payload;
    if sha256_bytes(&texture) != M0_R37_TEXTURE_SHA256 {
        return Err(candidate_error(
            "M2A-R37-TEXTURE-IDENTITY",
            "texture",
            "r37 must preserve the exact M0 texture payload",
        ));
    }
    let appearance = append_r37_appearance(base_appearance_two_da)?;
    if appearance.report.appended_row_index != M0_R37_APPEARANCE_ROW {
        return Err(candidate_error(
            "M2A-R37-APPEARANCE-ROW",
            "appearance.appendedPhysicalRow",
            "r37 fixture must bind physical row 15100",
        ));
    }
    require_r37_appearance_cell(
        &appearance.payload,
        M0_R37_APPEARANCE_ROW,
        "LABEL",
        M0_R37_APPEARANCE_LABEL,
    )?;
    require_r37_appearance_cell(&appearance.payload, M0_R37_APPEARANCE_ROW, "MODELTYPE", "S")?;
    require_r37_appearance_cell(
        &appearance.payload,
        M0_R37_APPEARANCE_ROW,
        "RACE",
        M0_R37_MODEL_RESREF,
    )?;

    let resources = vec![
        HakResourceInputV1 {
            resref: M0_R37_MODEL_RESREF.to_owned(),
            resource_type: 2002,
            payload: retarget.model.payload.clone(),
        },
        HakResourceInputV1 {
            resref: M0_R37_TEXTURE_RESREF.to_owned(),
            resource_type: 3,
            payload: texture.clone(),
        },
        HakResourceInputV1 {
            resref: "appearance".to_owned(),
            resource_type: 2017,
            payload: appearance.payload.clone(),
        },
    ];
    let package = write_model_package_v1(&resources, &HakWriterOptionsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let archive = ErfArchive::parse(&package.hak.payload).map_err(|error| {
        candidate_error(error.code, format!("hak@{}", error.offset), error.context)
    })?;
    for (resref, resource_type, expected) in [
        (M0_R37_MODEL_RESREF, 2002, retarget.model.payload.as_slice()),
        (M0_R37_TEXTURE_RESREF, 3, texture.as_slice()),
        ("appearance", 2017, appearance.payload.as_slice()),
    ] {
        let actual = archive.find(resref, resource_type).map_err(|error| {
            candidate_error(error.code, format!("hak@{}", error.offset), error.context)
        })?;
        if actual != expected {
            return Err(candidate_error(
                "M2A-R37-HAK-READBACK",
                format!("hak.{resref}:{resource_type}"),
                "generated HAK resource differs from its exact input payload",
            ));
        }
    }

    let fixtures = [BinaryCreatureOwnedFixtureV1 {
        id: M0_R37_FIXTURE_ID.to_owned(),
        template_resref: M0_R37_FIXTURE_TEMPLATE_RESREF.to_owned(),
        display_name: M0_R37_FIXTURE_DISPLAY_NAME.to_owned(),
        appearance_row: M0_R37_APPEARANCE_ROW,
        position: M0RuntimePositionV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
        },
        orientation: M0RuntimeDirectionV1 { x: 1.0, y: 0.0 },
    }];
    let module = build_binary_creature_multi_fixture_module_v1(
        &BinaryCreatureModuleIdentityV1 {
            module_resref: M0_R37_MODULE_RESREF.to_owned(),
            area_resref: M0_R37_AREA_RESREF.to_owned(),
            hak_resref: M0_R37_HAK_RESREF.to_owned(),
        },
        &fixtures,
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let binary_scene = inspect_binary_creature_multi_fixture_module_v1(&module.payload)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if binary_scene != module.readback
        || binary_scene.ordered_hak_resrefs != [M0_R37_HAK_RESREF]
        || binary_scene.fixtures != fixtures
    {
        return Err(candidate_error(
            "M2A-R37-MODULE-READBACK",
            "module",
            "runtime-complete r37 MOD differs from its exact identity or fixture",
        ));
    }

    let contract = M0R37AnimatedDonorCandidateContractV1 {
        schema_version: 1,
        profile: "M0_R37_DIRECT_ROOT_SKINMESH_CANDIDATE_V1".to_owned(),
        candidate_admissible: true,
        admitted_by_r36_proof_sha256: M0_R37_ACCEPTED_R36_PROOF_SHA256.to_owned(),
        diagnosed_failure_boundary: "exact r36 remained absent in NWN after adding a dedicated unweighted Aurora Root; r36 alone among 74 locally observed SkinMesh nodes attached its SkinMesh below the weighted skeleton root instead of directly below the model root".to_owned(),
        intended_functional_delta: vec![
            "preserve the exact r36 source, donor joints, weights, refs, UVs, indices, animation tracks, zero-terminated palette and rig-only type-5 states".to_owned(),
            "bake the former Hips bind-local rigid transform into SkinMesh positions, normals and tangents, then attach the SkinMesh directly to the dedicated Aurora Root while preserving exact bind-pose world geometry".to_owned(),
        ],
        source: byte_identity(source_glb),
        donor: byte_identity(donor_glb),
        base_appearance: byte_identity(base_appearance_two_da),
        retarget: retarget.report,
        skin_readback,
        model: resource_binding(M0_R37_MODEL_RESREF, &retarget.model.payload),
        texture: resource_binding(M0_R37_TEXTURE_RESREF, &texture),
        appearance_two_da: resource_binding("appearance", &appearance.payload),
        hak: resource_binding(M0_R37_HAK_RESREF, &package.hak.payload),
        module: resource_binding(M0_R37_MODULE_RESREF, &module.payload),
        package_manifest: package.manifest,
        binary_scene,
        materialization_count: 1,
        toolset_model_visibility: "not_tested".to_owned(),
        toolset_proof_completeness: "missing".to_owned(),
        nwn_model_visibility: "not_tested".to_owned(),
        nwn_proof_completeness: "missing".to_owned(),
    };
    let mut contract_json = serde_json::to_vec_pretty(&contract).map_err(|error| {
        candidate_error(
            "M2A-R37-CONTRACT-SERIALIZATION",
            "contract",
            error.to_string(),
        )
    })?;
    contract_json.push(b'\n');

    Ok(M0R37AnimatedDonorCandidateArtifactV1 {
        model: retarget.model.payload,
        texture,
        appearance_two_da: appearance.payload,
        hak: package.hak.payload,
        module: module.payload,
        contract,
        contract_json,
    })
}

pub fn verify_m0_r37_animated_donor_candidate_v1(
    contract: &M0R37AnimatedDonorCandidateContractV1,
    source_glb: &[u8],
    donor_glb: &[u8],
    base_appearance_two_da: &[u8],
    module: &[u8],
    hak: &[u8],
) -> Result<(), M0R37AnimatedDonorCandidateErrorV1> {
    let replay =
        build_m0_r37_animated_donor_candidate_v1(source_glb, donor_glb, base_appearance_two_da)?;
    if contract != &replay.contract || module != replay.module || hak != replay.hak {
        return Err(candidate_error(
            "M2A-R37-REPLAY-MISMATCH",
            "candidate",
            "contract, MOD or HAK differs from the exact-input deterministic replay",
        ));
    }
    Ok(())
}

pub fn build_m0_r38_animated_donor_candidate_v1(
    source_glb: &[u8],
    donor_glb: &[u8],
    base_appearance_two_da: &[u8],
) -> Result<M0R38AnimatedDonorCandidateArtifactV1, M0R38AnimatedDonorCandidateErrorV1> {
    require_exact_r38_input(
        source_glb,
        M0_R33_SOURCE_BYTE_LENGTH,
        M0_R38_SOURCE_SHA256,
        "sourceGlb",
    )?;
    require_exact_r38_input(
        donor_glb,
        M0_R33_DONOR_BYTE_LENGTH,
        M0_R38_DONOR_SHA256,
        "donorGlb",
    )?;
    require_exact_r38_input(
        base_appearance_two_da,
        M0_R33_BASE_APPEARANCE_BYTE_LENGTH,
        M0_R38_BASE_APPEARANCE_SHA256,
        "baseAppearanceTwoDa",
    )?;
    let appearance_inspection =
        inspect_two_da_v2(base_appearance_two_da, &TwoDaLimitsV1::default())
            .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if appearance_inspection.physical_row_count != M0_R33_BASE_APPEARANCE_ROWS {
        return Err(candidate_error(
            "M2A-R38-APPEARANCE-TRUST-ROOT",
            "baseAppearanceTwoDa",
            "r38 requires the exact 15100-row full runtime appearance table",
        ));
    }

    let retarget = retarget_static_mesh_to_animated_donor_v4(
        source_glb,
        donor_glb,
        &MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2,
            state_projection_profile:
                MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
            state_projection_provenance: None,
            model_resource_resref: M0_R38_MODEL_RESREF.to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: M0_R38_TEXTURE_RESREF.to_owned(),
            }],
        },
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if retarget.report.model_sha256 != M0_R38_MODEL_SHA256
        || retarget.report.rig_node_count != 25
        || retarget.report.skin_segment_count != 1
        || retarget.report.active_bone_count != 22
        || retarget.report.animation_clip_names != M0_R38_REQUIRED_CLIPS
        || retarget.animations.clips.iter().any(|clip| {
            clip.tracks
                .iter()
                .any(|track| track.path == crate::mdl::MdlAnimationTrackPathV1::Scale)
        })
    {
        return Err(candidate_error(
            "M2A-R38-RETARGET-CONTRACT",
            "retarget",
            frozen_retarget_drift_message(
                "r38",
                M0_R38_MODEL_SHA256,
                &retarget.report.model_sha256,
                "exact M0/H1 retarget differs from the admitted scale-normalized direct-root SkinMesh profile",
            ),
        ));
    }
    verify_direct_creature_state_projection_v1(
        &retarget.model.inspection,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
        None,
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let skin_readback = inspect_r38_skin_contract(&retarget.model.inspection)?;

    let ingest = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        candidate_error(
            error.code,
            error.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            error.message,
        )
    })?;
    let creature = retarget.conversion.creature.as_ref().ok_or_else(|| {
        candidate_error(
            "M2A-R38-RETARGET-CONTRACT",
            "retarget.conversion.creature",
            "successful retarget has no creature output",
        )
    })?;
    let texture_selection = resolve_base_color_image_index_v1(&ingest, creature)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let image = decode_embedded_image_to_tga_v1(
        source_glb,
        texture_selection.source_image_index,
        &GlbLimits::default(),
        &EmbeddedImageDecodeLimitsV1::default(),
    )
    .map_err(|error| {
        candidate_error(
            error.code,
            error
                .json_path
                .unwrap_or_else(|| "sourceGlb.images".to_owned()),
            error.message,
        )
    })?;
    let texture = write_tga_v1(&image, &TgaWriterOptionsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?
        .payload;
    if sha256_bytes(&texture) != M0_R38_TEXTURE_SHA256 {
        return Err(candidate_error(
            "M2A-R38-TEXTURE-IDENTITY",
            "texture",
            "r38 must preserve the exact M0 texture payload",
        ));
    }
    let appearance = append_r38_appearance(base_appearance_two_da)?;
    if appearance.report.appended_row_index != M0_R38_APPEARANCE_ROW {
        return Err(candidate_error(
            "M2A-R38-APPEARANCE-ROW",
            "appearance.appendedPhysicalRow",
            "r38 fixture must bind physical row 15100",
        ));
    }
    require_r38_appearance_cell(
        &appearance.payload,
        M0_R38_APPEARANCE_ROW,
        "LABEL",
        M0_R38_APPEARANCE_LABEL,
    )?;
    require_r38_appearance_cell(&appearance.payload, M0_R38_APPEARANCE_ROW, "MODELTYPE", "S")?;
    require_r38_appearance_cell(
        &appearance.payload,
        M0_R38_APPEARANCE_ROW,
        "RACE",
        M0_R38_MODEL_RESREF,
    )?;

    let resources = vec![
        HakResourceInputV1 {
            resref: M0_R38_MODEL_RESREF.to_owned(),
            resource_type: 2002,
            payload: retarget.model.payload.clone(),
        },
        HakResourceInputV1 {
            resref: M0_R38_TEXTURE_RESREF.to_owned(),
            resource_type: 3,
            payload: texture.clone(),
        },
        HakResourceInputV1 {
            resref: "appearance".to_owned(),
            resource_type: 2017,
            payload: appearance.payload.clone(),
        },
    ];
    let package = write_model_package_v1(&resources, &HakWriterOptionsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let archive = ErfArchive::parse(&package.hak.payload).map_err(|error| {
        candidate_error(error.code, format!("hak@{}", error.offset), error.context)
    })?;
    for (resref, resource_type, expected) in [
        (M0_R38_MODEL_RESREF, 2002, retarget.model.payload.as_slice()),
        (M0_R38_TEXTURE_RESREF, 3, texture.as_slice()),
        ("appearance", 2017, appearance.payload.as_slice()),
    ] {
        let actual = archive.find(resref, resource_type).map_err(|error| {
            candidate_error(error.code, format!("hak@{}", error.offset), error.context)
        })?;
        if actual != expected {
            return Err(candidate_error(
                "M2A-R38-HAK-READBACK",
                format!("hak.{resref}:{resource_type}"),
                "generated HAK resource differs from its exact input payload",
            ));
        }
    }

    let fixtures = [BinaryCreatureOwnedFixtureV1 {
        id: M0_R38_FIXTURE_ID.to_owned(),
        template_resref: M0_R38_FIXTURE_TEMPLATE_RESREF.to_owned(),
        display_name: M0_R38_FIXTURE_DISPLAY_NAME.to_owned(),
        appearance_row: M0_R38_APPEARANCE_ROW,
        position: M0RuntimePositionV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
        },
        orientation: M0RuntimeDirectionV1 { x: 1.0, y: 0.0 },
    }];
    let module = build_binary_creature_multi_fixture_module_v1(
        &BinaryCreatureModuleIdentityV1 {
            module_resref: M0_R38_MODULE_RESREF.to_owned(),
            area_resref: M0_R38_AREA_RESREF.to_owned(),
            hak_resref: M0_R38_HAK_RESREF.to_owned(),
        },
        &fixtures,
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let binary_scene = inspect_binary_creature_multi_fixture_module_v1(&module.payload)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if binary_scene != module.readback
        || binary_scene.ordered_hak_resrefs != [M0_R38_HAK_RESREF]
        || binary_scene.fixtures != fixtures
    {
        return Err(candidate_error(
            "M2A-R38-MODULE-READBACK",
            "module",
            "runtime-complete r38 MOD differs from its exact identity or fixture",
        ));
    }

    let contract = M0R38AnimatedDonorCandidateContractV1 {
        schema_version: 1,
        profile: "M0_R38_SCALE_NORMALIZED_SKINMESH_CANDIDATE_V1".to_owned(),
        candidate_admissible: true,
        admitted_by_r37_proof_sha256: M0_R38_ACCEPTED_R37_PROOF_SHA256.to_owned(),
        diagnosed_failure_boundary: "exact r37 remained absent in NWN; all seven r37 runtime clips contain a constant non-unit Hips scale controller while the observed native creature animation corpus has no animated scale controllers and the runtime SkinMesh palette composes translation and quaternion data without scale".to_owned(),
        intended_functional_delta: vec![
            "preserve the exact r37 geometry, direct-root SkinMesh topology, bind pose, inverse binds, weights, refs, UVs, indices, materials, translations, rotations, clips and rig-only type-5 states".to_owned(),
            "require the same positive constant scale on the preserved weighted Hips root in every materialized clip and remove only those seven unrepresentable scale tracks before binary writing".to_owned(),
        ],
        source: byte_identity(source_glb),
        donor: byte_identity(donor_glb),
        base_appearance: byte_identity(base_appearance_two_da),
        retarget: retarget.report,
        skin_readback,
        model: resource_binding(M0_R38_MODEL_RESREF, &retarget.model.payload),
        texture: resource_binding(M0_R38_TEXTURE_RESREF, &texture),
        appearance_two_da: resource_binding("appearance", &appearance.payload),
        hak: resource_binding(M0_R38_HAK_RESREF, &package.hak.payload),
        module: resource_binding(M0_R38_MODULE_RESREF, &module.payload),
        package_manifest: package.manifest,
        binary_scene,
        materialization_count: 1,
        toolset_model_visibility: "not_tested".to_owned(),
        toolset_proof_completeness: "missing".to_owned(),
        nwn_model_visibility: "not_tested".to_owned(),
        nwn_proof_completeness: "missing".to_owned(),
    };
    let mut contract_json = serde_json::to_vec_pretty(&contract).map_err(|error| {
        candidate_error(
            "M2A-R38-CONTRACT-SERIALIZATION",
            "contract",
            error.to_string(),
        )
    })?;
    contract_json.push(b'\n');

    Ok(M0R38AnimatedDonorCandidateArtifactV1 {
        model: retarget.model.payload,
        texture,
        appearance_two_da: appearance.payload,
        hak: package.hak.payload,
        module: module.payload,
        contract,
        contract_json,
    })
}

pub fn verify_m0_r38_animated_donor_candidate_v1(
    contract: &M0R38AnimatedDonorCandidateContractV1,
    source_glb: &[u8],
    donor_glb: &[u8],
    base_appearance_two_da: &[u8],
    module: &[u8],
    hak: &[u8],
) -> Result<(), M0R38AnimatedDonorCandidateErrorV1> {
    let replay =
        build_m0_r38_animated_donor_candidate_v1(source_glb, donor_glb, base_appearance_two_da)?;
    if contract != &replay.contract || module != replay.module || hak != replay.hak {
        return Err(candidate_error(
            "M2A-R38-REPLAY-MISMATCH",
            "candidate",
            "contract, MOD or HAK differs from the exact-input deterministic replay",
        ));
    }
    Ok(())
}

pub fn build_m0_r39_animated_donor_candidate_v1(
    source_glb: &[u8],
    donor_glb: &[u8],
    base_appearance_two_da: &[u8],
) -> Result<M0R39AnimatedDonorCandidateArtifactV1, M0R39AnimatedDonorCandidateErrorV1> {
    require_exact_r39_input(
        source_glb,
        M0_R33_SOURCE_BYTE_LENGTH,
        M0_R39_SOURCE_SHA256,
        "sourceGlb",
    )?;
    require_exact_r39_input(
        donor_glb,
        M0_R33_DONOR_BYTE_LENGTH,
        M0_R39_DONOR_SHA256,
        "donorGlb",
    )?;
    require_exact_r39_input(
        base_appearance_two_da,
        M0_R33_BASE_APPEARANCE_BYTE_LENGTH,
        M0_R39_BASE_APPEARANCE_SHA256,
        "baseAppearanceTwoDa",
    )?;
    let appearance_inspection =
        inspect_two_da_v2(base_appearance_two_da, &TwoDaLimitsV1::default())
            .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if appearance_inspection.physical_row_count != M0_R33_BASE_APPEARANCE_ROWS {
        return Err(candidate_error(
            "M2A-R39-APPEARANCE-TRUST-ROOT",
            "baseAppearanceTwoDa",
            "r39 requires the exact 15100-row full runtime appearance table",
        ));
    }

    let retarget = retarget_static_mesh_to_animated_donor_v4(
        source_glb,
        donor_glb,
        &MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile:
                MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedControllerlessRootV3,
            state_projection_profile:
                MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
            state_projection_provenance: None,
            model_resource_resref: M0_R39_MODEL_RESREF.to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: M0_R39_TEXTURE_RESREF.to_owned(),
            }],
        },
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if retarget.report.model_sha256 != M0_R39_MODEL_SHA256
        || retarget.report.rig_node_count != 25
        || retarget.report.skin_segment_count != 1
        || retarget.report.active_bone_count != 22
        || retarget.report.animation_clip_names != M0_R39_REQUIRED_CLIPS
        || retarget.animations.clips.iter().any(|clip| {
            clip.tracks
                .iter()
                .any(|track| track.path == crate::mdl::MdlAnimationTrackPathV1::Scale)
        })
    {
        return Err(candidate_error(
            "M2A-R39-RETARGET-CONTRACT",
            "retarget",
            frozen_retarget_drift_message(
                "r39",
                M0_R39_MODEL_SHA256,
                &retarget.report.model_sha256,
                "exact M0/H1 retarget differs from the admitted controllerless identity-root SkinMesh profile",
            ),
        ));
    }
    verify_direct_creature_state_projection_v1(
        &retarget.model.inspection,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
        None,
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let skin_readback = inspect_r39_skin_contract(&retarget.model.inspection)?;

    let ingest = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        candidate_error(
            error.code,
            error.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            error.message,
        )
    })?;
    let creature = retarget.conversion.creature.as_ref().ok_or_else(|| {
        candidate_error(
            "M2A-R39-RETARGET-CONTRACT",
            "retarget.conversion.creature",
            "successful retarget has no creature output",
        )
    })?;
    let texture_selection = resolve_base_color_image_index_v1(&ingest, creature)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let image = decode_embedded_image_to_tga_v1(
        source_glb,
        texture_selection.source_image_index,
        &GlbLimits::default(),
        &EmbeddedImageDecodeLimitsV1::default(),
    )
    .map_err(|error| {
        candidate_error(
            error.code,
            error
                .json_path
                .unwrap_or_else(|| "sourceGlb.images".to_owned()),
            error.message,
        )
    })?;
    let texture = write_tga_v1(&image, &TgaWriterOptionsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?
        .payload;
    if sha256_bytes(&texture) != M0_R39_TEXTURE_SHA256 {
        return Err(candidate_error(
            "M2A-R39-TEXTURE-IDENTITY",
            "texture",
            "r39 must preserve the exact M0 texture payload",
        ));
    }

    let appearance = append_r39_appearance(base_appearance_two_da)?;
    if appearance.report.appended_row_index != M0_R39_APPEARANCE_ROW {
        return Err(candidate_error(
            "M2A-R39-APPEARANCE-ROW",
            "appearance.appendedPhysicalRow",
            "r39 fixture must bind physical row 15100",
        ));
    }
    require_r39_appearance_cell(
        &appearance.payload,
        M0_R39_APPEARANCE_ROW,
        "LABEL",
        M0_R39_APPEARANCE_LABEL,
    )?;
    require_r39_appearance_cell(&appearance.payload, M0_R39_APPEARANCE_ROW, "MODELTYPE", "S")?;
    require_r39_appearance_cell(
        &appearance.payload,
        M0_R39_APPEARANCE_ROW,
        "RACE",
        M0_R39_MODEL_RESREF,
    )?;

    let resources = vec![
        HakResourceInputV1 {
            resref: M0_R39_MODEL_RESREF.to_owned(),
            resource_type: 2002,
            payload: retarget.model.payload.clone(),
        },
        HakResourceInputV1 {
            resref: M0_R39_TEXTURE_RESREF.to_owned(),
            resource_type: 3,
            payload: texture.clone(),
        },
        HakResourceInputV1 {
            resref: "appearance".to_owned(),
            resource_type: 2017,
            payload: appearance.payload.clone(),
        },
    ];
    let package = write_model_package_v1(&resources, &HakWriterOptionsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let archive = ErfArchive::parse(&package.hak.payload).map_err(|error| {
        candidate_error(error.code, format!("hak@{}", error.offset), error.context)
    })?;
    for (resref, resource_type, expected) in [
        (M0_R39_MODEL_RESREF, 2002, retarget.model.payload.as_slice()),
        (M0_R39_TEXTURE_RESREF, 3, texture.as_slice()),
        ("appearance", 2017, appearance.payload.as_slice()),
    ] {
        let actual = archive.find(resref, resource_type).map_err(|error| {
            candidate_error(error.code, format!("hak@{}", error.offset), error.context)
        })?;
        if actual != expected {
            return Err(candidate_error(
                "M2A-R39-HAK-READBACK",
                format!("hak.{resref}:{resource_type}"),
                "generated HAK resource differs from its exact input payload",
            ));
        }
    }

    let fixtures = [BinaryCreatureOwnedFixtureV1 {
        id: M0_R39_FIXTURE_ID.to_owned(),
        template_resref: M0_R39_FIXTURE_TEMPLATE_RESREF.to_owned(),
        display_name: M0_R39_FIXTURE_DISPLAY_NAME.to_owned(),
        appearance_row: M0_R39_APPEARANCE_ROW,
        position: M0RuntimePositionV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
        },
        orientation: M0RuntimeDirectionV1 { x: 1.0, y: 0.0 },
    }];
    let module = build_binary_creature_multi_fixture_module_v1(
        &BinaryCreatureModuleIdentityV1 {
            module_resref: M0_R39_MODULE_RESREF.to_owned(),
            area_resref: M0_R39_AREA_RESREF.to_owned(),
            hak_resref: M0_R39_HAK_RESREF.to_owned(),
        },
        &fixtures,
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let binary_scene = inspect_binary_creature_multi_fixture_module_v1(&module.payload)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if binary_scene != module.readback
        || binary_scene.ordered_hak_resrefs != [M0_R39_HAK_RESREF]
        || binary_scene.fixtures != fixtures
    {
        return Err(candidate_error(
            "M2A-R39-MODULE-READBACK",
            "module",
            "runtime-complete r39 MOD differs from its exact identity or fixture",
        ));
    }

    let contract = M0R39AnimatedDonorCandidateContractV1 {
        schema_version: 1,
        profile: "M0_R39_CONTROLLERLESS_IDENTITY_ROOT_SKINMESH_CANDIDATE_V1".to_owned(),
        candidate_admissible: true,
        admitted_by_r38_proof_sha256: M0_R39_ACCEPTED_R38_PROOF_SHA256.to_owned(),
        diagnosed_failure_boundary: "exact r38 remained visible in Toolset but absent in NWN; its dedicated identity model root is the only root in the 38-witness native creature corpus carrying redundant base position and orientation controllers".to_owned(),
        intended_functional_delta: vec![
            "preserve exact r38 geometry, SkinMesh topology, bind pose, inverse binds, weights, refs, UVs, indices, materials, animations, scale normalization and rig-only type-5 states".to_owned(),
            "omit only the two constant bind controllers and their 60-byte key/data payload from the single parentless, model-named, identity, unweighted base root".to_owned(),
        ],
        source: byte_identity(source_glb),
        donor: byte_identity(donor_glb),
        base_appearance: byte_identity(base_appearance_two_da),
        retarget: retarget.report,
        skin_readback,
        model: resource_binding(M0_R39_MODEL_RESREF, &retarget.model.payload),
        texture: resource_binding(M0_R39_TEXTURE_RESREF, &texture),
        appearance_two_da: resource_binding("appearance", &appearance.payload),
        hak: resource_binding(M0_R39_HAK_RESREF, &package.hak.payload),
        module: resource_binding(M0_R39_MODULE_RESREF, &module.payload),
        package_manifest: package.manifest,
        binary_scene,
        materialization_count: 1,
        toolset_model_visibility: "not_tested".to_owned(),
        toolset_proof_completeness: "missing".to_owned(),
        nwn_model_visibility: "not_tested".to_owned(),
        nwn_proof_completeness: "missing".to_owned(),
    };
    let mut contract_json = serde_json::to_vec_pretty(&contract).map_err(|error| {
        candidate_error(
            "M2A-R39-CONTRACT-SERIALIZATION",
            "contract",
            error.to_string(),
        )
    })?;
    contract_json.push(b'\n');

    Ok(M0R39AnimatedDonorCandidateArtifactV1 {
        model: retarget.model.payload,
        texture,
        appearance_two_da: appearance.payload,
        hak: package.hak.payload,
        module: module.payload,
        contract,
        contract_json,
    })
}

pub fn verify_m0_r39_animated_donor_candidate_v1(
    contract: &M0R39AnimatedDonorCandidateContractV1,
    source_glb: &[u8],
    donor_glb: &[u8],
    base_appearance_two_da: &[u8],
    module: &[u8],
    hak: &[u8],
) -> Result<(), M0R39AnimatedDonorCandidateErrorV1> {
    let replay =
        build_m0_r39_animated_donor_candidate_v1(source_glb, donor_glb, base_appearance_two_da)?;
    if contract != &replay.contract || module != replay.module || hak != replay.hak {
        return Err(candidate_error(
            "M2A-R39-REPLAY-MISMATCH",
            "candidate",
            "contract, MOD or HAK differs from the exact-input deterministic replay",
        ));
    }
    Ok(())
}

pub fn build_m0_r40_animated_donor_candidate_v1(
    source_glb: &[u8],
    donor_glb: &[u8],
    base_appearance_two_da: &[u8],
) -> Result<M0R40AnimatedDonorCandidateArtifactV1, M0R40AnimatedDonorCandidateErrorV1> {
    require_exact_r40_input(
        source_glb,
        M0_R33_SOURCE_BYTE_LENGTH,
        M0_R40_SOURCE_SHA256,
        "sourceGlb",
    )?;
    require_exact_r40_input(
        donor_glb,
        M0_R33_DONOR_BYTE_LENGTH,
        M0_R40_DONOR_SHA256,
        "donorGlb",
    )?;
    require_exact_r40_input(
        base_appearance_two_da,
        M0_R33_BASE_APPEARANCE_BYTE_LENGTH,
        M0_R40_BASE_APPEARANCE_SHA256,
        "baseAppearanceTwoDa",
    )?;
    let appearance_inspection =
        inspect_two_da_v2(base_appearance_two_da, &TwoDaLimitsV1::default())
            .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if appearance_inspection.physical_row_count != M0_R33_BASE_APPEARANCE_ROWS {
        return Err(candidate_error(
            "M2A-R40-APPEARANCE-TRUST-ROOT",
            "baseAppearanceTwoDa",
            "r40 requires the exact 15100-row full runtime appearance table",
        ));
    }

    let retarget = retarget_static_mesh_to_animated_donor_v5(
        source_glb,
        donor_glb,
        &MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile:
                MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedControllerlessRootV3,
            state_projection_profile:
                MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
            state_projection_provenance: None,
            model_resource_resref: M0_R40_MODEL_RESREF.to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: M0_R40_TEXTURE_RESREF.to_owned(),
            }],
        },
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if retarget.report.model_sha256 != M0_R40_MODEL_SHA256
        || retarget.report.rig_node_count != 25
        || retarget.report.skin_segment_count != 0
        || retarget.report.active_bone_count != 20
        || retarget.report.animation_clip_names != M0_R40_REQUIRED_CLIPS
        || retarget.animations.clips.iter().any(|clip| {
            clip.tracks
                .iter()
                .any(|track| track.path == crate::mdl::MdlAnimationTrackPathV1::Scale)
        })
    {
        return Err(candidate_error(
            "M2A-R40-RETARGET-CONTRACT",
            "retarget",
            frozen_retarget_drift_message(
                "r40",
                M0_R40_MODEL_SHA256,
                &retarget.report.model_sha256,
                "exact M0/H1 retarget differs from the admitted rigid triangle-group profile",
            ),
        ));
    }
    verify_direct_creature_state_projection_v1(
        &retarget.model.inspection,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
        None,
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let rigid_readback = inspect_r40_rigid_contract(&retarget.model.inspection)?;

    let ingest = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        candidate_error(
            error.code,
            error.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            error.message,
        )
    })?;
    let creature = retarget.conversion.creature.as_ref().ok_or_else(|| {
        candidate_error(
            "M2A-R40-RETARGET-CONTRACT",
            "retarget.conversion.creature",
            "successful retarget has no creature output",
        )
    })?;
    let texture_selection = resolve_base_color_image_index_v1(&ingest, creature)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let image = decode_embedded_image_to_tga_v1(
        source_glb,
        texture_selection.source_image_index,
        &GlbLimits::default(),
        &EmbeddedImageDecodeLimitsV1::default(),
    )
    .map_err(|error| {
        candidate_error(
            error.code,
            error
                .json_path
                .unwrap_or_else(|| "sourceGlb.images".to_owned()),
            error.message,
        )
    })?;
    let texture = write_tga_v1(&image, &TgaWriterOptionsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?
        .payload;
    if sha256_bytes(&texture) != M0_R40_TEXTURE_SHA256 {
        return Err(candidate_error(
            "M2A-R40-TEXTURE-IDENTITY",
            "texture",
            "r40 must preserve the exact M0 texture payload",
        ));
    }

    let appearance = append_r40_appearance(base_appearance_two_da)?;
    if appearance.report.appended_row_index != M0_R40_APPEARANCE_ROW {
        return Err(candidate_error(
            "M2A-R40-APPEARANCE-ROW",
            "appearance.appendedPhysicalRow",
            "r40 fixture must bind physical row 15100",
        ));
    }
    require_r40_appearance_cell(
        &appearance.payload,
        M0_R40_APPEARANCE_ROW,
        "LABEL",
        M0_R40_APPEARANCE_LABEL,
    )?;
    require_r40_appearance_cell(&appearance.payload, M0_R40_APPEARANCE_ROW, "MODELTYPE", "S")?;
    require_r40_appearance_cell(
        &appearance.payload,
        M0_R40_APPEARANCE_ROW,
        "RACE",
        M0_R40_MODEL_RESREF,
    )?;

    let resources = vec![
        HakResourceInputV1 {
            resref: M0_R40_MODEL_RESREF.to_owned(),
            resource_type: 2002,
            payload: retarget.model.payload.clone(),
        },
        HakResourceInputV1 {
            resref: M0_R40_TEXTURE_RESREF.to_owned(),
            resource_type: 3,
            payload: texture.clone(),
        },
        HakResourceInputV1 {
            resref: "appearance".to_owned(),
            resource_type: 2017,
            payload: appearance.payload.clone(),
        },
    ];
    let package = write_model_package_v1(&resources, &HakWriterOptionsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let archive = ErfArchive::parse(&package.hak.payload).map_err(|error| {
        candidate_error(error.code, format!("hak@{}", error.offset), error.context)
    })?;
    for (resref, resource_type, expected) in [
        (M0_R40_MODEL_RESREF, 2002, retarget.model.payload.as_slice()),
        (M0_R40_TEXTURE_RESREF, 3, texture.as_slice()),
        ("appearance", 2017, appearance.payload.as_slice()),
    ] {
        let actual = archive.find(resref, resource_type).map_err(|error| {
            candidate_error(error.code, format!("hak@{}", error.offset), error.context)
        })?;
        if actual != expected {
            return Err(candidate_error(
                "M2A-R40-HAK-READBACK",
                format!("hak.{resref}:{resource_type}"),
                "generated HAK resource differs from its exact input payload",
            ));
        }
    }

    let fixtures = [BinaryCreatureOwnedFixtureV1 {
        id: M0_R40_FIXTURE_ID.to_owned(),
        template_resref: M0_R40_FIXTURE_TEMPLATE_RESREF.to_owned(),
        display_name: M0_R40_FIXTURE_DISPLAY_NAME.to_owned(),
        appearance_row: M0_R40_APPEARANCE_ROW,
        position: M0RuntimePositionV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
        },
        orientation: M0RuntimeDirectionV1 { x: 1.0, y: 0.0 },
    }];
    let module = build_binary_creature_multi_fixture_module_v1(
        &BinaryCreatureModuleIdentityV1 {
            module_resref: M0_R40_MODULE_RESREF.to_owned(),
            area_resref: M0_R40_AREA_RESREF.to_owned(),
            hak_resref: M0_R40_HAK_RESREF.to_owned(),
        },
        &fixtures,
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let binary_scene = inspect_binary_creature_multi_fixture_module_v1(&module.payload)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if binary_scene != module.readback
        || binary_scene.ordered_hak_resrefs != [M0_R40_HAK_RESREF]
        || binary_scene.fixtures != fixtures
    {
        return Err(candidate_error(
            "M2A-R40-MODULE-READBACK",
            "module",
            "runtime-complete r40 MOD differs from its exact identity or fixture",
        ));
    }

    let contract = M0R40AnimatedDonorCandidateContractV1 {
        schema_version: 1,
        profile: "M0_R40_BONE_LOCAL_RIGID_TRIANGLE_GROUP_CANDIDATE_V1".to_owned(),
        candidate_admissible: true,
        admitted_by_r39_proof_sha256: M0_R40_ACCEPTED_R39_PROOF_SHA256.to_owned(),
        diagnosed_failure_boundary: "r33-r39 remained on the runtime-unproven 0x61 SkinMesh writer route, while the project-owned writer-produced runtime-visible witness uses rigid 0x21 TriMesh geometry; native creatures support rigid mesh pieces parented to animated nodes".to_owned(),
        intended_functional_delta: vec![
            "replace the single SkinMesh with deterministic per-bone rigid triangle groups while preserving every triangle, UV and bind-pose world-space position".to_owned(),
            "assign each complete triangle to the donor bone with the largest aggregate normalized weight, using stable rig-tree order for ties".to_owned(),
            "emit 20 bone-local 0x21 TriMesh nodes, zero SkinMesh palettes or weight streams, and preserve the exact 25-node rig, seven scale-free animations and rig-only type-5 states".to_owned(),
        ],
        source: byte_identity(source_glb),
        donor: byte_identity(donor_glb),
        base_appearance: byte_identity(base_appearance_two_da),
        retarget: retarget.report,
        rigid_readback,
        model: resource_binding(M0_R40_MODEL_RESREF, &retarget.model.payload),
        texture: resource_binding(M0_R40_TEXTURE_RESREF, &texture),
        appearance_two_da: resource_binding("appearance", &appearance.payload),
        hak: resource_binding(M0_R40_HAK_RESREF, &package.hak.payload),
        module: resource_binding(M0_R40_MODULE_RESREF, &module.payload),
        package_manifest: package.manifest,
        binary_scene,
        materialization_count: 1,
        toolset_model_visibility: "not_tested".to_owned(),
        toolset_proof_completeness: "missing".to_owned(),
        nwn_model_visibility: "not_tested".to_owned(),
        nwn_proof_completeness: "missing".to_owned(),
    };
    let mut contract_json = serde_json::to_vec_pretty(&contract).map_err(|error| {
        candidate_error(
            "M2A-R40-CONTRACT-SERIALIZATION",
            "contract",
            error.to_string(),
        )
    })?;
    contract_json.push(b'\n');

    Ok(M0R40AnimatedDonorCandidateArtifactV1 {
        model: retarget.model.payload,
        texture,
        appearance_two_da: appearance.payload,
        hak: package.hak.payload,
        module: module.payload,
        contract,
        contract_json,
    })
}

pub fn verify_m0_r40_animated_donor_candidate_v1(
    contract: &M0R40AnimatedDonorCandidateContractV1,
    source_glb: &[u8],
    donor_glb: &[u8],
    base_appearance_two_da: &[u8],
    module: &[u8],
    hak: &[u8],
) -> Result<(), M0R40AnimatedDonorCandidateErrorV1> {
    let replay =
        build_m0_r40_animated_donor_candidate_v1(source_glb, donor_glb, base_appearance_two_da)?;
    if contract != &replay.contract || module != replay.module || hak != replay.hak {
        return Err(candidate_error(
            "M2A-R40-REPLAY-MISMATCH",
            "candidate",
            "contract, MOD or HAK differs from the exact-input deterministic replay",
        ));
    }
    Ok(())
}

fn inspect_r40_rigid_contract(
    inspection: &crate::mdl::InspectionReport,
) -> Result<M0R40RigidReadbackV1, M0R40AnimatedDonorCandidateErrorV1> {
    fn collect_nodes<'a>(nodes: &'a [NodeReport], output: &mut Vec<&'a NodeReport>) {
        for node in nodes {
            output.push(node);
            collect_nodes(&node.children, output);
        }
    }

    let [aurora_root] = inspection.node_tree.roots.as_slice() else {
        return Err(candidate_error(
            "M2A-R40-MDL-READBACK",
            "model.nodeTree.roots",
            "r40 requires exactly one dedicated Aurora Root",
        ));
    };
    let skeleton_root = aurora_root
        .children
        .iter()
        .find(|node| node.name == "Hips")
        .ok_or_else(|| {
            candidate_error(
                "M2A-R40-MDL-READBACK",
                "model.nodeTree.roots[0].children",
                "r40 must preserve Hips immediately below the dedicated Aurora Root",
            )
        })?;
    let mut nodes = Vec::new();
    collect_nodes(&inspection.node_tree.roots, &mut nodes);
    let mesh_nodes = nodes
        .iter()
        .copied()
        .filter(|node| node.mesh.is_some())
        .collect::<Vec<_>>();
    let skin_node_count = nodes.iter().filter(|node| node.skin.is_some()).count();
    let parent_offsets = mesh_nodes
        .iter()
        .filter_map(|node| node.parent_offset)
        .collect::<std::collections::BTreeSet<_>>();
    let triangle_count = mesh_nodes
        .iter()
        .map(|node| node.mesh.as_ref().expect("filtered mesh").faces.len())
        .sum::<usize>();
    let duplicated_vertex_count = mesh_nodes
        .iter()
        .map(|node| node.mesh.as_ref().expect("filtered mesh").vertex_count)
        .sum::<usize>();
    let animation_names = inspection
        .animations
        .iter()
        .map(|animation| animation.name.clone())
        .collect::<Vec<_>>();
    let animation_node_counts = inspection
        .animations
        .iter()
        .map(|animation| animation.node_tree.node_count)
        .collect::<Vec<_>>();
    let animation_scale_controller_count = inspection
        .animations
        .iter()
        .map(|animation| {
            let mut count = 0;
            let mut pending = animation.node_tree.roots.iter().collect::<Vec<_>>();
            while let Some(node) = pending.pop() {
                count += node
                    .controllers
                    .iter()
                    .filter(|controller| controller.controller_type == 36)
                    .count();
                pending.extend(&node.children);
            }
            count
        })
        .sum::<usize>();
    let rigid_mesh_contract_ok = mesh_nodes.iter().all(|node| {
        let mesh = node.mesh.as_ref().expect("filtered mesh");
        node.content_flags == 0x21
            && node.skin.is_none()
            && node.parent_offset.is_some()
            && node.parent_offset != Some(aurora_root.offset)
            && mesh.mesh_type == 3
            && mesh.textures.first().map(String::as_str) == Some(M0_R40_TEXTURE_RESREF)
            && mesh.vertex_count == mesh.vertices.len()
            && mesh.vertex_count == mesh.uv0.len()
            && mesh.vertex_count == mesh.normals.len()
            && mesh.vertex_count == mesh.faces.len() * 3
    });
    let parent_contract_ok = parent_offsets.iter().all(|parent_offset| {
        nodes
            .iter()
            .any(|node| node.offset == *parent_offset && node.mesh.is_none())
    });
    if inspection.model.name != M0_R40_MODEL_RESREF
        || aurora_root.name != M0_R40_MODEL_RESREF
        || aurora_root.parent_offset.is_some()
        || !aurora_root.controllers.is_empty()
        || aurora_root.controller_keys_header.pointer != 0
        || aurora_root.controller_keys_header.used != 0
        || aurora_root.controller_keys_header.allocated != 0
        || aurora_root.controller_data_header.pointer != 0
        || aurora_root.controller_data_header.used != 0
        || aurora_root.controller_data_header.allocated != 0
        || skeleton_root.parent_offset != Some(aurora_root.offset)
        || inspection.node_tree.node_count != 45
        || mesh_nodes.len() != 20
        || skin_node_count != 0
        || triangle_count != 1_569
        || duplicated_vertex_count != 4_707
        || parent_offsets.len() != 20
        || !rigid_mesh_contract_ok
        || !parent_contract_ok
        || inspection.animations.len() != M0_R40_REQUIRED_CLIPS.len()
        || animation_names != M0_R40_REQUIRED_CLIPS
        || animation_node_counts.iter().any(|count| *count != 25)
        || animation_scale_controller_count != 0
        || inspection
            .animations
            .iter()
            .any(|animation| !animation.node_tree.roots[0].controllers.is_empty())
    {
        return Err(candidate_error(
            "M2A-R40-MDL-READBACK",
            "model",
            "r40 requires a controllerless identity root, 25 rig nodes, 20 bone-local rigid TriMeshes, zero SkinMeshes, the exact source surface and seven scale-free rig-only animations",
        ));
    }

    Ok(M0R40RigidReadbackV1 {
        model_name: inspection.model.name.clone(),
        base_node_count: inspection.node_tree.node_count,
        rig_node_count: 25,
        rigid_mesh_node_count: mesh_nodes.len(),
        skin_node_count,
        triangle_count,
        duplicated_vertex_count,
        active_parent_bone_count: parent_offsets.len(),
        aurora_root_name: aurora_root.name.clone(),
        skeleton_root_name: skeleton_root.name.clone(),
        base_root_controller_count: aurora_root.controllers.len(),
        animation_names,
        animation_node_counts,
        animation_scale_controller_count,
        format_profile:
            MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedControllerlessRootV3,
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
    })
}

pub fn build_m0_r41_animated_donor_candidate_v1(
    source_glb: &[u8],
    donor_glb: &[u8],
    base_appearance_two_da: &[u8],
) -> Result<M0R41AnimatedDonorCandidateArtifactV1, M0R41AnimatedDonorCandidateErrorV1> {
    require_exact_r41_input(
        source_glb,
        M0_R33_SOURCE_BYTE_LENGTH,
        M0_R41_SOURCE_SHA256,
        "sourceGlb",
    )?;
    require_exact_r41_input(
        donor_glb,
        M0_R33_DONOR_BYTE_LENGTH,
        M0_R41_DONOR_SHA256,
        "donorGlb",
    )?;
    require_exact_r41_input(
        base_appearance_two_da,
        M0_R33_BASE_APPEARANCE_BYTE_LENGTH,
        M0_R41_BASE_APPEARANCE_SHA256,
        "baseAppearanceTwoDa",
    )?;
    let appearance_inspection =
        inspect_two_da_v2(base_appearance_two_da, &TwoDaLimitsV1::default())
            .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if appearance_inspection.physical_row_count != M0_R33_BASE_APPEARANCE_ROWS {
        return Err(candidate_error(
            "M2A-R41-APPEARANCE-TRUST-ROOT",
            "baseAppearanceTwoDa",
            "r41 requires the exact 15100-row full runtime appearance table",
        ));
    }

    let retarget = retarget_static_mesh_to_animated_donor_v5(
        source_glb,
        donor_glb,
        &MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile:
                MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedControllerlessRootV3,
            state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
            state_projection_provenance: None,
            model_resource_resref: M0_R41_MODEL_RESREF.to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot: 0,
                resref: M0_R41_TEXTURE_RESREF.to_owned(),
            }],
        },
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if retarget.report.model_sha256 != M0_R41_MODEL_SHA256
        || retarget.report.rig_node_count != 25
        || retarget.report.skin_segment_count != 0
        || retarget.report.active_bone_count != 20
        || retarget.report.animation_clip_names != M0_R41_REQUIRED_CLIPS
        || retarget.animations.clips.iter().any(|clip| {
            clip.tracks
                .iter()
                .any(|track| track.path == crate::mdl::MdlAnimationTrackPathV1::Scale)
        })
    {
        return Err(candidate_error(
            "M2A-R41-RETARGET-CONTRACT",
            "retarget",
            frozen_retarget_drift_message(
                "r41",
                M0_R41_MODEL_SHA256,
                &retarget.report.model_sha256,
                "exact M0/H1 retarget differs from the admitted full-topology rigid state profile",
            ),
        ));
    }
    verify_direct_creature_state_projection_v1(
        &retarget.model.inspection,
        MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
        None,
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let rigid_readback = inspect_r41_rigid_contract(&retarget.model.inspection)?;

    let ingest = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        candidate_error(
            error.code,
            error.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            error.message,
        )
    })?;
    let creature = retarget.conversion.creature.as_ref().ok_or_else(|| {
        candidate_error(
            "M2A-R41-RETARGET-CONTRACT",
            "retarget.conversion.creature",
            "successful retarget has no creature output",
        )
    })?;
    let texture_selection = resolve_base_color_image_index_v1(&ingest, creature)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let image = decode_embedded_image_to_tga_v1(
        source_glb,
        texture_selection.source_image_index,
        &GlbLimits::default(),
        &EmbeddedImageDecodeLimitsV1::default(),
    )
    .map_err(|error| {
        candidate_error(
            error.code,
            error
                .json_path
                .unwrap_or_else(|| "sourceGlb.images".to_owned()),
            error.message,
        )
    })?;
    let texture = write_tga_v1(&image, &TgaWriterOptionsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?
        .payload;
    if sha256_bytes(&texture) != M0_R41_TEXTURE_SHA256 {
        return Err(candidate_error(
            "M2A-R41-TEXTURE-IDENTITY",
            "texture",
            "r41 must preserve the exact M0 texture payload",
        ));
    }

    let appearance = append_r41_appearance(base_appearance_two_da)?;
    if appearance.report.appended_row_index != M0_R41_APPEARANCE_ROW {
        return Err(candidate_error(
            "M2A-R41-APPEARANCE-ROW",
            "appearance.appendedPhysicalRow",
            "r41 fixture must bind physical row 15100",
        ));
    }
    require_r41_appearance_cell(
        &appearance.payload,
        M0_R41_APPEARANCE_ROW,
        "LABEL",
        M0_R41_APPEARANCE_LABEL,
    )?;
    require_r41_appearance_cell(&appearance.payload, M0_R41_APPEARANCE_ROW, "MODELTYPE", "S")?;
    require_r41_appearance_cell(
        &appearance.payload,
        M0_R41_APPEARANCE_ROW,
        "RACE",
        M0_R41_MODEL_RESREF,
    )?;

    let resources = vec![
        HakResourceInputV1 {
            resref: M0_R41_MODEL_RESREF.to_owned(),
            resource_type: 2002,
            payload: retarget.model.payload.clone(),
        },
        HakResourceInputV1 {
            resref: M0_R41_TEXTURE_RESREF.to_owned(),
            resource_type: 3,
            payload: texture.clone(),
        },
        HakResourceInputV1 {
            resref: "appearance".to_owned(),
            resource_type: 2017,
            payload: appearance.payload.clone(),
        },
    ];
    let package = write_model_package_v1(&resources, &HakWriterOptionsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let archive = ErfArchive::parse(&package.hak.payload).map_err(|error| {
        candidate_error(error.code, format!("hak@{}", error.offset), error.context)
    })?;
    for (resref, resource_type, expected) in [
        (M0_R41_MODEL_RESREF, 2002, retarget.model.payload.as_slice()),
        (M0_R41_TEXTURE_RESREF, 3, texture.as_slice()),
        ("appearance", 2017, appearance.payload.as_slice()),
    ] {
        let actual = archive.find(resref, resource_type).map_err(|error| {
            candidate_error(error.code, format!("hak@{}", error.offset), error.context)
        })?;
        if actual != expected {
            return Err(candidate_error(
                "M2A-R41-HAK-READBACK",
                format!("hak.{resref}:{resource_type}"),
                "generated HAK resource differs from its exact input payload",
            ));
        }
    }

    let fixtures = [BinaryCreatureOwnedFixtureV1 {
        id: M0_R41_FIXTURE_ID.to_owned(),
        template_resref: M0_R41_FIXTURE_TEMPLATE_RESREF.to_owned(),
        display_name: M0_R41_FIXTURE_DISPLAY_NAME.to_owned(),
        appearance_row: M0_R41_APPEARANCE_ROW,
        position: M0RuntimePositionV1 {
            x: 10.0,
            y: 14.5,
            z: 0.0,
        },
        orientation: M0RuntimeDirectionV1 { x: 1.0, y: 0.0 },
    }];
    let module = build_binary_creature_multi_fixture_module_v1(
        &BinaryCreatureModuleIdentityV1 {
            module_resref: M0_R41_MODULE_RESREF.to_owned(),
            area_resref: M0_R41_AREA_RESREF.to_owned(),
            hak_resref: M0_R41_HAK_RESREF.to_owned(),
        },
        &fixtures,
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let binary_scene = inspect_binary_creature_multi_fixture_module_v1(&module.payload)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if binary_scene != module.readback
        || binary_scene.ordered_hak_resrefs != [M0_R41_HAK_RESREF]
        || binary_scene.fixtures != fixtures
    {
        return Err(candidate_error(
            "M2A-R41-MODULE-READBACK",
            "module",
            "runtime-complete r41 MOD differs from its exact identity or fixture",
        ));
    }

    let contract = M0R41AnimatedDonorCandidateContractV1 {
        schema_version: 1,
        profile: "M0_R41_RETAIL_FULL_TOPOLOGY_RIGID_STATE_CANDIDATE_V1".to_owned(),
        candidate_admissible: true,
        admitted_by_r40_proof_sha256: M0_R41_ACCEPTED_R40_PROOF_SHA256.to_owned(),
        diagnosed_failure_boundary: "the exact r40 base tree contains 25 rig nodes and 20 renderable rigid mesh identities, but every type-5 local-animation state projects only the 25 rig identities; the owner verified that the same lineage is visible in Toolset and absent in NWN, while audited retail direct creatures preserve their complete base name/part/parent topology as generic state dummies".to_owned(),
        intended_functional_delta: vec![
            "preserve the exact r40 source geometry, 20 bone-local rigid TriMeshes, bind-pose transforms, texture, material, 25-node rig and seven scale-free clips".to_owned(),
            "replace only the r40 rig-only type-5 state projection with the audited retail full-topology family".to_owned(),
            "project all 45 base identities into every type-5 state, including 20 header-only 0x01 mesh-identity dummies with matching name, part number and parent and with no geometry, skin or controller payload".to_owned(),
        ],
        source: byte_identity(source_glb),
        donor: byte_identity(donor_glb),
        base_appearance: byte_identity(base_appearance_two_da),
        retarget: retarget.report,
        rigid_readback,
        model: resource_binding(M0_R41_MODEL_RESREF, &retarget.model.payload),
        texture: resource_binding(M0_R41_TEXTURE_RESREF, &texture),
        appearance_two_da: resource_binding("appearance", &appearance.payload),
        hak: resource_binding(M0_R41_HAK_RESREF, &package.hak.payload),
        module: resource_binding(M0_R41_MODULE_RESREF, &module.payload),
        package_manifest: package.manifest,
        binary_scene,
        materialization_count: 1,
        toolset_model_visibility: "not_tested".to_owned(),
        toolset_proof_completeness: "missing".to_owned(),
        nwn_model_visibility: "not_tested".to_owned(),
        nwn_proof_completeness: "missing".to_owned(),
    };
    let mut contract_json = serde_json::to_vec_pretty(&contract).map_err(|error| {
        candidate_error(
            "M2A-R41-CONTRACT-SERIALIZATION",
            "contract",
            error.to_string(),
        )
    })?;
    contract_json.push(b'\n');

    Ok(M0R41AnimatedDonorCandidateArtifactV1 {
        model: retarget.model.payload,
        texture,
        appearance_two_da: appearance.payload,
        hak: package.hak.payload,
        module: module.payload,
        contract,
        contract_json,
    })
}

pub fn verify_m0_r41_animated_donor_candidate_v1(
    contract: &M0R41AnimatedDonorCandidateContractV1,
    source_glb: &[u8],
    donor_glb: &[u8],
    base_appearance_two_da: &[u8],
    module: &[u8],
    hak: &[u8],
) -> Result<(), M0R41AnimatedDonorCandidateErrorV1> {
    let replay =
        build_m0_r41_animated_donor_candidate_v1(source_glb, donor_glb, base_appearance_two_da)?;
    if contract != &replay.contract || module != replay.module || hak != replay.hak {
        return Err(candidate_error(
            "M2A-R41-REPLAY-MISMATCH",
            "candidate",
            "contract, MOD or HAK differs from the exact-input deterministic replay",
        ));
    }
    Ok(())
}

pub fn build_h2_r42_visibility_candidate_v1(
    source_glb: &[u8],
    base_appearance_two_da: &[u8],
) -> Result<H2R42VisibilityCandidateArtifactV1, H2R42VisibilityCandidateErrorV1> {
    require_exact_h2_r42_input(
        source_glb,
        H2_R42_SOURCE_BYTE_LENGTH,
        H2_R42_SOURCE_SHA256,
        "sourceGlb",
    )?;
    require_exact_h2_r42_input(
        base_appearance_two_da,
        M0_R33_BASE_APPEARANCE_BYTE_LENGTH,
        H2_R42_BASE_APPEARANCE_SHA256,
        "baseAppearanceTwoDa",
    )?;
    let source = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        candidate_error(
            error.code,
            error.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            error.message,
        )
    })?;
    let appearance_inspection =
        inspect_two_da_v2(base_appearance_two_da, &TwoDaLimitsV1::default())
            .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if appearance_inspection.physical_row_count != M0_R33_BASE_APPEARANCE_ROWS {
        return Err(candidate_error(
            "M2A-H2-R42-APPEARANCE-TRUST-ROOT",
            "baseAppearanceTwoDa",
            "r42 requires the exact full 15100-row runtime appearance table",
        ));
    }

    let (rig, mapping) = derive_meshy_h1_profile_and_mapping_v1(&source)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let mut conversion = convert_profile_a_with_animations_v1(
        &source,
        &rig,
        &ProfileAOptionsV1::default(),
        &mapping,
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if !conversion.base.report.conversion_eligible {
        return Err(candidate_error(
            "M2A-H2-R42-CONVERSION-INELIGIBLE",
            "conversion.report",
            "the exact generated H2 source did not produce an eligible creature conversion",
        ));
    }
    let source_animations = conversion.animations.as_ref().ok_or_else(|| {
        candidate_error(
            "M2A-H2-R42-ANIMATION-MISSING",
            "conversion.animations",
            "the rigged H2 source must provide its own mapped Idle animation",
        )
    })?;
    let mut animations = materialize_direct_creature_runtime_clips(source_animations)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let mut creature = conversion.base.creature.take().ok_or_else(|| {
        candidate_error(
            "M2A-H2-R42-CONVERSION-INELIGIBLE",
            "conversion.creature",
            "eligible H2 conversion did not produce creature IR",
        )
    })?;
    let root_indexes = creature
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| node.parent_id.is_none().then_some(index))
        .collect::<Vec<_>>();
    let [skeleton_root_index] = root_indexes.as_slice() else {
        return Err(candidate_error(
            "M2A-H2-R42-RIG-ROOT",
            "conversion.creature.nodes",
            "H2 must produce exactly one source skeleton root",
        ));
    };
    if creature.segments.len() != 1
        || creature.segments[0].deformation != RigSegmentDeformationV1::Skin
    {
        return Err(candidate_error(
            "M2A-H2-R42-SOURCE-SURFACE",
            "conversion.creature.segments",
            "H2 r42 requires exactly one weighted source surface before root-rigid isolation",
        ));
    }
    let skeleton_root_id = creature.nodes[*skeleton_root_index].id;
    let skeleton_root_bind = creature.nodes[*skeleton_root_index].bind_local_matrix;
    let aurora_root_id = creature
        .nodes
        .iter()
        .map(|node| node.id)
        .max()
        .unwrap_or_default()
        .checked_add(1)
        .ok_or_else(|| {
            candidate_error(
                "M2A-H2-R42-RIG-ROOT",
                "conversion.creature.nodes",
                "cannot allocate a dedicated H2 Aurora Root id",
            )
        })?;
    if creature
        .nodes
        .iter()
        .any(|node| node.name.eq_ignore_ascii_case(H2_R42_MODEL_RESREF))
    {
        return Err(candidate_error(
            "M2A-H2-R42-RIG-ROOT",
            "conversion.creature.nodes",
            "dedicated H2 Aurora Root name collides with a source joint",
        ));
    }
    creature.nodes[*skeleton_root_index].parent_id = Some(aurora_root_id);
    creature.nodes.insert(
        0,
        AuroraCreatureNodeV1 {
            id: aurora_root_id,
            name: H2_R42_MODEL_RESREF.to_owned(),
            parent_id: None,
            bind_local_matrix: [
                1.0, 0.0, 0.0, 0.0, //
                0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 1.0, 0.0, //
                0.0, 0.0, 0.0, 1.0,
            ],
        },
    );
    let segment = &mut creature.segments[0];
    if segment.parent_node_id != skeleton_root_id {
        return Err(candidate_error(
            "M2A-H2-R42-SOURCE-SURFACE",
            "conversion.creature.segments[0].parentNodeId",
            "the H2 surface must be attached to the single source skeleton root",
        ));
    }
    for position in &mut segment.positions {
        *position = h2_r42_transform_point(skeleton_root_bind, *position).ok_or_else(|| {
            candidate_error(
                "M2A-H2-R42-ROOT-REPARENT",
                "conversion.creature.segments[0].positions",
                "baking the old skeleton-root transform produced a non-finite position",
            )
        })?;
    }
    for normal in &mut segment.normals {
        *normal = h2_r42_transform_direction(skeleton_root_bind, *normal).ok_or_else(|| {
            candidate_error(
                "M2A-H2-R42-ROOT-REPARENT",
                "conversion.creature.segments[0].normals",
                "baking the old skeleton-root transform produced an invalid normal",
            )
        })?;
    }
    if let Some(tangents) = &mut segment.tangents {
        for tangent in tangents {
            let direction = h2_r42_transform_direction(
                skeleton_root_bind,
                [tangent[0], tangent[1], tangent[2]],
            )
            .ok_or_else(|| {
                candidate_error(
                    "M2A-H2-R42-ROOT-REPARENT",
                    "conversion.creature.segments[0].tangents",
                    "baking the old skeleton-root transform produced an invalid tangent",
                )
            })?;
            *tangent = [direction[0], direction[1], direction[2], tangent[3]];
        }
    }
    segment.parent_node_id = aurora_root_id;
    segment.deformation = RigSegmentDeformationV1::Rigid;
    segment.weights.clear();
    for clip in &mut animations.clips {
        clip.animation_root = H2_R42_MODEL_RESREF.to_owned();
    }

    let texture_selection = resolve_base_color_image_index_v1(&source, &creature)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let material_slot = creature.segments[0].material_slot;
    let model = write_binary_mdl_with_animations(
        &creature,
        &animations,
        &MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile:
                MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedControllerlessRootV3,
            state_projection_profile:
                MdlStateProjectionProfileV1::OwnedRuntimePositiveType0RigOnlyV1,
            state_projection_provenance: None,
            model_resource_resref: H2_R42_MODEL_RESREF.to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot,
                resref: H2_R42_TEXTURE_RESREF.to_owned(),
            }],
        },
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if sha256_bytes(&model.payload) != H2_R42_MODEL_SHA256 {
        return Err(candidate_error(
            "M2A-H2-R42-MODEL-IDENTITY",
            "model",
            "H2 r42 model bytes differ from the frozen candidate identity",
        ));
    }
    verify_direct_creature_state_projection_v1(
        &model.inspection,
        MdlStateProjectionProfileV1::OwnedRuntimePositiveType0RigOnlyV1,
        None,
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;

    let image = decode_embedded_image_to_tga_v1(
        source_glb,
        texture_selection.source_image_index,
        &GlbLimits::default(),
        &EmbeddedImageDecodeLimitsV1::default(),
    )
    .map_err(|error| {
        candidate_error(
            error.code,
            error
                .json_path
                .unwrap_or_else(|| "sourceGlb.images".to_owned()),
            error.message,
        )
    })?;
    let texture = write_tga_v1(&image, &TgaWriterOptionsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?
        .payload;
    if sha256_bytes(&texture) != H2_R42_TEXTURE_SHA256 {
        return Err(candidate_error(
            "M2A-H2-R42-TEXTURE-IDENTITY",
            "texture",
            "H2 r42 texture bytes differ from the frozen candidate identity",
        ));
    }
    let appearance = append_h2_r42_appearance(base_appearance_two_da)?;
    if sha256_bytes(&appearance.payload) != H2_R42_APPEARANCE_SHA256 {
        return Err(candidate_error(
            "M2A-H2-R42-APPEARANCE-IDENTITY",
            "appearance",
            "H2 r42 appearance bytes differ from the frozen candidate identity",
        ));
    }
    require_h2_r42_appearance_cell(
        &appearance.payload,
        H2_R42_APPEARANCE_ROW,
        "LABEL",
        H2_R42_APPEARANCE_LABEL,
    )?;
    require_h2_r42_appearance_cell(&appearance.payload, H2_R42_APPEARANCE_ROW, "MODELTYPE", "S")?;
    require_h2_r42_appearance_cell(
        &appearance.payload,
        H2_R42_APPEARANCE_ROW,
        "RACE",
        H2_R42_MODEL_RESREF,
    )?;

    let resources = vec![
        HakResourceInputV1 {
            resref: H2_R42_MODEL_RESREF.to_owned(),
            resource_type: 2002,
            payload: model.payload.clone(),
        },
        HakResourceInputV1 {
            resref: H2_R42_TEXTURE_RESREF.to_owned(),
            resource_type: 3,
            payload: texture.clone(),
        },
        HakResourceInputV1 {
            resref: "appearance".to_owned(),
            resource_type: 2017,
            payload: appearance.payload.clone(),
        },
    ];
    let package = write_model_package_v1(&resources, &HakWriterOptionsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if sha256_bytes(&package.hak.payload) != H2_R42_HAK_SHA256 {
        return Err(candidate_error(
            "M2A-H2-R42-HAK-IDENTITY",
            "hak",
            "H2 r42 HAK bytes differ from the frozen candidate identity",
        ));
    }
    let archive = ErfArchive::parse(&package.hak.payload).map_err(|error| {
        candidate_error(error.code, format!("hak@{}", error.offset), error.context)
    })?;
    for (resref, resource_type, expected) in [
        (H2_R42_MODEL_RESREF, 2002, model.payload.as_slice()),
        (H2_R42_TEXTURE_RESREF, 3, texture.as_slice()),
        ("appearance", 2017, appearance.payload.as_slice()),
    ] {
        let actual = archive.find(resref, resource_type).map_err(|error| {
            candidate_error(error.code, format!("hak@{}", error.offset), error.context)
        })?;
        if actual != expected {
            return Err(candidate_error(
                "M2A-H2-R42-HAK-READBACK",
                format!("hak.{resref}:{resource_type}"),
                "generated H2 HAK resource differs from its exact input payload",
            ));
        }
    }

    let fixtures = [
        BinaryCreatureOwnedFixtureV1 {
            id: H2_R42_FIXTURE_ID.to_owned(),
            template_resref: H2_R42_FIXTURE_TEMPLATE_RESREF.to_owned(),
            display_name: H2_R42_FIXTURE_DISPLAY_NAME.to_owned(),
            appearance_row: H2_R42_APPEARANCE_ROW,
            position: M0RuntimePositionV1 {
                x: 10.0,
                y: 14.5,
                z: 0.0,
            },
            orientation: M0RuntimeDirectionV1 { x: 1.0, y: 0.0 },
        },
        BinaryCreatureOwnedFixtureV1 {
            id: H2_R42_STOCK_CONTROL_ID.to_owned(),
            template_resref: H2_R42_STOCK_CONTROL_TEMPLATE_RESREF.to_owned(),
            display_name: H2_R42_STOCK_CONTROL_DISPLAY_NAME.to_owned(),
            appearance_row: H2_R42_STOCK_CONTROL_APPEARANCE_ROW,
            position: M0RuntimePositionV1 {
                x: 7.0,
                y: 14.5,
                z: 0.0,
            },
            orientation: M0RuntimeDirectionV1 { x: 1.0, y: 0.0 },
        },
    ];
    let module = build_binary_creature_multi_fixture_module_v1(
        &BinaryCreatureModuleIdentityV1 {
            module_resref: H2_R42_MODULE_RESREF.to_owned(),
            area_resref: H2_R42_AREA_RESREF.to_owned(),
            hak_resref: H2_R42_HAK_RESREF.to_owned(),
        },
        &fixtures,
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let binary_scene = inspect_binary_creature_multi_fixture_module_v1(&module.payload)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if sha256_bytes(&module.payload) != H2_R42_MODULE_SHA256 {
        return Err(candidate_error(
            "M2A-H2-R42-MODULE-IDENTITY",
            "module",
            "H2 r42 MOD bytes differ from the frozen candidate identity",
        ));
    }
    if binary_scene != module.readback
        || binary_scene.ordered_hak_resrefs != [H2_R42_HAK_RESREF]
        || binary_scene.fixtures != fixtures
    {
        return Err(candidate_error(
            "M2A-H2-R42-MODULE-READBACK",
            "module",
            "runtime-complete r42 MOD differs from its exact identity or two fixtures",
        ));
    }

    let mut flat_nodes = Vec::new();
    collect_h2_r42_nodes(&model.inspection.node_tree.roots, &mut flat_nodes);
    let rigid_mesh_node_count = flat_nodes
        .iter()
        .filter(|node| node.mesh.is_some() && node.skin.is_none())
        .count();
    let skin_node_count = flat_nodes.iter().filter(|node| node.skin.is_some()).count();
    let triangle_count = flat_nodes
        .iter()
        .filter_map(|node| node.mesh.as_ref())
        .map(|mesh| mesh.faces.len())
        .sum::<usize>();
    let animation_names = model
        .inspection
        .animations
        .iter()
        .map(|animation| animation.name.clone())
        .collect::<Vec<_>>();
    if rigid_mesh_node_count != 1
        || skin_node_count != 0
        || triangle_count != 1_543
        || model.inspection.animations.iter().any(|animation| {
            animation.animation_type != 0
                || animation.node_tree.node_count + 1 != model.inspection.node_tree.node_count
        })
    {
        return Err(candidate_error(
            "M2A-H2-R42-MODEL-READBACK",
            "model",
            "r42 must contain one root-attached rigid H2 surface and seven owned-witness type-0 rig-only states",
        ));
    }

    let contract = H2R42VisibilityCandidateContractV1 {
        schema_version: 1,
        profile: "H2_R42_OWNED_TYPE0_ROOT_RIGID_VISIBILITY_CANDIDATE_V1".to_owned(),
        candidate_admissible: true,
        admitted_by_r41_proof_sha256: H2_R42_ACCEPTED_R41_PROOF_SHA256.to_owned(),
        diagnosed_failure_boundary: "the owner-bound r41 result proves that the 20-piece bone-parented M0 surface remains absent in NWN even after full type-5 state projection; the earlier project-owned H1 v20 witness proves that NWN can draw this writer's one-piece rigid surface with type-0 rig-only states, although attaching that surface to the animated skeleton root deformed its placement".to_owned(),
        intended_functional_delta: vec![
            "replace the gray M0 source with the newly generated turquoise-and-copper Meshy H2 source, including its own 24-joint skin and Idle clip provenance".to_owned(),
            "isolate visibility with exactly one rigid H2 surface instead of the r41 20-piece bone-parented surface".to_owned(),
            "insert a controllerless identity Aurora Root, bake the old skeleton-root bind transform into the surface and attach the rigid surface directly to the new root so bone animation cannot tear or move it".to_owned(),
            "emit the project-owned runtime-positive type-0 rig-only local-state family and include a stock Hook Horror side control in the same complete module".to_owned(),
        ],
        source: byte_identity(source_glb),
        base_appearance: byte_identity(base_appearance_two_da),
        model: resource_binding(H2_R42_MODEL_RESREF, &model.payload),
        texture: resource_binding(H2_R42_TEXTURE_RESREF, &texture),
        appearance_two_da: resource_binding("appearance", &appearance.payload),
        hak: resource_binding(H2_R42_HAK_RESREF, &package.hak.payload),
        module: resource_binding(H2_R42_MODULE_RESREF, &module.payload),
        package_manifest: package.manifest,
        binary_scene,
        appearance_row: H2_R42_APPEARANCE_ROW,
        triangle_count,
        rig_node_count: creature.nodes.len(),
        rigid_mesh_node_count,
        skin_node_count,
        animation_names,
        state_projection_profile:
            MdlStateProjectionProfileV1::OwnedRuntimePositiveType0RigOnlyV1,
        materialization_count: 1,
        toolset_model_visibility: "not_tested".to_owned(),
        toolset_proof_completeness: "missing".to_owned(),
        nwn_model_visibility: "not_tested".to_owned(),
        nwn_proof_completeness: "missing".to_owned(),
    };
    let mut contract_json = serde_json::to_vec_pretty(&contract).map_err(|error| {
        candidate_error(
            "M2A-H2-R42-CONTRACT-SERIALIZATION",
            "contract",
            error.to_string(),
        )
    })?;
    contract_json.push(b'\n');
    if sha256_bytes(&contract_json) != H2_R42_CONTRACT_SHA256 {
        return Err(candidate_error(
            "M2A-H2-R42-CONTRACT-IDENTITY",
            "contract",
            "H2 r42 contract bytes differ from the frozen candidate identity",
        ));
    }

    Ok(H2R42VisibilityCandidateArtifactV1 {
        model: model.payload,
        texture,
        appearance_two_da: appearance.payload,
        hak: package.hak.payload,
        module: module.payload,
        contract,
        contract_json,
    })
}

pub fn verify_h2_r42_visibility_candidate_v1(
    contract: &H2R42VisibilityCandidateContractV1,
    source_glb: &[u8],
    base_appearance_two_da: &[u8],
    module: &[u8],
    hak: &[u8],
) -> Result<(), H2R42VisibilityCandidateErrorV1> {
    let replay = build_h2_r42_visibility_candidate_v1(source_glb, base_appearance_two_da)?;
    if contract != &replay.contract || module != replay.module || hak != replay.hak {
        return Err(candidate_error(
            "M2A-H2-R42-REPLAY-MISMATCH",
            "candidate",
            "contract, MOD or HAK differs from the exact-input deterministic replay",
        ));
    }
    Ok(())
}

pub fn build_h2_r43_h1_root_layout_candidate_v1(
    source_glb: &[u8],
    base_appearance_two_da: &[u8],
) -> Result<H2R43H1RootLayoutCandidateArtifactV1, H2R43H1RootLayoutCandidateErrorV1> {
    require_exact_h2_r43_input(
        source_glb,
        H2_R43_SOURCE_BYTE_LENGTH,
        H2_R43_SOURCE_SHA256,
        "sourceGlb",
    )?;
    require_exact_h2_r43_input(
        base_appearance_two_da,
        M0_R33_BASE_APPEARANCE_BYTE_LENGTH,
        H2_R43_BASE_APPEARANCE_SHA256,
        "baseAppearanceTwoDa",
    )?;
    let source = ingest_glb(source_glb, &GlbLimits::default()).map_err(|error| {
        candidate_error(
            error.code,
            error.json_path.unwrap_or_else(|| "sourceGlb".to_owned()),
            error.message,
        )
    })?;
    let appearance_inspection =
        inspect_two_da_v2(base_appearance_two_da, &TwoDaLimitsV1::default())
            .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if appearance_inspection.physical_row_count != M0_R33_BASE_APPEARANCE_ROWS {
        return Err(candidate_error(
            "M2A-H2-R43-APPEARANCE-TRUST-ROOT",
            "baseAppearanceTwoDa",
            "r43 requires the exact full 15100-row runtime appearance table",
        ));
    }

    let (rig, mapping) = derive_meshy_h1_profile_and_mapping_v1(&source)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let mut conversion = convert_profile_a_with_animations_v1(
        &source,
        &rig,
        &ProfileAOptionsV1::default(),
        &mapping,
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if !conversion.base.report.conversion_eligible {
        return Err(candidate_error(
            "M2A-H2-R43-CONVERSION-INELIGIBLE",
            "conversion.report",
            "the exact generated H2 source did not produce an eligible creature conversion",
        ));
    }
    let source_animations = conversion.animations.as_ref().ok_or_else(|| {
        candidate_error(
            "M2A-H2-R43-ANIMATION-MISSING",
            "conversion.animations",
            "the rigged H2 source must provide its mapped Idle animation",
        )
    })?;
    let mut animations = materialize_direct_creature_runtime_clips(source_animations)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let mut creature = conversion.base.creature.take().ok_or_else(|| {
        candidate_error(
            "M2A-H2-R43-CONVERSION-INELIGIBLE",
            "conversion.creature",
            "eligible H2 conversion did not produce creature IR",
        )
    })?;
    let root_indexes = creature
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| node.parent_id.is_none().then_some(index))
        .collect::<Vec<_>>();
    let [root_index] = root_indexes.as_slice() else {
        return Err(candidate_error(
            "M2A-H2-R43-RIG-ROOT",
            "conversion.creature.nodes",
            "H2 must produce exactly one source skeleton root",
        ));
    };
    if creature.nodes.len() != 24
        || creature.segments.len() != 1
        || creature.segments[0].deformation != RigSegmentDeformationV1::Skin
    {
        return Err(candidate_error(
            "M2A-H2-R43-SOURCE-SHAPE",
            "conversion.creature",
            "r43 requires the exact 24-joint, one-skin H2 conversion before rigid isolation",
        ));
    }
    let root_id = creature.nodes[*root_index].id;
    if creature.segments[0].parent_node_id != root_id {
        return Err(candidate_error(
            "M2A-H2-R43-SURFACE-PARENT",
            "conversion.creature.segments[0].parentNodeId",
            "the source H2 surface must already be attached to its single skeleton root",
        ));
    }
    creature.nodes[*root_index].name = H2_R43_MODEL_RESREF.to_owned();
    creature.segments[0].deformation = RigSegmentDeformationV1::Rigid;
    creature.segments[0].weights.clear();
    for clip in &mut animations.clips {
        clip.animation_root = H2_R43_MODEL_RESREF.to_owned();
    }

    let texture_selection = resolve_base_color_image_index_v1(&source, &creature)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let material_slot = creature.segments[0].material_slot;
    let format_profile = MdlFormatProfileV1::M4DirectCreatureExtended64V1;
    let state_projection_profile = MdlStateProjectionProfileV1::OwnedRuntimePositiveType0RigOnlyV1;
    let model = write_binary_mdl_with_animations(
        &creature,
        &animations,
        &MdlWriterOptionsV1 {
            schema_version: 1,
            format_profile,
            state_projection_profile,
            state_projection_provenance: None,
            model_resource_resref: H2_R43_MODEL_RESREF.to_owned(),
            diffuse_texture_resref_by_material_slot: vec![MdlMaterialTextureBindingV1 {
                material_slot,
                resref: H2_R43_TEXTURE_RESREF.to_owned(),
            }],
        },
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if sha256_bytes(&model.payload) != H2_R43_MODEL_SHA256 {
        return Err(candidate_error(
            "M2A-H2-R43-MODEL-IDENTITY",
            "model",
            "H2 r43 model bytes differ from the frozen candidate identity",
        ));
    }
    verify_direct_creature_state_projection_v1(&model.inspection, state_projection_profile, None)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;

    let image = decode_embedded_image_to_tga_v1(
        source_glb,
        texture_selection.source_image_index,
        &GlbLimits::default(),
        &EmbeddedImageDecodeLimitsV1::default(),
    )
    .map_err(|error| {
        candidate_error(
            error.code,
            error
                .json_path
                .unwrap_or_else(|| "sourceGlb.images".to_owned()),
            error.message,
        )
    })?;
    let texture = write_tga_v1(&image, &TgaWriterOptionsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?
        .payload;
    if sha256_bytes(&texture) != H2_R43_TEXTURE_SHA256 {
        return Err(candidate_error(
            "M2A-H2-R43-TEXTURE-IDENTITY",
            "texture",
            "H2 r43 texture bytes differ from the frozen candidate identity",
        ));
    }
    let appearance = append_h2_r43_appearance(base_appearance_two_da)?;
    if sha256_bytes(&appearance.payload) != H2_R43_APPEARANCE_SHA256 {
        return Err(candidate_error(
            "M2A-H2-R43-APPEARANCE-IDENTITY",
            "appearance",
            "H2 r43 appearance bytes differ from the frozen candidate identity",
        ));
    }
    require_h2_r43_appearance_cell(
        &appearance.payload,
        H2_R43_APPEARANCE_ROW,
        "LABEL",
        H2_R43_APPEARANCE_LABEL,
    )?;
    require_h2_r43_appearance_cell(&appearance.payload, H2_R43_APPEARANCE_ROW, "MODELTYPE", "S")?;
    require_h2_r43_appearance_cell(
        &appearance.payload,
        H2_R43_APPEARANCE_ROW,
        "RACE",
        H2_R43_MODEL_RESREF,
    )?;

    let resources = vec![
        HakResourceInputV1 {
            resref: H2_R43_MODEL_RESREF.to_owned(),
            resource_type: 2002,
            payload: model.payload.clone(),
        },
        HakResourceInputV1 {
            resref: H2_R43_TEXTURE_RESREF.to_owned(),
            resource_type: 3,
            payload: texture.clone(),
        },
        HakResourceInputV1 {
            resref: "appearance".to_owned(),
            resource_type: 2017,
            payload: appearance.payload.clone(),
        },
    ];
    let package = write_model_package_v1(&resources, &HakWriterOptionsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if sha256_bytes(&package.hak.payload) != H2_R43_HAK_SHA256 {
        return Err(candidate_error(
            "M2A-H2-R43-HAK-IDENTITY",
            "hak",
            "H2 r43 HAK bytes differ from the frozen candidate identity",
        ));
    }
    let archive = ErfArchive::parse(&package.hak.payload).map_err(|error| {
        candidate_error(error.code, format!("hak@{}", error.offset), error.context)
    })?;
    for (resref, resource_type, expected) in [
        (H2_R43_MODEL_RESREF, 2002, model.payload.as_slice()),
        (H2_R43_TEXTURE_RESREF, 3, texture.as_slice()),
        ("appearance", 2017, appearance.payload.as_slice()),
    ] {
        let actual = archive.find(resref, resource_type).map_err(|error| {
            candidate_error(error.code, format!("hak@{}", error.offset), error.context)
        })?;
        if actual != expected {
            return Err(candidate_error(
                "M2A-H2-R43-HAK-READBACK",
                format!("hak.{resref}:{resource_type}"),
                "generated H2 r43 HAK resource differs from its exact input payload",
            ));
        }
    }

    let fixtures = [
        BinaryCreatureOwnedFixtureV1 {
            id: H2_R43_FIXTURE_ID.to_owned(),
            template_resref: H2_R43_FIXTURE_TEMPLATE_RESREF.to_owned(),
            display_name: H2_R43_FIXTURE_DISPLAY_NAME.to_owned(),
            appearance_row: H2_R43_APPEARANCE_ROW,
            position: M0RuntimePositionV1 {
                x: 10.0,
                y: 14.5,
                z: 0.0,
            },
            orientation: M0RuntimeDirectionV1 { x: 1.0, y: 0.0 },
        },
        BinaryCreatureOwnedFixtureV1 {
            id: H2_R43_STOCK_CONTROL_ID.to_owned(),
            template_resref: H2_R43_STOCK_CONTROL_TEMPLATE_RESREF.to_owned(),
            display_name: H2_R43_STOCK_CONTROL_DISPLAY_NAME.to_owned(),
            appearance_row: H2_R43_STOCK_CONTROL_APPEARANCE_ROW,
            position: M0RuntimePositionV1 {
                x: 7.0,
                y: 14.5,
                z: 0.0,
            },
            orientation: M0RuntimeDirectionV1 { x: 1.0, y: 0.0 },
        },
    ];
    let module = build_binary_creature_multi_fixture_module_v1(
        &BinaryCreatureModuleIdentityV1 {
            module_resref: H2_R43_MODULE_RESREF.to_owned(),
            area_resref: H2_R43_AREA_RESREF.to_owned(),
            hak_resref: H2_R43_HAK_RESREF.to_owned(),
        },
        &fixtures,
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let binary_scene = inspect_binary_creature_multi_fixture_module_v1(&module.payload)
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    if sha256_bytes(&module.payload) != H2_R43_MODULE_SHA256 {
        return Err(candidate_error(
            "M2A-H2-R43-MODULE-IDENTITY",
            "module",
            "H2 r43 MOD bytes differ from the frozen candidate identity",
        ));
    }
    if binary_scene != module.readback
        || binary_scene.ordered_hak_resrefs != [H2_R43_HAK_RESREF]
        || binary_scene.fixtures != fixtures
    {
        return Err(candidate_error(
            "M2A-H2-R43-MODULE-READBACK",
            "module",
            "runtime-complete r43 MOD differs from its exact identity or two fixtures",
        ));
    }

    let mut flat_nodes = Vec::new();
    collect_h2_r42_nodes(&model.inspection.node_tree.roots, &mut flat_nodes);
    let rigid_mesh_node_count = flat_nodes
        .iter()
        .filter(|node| node.mesh.is_some() && node.skin.is_none())
        .count();
    let skin_node_count = flat_nodes.iter().filter(|node| node.skin.is_some()).count();
    let triangle_count = flat_nodes
        .iter()
        .filter_map(|node| node.mesh.as_ref())
        .map(|mesh| mesh.faces.len())
        .sum::<usize>();
    let root = model.inspection.node_tree.roots.first().ok_or_else(|| {
        candidate_error("M2A-H2-R43-MODEL-READBACK", "model.root", "missing root")
    })?;
    let base_root_controller_count = root.controllers.len();
    let animation_names = model
        .inspection
        .animations
        .iter()
        .map(|animation| animation.name.clone())
        .collect::<Vec<_>>();
    let mesh_parent_is_root = flat_nodes.iter().any(|node| {
        node.mesh.is_some() && node.skin.is_none() && node.parent_offset == Some(root.offset)
    });
    if model.inspection.node_tree.node_count != 25
        || creature.nodes.len() != 24
        || root.name != H2_R43_MODEL_RESREF
        || base_root_controller_count != 2
        || rigid_mesh_node_count != 1
        || skin_node_count != 0
        || triangle_count != 1_543
        || !mesh_parent_is_root
        || model.inspection.animations.iter().any(|animation| {
            animation.animation_type != 0
                || animation.node_tree.node_count != 24
                || animation
                    .node_tree
                    .roots
                    .iter()
                    .any(|state_root| state_root.name != H2_R43_MODEL_RESREF)
        })
    {
        return Err(candidate_error(
            "M2A-H2-R43-MODEL-READBACK",
            "model",
            "r43 must match the H1 v20 root/controller/rig-only-state shape with one root-attached rigid H2 surface",
        ));
    }

    let contract = H2R43H1RootLayoutCandidateContractV1 {
        schema_version: 1,
        profile: "H2_R43_H1_V20_ROOT_RIGID_TYPE0_CANDIDATE_V1".to_owned(),
        candidate_admissible: true,
        admitted_by_r42_proof_sha256: H2_R43_ACCEPTED_R42_PROOF_SHA256.to_owned(),
        diagnosed_failure_boundary: "the owner-bound r42 result proves that a newly inserted controllerless identity Aurora Root using writer profile V3 remains absent in NWN, while the project-owned H1 v20 rigid witness was drawn with the mapped skeleton root renamed to the model resref and writer profile V1".to_owned(),
        intended_functional_delta: vec![
            "remove the dedicated controllerless r42 Aurora Root and preserve the exact 24-joint H2 source hierarchy".to_owned(),
            "rename the sole mapped skeleton root to the r43 model resref, retain its bind position/orientation controllers and attach the single rigid H2 surface directly to that root".to_owned(),
            "switch only the binary layout profile from controllerless-root V3 to the runtime-positive H1 v20 V1 family while retaining the type-0 rig-only state projection".to_owned(),
        ],
        source: byte_identity(source_glb),
        base_appearance: byte_identity(base_appearance_two_da),
        model: resource_binding(H2_R43_MODEL_RESREF, &model.payload),
        texture: resource_binding(H2_R43_TEXTURE_RESREF, &texture),
        appearance_two_da: resource_binding("appearance", &appearance.payload),
        hak: resource_binding(H2_R43_HAK_RESREF, &package.hak.payload),
        module: resource_binding(H2_R43_MODULE_RESREF, &module.payload),
        package_manifest: package.manifest,
        binary_scene,
        appearance_row: H2_R43_APPEARANCE_ROW,
        triangle_count,
        rig_node_count: creature.nodes.len(),
        rigid_mesh_node_count,
        skin_node_count,
        base_root_controller_count,
        animation_names,
        format_profile,
        state_projection_profile,
        materialization_count: 1,
        toolset_model_visibility: "not_tested".to_owned(),
        toolset_proof_completeness: "missing".to_owned(),
        nwn_model_visibility: "not_tested".to_owned(),
        nwn_proof_completeness: "missing".to_owned(),
    };
    let mut contract_json = serde_json::to_vec_pretty(&contract).map_err(|error| {
        candidate_error(
            "M2A-H2-R43-CONTRACT-SERIALIZATION",
            "contract",
            error.to_string(),
        )
    })?;
    contract_json.push(b'\n');
    if sha256_bytes(&contract_json) != H2_R43_CONTRACT_SHA256 {
        return Err(candidate_error(
            "M2A-H2-R43-CONTRACT-IDENTITY",
            "contract",
            "H2 r43 contract bytes differ from the frozen candidate identity",
        ));
    }

    Ok(H2R43H1RootLayoutCandidateArtifactV1 {
        model: model.payload,
        texture,
        appearance_two_da: appearance.payload,
        hak: package.hak.payload,
        module: module.payload,
        contract,
        contract_json,
    })
}

pub fn verify_h2_r43_h1_root_layout_candidate_v1(
    contract: &H2R43H1RootLayoutCandidateContractV1,
    source_glb: &[u8],
    base_appearance_two_da: &[u8],
    module: &[u8],
    hak: &[u8],
) -> Result<(), H2R43H1RootLayoutCandidateErrorV1> {
    let replay = build_h2_r43_h1_root_layout_candidate_v1(source_glb, base_appearance_two_da)?;
    if contract != &replay.contract || module != replay.module || hak != replay.hak {
        return Err(candidate_error(
            "M2A-H2-R43-REPLAY-MISMATCH",
            "candidate",
            "contract, MOD or HAK differs from the exact-input deterministic replay",
        ));
    }
    Ok(())
}

fn collect_h2_r42_nodes<'a>(nodes: &'a [NodeReport], output: &mut Vec<&'a NodeReport>) {
    for node in nodes {
        output.push(node);
        collect_h2_r42_nodes(&node.children, output);
    }
}

fn h2_r42_transform_point(matrix: [f32; 16], point: [f32; 3]) -> Option<[f32; 3]> {
    let output = [
        matrix[0] * point[0] + matrix[4] * point[1] + matrix[8] * point[2] + matrix[12],
        matrix[1] * point[0] + matrix[5] * point[1] + matrix[9] * point[2] + matrix[13],
        matrix[2] * point[0] + matrix[6] * point[1] + matrix[10] * point[2] + matrix[14],
    ];
    output
        .iter()
        .all(|value| value.is_finite())
        .then_some(output)
}

fn h2_r42_transform_direction(matrix: [f32; 16], direction: [f32; 3]) -> Option<[f32; 3]> {
    let output = [
        matrix[0] * direction[0] + matrix[4] * direction[1] + matrix[8] * direction[2],
        matrix[1] * direction[0] + matrix[5] * direction[1] + matrix[9] * direction[2],
        matrix[2] * direction[0] + matrix[6] * direction[1] + matrix[10] * direction[2],
    ];
    let length = (output[0] * output[0] + output[1] * output[1] + output[2] * output[2]).sqrt();
    (length.is_finite() && length > f32::EPSILON).then_some([
        output[0] / length,
        output[1] / length,
        output[2] / length,
    ])
}

fn inspect_r41_rigid_contract(
    inspection: &crate::mdl::InspectionReport,
) -> Result<M0R41RigidReadbackV1, M0R41AnimatedDonorCandidateErrorV1> {
    fn collect_nodes<'a>(nodes: &'a [NodeReport], output: &mut Vec<&'a NodeReport>) {
        for node in nodes {
            output.push(node);
            collect_nodes(&node.children, output);
        }
    }

    let [aurora_root] = inspection.node_tree.roots.as_slice() else {
        return Err(candidate_error(
            "M2A-R41-MDL-READBACK",
            "model.nodeTree.roots",
            "r41 requires exactly one dedicated Aurora Root",
        ));
    };
    let skeleton_root = aurora_root
        .children
        .iter()
        .find(|node| node.name == "Hips")
        .ok_or_else(|| {
            candidate_error(
                "M2A-R41-MDL-READBACK",
                "model.nodeTree.roots[0].children",
                "r41 must preserve Hips immediately below the dedicated Aurora Root",
            )
        })?;
    let mut nodes = Vec::new();
    collect_nodes(&inspection.node_tree.roots, &mut nodes);
    let mesh_nodes = nodes
        .iter()
        .copied()
        .filter(|node| node.mesh.is_some())
        .collect::<Vec<_>>();
    let skin_node_count = nodes.iter().filter(|node| node.skin.is_some()).count();
    let parent_offsets = mesh_nodes
        .iter()
        .filter_map(|node| node.parent_offset)
        .collect::<std::collections::BTreeSet<_>>();
    let triangle_count = mesh_nodes
        .iter()
        .map(|node| node.mesh.as_ref().expect("filtered mesh").faces.len())
        .sum::<usize>();
    let duplicated_vertex_count = mesh_nodes
        .iter()
        .map(|node| node.mesh.as_ref().expect("filtered mesh").vertex_count)
        .sum::<usize>();
    let animation_names = inspection
        .animations
        .iter()
        .map(|animation| animation.name.clone())
        .collect::<Vec<_>>();
    let animation_node_counts = inspection
        .animations
        .iter()
        .map(|animation| animation.node_tree.node_count)
        .collect::<Vec<_>>();
    let animation_mesh_identity_counts = inspection
        .animations
        .iter()
        .map(|animation| {
            let mut state_nodes = Vec::new();
            collect_nodes(&animation.node_tree.roots, &mut state_nodes);
            state_nodes
                .iter()
                .filter(|node| node.name.starts_with("m2a_seg_"))
                .count()
        })
        .collect::<Vec<_>>();
    let animation_scale_controller_count = inspection
        .animations
        .iter()
        .map(|animation| {
            let mut count = 0;
            let mut pending = animation.node_tree.roots.iter().collect::<Vec<_>>();
            while let Some(node) = pending.pop() {
                count += node
                    .controllers
                    .iter()
                    .filter(|controller| controller.controller_type == 36)
                    .count();
                pending.extend(&node.children);
            }
            count
        })
        .sum::<usize>();
    let rigid_mesh_contract_ok = mesh_nodes.iter().all(|node| {
        let mesh = node.mesh.as_ref().expect("filtered mesh");
        node.content_flags == 0x21
            && node.skin.is_none()
            && node.parent_offset.is_some()
            && node.parent_offset != Some(aurora_root.offset)
            && mesh.mesh_type == 3
            && mesh.textures.first().map(String::as_str) == Some(M0_R41_TEXTURE_RESREF)
            && mesh.vertex_count == mesh.vertices.len()
            && mesh.vertex_count == mesh.uv0.len()
            && mesh.vertex_count == mesh.normals.len()
            && mesh.vertex_count == mesh.faces.len() * 3
    });
    let parent_contract_ok = parent_offsets.iter().all(|parent_offset| {
        nodes
            .iter()
            .any(|node| node.offset == *parent_offset && node.mesh.is_none())
    });
    if inspection.model.name != M0_R41_MODEL_RESREF
        || aurora_root.name != M0_R41_MODEL_RESREF
        || aurora_root.parent_offset.is_some()
        || !aurora_root.controllers.is_empty()
        || aurora_root.controller_keys_header.pointer != 0
        || aurora_root.controller_keys_header.used != 0
        || aurora_root.controller_keys_header.allocated != 0
        || aurora_root.controller_data_header.pointer != 0
        || aurora_root.controller_data_header.used != 0
        || aurora_root.controller_data_header.allocated != 0
        || skeleton_root.parent_offset != Some(aurora_root.offset)
        || inspection.node_tree.node_count != 45
        || mesh_nodes.len() != 20
        || skin_node_count != 0
        || triangle_count != 1_569
        || duplicated_vertex_count != 4_707
        || parent_offsets.len() != 20
        || !rigid_mesh_contract_ok
        || !parent_contract_ok
        || inspection.animations.len() != M0_R41_REQUIRED_CLIPS.len()
        || animation_names != M0_R41_REQUIRED_CLIPS
        || animation_node_counts.iter().any(|count| *count != 45)
        || animation_mesh_identity_counts
            .iter()
            .any(|count| *count != 20)
        || animation_scale_controller_count != 0
        || inspection.animations.iter().any(|animation| {
            animation.node_tree.roots.is_empty()
                || !animation.node_tree.roots[0].controllers.is_empty()
        })
    {
        return Err(candidate_error(
            "M2A-R41-MDL-READBACK",
            "model",
            "r41 requires the unchanged r40 rigid base surface and seven full 45-identity, scale-free retail type-5 state trees",
        ));
    }

    Ok(M0R41RigidReadbackV1 {
        model_name: inspection.model.name.clone(),
        base_node_count: inspection.node_tree.node_count,
        rig_node_count: 25,
        rigid_mesh_node_count: mesh_nodes.len(),
        skin_node_count,
        triangle_count,
        duplicated_vertex_count,
        active_parent_bone_count: parent_offsets.len(),
        aurora_root_name: aurora_root.name.clone(),
        skeleton_root_name: skeleton_root.name.clone(),
        base_root_controller_count: aurora_root.controllers.len(),
        animation_names,
        animation_node_counts,
        animation_mesh_identity_counts,
        animation_scale_controller_count,
        format_profile:
            MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedControllerlessRootV3,
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5DummyV1,
    })
}

fn inspect_r37_skin_contract(
    inspection: &crate::mdl::InspectionReport,
) -> Result<M0R37SkinReadbackV1, M0R37AnimatedDonorCandidateErrorV1> {
    let [aurora_root] = inspection.node_tree.roots.as_slice() else {
        return Err(candidate_error(
            "M2A-R37-MDL-READBACK",
            "model.nodeTree.roots",
            "r37 requires exactly one dedicated Aurora Root",
        ));
    };
    let skeleton_root = aurora_root
        .children
        .iter()
        .find(|node| node.name == "Hips")
        .ok_or_else(|| {
            candidate_error(
                "M2A-R37-MDL-READBACK",
                "model.nodeTree.roots[0].children",
                "r37 must preserve Hips immediately below the dedicated Aurora Root",
            )
        })?;
    let direct_skin = aurora_root
        .children
        .iter()
        .find(|node| node.skin.is_some())
        .ok_or_else(|| {
            candidate_error(
                "M2A-R37-MDL-READBACK",
                "model.nodeTree.roots[0].children",
                "r37 SkinMesh must be a direct child of the dedicated Aurora Root",
            )
        })?;
    let mut skins = Vec::new();
    collect_skin_nodes(&inspection.node_tree.roots, &mut skins);
    let animation_node_counts = inspection
        .animations
        .iter()
        .map(|animation| animation.node_tree.node_count)
        .collect::<Vec<_>>();
    if inspection.model.name != M0_R37_MODEL_RESREF
        || aurora_root.name != M0_R37_MODEL_RESREF
        || aurora_root.parent_offset.is_some()
        || skeleton_root.parent_offset != Some(aurora_root.offset)
        || direct_skin.parent_offset != Some(aurora_root.offset)
        || inspection.node_tree.node_count != 26
        || skins.len() != 1
        || inspection.animations.len() != M0_R37_REQUIRED_CLIPS.len()
        || animation_node_counts.iter().any(|count| *count != 25)
    {
        return Err(candidate_error(
            "M2A-R37-MDL-READBACK",
            "model",
            "r37 requires a 26-node base with sibling Hips and SkinMesh below the model root, seven animations and 25 rig-only nodes per state",
        ));
    }
    let skin = direct_skin.skin.as_ref().expect("direct skin node");
    let active_bone_count = skin
        .node_to_bone_map
        .iter()
        .copied()
        .filter(|value| *value >= 0)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let active_inline_slot_count = skin
        .bone_references
        .iter()
        .flat_map(|references| references.iter().copied())
        .filter(|reference| *reference != u16::MAX)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let active_inline = skin
        .inline_mapping
        .get(..active_inline_slot_count)
        .ok_or_else(|| {
            candidate_error(
                "M2A-R37-SKIN-READBACK",
                "model.skin.inlineMapping",
                "active inline slot count is outside the 64-entry palette",
            )
        })?;
    let unused_inline_tail = skin
        .inline_mapping
        .get(active_inline_slot_count..)
        .ok_or_else(|| {
            candidate_error(
                "M2A-R37-SKIN-READBACK",
                "model.skin.inlineMapping",
                "active inline slot count is outside the 64-entry palette",
            )
        })?;
    if skin.node_to_bone_map.len() != 26
        || skin.node_to_bone_map[0] != -1
        || skin.q_header.used != 26
        || skin.t_header.used != 26
        || skin.constants_header.used != 26
        || skin.vertex_weights.len() != 2_380
        || skin.bone_references.len() != skin.vertex_weights.len()
        || active_bone_count != 22
        || active_inline_slot_count != 22
        || active_inline != (1_i16..=22).collect::<Vec<_>>()
        || unused_inline_tail.len() != 42
        || unused_inline_tail.iter().any(|value| *value != 0)
        || skin.vertex_weights.iter().any(|weights| {
            let sum = weights.iter().sum::<f32>();
            !sum.is_finite() || (sum - 1.0).abs() > 0.00001
        })
    {
        return Err(candidate_error(
            "M2A-R37-SKIN-READBACK",
            "model.skin",
            "r37 direct-root SkinMesh must preserve 22 dense active joints, normalized weights and a zero-terminated unused tail",
        ));
    }
    let animation_names = inspection
        .animations
        .iter()
        .map(|animation| animation.name.clone())
        .collect::<Vec<_>>();
    if animation_names != M0_R37_REQUIRED_CLIPS {
        return Err(candidate_error(
            "M2A-R37-ANIMATION-READBACK",
            "model.animations",
            "r37 animation names or order differ from the exact r36 inventory",
        ));
    }
    Ok(M0R37SkinReadbackV1 {
        model_name: inspection.model.name.clone(),
        base_node_count: inspection.node_tree.node_count,
        rig_node_count: 25,
        skin_node_count: skins.len(),
        weighted_vertex_count: skin.vertex_weights.len(),
        active_bone_count,
        active_inline_slot_count,
        unused_inline_tail_count: unused_inline_tail.len(),
        unused_inline_tail_value: 0,
        aurora_root_name: aurora_root.name.clone(),
        skeleton_root_name: skeleton_root.name.clone(),
        skin_parent_name: aurora_root.name.clone(),
        aurora_root_tree_ordinal: 0,
        aurora_root_forward_slot: skin.node_to_bone_map[0],
        first_active_inline_ordinal: active_inline[0],
        last_active_inline_ordinal: active_inline[active_inline.len() - 1],
        animation_names,
        animation_node_counts,
        format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2,
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
    })
}

fn inspect_r38_skin_contract(
    inspection: &crate::mdl::InspectionReport,
) -> Result<M0R38SkinReadbackV1, M0R38AnimatedDonorCandidateErrorV1> {
    let [aurora_root] = inspection.node_tree.roots.as_slice() else {
        return Err(candidate_error(
            "M2A-R38-MDL-READBACK",
            "model.nodeTree.roots",
            "r38 requires exactly one dedicated Aurora Root",
        ));
    };
    let skeleton_root = aurora_root
        .children
        .iter()
        .find(|node| node.name == "Hips")
        .ok_or_else(|| {
            candidate_error(
                "M2A-R38-MDL-READBACK",
                "model.nodeTree.roots[0].children",
                "r38 must preserve Hips immediately below the dedicated Aurora Root",
            )
        })?;
    let direct_skin = aurora_root
        .children
        .iter()
        .find(|node| node.skin.is_some())
        .ok_or_else(|| {
            candidate_error(
                "M2A-R38-MDL-READBACK",
                "model.nodeTree.roots[0].children",
                "r38 SkinMesh must be a direct child of the dedicated Aurora Root",
            )
        })?;
    let mut skins = Vec::new();
    collect_skin_nodes(&inspection.node_tree.roots, &mut skins);
    let animation_node_counts = inspection
        .animations
        .iter()
        .map(|animation| animation.node_tree.node_count)
        .collect::<Vec<_>>();
    let animation_scale_controller_count = inspection
        .animations
        .iter()
        .map(|animation| {
            let mut count = 0;
            let mut pending = animation.node_tree.roots.iter().collect::<Vec<_>>();
            while let Some(node) = pending.pop() {
                count += node
                    .controllers
                    .iter()
                    .filter(|controller| controller.controller_type == 36)
                    .count();
                pending.extend(&node.children);
            }
            count
        })
        .sum::<usize>();
    if inspection.model.name != M0_R38_MODEL_RESREF
        || aurora_root.name != M0_R38_MODEL_RESREF
        || aurora_root.parent_offset.is_some()
        || skeleton_root.parent_offset != Some(aurora_root.offset)
        || direct_skin.parent_offset != Some(aurora_root.offset)
        || inspection.node_tree.node_count != 26
        || skins.len() != 1
        || inspection.animations.len() != M0_R38_REQUIRED_CLIPS.len()
        || animation_node_counts.iter().any(|count| *count != 25)
        || animation_scale_controller_count != 0
    {
        return Err(candidate_error(
            "M2A-R38-MDL-READBACK",
            "model",
            "r38 requires the exact r37 topology and state inventory with zero animation-scale controllers",
        ));
    }
    let skin = direct_skin.skin.as_ref().expect("direct skin node");
    let active_bone_count = skin
        .node_to_bone_map
        .iter()
        .copied()
        .filter(|value| *value >= 0)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let active_inline_slot_count = skin
        .bone_references
        .iter()
        .flat_map(|references| references.iter().copied())
        .filter(|reference| *reference != u16::MAX)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let active_inline = skin
        .inline_mapping
        .get(..active_inline_slot_count)
        .ok_or_else(|| {
            candidate_error(
                "M2A-R38-SKIN-READBACK",
                "model.skin.inlineMapping",
                "active inline slot count is outside the 64-entry palette",
            )
        })?;
    let unused_inline_tail = skin
        .inline_mapping
        .get(active_inline_slot_count..)
        .ok_or_else(|| {
            candidate_error(
                "M2A-R38-SKIN-READBACK",
                "model.skin.inlineMapping",
                "active inline slot count is outside the 64-entry palette",
            )
        })?;
    if skin.node_to_bone_map.len() != 26
        || skin.node_to_bone_map[0] != -1
        || skin.q_header.used != 26
        || skin.t_header.used != 26
        || skin.constants_header.used != 26
        || skin.vertex_weights.len() != 2_380
        || skin.bone_references.len() != skin.vertex_weights.len()
        || active_bone_count != 22
        || active_inline_slot_count != 22
        || active_inline != (1_i16..=22).collect::<Vec<_>>()
        || unused_inline_tail.len() != 42
        || unused_inline_tail.iter().any(|value| *value != 0)
        || skin.vertex_weights.iter().any(|weights| {
            let sum = weights.iter().sum::<f32>();
            !sum.is_finite() || (sum - 1.0).abs() > 0.00001
        })
    {
        return Err(candidate_error(
            "M2A-R38-SKIN-READBACK",
            "model.skin",
            "r38 must preserve the exact r37 SkinMesh palette and normalized weights",
        ));
    }
    let animation_names = inspection
        .animations
        .iter()
        .map(|animation| animation.name.clone())
        .collect::<Vec<_>>();
    if animation_names != M0_R38_REQUIRED_CLIPS {
        return Err(candidate_error(
            "M2A-R38-ANIMATION-READBACK",
            "model.animations",
            "r38 animation names or order differ from the exact r37 inventory",
        ));
    }
    Ok(M0R38SkinReadbackV1 {
        model_name: inspection.model.name.clone(),
        base_node_count: inspection.node_tree.node_count,
        rig_node_count: 25,
        skin_node_count: skins.len(),
        weighted_vertex_count: skin.vertex_weights.len(),
        active_bone_count,
        active_inline_slot_count,
        unused_inline_tail_count: unused_inline_tail.len(),
        unused_inline_tail_value: 0,
        aurora_root_name: aurora_root.name.clone(),
        skeleton_root_name: skeleton_root.name.clone(),
        skin_parent_name: aurora_root.name.clone(),
        aurora_root_tree_ordinal: 0,
        aurora_root_forward_slot: skin.node_to_bone_map[0],
        first_active_inline_ordinal: active_inline[0],
        last_active_inline_ordinal: active_inline[active_inline.len() - 1],
        animation_names,
        animation_node_counts,
        animation_scale_controller_count,
        format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2,
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
    })
}

fn inspect_r39_skin_contract(
    inspection: &crate::mdl::InspectionReport,
) -> Result<M0R39SkinReadbackV1, M0R39AnimatedDonorCandidateErrorV1> {
    let [aurora_root] = inspection.node_tree.roots.as_slice() else {
        return Err(candidate_error(
            "M2A-R39-MDL-READBACK",
            "model.nodeTree.roots",
            "r39 requires exactly one dedicated Aurora Root",
        ));
    };
    let skeleton_root = aurora_root
        .children
        .iter()
        .find(|node| node.name == "Hips")
        .ok_or_else(|| {
            candidate_error(
                "M2A-R39-MDL-READBACK",
                "model.nodeTree.roots[0].children",
                "r39 must preserve Hips immediately below the dedicated Aurora Root",
            )
        })?;
    let direct_skin = aurora_root
        .children
        .iter()
        .find(|node| node.skin.is_some())
        .ok_or_else(|| {
            candidate_error(
                "M2A-R39-MDL-READBACK",
                "model.nodeTree.roots[0].children",
                "r39 SkinMesh must be a direct child of the dedicated Aurora Root",
            )
        })?;
    let mut skins = Vec::new();
    collect_skin_nodes(&inspection.node_tree.roots, &mut skins);
    let animation_node_counts = inspection
        .animations
        .iter()
        .map(|animation| animation.node_tree.node_count)
        .collect::<Vec<_>>();
    let animation_scale_controller_count = inspection
        .animations
        .iter()
        .map(|animation| {
            let mut count = 0;
            let mut pending = animation.node_tree.roots.iter().collect::<Vec<_>>();
            while let Some(node) = pending.pop() {
                count += node
                    .controllers
                    .iter()
                    .filter(|controller| controller.controller_type == 36)
                    .count();
                pending.extend(&node.children);
            }
            count
        })
        .sum::<usize>();
    if inspection.model.name != M0_R39_MODEL_RESREF
        || aurora_root.name != M0_R39_MODEL_RESREF
        || aurora_root.parent_offset.is_some()
        || !aurora_root.controllers.is_empty()
        || aurora_root.controller_keys_header.pointer != 0
        || aurora_root.controller_keys_header.used != 0
        || aurora_root.controller_keys_header.allocated != 0
        || aurora_root.controller_data_header.pointer != 0
        || aurora_root.controller_data_header.used != 0
        || aurora_root.controller_data_header.allocated != 0
        || skeleton_root.parent_offset != Some(aurora_root.offset)
        || direct_skin.parent_offset != Some(aurora_root.offset)
        || inspection.node_tree.node_count != 26
        || skins.len() != 1
        || inspection.animations.len() != M0_R39_REQUIRED_CLIPS.len()
        || animation_node_counts.iter().any(|count| *count != 25)
        || animation_scale_controller_count != 0
        || inspection
            .animations
            .iter()
            .any(|animation| !animation.node_tree.roots[0].controllers.is_empty())
    {
        return Err(candidate_error(
            "M2A-R39-MDL-READBACK",
            "model",
            "r39 requires a controllerless identity model root, the exact r38 direct-root SkinMesh topology, seven scale-free animations and 25 rig-only nodes per state",
        ));
    }
    let skin = direct_skin.skin.as_ref().expect("direct skin node");
    let active_bone_count = skin
        .node_to_bone_map
        .iter()
        .copied()
        .filter(|value| *value >= 0)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let active_inline_slot_count = skin
        .bone_references
        .iter()
        .flat_map(|references| references.iter().copied())
        .filter(|reference| *reference != u16::MAX)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let active_inline = skin
        .inline_mapping
        .get(..active_inline_slot_count)
        .ok_or_else(|| {
            candidate_error(
                "M2A-R39-SKIN-READBACK",
                "model.skin.inlineMapping",
                "active inline slot count is outside the 64-entry palette",
            )
        })?;
    let unused_inline_tail = skin
        .inline_mapping
        .get(active_inline_slot_count..)
        .ok_or_else(|| {
            candidate_error(
                "M2A-R39-SKIN-READBACK",
                "model.skin.inlineMapping",
                "active inline slot count is outside the 64-entry palette",
            )
        })?;
    if skin.node_to_bone_map.len() != 26
        || skin.node_to_bone_map[0] != -1
        || skin.q_header.used != 26
        || skin.t_header.used != 26
        || skin.constants_header.used != 26
        || skin.vertex_weights.len() != 2_380
        || skin.bone_references.len() != skin.vertex_weights.len()
        || active_bone_count != 22
        || active_inline_slot_count != 22
        || active_inline != (1_i16..=22).collect::<Vec<_>>()
        || unused_inline_tail.len() != 42
        || unused_inline_tail.iter().any(|value| *value != 0)
        || skin.vertex_weights.iter().any(|weights| {
            let sum = weights.iter().sum::<f32>();
            !sum.is_finite() || (sum - 1.0).abs() > 0.00001
        })
    {
        return Err(candidate_error(
            "M2A-R39-SKIN-READBACK",
            "model.skin",
            "r39 must preserve the exact r38 SkinMesh palette and normalized weights",
        ));
    }
    let animation_names = inspection
        .animations
        .iter()
        .map(|animation| animation.name.clone())
        .collect::<Vec<_>>();
    if animation_names != M0_R39_REQUIRED_CLIPS {
        return Err(candidate_error(
            "M2A-R39-ANIMATION-READBACK",
            "model.animations",
            "r39 animation names or order differ from the exact r38 inventory",
        ));
    }
    Ok(M0R39SkinReadbackV1 {
        model_name: inspection.model.name.clone(),
        base_node_count: inspection.node_tree.node_count,
        rig_node_count: 25,
        skin_node_count: skins.len(),
        weighted_vertex_count: skin.vertex_weights.len(),
        active_bone_count,
        active_inline_slot_count,
        unused_inline_tail_count: unused_inline_tail.len(),
        unused_inline_tail_value: 0,
        aurora_root_name: aurora_root.name.clone(),
        skeleton_root_name: skeleton_root.name.clone(),
        skin_parent_name: aurora_root.name.clone(),
        aurora_root_tree_ordinal: 0,
        aurora_root_forward_slot: skin.node_to_bone_map[0],
        base_root_controller_count: aurora_root.controllers.len(),
        first_active_inline_ordinal: active_inline[0],
        last_active_inline_ordinal: active_inline[active_inline.len() - 1],
        animation_names,
        animation_node_counts,
        animation_scale_controller_count,
        format_profile:
            MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedControllerlessRootV3,
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
    })
}

fn inspect_r35_skin_contract(
    inspection: &crate::mdl::InspectionReport,
) -> Result<M0R35SkinReadbackV1, M0R35AnimatedDonorCandidateErrorV1> {
    let mut skins = Vec::new();
    collect_skin_nodes(&inspection.node_tree.roots, &mut skins);
    let animation_node_counts = inspection
        .animations
        .iter()
        .map(|animation| animation.node_tree.node_count)
        .collect::<Vec<_>>();
    if inspection.model.name != M0_R35_MODEL_RESREF
        || inspection.node_tree.node_count != 25
        || skins.len() != 1
        || inspection.animations.len() != M0_R35_REQUIRED_CLIPS.len()
        || animation_node_counts.iter().any(|count| *count != 24)
    {
        return Err(candidate_error(
            "M2A-R35-MDL-READBACK",
            "model",
            "r35 requires a 25-node base, one SkinMesh, seven animations and exactly 24 rig-only nodes per state",
        ));
    }
    let skin = skins[0].skin.as_ref().expect("collected skin node");
    let active_bone_count = skin
        .node_to_bone_map
        .iter()
        .copied()
        .filter(|value| *value >= 0)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let active_inline_slot_count = skin
        .bone_references
        .iter()
        .flat_map(|references| references.iter().copied())
        .filter(|reference| *reference != u16::MAX)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let unused_inline_tail = skin
        .inline_mapping
        .get(active_inline_slot_count..)
        .ok_or_else(|| {
            candidate_error(
                "M2A-R35-SKIN-READBACK",
                "model.skin.inlineMapping",
                "active inline slot count is outside the 64-entry palette",
            )
        })?;
    if skin.vertex_weights.len() != 2_380
        || skin.bone_references.len() != skin.vertex_weights.len()
        || active_bone_count != 22
        || active_inline_slot_count != 22
        || unused_inline_tail.len() != 42
        || unused_inline_tail.iter().any(|value| *value != 0)
        || skin.vertex_weights.iter().any(|weights| {
            let sum = weights.iter().sum::<f32>();
            !sum.is_finite() || (sum - 1.0).abs() > 0.00001
        })
    {
        return Err(candidate_error(
            "M2A-R35-SKIN-READBACK",
            "model.skin",
            "r35 SkinMesh weights, active slots or zero-terminated unused palette tail differ from the admitted contract",
        ));
    }
    let animation_names = inspection
        .animations
        .iter()
        .map(|animation| animation.name.clone())
        .collect::<Vec<_>>();
    if animation_names != M0_R35_REQUIRED_CLIPS {
        return Err(candidate_error(
            "M2A-R35-ANIMATION-READBACK",
            "model.animations",
            "r35 animation names or order differ from the exact r34 inventory",
        ));
    }
    Ok(M0R35SkinReadbackV1 {
        model_name: inspection.model.name.clone(),
        base_node_count: inspection.node_tree.node_count,
        skin_node_count: skins.len(),
        weighted_vertex_count: skin.vertex_weights.len(),
        active_bone_count,
        active_inline_slot_count,
        unused_inline_tail_count: unused_inline_tail.len(),
        unused_inline_tail_value: 0,
        animation_names,
        animation_node_counts,
        format_profile: MdlFormatProfileV1::M4DirectCreatureExtended64ZeroTerminatedV2,
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
    })
}

fn inspect_r34_skin_contract(
    inspection: &crate::mdl::InspectionReport,
) -> Result<M0R34SkinReadbackV1, M0R34AnimatedDonorCandidateErrorV1> {
    let mut skins = Vec::new();
    collect_skin_nodes(&inspection.node_tree.roots, &mut skins);
    let animation_node_counts = inspection
        .animations
        .iter()
        .map(|animation| animation.node_tree.node_count)
        .collect::<Vec<_>>();
    if inspection.model.name != M0_R34_MODEL_RESREF
        || inspection.node_tree.node_count != 25
        || skins.len() != 1
        || inspection.animations.len() != M0_R34_REQUIRED_CLIPS.len()
        || animation_node_counts.iter().any(|count| *count != 24)
    {
        return Err(candidate_error(
            "M2A-R34-MDL-READBACK",
            "model",
            "r34 requires a 25-node base, one SkinMesh, seven animations and exactly 24 rig-only nodes per state",
        ));
    }
    let skin = skins[0].skin.as_ref().expect("collected skin node");
    let active_bone_count = skin
        .node_to_bone_map
        .iter()
        .copied()
        .filter(|value| *value >= 0)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    if skin.vertex_weights.len() != 2_380
        || skin.bone_references.len() != skin.vertex_weights.len()
        || active_bone_count != 22
        || skin.vertex_weights.iter().any(|weights| {
            let sum = weights.iter().sum::<f32>();
            !sum.is_finite() || (sum - 1.0).abs() > 0.00001
        })
    {
        return Err(candidate_error(
            "M2A-R34-SKIN-READBACK",
            "model.skin",
            "r34 SkinMesh weights or active-bone map differ from the exact r33 base contract",
        ));
    }
    let animation_names = inspection
        .animations
        .iter()
        .map(|animation| animation.name.clone())
        .collect::<Vec<_>>();
    if animation_names != M0_R34_REQUIRED_CLIPS {
        return Err(candidate_error(
            "M2A-R34-ANIMATION-READBACK",
            "model.animations",
            "r34 animation names or order differ from the exact r33 inventory",
        ));
    }
    Ok(M0R34SkinReadbackV1 {
        model_name: inspection.model.name.clone(),
        base_node_count: inspection.node_tree.node_count,
        skin_node_count: skins.len(),
        weighted_vertex_count: skin.vertex_weights.len(),
        active_bone_count,
        animation_names,
        animation_node_counts,
        state_projection_profile: MdlStateProjectionProfileV1::RetailDirectCreatureType5RigOnlyV1,
    })
}

fn inspect_skin_contract(
    inspection: &crate::mdl::InspectionReport,
) -> Result<M0R33SkinReadbackV1, M0R33AnimatedDonorCandidateErrorV1> {
    let mut skins = Vec::new();
    collect_skin_nodes(&inspection.node_tree.roots, &mut skins);
    if inspection.model.name != M0_R33_MODEL_RESREF
        || inspection.node_tree.node_count != 25
        || skins.len() != 1
        || inspection.animations.len() != M0_R33_REQUIRED_CLIPS.len()
    {
        return Err(candidate_error(
            "M2A-R33-MDL-READBACK",
            "model",
            "r33 requires exact 25-node, one-SkinMesh, seven-animation readback",
        ));
    }
    let skin = skins[0].skin.as_ref().expect("collected skin node");
    let active_bone_count = skin
        .node_to_bone_map
        .iter()
        .copied()
        .filter(|value| *value >= 0)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    if skin.vertex_weights.len() != 2_380
        || skin.bone_references.len() != skin.vertex_weights.len()
        || active_bone_count != 22
        || skin.vertex_weights.iter().any(|weights| {
            let sum = weights.iter().sum::<f32>();
            !sum.is_finite() || (sum - 1.0).abs() > 0.00001
        })
    {
        return Err(candidate_error(
            "M2A-R33-SKIN-READBACK",
            "model.skin",
            "r33 SkinMesh weights or active-bone map differ from the exact retarget contract",
        ));
    }
    let animation_names = inspection
        .animations
        .iter()
        .map(|animation| animation.name.clone())
        .collect::<Vec<_>>();
    if animation_names != M0_R33_REQUIRED_CLIPS {
        return Err(candidate_error(
            "M2A-R33-ANIMATION-READBACK",
            "model.animations",
            "r33 animation names or order differ from the direct-creature inventory",
        ));
    }
    Ok(M0R33SkinReadbackV1 {
        model_name: inspection.model.name.clone(),
        node_count: inspection.node_tree.node_count,
        skin_node_count: skins.len(),
        weighted_vertex_count: skin.vertex_weights.len(),
        active_bone_count,
        animation_names,
    })
}

fn collect_skin_nodes<'a>(nodes: &'a [NodeReport], output: &mut Vec<&'a NodeReport>) {
    for node in nodes {
        if node.skin.is_some() {
            output.push(node);
        }
        collect_skin_nodes(&node.children, output);
    }
}

fn append_r33_appearance(
    source: &[u8],
) -> Result<crate::two_da::TwoDaAppendArtifactV1, M0R33AnimatedDonorCandidateErrorV1> {
    let inspection = inspect_two_da_v2(source, &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let mut cells = [
        ("LABEL", M0_R33_APPEARANCE_LABEL),
        ("MOVERATE", "NORM"),
        ("MODELTYPE", "S"),
        ("RACE", M0_R33_MODEL_RESREF),
        ("PORTRAIT", "****"),
        ("ENVMAP", "****"),
        ("BLOODCOLR", "R"),
        ("WEAPONSCALE", "1.0"),
        ("SIZECATEGORY", "4"),
    ]
    .into_iter()
    .map(|(column, value)| TwoDaCellAssignmentV1 {
        column_name: column.to_owned(),
        value: if value == "****" {
            TwoDaCellValueV1::Null
        } else {
            TwoDaCellValueV1::Text {
                value: value.to_owned(),
            }
        },
    })
    .collect::<Vec<_>>();
    let aliases = [
        "DefaultPhenoType",
        "DefaultPhenotype",
        "DefaultPhenotypeID",
        "DefaultPheno",
    ];
    let present = inspection
        .columns
        .iter()
        .filter(|column| {
            aliases
                .iter()
                .any(|alias| column.eq_ignore_ascii_case(alias))
        })
        .collect::<Vec<_>>();
    if present.len() > 1 {
        return Err(candidate_error(
            "M2A-R33-APPEARANCE-PHENOTYPE-AMBIGUOUS",
            "appearance.columns",
            "full appearance table contains multiple supported phenotype aliases",
        ));
    }
    if let Some(column) = present.first() {
        cells.push(TwoDaCellAssignmentV1 {
            column_name: (*column).clone(),
            value: TwoDaCellValueV1::Text {
                value: "0".to_owned(),
            },
        });
    }
    append_two_da_row_v1(
        source,
        &TwoDaAppendRequestV1 {
            schema_version: 1,
            cells,
        },
        &TwoDaLimitsV1::default(),
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))
}

fn append_r34_appearance(
    source: &[u8],
) -> Result<crate::two_da::TwoDaAppendArtifactV1, M0R34AnimatedDonorCandidateErrorV1> {
    let inspection = inspect_two_da_v2(source, &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let mut cells = [
        ("LABEL", M0_R34_APPEARANCE_LABEL),
        ("MOVERATE", "NORM"),
        ("MODELTYPE", "S"),
        ("RACE", M0_R34_MODEL_RESREF),
        ("PORTRAIT", "****"),
        ("ENVMAP", "****"),
        ("BLOODCOLR", "R"),
        ("WEAPONSCALE", "1.0"),
        ("SIZECATEGORY", "4"),
    ]
    .into_iter()
    .map(|(column, value)| TwoDaCellAssignmentV1 {
        column_name: column.to_owned(),
        value: if value == "****" {
            TwoDaCellValueV1::Null
        } else {
            TwoDaCellValueV1::Text {
                value: value.to_owned(),
            }
        },
    })
    .collect::<Vec<_>>();
    let aliases = [
        "DefaultPhenoType",
        "DefaultPhenotype",
        "DefaultPhenotypeID",
        "DefaultPheno",
    ];
    let present = inspection
        .columns
        .iter()
        .filter(|column| {
            aliases
                .iter()
                .any(|alias| column.eq_ignore_ascii_case(alias))
        })
        .collect::<Vec<_>>();
    if present.len() > 1 {
        return Err(candidate_error(
            "M2A-R34-APPEARANCE-PHENOTYPE-AMBIGUOUS",
            "appearance.columns",
            "full appearance table contains multiple supported phenotype aliases",
        ));
    }
    if let Some(column) = present.first() {
        cells.push(TwoDaCellAssignmentV1 {
            column_name: (*column).clone(),
            value: TwoDaCellValueV1::Text {
                value: "0".to_owned(),
            },
        });
    }
    append_two_da_row_v1(
        source,
        &TwoDaAppendRequestV1 {
            schema_version: 1,
            cells,
        },
        &TwoDaLimitsV1::default(),
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))
}

fn append_r35_appearance(
    source: &[u8],
) -> Result<crate::two_da::TwoDaAppendArtifactV1, M0R35AnimatedDonorCandidateErrorV1> {
    let inspection = inspect_two_da_v2(source, &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let mut cells = [
        ("LABEL", M0_R35_APPEARANCE_LABEL),
        ("MOVERATE", "NORM"),
        ("MODELTYPE", "S"),
        ("RACE", M0_R35_MODEL_RESREF),
        ("PORTRAIT", "****"),
        ("ENVMAP", "****"),
        ("BLOODCOLR", "R"),
        ("WEAPONSCALE", "1.0"),
        ("SIZECATEGORY", "4"),
    ]
    .into_iter()
    .map(|(column, value)| TwoDaCellAssignmentV1 {
        column_name: column.to_owned(),
        value: if value == "****" {
            TwoDaCellValueV1::Null
        } else {
            TwoDaCellValueV1::Text {
                value: value.to_owned(),
            }
        },
    })
    .collect::<Vec<_>>();
    let aliases = [
        "DefaultPhenoType",
        "DefaultPhenotype",
        "DefaultPhenotypeID",
        "DefaultPheno",
    ];
    let present = inspection
        .columns
        .iter()
        .filter(|column| {
            aliases
                .iter()
                .any(|alias| column.eq_ignore_ascii_case(alias))
        })
        .collect::<Vec<_>>();
    if present.len() > 1 {
        return Err(candidate_error(
            "M2A-R35-APPEARANCE-PHENOTYPE-AMBIGUOUS",
            "appearance.columns",
            "full appearance table contains multiple supported phenotype aliases",
        ));
    }
    if let Some(column) = present.first() {
        cells.push(TwoDaCellAssignmentV1 {
            column_name: (*column).clone(),
            value: TwoDaCellValueV1::Text {
                value: "0".to_owned(),
            },
        });
    }
    append_two_da_row_v1(
        source,
        &TwoDaAppendRequestV1 {
            schema_version: 1,
            cells,
        },
        &TwoDaLimitsV1::default(),
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))
}

fn append_r36_appearance(
    source: &[u8],
) -> Result<crate::two_da::TwoDaAppendArtifactV1, M0R36AnimatedDonorCandidateErrorV1> {
    let inspection = inspect_two_da_v2(source, &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let mut cells = [
        ("LABEL", M0_R36_APPEARANCE_LABEL),
        ("MOVERATE", "NORM"),
        ("MODELTYPE", "S"),
        ("RACE", M0_R36_MODEL_RESREF),
        ("PORTRAIT", "****"),
        ("ENVMAP", "****"),
        ("BLOODCOLR", "R"),
        ("WEAPONSCALE", "1.0"),
        ("SIZECATEGORY", "4"),
    ]
    .into_iter()
    .map(|(column, value)| TwoDaCellAssignmentV1 {
        column_name: column.to_owned(),
        value: if value == "****" {
            TwoDaCellValueV1::Null
        } else {
            TwoDaCellValueV1::Text {
                value: value.to_owned(),
            }
        },
    })
    .collect::<Vec<_>>();
    let aliases = [
        "DefaultPhenoType",
        "DefaultPhenotype",
        "DefaultPhenotypeID",
        "DefaultPheno",
    ];
    let present = inspection
        .columns
        .iter()
        .filter(|column| {
            aliases
                .iter()
                .any(|alias| column.eq_ignore_ascii_case(alias))
        })
        .collect::<Vec<_>>();
    if present.len() > 1 {
        return Err(candidate_error(
            "M2A-R36-APPEARANCE-PHENOTYPE-AMBIGUOUS",
            "appearance.columns",
            "full appearance table contains multiple supported phenotype aliases",
        ));
    }
    if let Some(column) = present.first() {
        cells.push(TwoDaCellAssignmentV1 {
            column_name: (*column).clone(),
            value: TwoDaCellValueV1::Text {
                value: "0".to_owned(),
            },
        });
    }
    append_two_da_row_v1(
        source,
        &TwoDaAppendRequestV1 {
            schema_version: 1,
            cells,
        },
        &TwoDaLimitsV1::default(),
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))
}

fn append_r37_appearance(
    source: &[u8],
) -> Result<crate::two_da::TwoDaAppendArtifactV1, M0R37AnimatedDonorCandidateErrorV1> {
    let inspection = inspect_two_da_v2(source, &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let mut cells = [
        ("LABEL", M0_R37_APPEARANCE_LABEL),
        ("MOVERATE", "NORM"),
        ("MODELTYPE", "S"),
        ("RACE", M0_R37_MODEL_RESREF),
        ("PORTRAIT", "****"),
        ("ENVMAP", "****"),
        ("BLOODCOLR", "R"),
        ("WEAPONSCALE", "1.0"),
        ("SIZECATEGORY", "4"),
    ]
    .into_iter()
    .map(|(column, value)| TwoDaCellAssignmentV1 {
        column_name: column.to_owned(),
        value: if value == "****" {
            TwoDaCellValueV1::Null
        } else {
            TwoDaCellValueV1::Text {
                value: value.to_owned(),
            }
        },
    })
    .collect::<Vec<_>>();
    let aliases = [
        "DefaultPhenoType",
        "DefaultPhenotype",
        "DefaultPhenotypeID",
        "DefaultPheno",
    ];
    let present = inspection
        .columns
        .iter()
        .filter(|column| {
            aliases
                .iter()
                .any(|alias| column.eq_ignore_ascii_case(alias))
        })
        .collect::<Vec<_>>();
    if present.len() > 1 {
        return Err(candidate_error(
            "M2A-R37-APPEARANCE-PHENOTYPE-AMBIGUOUS",
            "appearance.columns",
            "full appearance table contains multiple supported phenotype aliases",
        ));
    }
    if let Some(column) = present.first() {
        cells.push(TwoDaCellAssignmentV1 {
            column_name: (*column).clone(),
            value: TwoDaCellValueV1::Text {
                value: "0".to_owned(),
            },
        });
    }
    append_two_da_row_v1(
        source,
        &TwoDaAppendRequestV1 {
            schema_version: 1,
            cells,
        },
        &TwoDaLimitsV1::default(),
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))
}

fn append_r38_appearance(
    source: &[u8],
) -> Result<crate::two_da::TwoDaAppendArtifactV1, M0R38AnimatedDonorCandidateErrorV1> {
    let inspection = inspect_two_da_v2(source, &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let mut cells = [
        ("LABEL", M0_R38_APPEARANCE_LABEL),
        ("MOVERATE", "NORM"),
        ("MODELTYPE", "S"),
        ("RACE", M0_R38_MODEL_RESREF),
        ("PORTRAIT", "****"),
        ("ENVMAP", "****"),
        ("BLOODCOLR", "R"),
        ("WEAPONSCALE", "1.0"),
        ("SIZECATEGORY", "4"),
    ]
    .into_iter()
    .map(|(column, value)| TwoDaCellAssignmentV1 {
        column_name: column.to_owned(),
        value: if value == "****" {
            TwoDaCellValueV1::Null
        } else {
            TwoDaCellValueV1::Text {
                value: value.to_owned(),
            }
        },
    })
    .collect::<Vec<_>>();
    let aliases = [
        "DefaultPhenoType",
        "DefaultPhenotype",
        "DefaultPhenotypeID",
        "DefaultPheno",
    ];
    let present = inspection
        .columns
        .iter()
        .filter(|column| {
            aliases
                .iter()
                .any(|alias| column.eq_ignore_ascii_case(alias))
        })
        .collect::<Vec<_>>();
    if present.len() > 1 {
        return Err(candidate_error(
            "M2A-R38-APPEARANCE-PHENOTYPE-AMBIGUOUS",
            "appearance.columns",
            "full appearance table contains multiple supported phenotype aliases",
        ));
    }
    if let Some(column) = present.first() {
        cells.push(TwoDaCellAssignmentV1 {
            column_name: (*column).clone(),
            value: TwoDaCellValueV1::Text {
                value: "0".to_owned(),
            },
        });
    }
    append_two_da_row_v1(
        source,
        &TwoDaAppendRequestV1 {
            schema_version: 1,
            cells,
        },
        &TwoDaLimitsV1::default(),
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))
}

fn append_r39_appearance(
    source: &[u8],
) -> Result<crate::two_da::TwoDaAppendArtifactV1, M0R39AnimatedDonorCandidateErrorV1> {
    let inspection = inspect_two_da_v2(source, &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let mut cells = [
        ("LABEL", M0_R39_APPEARANCE_LABEL),
        ("MOVERATE", "NORM"),
        ("MODELTYPE", "S"),
        ("RACE", M0_R39_MODEL_RESREF),
        ("PORTRAIT", "****"),
        ("ENVMAP", "****"),
        ("BLOODCOLR", "R"),
        ("WEAPONSCALE", "1.0"),
        ("SIZECATEGORY", "4"),
    ]
    .into_iter()
    .map(|(column, value)| TwoDaCellAssignmentV1 {
        column_name: column.to_owned(),
        value: if value == "****" {
            TwoDaCellValueV1::Null
        } else {
            TwoDaCellValueV1::Text {
                value: value.to_owned(),
            }
        },
    })
    .collect::<Vec<_>>();
    let aliases = [
        "DefaultPhenoType",
        "DefaultPhenotype",
        "DefaultPhenotypeID",
        "DefaultPheno",
    ];
    let present = inspection
        .columns
        .iter()
        .filter(|column| {
            aliases
                .iter()
                .any(|alias| column.eq_ignore_ascii_case(alias))
        })
        .collect::<Vec<_>>();
    if present.len() > 1 {
        return Err(candidate_error(
            "M2A-R39-APPEARANCE-PHENOTYPE-AMBIGUOUS",
            "appearance.columns",
            "full appearance table contains multiple supported phenotype aliases",
        ));
    }
    if let Some(column) = present.first() {
        cells.push(TwoDaCellAssignmentV1 {
            column_name: (*column).clone(),
            value: TwoDaCellValueV1::Text {
                value: "0".to_owned(),
            },
        });
    }
    append_two_da_row_v1(
        source,
        &TwoDaAppendRequestV1 {
            schema_version: 1,
            cells,
        },
        &TwoDaLimitsV1::default(),
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))
}

fn append_r40_appearance(
    source: &[u8],
) -> Result<crate::two_da::TwoDaAppendArtifactV1, M0R40AnimatedDonorCandidateErrorV1> {
    let inspection = inspect_two_da_v2(source, &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let mut cells = [
        ("LABEL", M0_R40_APPEARANCE_LABEL),
        ("MOVERATE", "NORM"),
        ("MODELTYPE", "S"),
        ("RACE", M0_R40_MODEL_RESREF),
        ("PORTRAIT", "****"),
        ("ENVMAP", "****"),
        ("BLOODCOLR", "R"),
        ("WEAPONSCALE", "1.0"),
        ("SIZECATEGORY", "4"),
    ]
    .into_iter()
    .map(|(column, value)| TwoDaCellAssignmentV1 {
        column_name: column.to_owned(),
        value: if value == "****" {
            TwoDaCellValueV1::Null
        } else {
            TwoDaCellValueV1::Text {
                value: value.to_owned(),
            }
        },
    })
    .collect::<Vec<_>>();
    let aliases = [
        "DefaultPhenoType",
        "DefaultPhenotype",
        "DefaultPhenotypeID",
        "DefaultPheno",
    ];
    let present = inspection
        .columns
        .iter()
        .filter(|column| {
            aliases
                .iter()
                .any(|alias| column.eq_ignore_ascii_case(alias))
        })
        .collect::<Vec<_>>();
    if present.len() > 1 {
        return Err(candidate_error(
            "M2A-R40-APPEARANCE-PHENOTYPE-AMBIGUOUS",
            "appearance.columns",
            "full appearance table contains multiple supported phenotype aliases",
        ));
    }
    if let Some(column) = present.first() {
        cells.push(TwoDaCellAssignmentV1 {
            column_name: (*column).clone(),
            value: TwoDaCellValueV1::Text {
                value: "0".to_owned(),
            },
        });
    }
    append_two_da_row_v1(
        source,
        &TwoDaAppendRequestV1 {
            schema_version: 1,
            cells,
        },
        &TwoDaLimitsV1::default(),
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))
}

fn append_r41_appearance(
    source: &[u8],
) -> Result<crate::two_da::TwoDaAppendArtifactV1, M0R41AnimatedDonorCandidateErrorV1> {
    let inspection = inspect_two_da_v2(source, &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let mut cells = [
        ("LABEL", M0_R41_APPEARANCE_LABEL),
        ("MOVERATE", "NORM"),
        ("MODELTYPE", "S"),
        ("RACE", M0_R41_MODEL_RESREF),
        ("PORTRAIT", "****"),
        ("ENVMAP", "****"),
        ("BLOODCOLR", "R"),
        ("WEAPONSCALE", "1.0"),
        ("SIZECATEGORY", "4"),
    ]
    .into_iter()
    .map(|(column, value)| TwoDaCellAssignmentV1 {
        column_name: column.to_owned(),
        value: if value == "****" {
            TwoDaCellValueV1::Null
        } else {
            TwoDaCellValueV1::Text {
                value: value.to_owned(),
            }
        },
    })
    .collect::<Vec<_>>();
    let aliases = [
        "DefaultPhenoType",
        "DefaultPhenotype",
        "DefaultPhenotypeID",
        "DefaultPheno",
    ];
    let present = inspection
        .columns
        .iter()
        .filter(|column| {
            aliases
                .iter()
                .any(|alias| column.eq_ignore_ascii_case(alias))
        })
        .collect::<Vec<_>>();
    if present.len() > 1 {
        return Err(candidate_error(
            "M2A-R41-APPEARANCE-PHENOTYPE-AMBIGUOUS",
            "appearance.columns",
            "full appearance table contains multiple supported phenotype aliases",
        ));
    }
    if let Some(column) = present.first() {
        cells.push(TwoDaCellAssignmentV1 {
            column_name: (*column).clone(),
            value: TwoDaCellValueV1::Text {
                value: "0".to_owned(),
            },
        });
    }
    append_two_da_row_v1(
        source,
        &TwoDaAppendRequestV1 {
            schema_version: 1,
            cells,
        },
        &TwoDaLimitsV1::default(),
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))
}

fn append_h2_r42_appearance(
    source: &[u8],
) -> Result<crate::two_da::TwoDaAppendArtifactV1, H2R42VisibilityCandidateErrorV1> {
    let inspection = inspect_two_da_v2(source, &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let mut cells = [
        ("LABEL", H2_R42_APPEARANCE_LABEL),
        ("MOVERATE", "NORM"),
        ("MODELTYPE", "S"),
        ("RACE", H2_R42_MODEL_RESREF),
        ("PORTRAIT", "****"),
        ("ENVMAP", "****"),
        ("BLOODCOLR", "R"),
        ("WEAPONSCALE", "1.0"),
        ("SIZECATEGORY", "4"),
    ]
    .into_iter()
    .map(|(column, value)| TwoDaCellAssignmentV1 {
        column_name: column.to_owned(),
        value: if value == "****" {
            TwoDaCellValueV1::Null
        } else {
            TwoDaCellValueV1::Text {
                value: value.to_owned(),
            }
        },
    })
    .collect::<Vec<_>>();
    let aliases = [
        "DefaultPhenoType",
        "DefaultPhenotype",
        "DefaultPhenotypeID",
        "DefaultPheno",
    ];
    let present = inspection
        .columns
        .iter()
        .filter(|column| {
            aliases
                .iter()
                .any(|alias| column.eq_ignore_ascii_case(alias))
        })
        .collect::<Vec<_>>();
    if present.len() > 1 {
        return Err(candidate_error(
            "M2A-H2-R42-APPEARANCE-PHENOTYPE-AMBIGUOUS",
            "appearance.columns",
            "full appearance table contains multiple supported phenotype aliases",
        ));
    }
    if let Some(column) = present.first() {
        cells.push(TwoDaCellAssignmentV1 {
            column_name: (*column).clone(),
            value: TwoDaCellValueV1::Text {
                value: "0".to_owned(),
            },
        });
    }
    append_two_da_row_v1(
        source,
        &TwoDaAppendRequestV1 {
            schema_version: 1,
            cells,
        },
        &TwoDaLimitsV1::default(),
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))
}

fn append_h2_r43_appearance(
    source: &[u8],
) -> Result<crate::two_da::TwoDaAppendArtifactV1, H2R43H1RootLayoutCandidateErrorV1> {
    let inspection = inspect_two_da_v2(source, &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let mut cells = [
        ("LABEL", H2_R43_APPEARANCE_LABEL),
        ("MOVERATE", "NORM"),
        ("MODELTYPE", "S"),
        ("RACE", H2_R43_MODEL_RESREF),
        ("PORTRAIT", "****"),
        ("ENVMAP", "****"),
        ("BLOODCOLR", "R"),
        ("WEAPONSCALE", "1.0"),
        ("SIZECATEGORY", "4"),
    ]
    .into_iter()
    .map(|(column, value)| TwoDaCellAssignmentV1 {
        column_name: column.to_owned(),
        value: if value == "****" {
            TwoDaCellValueV1::Null
        } else {
            TwoDaCellValueV1::Text {
                value: value.to_owned(),
            }
        },
    })
    .collect::<Vec<_>>();
    let aliases = [
        "DefaultPhenoType",
        "DefaultPhenotype",
        "DefaultPhenotypeID",
        "DefaultPheno",
    ];
    let present = inspection
        .columns
        .iter()
        .filter(|column| {
            aliases
                .iter()
                .any(|alias| column.eq_ignore_ascii_case(alias))
        })
        .collect::<Vec<_>>();
    if present.len() > 1 {
        return Err(candidate_error(
            "M2A-H2-R43-APPEARANCE-PHENOTYPE-AMBIGUOUS",
            "appearance.columns",
            "full appearance table contains multiple supported phenotype aliases",
        ));
    }
    if let Some(column) = present.first() {
        cells.push(TwoDaCellAssignmentV1 {
            column_name: (*column).clone(),
            value: TwoDaCellValueV1::Text {
                value: "0".to_owned(),
            },
        });
    }
    append_two_da_row_v1(
        source,
        &TwoDaAppendRequestV1 {
            schema_version: 1,
            cells,
        },
        &TwoDaLimitsV1::default(),
    )
    .map_err(|error| candidate_error(error.code, error.path, error.message))
}

fn require_appearance_cell(
    bytes: &[u8],
    row: u16,
    column: &str,
    expected: &str,
) -> Result<(), M0R33AnimatedDonorCandidateErrorV1> {
    let inspection = inspect_two_da_v2(bytes, &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let column_index = inspection
        .columns
        .iter()
        .position(|value| value.eq_ignore_ascii_case(column))
        .ok_or_else(|| {
            candidate_error(
                "M2A-R33-APPEARANCE-COLUMN",
                column,
                "required appearance column is absent",
            )
        })?;
    let row = read_two_da_row_v2(bytes, u32::from(row), &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    match row.cells.get(column_index) {
        Some(TwoDaCellValueV1::Text { value }) if value == expected => Ok(()),
        _ => Err(candidate_error(
            "M2A-R33-APPEARANCE-CELL",
            column,
            format!("expected exact value {expected}"),
        )),
    }
}

fn require_r34_appearance_cell(
    bytes: &[u8],
    row: u16,
    column: &str,
    expected: &str,
) -> Result<(), M0R34AnimatedDonorCandidateErrorV1> {
    let inspection = inspect_two_da_v2(bytes, &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let column_index = inspection
        .columns
        .iter()
        .position(|value| value.eq_ignore_ascii_case(column))
        .ok_or_else(|| {
            candidate_error(
                "M2A-R34-APPEARANCE-COLUMN",
                column,
                "required appearance column is absent",
            )
        })?;
    let row = read_two_da_row_v2(bytes, u32::from(row), &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    match row.cells.get(column_index) {
        Some(TwoDaCellValueV1::Text { value }) if value == expected => Ok(()),
        _ => Err(candidate_error(
            "M2A-R34-APPEARANCE-CELL",
            column,
            format!("expected exact value {expected}"),
        )),
    }
}

fn require_r35_appearance_cell(
    bytes: &[u8],
    row: u16,
    column: &str,
    expected: &str,
) -> Result<(), M0R35AnimatedDonorCandidateErrorV1> {
    let inspection = inspect_two_da_v2(bytes, &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let column_index = inspection
        .columns
        .iter()
        .position(|value| value.eq_ignore_ascii_case(column))
        .ok_or_else(|| {
            candidate_error(
                "M2A-R35-APPEARANCE-COLUMN",
                column,
                "required appearance column is absent",
            )
        })?;
    let row = read_two_da_row_v2(bytes, u32::from(row), &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    match row.cells.get(column_index) {
        Some(TwoDaCellValueV1::Text { value }) if value == expected => Ok(()),
        _ => Err(candidate_error(
            "M2A-R35-APPEARANCE-CELL",
            column,
            format!("expected exact value {expected}"),
        )),
    }
}

fn require_r36_appearance_cell(
    bytes: &[u8],
    row: u16,
    column: &str,
    expected: &str,
) -> Result<(), M0R36AnimatedDonorCandidateErrorV1> {
    let inspection = inspect_two_da_v2(bytes, &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let column_index = inspection
        .columns
        .iter()
        .position(|value| value.eq_ignore_ascii_case(column))
        .ok_or_else(|| {
            candidate_error(
                "M2A-R36-APPEARANCE-COLUMN",
                column,
                "required appearance column is absent",
            )
        })?;
    let row = read_two_da_row_v2(bytes, u32::from(row), &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    match row.cells.get(column_index) {
        Some(TwoDaCellValueV1::Text { value }) if value == expected => Ok(()),
        _ => Err(candidate_error(
            "M2A-R36-APPEARANCE-CELL",
            column,
            format!("expected exact value {expected}"),
        )),
    }
}

fn require_r37_appearance_cell(
    bytes: &[u8],
    row: u16,
    column: &str,
    expected: &str,
) -> Result<(), M0R37AnimatedDonorCandidateErrorV1> {
    let inspection = inspect_two_da_v2(bytes, &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let column_index = inspection
        .columns
        .iter()
        .position(|value| value.eq_ignore_ascii_case(column))
        .ok_or_else(|| {
            candidate_error(
                "M2A-R37-APPEARANCE-COLUMN",
                column,
                "required appearance column is absent",
            )
        })?;
    let row = read_two_da_row_v2(bytes, u32::from(row), &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    match row.cells.get(column_index) {
        Some(TwoDaCellValueV1::Text { value }) if value == expected => Ok(()),
        _ => Err(candidate_error(
            "M2A-R37-APPEARANCE-CELL",
            column,
            format!("expected exact value {expected}"),
        )),
    }
}

fn require_r38_appearance_cell(
    bytes: &[u8],
    row: u16,
    column: &str,
    expected: &str,
) -> Result<(), M0R38AnimatedDonorCandidateErrorV1> {
    let inspection = inspect_two_da_v2(bytes, &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let column_index = inspection
        .columns
        .iter()
        .position(|value| value.eq_ignore_ascii_case(column))
        .ok_or_else(|| {
            candidate_error(
                "M2A-R38-APPEARANCE-COLUMN",
                column,
                "required appearance column is absent",
            )
        })?;
    let row = read_two_da_row_v2(bytes, u32::from(row), &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    match row.cells.get(column_index) {
        Some(TwoDaCellValueV1::Text { value }) if value == expected => Ok(()),
        _ => Err(candidate_error(
            "M2A-R38-APPEARANCE-CELL",
            column,
            format!("expected exact value {expected}"),
        )),
    }
}

fn require_r39_appearance_cell(
    bytes: &[u8],
    row: u16,
    column: &str,
    expected: &str,
) -> Result<(), M0R39AnimatedDonorCandidateErrorV1> {
    let inspection = inspect_two_da_v2(bytes, &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let column_index = inspection
        .columns
        .iter()
        .position(|value| value.eq_ignore_ascii_case(column))
        .ok_or_else(|| {
            candidate_error(
                "M2A-R39-APPEARANCE-COLUMN",
                column,
                "required appearance column is absent",
            )
        })?;
    let row = read_two_da_row_v2(bytes, u32::from(row), &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    match row.cells.get(column_index) {
        Some(TwoDaCellValueV1::Text { value }) if value == expected => Ok(()),
        _ => Err(candidate_error(
            "M2A-R39-APPEARANCE-CELL",
            column,
            format!("expected exact value {expected}"),
        )),
    }
}

fn require_r40_appearance_cell(
    bytes: &[u8],
    row: u16,
    column: &str,
    expected: &str,
) -> Result<(), M0R40AnimatedDonorCandidateErrorV1> {
    let inspection = inspect_two_da_v2(bytes, &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let column_index = inspection
        .columns
        .iter()
        .position(|value| value.eq_ignore_ascii_case(column))
        .ok_or_else(|| {
            candidate_error(
                "M2A-R40-APPEARANCE-COLUMN",
                column,
                "required appearance column is absent",
            )
        })?;
    let row = read_two_da_row_v2(bytes, u32::from(row), &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    match row.cells.get(column_index) {
        Some(TwoDaCellValueV1::Text { value }) if value == expected => Ok(()),
        _ => Err(candidate_error(
            "M2A-R40-APPEARANCE-CELL",
            column,
            format!("expected exact value {expected}"),
        )),
    }
}

fn require_r41_appearance_cell(
    bytes: &[u8],
    row: u16,
    column: &str,
    expected: &str,
) -> Result<(), M0R41AnimatedDonorCandidateErrorV1> {
    let inspection = inspect_two_da_v2(bytes, &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let column_index = inspection
        .columns
        .iter()
        .position(|value| value.eq_ignore_ascii_case(column))
        .ok_or_else(|| {
            candidate_error(
                "M2A-R41-APPEARANCE-COLUMN",
                column,
                "required appearance column is absent",
            )
        })?;
    let row = read_two_da_row_v2(bytes, u32::from(row), &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    match row.cells.get(column_index) {
        Some(TwoDaCellValueV1::Text { value }) if value == expected => Ok(()),
        _ => Err(candidate_error(
            "M2A-R41-APPEARANCE-CELL",
            column,
            format!("expected exact value {expected}"),
        )),
    }
}

fn require_h2_r42_appearance_cell(
    bytes: &[u8],
    row: u16,
    column: &str,
    expected: &str,
) -> Result<(), H2R42VisibilityCandidateErrorV1> {
    let inspection = inspect_two_da_v2(bytes, &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let column_index = inspection
        .columns
        .iter()
        .position(|value| value.eq_ignore_ascii_case(column))
        .ok_or_else(|| {
            candidate_error(
                "M2A-H2-R42-APPEARANCE-COLUMN",
                column,
                "required appearance column is absent",
            )
        })?;
    let row = read_two_da_row_v2(bytes, u32::from(row), &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    match row.cells.get(column_index) {
        Some(TwoDaCellValueV1::Text { value }) if value == expected => Ok(()),
        _ => Err(candidate_error(
            "M2A-H2-R42-APPEARANCE-CELL",
            column,
            format!("expected exact value {expected}"),
        )),
    }
}

fn require_h2_r43_appearance_cell(
    bytes: &[u8],
    row: u16,
    column: &str,
    expected: &str,
) -> Result<(), H2R43H1RootLayoutCandidateErrorV1> {
    let inspection = inspect_two_da_v2(bytes, &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    let column_index = inspection
        .columns
        .iter()
        .position(|value| value.eq_ignore_ascii_case(column))
        .ok_or_else(|| {
            candidate_error(
                "M2A-H2-R43-APPEARANCE-COLUMN",
                column,
                "required appearance column is absent",
            )
        })?;
    let row = read_two_da_row_v2(bytes, u32::from(row), &TwoDaLimitsV1::default())
        .map_err(|error| candidate_error(error.code, error.path, error.message))?;
    match row.cells.get(column_index) {
        Some(TwoDaCellValueV1::Text { value }) if value == expected => Ok(()),
        _ => Err(candidate_error(
            "M2A-H2-R43-APPEARANCE-CELL",
            column,
            format!("expected exact value {expected}"),
        )),
    }
}

fn require_exact_input(
    bytes: &[u8],
    expected_length: u64,
    expected_sha256: &str,
    path: &str,
) -> Result<(), M0R33AnimatedDonorCandidateErrorV1> {
    if bytes.len() as u64 != expected_length || sha256_bytes(bytes) != expected_sha256 {
        return Err(candidate_error(
            "M2A-R33-INPUT-IDENTITY",
            path,
            "input bytes differ from the admitted r33 correction plan",
        ));
    }
    Ok(())
}

fn require_exact_r34_input(
    bytes: &[u8],
    expected_length: u64,
    expected_sha256: &str,
    path: &str,
) -> Result<(), M0R34AnimatedDonorCandidateErrorV1> {
    if bytes.len() as u64 != expected_length || sha256_bytes(bytes) != expected_sha256 {
        return Err(candidate_error(
            "M2A-R34-INPUT-IDENTITY",
            path,
            "input bytes differ from the admitted r34 correction plan",
        ));
    }
    Ok(())
}

fn require_exact_r35_input(
    bytes: &[u8],
    expected_length: u64,
    expected_sha256: &str,
    path: &str,
) -> Result<(), M0R35AnimatedDonorCandidateErrorV1> {
    if bytes.len() as u64 != expected_length || sha256_bytes(bytes) != expected_sha256 {
        return Err(candidate_error(
            "M2A-R35-INPUT-IDENTITY",
            path,
            "input bytes differ from the admitted r35 correction plan",
        ));
    }
    Ok(())
}

fn require_exact_r36_input(
    bytes: &[u8],
    expected_length: u64,
    expected_sha256: &str,
    path: &str,
) -> Result<(), M0R36AnimatedDonorCandidateErrorV1> {
    if bytes.len() as u64 != expected_length || sha256_bytes(bytes) != expected_sha256 {
        return Err(candidate_error(
            "M2A-R36-INPUT-IDENTITY",
            path,
            "input bytes differ from the admitted r36 correction plan",
        ));
    }
    Ok(())
}

fn require_exact_r37_input(
    bytes: &[u8],
    expected_length: u64,
    expected_sha256: &str,
    path: &str,
) -> Result<(), M0R37AnimatedDonorCandidateErrorV1> {
    if bytes.len() as u64 != expected_length || sha256_bytes(bytes) != expected_sha256 {
        return Err(candidate_error(
            "M2A-R37-INPUT-IDENTITY",
            path,
            "input bytes differ from the admitted r37 correction plan",
        ));
    }
    Ok(())
}

fn require_exact_r38_input(
    bytes: &[u8],
    expected_length: u64,
    expected_sha256: &str,
    path: &str,
) -> Result<(), M0R38AnimatedDonorCandidateErrorV1> {
    if bytes.len() as u64 != expected_length || sha256_bytes(bytes) != expected_sha256 {
        return Err(candidate_error(
            "M2A-R38-INPUT-IDENTITY",
            path,
            "input bytes differ from the admitted r38 correction plan",
        ));
    }
    Ok(())
}

fn require_exact_r39_input(
    bytes: &[u8],
    expected_length: u64,
    expected_sha256: &str,
    path: &str,
) -> Result<(), M0R39AnimatedDonorCandidateErrorV1> {
    if bytes.len() as u64 != expected_length || sha256_bytes(bytes) != expected_sha256 {
        return Err(candidate_error(
            "M2A-R39-INPUT-IDENTITY",
            path,
            "input bytes differ from the admitted r39 correction plan",
        ));
    }
    Ok(())
}

fn require_exact_r40_input(
    bytes: &[u8],
    expected_length: u64,
    expected_sha256: &str,
    path: &str,
) -> Result<(), M0R40AnimatedDonorCandidateErrorV1> {
    if bytes.len() as u64 != expected_length || sha256_bytes(bytes) != expected_sha256 {
        return Err(candidate_error(
            "M2A-R40-INPUT-IDENTITY",
            path,
            "input bytes differ from the admitted r40 correction plan",
        ));
    }
    Ok(())
}

fn require_exact_r41_input(
    bytes: &[u8],
    expected_length: u64,
    expected_sha256: &str,
    path: &str,
) -> Result<(), M0R41AnimatedDonorCandidateErrorV1> {
    if bytes.len() as u64 != expected_length || sha256_bytes(bytes) != expected_sha256 {
        return Err(candidate_error(
            "M2A-R41-INPUT-IDENTITY",
            path,
            "input bytes differ from the admitted r41 correction plan",
        ));
    }
    Ok(())
}

fn require_exact_h2_r42_input(
    bytes: &[u8],
    expected_length: u64,
    expected_sha256: &str,
    path: &str,
) -> Result<(), H2R42VisibilityCandidateErrorV1> {
    if bytes.len() as u64 != expected_length || sha256_bytes(bytes) != expected_sha256 {
        return Err(candidate_error(
            "M2A-H2-R42-INPUT-IDENTITY",
            path,
            "input bytes differ from the admitted H2 r42 correction plan",
        ));
    }
    Ok(())
}

fn require_exact_h2_r43_input(
    bytes: &[u8],
    expected_length: u64,
    expected_sha256: &str,
    path: &str,
) -> Result<(), H2R43H1RootLayoutCandidateErrorV1> {
    if bytes.len() as u64 != expected_length || sha256_bytes(bytes) != expected_sha256 {
        return Err(candidate_error(
            "M2A-H2-R43-INPUT-IDENTITY",
            path,
            "input bytes differ from the admitted H2 r43 correction plan",
        ));
    }
    Ok(())
}

fn resource_binding(resref: &str, bytes: &[u8]) -> M0R33ResourceBindingV1 {
    M0R33ResourceBindingV1 {
        resref: resref.to_owned(),
        byte_length: bytes.len() as u64,
        sha256: sha256_bytes(bytes),
    }
}

fn byte_identity(bytes: &[u8]) -> M6ByteIdentityV1 {
    M6ByteIdentityV1 {
        byte_length: bytes.len() as u64,
        sha256: sha256_bytes(bytes),
    }
}

fn sha256_bytes(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn frozen_retarget_drift_message(
    profile: &str,
    expected_sha256: &str,
    actual_sha256: &str,
    summary: &str,
) -> String {
    let cause = if matches!(
        profile,
        "r33" | "r34" | "r35" | "r36" | "r37" | "r38" | "r39"
    ) {
        "This historical SkinMesh lineage predates the mandatory r45 base-controller fix. The \
         current writer adds position/orientation controller keys (24 bytes) and data (36 bytes), \
         for an exact +60-byte core delta per SkinMesh. The archived artifact and its hash remain \
         immutable; this historical builder fails closed by design"
    } else {
        "This is unexplained lineage drift, not permission to update the frozen hash or allocate a \
         new rNN candidate"
    };
    format!(
        "{summary}; frozen {profile} model SHA-256 expected {expected_sha256}, generated \
         {actual_sha256}. {cause}. See documentation/audyt-bramek-pre-push-2026-07-27.md"
    )
}

fn candidate_error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> M0R33AnimatedDonorCandidateErrorV1 {
    M0R33AnimatedDonorCandidateErrorV1 {
        schema_version: 1,
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}
