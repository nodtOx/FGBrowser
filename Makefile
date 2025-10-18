.PHONY: run build clear-db run-clear-db test lint clean install upload-db deploy-nginx version bump-patch bump-minor bump-major release build-windows build-macos build-all

run:
	npm run tauri dev

build:
	npm run tauri build

# Database management
clear-db:
	rm -f repacks.db repacks.db-shm repacks.db-wal

runc:
	make clear-db
	make run

# Rust-specific tasks
test:
	cd src-tauri && cargo test

lint:
	cd src-tauri && cargo clippy

# Full clean (both npm and cargo)
clean:
	rm -rf node_modules
	rm -rf src-tauri/target
	rm -f repacks.db*

# Setup project
install:
	npm install

upload-db:
	@echo "Creating clean database (excluding user settings/downloads)..."
	@bash scripts/export-clean-db.sh
	@echo ""
	@echo "Uploading clean database to server..."
	rsync -avz --progress --checksum repacks-clean.db root@157.230.16.45:/var/www/fgbrowser/repacks.db
	@echo "✅ Clean database uploaded successfully"
	rm -f repacks-clean.db

deploy-nginx:
	@echo "Uploading nginx config to server..."
	scp server/nginx.conf root@157.230.16.45:/etc/nginx/sites-available/fgbrowser
	@echo "Testing nginx configuration..."
	ssh root@157.230.16.45 'nginx -t'
	@echo "Reloading nginx..."
	ssh root@157.230.16.45 'systemctl reload nginx'
	@echo "✅ Nginx config deployed and reloaded successfully"

# Version management
version:
	@cat VERSION

bump-patch:
	@bash scripts/bump-version.sh patch

bump-minor:
	@bash scripts/bump-version.sh minor

bump-major:
	@bash scripts/bump-version.sh major

release:
	@bash scripts/release.sh

# Build releases (requires GitHub CLI)
build-windows:
	@echo "🪟 Triggering Windows build..."
	gh workflow run "Build Windows Release"
	@echo "✅ Windows build triggered. Check GitHub Actions for progress."

build-macos:
	@echo "🍎 Triggering macOS build..."
	gh workflow run "Build macOS Release"
	@echo "✅ macOS build triggered. Check GitHub Actions for progress."

build-all:
	@echo "🚀 Triggering all platform builds..."
	gh workflow run "Build Windows Release"
	gh workflow run "Build macOS Release"
	@echo "✅ All builds triggered. Check GitHub Actions for progress."