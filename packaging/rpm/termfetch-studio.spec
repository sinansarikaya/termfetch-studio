Name:           termfetch-studio
Version:        2.0.0
Release:        1%{?dist}
Summary:        Cross-platform terminal customization and Fastfetch theme studio
License:        MIT
URL:            https://github.com/sinansarikaya/termfetch-studio
Requires:       fastfetch
Recommends:     chafa

%description
TermFetch Studio v2 is an interactive terminal customization tool built with
Rust and Ratatui for browsing, previewing, and configuring Fastfetch themes,
presets, modules, and logos in real time.

%prep
# No prep needed for prebuilt binary packaging

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}/usr/bin
mkdir -p %{buildroot}/usr/share/termfetch-studio
install -m 755 termfetch-studio %{buildroot}/usr/bin/termfetch-studio
cp -r assets %{buildroot}/usr/share/termfetch-studio/

%clean
rm -rf %{buildroot}

%files
/usr/bin/termfetch-studio
/usr/share/termfetch-studio/assets
