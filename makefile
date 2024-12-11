.PHONY: run-server run-example

RUST_LOG ?= debug

run-server:
	RUST_LOG=$(RUST_LOG) cargo run

run-example:
	@if [ -z "$(EXAMPLE)" ]; then \
		echo "Please specify an example name, e.g., 'make run-example EXAMPLE=chat'"; \
		exit 1; \
	fi
	cargo run --example $(EXAMPLE)

chat:
	make run-example EXAMPLE=chat

help:
	@echo "Available commands:"
	@echo "  make run-server          - Run the server with debug logging"
	@echo "  make run-example EXAMPLE=chat  - Run a specific example"
	@echo "  make chat               - Run the chat example"
	@echo ""
	@echo "Environment variables:"
	@echo "  RUST_LOG    - Set logging level (default: debug)"
