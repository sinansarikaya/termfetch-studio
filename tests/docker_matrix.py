#!/usr/bin/env python3
"""Isolated distribution smoke tests. Requires Docker daemon access on the host.

No host home or Docker socket is mounted in test containers. Run from the repo:
    python3 tests/docker_matrix.py
    python3 tests/docker_matrix.py --image debian:12 --scenario missing
"""
import argparse
import hashlib
import json
import os
import pty
import re
import select
import shutil
import subprocess
import sys
import tempfile
import time
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
IMAGES = ["ubuntu:24.04", "debian:12", "debian:13", "fedora:43",
          "archlinux:latest", "opensuse/tumbleweed:latest"]


def run(*args, timeout=600, check=True):
    result = subprocess.run(args, text=True, stdout=subprocess.PIPE,
                            stderr=subprocess.STDOUT, timeout=timeout)
    if check and result.returncode:
        raise RuntimeError(f"{' '.join(args)} failed ({result.returncode}):\n{result.stdout[-3000:]}")
    return result


def interactive(command, answers, timeout=900):
    """Wait for each exact prompt on a PTY before responding."""
    if command[:2] == ["docker", "exec"] and "-t" not in command:
        command = command[:2] + ["-t"] + command[2:]
    master, slave = pty.openpty()
    process = subprocess.Popen(command, stdin=slave, stdout=slave, stderr=slave,
                               start_new_session=True)
    os.close(slave)
    transcript = bytearray()
    deadline = time.monotonic() + timeout
    pending = list(answers)
    try:
        while process.poll() is None or select.select([master], [], [], 0)[0]:
            if time.monotonic() > deadline:
                process.kill()
                raise TimeoutError(f"Timed out waiting for {pending[:1]}; output: {transcript[-2000:]!r}")
            ready, _, _ = select.select([master], [], [], 0.25)
            if ready:
                try:
                    data = os.read(master, 65536)
                except OSError:
                    break
                transcript.extend(data)
                if os.environ.get("TFS_DEBUG"):
                    sys.stderr.write(data.decode(errors="replace"))
                    sys.stderr.flush()
                if pending and pending[0][0].encode() in transcript:
                    _, answer = pending.pop(0)
                    if os.environ.get("TFS_DEBUG"):
                        sys.stderr.write(f"\n[reply {answer!r}]\n")
                    os.write(master, answer.encode())
        process.wait(timeout=5)
    finally:
        os.close(master)
    output = transcript.decode(errors="replace")
    if pending:
        raise AssertionError(f"Missing prompt {pending[0][0]!r}; output:\n{output[-4000:]}")
    if process.returncode:
        raise AssertionError(f"Interactive command failed ({process.returncode}):\n{output[-4000:]}")
    return output


def docker_user(name, *command):
    return ["docker", "exec", "-i", "-u", "test", "-e", "HOME=/home/test",
            "-e", "USER=test", "-e", "TERM=dumb", "-e",
            "PATH=/home/test/.local/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin",
            name, *command]


def user_run(name, script, check=True):
    return run(*docker_user(name, "bash", "-c", script), check=check)


def root_run(name, script):
    return run("docker", "exec", name, "bash", "-c", script)


def setup(name, image, scenario):
    if image.startswith(("ubuntu", "debian")):
        root_run(name, "apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq sudo curl ca-certificates")
    elif image.startswith("fedora"):
        root_run(name, "dnf install -y sudo")
    elif image.startswith("archlinux"):
        root_run(name, "pacman -Sy --noconfirm sudo")
    else:
        root_run(name, "zypper --non-interactive refresh && zypper --non-interactive install sudo")
    root_run(name, "useradd -m -s /bin/bash test && echo 'test ALL=(ALL) NOPASSWD: ALL' > /etc/sudoers.d/test && chmod 0440 /etc/sudoers.d/test && printf 'keep-me\\n' > /home/test/unrelated.txt && chown test:test /home/test/unrelated.txt")
    if scenario == "preinstalled":
        if image.startswith(("ubuntu", "debian")):
            root_run(name, "curl -fL --retry 3 -o /tmp/fastfetch.deb https://github.com/fastfetch-cli/fastfetch/releases/download/2.53.0/fastfetch-linux-amd64.deb && dpkg -i /tmp/fastfetch.deb")
        elif image.startswith("fedora"):
            root_run(name, "dnf install -y fastfetch")
        elif image.startswith("archlinux"):
            root_run(name, "pacman -S --noconfirm fastfetch")
        else:
            root_run(name, "zypper --non-interactive install fastfetch")


def check(name, script, label, checks):
    result = user_run(name, "set -e\n" + script, check=False)
    checks[label] = "passed" if result.returncode == 0 else f"failed: {result.stdout[-600:]}"
    return result


def scenario(image, state, keep=False, source_dir=ROOT):
    name = "tfs-matrix-" + uuid.uuid4().hex[:12]
    info = {"image": image, "scenario": state, "checks": {},
            "installer_sha256": hashlib.sha256((source_dir / "install.sh").read_bytes()).hexdigest()}
    try:
        pulled = run("docker", "image", "inspect", image, "--format", "{{index .RepoDigests 0}}")
        info["digest"] = pulled.stdout.strip()
        run("docker", "run", "-d", "--name", name, "--network", "bridge",
            "-v", f"{source_dir}:/src:ro", image, "sleep", "infinity")
        setup(name, image, state)
        info["os_release"] = root_run(name, ". /etc/os-release; echo \"$ID $VERSION_ID $ID_LIKE\"").stdout.strip()
        info["detection_expected"] = ("arch" if image.startswith("arch") else
                                       "suse" if image.startswith("opensuse") else
                                       "fedora" if image.startswith("fedora") else "debian")
        # Obtain the actual function from the installer, while running it against
        # the container's own /etc/os-release.
        detect = "source <(sed -n '/^detect_distro_base()/,/^}/p' /src/install.sh); detect_distro_base"
        actual = user_run(name, detect).stdout.strip()
        info["checks"]["detection"] = "passed" if actual == info["detection_expected"] else f"failed: {actual}"
        info["fastfetch_before"] = user_run(name, "fastfetch --version 2>/dev/null || true").stdout.strip()
        if state == "missing" and info["fastfetch_before"]:
            info["checks"]["initial_state"] = "failed: fastfetch unexpectedly installed"
        else:
            info["checks"]["initial_state"] = "passed"
        prompts = [("Install Nerd Fonts? (Y/n)", "n"),
                   ("Would you like to start TermFetch Studio now? (Y/n):", "n")]
        try:
            output = interactive(docker_user(name, "bash", "/src/install.sh"), prompts)
            info["checks"]["install"] = "passed"
            info["install_tail"] = output[-1000:]
        except Exception as exc:
            info["checks"]["install"] = f"failed: {exc}"
            return info
        info["fastfetch_after"] = user_run(name, "fastfetch --version").stdout.strip()
        info["checks"]["fastfetch_version"] = ("passed" if state == "missing" or
            info["fastfetch_before"] == info["fastfetch_after"] else "failed: preinstalled fastfetch changed")
        check(name, "command -v chafa >/dev/null && (command -v magick >/dev/null || command -v convert >/dev/null)", "optional_dependencies", info["checks"])
        check(name, "test -f ~/.local/share/termfetch-studio/lib/core.sh && test -f ~/.local/share/termfetch-studio/lib/fastfetch.sh && test -x ~/.local/bin/termfetch-studio && test -f ~/.config/termfetch-studio/fastfetch.jsonc", "files", info["checks"])
        check(name, "termfetch-studio --version | grep -q '1.1.0'", "version", info["checks"])
        check(name, "termfetch-studio --help | grep -q -- '--preview'", "help", info["checks"])
        check(name, 'CONFIG_DIR="$HOME/.config/termfetch-studio"; source ~/.local/share/termfetch-studio/lib/core.sh; before=$(cat "$CONFIG_FILE"); awk() { return 1; }; if set_config_value fastfetch preset minimal; then exit 1; fi; test "$(cat "$CONFIG_FILE")" = "$before"', "failed_settings_write_preserves_file", info["checks"])
        check(name, 'bash -c "source ~/.bashrc" >/tmp/noninteractive.out 2>&1 && test ! -s /tmp/noninteractive.out', "noninteractive_shell_quiet", info["checks"])
        check(name, "termfetch-studio --preview >/tmp/preview.out 2>&1", "preview", info["checks"])
        check(name, "fastfetch --config ~/.config/termfetch-studio/fastfetch.jsonc >/tmp/config.out 2>&1", "fastfetch_config", info["checks"])
        check(name, r'''INSTALL_DIR="$HOME/.local/share/termfetch-studio"
CONFIG_DIR="$HOME/.config/termfetch-studio"
source "$INSTALL_DIR/lib/core.sh"
source "$INSTALL_DIR/lib/fastfetch.sh"
for preset in full minimal focused developer gaming custom; do
    "create_${preset}_config" auto "" nord
    fastfetch --config "$CONFIG_DIR/fastfetch.jsonc" --show-errors --pipe true >/tmp/config-errors 2>&1
    if grep -Eq 'Unknown JSON key|JsonConfig Error|Invalid enum' /tmp/config-errors; then
        cat /tmp/config-errors
        exit 1
    fi
done
create_full_config auto "" dracula''', "all_preset_config_keys", info["checks"])
        check(name, r"""INSTALL_DIR="$HOME/.local/share/termfetch-studio"
CONFIG_DIR="$HOME/.config/termfetch-studio"
source "$INSTALL_DIR/lib/core.sh"
source "$INSTALL_DIR/lib/fastfetch.sh"
set_config_value fastfetch use_theme_colors false
for preset in full minimal focused developer gaming custom; do
    for pair in 'theme 31' '32 theme' 'default default'; do
        read -r icon text <<< "$pair"
        set_config_value fastfetch key_color "$icon"
        set_config_value fastfetch value_color "$text"
        "create_${preset}_config" auto "" nord
        fastfetch --config "$CONFIG_DIR/fastfetch.jsonc" --format json >/dev/null
    done
done
set_config_value fastfetch use_theme_colors true
set_config_value fastfetch key_color theme
set_config_value fastfetch value_color theme
create_full_config auto "" dracula""", "live_color_configs", info["checks"])
        check(name, r"""INSTALL_DIR="$HOME/.local/share/termfetch-studio"
CONFIG_DIR="$HOME/.config/termfetch-studio"
source "$INSTALL_DIR/lib/core.sh"
source "$INSTALL_DIR/lib/fastfetch.sh"
set_config_value group_colors os '#ff8c00'
set_config_value group_colors desktop '#40e0d0'
set_config_value group_colors hardware '#8a2be2'
set_config_value group_colors network '#ff69b4'
set_config_value group_colors time '#ffd700'
for preset in full minimal focused developer gaming custom; do
    "create_${preset}_config" auto "" nord
    fastfetch --config "$CONFIG_DIR/fastfetch.jsonc" --format json >/dev/null
done
for group in os desktop hardware network time; do set_config_value group_colors "$group" inherit; done
create_full_config auto "" dracula""", "group_rgb_configs", info["checks"])
        check(name, 'INSTALL_DIR="$HOME/.local/share/termfetch-studio"; CONFIG_DIR="$HOME/.config/termfetch-studio"; CONFIG_FILE="$CONFIG_DIR/config.ini"; IMAGES_DIR="$CONFIG_DIR/images"; source "$INSTALL_DIR/lib/core.sh"; source "$INSTALL_DIR/lib/fastfetch.sh"; image_convert -size 2x2 xc:red /tmp/tfs-logo.png; set_config_value fastfetch logo_type image; set_config_value fastfetch custom_image /tmp/tfs-logo.png; apply_logo_config >/dev/null; fastfetch --config "$CONFIG_DIR/fastfetch.jsonc" --format json >/dev/null; tfs-fastfetch --format json >/dev/null; set_config_value fastfetch logo_type auto; set_config_value fastfetch custom_image ""; apply_logo_config >/dev/null', "image_ascii_fallback_config", info["checks"])
        check(name, r"""INSTALL_DIR="$HOME/.local/share/termfetch-studio"
CONFIG_DIR="$HOME/.config/termfetch-studio"
source "$INSTALL_DIR/lib/core.sh"
source "$INSTALL_DIR/lib/fastfetch.sh"
set_config_value fastfetch logo_type image
set_config_value fastfetch custom_image /tmp/tfs-logo.png
capture=$(mktemp)
export capture
fastfetch() {
    if [ "$1" = --config ]; then cp "$2" "$capture"; fi
    command fastfetch "$@"
}
export -f fastfetch
trap 'rm -f -- "$capture"' EXIT
tfs-fastfetch --format json >/dev/null
grep -q '"source": ".*/logos/tux.txt"' "$capture"
KITTY_WINDOW_ID=1 TERM=xterm-kitty tfs-fastfetch --format json >/dev/null
grep -q '"type": "kitty-direct"' "$capture"
grep -q '"width": 26' "$capture"
grep -q '"height": 13' "$capture"
test -z "$(find "$CONFIG_DIR" -maxdepth 1 -name '.fastfetch.*.jsonc' -print)"
unset -f fastfetch
set_config_value fastfetch logo_type auto
set_config_value fastfetch custom_image ""
apply_logo_config >/dev/null""".replace("/logos/tux.txt", "/logos/arch.txt" if image.startswith("archlinux") else "/logos/tux.txt"), "runtime_logo_rewrite", info["checks"])
        check(name, "CONFIG_DIR=~/.config/termfetch-studio; source ~/.local/share/termfetch-studio/lib/core.sh; image_convert -size 2x2 xc:red /tmp/tfs-test.png && test \"$(image_identify -format '%wx%h' /tmp/tfs-test.png)\" = 2x2", "imagemagick_conversion", info["checks"])
        check(name, "CONFIG_DIR=~/.config/termfetch-studio; source ~/.local/share/termfetch-studio/lib/core.sh; TERM=dumb TERM_PROGRAM='' KITTY_WINDOW_ID='' WEZTERM_EXECUTABLE='' KONSOLE_VERSION='' KONSOLE_DBUS_SERVICE='' GHOSTTY_RESOURCES_DIR='' test \"$(check_terminal_image_support)\" = none", "unknown_terminal_fallback", info["checks"])
        check(name, "CONFIG_DIR=~/.config/termfetch-studio; source ~/.local/share/termfetch-studio/lib/core.sh; TERM_PROGRAM=foot test \"$(TERM_PROGRAM=foot check_terminal_image_support)\" = foot", "terminal_environment", info["checks"])
        # Preserve a user choice across reinstall, then exercise backup round trip.
        check(name, "sed -i 's/^theme=.*/theme=nord/; s/^preset=.*/preset=minimal/' ~/.config/termfetch-studio/config.ini; grep -q '^theme=nord$' ~/.config/termfetch-studio/config.ini", "settings_edit", info["checks"])
        check(name, '''file="$HOME/.config/termfetch-studio/fastfetch.jsonc"
awk '!done && /"keyWidth":/ { print $0 ","; print "            \\"keyShow\\": false"; done=1; next } {print}' "$file" > /tmp/legacy.jsonc
cp /tmp/legacy.jsonc "$file"
cp "$file" "$HOME/legacy-original.jsonc"''', "legacy_config_fixture", info["checks"])
        try:
            interactive(docker_user(name, "bash", "/src/install.sh"), prompts)
            check(name, "grep -q '^theme=nord$' ~/.config/termfetch-studio/config.ini && grep -q '^preset=minimal$' ~/.config/termfetch-studio/config.ini", "reinstall_preserves_settings", info["checks"])
            check(name, '''! grep -q '"keyShow"' ~/.config/termfetch-studio/fastfetch.jsonc
set -- ~/.config/termfetch-studio/backups/legacy-fastfetch.*.jsonc
test "$(sha256sum "$HOME/legacy-original.jsonc" | cut -d' ' -f1)" = "$(sha256sum "$1" | cut -d' ' -f1)"
fastfetch --config ~/.config/termfetch-studio/fastfetch.jsonc --show-errors --pipe true > /tmp/migrated-output
! grep -E 'Unknown JSON key|JsonConfig Error|Invalid enum' /tmp/migrated-output''', "legacy_config_migration", info["checks"])
        except Exception as exc:
            info["checks"]["reinstall"] = f"failed: {exc}"
        check(name, "termfetch-studio --backup >/tmp/backup.out && set -- ~/.config/termfetch-studio/backups/*.tar.gz; test -f \"$1\" && tar -tzf \"$1\" >/tmp/backup-list && ! grep -q '^./backups' /tmp/backup-list", "backup_excludes_backups", info["checks"])
        check(name, "set -- ~/.config/termfetch-studio/backups/*.tar.gz; mv -- \"$1\" ~/.config/termfetch-studio/backups/'backup with spaces.tar.gz' && termfetch-studio --backup >/tmp/backup2.out && for f in ~/.config/termfetch-studio/backups/*.tar.gz; do tar -tzf \"$f\" >/tmp/backup-list || exit 1; ! grep -q '^./backups' /tmp/backup-list || exit 1; done", "second_backup_no_recursion", info["checks"])
        check(name, "sed -i 's/^theme=.*/theme=dracula/' ~/.config/termfetch-studio/config.ini", "mutate_settings", info["checks"])
        try:
            interactive(docker_user(name, "bash", "-c", "termfetch-studio --restore"), [("Enter backup number to restore:", "1\n")])
            check(name, "grep -q '^theme=nord$' ~/.config/termfetch-studio/config.ini", "restore_round_trip", info["checks"])
            invalid = user_run(name, "printf 'oops\\n' | termfetch-studio --restore", check=False)
            info["checks"]["invalid_restore_choice"] = ("passed" if invalid.returncode != 0 and
                "Invalid choice" in invalid.stdout else f"failed: rc={invalid.returncode} {invalid.stdout[-300:]}")
        except Exception as exc:
            info["checks"]["restore_round_trip"] = f"failed: {exc}"
        try:
            interactive(docker_user(name, "bash", "-c", "termfetch-studio uninstall"),
                        [("Are you sure you want to continue? (y/N):", "y"),
                         ("Remove config directory? (y/N):", "n")])
            check(name, "test ! -e ~/.local/bin/termfetch-studio && test ! -e ~/.local/share/termfetch-studio && test -f ~/.config/termfetch-studio/config.ini && test \"$(cat ~/unrelated.txt)\" = keep-me", "uninstall_preserves_unrelated", info["checks"])
        except Exception as exc:
            info["checks"]["uninstall"] = f"failed: {exc}"
        return info
    except Exception as exc:
        info["blocked"] = str(exc)
        return info
    finally:
        if not keep:
            run("docker", "rm", "-f", name, check=False)
        else:
            info["container"] = name


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--image", choices=IMAGES, action="append")
    parser.add_argument("--scenario", choices=["missing", "preinstalled"], action="append")
    parser.add_argument("--keep", action="store_true")
    parser.add_argument("--source-dir", type=Path, default=ROOT,
                        help="Directory to mount as /src (for baseline comparison)")
    parser.add_argument("--output", default="tests/docker-results.json")
    args = parser.parse_args()
    run("docker", "info", "--format", "{{.ServerVersion}}", timeout=15)
    results = []
    with tempfile.TemporaryDirectory(prefix="tfs-source-") as temporary:
        snapshot = Path(temporary)
        snapshot.chmod(0o755)
        shutil.copy2(args.source_dir / "install.sh", snapshot / "install.sh")
        (snapshot / "install.sh").chmod(0o644)
        if (args.source_dir / "preset-logos").is_dir():
            shutil.copytree(args.source_dir / "preset-logos", snapshot / "preset-logos")
            (snapshot / "preset-logos").chmod(0o755)
        for image in args.image or IMAGES:
            for state in args.scenario or ["missing", "preinstalled"]:
                print(f"Running {image} {state}", flush=True)
                result = scenario(image, state, args.keep, snapshot)
                results.append(result)
                print(json.dumps(result, ensure_ascii=False)[-1200:], flush=True)
                Path(args.output).write_text(json.dumps(results, indent=2, ensure_ascii=False) + "\n")
    return int(any(r.get("blocked") or any(v != "passed" for v in r["checks"].values()) for r in results))


if __name__ == "__main__":
    sys.exit(main())
