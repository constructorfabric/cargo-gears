.PHONY: docs-preview docs-lint

# Preview the full documentation site locally (CLI docs from this repo + the rest
# from gears-rust) at http://localhost:4321.
docs-preview:
	@bash tools/scripts/docs-site.sh dev

# Lint this repo's docs/web-docs markdown with the docs site's markdownlint config.
docs-lint:
	@bash tools/scripts/docs-site.sh lint
