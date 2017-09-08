.PHONY: all

all: Readme.md

Readme.md: src/lib.rs
	grep '//!' src/lib.rs | sed -e 's/^\/\/\!//' > Readme.md