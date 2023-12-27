.PHONY: run

run:
	cargo build -p open
	SENDMAIL=./target/debug/open DBUS_CONNECTION=session cargo run -p outboxd
