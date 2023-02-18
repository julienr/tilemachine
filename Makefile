all: help

.PHONY: help
help:
	@echo Usage:
	@echo   - make watch
	@echo   - make minio

watch:
	cargo watch -x run

minio:
	docker-compose up