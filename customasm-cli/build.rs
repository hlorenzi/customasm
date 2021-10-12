extern crate vergen;


use vergen::{generate_cargo_keys, ConstantsFlags};


fn main()
{
    let mut flags = ConstantsFlags::empty();
    flags.toggle(ConstantsFlags::REBUILD_ON_HEAD_CHANGE);
    flags.toggle(ConstantsFlags::SEMVER_LIGHTWEIGHT);
    flags.toggle(ConstantsFlags::COMMIT_DATE);
    flags.toggle(ConstantsFlags::TARGET_TRIPLE);

    generate_cargo_keys(flags).expect("Unable to generate the cargo keys!");
}
