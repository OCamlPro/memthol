.PHONY: all build build-deps install
.PHONY: clean distclean

all: build

build:
	opam exec -- dune build @install
	cp -f _build/default/src/min_memtrace/main.exe min_memtrace

build-deps:
	if ! [ -e _opam ]; then \
	   opam switch create . --empty && \
	   opam install ocaml.4.10.0 ; \
	fi
	opam install ./*.opam --deps-only

install:
	opam exec -- dune install

opam:
	opam pin -k path .

uninstall:
	opam exec -- dune uninstall

clean:
	rm -rf _build

distclean: clean
	rm -rf _opam


