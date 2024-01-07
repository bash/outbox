# Helpful documentation links:
# * Documentation of rpkg: https://docs.pagure.org/rpkg-util/v3/index.html
# * Directives for the %files list: http://ftp.rpm.org/max-rpm/s1-rpm-inside-files-list-directives.html
# * RPM Macros: https://docs.fedoraproject.org/en-US/packaging-guidelines/RPMMacros/
# * RPM Macros available locally: /usr/lib/rpm/macros.d/
# * Sysusers: `man sysusers.d`

%global selinuxtype targeted

Name:       outbox
Version:    0.1.0
Release:    1%{?dist}
Summary:    A mail queue daemon for msmtp
License:    MIT or Apache-2.0
URL:        https://github.com/bash/outbox
Requires:   msmtp
Requires:   (%{name}-selinux = %{version}-%{release} if selinux-policy-%{selinuxtype})

Source:     sources.tar.gz
%{?systemd_requires}
BuildRequires: cargo
BuildRequires: systemd

%description
A mail queue daemon for msmtp.

%package selinux
Summary:    SELinux policy module for waydroid
Requires:   %{name} = %{version}-%{release}
%{?selinux_requires}

%description selinux
This package contains the SELinux policy module necessary to run outbox.

%prep
tar -xf %{SOURCE0}
mkdir SELinux
cp *.te SELinux/

%build
BINDIR=%{_bindir} SYSCONFDIR=%{_sysconfdir} LOGDIR=%{_localstatedir}/log SHAREDSTATEDIR=%{_sharedstatedir} envsubst < systemd/system/outboxd.service > outboxd.service
SHAREDSTATEDIR=%{_sharedstatedir} envsubst < sysusers.d/outbox.conf > outbox-sysusers.conf
meson setup --prefix=%{buildroot}/usr _build -Dbuildtype=release
ninja -C _build
cd SELinux
%{__make} NAME=%{selinuxtype} -f /usr/share/selinux/devel/Makefile

%install
ninja -C _build install
install -D -p -m 644 outbox-sysusers.conf %{buildroot}%{_sysusersdir}/%{name}.conf
install -D -p -m 644 outboxd.service %{buildroot}%{_unitdir}/outboxd.service
install -D -p -m 644 dbus-1/system-services/garden.tau.Outbox.service %{buildroot}%{_datarootdir}/dbus-1/system-services/garden.tau.Outbox.service
install -D -p -m 644 dbus-1/system.d/garden.tau.Outbox.conf %{buildroot}%{_datarootdir}/dbus-1/system.d/garden.tau.Outbox.conf
install -D -p -m 600 msmtprc %{buildroot}%{_sysconfdir}/outboxd/msmtprc
install -D -d -m 700 %{buildroot}%{_sharedstatedir}/outboxd
install -D -d -m 700 %{buildroot}%{_localstatedir}/log/outboxd
install -D -p -m 644 SELinux/%{name}.pp %{buildroot}%{_datadir}/selinux/%{selinuxtype}/%{name}.pp

%pre
# This ensures that the users are created *before* file attributes are applied
%sysusers_create_package %{name} outbox-sysusers.conf

%post
%systemd_post outboxd.service

%post selinux
%selinux_modules_install -s %{selinuxtype} %{_datadir}/selinux/%{selinuxtype}/%{name}.pp

%preun
%systemd_preun outboxd.service

%postun
%systemd_postun_with_restart outboxd.service

%postun selinux
if [ $1 -eq 0 ]; then
  %selinux_modules_uninstall -s %{selinuxtype} %{name}
fi

%files
%{_sysusersdir}/%{name}.conf
%{_bindir}/outboxd
%{_mandir}/man1/garden.tau.Outbox.*
%{_mandir}/man1/outboxd.*
%{_unitdir}/outboxd.service
%{_datarootdir}/dbus-1/system-services/garden.tau.Outbox.service
%{_datarootdir}/dbus-1/system.d/garden.tau.Outbox.conf
%config(noreplace) %attr(-, outboxd, outboxd) %{_sysconfdir}/outboxd/msmtprc
%dir %attr(-, outboxd, outboxd) %{_sharedstatedir}/outboxd
%dir %attr(-, outboxd, outboxd) %{_localstatedir}/log/outboxd
%ghost %{_localstatedir}/log/outboxd/msmtp.log

%files selinux
%{_datadir}/selinux/%{selinuxtype}/%{name}.pp
