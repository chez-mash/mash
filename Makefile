# A shim for the transition from Make to just. The justfile is the source of
# truth: every target here runs the recipe of the same name and holds no
# commands of its own. Use `just RECIPE` (`just` lists them); this file will
# be removed.
#
# Variables given to make reach the recipe unchanged, because make exports
# them and the justfile reads the same names from the environment:
#
#   make iterate FOCUS='test(changed_behavior)' PACKAGE=mash-dash
#   make build CODEGEN=1 CARGO_ARGS='--release --locked'
#   make protocol-check DEV_RUN=
JUST ?= just

FORWARDED := mutation-tools mutation-check mutation-services \
	mutation-certification mutation-runtime protocol-tools protocol-check \
	release-tools deny test-cleanup iterate dev-build validate-dev build \
	check-skills generate-skills generate-docs generate-queries \
	check-generated coverage site-review

.PHONY: $(FORWARDED)
$(FORWARDED):
	@command -v $(JUST) >/dev/null 2>&1 || { \
		echo 'make $@ now runs `just $@`, and just is not installed.' >&2; \
		echo 'Install it: bash tools/install-just.sh (Linux, into ~/.local/bin), brew install just (macOS),' >&2; \
		echo 'scoop install just or winget install --id Casey.Just --exact (Windows).' >&2; \
		exit 127; }
	@$(JUST) $@
