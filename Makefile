.PHONY: run rpm

run:
	cargo build -p open
	SENDMAIL=./target/debug/open DBUS_CONNECTION=session cargo run -p outboxd

rpm:
	rpkg local
