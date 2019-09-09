.PHONY: all clean deploy help

RUSTCLEAN=cargo clean
RUST=wasm-pack build
PKGDIR=pkg
OUTDIR=docs
EXEC=fivedice_bg.wasm
OPT=./shrink-wasm.sh -f=speed -l=aggro

all: $(PKGDIR)/$(EXEC)
	$(OPT)

$(PKGDIR)/$(EXEC):
	$(RUST)

clean:
	$(RUSTCLEAN)
	rm $(OUTDIR)/*

deploy: clean all
	npm run build

help:
    @echo "Usage: make {all|clean|help}" 1>&2 && false