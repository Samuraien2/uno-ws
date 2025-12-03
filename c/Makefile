all:
	cc src/*.c src/cws/*.c -o main -Wall -Wextra -g -fsanitize=address -fno-omit-frame-pointer

old:
	cc main.c -o main -lwebsockets

r: all
	./main