.PHONY: all clean help

OPKGDIR=pkg
EXEC=fivedice_bg.wasm
OPT=./shrink-wasm.sh -l=aggro

all: $(PKGDIR)/$(EXEC)
	$(OPT)

$(PKGDIR)/$(EXEC):
	wasm-pack build

clean:
	cargo clean

help:
    @echo "Usage: make {all|clean|help}" 1>&2 && false