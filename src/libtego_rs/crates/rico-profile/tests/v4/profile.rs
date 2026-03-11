// std
use std::path::Path;

// internal
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
