use maven_toolbox::default_impl::*;
use maven_toolbox::*;

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
#[cfg(feature = "default-impl")]
fn test_build_effective_pom() {
    init();

    let root = ArtifactFqn::pom(
        "com.walmartlabs.concord.plugins.basic",
        "smtp-tasks",
        "1.76.1",
    );

    let mut resolver = Resolver::default();
    let url_fetcher = DefaultUrlFetcher {};
    let pom_parser = DefaultPomParser {};

    let project = resolver
        .build_effective_pom(&root, &url_fetcher, &pom_parser)
        .unwrap();

    assert!(project.parent.is_some());

    let mut deps = project
        .dependencies
        .into_iter()
        .map(|(_, dep)| dep)
        .collect::<Vec<_>>();

    deps.sort_by(|a, b| {
        let a = a.get_key();
        let b = b.get_key();
        a.group_id.cmp(&b.group_id)
    });

    for dep in deps {
        println!("{:?}", dep);
    }
}

#[test]
#[cfg(feature = "default-impl")]
fn test_fetch_project() {
    init();

    let root = ArtifactFqn::pom(
        "com.walmartlabs.concord.plugins.basic",
        "smtp-tasks",
        "1.76.1",
    );

    let mut resolver = Resolver::default();
    let url_fetcher = DefaultUrlFetcher {};
    let pom_parser = DefaultPomParser {};

    let project = resolver
        .fetch_project(&root, &url_fetcher, &pom_parser)
        .unwrap();

    assert!(project.parent.is_some());
    assert_eq!(1, resolver.project_cache.len());

    let parent = resolver
        .fetch_project(
            &project.parent.unwrap().artifact_fqn.with_packaging("pom"),
            &url_fetcher,
            &pom_parser,
        )
        .unwrap();

    assert_eq!("parent", parent.artifact_fqn.artifact_id.unwrap());
    assert_eq!(2, resolver.project_cache.len());
}
