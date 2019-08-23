.PHONY: all clean help

RUSTCLEAN=cargo clean
RUST=wasm-pack build
OPKGDIR=pkg
EXEC=fivedice_bg.wasm
OPT=./shrink-wasm.sh -f=speed -l=aggro

all: $(PKGDIR)/$(EXEC)
	$(OPT)

$(PKGDIR)/$(EXEC):
	$(RUST)

clean:
	$(RUSTCLEAN)

help:
    @echo "Usage: make {all|clean|help}" 1>&2 && false