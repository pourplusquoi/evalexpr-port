CC = clang++
FLAGS = -g -O3 -Wall --std=c++17

rs:
	cargo +nightly build --release --lib

cc: rs
	mkdir -p target
	$(CC) $(FLAGS) -c -o target/port.o cc/port.cc

test: cc rs
	mkdir -p target
	$(CC) $(FLAGS) -c -o target/main.o cc/main.cc
	$(CC) $(FLAGS) -o target/main target/main.o target/port.o -Ltarget/release -levalexpr_port

clean:
	rm -r target
