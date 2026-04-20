.PHONY: build release run clean-db clean-database

NPM ?= npm
POWERSHELL ?= powershell -NoProfile -ExecutionPolicy Bypass -Command
DB_FILE ?= data/club.db

build:
	$(NPM) run build

release:
	$(NPM) run tauri:build

run:
	$(NPM) run tauri:dev

clean-db:
	$(POWERSHELL) '$$targets = @("$(DB_FILE)", "$(DB_FILE)-wal", "$(DB_FILE)-shm"); foreach ($$target in $$targets) { if (Test-Path -LiteralPath $$target) { Remove-Item -LiteralPath $$target -Force; Write-Host ("Removed " + $$target) } else { Write-Host ("Skipped " + $$target) } }'

clean-database: clean-db
