use sr_primitives::BuildStorage as _;
use srml_support::storage::{StorageMap as _, StorageValue as _};
use substrate_primitives::{crypto::Pair as _, ed25519};

use radicle_registry_runtime::{
    registry::{self, Project, ProjectId, RegisterProjectParams},
    GenesisConfig, Origin, Runtime,
};

type Registry = registry::Module<Runtime>;

#[test]
fn register_project() {
    run(|| {
        let signer = key_pair_from_string("Alice");
        let origin = Origin::signed(signer.public());
        let project_id = ProjectId::random();
        Registry::register_project(
            origin,
            RegisterProjectParams {
                id: project_id,
                name: "NAME".to_string(),
                description: "DESCRIPTION".to_string(),
                img_url: "IMG_URL".to_string(),
            },
        )
        .unwrap();

        let project = registry::store::Projects::get(project_id).unwrap();
        assert_eq!(
            project,
            Project {
                id: project_id,
                name: "NAME".to_string(),
                description: "DESCRIPTION".to_string(),
                img_url: "IMG_URL".to_string(),
                members: vec![signer.public()],
            }
        );

        let project_ids = registry::store::ProjectIds::get();
        assert_eq!(project_ids, vec![project_id]);
    });
}

fn key_pair_from_string(value: impl AsRef<str>) -> ed25519::Pair {
    ed25519::Pair::from_string(format!("//{}", value.as_ref()).as_str(), None).unwrap()
}

/// Run runtime code with test externalities
fn run(f: impl FnOnce() -> ()) {
    let genesis_config = GenesisConfig {
        srml_aura: None,
        srml_balances: None,
        srml_sudo: None,
        system: None,
    };
    let mut test_ext = sr_io::TestExternalities::new(genesis_config.build_storage().unwrap());
    test_ext.execute_with(f)
}
