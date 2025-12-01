TARGET = uno-ws.exe

.PHONY: all b r

all: b r

b:
	go build -trimpath -o $(TARGET) ./src

r:
	./$(TARGET)
