.PHONY: all build run run-only clean

all: build

build:
	./build.sh

run: build
	./run.sh

run-only:
	./run.sh

clean:
	rm -rf build disk/os.img kernel/target
