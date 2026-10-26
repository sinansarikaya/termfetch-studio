%global debug_package %{nil}

Name:           termfetch-studio
Version:        2.0.0
Release:        1%{?dist}
Summary:        Cross-platform terminal customization and Fastfetch theme studio
License:        MIT
URL:            https://github.com/sinansarikaya/termfetch-studio
Source0:        %{name}-v%{version}-x86_64-unknown-linux-gnu.tar.gz
Requires:       fastfetch
Recommends:     chafa

%description
TermFetch Studio v2 is an interactive terminal customization tool built with
Rust and Ratatui for browsing, previewing, and configuring Fastfetch themes,
presets, modules, and logos in real time.

%prep
%setup -q -n %{name}-v%{version}-x86_64-unknown-linux-gnu

%install
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_datadir}/%{name}
install -m 755 termfetch-studio %{buildroot}%{_bindir}/termfetch-studio
ln -sf termfetch-studio %{buildroot}%{_bindir}/termfetch
cp -r assets %{buildroot}%{_datadir}/%{name}/

# Existing v2.0.0 archives have no brand assets; future release builds check them.
: > %{_builddir}/termfetch-studio-brand-files
if [ -f assets/brand/termfetch-symbol.svg ] && [ -f assets/brand/termfetch-studio.desktop ]; then
    install -D -m 644 assets/brand/termfetch-symbol.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/termfetch-studio.svg
    install -D -m 644 assets/brand/termfetch-studio.desktop %{buildroot}%{_datadir}/applications/termfetch-studio.desktop
    echo '%{_datadir}/icons/hicolor/scalable/apps/termfetch-studio.svg' >> %{_builddir}/termfetch-studio-brand-files
    echo '%{_datadir}/applications/termfetch-studio.desktop' >> %{_builddir}/termfetch-studio-brand-files
fi

%files -f %{_builddir}/termfetch-studio-brand-files
%{_bindir}/termfetch-studio
%{_bindir}/termfetch
%{_datadir}/%{name}/assets
%license LICENSE
%doc README.md

%changelog
* Sun Oct 25 2026 Sinan Sarikaya <sinan@sinansarikaya.com> - 2.0.0-1
- TermFetch Studio v2.0.0: Native Rust rewrite with live preview, pixel art themes and cross-platform support
