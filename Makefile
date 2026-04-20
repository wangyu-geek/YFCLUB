.PHONY: build release

NPM ?= npm

build:
	$(NPM) run build

release:
	$(NPM) run tauri:build

run:
	$(NPM) run tauri:dev