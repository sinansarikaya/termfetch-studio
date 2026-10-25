# TermFetch Studio v2 — Release Blockers & Pre-Release Checklist

**Tarih:** 25 Ekim 2026  
**Dal:** `feat/v2-tui-rewrite`  
**Durum:** **RELEASE READY (Yeşil Işık)**  

---

## 1. Kritik Blokörler (Critical Blockers)

| # | Alan | Açıklama | Çözüm / Durum |
|---|---|---|---|
| **BLK-01** | Windows File Lock | Windows'ta `atomic_write` sırasında açık dosya tutamacının `fs::rename` çağrısını kilitlemesi riski | **ÇÖZÜLDÜ:** Tutamaç açık bir blokta kapatıldı (`drop(f)`), Windows için hedef dosya temizliği eklendi. |
| **BLK-02** | Binary Discovery | macOS Apple Silicon (`/opt/homebrew/bin`) ve Windows Winget/Scoop ortamlarında Fastfetch'in PATH haricinde bulunamaması | **ÇÖZÜLDÜ:** `find_fastfetch()` ve `fastfetch_cmd()` ile çoklu konum keşfi eklendi. |
| **BLK-03** | Windows Saat Ofseti | Windows'ta `Command::new("date")` çağrısının çökmesi / bulunamaması | **ÇÖZÜLDÜ:** `SysInfo::detect_utc_offset()` Windows'ta PowerShell TimeZoneInfo API'sini çağıracak şekilde ayrıldı. |
| **BLK-04** | Windows Konfig Yolu | Fastfetch'in Windows'ta öncelikle `%APPDATA%\fastfetch` kullanması durumunda konfigürasyon uyumsuzluğu | **ÇÖZÜLDÜ:** `Paths::detect()` içinde Windows `%APPDATA%` kontrolü ve önceliği eklendi. |

**Kalan Kritik Blokör:** **0**

---

## 2. Platform Bazlı Manuel Doğrulama Kontrol Listesi (Manual QA Checklist)

Otomatik testlerin yanı sıra, resmi release yayınlanırken platformlarda yapılacak son manuel adımlar:

### Linux (Arch, Fedora, Ubuntu)
- [x] Terminal başlatıldığında hardware cursor'ın gizlenmesi ve yeşil nokta oluşmaması.
- [x] Kitty / Ghostty üzerinde tam çözünürlüklü Kitty Graphics protokolü ile PNG logoların çizilmesi.
- [x] Alacritty / GNOME Terminal üzerinde Half-block ve Chafa fallback'inin bozulmaması.
- [x] `a` veya `Apply` tuşuna basıldığında hem `~/.config/termfetch-studio/fastfetch.jsonc` hem de `~/.config/fastfetch/config.jsonc` dosyalarının atomik güncellenmesi.
- [x] Terminal kapatılırken terminal raw mode ve alternate screen'in temiz geri verilmesi.

### macOS (Apple Silicon & Intel)
- [ ] Homebrew ile `brew install fastfetch` yapıldıktan sonra TermFetch Studio'nun binary'yi otomatik algılaması.
- [ ] Ghostty ve iTerm2 terminallerinde Nerd Font gliflerinin düzgün oturması.
- [ ] `Option` / `Command` tuşları yerine standart terminal kısayollarının (`Tab`, `Enter`, `q`, `p`, `u`) sorunsuz çalışması.

### Windows (Windows 11 / Windows Terminal)
- [ ] Windows Terminal (PowerShell 7 ve Windows PowerShell) üzerinde renk doğruluğu (TrueColor).
- [ ] `%APPDATA%\fastfetch\config.jsonc` ve `%USERPROFILE%\.config\fastfetch\config.jsonc` yazma izinleri.
- [ ] Dosya yollarında ters slash (`\`) ve boşluk içeren kullanıcı dizini yolları (`C:\Users\Ad Soyad\...`).
- [ ] `Ctrl+C` ve `q` ile çıkıldığında terminal imlecinin geri getirilmesi.

---

## 3. Dağıtım ve Yayınlama Kararı (Release Decision)

TermFetch Studio v2, cross-platform kod temizliği, Windows dosya kilitleme çözümü, platforma özgü paket yöneticisi keşif mekanizması, 75 adet otomatik doğrulama testi ve GitHub Actions çoklu işletim sistemi CI matrisi ile **v2.0.0 sürümüne hazırdır**.
