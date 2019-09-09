.PHONY: all clean deploy pkgclean help

RUSTCLEAN=cargo clean
RUST=wasm-pack build
PKGDIR=pkg
OUTDIR=docs
EXEC=fivedice_bg.wasm
OPT=./shrink-wasm.sh -f=speed -l=aggro

all: pkgclean $(PKGDIR)/$(EXEC)
	$(OPT)

$(PKGDIR)/$(EXEC):
	$(RUST)

clean: pkgclean
	rm $(OUTDIR)/*

deploy: clean all
	npm run build

pkgclean:
	rm $(PKGDIR)/$(EXEC)

help:
    @echo "Usage: make {all|clean|deploy|help}" 1>&2 && false