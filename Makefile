.PHONY: docs-preview docs-lint lints-tree lints-tree-check

# Preview the full documentation site locally (CLI docs from this repo + the rest
# from gears-rust) at http://localhost:4321.
docs-preview:
	@bash tools/scripts/docs-site.sh dev

# Lint this repo's docs/web-docs markdown with the docs site's markdownlint config.
docs-lint:
	@bash tools/scripts/docs-site.sh lint

# Record the git tree id of the staged cargo-gears-lints sources in
# crates/cargo-gears/lints.tree, so lints changes release with cargo-gears.
lints-tree:
	@bash tools/scripts/lints-tree.sh update

# Fail if crates/cargo-gears/lints.tree doesn't match the staged lints sources.
lints-tree-check:
	@bash tools/scripts/lints-tree.sh check
