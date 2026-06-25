use std::borrow::Cow;
use crate::manifest::{TemplateDefinition, TemplateSource};

pub const DEFAULT_GIT_URL: &str = "git@github.com:Bechma/cf-template-rust.git";
pub const DEFAULT_BRANCH: &str = "main";

const fn source_path(folder: &str) -> TemplateSource<'_> {
    TemplateSource::Git {
        url: Cow::Borrowed(DEFAULT_GIT_URL),
        revision: None,
        tag: None,
        branch: Some(Cow::Borrowed(DEFAULT_BRANCH)),
        subfolder: Some(Cow::Borrowed(folder)),
    }
}

pub const BUILTIN_TEMPLATES_GEARS: &[TemplateDefinition] = &[
    TemplateDefinition {
        name: Cow::Borrowed("pokemon-demo"),
        description: Cow::Borrowed("pokemon demo that shows a rest api with infra capabilities(external requests) and a database"),
        source: source_path("Modules/api-db-handler"),
    },
    TemplateDefinition {
        name: Cow::Borrowed("api-fetcher"),
        description: Cow::Borrowed("simple gear example that only handles external requests"),
        source: source_path("Modules/api-fetcher"),
    },
    TemplateDefinition {
        name: Cow::Borrowed("api-gateway"),
        description: Cow::Borrowed("simple gateway required for the system to have rest api. Prefer to use the system module cf-gears-api-gateway"),
        source: source_path("Modules/api-gateway"),
    },
    TemplateDefinition {
        name: Cow::Borrowed("background-worker"),
        description: Cow::Borrowed("simple gear example that spawn a background worker"),
        source: source_path("Modules/background-worker"),
    },
    TemplateDefinition {
        name: Cow::Borrowed("db-handler"),
        description: Cow::Borrowed("simple gear example that only handles database interactions"),
        source: source_path("Modules/db-handler"),
    },
    TemplateDefinition {
        name: Cow::Borrowed("rest-api"),
        description: Cow::Borrowed("simple gear example that only handles rest api calls"),
        source: source_path("Modules/rest-api"),
    },
];

pub const BUILTIN_TEMPLATES_WORKSPACE: &[TemplateDefinition] = &[
    TemplateDefinition {
        name: Cow::Borrowed("basic-init"),
        description: Cow::Borrowed("skeleton to initialize a new gears project"),
        source: source_path("Init"),
    },
];
