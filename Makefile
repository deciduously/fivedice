.PHONY: all deploy help preapre

RUSTCLEAN=cargo clean
RUST=wasm-pack build
PKGDIR=pkg
EXEC=fivedice_bg.wasm
OPT=./shrink-wasm.sh -f=speed -l=aggro

all: prepare $(PKGDIR)/$(EXEC)
	$(OPT)

$(PKGDIR)/$(EXEC):
	$(RUST)

deploy: all
	npm run build

help:
    @echo "Usage: make {all|deploy|help}" 1>&2 && false

prepare:
	rm -f $(PKGDIR)/$(EXEC)
