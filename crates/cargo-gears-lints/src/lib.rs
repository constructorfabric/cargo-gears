#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_lint;
extern crate rustc_session;

pub(crate) const LIBRARY_NAME: &str = env!("CARGO_PKG_NAME");

mod lint_utils;

mod de01_domain_layer {
    pub(crate) mod de0101_no_serde_in_domain;
    pub(crate) mod de0102_no_toschema_in_domain;
    pub(crate) mod de0104_no_api_dto_in_domain;
}

mod de02_api_layer {
    pub(crate) mod de0201_dtos_only_in_api_rest;
    pub(crate) mod de0202_dtos_not_referenced_outside_api;
    pub(crate) mod de0203_dtos_must_use_api_dto;
    pub(crate) mod de0204_dtos_must_have_toschema_derive;
}

mod de03_domain_layer {
    pub(crate) mod de0301_no_infra_in_domain;
    pub(crate) mod de0308_no_http_in_domain;
}

mod de05_client_layer {
    pub(crate) mod de0503_plugin_client_suffix;
    pub(crate) mod de0504_client_versioning;
}

mod de07_security {
    pub(crate) mod de0706_no_direct_sqlx;
    pub(crate) mod de0707_drop_zeroize;
}

mod de08_rest_api_conventions {
    pub(crate) mod de0801_api_endpoint_version;
    pub(crate) mod de0802_use_odata_ext;
    pub(crate) mod de0803_api_snake_case;
}

mod de09_gts_layer {
    pub(crate) mod de0901_gts_string_pattern;
    pub(crate) mod de0902_no_schema_for_on_gts_structs;
}

mod de11_testing {
    pub(crate) mod de1101_tests_in_separate_files;
}

mod de12_documentation {
    pub(crate) mod de1201_docs_rs_all_features;
}

mod de13_common_patterns {
    pub(crate) mod de1301_no_print_macros;
    pub(crate) mod de1302_error_from_to_string;
    pub(crate) mod de1303_no_primitive_type_alias;
}

dylint_linting::dylint_library!();

#[unsafe(no_mangle)]
pub fn register_lints(sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    dylint_linting::init_config(sess);

    lint_store.register_lints(&[
        de01_domain_layer::de0101_no_serde_in_domain::DE0101_NO_SERDE_IN_CONTRACT,
        de01_domain_layer::de0102_no_toschema_in_domain::DE0102_NO_TOSCHEMA_IN_CONTRACT,
        de01_domain_layer::de0104_no_api_dto_in_domain::DE0104_NO_API_DTO_IN_CONTRACT,
        de02_api_layer::de0201_dtos_only_in_api_rest::DE0201_DTOS_ONLY_IN_API_REST,
        de02_api_layer::de0202_dtos_not_referenced_outside_api::DE0202_DTOS_NOT_REFERENCED_OUTSIDE_API,
        de02_api_layer::de0203_dtos_must_use_api_dto::DE0203_DTOS_MUST_USE_API_DTO,
        de02_api_layer::de0204_dtos_must_have_toschema_derive::DE0204_DTOS_MUST_HAVE_TOSCHEMA_DERIVE,
        de03_domain_layer::de0301_no_infra_in_domain::DE0301_NO_INFRA_IN_DOMAIN,
        de03_domain_layer::de0308_no_http_in_domain::DE0308_NO_HTTP_IN_DOMAIN,
        de05_client_layer::de0503_plugin_client_suffix::DE0503_PLUGIN_CLIENT_SUFFIX,
        de05_client_layer::de0504_client_versioning::DE0504_CLIENT_VERSIONING,
        de07_security::de0706_no_direct_sqlx::DE0706_NO_DIRECT_SQLX,
        de07_security::de0707_drop_zeroize::DE0707_DROP_ZEROIZE,
        de08_rest_api_conventions::de0801_api_endpoint_version::DE0801_API_ENDPOINT_VERSION,
        de08_rest_api_conventions::de0802_use_odata_ext::DE0802_USE_ODATA_EXT,
        de08_rest_api_conventions::de0803_api_snake_case::DE0803_API_SNAKE_CASE,
        de09_gts_layer::de0901_gts_string_pattern::DE0901_GTS_STRING_PATTERN,
        de09_gts_layer::de0902_no_schema_for_on_gts_structs::DE0902_NO_SCHEMA_FOR_ON_GTS_STRUCTS,
        de11_testing::de1101_tests_in_separate_files::DE1101_TESTS_IN_SEPARATE_FILES,
        de12_documentation::de1201_docs_rs_all_features::DE1201_DOCS_RS_ALL_FEATURES,
        de13_common_patterns::de1301_no_print_macros::DE1301_NO_PRINT_MACROS,
        de13_common_patterns::de1302_error_from_to_string::DE1302_ERROR_FROM_TO_STRING,
        de13_common_patterns::de1303_no_primitive_type_alias::DE1303_NO_PRIMITIVE_TYPE_ALIAS,
    ]);

    lint_store.register_pre_expansion_pass(|| {
        Box::new(de01_domain_layer::de0101_no_serde_in_domain::De0101NoSerdeInContract)
    });
    lint_store.register_pre_expansion_pass(|| {
        Box::new(de01_domain_layer::de0102_no_toschema_in_domain::De0102NoToschemaInContract)
    });
    lint_store.register_pre_expansion_pass(|| {
        Box::new(de01_domain_layer::de0104_no_api_dto_in_domain::De0104NoApiDtoInContract)
    });
    lint_store.register_pre_expansion_pass(|| {
        Box::new(de02_api_layer::de0203_dtos_must_use_api_dto::De0203DtosMustUseApiDto)
    });
    lint_store.register_pre_expansion_pass(|| {
        Box::new(
            de02_api_layer::de0204_dtos_must_have_toschema_derive::De0204DtosMustHaveToschemaDerive,
        )
    });
    lint_store.register_pre_expansion_pass(|| {
        Box::new(de08_rest_api_conventions::de0803_api_snake_case::De0803ApiSnakeCase)
    });
    lint_store.register_pre_expansion_pass(|| {
        Box::new(de11_testing::de1101_tests_in_separate_files::De1101TestsInSeparateFiles::new())
    });
    lint_store.register_pre_expansion_pass(|| {
        Box::new(de09_gts_layer::de0901_gts_string_pattern::De0901GtsStringPattern::new())
    });
    lint_store.register_pre_expansion_pass(|| {
        Box::new(de13_common_patterns::de1301_no_print_macros::De1301NoPrintMacros)
    });

    lint_store.register_early_pass(|| {
        Box::new(de02_api_layer::de0201_dtos_only_in_api_rest::De0201DtosOnlyInApiRest)
    });
    lint_store.register_early_pass(|| {
        Box::new(de03_domain_layer::de0301_no_infra_in_domain::De0301NoInfraInDomain)
    });
    lint_store.register_early_pass(|| {
        Box::new(de03_domain_layer::de0308_no_http_in_domain::De0308NoHttpInDomain)
    });
    lint_store.register_early_pass(|| {
        Box::new(de05_client_layer::de0503_plugin_client_suffix::De0503PluginClientSuffix)
    });
    lint_store.register_early_pass(|| {
        Box::new(de05_client_layer::de0504_client_versioning::De0504ClientVersioning)
    });
    lint_store
        .register_early_pass(|| Box::new(de07_security::de0706_no_direct_sqlx::De0706NoDirectSqlx));
    lint_store.register_early_pass(|| {
        Box::new(de13_common_patterns::de1303_no_primitive_type_alias::De1303NoPrimitiveTypeAlias)
    });

    lint_store.register_late_pass(|_| {
        Box::new(
            de02_api_layer::de0202_dtos_not_referenced_outside_api::De0202DtosNotReferencedOutsideApi,
        )
    });
    lint_store.register_late_pass(|_| {
        Box::new(de12_documentation::de1201_docs_rs_all_features::De1201DocsRsAllFeatures::new())
    });
    lint_store
        .register_late_pass(|_| Box::new(de07_security::de0707_drop_zeroize::De0707DropZeroize));
    lint_store.register_late_pass(|_| {
        Box::new(de08_rest_api_conventions::de0801_api_endpoint_version::De0801ApiEndpointVersion)
    });
    lint_store.register_late_pass(|_| {
        Box::new(de08_rest_api_conventions::de0802_use_odata_ext::De0802UseOdataExt)
    });
    lint_store.register_late_pass(|_| {
        Box::new(de09_gts_layer::de0902_no_schema_for_on_gts_structs::De0902NoSchemaForOnGtsStructs)
    });
    lint_store.register_late_pass(|_| {
        Box::new(de13_common_patterns::de1302_error_from_to_string::De1302ErrorFromToString)
    });
}

#[cfg(test)]
mod tests {
    use super::LIBRARY_NAME;

    #[test]
    fn ui_examples() {
        dylint_testing::ui_test_examples(LIBRARY_NAME);
    }
}
