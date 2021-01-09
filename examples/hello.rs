use maven_toolbox::{default_impl::*, *};

fn main() {
    let artifact = ArtifactFqn::pom(
        "com.walmartlabs.concord.plugins.basic",
        "smtp-tasks",
        "1.76.1",
    );

    println!("Resolving {}...", artifact);

    let mut resolver = Resolver::default();
    let url_fetcher = DefaultUrlFetcher {};
    let pom_parser = DefaultPomParser {};

    let project = resolver
        .build_effective_pom(&artifact, &url_fetcher, &pom_parser)
        .unwrap();

    // print out all dependencies with "compile" scope
    project
        .dependencies
        .values()
        .filter(|dep| dep.scope.as_deref() == Some("compile"))
        .for_each(|dep| {
            println!("{}", dep.artifact_fqn);
        });
}
