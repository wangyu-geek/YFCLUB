.PHONY: build release portable portable-zip run clean-db clean-database

NPM ?= npm
POWERSHELL ?= powershell -NoProfile -ExecutionPolicy Bypass -Command
DB_FILE ?= data/club.db
TAURI_RELEASE_EXE ?= src-tauri/target/release/member-club.exe
PORTABLE_OUTPUT_DIR ?= dist/portable

build:
	$(NPM) run build

release:
	$(NPM) run tauri:build

portable:
	$(NPM) run tauri:build -- --no-bundle
	$(POWERSHELL) '& "./scripts/package-windows-portable.ps1" -SourceExe "$(TAURI_RELEASE_EXE)" -OutputDir "$(PORTABLE_OUTPUT_DIR)"'

portable-zip: portable

run:
	$(NPM) run tauri:dev

clean-db:
	$(POWERSHELL) '$$targets = @("$(DB_FILE)", "$(DB_FILE)-wal", "$(DB_FILE)-shm"); foreach ($$target in $$targets) { if (Test-Path -LiteralPath $$target) { Remove-Item -LiteralPath $$target -Force; Write-Host ("Removed " + $$target) } else { Write-Host ("Skipped " + $$target) } }'

clean-database: clean-db
