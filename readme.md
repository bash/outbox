# Outbox 📤
A very crude attempt at a mail queue for `sendmail`-compatible tools like `msmtp` that lack queues.

## Running Locally
Use `make run` to run `outboxd` using the session bus.
Instead of sending emails it will open them in the default email program.

## Example Config
### Systemd Unit

**/etc/systemd/system/outboxd.service**
```desktop
[Unit]
Description=A rudimentary outbox that delivers mail using a sendmail compatible command
After=network-online.target

[Service]
BusName=garden.tau.Outbox
ExecStart=/usr/local/bin/outboxd -d -C /etc/outbox/msmtprc
Restart=always
RestartSec=10
PrivateDevices=yes
PrivateNetwork=yes
PrivateTmp=yes
ProtectProc=invisible
ProtectControlGroups=yes
ProtectHome=yes
ProtectKernelLogs=yes
ProtectKernelModules=yes
ProtectKernelTunables=yes
ProtectSystem=strict
ReadWritePaths=/var/lib/outbox /var/log/msmtp
Environment=SENDMAIL=/usr/local/bin/msmtp
```

**/usr/local/share/dbus-1/system-services/garden.tau.Outbox.service**
See the [DBus Specification] for details.

```desktop
[D-BUS Service]
Name=garden.tau.Outbox
Exec=/bin/false
User=root
SystemdService=outboxd.service
```

### DBus Config
See the [DBus Specification] for details.

**/etc/dbus-1/system.d/garden.tau.Outbox.conf**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE busconfig PUBLIC
 "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <policy user="root">
    <allow own="garden.tau.Outbox"/>
  </policy>

  <!-- Allow our other service to use the garden.tau.Outbox bus -->
  <!-- You can alternatively use <policy context="default"> to allow it to everyone. -->
  <policy user="game-night-service">
    <allow send_destination="garden.tau.Outbox" />
  </policy>
</busconfig>
```

### SELinux
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
