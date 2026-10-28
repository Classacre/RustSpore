//! Logical effect-directory requests configured during application startup.
//!
//! SporeApp.exe `0xF47DC0` installs the list at `0x148E2A0` on the
//! object constructed by `0x7E7370`; its virtual setter is `0x7E5D80`.
//! `0x7E9730` requests each nonzero instance in list order using the type
//! and group below, followed by group-template enumeration. Evidence: REA
//! dossiers 433, 454, 458, 460, and 462 under
//! `target/rea/20261028-source-runtime-v6`, executable SHA-256
//! `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
//!
//! These are provider keys, not physical DBPF keys. The caller supplies the
//! configuration byte; this plan does not perform loading, decide package
//! precedence, or assert that any request succeeds. The list is installed by
//! general application startup and is not specific to the Cell stage.
//! A single supplied byte assumes the native configuration value remains
//! unchanged across its separate reads during this projected sequence.

pub const EFFECT_DIRECTORY_TYPE: u32 = 0xea51_18b0;
pub const STARTUP_DIRECTORY_INSTANCES: [u32; 5] = [
    0x4cf9_b596,
    0x2c97_8da4,
    0xbafc_5ca1,
    0x5f84_8353,
    0xc119_5706,
];
pub const STARTUP_ENUMERATION_GROUP_TEMPLATES: [u32; 4] = [
    0x4080_4300,
    0x4060_4300,
    0x4088_4300,
    0x408a_4300,
];

/// The three logical words passed to the resource provider.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderKey {
    pub instance: u32,
    pub resource_type: u32,
    pub group: u32,
}

/// Build the ordered explicit-list requests for the supplied configuration byte.
pub fn startup_directory_requests(config_byte: u8) -> [ProviderKey; 5] {
    startup_directory_requests_with_config_bytes([config_byte; 5])
}

/// Supply the configuration byte observed at each individual source read.
pub fn startup_directory_requests_with_config_bytes(config: [u8; 5]) -> [ProviderKey; 5] {
    std::array::from_fn(|index| ProviderKey {
        instance: STARTUP_DIRECTORY_INSTANCES[index],
        resource_type: EFFECT_DIRECTORY_TYPE,
        group: 0x4045_0200 | u32::from(config[index]),
    })
}

/// Ordered logical group filters used to enumerate further effect directories.
/// The provider determines enumeration results and their order.
pub fn startup_enumeration_groups(config_byte: u8) -> [u32; 4] {
    startup_enumeration_groups_with_config_bytes([config_byte; 4])
}

/// Supply the configuration byte observed at each enumeration-template read.
pub fn startup_enumeration_groups_with_config_bytes(config: [u8; 4]) -> [u32; 4] {
    std::array::from_fn(|index| {
        (STARTUP_ENUMERATION_GROUP_TEMPLATES[index] & !0xff) | u32::from(config[index])
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_observed_order_and_logical_key_fields() {
        let requests = startup_directory_requests(1);
        assert_eq!(requests.map(|key| key.instance), [
            0x4cf9_b596, 0x2c97_8da4, 0xbafc_5ca1, 0x5f84_8353, 0xc119_5706,
        ]);
        assert!(requests.iter().all(|key| key.resource_type == 0xea51_18b0
            && key.group == 0x4045_0201));
    }

    #[test]
    fn caller_byte_controls_only_the_low_group_byte() {
        for (byte, expected) in [(0, 0x4045_0200), (2, 0x4045_0202), (255, 0x4045_02ff)] {
            assert!(startup_directory_requests(byte).iter().all(|key| key.group == expected));
        }
    }

    #[test]
    fn enumeration_group_filters_keep_the_startup_template_order() {
        assert_eq!(startup_enumeration_groups(2), [
            0x4080_4302, 0x4060_4302, 0x4088_4302, 0x408a_4302,
        ]);
        assert_eq!(startup_enumeration_groups(255), [
            0x4080_43ff, 0x4060_43ff, 0x4088_43ff, 0x408a_43ff,
        ]);
    }

    #[test]
    fn changing_configuration_is_preserved_at_each_read() {
        assert_eq!(startup_directory_requests_with_config_bytes([0, 1, 2, 3, 4]).map(|key| key.group),
            [0x4045_0200, 0x4045_0201, 0x4045_0202, 0x4045_0203, 0x4045_0204]);
        assert_eq!(startup_enumeration_groups_with_config_bytes([4, 3, 2, 1]),
            [0x4080_4304, 0x4060_4303, 0x4088_4302, 0x408a_4301]);
    }
}
