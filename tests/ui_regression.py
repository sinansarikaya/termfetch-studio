#!/usr/bin/env python3
"""Black-box generated Bash UI tests. Run in Docker, with python3-pyte/ImageMagick."""
import binascii
import codecs
import fcntl
import json
import os
import pty
import re
import select
import signal
import struct
import subprocess
import tempfile
import termios
import time
import unittest
import zlib
from pathlib import Path
import pyte

ROOT = Path(__file__).resolve().parents[1]
SOURCE = (ROOT / 'install.sh').read_text()


def generated(marker):
    return re.search(r"<< '" + marker + r"'\n(.*?)\n" + marker + r'\n', SOURCE, re.S).group(1) + '\n'


def png(path, width, height):
    def chunk(tag, data):
        return struct.pack('>I', len(data)) + tag + data + struct.pack('>I', binascii.crc32(tag + data) & 0xffffffff)
    raw = (b'\x00' + b'\xff\xcc\x00' * width) * height
    path.write_bytes(b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0)) + chunk(b'IDAT', zlib.compress(raw)) + chunk(b'IEND', b''))


class Terminal:
    def __init__(self, case, body, columns=80, rows=24):
        self.master, slave = pty.openpty()
        self.slave = slave
        self.columns, self.rows = columns, rows
        self.screen = pyte.HistoryScreen(columns, rows, history=1000)
        self.stream = pyte.Stream(self.screen)
        self.decoder = codecs.getincrementaldecoder("utf-8")(errors="replace")
        self.raw = b''
        self.resize(columns, rows)
        self.initial_attributes = termios.tcgetattr(slave)
        env = os.environ | {'HOME': str(case.home), 'TMPDIR': str(case.home), 'TERM': 'xterm-256color', 'LC_ALL': 'C.UTF-8'}
        env.pop('LINES', None); env.pop('COLUMNS', None)
        script = case.home / 'menu.sh'
        script.write_text('source "' + str(case.core) + '"\n' + body)
        self.process = subprocess.Popen(['bash', str(script)], stdin=slave, stdout=slave, stderr=slave,
                                        env=env, start_new_session=True,
                                        preexec_fn=lambda: fcntl.ioctl(0, termios.TIOCSCTTY, 0))
        self.read(0.3)

    def resize(self, columns, rows):
        self.columns, self.rows = columns, rows
        fcntl.ioctl(self.master, termios.TIOCSWINSZ, struct.pack('HHHH', rows, columns, columns * 10, rows * 22))
        self.screen.resize(lines=rows, columns=columns)

    def read(self, duration=0.15):
        deadline = time.monotonic() + duration
        start = len(self.raw)
        while time.monotonic() < deadline:
            if select.select([self.master], [], [], 0.02)[0]:
                try:
                    data = os.read(self.master, 65536)
                except OSError:
                    break
                self.raw += data
                # pyte 0.8 aborts a whole text run on a non-combining VS16.
                # Feed codepoints separately so the following label is retained.
                for character in self.decoder.decode(data):
                    self.stream.feed(character)
        return self.raw[start:]

    def send(self, keys):
        os.write(self.master, keys)
        return self.read()

    @property
    def text(self):
        return '\n'.join(self.screen.display)

    def finish(self, keys=b'q'):
        self.send(keys)
        deadline = time.monotonic() + 3
        while self.process.poll() is None and time.monotonic() < deadline:
            self.read(.05)
        result = self.process.wait(timeout=.1)
        self.read()
        return result

    def close(self):
        if self.process.poll() is None:
            os.killpg(self.process.pid, signal.SIGKILL)
            self.process.wait(timeout=3)
        os.close(self.master); os.close(self.slave)


class UIRegression(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='tfs-ui-')
        self.home = Path(self.temp.name)
        self.core = self.home / 'core.sh'
        self.core.write_text(generated('CORELIB'))
        self.sessions = []

    def tearDown(self):
        for session in self.sessions:
            session.close()
        self.temp.cleanup()

    def terminal(self, body, columns=80, rows=24):
        session = Terminal(self, body, columns, rows)
        self.sessions.append(session)
        return session

    def bash(self, body):
        return subprocess.run(['bash', '-c', 'CONFIG_DIR="$HOME/config"; source "$HOME/core.sh";\n' + body],
                              env=os.environ | {'HOME': str(self.home), 'LC_ALL': 'C.UTF-8'},
                              text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=5)

    def test_library_source_is_quiet_and_preserves_traps(self):
        result = self.bash('printf SOURCE_OK')
        self.assertEqual((result.returncode, result.stdout, result.stderr), (0, 'SOURCE_OK', ''))
        result = subprocess.run(['bash', '-c', 'trap "printf ORIGINAL_EXIT" EXIT; source "$HOME/core.sh"'],
                                env=os.environ | {'HOME': str(self.home)}, capture_output=True, text=True)
        self.assertEqual(result.stdout, 'ORIGINAL_EXIT')

    def test_dimensions_are_pixel_aware_and_bounded(self):
        for name, width, height in [('square', 400, 400), ('wide', 400, 200), ('tall', 200, 400)]:
            path = self.home / (name + '.png'); png(path, width, height)
            result = self.bash('get_terminal_cell_dimensions() { echo "10 22"; };\ncompute_logo_dimensions "$HOME/' + name + '.png" 20 14')
            self.assertEqual(result.returncode, 0, result.stderr)
            cols, rows = map(int, result.stdout.split())
            self.assertTrue(1 <= cols <= 20 and 1 <= rows <= 14)
            self.assertLess(abs((cols * 10 / (rows * 22)) / (width / height) - 1), 0.05)
        result = self.bash('compute_logo_dimensions "$HOME/missing.png" 20 14')
        self.assertRegex(result.stdout, r'^\d+ \d+\n$')

    def test_dimensions_use_real_pty_winsize(self):
        path = self.home / 'square.png'; png(path, 400, 400)
        session = self.terminal('get_terminal_cell_dimensions > "$HOME/cells"\ncompute_logo_dimensions "$HOME/square.png" 20 14 > "$HOME/dims"')
        self.assertEqual(session.process.wait(timeout=3), 0)
        self.assertEqual((self.home / 'cells').read_text().strip(), '10 22')
        self.assertEqual((self.home / 'dims').read_text().strip(), '20 9')

    def test_runtime_wrapper_and_all_generated_presets(self):
        lib = self.home / '.local/share/termfetch-studio/lib'; lib.mkdir(parents=True)
        (lib / 'core.sh').write_text(generated('CORELIB')); (lib / 'fastfetch.sh').write_text(generated('FASTFETCHLIB'))
        bindir = self.home / '.local/bin'; bindir.mkdir(parents=True)
        wrapper = bindir / 'tfs-fastfetch'; wrapper.write_text(generated('WRAPPER')); wrapper.chmod(0o755)
        fake = bindir / 'fastfetch'
        fake.write_text('#!/usr/bin/python3\nimport sys,re,json\np=sys.argv[sys.argv.index("--config")+1]\ns=open(p).read()\ns=re.sub(r"(?m)^\\s*//.*$","",s)\ns=re.sub(r",(\\s*[}\\]])",r"\\1",s)\nprint(json.dumps(json.loads(s)))\n')
        fake.chmod(0o755)
        config = self.home / '.config/termfetch-studio'; config.mkdir(parents=True)
        image = self.home / 'square.png'; png(image, 400, 400)
        (config / 'config.ini').write_text('[colors]\ntheme=nord\n[fastfetch]\nlogo_type=image\ncustom_image=' + str(image) + '\n')
        env = os.environ | {'HOME': str(self.home), 'PATH': str(bindir) + ':' + os.environ['PATH'], 'KITTY_WINDOW_ID': '1', 'TERM': 'xterm-kitty'}
        code = 'CONFIG_DIR="$HOME/.config/termfetch-studio"; INSTALL_DIR="$HOME/.local/share/termfetch-studio"; source "$INSTALL_DIR/lib/core.sh"; source "$INSTALL_DIR/lib/fastfetch.sh"; '
        for preset in ['full', 'minimal', 'focused', 'developer', 'gaming', 'custom']:
            result = subprocess.run(['bash', '-c', code + 'create_' + preset + '_config image "$HOME/square.png" nord; tfs-fastfetch'],
                                    env=env, capture_output=True, text=True, timeout=5)
            self.assertEqual(result.returncode, 0, result.stderr)
            document = json.loads(result.stdout)
            formats = [m.get('format', '') for m in document['modules'] if isinstance(m, dict)]
            dots = [v for v in formats if '●' in v]
            for value in dots:
                self.assertEqual(value.count('●'), 8)
                self.assertEqual(len(re.findall(r'\x1b\[1;38;2;[0-9]+;[0-9]+;[0-9]+m', value)), 8)
                self.assertNotIn('#', value)
            if preset == 'minimal':
                self.assertTrue(dots, 'minimal color dots must be validated')
            value = document['logo']
            self.assertEqual((value['type'], value['width'], value['height']), ('kitty-direct', 26, 13))
            self.assertEqual(value['padding']['right'] > 0, True)

    def test_concurrent_wrappers_keep_private_configs_and_clean_up(self):
        self.test_runtime_wrapper_and_all_generated_presets()
        bindir = self.home / '.local/bin'
        fake = bindir / 'fastfetch'
        fake.write_text('#!/usr/bin/python3\nimport sys,time,json\np=sys.argv[sys.argv.index("--config")+1]\ntime.sleep(.3)\ns=open(p).read()\nprint(json.dumps({"path":p,"config":s}))\nsys.exit(7 if "--fail" in sys.argv else 0)\n')
        env = os.environ | {'HOME': str(self.home), 'PATH': str(bindir) + ':' + os.environ['PATH'], 'KITTY_WINDOW_ID': '1', 'TERM': 'xterm-kitty'}
        commands = [subprocess.Popen(['bash', str(bindir / 'tfs-fastfetch')], env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True) for _ in range(3)]
        values = []
        for process in commands:
            out, err = process.communicate(timeout=5)
            self.assertEqual(process.returncode, 0, err)
            values.append(json.loads(out))
        self.assertEqual(len({v['path'] for v in values}), 3)
        for value in values:
            self.assertFalse(Path(value['path']).exists())
            self.assertIn('"logo"', value['config'])
        failed = subprocess.run(['bash', str(bindir / 'tfs-fastfetch'), '--fail'], env=env, capture_output=True, text=True, timeout=5)
        self.assertEqual(failed.returncode, 7)
        self.assertFalse(Path(json.loads(failed.stdout)['path']).exists())

    def test_runtime_wrapper_obeys_saved_logo_sizes(self):
        self.test_runtime_wrapper_and_all_generated_presets()
        bindir = self.home / '.local/bin'
        env = os.environ | {'HOME': str(self.home), 'PATH': str(bindir) + ':' + os.environ['PATH'], 'KITTY_WINDOW_ID': '1', 'TERM': 'xterm-kitty', 'COLUMNS': '180', 'LINES': '50'}
        config = self.home / '.config/termfetch-studio/config.ini'
        for size, expected in [('small', (20, 10)), ('medium', (36, 18)), ('large', (52, 26))]:
            config.write_text(config.read_text().split('logo_size=')[0] + 'logo_size=' + size + '\n')
            result = subprocess.run(['bash', str(bindir / 'tfs-fastfetch')], env=env, capture_output=True, text=True, timeout=5)
            self.assertEqual(result.returncode, 0, result.stderr)
            logo = json.loads(result.stdout)['logo']
            self.assertEqual((logo['width'], logo['height']), expected)

    def test_small_menu_frames_and_selection_stay_visible(self):
        for columns, rows in [(80, 24), (60, 16), (40, 12)]:
            session = self.terminal('preview_image_fixture() { printf "Preview %s\\n" {1..8}; }; options=(); for i in {0..17}; do options+=("Image $i"); done; select_option "Images" "${options[@]}" preview_image_fixture; exit $?', columns, rows)
            self.assertIn('TermFetch Studio', session.text)
            self.assertIn('Image 0', session.text)
            for _ in range(17):
                session.send(b'\x1b[B')
            self.assertIn('Image 17', session.text)
            self.assertIn('TermFetch Studio', session.text)
            self.assertEqual(len(session.screen.history.top), 0, 'Menu overflow scrolled the terminal')
            self.assertEqual(session.finish(b'\r'), 17)
            self.assertIn(b'\x1b[?1049l', session.raw)

    def test_theme_columns_are_anchored_and_narrow_mode_is_single_column(self):
        body = 'select_option_2col "Themes" "🦇 Dracula" "❄️ Nord ✓" "🌸 Sakura" "⚡ Pokemon"; exit $?'
        session = self.terminal(body)
        positions = {}
        for row in range(session.rows):
            for col, char in session.screen.buffer[row].items():
                if char.data in ('S', 'P') and 'Themes' not in session.screen.display[row]:
                    tail = ''.join(session.screen.buffer[row][n].data for n in range(col, min(col + 7, session.columns)))
                    if tail.startswith(('Sakura', 'Pokemon')): positions[tail.split()[0]] = col
        self.assertEqual(sorted(positions.values()), [47, 47])
        session.send(b'\x1b[C')
        self.assertIn('Selected 3/4', session.text)
        self.assertEqual(session.finish(b'\r'), 2)
        session = self.terminal(body, 40, 12)
        session.send(b'\x1b[B')
        self.assertIn('Selected 2/4', session.text)
        self.assertEqual(session.finish(b'\r'), 1)

    def test_compact_header_fixed_icon_columns_and_responsive_preview(self):
        body = 'preview_fixture() { printf "RIGHT_MARKER\\nsecond line"; }; select_option "Choices [Old navigation hint]" "⚡ Short" "❄️ Longer" "Plain" preview_fixture; exit $?'
        session = self.terminal(body, 120, 24)
        self.assertNotIn('Old navigation', session.text)
        self.assertNotIn('Professional', session.text)
        self.assertIn('Enter Apply', session.text)
        self.assertIn('Short', session.screen.display[4])
        self.assertIn('Longer', session.screen.display[5])
        for row, initial in [(4, 'S'), (5, 'L'), (6, 'P')]:
            self.assertEqual(session.screen.buffer[row][7].data, initial)
        self.assertGreater(session.screen.display[4].index('RIGHT_MARKER'), 60)
        session.resize(80, 24); session.read(.5)
        marker_row = next(i for i, v in enumerate(session.screen.display) if 'RIGHT_MARKER' in v)
        self.assertGreater(marker_row, 7)
        session.resize(40, 12); session.read(.5)
        self.assertNotIn('RIGHT_MARKER', session.text)
        self.assertEqual(session.finish(), 255)

    def test_logo_size_choice_persists_and_scales_within_bounds(self):
        config = self.home / 'config'; config.mkdir()
        (config / 'config.ini').write_text('[fastfetch]\nlogo_size=medium\n')
        png(self.home / 'square.png', 400, 400)
        values = []
        for size in ['small', 'medium', 'large']:
            result = self.bash('COLUMNS=180; LINES=50; set_config_value fastfetch logo_size ' + size + '; configured_logo_dimensions "$HOME/square.png" 14')
            self.assertEqual(result.returncode, 0, result.stderr)
            values.append(tuple(map(int, result.stdout.split())))
            self.assertIn('logo_size=' + size, (config / 'config.ini').read_text())
        self.assertLess(values[0][0], values[1][0]); self.assertLess(values[1][0], values[2][0])
        self.assertTrue(all(w <= 60 and h <= 46 for w, h in values))
        for w, h in values: self.assertLess(abs(w / (h * 2) - 1), .06)
        # Exercise actual size submenu through a PTY and reopen its persisted config.
        (self.home / 'fastfetch.sh').write_text(generated('FASTFETCHLIB'))
        result = self.bash('source "$HOME/fastfetch.sh"; set_config_value fastfetch preset custom; set_config_value fastfetch logo_type auto; set_config_value colors theme nord; set_config_value custom modules os,kernel; apply_logo_config >/dev/null')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('Custom Preset', (config / 'fastfetch.jsonc').read_text())
        self.assertIn('modules=os,kernel', (config / 'config.ini').read_text())
        session = self.terminal('CONFIG_DIR="$HOME/config"; CONFIG_FILE="$CONFIG_DIR/config.ini"; source "$HOME/fastfetch.sh"; apply_logo_config() { :; }; logo_size_menu; get_config_value fastfetch logo_size > "$HOME/chosen"')
        session.send(b'\x1b[B'); session.send(b'\x1b[B'); session.finish(b'\r')
        self.assertEqual((self.home / 'chosen').read_text().strip(), 'large')
        self.assertEqual(self.bash('get_config_value fastfetch logo_size').stdout.strip(), 'large')

    def test_slow_preview_keeps_previous_frame_visible_until_replacement_ready(self):
        session = self.terminal('preview_slow() { if (($1 == 1)); then sleep .6; fi; printf "READY_%s" "$1"; }; select_option "Preview" A B preview_slow; exit $?')
        self.assertIn('READY_0', session.text)
        before = len(session.raw)
        session.send(b'\x1b[B')
        session.read(.15)
        self.assertIn('READY_0', session.text)
        self.assertNotIn(b'\x1b[2J', session.raw[before:], 'Visible screen cleared before slow preview finished')
        session.read(.7)
        self.assertIn('READY_1', session.text)
        self.assertEqual(session.finish(), 255)
        self.assertFalse(list(self.home.glob('tfs-frame.*')) + list(self.home.glob('tfs-menu.*')))

    def test_real_application_keeps_one_screen_session_across_submenus(self):
        lib = self.home / '.local/share/termfetch-studio/lib'; lib.mkdir(parents=True)
        for marker, name in [('CORELIB', 'core.sh'), ('FASTFETCHLIB', 'fastfetch.sh')]: (lib / name).write_text(generated(marker))
        app = self.home / 'app'; app.write_text(generated('MAINSCRIPT'))
        bindir = self.home / '.local/bin'; bindir.mkdir(parents=True)
        fake = bindir / 'fastfetch'; fake.write_text('#!/bin/bash\necho called >> "$HOME/fetch-calls"\necho EXIT_PREVIEW\n'); fake.chmod(0o755)
        config = self.home / '.config/termfetch-studio'; config.mkdir(parents=True)
        (config / 'config.ini').write_text('[fastfetch]\npreset=minimal\nlogo_type=none\nuse_theme_colors=true\n[colors]\ntheme=nord\n[general]\nfirst_run=false\n')
        (config / 'fastfetch.jsonc').write_text('{}')
        session = self.terminal('export PATH="$HOME/.local/bin:$PATH"; printf "UNDERLYING_PREVIEW\\n"; exec bash "$HOME/app"', 120, 24)
        self.assertIn('Main Menu', session.text)
        self.assertEqual(session.raw.count(b'\x1b[?1049h'), 1)
        before=len(session.raw)
        session.send(b'\x1b[B'*3+b'\r');session.read(.4)
        self.assertIn('Icon & Text', session.text)
        session.send(b'q');session.read(.3)
        self.assertIn('Main Menu', session.text)
        session.send(b'\x1b[B'*2+b'\r');session.read(.3)
        self.assertIn('Logo Settings', session.text)
        self.assertNotIn(b'\x1b[?1049', session.raw[before:])
        self.assertFalse((self.home / 'fetch-calls').exists())
        session.send(b'q');session.read(.3)
        self.assertEqual(session.finish(), 0)
        self.assertEqual(session.raw.count(b'\x1b[?1049l'), 1)
        self.assertEqual(session.raw.count(b'\x1b[?1049h'), 1)
        self.assertIn('EXIT_PREVIEW', session.text)
        self.assertEqual((self.home / 'fetch-calls').read_text().splitlines(), ['called'])
        self.assertEqual(termios.tcgetattr(session.slave), session.initial_attributes)
        session = self.terminal('export PATH="$HOME/.local/bin:$PATH"; exec bash "$HOME/app"', 120, 24)
        session.read(.3)
        self.assertIn('Main Menu', session.text)
        os.kill(session.process.pid, signal.SIGTERM)
        self.assertEqual(session.process.wait(timeout=3), 143)
        session.read()
        self.assertEqual(session.raw.count(b'\x1b[?1049l'), 1)
        self.assertEqual(termios.tcgetattr(session.slave), session.initial_attributes)

    def test_navigation_never_clears_screen_or_reenters_alternate_buffer(self):
        for columns, rows in [(120, 24), (80, 24), (40, 12)]:
            session = self.terminal('preview_fixture() { printf "Preview %s" "$1"; }; select_option "Stable" A B C preview_fixture; exit $?', columns, rows)
            before = len(session.raw)
            session.send(b'\x1b[B'); session.send(b'\x1b[B')
            delta = session.raw[before:]
            self.assertNotIn(b'\x1b[2J', delta)
            self.assertNotIn(b'\x1b[?1049', delta)
            self.assertIn('Selected 3/3', session.text)
            self.assertEqual(session.finish(), 255)

    def test_kitty_upload_is_hidden_until_placement_and_deletion_is_scoped(self):
        for name in ['one', 'two']: png(self.home / (name + '.png'), 10, 10)
        body = 'check_terminal_image_support() { echo kitty; }; PREVIEW_PRESET_IMAGES=("$HOME/one.png" "$HOME/two.png"); select_option Images One Two preview_image_logo; exit $?'
        session = self.terminal(body, 120, 24)
        session.read(.5)
        self.assertIn(b'\x1b_Ga=t,i=', session.raw)
        self.assertNotIn(b'\x1b_Ga=T,', session.raw)
        self.assertLess(session.raw.index(b'\x1b[2J'), session.raw.index(b'\x1b_Ga=t,i='))
        first_id = re.search(rb'\x1b_Ga=t,i=(\d+)', session.raw).group(1)
        before = len(session.raw)
        session.send(b'\x1b[B'); session.read(.5)
        delta = session.raw[before:]
        second_id = re.search(rb'\x1b_Ga=t,i=(\d+)', delta).group(1)
        self.assertNotEqual(first_id, second_id)
        put = delta.index(b'\x1b_Ga=p,i=' + second_id)
        delete = delta.index(b'\x1b_Ga=d,d=I,i=' + first_id)
        self.assertLess(put, delete)
        self.assertLess(delta.index(b'\x1b_Ga=t,i='), delta.index(b'\x1b[?2026h'))
        self.assertNotIn(b'\x1b[2J', delta)
        self.assertNotIn(b'a=d,d=A', delta)
        self.assertEqual(session.finish(), 255)
        self.assertIn(b'\x1b_Ga=d,d=I,i=' + second_id, session.raw)
        self.assertFalse(list(self.home.glob('tfs-menu.*')))

    def test_symbol_preview_clears_only_its_rectangle_when_shrinking(self):
        for name in ['one', 'two']: png(self.home / (name + '.png'), 10, 10)
        body = 'check_terminal_image_support() { echo none; }; chafa() { if [[ "${!#}" == *one.png ]]; then printf "LONG_SYMBOL_PREVIEW"; else printf "X"; fi; }; PREVIEW_PRESET_IMAGES=("$HOME/one.png" "$HOME/two.png"); select_option Images One Two preview_image_logo; exit $?'
        session = self.terminal(body, 120, 24)
        self.assertIn('LONG_SYMBOL_PREVIEW', session.text)
        before=len(session.raw); session.send(b'\x1b[B')
        self.assertNotIn('LONG_SYMBOL_PREVIEW', session.text)
        self.assertIn('Selected 2/2', session.text)
        self.assertNotIn(b'\x1b[2J', session.raw[before:])
        self.assertEqual(session.finish(), 255)

    def color_editor(self, columns=120, rows=24):
        config = self.home / 'config'; config.mkdir(exist_ok=True)
        if not (config / 'config.ini').exists():
            (config / 'config.ini').write_text('[fastfetch]\npreset=minimal\nlogo_type=none\nuse_theme_colors=true\nkey_color=theme\nvalue_color=theme\n[colors]\ntheme=nord\n')
        (self.home / 'fastfetch.sh').write_text(generated('FASTFETCHLIB'))
        session=self.terminal('CONFIG_DIR="$HOME/config"; CONFIG_FILE="$CONFIG_DIR/config.ini"; source "$HOME/fastfetch.sh"; color_settings_menu', columns, rows)
        deadline=time.monotonic()+3
        while 'Icons:' not in session.text and time.monotonic()<deadline:session.read(.05)
        self.assertIn('Icons:',session.text)
        return session

    def test_live_color_editor_previews_without_writes_then_saves_and_resets(self):
        session = self.color_editor()
        config = self.home / 'config/config.ini'
        original = config.read_bytes()
        self.assertIn('Linux', session.text)
        self.assertNotIn('keyColor', session.text)
        start = len(session.raw)
        session.send(b'\x1b[C\x1b[C')
        self.assertIn('Icons: Red', session.text)
        self.assertIn(b'\x1b[31m', session.raw[start:])
        self.assertEqual(config.read_bytes(), original)
        session.send(b'\x1b[B\x1b[C\x1b[C\x1b[C')
        session.read(.4)
        self.assertIn('Text: Green', session.text)
        self.assertEqual(config.read_bytes(), original)
        session.send(b'\x1b[B'*7)
        self.assertEqual(session.finish(b'\r'), 0)
        self.assertIn('key_color=31', config.read_text())
        self.assertIn('value_color=32', config.read_text())
        self.assertIn('use_theme_colors=false', config.read_text())
        self.assertIn('"32"', (self.home / 'config/fastfetch.jsonc').read_text())
        session = self.color_editor()
        session.send(b'\x1b[B'*7+b'\r')
        self.assertIn('Icons: Theme', session.text)
        self.assertIn('Text: Theme', session.text)
        session.send(b'\x1b[B')
        self.assertEqual(session.finish(b'\r'), 0)
        self.assertIn('use_theme_colors=true', config.read_text())
        self.assertIn('key_color=theme', config.read_text())

    def test_color_editor_cancel_and_small_window_preview(self):
        session = self.color_editor(40, 12)
        config = self.home / 'config/config.ini'; original = config.read_bytes()
        self.assertIn('Linux', session.text)
        self.assertIn('Q:Cancel', session.text)
        session.send(b'\x1b[C\x1b[C')
        self.assertEqual(session.finish(), 0)
        self.assertEqual(config.read_bytes(), original)
        self.assertFalse((self.home / 'config/fastfetch.jsonc').exists())

    def test_advanced_menu_has_no_duplicate_color_entry(self):
        session=self.terminal('settings_menu')
        self.assertNotIn('Color Settings',session.text)
        self.assertIn('Items 1-4/4',session.text)
        session.send(b'\x1b[B'*3)
        self.assertEqual(session.finish(b'\r'),0)

    def test_rgb_palette_selection_is_live_and_persisted(self):
        session=self.color_editor()
        session.send(b'\x1b[D');session.read(.3)
        self.assertIn('Icons: Silver',session.text)
        self.assertIn(b'\x1b[38;2;192;192;192m',session.raw)
        session.send(b'\x1b[B'*8)
        self.assertEqual(session.finish(b'\r'),0)
        self.assertIn('key_color=#c0c0c0',(self.home/'config/config.ini').read_text())
        self.assertIn('"keyColor": "#c0c0c0"',(self.home/'config/fastfetch.jsonc').read_text())

    def test_inherit_sample_matches_selected_preset_modules(self):
        for preset,label,module in [('minimal','Time','uptime'),('gaming','Hardware','cpu')]:
            config=self.home/'config';config.mkdir(exist_ok=True)
            (config/'config.ini').write_text('[fastfetch]\npreset='+preset+'\nlogo_type=none\nuse_theme_colors=true\n[colors]\ntheme=nord\n')
            session=self.color_editor()
            session.send(b'\x1b[B'*8)
            self.assertEqual(session.finish(b'\r'),0)
            raw=(config/'fastfetch.jsonc').read_text()
            doc=json.loads(re.sub(r'^\s*//.*$', '',raw,flags=re.M))
            color=next(m['keyColor'] for m in doc['modules'] if isinstance(m,dict) and m.get('type')==module)
            line=next(y for y,text in enumerate(session.screen.display) if '● '+label in text)
            col=session.screen.display[line].index('● '+label)
            self.assertEqual(session.screen.buffer[line][col].fg,color.lstrip('#'),(preset,label,color))

    def test_group_colors_preserve_layout_and_override_only_target_modules(self):
        self.test_runtime_wrapper_and_all_generated_presets()
        bindir=self.home / '.local/bin'
        env=os.environ | {'HOME':str(self.home), 'PATH':str(bindir)+':'+os.environ['PATH']}
        code='CONFIG_DIR="$HOME/.config/termfetch-studio"; INSTALL_DIR="$HOME/.local/share/termfetch-studio"; source "$INSTALL_DIR/lib/core.sh"; source "$INSTALL_DIR/lib/fastfetch.sh"; '
        sets={'os':{'os','kernel','packages','shell'},'desktop':{'wm','de','theme','terminal','terminalfont','cursor','icons'},'hardware':{'host','cpu','gpu','memory','swap','disk','display','battery'},'network':{'localip','publicip','wifi'},'time':{'datetime','uptime'}}
        colors=['#ff8c00','#40e0d0','#8a2be2','#ff69b4','#ffd700']
        setup='; '.join('set_config_value group_colors '+g+" '"+c+"'" for g,c in zip(sets,colors))+'; '
        for preset in ['full','minimal','focused','developer','gaming','custom']:
            result=subprocess.run(['bash','-c',code+setup+'create_'+preset+'_config none "" nord; tfs-fastfetch'],env=env,capture_output=True,text=True,timeout=10)
            self.assertEqual(result.returncode,0,result.stderr)
            doc=json.loads(result.stdout);seen=0
            for module in doc['modules']:
                if not isinstance(module,dict):continue
                for group,color in zip(sets,colors):
                    if module.get('type') in sets[group]:
                        self.assertEqual(module.get('keyColor'),color,(preset,module))
                        self.assertEqual(module.get('outputColor'),color,(preset,module));seen+=1
            self.assertGreater(seen,0)
        # Group overrides and extended RGB can be edited without touching files.
        session=self.color_editor()
        config=self.home/'config/config.ini';old=config.read_bytes()
        session.send(b'\x1b[B'*2+b'\x1b[C'*3)
        self.assertIn('OS group: Red',session.text)
        self.assertEqual(config.read_bytes(),old)
        session.send(b'\x1b[B'*6)
        self.assertEqual(session.finish(b'\r'),0)
        self.assertIn('os=31',config.read_text())
        self.assertIn('"keyColor": "31"',(self.home/'config/fastfetch.jsonc').read_text())

    def test_independent_theme_and_default_colors_generate_valid_presets(self):
        self.test_runtime_wrapper_and_all_generated_presets()
        bindir = self.home / '.local/bin'
        env = os.environ | {'HOME': str(self.home), 'PATH': str(bindir) + ':' + os.environ['PATH']}
        code = 'CONFIG_DIR="$HOME/.config/termfetch-studio"; INSTALL_DIR="$HOME/.local/share/termfetch-studio"; source "$INSTALL_DIR/lib/core.sh"; source "$INSTALL_DIR/lib/fastfetch.sh"; '
        for preset in ['full', 'minimal', 'focused', 'developer', 'gaming', 'custom']:
            for icon, text in [('theme', '31'), ('32', 'theme'), ('default', 'default')]:
                result = subprocess.run(['bash', '-c', code + 'set_config_value fastfetch use_theme_colors false; set_config_value fastfetch key_color ' + icon + '; set_config_value fastfetch value_color ' + text + '; create_' + preset + '_config none "" nord; tfs-fastfetch'], env=env, capture_output=True, text=True, timeout=5)
                self.assertEqual(result.returncode, 0, result.stderr)
                doc = json.loads(result.stdout)
                for mod in doc['modules']:
                    if isinstance(mod, dict):
                        self.assertNotEqual(mod.get('keyColor'), 'theme')
                        self.assertNotEqual(mod.get('outputColor'), 'theme')
                        if 'outputColor' in mod and text == 'default': self.assertEqual(mod['outputColor'], '39')

    def test_resize_redraws_without_keypress(self):
        session = self.terminal('options=(); for i in {0..17}; do options+=("Item $i"); done; select_option "Resize" "${options[@]}"; exit $?')
        session.resize(40, 12)
        data = session.read(0.5)
        self.assertIn('TermFetch Studio', session.text)
        self.assertIn('Item 0', session.text)
        anchors = re.findall(rb'\x1b\[(\d+);(\d+)H', data)
        self.assertTrue(anchors)
        self.assertTrue(all(int(y) <= 12 and int(x) <= 40 for y, x in anchors))
        self.assertEqual(session.finish(), 255)

    def test_eof_and_tiny_window_do_not_select_invisible_items(self):
        result = self.bash('select_option "EOF" A B; exit $?')
        self.assertEqual(result.returncode, 255)
        session = self.terminal('select_option "Tiny" A B; exit $?', 15, 6)
        self.assertIn('Resize', session.text)
        session.send(b'\r'); self.assertIsNone(session.process.poll())
        self.assertEqual(session.finish(b'\x04'), 255)

    def test_menu_restores_caller_traps_and_terminal_flags(self):
        body = 'trap ":" TERM; before=$(trap -p TERM); stty -echo -icanon; saved=$(stty -g); select_option "Cleanup" A B; result=$?; test "$(stty -g)" = "$saved" && test "$(trap -p TERM)" = "$before" || exit 99; exit "$result"'
        session = self.terminal(body)
        self.assertEqual(session.finish(), 255)
        self.assertIn(b'\x1b[?25h', session.raw)
        for sig, expected in [(signal.SIGINT, 130), (signal.SIGTERM, 143)]:
            session = self.terminal('select_option "Signal" A B; exit $?')
            os.killpg(session.process.pid, sig)
            self.assertEqual(session.process.wait(timeout=3), expected)
            session.read()
            self.assertIn(b'\x1b[?1049l', session.raw)
            self.assertEqual(termios.tcgetattr(session.slave), session.initial_attributes)


if __name__ == '__main__':
    unittest.main(verbosity=2)
