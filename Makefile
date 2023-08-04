all: help

.PHONY: help
help:
	@echo Usage:
	@echo   - make watch
	@echo   - make minio

watch:
	cargo watch -x run

jstest:
	cargo run --bin jstest && eog out.png

minio:
	docker-compose up