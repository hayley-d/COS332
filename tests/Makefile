.PHONY: clean

ARGS = -lraylib -lGL -lm -lc -lpthread -ldl -lrt -lX11
OUT = main

$(OUT): main.rs
	rustc --edition 2021 -g -C opt-level=z -C link-args="$(ARGS)" -C panic=abort main.rs -o $(OUT)

clean:
	rm -f main
	rm -f *.o
	rm -f *.pdb


