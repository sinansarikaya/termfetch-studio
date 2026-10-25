# TermFetch Studio v2 — Official Release & Distribution Plan

**Sürüm:** `2.0.0`  
**Hedef Platformlar:** Linux (Arch, Ubuntu, Debian, Fedora, openSUSE, Alpine), macOS (Apple Silicon & Intel), Windows 10/11  
**Tarih:** 25 Ekim 2026  

---

## 1. Dağıtım Kanalları ve Yol Haritası

TermFetch Studio v2'nin yayın süreci 4 aşamalı olarak tasarlanmıştır. Dış onay gerektirmeyen (kendi kontrolümüzdeki) kanallar ilk gün anında aktif edilecek, topluluk incelemesi gerektiren kanallar ise hazır manifestler üzerinden PR ile başlatılacaktır.

```
+-----------------------------------------------------------------------------------+
| AŞAMA 1: Kendi Kontrolümüzdeki Anında Dağıtım                                      |
| - GitHub Releases (Tar.gz, Zip, DEB, Checksums)                                   |
| - Tek Satırlık Hızlı Kurulum Scripti (curl | bash)                                 |
+-----------------------------------------------------------------------------------+
                                       |
                                       v
+-----------------------------------------------------------------------------------+
| AŞAMA 2: Arch Linux AUR Yayınlaması                                                |
| - termfetch-studio-bin (Hazır binary ile saniyeler içinde kurulum)                 |
| - termfetch-studio (Kaynak koddan derleme)                                        |
+-----------------------------------------------------------------------------------+
                                       |
                                       v
+-----------------------------------------------------------------------------------+
| AŞAMA 3: macOS Homebrew Tap                                                       |
| - sinansarikaya/homebrew-tap deposu oluşturulması                                 |
| - brew tap sinansarikaya/tap && brew install termfetch-studio                      |
+-----------------------------------------------------------------------------------+
                                       |
                                       v
+-----------------------------------------------------------------------------------+
| AŞAMA 4: Windows WinGet & Resmî Paket Depoları                                    |
| - microsoft/winget-pkgs PR gönderimi                                              |
| - winget install SinanSarikaya.TermFetchStudio                                    |
+-----------------------------------------------------------------------------------+
```

---

## 2. README ve Web Sitesi İçin Kurulum Komutları

Tanıtım sitesi ve GitHub README'ye eklenecek temiz, modern kurulum bölümleri:

### 🚀 Arch Linux (AUR)
```bash
# yay ile
yay -S termfetch-studio-bin

# paru ile
paru -S termfetch-studio-bin
```

### 🍎 macOS (Homebrew)
```bash
brew tap sinansarikaya/tap
brew install termfetch-studio
```

### 🪟 Windows (WinGet)
```powershell
winget install SinanSarikaya.TermFetchStudio
```

### 🐧 Debian / Ubuntu
```bash
# GitHub Releases üzerinden son .deb paketini kurun
curl -sLO https://github.com/sinansarikaya/termfetch-studio/releases/latest/download/termfetch-studio_2.0.0_amd64.deb
sudo apt install ./termfetch-studio_2.0.0_amd64.deb
```

### ⚡ Hızlı Kurulum (Linux & macOS Tek Komut)
```bash
curl -fsSL https://raw.githubusercontent.com/sinansarikaya/termfetch-studio/main/install.sh | bash
```

### 📦 Bağımsız Taşınabilir Arşivler (GitHub Releases)
Her release ile birlikte `SHA256SUMS.txt` eşliğinde indirilebilir dosyalar:
- `termfetch-studio-v2.0.0-x86_64-unknown-linux-gnu.tar.gz`
- `termfetch-studio-v2.0.0-x86_64-unknown-linux-musl.tar.gz` (Statik, Alpine & Docker uyumlu)
- `termfetch-studio-v2.0.0-aarch64-apple-darwin.tar.gz` (macOS Apple Silicon)
- `termfetch-studio-v2.0.0-x86_64-apple-darwin.tar.gz` (macOS Intel)
- `termfetch-studio-v2.0.0-x86_64-pc-windows-msvc.zip` (Windows x64 Portable)
- `termfetch-studio_2.0.0_amd64.deb` (Debian/Ubuntu Paketi)

---

## 3. Paket Yöneticisi Bağımlılık Stratejisi

| Paket Yöneticisi | Fastfetch Bağımlılığı | Kurulum/Kaldırma Davranışı |
|---|---|---|
| **AUR** | `depends=('fastfetch')` | Arch resmi deposundaki `extra/fastfetch` otomatik kurulur. Kaldırıldığında kullanıcı konfigürasyonu (`~/.config/termfetch-studio`) silinmez. |
| **Homebrew** | `depends_on "fastfetch"` | `brew install` sırasında Fastfetch formula'sı otomatik çekilir. |
| **WinGet** | `PackageDependencies: Fastfetch-cli.Fastfetch` | WinGet Fastfetch CLI paketini eş zamanlı çözer. |
| **Debian (.deb)** | `Depends: fastfetch` | PPA veya sistem deposundaki fastfetch otomatik yüklenir. |

---

## 4. Yayın Günü Manuel Adım Listesi (Release Runbook)

Yayın yapmaya karar verdiğinizde sırasıyla izlenecek adımlar:

1. **Git Tag Oluşturma ve Push:**
   ```bash
   git tag -a v2.0.0 -m "Release v2.0.0"
   git push origin v2.0.0
   ```
2. **GitHub Actions Bekleme:**
   - `.github/workflows/release.yml` otomatik tetiklenir.
   - Tüm platformlar derlenir, arşivler oluşturulur ve `v2.0.0` GitHub Release altında toplanır.
   - Release sayfasından `SHA256SUMS.txt` kopyalanır.
3. **AUR Yayınlama:**
   ```bash
   git clone ssh://aur@aur.archlinux.org/termfetch-studio-bin.git
   cp packaging/aur/PKGBUILD-bin termfetch-studio-bin/PKGBUILD
   # PKGBUILD içindeki sha256sums değerini SHA256SUMS.txt ile güncelle
   cd termfetch-studio-bin && makepkg --printsrcinfo > .SRCINFO
   git commit -am "chore: release 2.0.0" && git push
   ```
4. **Homebrew Tap Yayınlama:**
   - `sinansarikaya/homebrew-tap` deposunu oluşturun veya açın.
   - `packaging/homebrew/termfetch-studio.rb` dosyasını `Formula/termfetch-studio.rb` olarak ekleyin.
   - SHA256 hash'lerini release çıktıları ile güncelleyin.
5. **WinGet PR Açma:**
   - `https://github.com/microsoft/winget-pkgs` fork'unuza `packaging/winget/manifests/` klasörünü ekleyip PR açın.
