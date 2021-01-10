//! Utilities to work with Maven project files and repositories implemented in
//! Rust.
//!
//! # Quick Start
//!
//! ```
//! use maven_toolbox::{default_impl::*, *};
//!
//! // the artifact's GAV
//! let artifact = ArtifactFqn::pom(
//!     "com.walmartlabs.concord.plugins.basic",
//!     "smtp-tasks",
//!     "1.76.1",
//! );
//!
//! let mut resolver = Resolver::default();
//!
//! // default implementations, you can plug in your own
//! let url_fetcher = DefaultUrlFetcher {};
//! let pom_parser = DefaultPomParser {};
//!
//! let project = resolver
//!     .build_effective_pom(&artifact, &url_fetcher, &pom_parser)
//!     .unwrap();
//! ```
//!
//! The `build_effective_pom` call requires a [`UrlFetcher`] and a [`PomParser`].
//! The [`default_impl`] module provides minimal implementations of of those
//! traits.

use std::collections::HashMap;

#[cfg(feature = "default-impl")]
pub mod default_impl;

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash)]
pub struct ArtifactFqn {
    pub group_id: Option<String>,
    pub artifact_id: Option<String>,
    pub version: Option<String>,
    pub packaging: Option<String>,
    pub classifier: Option<String>,
}

impl ArtifactFqn {
    pub fn new(
        group_id: &str,
        artifact_id: &str,
        version: &str,
        packaging: &str,
        classifier: &str,
    ) -> Self {
        ArtifactFqn {
            group_id: Some(group_id.to_owned()),
            artifact_id: Some(artifact_id.to_owned()),
            version: Some(version.to_owned()),
            packaging: Some(packaging.to_owned()),
            classifier: Some(classifier.to_owned()),
            ..Default::default()
        }
    }

    pub fn pom(group_id: &str, artifact_id: &str, version: &str) -> Self {
        ArtifactFqn {
            group_id: Some(group_id.to_owned()),
            artifact_id: Some(artifact_id.to_owned()),
            version: Some(version.to_owned()),
            packaging: Some("pom".to_owned()),
            ..Default::default()
        }
    }

    pub fn interpolate(&self, properties: &HashMap<String, String>) -> Self {
        // TODO other fields
        ArtifactFqn {
            version: self
                .version
                .clone()
                .filter(|v| v.contains("${"))
                .map(|mut s| {
                    if let Some(start) = s.find("${") {
                        if let Some(end) = s[start..].find("}") {
                            let expr = s[start + 2..end].to_owned();
                            if let Some(v) = properties.get(&expr) {
                                s.replace_range(start..end + 1, v);
                            }
                        }
                    }
                    s
                })
                .or_else(|| self.version.clone()),
            ..self.clone()
        }
    }

    pub fn with_packaging(&self, packaging: &str) -> Self {
        ArtifactFqn {
            packaging: Some(packaging.to_owned()),
            ..self.clone()
        }
    }

    pub fn same_ga(&self, other: &Self) -> bool {
        self.group_id == other.group_id && self.artifact_id == other.artifact_id
    }

    pub fn normalize(self, parent: &Self, default_packaging: &str) -> Self {
        ArtifactFqn {
            group_id: self.group_id.or_else(|| parent.group_id.clone()),
            artifact_id: self.artifact_id.or_else(|| parent.artifact_id.clone()),
            version: self.version.or_else(|| parent.version.clone()),
            packaging: self
                .packaging
                .or_else(|| Some(default_packaging.to_owned())),
            ..Default::default()
        }
    }
}

impl std::fmt::Display for ArtifactFqn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let def = "?".to_owned();
        write!(
            f,
            "{}:{}:{}:{}:{}",
            self.group_id.as_ref().unwrap_or(&def),
            self.artifact_id.as_ref().unwrap_or(&def),
            self.version.as_ref().unwrap_or(&def),
            self.packaging.as_ref().unwrap_or(&def),
            self.classifier.as_ref().unwrap_or(&def)
        )
    }
}

#[derive(Default, Debug, Clone)]
pub struct Dependency {
    pub artifact_fqn: ArtifactFqn,
    pub scope: Option<String>,
}

impl Dependency {
    pub fn get_key(&self) -> DependencyKey {
        DependencyKey {
            group_id: self.artifact_fqn.group_id.clone(),
            artifact_id: self.artifact_fqn.artifact_id.clone(),
        }
    }

    pub fn normalize(self, parent_id: &ArtifactFqn, default_packaging: &str) -> Self {
        Dependency {
            artifact_fqn: self.artifact_fqn.normalize(parent_id, default_packaging),
            scope: self.scope.or_else(|| Some("compile".to_owned())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Parent {
    pub artifact_fqn: ArtifactFqn,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DependencyKey {
    pub group_id: Option<String>,
    pub artifact_id: Option<String>,
}

impl std::fmt::Display for DependencyKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let def = "?".to_owned();
        write!(
            f,
            "{}:{}",
            self.group_id.as_ref().unwrap_or(&def),
            self.artifact_id.as_ref().unwrap_or(&def)
        )
    }
}

#[derive(Debug, Clone)]
pub struct DependencyManagement {
    pub dependencies: HashMap<DependencyKey, Dependency>,
}

#[derive(Debug, Clone)]
pub struct Project {
    pub parent: Option<Parent>,
    pub artifact_fqn: ArtifactFqn,
    pub dependency_management: Option<DependencyManagement>,
    pub dependencies: HashMap<DependencyKey, Dependency>,
    pub properties: HashMap<String, String>,
}

pub struct Repository {
    pub base_url: String,
}

#[derive(Debug)]
pub enum ErrorKind {
    ClientError,
    // RepositoryError,
}

#[derive(Debug)]
pub struct ResolverError {
    pub kind: ErrorKind,
    pub msg: String,
}

impl ResolverError {
    pub fn missing_parameter<D: std::fmt::Display>(fqn: &ArtifactFqn, field_name: &D) -> Self {
        ResolverError {
            kind: ErrorKind::ClientError,
            msg: format!("'{}' is missing from {}", field_name, fqn),
        }
    }

    pub fn invalid_data(details: &str) -> Self {
        ResolverError {
            kind: ErrorKind::ClientError,
            msg: format!("Invalid input data: {}", details),
        }
    }

    pub fn cant_resolve(artifact_id: &ArtifactFqn, cause: &str) -> Self {
        ResolverError {
            kind: ErrorKind::ClientError,
            msg: format!("Can't resolve {:?}: {}", artifact_id, cause),
        }
    }
}

pub trait UrlFetcher {
    fn fetch(&self, url: &str) -> Result<String, ResolverError>;
}

pub trait PomParser {
    fn parse(&self, input: String) -> Result<Project, ResolverError>;
}

pub struct Resolver {
    pub repository: Repository,
    pub project_cache: HashMap<ArtifactFqn, Project>,
}

impl Default for Resolver {
    fn default() -> Self {
        Resolver {
            repository: Repository {
                base_url: "https://repo.maven.apache.org/maven2".into(),
            },
            project_cache: HashMap::new(),
        }
    }
}

fn normalize_gavs(
    dependencies: HashMap<DependencyKey, Dependency>,
    parent_fqn: &ArtifactFqn,
    default_packaging: &str,
) -> HashMap<DependencyKey, Dependency> {
    dependencies
        .into_iter()
        .map(|(_, dep)| {
            let dep = dep.normalize(parent_fqn, default_packaging);
            (dep.get_key(), dep)
        })
        .collect()
}

impl Resolver {
    pub fn create_url(&self, id: &ArtifactFqn) -> Result<String, ResolverError> {
        // a little helper
        fn require<'a, F, D>(
            id: &'a ArtifactFqn,
            f: F,
            field_name: &D,
        ) -> Result<&'a String, ResolverError>
        where
            F: Fn(&ArtifactFqn) -> Option<&String>,
            D: std::fmt::Display,
        {
            f(id).ok_or_else(|| ResolverError::missing_parameter(id, field_name))
        }

        let group_id = require(id, |id| id.group_id.as_ref(), &"groupId")?;
        let artifact_id = require(id, |id| id.artifact_id.as_ref(), &"artifactId")?;
        let version = require(id, |id| id.version.as_ref(), &"version")?;
        let packaging = require(id, |id| id.packaging.as_ref(), &"packaging")?;

        let mut url = format!(
            "{}/{}/{}/{}/{}-{}",
            self.repository.base_url,
            group_id.replace(".", "/"),
            artifact_id,
            version,
            artifact_id,
            version
        );

        if let Some(classifier) = &id.classifier {
            url += &format!("-{}", classifier);
        }

        url += &format!(".{}", packaging);

        Ok(url)
    }

    pub fn build_effective_pom<UF, P>(
        &mut self,
        project_id: &ArtifactFqn,
        url_fetcher: &UF,
        pom_parser: &P,
    ) -> Result<Project, ResolverError>
    where
        UF: UrlFetcher,
        P: PomParser,
    {
        log::debug!("building an effective pom for {}", project_id);

        let project_id = &project_id.with_packaging("pom");

        let mut project = self.fetch_project(project_id, url_fetcher, pom_parser)?;

        if let Some(version) = &project_id.version {
            project
                .properties
                .insert("project.version".to_owned(), version.clone());
        }

        // merge in the dependencies from the parent POM
        if let Some(parent) = &project.parent {
            let parent_project =
                self.build_effective_pom(&parent.artifact_fqn, url_fetcher, pom_parser)?;

            log::trace!("got a parent POM: {}", parent_project.artifact_fqn);

            let extra_deps = parent_project
                .dependencies
                .into_iter()
                .filter(|(dep_key, _)| !project.dependencies.contains_key(dep_key))
                .collect::<HashMap<_, _>>();

            project.dependencies.extend(extra_deps);
        }

        if let Some(mut project_dm) = project.dependency_management.clone() {
            for (_, dep) in &mut project_dm.dependencies {
                dep.artifact_fqn = dep.artifact_fqn.interpolate(&project.properties);
            }

            let boms: Vec<Dependency> = project_dm
                .dependencies
                .iter()
                .filter(|(_, dep)| dep.scope.as_deref() == Some("import"))
                .map(|(_, dep)| dep.clone())
                .collect();

            for bom in boms {
                log::trace!("got a BOM artifact: {}", bom.artifact_fqn);

                // TODO add protection against infinite recursion
                let bom_project =
                    self.build_effective_pom(&bom.artifact_fqn, url_fetcher, pom_parser)?;

                if let Some(DependencyManagement {
                    dependencies: bom_deps,
                }) = bom_project.dependency_management
                {
                    project_dm.dependencies.extend(bom_deps);
                }
            }
        };

        Ok(project)
    }

    pub fn fetch_project<UF, P>(
        &mut self,
        project_id: &ArtifactFqn,
        url_fetcher: &UF,
        pom_parser: &P,
    ) -> Result<Project, ResolverError>
    where
        UF: UrlFetcher,
        P: PomParser,
    {
        // we're looking only for POMs here
        let project_id = project_id.with_packaging("pom");

        // check the cache first
        if let Some(cached_project) = self.project_cache.get(&project_id) {
            log::debug!("returning from cache {}...", project_id);
            return Ok(cached_project.clone());
        }

        // grab the remote POM
        let url = self.create_url(&project_id)?;

        log::debug!("fetching {}...", url);
        let text = url_fetcher.fetch(&url)?;

        // parse the POM - it will be our "root" project
        // TODO handle multiple "roots"
        let mut project = pom_parser.parse(text)?;

        // make sure the packaging type is set to "pom"
        let mut project_id = project.artifact_fqn.with_packaging("pom");

        // TODO consider moving this to build_effective_pom
        // update the parent and fill-in the project's missing properties using the parent's GAV
        if let Some(parent) = &project.parent {
            let parent_fqn = parent.artifact_fqn.with_packaging("pom");

            project_id = project_id.normalize(&parent_fqn, "pom");

            // normalize dependency GAVs
            project.dependencies = normalize_gavs(project.dependencies, &parent_fqn, "jar");
            project.dependency_management = project.dependency_management.map(|mut dm| {
                dm.dependencies = normalize_gavs(dm.dependencies, &parent_fqn, "jar");
                dm
            });

            // save the updated FQN
            project.parent = project.parent.map(|mut p| {
                p.artifact_fqn = parent_fqn;
                p
            });
        }

        // save the updated FQN
        project.artifact_fqn = project_id.clone();

        // we're going to save all parsed projects into a HashMap
        // as a "cache"
        log::trace!("caching {}", project_id);
        self.project_cache.insert(project_id, project.clone());

        Ok(project)
    }
}
