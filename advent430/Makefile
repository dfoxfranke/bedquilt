# Makefile for the Glulx edition of Open Adventure

# SPDX-FileCopyrightText: (C) Eric S. Raymond <esr@thyrsus.com>, Daniel Fox Franke <dfoxfranke@gmail.com>
# SPDX-License-Identifier: BSD-2-Clause

WASM2GLULX=cargo run --bin wasm2glulx --
WASMTOOLS=wasm-tools
CC=clang
OBJS=main.o init.o actions.o score.o misc.o saveresume.o bedquilt.o dungeon.o

all: advent430.ulx

%.o: %.c
	$(CC) --target=wasm32-unknown-unknown -std=c99 -c -Wall -Wextra -O2 -ffreestanding -nostdinc -nostdlib $(CFLAGS) $<

dungeon.c dungeon.h &: make_dungeon.py adventure.yaml advent.h templates/*.tpl
	./make_dungeon.py

advent430.wasm: $(OBJS)
	$(CC) -o advent430.wasm --target=wasm32-unknown-unknown -std=c99 -Wall -Wextra -O2 -ffreestanding -nostdinc -nostdlib -Wl,--no-entry -Wl,--import-undefined -Wl,--export,glulx_main $(LDFLAGS) $(OBJS)

%.ulx: %.wasm
	$(WASM2GLULX) $<

%.glulxasm: %.wasm
	$(WASM2GLULX) --text $<

%.wat: %.wasm
	$(WASMTOOLS) print -o $@ $<

main.o:	 	advent.h bedquilt.h dungeon.h wasmglk.h
init.o:	 	advent.h bedquilt.h dungeon.h wasmglk.h
actions.o:	advent.h bedquilt.h dungeon.h wasmglk.h
score.o:	advent.h bedquilt.h dungeon.h wasmglk.h
misc.o:		advent.h bedquilt.h dungeon.h wasmglk.h
saveresume.o:	advent.h bedquilt.h dungeon.h wasmglk.h

clean:
	rm -f $(OBJS) advent430.wasm advent430.ulx advent430.wat advent430.glulxasm dungeon.c dungeon.h

reflow:
	clang-format -i $$(find . -name "*.[ch]")
	
.PHONY: all clean reflow