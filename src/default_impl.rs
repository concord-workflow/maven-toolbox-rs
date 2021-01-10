use crate::*;
pub struct DefaultUrlFetcher {}

impl UrlFetcher for DefaultUrlFetcher {
    fn fetch(&self, url: &str) -> Result<String, ResolverError> {
        let text = ureq::get(url.into()).call().unwrap().into_string();
        Ok(text.unwrap())
    }
}

fn node<'a, 'input: 'a>(
    parent: &'input roxmltree::Node,
    tag_name: &'a str,
) -> Option<roxmltree::Node<'a, 'input>> {
    parent
        .children()
        .find(|child| child.is_element() && child.has_tag_name(tag_name))
}

fn node_text<'a, 'input: 'a>(parent: &'input roxmltree::Node, tag_name: &'a str) -> Option<String> {
    let n = node(parent, tag_name)?;
    n.text().map(|t| t.to_owned())
}

fn parse_gav(n: &roxmltree::Node) -> ArtifactFqn {
    ArtifactFqn {
        group_id: node_text(n, "groupId"),
        artifact_id: node_text(n, "artifactId"),
        version: node_text(n, "version"),
        packaging: node_text(n, "type").or_else(|| node_text(n, "packaging")), // TODO dirty hack
        classifier: node_text(n, "classifier"),
    }
}

fn parse_parent(n: &roxmltree::Node) -> Option<Parent> {
    let n = node(n, "parent")?;
    Some(Parent {
        artifact_fqn: parse_gav(&n),
    })
}

fn parse_dependency(n: &roxmltree::Node) -> Dependency {
    Dependency {
        artifact_fqn: parse_gav(n),
        scope: node_text(n, "scope"),
    }
}

fn parse_dependencies(n: &roxmltree::Node) -> HashMap<DependencyKey, Dependency> {
    match node(n, "dependencies") {
        Some(n) => n
            .children()
            .filter(|child| child.is_element() && child.has_tag_name("dependency"))
            .map(|child| {
                let dep = parse_dependency(&child);
                (dep.get_key(), dep)
            })
            .collect(),
        _ => HashMap::new(),
    }
}

fn parse_dependency_management(n: &roxmltree::Node) -> Option<DependencyManagement> {
    node(n, "dependencyManagement").map(|n| DependencyManagement {
        dependencies: parse_dependencies(&n),
    })
}

pub struct DefaultPomParser {}

impl PomParser for DefaultPomParser {
    fn parse(&self, input: String) -> Result<Project, ResolverError> {
        let doc = roxmltree::Document::parse(&input).unwrap();

        let n = doc.root();
        let project_node = node(&n, "project")
            .ok_or_else(|| ResolverError::invalid_data("invalid XML content, no <project> tag"))?;

        Ok(Project {
            artifact_fqn: parse_gav(&project_node),
            parent: parse_parent(&project_node),
            dependency_management: parse_dependency_management(&project_node),
            dependencies: parse_dependencies(&project_node),
            properties: HashMap::new(),
        })
    }
}
