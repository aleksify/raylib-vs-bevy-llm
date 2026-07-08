# terra-c — raylib 6.0, vendored source build (no cmake dependency).
# `make vendor` clones raylib at tag 6.0 into vendor/raylib (once).

RAYLIB_DIR := vendor/raylib
RAYLIB_LIB := $(RAYLIB_DIR)/src/libraylib.a

# -ffp-contract=off: no fma contraction, keeps float math bit-identical to Rust
# (both clang and gcc honor it)
CFLAGS  := -std=c99 -Wall -Wextra -O2 -ffp-contract=off -I$(RAYLIB_DIR)/src

UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
CC      := clang
LDFLAGS := -framework CoreVideo -framework IOKit -framework Cocoa -framework OpenGL
else
# Linux (X11). Needs the usual raylib build deps, e.g. on Debian/Ubuntu:
#   apt install build-essential libx11-dev libxrandr-dev libxi-dev \
#       libxcursor-dev libxinerama-dev libgl1-mesa-dev libasound2-dev
CC      ?= gcc
LDFLAGS := -lGL -lm -lpthread -ldl -lrt -lX11
endif

SRC := $(wildcard src/*.c)
BIN := terra

$(BIN): $(SRC) $(RAYLIB_LIB)
	$(CC) $(CFLAGS) $(SRC) $(RAYLIB_LIB) $(LDFLAGS) -o $(BIN)

$(RAYLIB_LIB): | $(RAYLIB_DIR)
	$(MAKE) -C $(RAYLIB_DIR)/src PLATFORM=PLATFORM_DESKTOP

$(RAYLIB_DIR):
	mkdir -p vendor
	curl -sL https://github.com/raysan5/raylib/archive/refs/tags/6.0.tar.gz | tar xz -C vendor --exclude '.vscode'
	mv vendor/raylib-6.0 $(RAYLIB_DIR)

vendor: $(RAYLIB_DIR)

run: $(BIN)
	./$(BIN)

clean:
	rm -f $(BIN)

.PHONY: vendor run clean
