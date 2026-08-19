# Cargo Gears Lints Documentation

Index of per-lint documentation for `cargo-gears-lints`. Each lint's README lives alongside its implementation in `src/`.

## DE01xx - Domain Layer

- [DE0101 - No Serde in Domain](../src/de01_domain_layer/de0101_no_serde_in_domain/README.md)
- [DE0102 - No ToSchema in Domain](../src/de01_domain_layer/de0102_no_toschema_in_domain/README.md)
- [DE0104 - No API DTO in Domain](../src/de01_domain_layer/de0104_no_api_dto_in_domain/README.md)

## DE02xx - API Layer

- [DE0201 - DTOs Only in API Rest](../src/de02_api_layer/de0201_dtos_only_in_api_rest/README.md)
- [DE0202 - DTOs Not Referenced Outside API](../src/de02_api_layer/de0202_dtos_not_referenced_outside_api/README.md)
- [DE0203 - DTOs Must Use API DTO](../src/de02_api_layer/de0203_dtos_must_use_api_dto/README.md)
- [DE0204 - DTOs Must Have ToSchema Derive](../src/de02_api_layer/de0204_dtos_must_have_toschema_derive/README.md)

## DE03xx - Domain Layer Boundaries

- [DE0301 - No Infra in Domain](../src/de03_domain_layer/de0301_no_infra_in_domain/README.md)
- [DE0308 - No HTTP in Domain](../src/de03_domain_layer/de0308_no_http_in_domain/README.md)
- [DE0309 - Must Have Domain Model](../src/de03_domain_layer/de0309_must_have_domain_model/README.md)

## DE05xx - Client Layer

- [DE0503 - Plugin Client Suffix](../src/de05_client_layer/de0503_plugin_client_suffix/README.md)

## DE07xx - Security

- [DE0706 - No Direct SQLx](../src/de07_security/de0706_no_direct_sqlx/README.md)
- [DE0707 - Drop Zeroize](../src/de07_security/de0707_drop_zeroize/README.md)
- [DE0708 - No Non-FIPS Hasher](../src/de07_security/de0708_no_non_fips_hasher/README.md)

## DE08xx - REST API Conventions

- [DE0801 - API Endpoint Version](../src/de08_rest_api_conventions/de0801_api_endpoint_version/README.md)
- [DE0803 - API Snake Case](../src/de08_rest_api_conventions/de0803_api_snake_case/README.md)

## DE09xx - GTS Layer

- [DE0901 - GTS String Pattern](../src/de09_gts_layer/de0901_gts_string_pattern/README.md)
- [DE0902 - No Schema For On GTS Structs](../src/de09_gts_layer/de0902_no_schema_for_on_gts_structs/README.md)
- [DE0904 - No Hard-Coded GTS Prefix](../src/de09_gts_layer/de0904_no_hardcoded_gts_prefix/README.md)

## DE12xx - Documentation

- [DE1201 - Docs.rs All Features](../src/de12_documentation/de1201_docs_rs_all_features/README.md)
- [DE1202 - Missing Docs for `pub(crate)` APIs](../src/de12_documentation/de1202_missing_docs_for_pub_crate/README.md)

## DE11xx - Testing

- [DE1101 - Tests in Separate Files](../src/de11_testing/de1101_tests_in_separate_files/README.md)

## DE13xx - Common Patterns

- [DE1301 - No Print Macros](../src/de13_common_patterns/de1301_no_print_macros/README.md)
- [DE1302 - Error From To String](../src/de13_common_patterns/de1302_error_from_to_string/README.md)
- [DE1303 - No Primitive Type Alias](../src/de13_common_patterns/de1303_no_primitive_type_alias/README.md)
