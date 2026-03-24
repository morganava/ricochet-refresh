// std
use std::path::Path;

// extern
use tor_interface::tor_crypto::{Ed25519PrivateKey, Ed25519PublicKey, V3OnionServiceId};

// internal
#[cfg(feature = "v3-profile")]
use rico_profile::v3;
use rico_profile::v4;

#[test]
fn test_construction() -> anyhow::Result<()> {
    let mut path = std::env::temp_dir();
    path.push("test_construction.ricochet-profile");
    if Path::exists(&path) {
        std::fs::remove_file(&path)?;
    }

    println!("path: {path:?}");

    let mut profile = v4::profile::Profile::new(&path, "hunter42")?;

    assert_eq!(profile.get_version()?, v4::profile::Version::LATEST);

    Ok(())
}

#[test]
#[cfg(feature = "v3-profile")]
fn test_legacy_import() -> anyhow::Result<()> {
    use std::collections::BTreeMap;
    // (json!({
    //     "identity" : {
    //         "privateKey" : "ED25519-V3:YLj7W9DouVzO4a1yPY1a3yIT7Hv3FYVwg/d3sE/h0F9oFE0vGhXtE61kFNjq6MhstvxaNGCRXmVm+mta2nwfgg=="
    //     },
    //     "users" : {
    //         "um7kahbtdqiijlohv3cfsbi7iqo4bvidngshr6zshi6rxseu3bbiriid" : {"nickname" : "alice", "type" : "allowed"},
    //         "kndfzlfstthybcnf62brkk5dn2ypqlzyra5srheqenpvmesoizodihad" : {"nickname" : "bridgette", "type" : "requesting"},
    //         "mj4kpnujlesrslrmtqqbyu3yntr7macr6sbtmime5mktl272mdcn36yd" : {"nickname" : "claire", "type" : "blocked"},
    //         "arn2oq6qp2gvcicolecju5x44x74zv56llno6cujvnxvtlcxyjhwlvid" : {"nickname" : "danielle", "type" : "pending"},
    //         "zdqen2zfqcx25fcf4youogtlfjodwq6vx2u44pfr2vtjpktwbirm44yd" : {"nickname" : "evelyn", "type" : "rejected"},
    //     },
    // }),
    let v3_profile = v3::profile::Profile{
        private_key: Ed25519PrivateKey::from_key_blob_legacy("ED25519-V3:YLj7W9DouVzO4a1yPY1a3yIT7Hv3FYVwg/d3sE/h0F9oFE0vGhXtE61kFNjq6MhstvxaNGCRXmVm+mta2nwfgg==")?,
        users: BTreeMap::from([
            (V3OnionServiceId::from_string("um7kahbtdqiijlohv3cfsbi7iqo4bvidngshr6zshi6rxseu3bbiriid")?,
                v3::profile::User{
                    nickname: "alice".to_string(),
                    user_type: v3::profile::UserType::Allowed}),
            (V3OnionServiceId::from_string("kndfzlfstthybcnf62brkk5dn2ypqlzyra5srheqenpvmesoizodihad")?,
                v3::profile::User{
                    nickname: "bridgette".to_string(),
                    user_type: v3::profile::UserType::Requesting}),
            (V3OnionServiceId::from_string("mj4kpnujlesrslrmtqqbyu3yntr7macr6sbtmime5mktl272mdcn36yd")?,
                v3::profile::User{
                    nickname: "claire".to_string(),
                    user_type: v3::profile::UserType::Blocked}),
            (V3OnionServiceId::from_string("arn2oq6qp2gvcicolecju5x44x74zv56llno6cujvnxvtlcxyjhwlvid")?,
                v3::profile::User{
                    nickname: "danielle".to_string(),
                    user_type: v3::profile::UserType::Pending}),
            (V3OnionServiceId::from_string("zdqen2zfqcx25fcf4youogtlfjodwq6vx2u44pfr2vtjpktwbirm44yd")?,
                v3::profile::User{
                    nickname: "evelyn".to_string(),
                    user_type: v3::profile::UserType::Rejected}),
        ]),
    };
    let mut path = std::env::temp_dir();
    path.push("test_legacy_import.ricochet-profile");
    if Path::exists(&path) {
        std::fs::remove_file(&path)?;
    }

    println!("path: {path:?}");

    let v4_profile =
        v4::profile::Profile::new_from_v3_profile(v3_profile, "morgan", &path, "hunter42")?;
    Ok(())
}
