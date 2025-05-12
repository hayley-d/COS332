# Makefile in practical-7-go

.PHONY: all run md5 clean

all: run

run: md5
	@echo "Starting POP3 server..."
	@(cd ../pop3-server && go run main.go) & \
	sleep 1 && \
	echo "Starting POP3 client..." && \
	go run main.go

md5:
	@echo "MD5 of main.go:"
	@md5sum main.go

clean:
	@echo "No binaries to clean when using go run"

