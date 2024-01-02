.PHONY: all run rpm

all: man/outboxd.1.gz man/garden.tau.Outbox.1.gz

man/%.gz: man/%
	gzip -c $< > $@

run:
	cargo build -p open
	SENDMAIL=./target/debug/open DBUS_CONNECTION=session cargo run -p outboxd

rpm:
	rpkg local
