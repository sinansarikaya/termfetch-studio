# TermFetch Studio v2 — Cross-Platform Compatibility Report

**Hazırlayan:** Takım Lideri & Release Engineering Sorumlusu  
**Tarih:** 25 Ekim 2026  
**Dal:** `feat/v2-tui-rewrite`  
**Referans Commit:** `1875a68` + Cross-Platform Hardening  

---

## 1. Yönetici Özeti

TermFetch Studio v2'nin Linux, macOS ve Windows platformlarında güvenilir, taşınabilir ve kesintisiz çalışması için kod denetimi, platform bağımsızlık düzeltmeleri ve otomatik doğrulama altyapısı tamamlanmıştır.

Mevcut 67 teste ek olarak **8 adet platformlar arası entegrasyon ve PTY lifecycle testi** eklenmiş, toplam test sayısı **75'e** çıkarılmıştır ve tüm testler hatasız geçmektedir (`75 passed; 0 failed`).

---

## 2. Platform ve Dağıtım Uyumluluk Matrisi

| Platform / Dağıtım | Mimari | Destek Durumu | Fastfetch Keşfi | Görüntü Protokolü | Notlar / Gereksinimler |
|---|---|---|---|---|---|
| **Arch Linux / CachyOS** | x86_64, aarch64 | **Tam Destek (Doğrulandı)** | `/usr/bin/fastfetch` (pacman) | Kitty, Sixel, Chafa | Test edildi; yerel glibc 2.41 ile sıfır sorun |
| **Ubuntu 24.04 / 22.04** | x86_64, arm64 | **Tam Destek (Doğrulandı)** | PPA / apt | Kitty, Sixel, Chafa | Container testinden başarıyla geçti |
| **Debian 13 (Trixie)** | x86_64, arm64 | **Tam Destek (Doğrulandı)** | `/usr/bin/fastfetch` | Sixel, Chafa | Container testinden başarıyla geçti |
| **Debian 12 (Bookworm)** | x86_64, arm64 | **Destekleniyor (Musl/Older Glibc)** | `/usr/bin/fastfetch` | Sixel, Chafa | Standart glibc derlemesi glibc >= 2.39 gerektirir; musl-static ikili dosyası ile %100 uyumludur |
| **Fedora 40/41/43** | x86_64, aarch64 | **Tam Destek (Doğrulandı)** | dnf (`/usr/bin/fastfetch`) | Kitty, Sixel, Chafa | Container testinden başarıyla geçti |
| **openSUSE Tumbleweed** | x86_64 | **Tam Destek (Doğrulandı)** | zypper | Sixel, Chafa | Container testinden başarıyla geçti |
| **Alpine Linux** | x86_64, aarch64 | **Tam Destek (Musl Static)** | `/usr/bin/fastfetch` (apk) | Sixel, Chafa | Musl statik ikili dosyası (`x86_64-unknown-linux-musl`) tam uyumludur |
| **macOS (Apple Silicon)** | aarch64 (M1-M4) | **Tam Destek (Hazır)** | `/opt/homebrew/bin/fastfetch` | Kitty (Ghostty/Kitty), iTerm2 | Standart Homebrew konumu otomatik taranır |
| **macOS (Intel)** | x86_64 | **Tam Destek (Hazır)** | `/usr/local/bin/fastfetch` | iTerm2, Kitty, Chafa | Homebrew & MacPorts yolları çözülür |
| **Windows 11 / 10** | x64 | **Tam Destek (Hazır)** | WinGet, Scoop, Choco, PATH | Chafa, Sixel (Windows Terminal) | Dosya kilitleme ve `%APPDATA%` yolları düzeltildi |

---

## 3. Yapılan Kritik Cross-Platform İyileştirmeleri

### 3.1. Atomik Dosya Yazma ve Windows Paylaşım Kilidi (Sharing Violation Fix)
- **Sorun:** `src/adapters/fsutil.rs` içindeki `atomic_write` fonksiyonunda geçici dosyayı açan `fs::File` tutamacı (`f`), `fs::rename` çağrılmadan önce serbest bırakılmıyordu. POSIX sistemlerinde (Linux/macOS) açık dosya yeniden adlandırılabilirken, Windows dosya sistemi `ERROR_SHARING_VIOLATION` (kod 32 / OS error 5) hatası verir.
- **Düzeltme:** Dosya yazma ve `sync_all()` işlemi açık bir blok içerisine alınarak `drop(f)` ile kapatıldı; ardından Windows üzerinde hedef dosyanın önceden silinmesi sağlanarak atomik değiştirme garantilendi.

### 3.2. Fastfetch Çoklu Konum ve Paket Yöneticisi Keşfi
- **Sorun:** `runner::version()`, `probe_real_info()` ve `run_live()` doğrudan `Command::new("fastfetch")` çağırıyordu. GUI terminallerinde (özellikle macOS Apple Silicon Homebrew `/opt/homebrew/bin` veya Windows Scoop/WinGet shims) `PATH` ortam değişkeninde fastfetch bulunmadığında uygulama fastfetch'i algılayamıyordu.
- **Düzeltme:** `find_fastfetch()` ve `fastfetch_cmd()` modülü eklendi:
  - `FASTFETCH_BIN` ortam değişkeni önceliği
  - Mevcut `PATH` dizinleri
  - macOS Homebrew (`/opt/homebrew/bin`, `/usr/local/bin`) ve MacPorts (`/opt/local/bin`)
  - Windows WinGet (`%LOCALAPPDATA%\Microsoft\WinGet\Links`), Scoop (`%USERPROFILE%\scoop\shims`), Chocolatey (`C:\ProgramData\chocolatey\bin`) ve Standart Program Files
  - Linux `~/.local/bin`, `~/.nix-profile/bin`, `/usr/local/bin`

### 3.3. Windows Konfigürasyon Dizini Önceliği (`%APPDATA%` Fallback)
- **Sorun:** Fastfetch Linux/macOS üzerinde `~/.config/fastfetch` kullanırken, Windows üzerinde hem `%APPDATA%\fastfetch\config.jsonc` hem de `%USERPROFILE%\.config\fastfetch\config.jsonc` yollarını okur. Eğer kullanıcının mevcut konfigürasyonu `%APPDATA%` altında bulunuyorsa yapılan ayarlar Fastfetch tarafından görünmüyordu.
- **Düzeltme:** `Paths::detect()` fonksiyonuna Windows kontrolü eklendi; `%APPDATA%\fastfetch\config.jsonc` mevcutsa doğrudan hedef olarak seçilir, aksi halde standart XDG/userprofile köküne gidilir.

### 3.4. Sistem Bilgisi ve Saat Dilimi Ofseti (Platform-Agnostic SysInfo)
- **Sorun:** Linux'ta `date +%z` komutu çalıştırılıyordu. Windows üzerinde `date` komutu harici bir çalıştırılabilir ikili dosya değil, `cmd.exe` dahili komutudur; dolayısıyla `Command::new("date")` hata (os error 2) veriyordu. Ayrıca Windows ve macOS için dağıtım ikonları bulunmuyordu.
- **Düzeltme:**
  - Unix sistemlerinde `date +%z`, Windows sistemlerinde PowerShell TimeZone sorgusu (`[System.TimeZoneInfo]::Local.GetUtcOffset`) kullanılarak saat ofseti hatasız hesaplandı.
  - Dağıtım adları: macOS için `"macOS"`, Windows için `"Windows"` olarak standardize edildi.
  - Nerd Font ikonları: Windows için `\u{f05be}`, openSUSE için `\u{f08e8}`, Alpine için `\u{f08cd}`, NixOS için `\u{f08e4}` ve macOS için `\u{f0035}` eklendi.

### 3.5. PowerShell Profil Tespiti ve Font Denetimi (Doctor)
- **Sorun:** Doctor ekranındaki kabuk entegrasyonu yalnızca `.bashrc`, `.zshrc` ve `.config/fish` dosyalarını denetliyordu. Windows'ta PowerShell profili (`Documents\PowerShell\Microsoft.PowerShell_profile.ps1`) kontrol edilmiyordu. Ayrıca macOS ve Windows'ta `fc-list` bulunmadığında font denetimi yapılamıyordu.
- **Düzeltme:** PowerShell profilleri eklendi. `fc-list` bulunmadığında `~/Library/Fonts`, `/Library/Fonts`, `C:\Windows\Fonts` ve `%LOCALAPPDATA%\Microsoft\Windows\Fonts` doğrudan taranarak Nerd Font varlığı tespit edilir.

---

## 4. Terminal Emülatörleri Uyumluluk Tablosu

| Terminal Emülatörü | Platform | TrueColor | Unicode 15 | Kitty Graphics | Sixel Graphics | Chafa / Half-block |
|---|---|---|---|---|---|---|
| **Ghostty** | Linux / macOS | Evet | Evet | Evet (Direct) | Hayır | Evet |
| **Kitty** | Linux / macOS | Evet | Evet | Evet (Direct) | Hayır | Evet |
| **WezTerm** | Linux / macOS / Windows | Evet | Evet | Evet | Hayır | Evet |
| **Alacritty** | Linux / macOS / Windows | Evet | Evet | Hayır | Hayır | Evet (Half-block fallback) |
| **iTerm2** | macOS | Evet | Evet | Hayır | Hayır (iTerm protocol) | Evet |
| **Apple Terminal** | macOS | Sınırlı | Evet | Hayır | Hayır | Evet (ANSI half-block) |
| **Windows Terminal** | Windows 10/11 | Evet | Evet | Hayır | Evet (v1.22+) | Evet (Chafa/Half-block) |
| **Foot** | Linux (Wayland) | Evet | Evet | Hayır | Evet | Evet |
| **Konsole** | Linux | Evet | Evet | Hayır | Hayır (iTerm protocol) | Evet |
