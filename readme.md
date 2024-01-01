# Outbox 📤
A mail queue daemon for msmtp.

## Installing
TODO

## Using during Development
Use `make run` to run `outboxd` using the session bus.
Instead of sending emails it will open them in the default email program.

## SELinux
If you have SELinux enabled, it might cause issues with passing fifo pipes.
This can be solved using a custom module.

**dbus_allow_fifo_read.te**
```sepolicy
module dbus_allow_fifo_read 1.0;

require {
        type system_dbusd_t;
        type unconfined_service_t;
        class fifo_file read;
}

allow system_dbusd_t unconfined_service_t:fifo_file read;
```

Then generate and install the module:
```sh
checkmodule -M -m -o dbus_allow_fifo_read.mod dbus_allow_fifo_read.te
semodule_package -o dbus_allow_fifo_read.pp -m dbus_allow_fifo_read.mod
semodule -i dbus_allow_fifo_read.pp
```


[DBus Specification]: https://dbus.freedesktop.org/doc/dbus-specification.html
