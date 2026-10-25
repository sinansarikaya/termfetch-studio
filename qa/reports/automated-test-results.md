# TermFetch Studio v2 — Automated Test Results & CI Matrix Report

**Tarih:** 25 Ekim 2026  
**Test Suite Sürümü:** v2.0.0 (feat/v2-tui-rewrite)  
**Toplam Test Sayısı:** 75  
**Başarılı (PASS):** 75  
**Başarısız (FAIL):** 0  
**Yoksayılan (IGNORED):** 0  
**Clippy & Format:** 0 Warning / Clean  

---

## 1. Test Paketleri Dağılımı ve Sonuçları

```text
running unittests src/lib.rs ............................................ 30 passed
running tests/audit_and_regression_tests.rs .............................. 7 passed
running tests/cli_tests.rs ............................................... 2 passed
running tests/config_tests.rs ............................................ 6 passed
running tests/contrast_tests.rs .......................................... 3 passed
running tests/cross_platform_tests.rs .................................... 8 passed [YENİ]
running tests/theme_tests.rs ............................................. 7 passed
running tests/tui_render_tests.rs ....................................... 12 passed
-----------------------------------------------------------------------------------
TOPLAM: 75 passed; 0 failed; 0 ignored; 0 filtered out
```

---

## 2. Yeni Eklenen Cross-Platform Test Paketi (`tests/cross_platform_tests.rs`)

| Test Adı | Kapsanan İşlev | Sonuç | Süre |
|---|---|---|---|
| `test_paths_sandboxing_and_targets` | XDG, APPDATA, `--root` izolasyonu ve hedef dosya tespiti | PASS | <1 ms |
| `test_atomic_write_cross_platform_overwrites_and_cleans_up` | Windows paylaşım kilidi (drop handle), üzerine yazma, artık dosya temizliği | PASS | 1 ms |
| `test_fastfetch_binary_resolution_with_env_override` | `FASTFETCH_BIN`, Homebrew, WinGet ve PATH çözümleyici | PASS | 1 ms |
| `test_probe_output_parser_with_crlf_and_lf` | Windows (`\r\n`) ve Unix (`\n`) satır sonları ile gerçek spec ayrıştırma | PASS | <1 ms |
| `test_sysinfo_icons_and_wallclock_offsets` | Çoklu OS dağıtım ikonları ve negatif/pozitif UTC saat dilimi hesaplama | PASS | <1 ms |
| `test_image_protocol_detection` | Ortam değişkenlerine göre Kitty / iTerm / Sixel / Chafa tespiti | PASS | <1 ms |
| `test_headless_pty_terminal_render_lifecycle` | PTY / Ratatui TestBackend üzerinde ekran döngüleri, resize (80x24 -> 180x60) ve state yönetimi | PASS | 36 ms |
| `test_real_vs_mock_fastfetch_execution` | Gerçek fastfetch varlığında probe çalıştırma; yokluğunda mock ayrıştırma | PASS | 2 ms |

---

## 3. Gerçek vs Mock Fastfetch Test Doğrulaması

1. **Mock Fastfetch Testleri:**
   - CI ortamında veya Fastfetch kurulu olmayan sistemlerde testler kesilmeden çalışır.
   - `test_probe_output_parser_with_crlf_and_lf` mock veri üzerinde test edilir.
   - `title_has_no_key_and_demo_mode_controls_formats` ve JSONC jeneratör testleri Fastfetch ikili dosyası gerektirmeden tam kapsamlı doğrulanır.
2. **Gerçek Fastfetch Testleri:**
   - `generated_config_runs_in_fastfetch_when_available` testi host üzerinde Fastfetch varsa gerçek `--pipe` çıktısını test eder.
   - `test_real_vs_mock_fastfetch_execution` Fastfetch binary'si algılandığında dinamik olarak gerçek komutu koşturur.

---

## 4. Linux Dağıtımları İkili Çalıştırma Testleri (Docker Container Tests)

Aşağıdaki resmi konteyner imajlarında derlenen `termfetch-studio` ikili dosyası (`target/release/termfetch-studio`) bağımsız olarak çalıştırılmıştır:

| Dağıtım / Konteyner | Sürüm | Komut | Sonuç | Gözlemler |
|---|---|---|---|---|
| **Arch Linux** | `archlinux:latest` | `termfetch-studio --help` | **PASS (0)** | Kusursuz çalıştı. |
| **Ubuntu** | `ubuntu:24.04` | `termfetch-studio --version` | **PASS (0)** | Çıktı: `termfetch-studio 2.0.0` |
| **Debian** | `debian:13 (Trixie)` | `termfetch-studio --version` | **PASS (0)** | Çıktı: `termfetch-studio 2.0.0` |
| **Fedora** | `fedora:43` | `termfetch-studio --version` | **PASS (0)** | Çıktı: `termfetch-studio 2.0.0` |
| **openSUSE** | `opensuse/tumbleweed:latest` | `termfetch-studio --version` | **PASS (0)** | Çıktı: `termfetch-studio 2.0.0` |
| **Debian 12** | `debian:12 (Bookworm)` | `termfetch-studio --version` | **Glibc Gereksinimi** | Host Arch Linux (glibc 2.41) derlemesi glibc 2.39 sembolü aradı. Musl static derleme ile çözüldü. |
| **Alpine Linux** | `alpine:latest` | `termfetch-studio --version` | **Musl Gereksinimi** | Musl loader gerektirir. CI'a eklenen `x86_64-unknown-linux-musl` hedefi ile çözüldü. |

---

## 5. GitHub Actions CI Matrisi Yapılandırması (`.github/workflows/ci.yml`)

1. **fmt Job:** `ubuntu-latest` üzerinde `cargo fmt --check`
2. **ci Job (Matris):**
   - `ubuntu-latest`: Fastfetch PPA kurulumu, Clippy, 75 test, Release glibc ikili derlemesi + `x86_64-unknown-linux-musl` statik ikili derlemesi.
   - `macos-latest`: Homebrew Fastfetch kurulumu, Clippy, 75 test, Release universal ikili derlemesi.
   - `windows-latest`: WinGet Fastfetch kurulumu / mock fallback, Clippy, 75 test, Release `termfetch-studio.exe` derlemesi.
