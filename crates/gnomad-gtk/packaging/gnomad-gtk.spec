# Fedora/Nobara RPM template — Phase 1 (no WebKit)
Name:           gnomad
Version:        0.2.0
Release:        1%{?dist}
Summary:        Gnomad Linux-native desktop assistant (GTK4)

License:        Apache-2.0
URL:            https://gnomadstudio.org
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust cargo
BuildRequires:  gtk4-devel >= 4.14
BuildRequires:  libadwaita-devel >= 1.5
Requires:       gtk4 >= 4.14
Requires:       libadwaita >= 1.5
Recommends:     wl-clipboard bubblewrap ollama wmctrl

%description
Linux-first Gnomad shell using GTK 4 and Libadwaita (no WebKitGTK).

%prep
%autosetup

%build
cargo build --release -p gnomad-gtk

%install
install -Dpm 0755 target/release/gnomad %{buildroot}%{_bindir}/gnomad
install -Dpm 0644 packaging/com.gnomadstudio.gnomad.desktop %{buildroot}%{_datadir}/applications/com.gnomadstudio.gnomad.desktop
install -Dpm 0644 crates/gnomad-gtk/resources/GNOMAD_HELP.md %{buildroot}%{_datadir}/doc/gnomad/GNOMAD_HELP.md

%files
%{_bindir}/gnomad
%{_datadir}/applications/com.gnomadstudio.gnomad.desktop
%{_datadir}/doc/gnomad/GNOMAD_HELP.md

%changelog
* Fri Jun 05 2026 Gnomad Studio <dev@gnomadstudio.org> - 0.1.0-1
- Phase 1 GTK shell spike
